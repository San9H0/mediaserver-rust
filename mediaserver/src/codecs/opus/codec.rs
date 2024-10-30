use crate::utils::types::types;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct OpusCodec;

impl OpusCodec {
    pub fn new() -> Self {
        Self
    }

    pub fn kind(&self) -> types::MediaKind {
        types::MediaKind::Audio
    }

    pub fn mime_type(&self) -> &'static str {
        "audio/opus"
    }

    pub fn clock_rate(&self) -> u32 {
        48000
    }

    pub fn channels(&self) -> u16 {
        2
    }

    pub fn samples(&self) -> u32 {
        960
    }

    pub fn rtp_codec_capability(&self) -> RTCRtpCodecCapability {
        RTCRtpCodecCapability {
            mime_type: self.mime_type().to_string(),
            clock_rate: self.clock_rate(),
            channels: self.channels(),
            sdp_fmtp_line: "minptime=10;useinbandfec=1".to_string(),
            rtcp_feedback: vec![],
        }
    }
}
