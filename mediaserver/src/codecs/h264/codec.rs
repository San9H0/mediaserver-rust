use std::ptr;
use crate::codecs::h264::config::Config;
use ffmpeg_next as ffmpeg;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;

#[derive(Debug, Clone)]
pub struct H264Codec {
    config: Config,
}

impl H264Codec {
    pub fn new(config: Config) -> H264Codec {
        H264Codec { config }
    }

    pub fn kind(&self) -> &'static str {
        "video"
    }

    pub fn mime_type(&self) -> &'static str {
        "video/h264"
    }

    pub fn clock_rate(&self) -> u32 {
        90000
    }

    pub fn channels(&self) -> u16 {
        0
    }

    pub fn samples(&self) -> u32 {
        3000
    }

    pub fn rtp_codec_capability(&self) -> RTCRtpCodecCapability {
        RTCRtpCodecCapability {
            mime_type: self.mime_type().to_string(),
            clock_rate: self.clock_rate(),
            channels: self.channels(),
            sdp_fmtp_line: "level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=42001f"
                .to_string(),
            rtcp_feedback: vec![],
        }
    }

    pub fn set_av_video(
        &self,
        video: &mut ffmpeg::codec::encoder::video::Video,
    ) -> anyhow::Result<()> {

        video.set_width(self.config.sps.width());
        video.set_height(self.config.sps.height());
        video.set_format(ffmpeg::format::Pixel::YUV420P);
        video.set_time_base(ffmpeg::Rational::new(1, 30));


        unsafe {
            let extradata = self.config.extradata();
            let extradata_ptr = ffmpeg_sys_next::av_malloc(extradata.len()) as *mut u8;
            ptr::copy_nonoverlapping(extradata.as_ptr(), extradata_ptr, extradata.len());
            (*video.as_mut_ptr()).extradata = extradata_ptr;
            (*video.as_mut_ptr()).extradata_size = extradata.len() as i32;
        }

        Ok(())
    }
}
