use crate::codecs::h264::codec::H264Codec;
use crate::codecs::opus::codec::OpusCodec;
use crate::utils::types::types;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum Codec {
    Opus(OpusCodec),
    H264(H264Codec),
}

impl Codec {
    // pub fn new(mime_type: &str) -> anyhow::Result<Codec> {
    //     match mime_type.to_lowercase().as_str() {
    //         "audio/opus" => Ok(Codec::Opus(OpusCodec)),
    //         "video/h264" => Ok(Codec::H264(H264Codec)),
    //         _ => Err(anyhow!("Unsupported codec: {}", mime_type)),
    //     }
    // }

    pub fn kind(&self) -> types::MediaKind {
        match self {
            Codec::Opus(codec) => codec.kind(),
            Codec::H264(codec) => codec.kind(),
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            Codec::Opus(codec) => codec.mime_type(),
            Codec::H264(codec) => codec.mime_type(),
        }
    }

    pub fn clock_rate(&self) -> u32 {
        match self {
            Codec::Opus(codec) => codec.clock_rate(),
            Codec::H264(codec) => codec.clock_rate(),
        }
    }

    pub fn samples(&self) -> u32 {
        match self {
            Codec::Opus(codec) => codec.samples(),
            Codec::H264(codec) => codec.samples(),
        }
    }

    pub fn rtp_codec_capability(&self) -> RTCRtpCodecCapability {
        match self {
            Codec::Opus(codec) => codec.rtp_codec_capability(),
            Codec::H264(codec) => codec.rtp_codec_capability(),
        }
    }

    pub fn codec_string(&self) -> String {
        match self {
            Codec::H264(codec) => codec.codec_string(),
            Codec::Opus(codec) => codec.codec_string(),
            _ => "".to_string(),
        }
    }

    pub fn sps(&self) -> Option<Vec<u8>> {
        match self {
            Codec::H264(codec) => codec.sps(),
            _ => None,
        }
    }

    pub fn pps(&self) -> Option<Vec<u8>> {
        match self {
            Codec::H264(codec) => codec.pps(),
            _ => None,
        }
    }

    pub fn width(&self) -> u32 {
        match self {
            Codec::H264(codec) => codec.width(),
            _ => 0,
        }
    }

    pub fn height(&self) -> u32 {
        match self {
            Codec::H264(codec) => codec.height(),
            _ => 0,
        }
    }
}
