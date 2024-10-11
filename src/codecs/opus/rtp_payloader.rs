use bytes::Bytes;
use webrtc::rtp::codecs::h264::H264Payloader;
use webrtc::rtp::codecs::opus::OpusPayloader;
use webrtc::rtp::packetizer::Payloader;

pub struct OpusRtpPayloader {
    payloader: OpusPayloader,
}

impl OpusRtpPayloader {
    pub fn new() -> Self {
        let payloader = OpusPayloader::default();
        OpusRtpPayloader { payloader }
    }
}

impl OpusRtpPayloader {
    pub fn payload(&mut self, mtu: usize, payload: &Bytes) -> anyhow::Result<Vec<Bytes>> {
        match self.payloader.payload(mtu, payload) {
            Ok(packets) => return Ok(packets),
            Err(err) => return Err(err.into()),
        }
    }
}
