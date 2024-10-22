use crate::readers::bitreader::BitReader;
use bytes::Bytes;

pub struct NalUnit<'a> {
    pub nal_ref_idc: u8,
    pub nal_unit_type: u8,
    pub(crate) reader: BitReader<'a>,
}

impl<'a> NalUnit<'a> {
    pub fn from(data: &'a [u8]) -> anyhow::Result<Self> {
        let mut reader = BitReader::new(data);
        let _ = reader.read_bits::<u8>(1)?; // forbidden_zero_bit
        let nal_ref_idc = reader.read_bits(2)?;
        let nal_unit_type = reader.read_bits(5)?;
        Ok(NalUnit {
            nal_ref_idc,
            nal_unit_type,
            reader,
        })
    }

    pub fn to_bytes(&self) -> Bytes {
        Bytes::copy_from_slice(self.reader.as_ref())
    }
}
