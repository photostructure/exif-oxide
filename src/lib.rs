//! exif-oxide: High-performance Rust implementation of ExifTool
//!
//! This library provides metadata extraction from image files with ExifTool compatibility.
//! The architecture uses runtime registries for PrintConv/ValueConv implementations to avoid
//! code generation bloat while maintaining flexible extensibility.

pub mod exif;
pub mod formats;
pub mod generated;
pub mod implementations;
pub mod registry;
pub mod types;

pub use generated::*;
pub use registry::Registry;
pub use types::{ExifData, ExifError, TagValue};

// Initialize all conversion implementations when library is loaded
lazy_static::lazy_static! {
    static ref _INIT: () = {
        implementations::register_all_conversions();
    };
}

/// Ensure conversions are registered (call this before using the library)
pub fn init() {
    lazy_static::initialize(&_INIT);
}

use serde_json::Value;
use std::path::Path;

/// Extract metadata from a file and return it as JSON (matching CLI output format)
///
/// This is a high-level convenience function that matches the CLI output format,
/// making it easy to compare with ExifTool output in tests.
pub fn extract_metadata_json(file_path: &str) -> Result<Value, ExifError> {
    // Ensure conversions are registered
    init();

    // Use the existing extract_metadata function from formats module
    let path = Path::new(file_path);
    let exif_data = formats::extract_metadata(path, false)?;

    // Convert ExifData to JSON
    let json = serde_json::to_value(&exif_data)
        .map_err(|e| ExifError::ParseError(format!("Failed to serialize to JSON: {e}")))?;

    Ok(json)
}
