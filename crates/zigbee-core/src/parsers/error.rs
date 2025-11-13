use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Incomplete data: need {needed} more bytes")]
    Incomplete { needed: usize },
    
    #[error("Invalid frame: {0}")]
    Invalid(String),
    
    #[error("Unsupported frame type: {0}")]
    UnsupportedFrameType(u8),
    
    #[error("Invalid address mode: {0}")]
    InvalidAddressMode(u8),
    
    #[error("FCS check failed")]
    FcsCheckFailed,
    
    #[error("Frame too short: expected at least {expected}, got {actual}")]
    FrameTooShort { expected: usize, actual: usize },
    
    #[error("Security not supported yet")]
    SecurityNotSupported,
    
    #[error("Parse error: {0}")]
    ParseFailed(String),
}

pub type ParseResult<T> = Result<T, ParseError>;