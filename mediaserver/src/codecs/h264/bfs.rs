use crate::codecs::h264::format::NALUType;
use crate::hubs::unit::HubUnit;
use crate::utils;
use bytes::BytesMut;

pub fn make_packet_with_avcc(unit: &HubUnit) -> Option<crate::utils::packet::packet::Packet> {
    let nalu_type = NALUType::from_byte(unit.payload[0]);
    if nalu_type == NALUType::SPS || nalu_type == NALUType::PPS {
        return None;
    }

    let data_len = unit.payload.len() as u32;
    let length_prefix = data_len.to_be_bytes();
    let mut pkt = utils::packet::packet::Packet::new();

    // BytesMut 초기화
    let mut bytes_mut = BytesMut::with_capacity(4 + unit.payload.len());
    bytes_mut.extend_from_slice(&length_prefix);
    bytes_mut.extend_from_slice(&unit.payload);

    pkt.set_payload(bytes_mut.freeze());
    Some(pkt)
}
