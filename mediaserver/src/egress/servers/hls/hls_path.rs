const MASTER_M3U8: &str = "index.m3u8";
const VIDEO_M3U8: &str = "video.m3u8";
const INIT_FILE_NAME: &str = "init.mp4";
const OUTPUT_PREFIX: &str = "output";
const PUBLIC: &str = "public";

#[derive(Debug, Clone)]
pub struct HlsPath {
    pub prefix: String,

    pub master_endpoint_llhls: String,
    pub master_endpoint_hls: String,
    pub playlist_endpoint_llhls: String,
    pub playlist_endpoint_hls: String,
    pub init_video_endpoint: String,
    pub video_endpoint_prefx: String,
}

impl HlsPath {
    pub fn new(session_id: String) -> Self {
        let prefix = format!("{}/{}", PUBLIC, session_id);
        let master_endpoint_llhls = fmt::format(format_args!("{prefix}/llhls/{MASTER_M3U8}"));
        let master_endpoint_hls = fmt::format(format_args!("{prefix}/hls/{MASTER_M3U8}"));
        let playlist_endpoint_llhls = fmt::format(format_args!("{prefix}/llhls/{VIDEO_M3U8}"));
        let playlist_endpoint_hls = fmt::format(format_args!("{prefix}/hls/{VIDEO_M3U8}"));
        let init_video_endpoint = fmt::format(format_args!("{prefix}/video/{INIT_FILE_NAME}"));
        let video_endpoint_prefx = fmt::format(format_args!("{prefix}/video/"));
        Self {
            prefix: format!("{}/{}", PUBLIC, session_id),
            master_endpoint_llhls,
            playlist_endpoint_llhls,
            playlist_endpoint_hls,
            init_video_endpoint,
            video_endpoint_prefx,
            master_endpoint_hls,
        }
    }
    pub fn make_master_path(&self, is_llhls: bool) -> String {
        if is_llhls {
            format!("{}/llhls/{}", self.prefix, MASTER_M3U8)
        } else {
            format!("{}/hls/{}", self.prefix, MASTER_M3U8)
        }
    }
    pub fn make_playlist_path(&self, is_llhls: bool) -> String {
        if is_llhls {
            format!("{}/llhls/{}", self.prefix, VIDEO_M3U8)
        } else {
            format!("{}/hls/{}", self.prefix, VIDEO_M3U8)
        }
    }
    pub fn make_init_video_path(&self) -> String {
        format!("{}/video/{}", self.prefix, INIT_FILE_NAME)
    }
    pub fn make_video_path(&self, filename: &str) -> String {
        format!("{}/video/{}", self.prefix, filename)
    }

    pub fn make_part_path(&self, segment_index: i32, part_index: i32) -> std::path::PathBuf {
        std::path::PathBuf::from(format!(
            "{}/video/{}_{}_{}.m4s",
            self.prefix, OUTPUT_PREFIX, segment_index, part_index,
        ))
    }
    pub fn make_segment_path(&self, segment_index: i32) -> std::path::PathBuf {
        std::path::PathBuf::from(format!(
            "{}/video/{}_{}.m4s",
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
