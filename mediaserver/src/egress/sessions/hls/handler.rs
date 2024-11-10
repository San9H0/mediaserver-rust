use crate::codecs::codec::Codec;
use crate::egress::servers::hls::HlsService;
use crate::egress::sessions::hls::output::OutputWrap;
use crate::egress::sessions::hls::track_context;
use crate::egress::sessions::session::SessionHandler;
use crate::hubs::source::HubSource;
use crate::hubs::stream::HubStream;
use crate::hubs::unit::HubUnit;
use crate::utils::types::types;
use bitstreams::h264::nal_unit::NalUnit;
use ffmpeg_next as ffmpeg;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

struct HlsState {
    index: i32,
    started: bool,
    prev_pts: u32,
    duration_sum: i64,
    count: i32,
    prev_time: tokio::time::Instant,
}

impl HlsState {
    fn new() -> Self {
        Self {
            started: false,
            index: 0,
            count: 0,
            prev_time: tokio::time::Instant::now(),
            prev_pts: 0,
            duration_sum: 0,
        }
    }
}

pub struct HlsHandler {
    started: AtomicBool,
    state: RwLock<HlsState>,

    output_ctx: Mutex<OutputWrap>,
    sources: Vec<Arc<HubSource>>,
    target: Arc<HlsService>,
}

impl HlsHandler {
    pub async fn new(hub_stream: &Arc<HubStream>, target: Arc<HlsService>) -> anyhow::Result<Self> {
        let mut output_ctx = OutputWrap::new()?;
        let mut sources = vec![];
        for source in hub_stream.get_sources().await {
            let codec_info = source.get_codec().await.unwrap();
            if codec_info.kind() == types::MediaKind::Audio {
                let codec = ffmpeg::codec::encoder::find(codec_info.av_codec_id())
                    .ok_or(ffmpeg::Error::EncoderNotFound)?;
                let encoder_ctx = ffmpeg::codec::context::Context::new_with_codec(codec);
                let mut audio = encoder_ctx.encoder().audio()?;
                let _ = codec_info.set_av_audio(&mut audio);
                let encoder = audio.open_as(codec)?;
                output_ctx.add_stream_with(&encoder)?;
            } else if codec_info.kind() == types::MediaKind::Video {
                let codec = ffmpeg::codec::encoder::find(codec_info.av_codec_id())
                    .ok_or(ffmpeg::Error::EncoderNotFound)?;
                let encoder_ctx = ffmpeg::codec::context::Context::new_with_codec(codec);
                let mut video = encoder_ctx.encoder().video()?;
                let _ = codec_info.set_av_video(&mut video)?;
                let encoder = video.open_as(codec)?;
                println!(
                    "encoder.time_base(): {}, {}",
                    encoder.time_base().0,
                    encoder.time_base().1
                );

                output_ctx.add_stream_with(&encoder)?;
            }
            sources.push(source);
        }

        Ok(HlsHandler {
            state: RwLock::new(HlsState::new()),
            started: AtomicBool::new(false),
            output_ctx: Mutex::new(output_ctx),
            sources,
            target,
        })
    }
}

impl SessionHandler for HlsHandler {
    type TrackContext = track_context::TrackContext;

    async fn on_initialize(&self) -> anyhow::Result<()> {
        {
            let mut output_ctx = self.output_ctx.lock().await;
            let mut dict = ffmpeg::Dictionary::new();
            dict.set("movflags", "frag_keyframe+empty_moov+default_base_moof");
            output_ctx.write_header_with(dict)?;
        }

        let payload = {
            let mut p = self.output_ctx.lock().await;
            let b = p.get_payload()?;
            b
        };

        self.target.init_segment(payload).await?;

        Ok(())
    }
    async fn on_finalize(&self) -> anyhow::Result<()> {
        let mut output_ctx = self.output_ctx.lock().await;
        output_ctx.write_trailer()?;

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

        let duration = pkt.duration();
        let time_base = pkt.time_base();
        let pkt2 = pkt.clone();
        if let Err(err) = {
            let mut output_ctx = self.output_ctx.lock().await;
            pkt.write_interleaved(&mut output_ctx)
        } {
            log::warn!("failed to write packet: {}", err);
        };

        log::info!("[TESTDEBUG] video unit.pts:{}, pkt2.pts:{:?}", unit.pts, pkt2.pts());
        let (index, duration) = {
            let mut state = self.state.write().await;
            if state.duration_sum == 0 {
                log::info!("first packet? pts: {:?}, dts:{:?}, data[0]:{:02x}, data[1]:{:02x}, data[2]:{:02x}, data[3]:{:02x}, data[4]:{:02x}", 
                pkt2.pts(), pkt2.dts(), unit.payload[0], unit.payload[1], unit.payload[2], unit.payload[3], unit.payload[4]);
            }
            state.duration_sum += duration;

            if !state.started {
                state.started = true;
                state.prev_time = tokio::time::Instant::now();
                return;
            } else if state.prev_time.elapsed() < tokio::time::Duration::from_secs(1) {
                return;
            } 
            let duration = state.duration_sum as f32 / time_base.1 as f32;
            if duration < 1.0 {
                return;
            }
            state.prev_time = tokio::time::Instant::now();

        
            log::info!("last packet duration:{}, pts: {:?}, dts:{:?}, data[0]:{:02x}, data[1]:{:02x}, data[2]:{:02x}, data[3]:{:02x}, data[4]:{:02x}", 
                duration, pkt2.pts(), pkt2.dts(), unit.payload[0], unit.payload[1], unit.payload[2], unit.payload[3], unit.payload[4]);
            
            state.duration_sum = 0;
            let index = state.index;
            state.index += 1;
            state.prev_pts = unit.pts;

            (index, duration)
        };

        let payload = {
            let mut p = self.output_ctx.lock().await;
            let Ok(b) = p.get_payload() else {
                log::warn!("failed to get payload");
                return;
            };
            b
        };


        if let Err(err) = self.target.write_segment(index, duration, payload).await {
            log::warn!("failed to write segment: {}", err);
        }

    }

    async fn on_audio(&self, ctx: &mut Self::TrackContext, unit: &HubUnit) {
        if !self.started.load(Ordering::Acquire) {
            return;
        }

        let Some(pkt) = ctx.make_packet(unit) else {
            return;
        };

        
        let pkt2 = pkt.clone();
        if let Err(err) = {
            let mut output_ctx = self.output_ctx.lock().await;
            pkt.write_interleaved(&mut output_ctx)
        } {
            log::warn!("failed to write packet: {}", err);
        };

        let mut state = self.state.write().await;
        log::info!("[TESTDEBUG] audio unit.pts:{}, pkt2.pts:{:?}", unit.pts, pkt2.pts());
    }
}
