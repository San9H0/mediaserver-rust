use crate::hubs::unit::HubUnit;
use ffmpeg_next as ffmpeg;

pub fn make_packet(unit: &HubUnit) -> Option<ffmpeg::packet::Packet> {
    let mut pkt = ffmpeg::packet::Packet::new(unit.payload.len());
    if let Some(data_mut) = pkt.data_mut() {
        data_mut.copy_from_slice(&unit.payload);
    };
    Some(pkt)
}
