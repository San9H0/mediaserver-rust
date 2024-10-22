use crate::codecs::codec::Codec;
use crate::codecs::codec::Codec::H264;
use crate::codecs::h264::codec::H264Codec;
use crate::codecs::h264::config::Config;
use crate::codecs::h264::format::NALUType;
use crate::hubs::unit::FrameInfo;
use anyhow::{anyhow, Error};
use bitstreams::h264::nal_unit::NalUnit;
use bitstreams::h264::pps::PPS;
use bitstreams::h264::sps::SPS;
use byteorder::{BigEndian, ByteOrder};
use bytes::Bytes;
use std::future::Future;
use std::io::Cursor;
use std::pin::Pin;
use std::rc::Rc;

pub struct MyParser {
    pub on_codec: Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()>>> + Send>,
}

impl MyParser {
    pub fn new(on_codec: Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()>>> + Send>) -> Self {
        MyParser { on_codec }
    }
}

type OnCodecCallback = Box<dyn Fn(Codec) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

pub struct H264RtpParser {
    sps: Option<Bytes>,
    pps: Option<Bytes>,
    fragments: Vec<u8>,

    sps_temp: Option<Bytes>,
    pps_temp: Option<Bytes>,

    on_codec: OnCodecCallback,
}

impl H264RtpParser {
    pub fn new(on_codec: OnCodecCallback) -> Self {
        H264RtpParser {
            fragments: vec![],
            sps: None,
            pps: None,

            sps_temp: None,
            pps_temp: None,

            on_codec,
        }
    }

    pub async fn parse(&mut self, data: Bytes) -> Option<(Vec<Bytes>, FrameInfo)> {
        let mut flag = 0;
        let result: Vec<Bytes> = self
            .parse_h264(data)?
            .into_iter()
            .filter(|payload| {
                let nalu_type = NALUType::from_byte(payload[0]);
                if nalu_type == NALUType::IDR {
                    flag = 1;
                } else if nalu_type == NALUType::SPS {
                    flag = 1;
                    self.sps_temp = Some(payload.clone());
                    self.pps_temp = None;
                    return false; // drop
                } else if nalu_type == NALUType::PPS {
                    flag = 1;
                    self.pps_temp = Some(payload.clone());
                    return false; // drop
                }
                !matches!(
                    nalu_type,
                    NALUType::SEI | NALUType::AccessUnitDelimiter | NALUType::FillerData
                )
            })
            .collect();

        if self.sps_temp.is_none() || self.pps_temp.is_none() {
            return None;
        }

        if self.sps != self.sps_temp || self.pps != self.pps_temp {
            self.sps = self.sps_temp.clone();
            self.pps = self.pps_temp.clone();

            let sps_buffer = self.sps_temp.clone().unwrap();
            let mut sps_nal_unit = NalUnit::from(sps_buffer.as_ref()).unwrap();
            let sps = SPS::from(&mut sps_nal_unit).unwrap();

            let pps_buffer = self.pps_temp.clone().unwrap();
            let mut pps_nal_unit = NalUnit::from(pps_buffer.as_ref()).unwrap();
            let pps = PPS::from(&mut pps_nal_unit).unwrap();

            let codec = H264Codec::new(Config::from(sps, pps));
            (*self.on_codec)(H264(codec)).await;

            flag = 1;
        }

        Some((result, FrameInfo { flag }))
    }

    fn parse_h264(&mut self, data: Bytes) -> Option<Vec<Bytes>> {
        if data.len() < 1 {
            log::warn!("short packet");
            return None;
        }
        let nalu_type = NALUType::from_byte(data[0]);
        match nalu_type {
            _ if (1..=23).contains(&(nalu_type as u8)) => Some(vec![data]),
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
                    let header = frag_nalu_type | (data[0] & 0xE0);
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
