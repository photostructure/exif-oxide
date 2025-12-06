//! Error types for exif-oxide
//!
//! This module re-exports the unified ExifError type from codegen-runtime
//! and provides conversions from other error types.

use crate::file_detection::FileDetectionError;

// Re-export ExifError from codegen-runtime as the single source of truth
pub use codegen_runtime::ExifError;

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, ExifError>;

impl From<FileDetectionError> for ExifError {
    fn from(err: FileDetectionError) -> Self {
        ExifError::FileDetection(err.to_string())
    }
}
