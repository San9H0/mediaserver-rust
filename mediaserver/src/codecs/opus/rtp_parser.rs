use crate::codecs::codec::Codec;
use crate::codecs::codec::Codec::Opus;
use crate::codecs::opus::codec::OpusCodec;
use crate::hubs::unit::FrameInfo;
use bytes::Bytes;
use std::future::Future;
use std::pin::Pin;

type OnCodecCallback = Box<dyn Fn(Codec) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;
pub struct OpusRtpParser {
    call_on_codec: bool,
    on_codec: OnCodecCallback,
}

impl OpusRtpParser {
    pub fn new(on_codec: OnCodecCallback) -> Self {
        OpusRtpParser {
            call_on_codec: false,
            on_codec,
        }
    }
}

impl OpusRtpParser {
    pub async fn parse(&mut self, data: Bytes) -> Option<(Vec<Bytes>, FrameInfo)> {
        if !self.call_on_codec {
            self.call_on_codec = true;
            (*self.on_codec)(Opus(OpusCodec)).await;
        }
        Some((vec![data], FrameInfo::default()))
    }
}
