use crate::hubs::unit::FrameInfo;
use bytes::Bytes;

pub mod codec;
pub mod h264;
pub mod opus;
pub mod rtp_packetizer;
pub mod rtp_parser;
pub mod rtp_payloader;

trait RtpParser {
    fn parse(&mut self, data: Bytes) -> Option<(Vec<Bytes>, FrameInfo)>;
}
