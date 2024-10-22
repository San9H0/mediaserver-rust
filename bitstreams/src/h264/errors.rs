use thiserror::Error;

#[derive(Debug, Error)]
pub enum H264Error {
    #[error("should be SPS NAL unit, got {0}")]
    SPSInvalidNalUnitTyp(u8),
}
