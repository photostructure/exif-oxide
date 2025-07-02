//! Error types for exif-oxide
//!
//! This module defines the error types used throughout the exif-oxide library,
//! following ExifTool's error classification patterns.

use thiserror::Error;

/// Error types for exif-oxide
// TODO: Enhance error types to match ExifTool's sophisticated error classification system (warnings, errors, fatal)
#[derive(Error, Debug)]
pub enum ExifError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid file format: {0}")]
    InvalidFormat(String),

    #[error("Parsing error: {0}")]
    ParseError(String),

    #[error("Unsupported feature: {0}")]
    Unsupported(String),

    #[error("Registry error: {0}")]
    Registry(String),
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, ExifError>;
