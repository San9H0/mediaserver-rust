use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio_util::sync::CancellationToken;
use crate::hubs::stream::HubStream;
use ffmpeg_next as ffmpeg;
use ffmpeg_next::format::context;
use ffmpeg_next::Rescale;
use ffmpeg_sys_next::sprintf;
use tokio::sync::Mutex;
use crate::codecs::bfs::Bfs;
use crate::codecs::codec::Codec;
use crate::egress::sessions::record::track_context;
use crate::egress::sessions::session::SessionHandler;
use crate::hubs::source::HubSource;
use crate::hubs::unit::HubUnit;
use crate::utils::files::directory::create_directory_if_not_exists;

pub struct RecordHandler {
    started: AtomicBool,
    hub_stream: Arc<HubStream>,
    token: CancellationToken,

    output_ctx: Arc<Mutex<context::Output>>,
    sources: Vec<Arc<HubSource>>,
}

impl RecordHandler {
    pub async fn new(hub_stream: &Arc<HubStream>, session_id: &str) -> anyhow::Result<Self> {
        let token = CancellationToken::new();
        let name_filename = format!("{}/output.mp4", session_id);
        create_directory_if_not_exists(&name_filename)?;
        println!("name_filename:{}", &name_filename);
        let mut output_ctx = ffmpeg::format::output(&name_filename)?;
        let mut sources =  vec![];
        for source in hub_stream.get_sources().await {
            let codec_info = source.get_codec().await.unwrap();
            if codec_info.kind() == "audio" {
                let codec = ffmpeg::codec::encoder::find(codec_info.av_codec_id())
                    .ok_or(ffmpeg::Error::EncoderNotFound)?;
                let encoder_ctx = ffmpeg::codec::context::Context::new_with_codec(codec);
                let mut audio = encoder_ctx.encoder().audio()?;
                let a = codec_info.set_av_audio(&mut audio);
                let encoder = audio.open_as(codec)?;
                output_ctx.add_stream_with(&encoder)?;

            } else if codec_info.kind() == "video" {
                let codec = ffmpeg::codec::encoder::find(codec_info.av_codec_id())
                    .ok_or(ffmpeg::Error::EncoderNotFound)?;
                let encoder_ctx = ffmpeg::codec::context::Context::new_with_codec(codec);

                let mut video = encoder_ctx.encoder().video()?;
                codec_info.set_av_video(&mut video)?;
                let encoder = video.open_as(codec)?;
                output_ctx.add_stream_with(&encoder)?;
            }
            sources.push(source);
        }

        Ok(RecordHandler {
            started: AtomicBool::new(false),
            hub_stream: hub_stream.clone(),
            token,
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

    fn stop(&self) {
        self.token.cancel();
    }
    fn cancel_token(&self) -> CancellationToken {
        self.token.clone()
    }

    fn get_sources(&self) -> Vec<Arc<HubSource>> {
        self.sources.clone()
    }

    fn on_track(&self, idx: usize, codec: &Codec) -> track_context::TrackContext {
        track_context::TrackContext::new(idx, codec)
    }

    async fn on_video(&self, ctx: &mut track_context::TrackContext, unit: &HubUnit) {
        if !self.started.load(Ordering::Acquire) {
            if unit.frame_info.flag != 1 {
                return
            }
            self.started.store(true, Ordering::Release);
        }

        let pkt = ctx.make_packet(unit);

        let mut output_ctx = self.output_ctx.lock().await;
        if let Err(err) = pkt.write_interleaved(&mut output_ctx) {
            log::warn!("failed to write packet: {}", err);
        };
    }

    async fn on_audio(&self, ctx: &mut track_context::TrackContext, unit: &HubUnit) {
        if !self.started.load(Ordering::Acquire) {
            return;
        }

        let pkt = ctx.make_packet(unit);

        let mut output_ctx = self.output_ctx.lock().await;
        if let Err(err) = pkt.write_interleaved(&mut output_ctx) {
            log::warn!("failed to write packet: {}", err);
        };
    }
}

