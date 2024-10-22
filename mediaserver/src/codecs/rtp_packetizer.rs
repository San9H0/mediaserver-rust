use crate::codecs::rtp_payloader::RtpPayloader;
use bytes::Bytes;
use webrtc::rtp::header::Header;
use webrtc::rtp::packet::Packet;

pub struct RtpPacketizer {
    mtu: usize,
    payloader: RtpPayloader,
    payload_type: u8,
    ssrc: u32,
    timestamp: u32,
    sequence: u16,
    clock_rate: u32,
}

impl RtpPacketizer {
    pub fn new(payloader: RtpPayloader, clock_rate: u32) -> Self {
        RtpPacketizer {
            mtu: 1200,
            payloader,
            payload_type: 0,
            ssrc: 0,
            timestamp: rand::random::<u32>(),
            sequence: rand::random::<u16>(),
            clock_rate,
        }
    }

    pub fn packetize(&mut self, payload: &Bytes, samples: u32) -> anyhow::Result<Vec<Packet>> {
        let payloads = self.payloader.payload(self.mtu - 12, payload)?;
        if payloads.len() == 0 {
            return Ok(vec![]);
        }
        let payloads_len = payloads.len();
        let mut packets = Vec::with_capacity(payloads_len);

        for (i, payload) in payloads.into_iter().enumerate() {
            packets.push(Packet {
                header: Header {
                    version: 2,
                    padding: false,
                    extension: false,
                    marker: i == payloads_len - 1,
                    payload_type: self.payload_type,
                    sequence_number: self.sequence,
                    timestamp: self.timestamp, //TODO: Figure out how to do timestamps
                    ssrc: self.ssrc,
                    ..Default::default()
                },
                payload,
            });
            self.sequence = self.sequence.wrapping_add(1);
        }

        self.timestamp = self.timestamp.wrapping_add(samples);

        Ok(packets)
    }
}
