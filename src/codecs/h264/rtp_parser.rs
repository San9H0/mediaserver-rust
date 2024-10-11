use crate::codecs::h264::config::Config;
use crate::codecs::h264::format;
use crate::codecs::h264::format::NALUType;
use crate::codecs::RtpParser;
use crate::hubs::unit::FrameInfo;
use byteorder::{BigEndian, ByteOrder};
use bytes::Bytes;

pub struct H264RtpParser {
    sps: Option<Bytes>,
    pps: Option<Bytes>,
    fragments: Vec<u8>,
}

impl H264RtpParser {
    pub fn new() -> Self {
        H264RtpParser {
            fragments: vec![],
            sps: None,
            pps: None,
        }
    }

    pub fn parse(&mut self, data: Bytes) -> Option<(Vec<Bytes>, FrameInfo)> {
        let mut flag = 0;
        let result: Vec<Bytes> = self
            .parse_h264(data)?
            .into_iter()
            .filter(|payload| {
                let nalu_type = NALUType::from_byte(payload[0]);
                if nalu_type == NALUType::IDR {
                    flag = 1;
                }
                if nalu_type == NALUType::SPS {
                    self.sps = Some(payload.clone());
                    self.pps = None
                } else if nalu_type == format::NALUType::PPS {
                    self.pps = Some(payload.clone());
                }
                !matches!(
                    nalu_type,
                    NALUType::SEI | NALUType::AccessUnitDelimiter | NALUType::FillerData
                )
            })
            .collect();
        Some((result, FrameInfo { flag }))
    }

    fn parse_h264(&mut self, data: Bytes) -> Option<Vec<Bytes>> {
        if data.len() < 1 {
            log::warn!("short packet");
            return None;
        }
        let nalu_type = NALUType::from_byte(data[0]);
        match nalu_type {
            n if (1..=23).contains(&(nalu_type as u8)) => Some(vec![data]),
            NALUType::STAPA => {
                let mut units = Vec::new();
                let mut offset = 1;
                while offset < data.len() {
                    if data.len() < offset + 2 {
                        log::warn!("stapa short packet 1");
                        return None;
                    }
                    let nalu_size = BigEndian::read_u16(&data[offset..]) as usize;
                    offset += 2;

                    if data.len() < offset + nalu_size {
                        log::warn!("stapa short packet 2");
                        return None;
                    }
                    let bytes_data = Bytes::copy_from_slice(&data[offset..offset + nalu_size]);
                    units.push(bytes_data);
                    offset += nalu_size;
                }
                Some(units)
            }
            NALUType::FUA => {
                if data.len() < 2 {
                    log::warn!("fua short packet");
                    return None;
                }
                let s = (data[1] & 0x80) >> 7;
                let e = (data[1] & 0x40) >> 6;
                let frag_nalu_type = data[1] & 0x1F;

                if s != 0 {
                    let header = frag_nalu_type | (data[1] & 0xE0);
                    self.fragments.clear();
                    self.fragments.push(header);
                    self.fragments.extend_from_slice(&data[2..]);
                } else {
                    if !self.fragments.is_empty() {
                        self.fragments.extend_from_slice(&data[2..]);
                    }
                }

                if e != 0 {
                    if self.fragments.is_empty() {
                        log::warn!("fragmented packet");
                        return None;
                    }
                    let v = vec![Bytes::from(self.fragments.clone())];
                    return Some(v);
                }
                None
            }
            _ => {
                log::warn!("unsupported nalu type {:?}", nalu_type);
                None
            }
        }
    }
}
