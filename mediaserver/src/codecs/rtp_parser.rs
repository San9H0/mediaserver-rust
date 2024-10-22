use crate::codecs::codec::Codec;
use crate::codecs::h264::rtp_parser::H264RtpParser;
use crate::codecs::opus::rtp_parser::OpusRtpParser;
use crate::hubs::unit::FrameInfo;
use anyhow::anyhow;
use bytes::Bytes;
use std::future::Future;
use std::pin::Pin;

pub enum RtpParser {
    Opus(OpusRtpParser),
    H264(H264RtpParser),
}

type OnCodecCallback = Box<dyn Fn(Codec) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

impl RtpParser {
    pub fn new(mime_type: &str, callback: OnCodecCallback) -> anyhow::Result<RtpParser> {
        match mime_type.to_lowercase().as_str() {
            "audio/opus" => Ok(RtpParser::Opus(OpusRtpParser::new(callback))),
            "video/h264" => Ok(RtpParser::H264(H264RtpParser::new(callback))),
            _ => Err(anyhow!("Unsupported codec: {}", mime_type)),
        }
    }

    pub async fn parse(&mut self, data: Bytes) -> Option<(Vec<Bytes>, FrameInfo)> {
        match self {
            RtpParser::Opus(ref mut parser) => parser.parse(data).await,
            RtpParser::H264(ref mut parser) => parser.parse(data).await,
        }
    }
}
