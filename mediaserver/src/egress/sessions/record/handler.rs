use crate::codecs::codec::Codec;
use crate::egress::sessions::record::track_context;
use crate::egress::sessions::session::SessionHandler;
use crate::hubs::source::HubSource;
use crate::hubs::stream::HubStream;
use crate::hubs::unit::HubUnit;
use crate::utils::files::directory::create_directory_if_not_exists;
use crate::utils::types::types;
use ffmpeg_next as ffmpeg;
use ffmpeg_next::format::context;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct RecordHandler {
    started: AtomicBool,

    output_ctx: Arc<Mutex<context::Output>>,
    sources: Vec<Arc<HubSource>>,
}

impl RecordHandler {
    pub async fn new(hub_stream: &Arc<HubStream>, base_path: &str) -> anyhow::Result<Self> {
        let name_filename = format!("{}/output.mp4", base_path);
        create_directory_if_not_exists(&name_filename)?;

        let mut output_ctx = ffmpeg::format::output(&name_filename)?;
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
                output_ctx.add_stream_with(&encoder)?;
            }
            sources.push(source);
        }

        Ok(RecordHandler {
            started: AtomicBool::new(false),
            output_ctx: Arc::new(Mutex::new(output_ctx)),
            sources,
        })
    }
}

impl SessionHandler for RecordHandler {
    type TrackContext = track_context::TrackContext;
    async fn on_initialize(&self) -> anyhow::Result<()> {
        let mut output_ctx = self.output_ctx.lock().await;
        output_ctx.write_header()?;

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

    fn on_track_context(&self, idx: usize, codec: &Codec) -> track_context::TrackContext {
        track_context::TrackContext::new(idx, codec)
    }

    async fn on_video(&self, ctx: &mut track_context::TrackContext, unit: &HubUnit) {
        if !self.started.load(Ordering::Acquire) {
            if unit.frame_info.flag != 1 {
                return;
            }
            self.started.store(true, Ordering::Release);
        }

        let Some(pkt) = ctx.make_packet(unit) else {
            return;
        };

        let mut output_ctx = self.output_ctx.lock().await;
        if let Err(err) = pkt.write_interleaved(&mut output_ctx) {
            log::warn!("failed to write packet: {}", err);
        };
    }

    async fn on_audio(&self, ctx: &mut track_context::TrackContext, unit: &HubUnit) {
        if !self.started.load(Ordering::Acquire) {
            return;
        }

        let Some(pkt) = ctx.make_packet(unit) else {
            return;
        };

        let mut output_ctx = self.output_ctx.lock().await;
        if let Err(err) = pkt.write_interleaved(&mut output_ctx) {
            log::warn!("failed to write packet: {}", err);
        };
    }
}
