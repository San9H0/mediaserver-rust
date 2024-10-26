use ffmpeg_next as ffmpeg;
use crate::codecs::h264::format::NALUType;
use crate::hubs::unit::HubUnit;

pub fn make_packet_with_avcc(unit: &HubUnit) -> Option<ffmpeg::packet::Packet>{
    let nalu_type = NALUType::from_byte(unit.payload[0]);
    if nalu_type == NALUType::SPS || nalu_type == NALUType::PPS {
        return None;
    }

    let data_len = unit.payload.len() as u32;
    let length_prefix = data_len.to_be_bytes();

    let mut pkt = ffmpeg::packet::Packet::new(4 + unit.payload.len());
    if let Some(data_mut) = pkt.data_mut() {
        data_mut[..4].copy_from_slice(&length_prefix);
        data_mut[4..].copy_from_slice(&unit.payload);
    };
    Some(pkt)
}