use crate::codecs::bfs::Bfs;
use crate::codecs::codec::Codec;
use crate::hubs::unit::HubUnit;
use ffmpeg_next as ffmpeg;
use ffmpeg_next::Rescale;
use crate::codecs::h264::format::NALUType;

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
    pub fn make_packet (&mut self, unit: &HubUnit) -> Option<ffmpeg::packet::Packet> {
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

        let Some(mut pkt) = self.bfs.make_packet(&unit) else {
            return None;
        };

        let input_time_base = ffmpeg::Rational::new(1, unit.timebase as i32);
        let output_time_base = ffmpeg::Rational::new(1, self.codec.clock_rate() as i32);

        pkt.set_stream(self.idx);
        pkt.set_pts(Some(
            (self.pts as i64).rescale(input_time_base, output_time_base),
        ));
        pkt.set_dts(Some(
            (self.dts as i64).rescale(input_time_base, output_time_base),
        ));
        pkt.set_duration((unit.duration as i64).rescale_with(
            input_time_base,
            output_time_base,
            ffmpeg::mathematics::Rounding::NearInfinity,
        ));
        if unit.frame_info.flag == 1 {
            pkt.set_flags(ffmpeg::packet::Flags::KEY);
        }

        Some(pkt)
    }
}
