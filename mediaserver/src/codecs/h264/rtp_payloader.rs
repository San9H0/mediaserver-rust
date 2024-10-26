use crate::codecs::h264::format::NALUType;
use bytes::Bytes;
use webrtc::rtp::codecs::h264::H264Payloader;
use webrtc::rtp::packetizer::Payloader;

pub struct H264RtpPayloader {
    send_keyframe: bool,
    payloader: H264Payloader,
}

impl H264RtpPayloader {
    pub fn new() -> Self {
        let payloader = H264Payloader::default();
        H264RtpPayloader {
            send_keyframe: false,
            payloader,
        }
    }
}

impl H264RtpPayloader {
    pub fn payload(&mut self, mtu: usize, payload: &Bytes) -> anyhow::Result<Vec<Bytes>> {
        let nalu_type = NALUType::from_byte(payload[0]);
        if nalu_type == NALUType::SPS || nalu_type == NALUType::PPS {
            self.send_keyframe = true;
        } else if !self.send_keyframe {
            return Ok(vec![]);
        }
        match self.payloader.payload(mtu, payload) {
            Ok(packets) => Ok(packets),
            Err(err) => Err(err.into()),
        }
    }
}
