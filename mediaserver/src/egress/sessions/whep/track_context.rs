use crate::codecs::codec::Codec;
use crate::codecs::rtp_packetizer::RtpPacketizer;
use crate::codecs::rtp_payloader::RtpPayloader;
use crate::hubs::unit::HubUnit;
use webrtc::rtp::packet::Packet;

pub struct TrackContext {
    codec: Codec,
    rtp_packetizer: RtpPacketizer,
}

impl TrackContext {
    pub fn new(codec: &Codec) -> Self {
        let payloader = RtpPayloader::new(codec, codec.mime_type()).unwrap();
        let rtp_packetizer = RtpPacketizer::new(payloader, codec, codec.clock_rate());

        TrackContext {
            codec: codec.clone(),
            rtp_packetizer,
        }
    }
    pub fn make_packet(&mut self, unit: &HubUnit) -> anyhow::Result<Vec<Packet>> {
        self.rtp_packetizer.packetize(&unit.payload, unit.duration)
    }
}
