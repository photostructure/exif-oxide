//! exif-oxide - High-performance EXIF metadata extraction
//!
//! This crate provides fast, safe parsing of EXIF metadata from image files,
//! with a focus on compatibility with Phil Harvey's ExifTool.

pub mod binary;
pub mod core;
pub mod detection;
pub mod error;
pub mod maker;
pub mod tables;
pub mod xmp;

// Re-export commonly used types
pub use core::{MetadataCollection, MetadataSegment, MetadataType};

use error::Result;
use std::path::Path;

/// Basic EXIF data
#[derive(Debug, Clone, PartialEq)]
pub struct BasicExif {
    /// Camera manufacturer (EXIF tag 0x10F)
    pub make: Option<String>,

    /// Camera model (EXIF tag 0x110)
    pub model: Option<String>,

    /// Image orientation (EXIF tag 0x112)
    /// Values: 1=Normal, 6=Rotate 90 CW, 8=Rotate 270 CW, 3=Rotate 180
    pub orientation: Option<u16>,
}

/// Read basic EXIF data from any supported image file
///
/// Supports JPEG, TIFF, PNG, HEIF/HEIC, and many RAW formats.
/// Extracts Make, Model, and Orientation.
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

    Ok(BasicExif {
        make,
        model,
        orientation,
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
