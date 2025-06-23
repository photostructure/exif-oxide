//! DateTime intelligence module for exif-oxide
//!
//! This module provides sophisticated datetime handling with timezone inference,
//! multi-source validation, and manufacturer quirk handling.
//!
//! It ports the battle-tested heuristics from exiftool-vendored.js to provide:
//! - GPS coordinate-based timezone inference
//! - UTC timestamp delta calculations  
//! - Manufacturer-specific datetime quirks
//! - Multi-source datetime validation and prioritization

#![doc = "EXIFTOOL-SOURCE: vendored/exiftool-vendored.js/src/ExifDateTime.ts"]
#![doc = "EXIFTOOL-SOURCE: vendored/exiftool-vendored.js/src/Timezones.ts"]
#![doc = "EXIFTOOL-SOURCE: vendored/exiftool-vendored.js/docs/DATES.md"]

pub mod extractor;
pub mod gps_timezone;
pub mod intelligence;
pub mod parser;
pub mod quirks;
pub mod types;
pub mod utc_delta;

pub use intelligence::DateTimeIntelligence;
pub use types::*;

use crate::error::Result;

/// Extract datetime intelligence from EXIF and XMP data
///
/// This is the main entry point for datetime processing that applies all
/// available heuristics to infer timezone information and resolve the best
/// capture datetime.
///
/// # Example
/// ```rust,no_run
/// use exif_oxide::datetime::extract_datetime_intelligence;
/// use std::collections::HashMap;
///
/// // Mock EXIF data
/// let mut exif_data = HashMap::new();
/// exif_data.insert(0x9003, "2024:03:15 14:30:00".to_string());  // DateTimeOriginal
/// exif_data.insert(0x10F, "Canon".to_string());                  // Make
///
/// let intelligence = extract_datetime_intelligence(&exif_data, None)?;
/// if let Some(resolved) = intelligence {
///     println!("Capture time: {}", resolved.datetime);
///     println!("Timezone: {:?}", resolved.datetime.local_offset);
///     println!("Confidence: {:.2}", resolved.confidence);
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn extract_datetime_intelligence(
    exif_data: &std::collections::HashMap<u16, String>,
    xmp_data: Option<&crate::xmp::types::XmpMetadata>,
) -> Result<Option<ResolvedDateTime>> {
    let engine = DateTimeIntelligence::new();
    let collection = extractor::DateTimeExtractor::extract_all_datetimes(exif_data, xmp_data)?;

    if collection.is_empty() {
        return Ok(None);
    }

    let camera_info = CameraInfo::from_exif(exif_data);
    let resolved = engine.resolve_capture_datetime(&collection, &camera_info)?;

    Ok(Some(resolved))
}
