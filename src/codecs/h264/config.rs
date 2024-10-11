use anyhow::anyhow;
use bytes::{Bytes, BytesMut};
use bytesio::bytes_reader::BytesReader;
use h264_decoder::sps::SpsParser;

pub struct Config {}

impl Config {
    pub fn new() -> Self {
        Config {}
    }
    pub fn from_bytes(&self, sps: Option<Bytes>, pps: Option<Bytes>) -> anyhow::Result<()> {
        // let sps = BytesMut::from(sps.ok_or_else(|| anyhow::anyhow!("sps is None"))?);
        // let pps = BytesMut::from(pps.ok_or_else(|| anyhow::anyhow!("pps is None")));
        //
        // let mut parser = SpsParser::new(BytesReader::new(sps));
        // if let Err(err) = parser.parse() {
        //     return Err(anyhow!(err));
        // }

        Ok(())
    }
}
