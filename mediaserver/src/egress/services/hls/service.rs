use std::{io::Cursor, sync::Arc};

use m3u8_rs::{MasterPlaylist, MediaPlaylist};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::RwLock,
};

use crate::utils;

use super::config::{HlsConfig, PathBufExt};

const INIT_FILE_NAME: &str = "init.mp4";

pub struct HlsPayload {
    pub duration: f32,
    pub payload: bytes::Bytes,
}

pub struct HlsService {
    config: HlsConfig,

    // m3u8, playlist, video
    master: RwLock<MasterPlaylist>,
    video0: RwLock<MediaPlaylist>,

    created_signal: tokio::sync::watch::Sender<(i32, i32)>,
}

impl HlsService {
    pub fn new(config: HlsConfig) -> Self {
        let mut master = MasterPlaylist::default();
        master.version = Some(10);
        master.independent_segments = true;

        let playlist_path = config.video_m3u8_path();
        let mut varient = m3u8_rs::VariantStream::default();
        varient.bandwidth = config.bandwidth;
        varient.codecs = Some(config.codecs.to_string());
        varient.resolution = Some(m3u8_rs::Resolution {
            width: config.width,
            height: config.height,
        });
        varient.frame_rate = Some(config.framerate);
        varient.uri = playlist_path;
        master.variants.push(varient);

        let playlist = MediaPlaylist {
            version: Some(10),
            target_duration: (1) as u64,
            server_control: Some(m3u8_rs::ServerControl {
                can_block_reload: true,
                part_hold_back: Some(0.5),
                ..Default::default()
            }),
            media_sequence: 0,
            discontinuity_sequence: 0,
            independent_segments: true,
            end_list: false,
            playlist_type: None,
            ..Default::default()
        };

        let mut video0 = playlist.clone();
        video0.part_inf = Some(config.part_duration);
        video0.map = Some(m3u8_rs::Map {
            uri: INIT_FILE_NAME.to_string(),
            ..Default::default()
        });

        let (created_signal, _) = tokio::sync::watch::channel((-1, -1));

        Self {
            config: config.clone(),
            video0: RwLock::new(video0),
            master: RwLock::new(master),
            created_signal,
        }
    }

    pub async fn init(&self) -> anyhow::Result<()> {
        let mut buffer = Vec::new();
        let master = self.master.write().await;
        if let Err(err) = master.write_to(&mut buffer) {
            log::warn!("failed to write playlist: {}", err);
        }

        utils::files::files::write_file_force(&self.config.get_master_path(), &buffer).await?;

        Ok(())
    }

    pub async fn init_segment(&self, payload: bytes::Bytes) -> anyhow::Result<()> {
        let fullpath = self.config.get_init_video_path();
        utils::files::files::write_file_force(&fullpath, &payload).await?;

        Ok(())
    }

    pub async fn init_segment2(&self, payload: bytes::Bytes) -> anyhow::Result<()> {
        let fullpath = self.config.get_init2_video_path();
        utils::files::files::write_file_force(&fullpath, &payload).await?;

        Ok(())
    }

    pub async fn write_segment2(&self, index: i32, hls_payload: HlsPayload) -> anyhow::Result<()> {
        let segment_index = index / self.config.part_max_count;
        let part_index = index % self.config.part_max_count;
        {
            let part_temp = self.config.make_part2_path(segment_index, part_index);
            let fullpath_temp: String = part_temp.get_fullpath()?;
            utils::files::files::write_file_force(&fullpath_temp, &hls_payload.payload).await?;
        }
        Ok(())
    }

    pub async fn write_segment(&self, index: i32, hls_payload: HlsPayload) -> anyhow::Result<()> {
        let segment_index = index / self.config.part_max_count;
        let part_index = index % self.config.part_max_count;

        let mut video0 = self.video0.write().await;
        let part = self.config.make_part_path(segment_index, part_index);
        // part video 쓰기
        let fullpath = part.get_fullpath()?;
        utils::files::files::write_file_force(&fullpath, &hls_payload.payload).await?;

        video0.parts.push(m3u8_rs::Part {
            duration: hls_payload.duration,
            uri: part.get_filename()?,
            independent: true,
        });
        if part_index == self.config.part_max_count - 1 {
            // need media segment
            let segment = self.config.make_segment_path(segment_index);
            let mut paths: Vec<std::path::PathBuf> = Vec::new();
            for i in 0..self.config.part_max_count {
                let filepath = self.config.make_part_path(segment_index, i);
                paths.push(filepath);
            }
            let fullpath = segment.get_fullpath()?;
            let paths: Vec<&std::path::Path> = paths.iter().map(|p| p.as_path()).collect();
            append_files(std::path::Path::new(&fullpath), &paths).await?;

            if video0.segments.len() > self.config.part_max_count as usize {
                video0.segments.remove(0);
                video0.media_sequence += 1;
            }

            let segment_duration: f32 = video0.parts.iter().map(|part| part.duration).sum();
            let parts_clone = video0.parts.clone();
            let title = segment_index.to_string();
            video0.segments.push(m3u8_rs::MediaSegment {
                uri: segment.get_filename()?,
                duration: segment_duration,
                title: Some(title),
                parts: parts_clone,
                ..Default::default()
            });
            video0.parts = vec![];

            let prepload = self.config.make_part_path(segment_index + 1, 0);
            video0.preload_hint = Some(m3u8_rs::PreloadHint {
                r#type: "PART".to_string(),
                uri: prepload.get_filename()?,
            });
        } else {
            // need media part
            let preload = self.config.make_part_path(segment_index, part_index + 1);
            video0.preload_hint = Some(m3u8_rs::PreloadHint {
                r#type: "PART".to_string(),
                uri: preload.get_filename()?,
            });
        }

        {
            // llhls playlist 쓰기
            let mut buffer = Vec::new();
            if let Err(err) = video0.write_to(&mut buffer) {
                log::warn!("failed to write playlist: {}", err);
            }
            let playlist_path = self.config.get_playlist_path();
            utils::files::files::write_file_force(&playlist_path, &buffer).await?;
        }

        let _ = self.created_signal.send((segment_index, part_index));
        Ok(())
    }

    pub fn subscribe_signal(&self) -> tokio::sync::watch::Receiver<(i32, i32)> {
        self.created_signal.subscribe()
    }
}

async fn append_files(
    output: &std::path::Path,
    inputs: &[&std::path::Path],
) -> tokio::io::Result<()> {
    // 출력 파일을 비동기로 열고, 없으면 생성하고 기존 내용을 지웁니다.
    let mut output_file = tokio::fs::File::create(output).await?;

    for input_path in inputs {
        // 입력 파일을 비동기로 열고 모든 내용을 읽어들입니다.
        let mut input_file = tokio::fs::File::open(input_path).await?;
        let mut buffer = Vec::new();

        // 파일 내용을 모두 비동기로 읽어서 버퍼에 저장합니다.
        input_file.read_to_end(&mut buffer).await?;

        // 출력 파일에 버퍼 내용을 비동기로 씁니다.
        output_file.write_all(&buffer).await?;
    }

    Ok(())
}
