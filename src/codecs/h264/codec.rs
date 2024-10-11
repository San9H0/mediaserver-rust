#[derive(Debug, Clone, Copy)]
pub struct H264Codec;

impl H264Codec {
    pub fn new() -> Self {
        Self
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

    pub fn samples(&self) -> u32 {
        3000
    }
}
