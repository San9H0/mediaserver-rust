use std::future::Future;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use crate::codecs::codec::Codec;
use crate::egress::sessions::hls::track_context;
use crate::egress::sessions::session::SessionHandler;
use crate::hubs::source::HubSource;
use crate::hubs::stream::HubStream;
use crate::hubs::unit::HubUnit;
use ffmpeg_next as ffmpeg;

pub struct HlsHandler {
    // session_id: String,
    // started: AtomicBool,
    // token: CancellationToken,
    //
    // output_ctx: Arc<Mutex<context::Output>>,
    // sources: Vec<Arc<HubSource>>,
}

impl HlsHandler {
    pub fn new() -> Self {
        pub async fn new(hub_stream: &Arc<HubStream>, base_path: &str) -> anyhow::Result<Self> {
            let token = CancellationToken::new();

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

            Ok(HlsHandler {
                started: AtomicBool::new(false),
                token,
                output_ctx: Arc::new(Mutex::new(output_ctx)),
                sources,
            })
        }
    }
}

impl SessionHandler for HlsHandler {
    type TrackContext = track_context::TrackContext;

    fn get_sources(&self) -> Vec<Arc<HubSource>> {
        todo!()
    }

    fn on_track_context(&self, idx: usize, codec: &Codec) -> Self::TrackContext {
        todo!()
    }

    fn on_video(&self, ctx: &mut Self::TrackContext, unit: &HubUnit) -> impl Future<Output=()> + Send {
        todo!()
    }

    fn on_audio(&self, ctx: &mut Self::TrackContext, unit: &HubUnit) -> impl Future<Output=()> + Send {
        todo!()
    }
}