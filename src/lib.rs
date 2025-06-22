//! exif-oxide - High-performance EXIF metadata extraction
//!
//! This crate provides fast, safe parsing of EXIF metadata from image files,
//! with a focus on compatibility with Phil Harvey's ExifTool.

pub mod core;
pub mod error;
pub mod extract;
pub mod maker;
pub mod tables;
pub mod xmp;

use error::Result;
use std::path::Path;

/// Basic EXIF data for Spike 1
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

/// Read basic EXIF data from a JPEG file
///
/// This is the Spike 1 implementation that extracts only Make, Model, and Orientation.
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
    use std::fs::File;

    let mut file = File::open(path)?;
    let exif_segment = core::jpeg::find_exif_segment(&mut file)?.ok_or(error::Error::NoExif)?;
    let ifd = core::ifd::IfdParser::parse(exif_segment.data)?;

    Ok(BasicExif {
        make: ifd.get_string(0x10F)?,
        model: ifd.get_string(0x110)?,
        orientation: ifd.get_u16(0x112)?,
    })
}

/// Extract thumbnail image from EXIF data
///
/// Extracts the thumbnail image stored in IFD1 of the EXIF data.
/// Returns None if no thumbnail is available.
///
/// # Example
/// ```no_run
/// use exif_oxide::extract_thumbnail;
///
/// if let Some(thumbnail) = extract_thumbnail("photo.jpg").unwrap() {
///     std::fs::write("thumbnail.jpg", thumbnail).unwrap();
/// }
/// ```
pub fn extract_thumbnail<P: AsRef<Path>>(path: P) -> Result<Option<Vec<u8>>> {
    use std::fs::{read, File};

    let mut file = File::open(&path)?;
    let exif_segment = core::jpeg::find_exif_segment(&mut file)?.ok_or(error::Error::NoExif)?;
    let ifd = core::ifd::IfdParser::parse(exif_segment.data)?;

    // Read the entire file for thumbnail extraction
    let original_data = read(path)?;

    extract::extract_thumbnail(&ifd, &original_data)
}

/// Extract Canon preview image from maker notes
///
/// Extracts the larger preview image stored in Canon maker notes.
/// Returns None if no Canon preview is available.
///
/// # Example
/// ```no_run
/// use exif_oxide::extract_canon_preview;
///
/// if let Some(preview) = extract_canon_preview("canon_photo.jpg").unwrap() {
///     std::fs::write("preview.jpg", preview).unwrap();
/// }
/// ```
pub fn extract_canon_preview<P: AsRef<Path>>(path: P) -> Result<Option<Vec<u8>>> {
    use std::fs::{read, File};

    let mut file = File::open(&path)?;
    let exif_segment = core::jpeg::find_exif_segment(&mut file)?.ok_or(error::Error::NoExif)?;
    let ifd = core::ifd::IfdParser::parse(exif_segment.data)?;

    // Read the entire file for preview extraction
    let original_data = read(path)?;

    extract::extract_canon_preview(&ifd, &original_data)
}

/// Extract the largest available preview image
///
/// Attempts to extract the largest preview image available, trying Canon
/// preview first (if available), then falling back to EXIF thumbnail.
///
/// # Example
/// ```no_run
/// use exif_oxide::extract_largest_preview;
///
/// if let Some(preview) = extract_largest_preview("photo.jpg").unwrap() {
///     std::fs::write("largest_preview.jpg", preview).unwrap();
/// }
/// ```
pub fn extract_largest_preview<P: AsRef<Path>>(path: P) -> Result<Option<Vec<u8>>> {
    use std::fs::{read, File};

    let mut file = File::open(&path)?;
    let exif_segment = core::jpeg::find_exif_segment(&mut file)?.ok_or(error::Error::NoExif)?;
    let ifd = core::ifd::IfdParser::parse(exif_segment.data)?;

    // Read the entire file for preview extraction
    let original_data = read(path)?;

    extract::extract_largest_preview(&ifd, &original_data)
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
