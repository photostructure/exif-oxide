//! exif-oxide: High-performance Rust implementation of ExifTool
//!
//! This library provides metadata extraction from image files with ExifTool compatibility.
//! The architecture uses runtime registries for PrintConv/ValueConv implementations to avoid
//! code generation bloat while maintaining flexible extensibility.
//!
//! ## Testing
//!
//! This crate includes both unit tests and integration tests:
//!
//! - **Unit tests**: Always available, test individual components in isolation
//! - **Integration tests**: Require the `integration-tests` feature flag and external test assets
//!
//! To run all tests including integration tests:
//! ```bash
//! cargo test --features integration-tests
//! ```
//!
//! Integration tests compare our output against ExifTool reference data and require
//! test images and the ExifTool submodule to be available. They are automatically
//! excluded from published crates to keep package size manageable.

pub mod composite_tags;
pub mod examples;
pub mod exif;
pub mod expressions;
pub mod file_detection;
pub mod file_types;
pub mod formats;
pub mod generated;

pub mod implementations;
pub mod processor_registry;
pub mod raw;
pub mod registry;
pub mod stubs;
pub mod tiff_types;
pub mod tiff_utils;
pub mod types;
pub mod value_extraction;
pub mod xmp;

pub use file_detection::{FileDetectionError, FileTypeDetectionResult, FileTypeDetector};
pub use generated::*;
pub use registry::Registry;
pub use types::{ExifData, ExifError, TagValue};

pub use generated::COMPOSITE_TAG_LOOKUP as COMPOSITE_TAG_BY_NAME;

// Initialize all conversion implementations when library is loaded
use std::sync::LazyLock;
static _INIT: LazyLock<()> = LazyLock::new(|| {
    implementations::register_all_conversions();
});

/// Ensure conversions are registered (call this before using the library)
pub fn init() {
    LazyLock::force(&_INIT);
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
    let mut exif_data = formats::extract_metadata(path, false, false)?;

    // Prepare for serialization (converts TagEntry to legacy format with PrintConv)
    exif_data.prepare_for_serialization(None);

    // Convert ExifData to JSON
    let json = serde_json::to_value(&exif_data)
        .map_err(|e| ExifError::ParseError(format!("Failed to serialize to JSON: {e}")))?;

    Ok(json)
}
