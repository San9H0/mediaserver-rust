use crate::codecs::bfs::Bfs;
use crate::codecs::codec::Codec;
use crate::codecs::h264::format::NALUType;
use crate::codecs::rtp_packetizer::RtpPacketizer;
use crate::codecs::rtp_payloader::RtpPayloader;
use crate::hubs::unit::HubUnit;
use ffmpeg_next as ffmpeg;
use ffmpeg_next::Rescale;
use webrtc::rtp::packet::Packet;

pub struct TrackContext {
    codec: Codec,
    rtp_packetizer: RtpPacketizer,
}

impl TrackContext {
    pub fn new(codec: &Codec) -> Self {
        let payloader = RtpPayloader::new(codec.mime_type()).unwrap();
        let rtp_packetizer = RtpPacketizer::new(payloader, codec.clock_rate());

        TrackContext {
            codec: codec.clone(),
            rtp_packetizer,
        }
    }
    pub fn make_packet(&mut self, unit: &HubUnit) -> anyhow::Result<Vec<Packet>> {
        self.rtp_packetizer.packetize(&unit.payload, unit.duration)
    }
}
