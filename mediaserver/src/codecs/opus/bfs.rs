use crate::hubs::unit::HubUnit;

pub fn make_packet(unit: &HubUnit) -> Option<crate::utils::packet::packet::Packet> {
    let mut pkt = crate::utils::packet::packet::Packet::new();
    pkt.set_payload(unit.payload.clone());
    Some(pkt)
}
