use crate::codecs::codec::Codec;
use crate::egress::services::hls::service::{HlsPayload, HlsService};
use crate::egress::sessions::hls::output::OutputWrap;
use crate::egress::sessions::hls::track_context;
use crate::egress::sessions::session::SessionHandler;
use crate::hubs::source::HubSource;
use crate::hubs::stream::HubStream;
use crate::hubs::unit::HubUnit;
use crate::utils::types::types;
use bitstreams::h264::nal_unit::NalUnit;
use ffmpeg_next as ffmpeg;
use mp4::Mp4Config;
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

    pub async fn write_hls_segment(&self, pkt: &ffmpeg::packet::Packet) {
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
            let duration = state.duration_sum as f32 / pkt.time_base().1 as f32;
            // if duration < 1.0 {
            //     return;
            // }
            state.prev_time = tokio::time::Instant::now();

            state.duration_sum = 0;
            let index = state.index;
            state.index += 1;

            (index, duration)
        };

        let payload = {
            let mut output = self.output_ctx.lock().await;
            let Ok(payload) = output.get_payload() else {
                log::warn!("failed to get payload");
                return;
            };
            payload
        };

        if let Err(err) = self
            .target
            .write_segment(index, HlsPayload { duration, payload })
            .await
        {
            log::warn!("failed to write segment: {}", err);
        }
    }
}

fn write_to_temp_file(data: Vec<u8>, file_path: &str) -> anyhow::Result<()> {
    // 지정된 경로에 파일 생성
    let path = std::path::Path::new(file_path);
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    // Vec<u8> 데이터를 파일에 작성
    file.write_all(&data)?;

    Ok(())
}

impl SessionHandler for HlsHandler {
    type TrackContext = track_context::TrackContext;

    async fn on_initialize(&self) -> anyhow::Result<()> {
        let config = Mp4Config {
            major_brand: str::parse("mp42").unwrap(),
            minor_version: 1,
            compatible_brands: vec![
                str::parse("mp41").unwrap(),
                str::parse("mp42").unwrap(),
                str::parse("isom").unwrap(),
                str::parse("hlsf").unwrap(),
            ],
            timescale: 1000,
        };

        let data = Cursor::new(Vec::<u8>::new());
        let mut writer = mp4::Mp4Writer::write_start(data, &config)?;
        writer.write_end()?;

        let data: Vec<u8> = writer.into_writer().into_inner();
        write_to_temp_file(data, "./test.mp4");

        {
            let mut output_ctx = self.output_ctx.lock().await;
            let mut dict = ffmpeg::Dictionary::new();
            // dict.set("movflags", "frag_keyframe+empty_moov+default_base_moof");
            dict.set("movflags", "empty_moov+default_base_moof");
            dict.set("frag_duration", "100000"); // 1_000_000

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

        self.write_hls_segment(&pkt).await;

        if let Err(err) = {
            let mut output_ctx = self.output_ctx.lock().await;
            pkt.write_interleaved(&mut output_ctx)
        } {
            log::warn!("failed to write packet: {}", err);
            return;
        };
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
    }
}
