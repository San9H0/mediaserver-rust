use bitstreams::h264::pps::PPS;
use bitstreams::h264::sps::SPS;
use bytes::Bytes;

#[derive(Debug, Clone)]
pub struct Config {
    pub sps: SPS,
    pub pps: PPS,
}

impl Config {
    pub fn from(sps: SPS, pps: PPS) -> Config {
        Config { sps, pps }
    }

    pub fn profile(&self) -> u8 {
        self.sps.profile_idc
    }

    pub fn constraint_set(&self) -> u8 {
        self.sps.constraint_compatibility_flag
    }

    pub fn level(&self) -> u8 {
        self.sps.level_idc
    }

    pub fn width(&self) -> u32 {
        self.sps.width()
    }

    pub fn height(&self) -> u32 {
        self.sps.height()
    }

    pub fn extradata(&self) -> Bytes {
        let mut b = vec![0u8; self.sps.payload.len() + self.pps.payload.len() + 8 + 3];

        let sps = &self.sps.payload;
        let pps = &self.pps.payload;
        b[0] = 0x01;
        b[1] = sps[1];
        b[2] = sps[2];
        b[3] = sps[3];
        b[4] = 0xfc | 3; // NALU의 길이는 4비트이다. 4-1=3
        b[5] = 0xe0 | 1; // SPS 개수는 1개이다.
        b[6] = (sps.len() >> 8) as u8;
        b[7] = (sps.len() & 0xff) as u8;

        b[8..8 + sps.len()].copy_from_slice(&sps);

        b[8 + sps.len()] = 1; // PPS 개수는 1개이다.
        b[9 + sps.len()] = (pps.len() >> 8) as u8;
        b[10 + sps.len()] = (pps.len() & 0xff) as u8;

        b[11 + sps.len()..].copy_from_slice(pps);
        Bytes::from(b)
    }
}
