use crate::codecs::codec::Codec;
use crate::codecs::h264::format::NALUType;
use bytes::Bytes;
use webrtc::rtp::codecs::h264::H264Payloader;
use webrtc::rtp::packetizer::Payloader;

pub struct H264RtpPayloader {
    codec: Codec,
    payloader: H264Payloader,
}

impl H264RtpPayloader {
    pub fn new(codec: &Codec) -> Self {
        let payloader = H264Payloader::default();
        H264RtpPayloader {
            codec: codec.clone(),
            payloader,
        }
    }
}

impl H264RtpPayloader {
    pub fn payload(&mut self, mtu: usize, payload: &Bytes) -> anyhow::Result<Vec<Bytes>> {
        let nalu_type = NALUType::from_byte(payload[0]);
        if nalu_type == NALUType::SPS || nalu_type == NALUType::PPS {
            return Ok(vec![]);
        }

        let mut result = vec![];
        if nalu_type == NALUType::IDR {
            if let Some(sps) = self.codec.sps() {
                result.push(Bytes::from(sps.clone()));
            }
            if let Some(pps) = self.codec.pps() {
                result.push(Bytes::from(pps.clone()));
            }
        }

        match self.payloader.payload(mtu, payload) {
            Ok(packets) => {
                result.extend(packets);
                return Ok(result);
            }
            Err(err) => Err(err.into()),
        }
    }
}
