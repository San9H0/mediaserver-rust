use std::ptr;
use crate::codecs::codec::Codec;
use crate::egress::sessions::hls::track_context;
use crate::egress::sessions::session::SessionHandler;
use crate::hubs::source::HubSource;
use crate::hubs::stream::HubStream;
use crate::hubs::unit::HubUnit;
use anyhow::anyhow;
use ffmpeg_next as ffmpeg;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::io::BufWriter;
use tokio::sync::Mutex;
use crate::egress::servers::hls::{HlsPayloader, HlsService2};
use crate::egress::sessions::hls::output::OutputWrap;
use crate::utils::types::types;

pub struct HlsHandler {
    started: AtomicBool,
    // session_id: String,
    // started: AtomicBool,
    // token: CancellationToken,
    //
    output_ctx: Arc<Mutex<OutputWrap>>,
    sources: Vec<Arc<HubSource>>,
    target: Arc<Mutex<HlsService2>>,
}

impl HlsHandler {
    pub async fn new(hub_stream: &Arc<HubStream>, target: HlsService2) -> anyhow::Result<Self> {
        let file = tokio::fs::File::create("output.txt").await?;
        let mut writer = tokio::io::BufWriter::new(file);

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
                output_ctx.add_stream_with(&encoder)?;
            }
            sources.push(source);
        }

        Ok(HlsHandler {
            started: AtomicBool::new(false),
            // token,
            output_ctx: Arc::new(Mutex::new(output_ctx)),
            sources,
            target: Arc::new(Mutex::new(target)),
        })
    }
}

impl SessionHandler for HlsHandler {
    type TrackContext = track_context::TrackContext;

    async fn on_initialize(&self) -> anyhow::Result<()> {
        let mut output_ctx = self.output_ctx.lock().await;

        let mut dict = ffmpeg::Dictionary::new();
        dict.set("movflags", "frag_keyframe+empty_moov+default_base_moof");
        output_ctx.write_header_with(dict)?;

        // let Ok(payloads) = output_ctx.get_buffer() else {
        //     return Err(anyhow!("get_buffer failed"));
        // };
        // println!("init payloads: {}", payloads.len());

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

    async fn on_video(
        &self,
        ctx: &mut Self::TrackContext,
        unit: &HubUnit,
    ) {
        if !self.started.load(Ordering::Acquire) {
            if unit.frame_info.flag != 1 {
                return;
            }
            self.started.store(true, Ordering::Release);
        }

        let Some(pkt) = ctx.make_packet(unit) else {
            return;
        };

        if let Err(err) = {
            let mut hls_service = self.target.lock().await;
            hls_service.write_segment(self.output_ctx.clone()).await
        } {
            log::warn!("failed to write segment: {}", err);
        }

        let mut output_ctx = self.output_ctx.lock().await;
        if let Err(err) = pkt.write_interleaved(&mut output_ctx) {
            log::warn!("failed to write packet: {}", err);
        };
    }

    async fn on_audio(
        &self,
        ctx: &mut Self::TrackContext,
        unit: &HubUnit,
    ) {
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
