use crate::codecs::bfs::Bfs;
use crate::codecs::codec::Codec;
use crate::hubs::unit::HubUnit;
use crate::utils::{self, rescale};

pub struct TrackContext {
    codec: Codec,
    bfs: Bfs,
    idx: usize,
    started: bool,

    base_ts: u32,
    pts: u32,
    dts: u32,
}

impl TrackContext {
    pub fn new(idx: usize, codec: &Codec) -> Self {
        TrackContext {
            codec: codec.clone(),
            bfs: Bfs::new(codec.mime_type()).unwrap(),
            idx,
            started: false,
            base_ts: 0,
            pts: 0,
            dts: 0,
        }
    }

    pub fn track_id(&self) -> u32 {
        (self.idx + 1) as u32
    }

    pub fn make_packet(&mut self, unit: &HubUnit) -> Option<utils::packet::packet::Packet> {
        if !self.started {
            self.started = true;
            self.base_ts = unit.pts;
        }

        if self.base_ts < unit.pts {
            self.pts = unit.pts - self.base_ts;
        }
        if self.base_ts < unit.dts {
            self.dts = unit.dts - self.base_ts;
        }

        let Some(mut pkt) = self.bfs.make_packet2(&unit) else {
            return None;
        };

        let input_timebase = rescale::rescale::Rational::new(1, unit.timebase as i32);
        let output_timebase = rescale::rescale::Rational::new(1, self.codec.clock_rate() as i32);

        let pts = rescale::rescale::rescale_with_rounding(
            self.pts as i64,
            &input_timebase,
            &output_timebase,
        );
        let dts = rescale::rescale::rescale_with_rounding(
            self.dts as i64,
            &input_timebase,
            &output_timebase,
        );
        let duration = rescale::rescale::rescale_with_rounding(
            unit.duration as i64,
            &input_timebase,
            &output_timebase,
        );

        pkt.set_stream(self.idx);
        pkt.set_pts(pts);
        pkt.set_dts(dts);
        pkt.set_time_base(output_timebase);
        pkt.set_duration(duration);
        if unit.frame_info.flag == 1 {
            pkt.set_flags(1);
        }

        Some(pkt)
    }
}
