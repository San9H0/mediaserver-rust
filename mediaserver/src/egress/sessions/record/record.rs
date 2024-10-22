use crate::hubs::stream::HubStream;
use crate::hubs::unit::HubUnit;
use bytes::BytesMut;
use ffmpeg_next as ffmpeg;
use ffmpeg_next::format::context;
use ffmpeg_next::packet::Flags;
use ffmpeg_next::util::mathematics::Rescale;
use futures::lock::Mutex;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;
use crate::codecs::bfs::Bfs;
use crate::egress::sessions::session::Session;
use crate::hubs::source::HubSource;

pub struct RecordSession {
    hub_stream: Arc<HubStream>,
    token: CancellationToken,

    output_ctx: Arc<Mutex<context::Output>>,
}

impl RecordSession {
    pub async fn new(hub_stream: Arc<HubStream>) -> anyhow::Result<Arc<Self>> {
        let token = CancellationToken::new();
        let name_filename = "output.mp4";
        let output_ctx = ffmpeg::format::output(&name_filename)?;
        let whip_session = RecordSession {
            hub_stream,
            token,
            output_ctx: Arc::new(Mutex::new(output_ctx)),
        };
        Ok(Arc::new(whip_session))
    }

    pub fn stop(self: &Arc<Self>) {
        self.token.cancel();
    }
    pub async fn init(self: &Arc<Self>) -> anyhow::Result<()> {
        let mut output_ctx = self.output_ctx.lock().await;

        for source in self.hub_stream.get_sources().await {
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
        }

        Ok(())
    }

    pub async fn run(self: &Arc<Self>) -> anyhow::Result<()> {
        self.write_header().await?;
        let mut handles = vec![];

        for (idx, source) in self.hub_stream.get_sources().await.iter().enumerate() {
            let codec = source.get_codec().await.unwrap();
            let track = source.get_track().await;
            let sink = track.add_sink().await;
            let token = self.token.clone();
            let self_ = self.clone();
            let bfs = Bfs::new(codec.mime_type())?;
            let handle = tokio::spawn(async move {
                let stream_index = idx;
                let mut pts = 0;
                let mut dts = 0;
                let mut base_ts = 0;
                let mut set_pts = false;
                let mut keyframe = false;
                loop {
                    tokio::select! {
                        _ = token.cancelled() => {
                            break;
                        }
                        result = sink.read_unit() => {
                            let Ok(unit) = result else {
                                log::warn!("read unit failed");
                                break;
                            };

                            if codec.kind() == "video" && !keyframe {
                                if unit.frame_info.flag != 1 {
                                    continue;
                                }
                                keyframe = true;
                            }

                            if !set_pts {
                                set_pts = true;
                                base_ts = unit.pts;
                            }
                            pts = unit.pts - base_ts;
                            dts = unit.dts - base_ts;

                            let data_len = unit.payload.len() as u32;
                            let input_time_base = ffmpeg::Rational::new(1, unit.timebase as i32);
                            let output_time_base = ffmpeg::Rational::new(1, codec.clock_rate() as i32);
                            let mut pkt = bfs.make_packet(&unit);

                            pkt.set_stream(stream_index);
                            pkt.set_pts(Some((pts as i64).rescale(input_time_base, output_time_base)));
                            pkt.set_dts(Some((dts as i64).rescale(input_time_base, output_time_base)));
                            pkt.set_duration((unit.duration as i64).rescale_with(
                                input_time_base,
                                output_time_base,
                                ffmpeg::mathematics::Rounding::NearInfinity,
                            ));
                            if unit.frame_info.flag == 1 {
                                pkt.set_flags(ffmpeg::packet::Flags::KEY);
                            }
                            {
                                let mut output_ctx = self_.output_ctx.lock().await;
                                if let Err(err) = pkt.write_interleaved(&mut output_ctx) {
                                    log::warn!("failed to write packet: {}", err);
                                    continue;
                                };
                            }
                        }
                    }
                }
            });
            handles.push(handle);
        }
        // JoinHandle을 for loop에서 하나씩 await하여 모든 작업이 끝날 때까지 대기
        for handle in handles {
            match handle.await {
                Ok(_) => println!("Task completed successfully."),
                Err(e) => println!("Task failed: {:?}", e),
            }
        }

        self.write_trailer().await?;

        Ok(())
    }

    pub async fn write_header(self: &Arc<Self>) -> anyhow::Result<()> {
        let mut output_ctx = self.output_ctx.lock().await;
        output_ctx.write_header()?;

        Ok(())
    }

    pub async fn write_trailer(self: &Arc<Self>) -> anyhow::Result<()> {
        let mut output_ctx = self.output_ctx.lock().await;
        output_ctx.write_trailer()?;

        Ok(())
    }

}

// Drop 트레잇 구현
impl Drop for RecordSession {
    fn drop(&mut self) {
        println!("RecordSession is being dropped!");
    }
}
