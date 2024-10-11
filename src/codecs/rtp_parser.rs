use crate::codecs::h264::rtp_parser::H264RtpParser;
use crate::codecs::h264::rtp_payloader::H264RtpPayloader;
use crate::codecs::opus::rtp_parser::OpusRtpParser;
use crate::hubs::unit::FrameInfo;
use anyhow::anyhow;
use bytes::Bytes;

pub enum RtpParser {
    Opus(OpusRtpParser),
    H264(H264RtpParser),
}

impl RtpParser {
    pub fn new(mime_type: &str) -> anyhow::Result<RtpParser> {
        match mime_type.to_lowercase().as_str() {
            "audio/opus" => Ok(RtpParser::Opus(OpusRtpParser::new())),
            "video/h264" => Ok(RtpParser::H264(H264RtpParser::new())),
            _ => Err(anyhow!("Unsupported codec: {}", mime_type)),
        }
    }

    pub fn parse(&mut self, data: bytes::Bytes) -> Option<(Vec<Bytes>, FrameInfo)> {
        match self {
            RtpParser::Opus(ref mut parser) => parser.parse(data),
            RtpParser::H264(ref mut parser) => parser.parse(data),
        }
    }
}
