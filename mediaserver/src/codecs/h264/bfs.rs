use ffmpeg_next as ffmpeg;
use crate::hubs::unit::HubUnit;

pub fn make_packet_with_avcc(unit: &HubUnit) -> ffmpeg::packet::Packet{
    let data_len = unit.payload.len() as u32;
    let length_prefix = data_len.to_be_bytes();

    let mut pkt = ffmpeg::packet::Packet::new(4 + unit.payload.len());
    if let Some(data_mut) = pkt.data_mut() {
        data_mut[..4].copy_from_slice(&length_prefix);
        data_mut[4..].copy_from_slice(&unit.payload);
    };
    pkt
}