use crate::hubs::unit::FrameInfo;
use bytes::Bytes;

pub struct OpusRtpParser {}

impl OpusRtpParser {
    pub fn new() -> Self {
        OpusRtpParser {}
    }
}

impl OpusRtpParser {
    pub fn parse(&self, data: Bytes) -> Option<(Vec<Bytes>, FrameInfo)> {
        Some((vec![data], FrameInfo::default()))
    }
}
