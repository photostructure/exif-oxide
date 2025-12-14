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

pub mod compat;
pub mod composite_tags;
pub mod core;
pub mod examples;
pub mod exif;
pub mod file_detection;
pub mod file_types;
pub mod fmt;
pub mod formats;
pub mod generated;
pub mod hash;

pub mod implementations;
pub mod processor_registry;
pub mod raw;
pub mod registry;
pub mod runtime;
pub mod tiff_types;
pub mod tiff_utils;
pub mod types;
pub mod utils;
pub mod value_extraction;
pub mod xmp;

pub use file_detection::{FileDetectionError, FileTypeDetectionResult, FileTypeDetector};
pub use generated::*;
pub use hash::{ImageDataHasher, ImageHashType};
pub use registry::Registry;
pub use types::{ExifData, ExifError, FilterOptions, TagValue};

// TODO P07: COMPOSITE_TAG_LOOKUP doesn't exist yet, using lookup_composite_tag function instead
// pub use generated::COMPOSITE_TAG_LOOKUP as COMPOSITE_TAG_BY_NAME;

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
    let mut exif_data = formats::extract_metadata(path, false, false, None)?;

    // Prepare for serialization (converts TagEntry to legacy format with PrintConv)
    exif_data.prepare_for_serialization(None);

    // Convert ExifData to JSON
    let json = serde_json::to_value(&exif_data)
        .map_err(|e| ExifError::ParseError(format!("Failed to serialize to JSON: {e}")))?;

    Ok(json)
}

/// Extract metadata from a file with tag filtering and return it as JSON
///
/// This function provides the same functionality as the CLI, allowing programmatic
/// access to ExifTool-style tag filtering and value formatting control.
///
/// # Examples
///
/// ```no_run
/// use exif_oxide::{extract_metadata_json_with_filter, FilterOptions};
/// use std::collections::HashSet;
///
/// // Extract only MIMEType tag
/// let filter = FilterOptions::tags_only(vec!["MIMEType".to_string()]);
/// let result = extract_metadata_json_with_filter("image.jpg", Some(filter))?;
///
/// // Extract Orientation with numeric value (like -Orientation#)
/// let mut numeric_tags = HashSet::new();
/// numeric_tags.insert("Orientation".to_string());
/// let filter = FilterOptions {
///     requested_tags: vec!["Orientation".to_string()],
///     requested_groups: vec![],
///     group_all_patterns: vec![],
///     extract_all: false,
///     numeric_tags,
///     glob_patterns: vec![],
///     ..Default::default()
/// };
/// let result = extract_metadata_json_with_filter("image.jpg", Some(filter))?;
///
/// // Extract all EXIF group tags (like -EXIF:all)
/// let filter = FilterOptions {
///     requested_tags: vec![],
///     requested_groups: vec![],
///     group_all_patterns: vec!["EXIF:all".to_string()],
///     extract_all: false,
///     numeric_tags: HashSet::new(),
///     glob_patterns: vec![],
///     ..Default::default()
/// };
/// let result = extract_metadata_json_with_filter("image.jpg", Some(filter))?;
///
/// // Extract GPS tags with wildcard (like -GPS*)
/// let filter = FilterOptions {
///     requested_tags: vec![],
///     requested_groups: vec![],
///     group_all_patterns: vec![],
///     extract_all: false,
///     numeric_tags: HashSet::new(),
///     glob_patterns: vec!["GPS*".to_string()],
///     ..Default::default()
/// };
/// let result = extract_metadata_json_with_filter("image.jpg", Some(filter))?;
/// # Ok::<(), exif_oxide::ExifError>(())
/// ```
pub fn extract_metadata_json_with_filter(
    file_path: &str,
    filter_options: Option<FilterOptions>,
) -> Result<Value, ExifError> {
    // Ensure conversions are registered
    init();

    // Use the existing extract_metadata function from formats module
    let path = Path::new(file_path);
    let mut exif_data = formats::extract_metadata(path, false, false, filter_options.clone())?;

    // Prepare for serialization with numeric tags if specified
    let numeric_tags_ref = filter_options.as_ref().and_then(|f| {
        if f.numeric_tags.is_empty() {
            None
        } else {
            Some(&f.numeric_tags)
        }
    });

    exif_data.prepare_for_serialization(numeric_tags_ref);

    // Convert ExifData to JSON
    let json = serde_json::to_value(&exif_data)
        .map_err(|e| ExifError::ParseError(format!("Failed to serialize to JSON: {e}")))?;

    Ok(json)
}

/// Extract metadata from a file with tag filtering and return structured data
///
/// This function provides direct access to the TagEntry structure without JSON conversion,
/// allowing for more efficient programmatic processing of metadata.
///
/// # Examples
///
/// ```no_run
/// use exif_oxide::{extract_metadata_with_filter, FilterOptions};
/// use std::path::Path;
///
/// // Extract all metadata
/// let result = extract_metadata_with_filter(Path::new("image.jpg"), None)?;
/// println!("Found {} tags", result.tags.len());
///
/// // Extract only File group tags for performance
/// let filter = FilterOptions::groups_only(vec!["File".to_string()]);
/// let result = extract_metadata_with_filter(Path::new("image.jpg"), Some(filter))?;
/// # Ok::<(), exif_oxide::ExifError>(())
/// ```
pub fn extract_metadata_with_filter(
    file_path: &Path,
    filter_options: Option<FilterOptions>,
) -> Result<ExifData, ExifError> {
    // Ensure conversions are registered
    init();

    // Use the existing extract_metadata function from formats module
    formats::extract_metadata(file_path, false, false, filter_options)
}
