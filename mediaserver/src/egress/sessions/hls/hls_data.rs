use std::mem::ManuallyDrop;
use ffmpeg_next::format::context;

pub struct HlsData {
    data: bytes::Bytes,
}

impl HlsData {
    pub(crate) fn from(data: &[u8]) -> Self {
        Self {
            data: bytes::Bytes::copy_from_slice(data),
        }
    }
}
