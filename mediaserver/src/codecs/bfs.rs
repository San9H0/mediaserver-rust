use anyhow::anyhow;
use crate::codecs::bfs::Bfs::{Opus, H264};
use crate::hubs::unit::HubUnit;
use ffmpeg_next as ffmpeg;
use crate::codecs;

pub enum Bfs {
    Opus,
    H264,
}

impl Bfs {
    pub fn new(mime_type: &str) -> anyhow::Result<Bfs> {
        match mime_type.to_lowercase().as_str() {
            "audio/opus" => Ok(Opus),
            "video/h264" => Ok(H264),
            _ => Err(anyhow!("Unsupported codec: {}", mime_type)),
        }
    }

    pub fn make_packet(&self, unit: &HubUnit) -> Option<ffmpeg::packet::Packet> {
        match self {
            Opus => codecs::opus::bfs::make_packet(unit),
            H264 => codecs::h264::bfs::make_packet_with_avcc(unit),
        }
    }
}
