use crate::codecs::h264::config::Config;
use crate::utils::types::types;
use std::hash::Hash;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;

#[derive(Debug, Clone)]
pub struct H264Codec {
    config: Config,
}

impl H264Codec {
    pub fn new(config: Config) -> H264Codec {
        H264Codec { config }
    }

    pub fn codec_string(&self) -> String {
        let sps = self.config.sps.payload.clone().to_vec();

        // SPS에서 필요한 값 추출
        let profile_idc = sps[1];
        let constraint_flags = sps[2];
        let level_idc = sps[3];

        // constraint_flags의 상위 6비트를 사용
        let constraint_bits = (constraint_flags & 0b11111100) >> 2;

        // 문자열 포맷으로 변환
        format!(
            "avc1.{:02X}{:02X}{:02X}",
            profile_idc, constraint_bits, level_idc
        )
    }

    pub fn sps(&self) -> Option<Vec<u8>> {
        Some(self.config.sps.payload.clone().to_vec())
    }

    pub fn pps(&self) -> Option<Vec<u8>> {
        Some(self.config.pps.payload.clone().to_vec())
    }

    pub fn width(&self) -> u32 {
        self.config.sps.width()
    }

    pub fn height(&self) -> u32 {
        self.config.sps.height()
    }

    pub fn kind(&self) -> types::MediaKind {
        types::MediaKind::Video
    }
    pub fn mime_type(&self) -> &'static str {
        "video/h264"
    }

    pub fn clock_rate(&self) -> u32 {
        90000
    }

    pub fn channels(&self) -> u16 {
        0
    }

    pub fn samples(&self) -> u32 {
        3000
    }

    pub fn rtp_codec_capability(&self) -> RTCRtpCodecCapability {
        RTCRtpCodecCapability {
            mime_type: self.mime_type().to_string(),
            clock_rate: self.clock_rate(),
            channels: self.channels(),
            sdp_fmtp_line: "level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=42001f"
                .to_string(),
            rtcp_feedback: vec![],
        }
    }
}

impl PartialEq for H264Codec {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind()
            && self.mime_type() == other.mime_type()
            && self.clock_rate() == other.clock_rate()
            && self.channels() == other.channels()
            && self.samples() == other.samples()
            && self.config.width() == other.config.width()
            && self.config.height() == other.config.height()
    }
}

impl Eq for H264Codec {}

impl Hash for H264Codec {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.kind().hash(state);
        self.mime_type().hash(state);
        self.clock_rate().hash(state);
        self.channels().hash(state);
        self.samples().hash(state);
        self.config.width().hash(state);
        self.config.height().hash(state);
    }
}
