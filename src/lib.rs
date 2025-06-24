//! exif-oxide - High-performance EXIF metadata extraction
//!
//! This crate provides fast, safe parsing of EXIF metadata from image files,
//! with a focus on compatibility with Phil Harvey's ExifTool.

pub mod binary;
pub mod core;
pub mod datetime;
pub mod detection;
pub mod error;
pub mod maker;
pub mod tables;
pub mod xmp;

// Re-export commonly used types
pub use core::{MetadataCollection, MetadataSegment, MetadataType};

use datetime::ResolvedDateTime;
use error::Result;
use std::collections::HashMap;
use std::path::Path;

/// Basic EXIF data with datetime intelligence
#[derive(Debug, Clone, PartialEq)]
pub struct BasicExif {
    /// Camera manufacturer (EXIF tag 0x10F)
    pub make: Option<String>,

    /// Camera model (EXIF tag 0x110)
    pub model: Option<String>,

    /// Image orientation (EXIF tag 0x112)
    /// Values: 1=Normal, 6=Rotate 90 CW, 8=Rotate 270 CW, 3=Rotate 180
    pub orientation: Option<u16>,

    /// Resolved datetime with timezone intelligence
    pub resolved_datetime: Option<ResolvedDateTime>,
}

/// Read basic EXIF data from any supported image file
///
/// Supports JPEG, TIFF, PNG, HEIF/HEIC, and many RAW formats.
/// Extracts Make, Model, Orientation, and datetime intelligence.
///
/// # Example
/// ```no_run
/// use exif_oxide::read_basic_exif;
///
/// let exif = read_basic_exif("photo.jpg").unwrap();
/// println!("Camera: {} {}",
///     exif.make.as_deref().unwrap_or("Unknown"),
///     exif.model.as_deref().unwrap_or("Unknown")
/// );
/// ```
pub fn read_basic_exif<P: AsRef<Path>>(path: P) -> Result<BasicExif> {
    let metadata_segment = core::find_metadata_segment(&path)?.ok_or(error::Error::NoExif)?;
    let ifd = core::ifd::IfdParser::parse(metadata_segment.data)?;

    // Extract basic fields
    let make = ifd.get_string(0x10F)?;
    let model = ifd.get_string(0x110)?;
    let orientation = ifd.get_u16(0x112)?;

    // Extract datetime intelligence
    let resolved_datetime = extract_datetime_intelligence(&path).unwrap_or(None);

    Ok(BasicExif {
        make,
        model,
        orientation,
        resolved_datetime,
    })
}

/// Extract XMP metadata from a JPEG file
///
/// Returns a HashMap of XMP properties in "namespace:property" format.
/// This is a Phase 1 implementation that only extracts simple properties.
///
/// # Example
/// ```no_run
/// use exif_oxide::extract_xmp_properties;
///
/// let xmp_props = extract_xmp_properties("photo.jpg").unwrap();
/// if let Some(title) = xmp_props.get("dc:title") {
///     println!("Title: {}", title);
/// }
/// ```
pub fn extract_xmp_properties<P: AsRef<Path>>(
    path: P,
) -> Result<std::collections::HashMap<String, String>> {
    xmp::extract_xmp_properties(path).map_err(|e| error::Error::XmpError(e.to_string()))
}

/// Extract datetime intelligence from EXIF metadata with timezone inference
///
/// This function applies sophisticated datetime analysis including:
/// - GPS coordinate-based timezone inference
/// - UTC timestamp delta calculations
/// - Manufacturer-specific datetime quirks
/// - Multi-source datetime validation and prioritization
///
/// Supports any file format with EXIF data.
/// Returns None if no datetime information is found, or a ResolvedDateTime
/// with confidence scoring and detailed inference information.
///
/// # Example
/// ```no_run
/// use exif_oxide::extract_datetime_intelligence;
///
/// if let Some(resolved) = extract_datetime_intelligence("photo.nef")? {
///     println!("Capture time: {} (UTC)", resolved.datetime.datetime);
///     if let Some(offset) = resolved.datetime.local_offset {
///         println!("Local timezone: {}", offset);
///     }
///     println!("Confidence: {:.1}%", resolved.confidence * 100.0);
///     for warning in &resolved.warnings {
///         println!("Warning: {:?}", warning);
///     }
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn extract_datetime_intelligence<P: AsRef<Path>>(path: P) -> Result<Option<ResolvedDateTime>> {
    let metadata_segment = core::find_metadata_segment(&path)?.ok_or(error::Error::NoExif)?;
    let ifd = core::ifd::IfdParser::parse(metadata_segment.data)?;

    // Build EXIF data HashMap for datetime extraction
    let mut exif_data = HashMap::new();

    // Extract all string tags that might contain datetime information
    for (tag_id, _tag_info) in tables::EXIF_TAGS.iter() {
        if let Ok(Some(value)) = ifd.get_string(*tag_id) {
            exif_data.insert(*tag_id, value);
        }
    }

    // Add GPS coordinates if available (convert rational to float)
    if let Ok(Some((num, den))) = ifd.get_rational(0x0002) {
        // GPSLatitude
        if den != 0 {
            let lat = num as f64 / den as f64;
            exif_data.insert(0x0002, lat.to_string());
        }
    }
    if let Ok(Some((num, den))) = ifd.get_rational(0x0004) {
        // GPSLongitude
        if den != 0 {
            let lng = num as f64 / den as f64;
            exif_data.insert(0x0004, lng.to_string());
        }
    }

    // Try to extract XMP data for additional datetime sources
    let xmp_data = match xmp::extract_xmp_properties(&path) {
        Ok(props) => {
            // Convert XMP properties to XmpMetadata structure
            let mut metadata = xmp::types::XmpMetadata::new();
            for (key, value) in props {
                // Split namespace:property format
                if let Some((namespace, property)) = key.split_once(':') {
                    metadata
                        .properties
                        .entry(namespace.to_string())
                        .or_default()
                        .insert(property.to_string(), xmp::types::XmpValue::Simple(value));
                }
            }
            Some(metadata)
        }
        Err(_) => None, // XMP extraction failed, continue without it
    };

    // Apply datetime intelligence
    datetime::extract_datetime_intelligence(&exif_data, xmp_data.as_ref())
}
