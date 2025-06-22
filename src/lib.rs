//! exif-oxide - High-performance EXIF metadata extraction
//!
//! This crate provides fast, safe parsing of EXIF metadata from image files,
//! with a focus on compatibility with Phil Harvey's ExifTool.

pub mod core;
pub mod error;

use std::path::Path;
use error::Result;

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
    let jpeg_data = core::jpeg::JpegReader::read_file(path.as_ref())?;
    let exif_segment = jpeg_data.find_exif_segment()?;
    let ifd = core::ifd::IfdParser::parse(exif_segment)?;
    
    Ok(BasicExif {
        make: ifd.get_string(0x10F)?,
        model: ifd.get_string(0x110)?,
        orientation: ifd.get_u16(0x112)?,
    })
}