//! Error types for exif-oxide

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid JPEG format: {0}")]
    InvalidJpeg(String),
    
    #[error("Invalid EXIF data: {0}")]
    InvalidExif(String),
    
    #[error("EXIF data not found")]
    NoExif,
    
    #[error("Tag not found: 0x{0:04X}")]
    TagNotFound(u16),
    
    #[error("Invalid tag format: expected {expected}, got {actual}")]
    InvalidFormat { expected: String, actual: String },
    
    #[error("Invalid UTF-8 in string")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
    
    #[error("Value out of range")]
    OutOfRange,
}