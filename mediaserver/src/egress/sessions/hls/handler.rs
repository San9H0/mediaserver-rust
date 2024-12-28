use crate::codecs::codec::Codec;
use crate::egress::services::hls::service::{HlsPayload, HlsService};
use crate::egress::sessions::hls::track_context;
use crate::egress::sessions::session::SessionHandler;
use crate::hubs::source::HubSource;
use crate::hubs::stream::HubStream;
use crate::hubs::unit::HubUnit;
use crate::utils;
use crate::utils::types::types;
use std::io::{Cursor, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

struct HlsState {
    index: i32,
    started: bool,
    duration_sum: i64,
    prev_time: tokio::time::Instant,
}

impl HlsState {
    fn new() -> Self {
        Self {
            started: false,
            index: 0,
            prev_time: tokio::time::Instant::now(),
            duration_sum: 0,
        }
    }
}

pub struct HlsHandler {
    started: AtomicBool,
    state: RwLock<HlsState>,

    sources: Vec<Arc<HubSource>>,
    target: Arc<HlsService>,

    fmp4: Mutex<mp4::Fmp4Writer>,
}

impl HlsHandler {
    pub async fn new(hub_stream: &Arc<HubStream>, target: Arc<HlsService>) -> anyhow::Result<Self> {
        let mut width = 0;
        let mut height = 0;
        let mut sps = None;
        let mut pps = None;
        let mut audio_timescale = 0;
        let mut video_timescale = 0;

        let mut sources = vec![];
        for source in hub_stream.get_sources().await {
            let codec_info = source.get_codec().await.unwrap();
            if codec_info.kind() == types::MediaKind::Audio {
                audio_timescale = codec_info.clock_rate();
            } else if codec_info.kind() == types::MediaKind::Video {
                sps = codec_info.sps();
                pps = codec_info.pps();

                width = codec_info.width();
                height = codec_info.height();

                video_timescale = codec_info.clock_rate();
            }
            sources.push(source);
        }

        let fmp4_config = mp4::Mp4Config {
            major_brand: str::parse("iso5").unwrap(),
            minor_version: 512,
            compatible_brands: vec![
                str::parse("iso5").unwrap(),
                str::parse("iso6").unwrap(),
                str::parse("mp41").unwrap(),
            ],
            timescale: 1000,
        };
        let mut fmp4 = mp4::Fmp4Writer::new(&fmp4_config).unwrap();
        fmp4.add_track(&mp4::TrackConfig {
            track_type: mp4::TrackType::Audio,
            timescale: audio_timescale,
            language: String::from("und"),
            media_conf: mp4::MediaConfig::OpusConfig(mp4::OpusConfig {}),
        })
        .unwrap();
        fmp4.add_track(&mp4::TrackConfig {
            track_type: mp4::TrackType::Video,
            timescale: video_timescale,
            language: String::from("und"),
            media_conf: mp4::MediaConfig::AvcConfig(mp4::AvcConfig {
                width: width as u16,
                height: height as u16,
                seq_param_set: sps.unwrap(),
                pic_param_set: pps.unwrap(),
            }),
        })
        .unwrap();

        Ok(HlsHandler {
            state: RwLock::new(HlsState::new()),
            started: AtomicBool::new(false),
            sources,
            target,
            fmp4: Mutex::new(fmp4),
        })
    }

    pub async fn write_hls_segment(&self, pkt: &utils::packet::packet::Packet) {
        let (index, duration) = {
            let mut state = self.state.write().await;
            state.duration_sum += pkt.duration();

            if !state.started {
                state.started = true;
                state.prev_time = tokio::time::Instant::now();
                return;
            } else if state.prev_time.elapsed() < tokio::time::Duration::from_millis(500) {
                return;
            }
            let duration = state.duration_sum as f32 / pkt.time_base().den as f32;
            // if duration < 1.0 {
            //     return;
            // }
            state.prev_time = tokio::time::Instant::now();

            state.duration_sum = 0;
            let index = state.index;
            state.index += 1;

            (index, duration)
        };

        {
            let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::<u8>::new());
            let mut fmp4 = self.fmp4.lock().await;
            if let Err(err) = fmp4.write_end(&mut cursor) {
                println!("failed to write end: {}", err);
            }

            let data = cursor.into_inner();
            if let Err(err) = self
                .target
                .write_segment(
                    index,
                    HlsPayload {
                        duration,
                        payload: bytes::Bytes::from(data),
                    },
                )
                .await
            {
                println!("failed to write segment: {}", err);
            }
        }
    }
}

impl SessionHandler for HlsHandler {
    type TrackContext = track_context::TrackContext;

    async fn on_initialize(&self) -> anyhow::Result<()> {
        {
            let mut fmp4 = self.fmp4.lock().await;
            let mut cursor = Cursor::new(Vec::<u8>::new());
            fmp4.write_header(&mut cursor)?;
            let data = cursor.into_inner();
            self.target.init_segment(bytes::Bytes::from(data)).await?;
        }
        Ok(())
    }
    async fn on_finalize(&self) -> anyhow::Result<()> {
        Ok(())
    }
    fn get_sources(&self) -> Vec<Arc<HubSource>> {
        self.sources.clone()
    }

    fn on_track_context(&self, idx: usize, codec: &Codec) -> Self::TrackContext {
        track_context::TrackContext::new(idx, codec)
    }

    async fn on_video(&self, ctx: &mut Self::TrackContext, unit: &HubUnit) {
        if !self.started.load(Ordering::Acquire) {
            if unit.frame_info.flag != 1 {
                return;
            }
            self.started.store(true, Ordering::Release);
        }

        let Some(pkt) = ctx.make_packet(unit) else {
            return;
        };

        {
            if let Some(data) = pkt.data() {
                let bytes = bytes::Bytes::copy_from_slice(data);

                let mut fmp4 = self.fmp4.lock().await;
                let mp4Sample = mp4::Mp4Sample {
                    start_time: unit.pts as u64,
                    duration: unit.duration as u32,
                    rendering_offset: 0,
                    is_sync: false,
                    bytes,
                };
                if let Err(err) = fmp4.write_sample(2, &mp4Sample) {
                    log::warn!("failed to write sample: {}", err);
                }
            }
        }

        self.write_hls_segment(&pkt).await;
    }

    async fn on_audio(&self, ctx: &mut Self::TrackContext, unit: &HubUnit) {
        if !self.started.load(Ordering::Acquire) {
            return;
        }

        let Some(pkt) = ctx.make_packet(unit) else {
            return;
        };

        {
            if let Some(data) = pkt.data() {
                let bytes = bytes::Bytes::copy_from_slice(data);
                let mut fmp4 = self.fmp4.lock().await;
                let mp4Sample = mp4::Mp4Sample {
                    start_time: unit.pts as u64,
                    duration: unit.duration as u32,
                    rendering_offset: 0,
                    is_sync: false,
                    bytes,
                };
                if let Err(err) = fmp4.write_sample(1, &mp4Sample) {
                    log::warn!("failed to write sample: {}", err);
                }
            }
        }
    }
}
