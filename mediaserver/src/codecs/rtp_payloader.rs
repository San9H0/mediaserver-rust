use crate::codecs::h264::rtp_payloader::H264RtpPayloader;
use crate::codecs::opus::rtp_payloader::OpusRtpPayloader;
use anyhow::anyhow;
use bytes::Bytes;

use super::codec::Codec;

pub enum RtpPayloader {
    Opus(OpusRtpPayloader),
    H264(H264RtpPayloader),
}

impl RtpPayloader {
    pub fn new(codec: &Codec, mime_type: &str) -> anyhow::Result<RtpPayloader> {
        match mime_type.to_lowercase().as_str() {
            "audio/opus" => Ok(RtpPayloader::Opus(OpusRtpPayloader::new())),
            "video/h264" => Ok(RtpPayloader::H264(H264RtpPayloader::new(codec))),
            _ => Err(anyhow!("Unsupported codec: {}", mime_type)),
        }
    }

    pub fn payload(&mut self, mtu: usize, b: &Bytes) -> anyhow::Result<Vec<Bytes>> {
        match self {
            RtpPayloader::Opus(payloader) => payloader.payload(mtu, b),
            RtpPayloader::H264(payloader) => payloader.payload(mtu, b),
        }
    }
}
