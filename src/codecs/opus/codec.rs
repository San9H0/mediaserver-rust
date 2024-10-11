#[derive(Debug, Clone, Copy)]
pub struct OpusCodec;

impl OpusCodec {
    pub fn new() -> Self {
        Self
    }

    pub fn kind(&self) -> &'static str {
        "audio"
    }

    pub fn mime_type(&self) -> &'static str {
        "audio/opus"
    }

    pub fn clock_rate(&self) -> u32 {
        48000
    }

    pub fn samples(&self) -> u32 {
        960
    }
}
