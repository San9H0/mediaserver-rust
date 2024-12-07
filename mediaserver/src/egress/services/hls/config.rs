use std::fmt;

use chrono::format;

const MASTER_M3U8: &str = "index.m3u8";
const VIDEO_M3U8: &str = "video.m3u8";
const INIT_FILE_NAME: &str = "init.mp4";
const OUTPUT_PREFIX: &str = "output";
const PUBLIC: &str = "public";

#[derive(Debug, Clone)]
pub struct HlsConfig {
    pub prefix: String,
    pub video_base: String,

    pub codecs: String,
    pub bandwidth: u64,
    pub width: u64,
    pub height: u64,
    pub framerate: f64,
    pub part_duration: f32,
    pub part_max_count: i32,
}

pub struct ConfigParams {
    pub session_id: String,
    pub video_base: String,
    pub codecs: String,
    pub bandwidth: u64,
    pub width: u64,
    pub height: u64,
    pub framerate: f64,
    pub part_duration: f32,
    pub part_max_count: i32,
}

impl HlsConfig {
    pub fn new(params: ConfigParams) -> Self {
        Self {
            prefix: format!("{}/hls/{}", PUBLIC, params.session_id),
            video_base: params.video_base.clone(),
            codecs: params.codecs,
            bandwidth: params.bandwidth,
            width: params.width,
            height: params.height,
            framerate: params.framerate,
            part_duration: params.part_duration,
            part_max_count: params.part_max_count,
        }
    }

    pub fn base_path(&self, filename: &str) -> String {
        format!("{}/{}", self.prefix, filename)
    }

    pub fn video_m3u8_path(&self) -> String {
        format!("{}/{}", &self.video_base, VIDEO_M3U8)
    }

    pub fn get_path(&self, filename: &str) -> anyhow::Result<String> {
        if filename.ends_with(MASTER_M3U8) {
            return Ok(self.base_path(filename));
        }
        if filename.ends_with(VIDEO_M3U8) {
            return Ok(self.base_path(filename));
        }
        if filename.ends_with(INIT_FILE_NAME) {
            return Ok(self.base_path(filename));
        }
        if filename.ends_with(".mp4") || filename.ends_with(".m4s") {
            return Ok(self.base_path(filename));
        }
        return Err(anyhow::anyhow!("Bad request"));
    }

    pub fn get_master_path(&self) -> String {
        format!("{}/{}", &self.prefix, MASTER_M3U8)
    }
    pub fn get_playlist_path(&self) -> String {
        format!("{}/{}/{}", &self.prefix, &self.video_base, VIDEO_M3U8)
    }
    pub fn get_init_video_path(&self) -> String {
        format!("{}/{}/{}", &self.prefix, &self.video_base, INIT_FILE_NAME)
    }
    pub fn make_part_path(&self, segment_index: i32, part_index: i32) -> std::path::PathBuf {
        std::path::PathBuf::from(format!(
            "{}/{}/{}_{}_{}.m4s",
            self.prefix, &self.video_base, OUTPUT_PREFIX, segment_index, part_index,
        ))
    }
    pub fn make_segment_path(&self, segment_index: i32) -> std::path::PathBuf {
        std::path::PathBuf::from(format!(
            "{}/{}/{}_{}.m4s",
            self.prefix, &self.video_base, OUTPUT_PREFIX, segment_index,
        ))
    }
}

pub trait PathBufExt {
    fn get_fullpath(&self) -> anyhow::Result<String>;
    fn get_filename(&self) -> anyhow::Result<String>;
    fn get_extension(&self) -> anyhow::Result<String>;
    fn get_parent(&self) -> anyhow::Result<std::path::PathBuf>;
}

impl PathBufExt for std::path::PathBuf {
    fn get_fullpath(&self) -> anyhow::Result<String> {
        self.to_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve fullpath as UTF-8"))
    }

    fn get_filename(&self) -> anyhow::Result<String> {
        self.file_name()
            .and_then(|name| name.to_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve filename as UTF-8"))
    }

    fn get_extension(&self) -> anyhow::Result<String> {
        self.extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve extension as UTF-8"))
    }

    fn get_parent(&self) -> anyhow::Result<std::path::PathBuf> {
        self.parent()
            .map(|p| p.to_path_buf())
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve parent directory"))
    }
}
