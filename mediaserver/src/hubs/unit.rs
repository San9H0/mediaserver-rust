use webrtc::rtp::packet::Packet;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HubUnit {
    pub payload: bytes::Bytes,
    pub pts: u32,
    pub dts: u32,
    pub duration: u32,
    pub timebase: u32,
    pub marker: bool,
    pub frame_info: FrameInfo,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FrameInfo {
    pub flag: i32,
}
