use std::fmt;

const MASTER_M3U8: &str = "index.m3u8";
const VIDEO_M3U8: &str = "video.m3u8";
const INIT_FILE_NAME: &str = "init.mp4";
const OUTPUT_PREFIX: &str = "output";
const PUBLIC: &str = "public";

#[derive(Debug, Clone)]
pub struct HlsPath {
    pub prefix: String,
}

impl HlsPath {
    pub fn new(session_id: String) -> Self {
        Self {
            prefix: format!("{}/hls/{}", PUBLIC, session_id),
        }
    }

    pub fn get_path(&self, filename: &str) -> anyhow::Result<String> {
        if filename == MASTER_M3U8 {
            return Ok(self.get_master_path());
        }
        if filename == VIDEO_M3U8 {
            return Ok(self.get_playlist_path());
        }
        if filename == INIT_FILE_NAME {
            return Ok(self.get_init_video_path());
        }

        if filename.ends_with(".mp4") || filename.ends_with(".m4s") {
            return Ok(self.get_video_path(filename));
        }
        return Err(anyhow::anyhow!("Bad request"));
    }

    pub fn get_master_path(&self) -> String {
        format!("{}/{}", self.prefix, MASTER_M3U8)
    }
    pub fn get_playlist_path(&self) -> String {
        format!("{}/{}", self.prefix, VIDEO_M3U8)
    }
    pub fn get_init_video_path(&self) -> String {
        format!("{}/{}", self.prefix, INIT_FILE_NAME)
    }
    pub fn get_video_path(&self, filename: &str) -> String {
        format!("{}/{}", self.prefix, filename)
    }
    pub fn make_part_path(&self, segment_index: i32, part_index: i32) -> std::path::PathBuf {
        std::path::PathBuf::from(format!(
            "{}/{}_{}_{}.m4s",
            self.prefix, OUTPUT_PREFIX, segment_index, part_index,
        ))
    }
    pub fn make_segment_path(&self, segment_index: i32) -> std::path::PathBuf {
        std::path::PathBuf::from(format!(
            "{}/{}_{}.m4s",
            self.prefix, OUTPUT_PREFIX, segment_index,
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
