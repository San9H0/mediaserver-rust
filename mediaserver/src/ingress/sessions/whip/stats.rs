use bytes::Bytes;
use std::default::Default;
use std::ops::Sub;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::sync::RwLock;
use webrtc::rtcp::receiver_report::ReceiverReport;
use webrtc::rtcp::reception_report::ReceptionReport;
use webrtc::rtp;
use webrtc::util::MarshalSize;

const MAX_SEQ_NO: u32 = 65535;

struct ReadStats {
    clock_rate: u32,
    total_bytes: u32,
    packet_count: u32,
    base_seq_no: u16,
    max_seq_no: u16,
    cycle: u32,
    last_transit: u32,
    jitter: f64,
}

impl ReadStats {
    fn new(clock_rate: u32) -> Self {
        ReadStats {
            clock_rate,
            total_bytes: 0,
            packet_count: 0,
            base_seq_no: 0,
            max_seq_no: 0,
            cycle: 0,
            last_transit: 0,
            jitter: 0.0,
        }
    }

    async fn read_stat(&mut self, packet: &rtp::packet::Packet, diff_milli: u32) {
        let sn = packet.header.sequence_number;
        if self.packet_count == 0 {
            self.base_seq_no = sn;
            self.max_seq_no = sn;
        } else if (sn.wrapping_sub(self.max_seq_no)) & 0x8000 == 0 {
            if sn < self.max_seq_no {
                self.cycle += MAX_SEQ_NO + 1;
            }
            self.max_seq_no = sn;
        } else if (sn.wrapping_sub(self.max_seq_no)) & 0x8000 > 0 {
            // todo 재전송
        }
        self.packet_count += 1;
        self.total_bytes = self
            .total_bytes
            .wrapping_add((packet.payload.len() + packet.header.marshal_size()) as u32);

        let arrival = diff_milli * (self.clock_rate / 1_000);
        let transit = arrival.wrapping_sub(packet.header.timestamp);

        if self.last_transit != 0 {
            let mut d = transit as i32 - self.last_transit as i32;
            if d < 0 {
                d = -d;
            }
            self.jitter += (d as f64 - self.jitter) / 16.0;
        }

        self.last_transit = transit;
    }

    async fn get_receiver_report_params(&self) -> (f64, u16, u32, u32) {
        let packet_expect = self.cycle | self.max_seq_no as u32;
        let packet_lost = packet_expect - self.packet_count;

        (self.jitter, self.max_seq_no, packet_expect, packet_lost)
    }
}

pub struct Stats {
    start_time: chrono::DateTime<chrono::Local>,

    read_stat: RwLock<ReadStats>,

    prev_expected: std::sync::atomic::AtomicU32,
    prev_packet_lost: std::sync::atomic::AtomicU32,
    last_sr_ntp_time: std::sync::atomic::AtomicU64,
    last_sr_time: std::sync::atomic::AtomicI64,
}

impl Stats {
    pub fn new(clock_rate: u32) -> Arc<Self> {
        Arc::new(Stats {
            start_time: chrono::Local::now(),
            read_stat: RwLock::new(ReadStats::new(clock_rate)),
            prev_expected: Default::default(),
            prev_packet_lost: Default::default(),
            last_sr_ntp_time: Default::default(),
            last_sr_time: Default::default(),
        })
    }

    pub async fn calc_rtp_stats(self: &Arc<Self>, packet: &rtp::packet::Packet) {
        let now = chrono::Local::now();
        let diff_milli = now.sub(self.start_time).num_milliseconds() as u32;

        let mut read_stat = self.read_stat.write().await;
        read_stat.read_stat(packet, diff_milli).await;
    }

    pub async fn make_receiver_report(self: &Arc<Self>, ssrc: u32) -> ReceiverReport {
        let (jitter, max_seq_no, packet_expect, packet_lost) = {
            let read_stats = self.read_stat.read().await;
            read_stats.get_receiver_report_params().await
        };

        let expected_interval = packet_expect - self.prev_expected.load(Ordering::Acquire);
        let lost_interval = packet_lost - self.prev_packet_lost.load(Ordering::Acquire);
        let lost_rate = lost_interval as f32 / expected_interval as f32;
        let fraction_lost = (lost_rate * 256.0) as u8;
        let last_sender_report = (self.last_sr_ntp_time.load(Ordering::Acquire) >> 16) as u32;

        let mut dlsr: u32 = 0;
        let last_sr_time = self.last_sr_time.load(Ordering::Acquire);
        if last_sr_time != 0 {
            let nano = chrono::Local::now().timestamp_nanos_opt().unwrap();
            let delay_ms = ((nano - last_sr_time) / 1_000_000) as u32;
            dlsr = (delay_ms / 1000) << 16;
            dlsr |= (delay_ms % 1000) * 65536 / 1000;
        }

        self.prev_expected.store(packet_expect, Ordering::Release);
        self.prev_packet_lost.store(packet_lost, Ordering::Release);

        ReceiverReport {
            ssrc,
            reports: vec![ReceptionReport {
                ssrc,
                fraction_lost,
                total_lost: packet_lost,
                last_sequence_number: max_seq_no as u32,
                jitter: jitter as u32,
                last_sender_report,
                delay: dlsr,
            }],
            profile_extensions: Bytes::new(),
        }
    }
}
