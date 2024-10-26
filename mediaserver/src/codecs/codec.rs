use std::ptr::null;
use crate::codecs::h264::codec::H264Codec;
use crate::codecs::opus::codec::OpusCodec;
use anyhow::anyhow;
use ffmpeg_next as ffmpeg;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use crate::utils::types::types;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum Codec {
    Opus(OpusCodec),
    H264(H264Codec),
}

impl Codec {
    // pub fn new(mime_type: &str) -> anyhow::Result<Codec> {
    //     match mime_type.to_lowercase().as_str() {
    //         "audio/opus" => Ok(Codec::Opus(OpusCodec)),
    //         "video/h264" => Ok(Codec::H264(H264Codec)),
    //         _ => Err(anyhow!("Unsupported codec: {}", mime_type)),
    //     }
    // }

    pub fn kind(&self) -> types::MediaKind {
        match self {
            Codec::Opus(codec) => codec.kind(),
            Codec::H264(codec) => codec.kind(),
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            Codec::Opus(codec) => codec.mime_type(),
            Codec::H264(codec) => codec.mime_type(),
        }
    }

    pub fn clock_rate(&self) -> u32 {
        match self {
            Codec::Opus(codec) => codec.clock_rate(),
            Codec::H264(codec) => codec.clock_rate(),
        }
    }

    pub fn samples(&self) -> u32 {
        match self {
            Codec::Opus(codec) => codec.samples(),
            Codec::H264(codec) => codec.samples(),
        }
    }

    pub fn av_codec_id(&self) -> ffmpeg::codec::Id {
        match self {
            Codec::Opus(_) => ffmpeg::codec::Id::OPUS,
            Codec::H264(_) => ffmpeg::codec::Id::H264,
        }
    }

    pub fn rtp_codec_capability(&self) -> RTCRtpCodecCapability {
        match self {
            Codec::Opus(codec) => codec.rtp_codec_capability(),
            Codec::H264(codec) => codec.rtp_codec_capability(),
        }
    }

    pub fn set_av_audio(
        &self,
        audio: &mut ffmpeg::codec::encoder::audio::Audio,
    ) -> anyhow::Result<()> {
        match self {
            Codec::Opus(_) => {
                audio.set_rate(48000);
                audio.set_channel_layout(ffmpeg::channel_layout::ChannelLayout::default(2));
                audio.set_format(ffmpeg::format::Sample::F32(
                    ffmpeg::format::sample::Type::Packed,
                ));
                Ok(())
            }
            Codec::H264(_) => Ok(()),
        }
    }

    pub fn set_av_video(
        &self,
        video: &mut ffmpeg::codec::encoder::video::Video,
    ) -> anyhow::Result<()> {
        match self {
            Codec::Opus(_) => Ok(()),
            Codec::H264(codec) => codec.set_av_video(video),
        }
    }
}
