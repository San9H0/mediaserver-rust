use thiserror::Error;

#[derive(Debug, Error)]
pub enum BitReaderError {
    #[error("Invalid parameter: {0}")]
    InvalidParameter(usize),
    #[error("End Of Stream")]
    EndOfStream,
    #[error("Invalid Type Cast")]
    InvalidTypeCast,
}
