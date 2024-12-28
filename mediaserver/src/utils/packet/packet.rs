use crate::utils::rescale;

#[derive(Debug, Clone)]
pub struct Packet {
    pub stream_index: usize,
    pub pkt: Option<i64>,
    pub dts: Option<i64>,
    pub time_base: rescale::rescale::Rational,
    pub duration: Option<i64>,
    pub flag: i32,
    pub payload: bytes::Bytes,
}

impl Packet {
    pub fn new() -> Self {
        Self {
            stream_index: 0,
            pkt: None,
            dts: None,
            duration: None,
            payload: bytes::Bytes::new(),
            time_base: rescale::rescale::Rational::new(0, 0),
            flag: 0,
        }
    }
    pub fn set_payload(&mut self, payload: bytes::Bytes) {
        self.payload = payload;
    }
    pub fn set_stream(&mut self, index: usize) {
        self.stream_index = index;
    }
    pub fn set_pts(&mut self, value: Option<i64>) {
        self.pkt = value;
    }
    pub fn set_dts(&mut self, value: Option<i64>) {
        self.dts = value;
    }
    pub fn set_time_base(&mut self, value: rescale::rescale::Rational) {
        self.time_base = value;
    }
    pub fn time_base(&self) -> rescale::rescale::Rational {
        self.time_base.clone()
    }
    pub fn set_duration(&mut self, value: Option<i64>) {
        self.duration = value;
    }
    pub fn duration(&self) -> i64 {
        if let Some(duration) = self.duration {
            duration
        } else {
            0
        }
    }
    pub fn set_flags(&mut self, value: i32) {
        self.flag = value;
    }
    pub fn data(&self) -> Option<&[u8]> {
        if self.payload.is_empty() {
            None
        } else {
            Some(&self.payload)
        }
    }
}
