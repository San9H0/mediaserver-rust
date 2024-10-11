use crate::hubs::stream::HubStream;
use ffmpeg_next as ffmpeg;
use std::sync::Arc;

pub struct RecordFileSession {
    hub_stream: Arc<HubStream>,
}

impl RecordFileSession {
    pub async fn new(hub_stream: Arc<HubStream>) -> Arc<Self> {
        let whip_session = Self { hub_stream };
        Arc::new(whip_session)
    }

    pub async fn init(self: Arc<Self>) -> anyhow::Result<()> {
        // let input_filename = "test.mp4";
        // let name_filename = "output.mp4";
        //
        // let mut input_ctx = ffmpeg::format::input(&input_filename)?;
        //
        // let mut packet = ffmpeg::codec::packet::Packet::empty();
        //
        // loop {
        //     packet.read(&mut input_ctx)?;
        //     println!("stream_index:{}", packet.stream());
        // }

        // for stream in input_ctx.streams() {}

        // let mut output_ctx = ffmpeg::format::output(&name_filename)?;
        //
        // let codec = ffmpeg::codec::encoder::find(ffmpeg::codec::Id::H264)
        //     .ok_or(ffmpeg::Error::EncoderNotFound)?;
        // let codec_ctx = ffmpeg::codec::context::Context::new_with_codec(codec);
        // let mut encoder = codec_ctx.encoder().video()?;
        // encoder.set_height(1280);
        // encoder.set_width(720);
        // encoder.set_format(ffmpeg::format::Pixel::YUV420P);
        // encoder.set_time_base(ffmpeg::Rational::new(1, 30));
        //
        // let stream = output_ctx.add_stream_with(&encoder)?;

        Ok(())
    }
}
