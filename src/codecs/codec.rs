use crate::codecs::h264::codec::H264Codec;
use crate::codecs::opus::codec::OpusCodec;
use anyhow::anyhow;

#[derive(Debug, Clone, Copy)]
pub enum Codec {
    Opus(OpusCodec),
    H264(H264Codec),
}

impl Codec {
    pub fn new(mime_type: &str) -> anyhow::Result<Codec> {
        match mime_type.to_lowercase().as_str() {
            "audio/opus" => Ok(Codec::Opus(OpusCodec)),
            "video/h264" => Ok(Codec::H264(H264Codec)),
            _ => Err(anyhow!("Unsupported codec: {}", mime_type)),
        }
    }

    pub fn kind(&self) -> &'static str {
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
}
