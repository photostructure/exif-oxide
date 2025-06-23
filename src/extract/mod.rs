//! Image extraction module for thumbnails and preview images

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Exif.pm"]
#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Canon.pm"]

pub mod preview;
pub mod thumbnail;

use crate::core::ifd::ParsedIfd;
use crate::error::Result;

/// Extract thumbnail image from IFD1 if available
pub fn extract_thumbnail(ifd: &ParsedIfd, original_data: &[u8]) -> Result<Option<Vec<u8>>> {
    thumbnail::extract_thumbnail(ifd, original_data)
}

/// Extract preview images from maker notes (Canon-specific)
pub fn extract_canon_preview(ifd: &ParsedIfd, original_data: &[u8]) -> Result<Option<Vec<u8>>> {
    preview::extract_canon_preview(ifd, original_data)
}

/// Extract the largest available preview image
pub fn extract_largest_preview(ifd: &ParsedIfd, original_data: &[u8]) -> Result<Option<Vec<u8>>> {
    // Try Canon preview first (usually larger), then thumbnail
    if let Ok(Some(preview)) = extract_canon_preview(ifd, original_data) {
        Ok(Some(preview))
    } else {
        extract_thumbnail(ifd, original_data)
    }
}

/// Validate that extracted data is a valid JPEG
pub fn validate_jpeg(data: &[u8]) -> bool {
    if data.len() < 4 {
        return false;
    }

    // Check for JPEG SOI marker (0xFFD8)
    let has_soi = data[0] == 0xFF && data[1] == 0xD8;

    // Check for JPEG EOI marker (0xFFD9) at the end
    let has_eoi = data.len() >= 2 && data[data.len() - 2] == 0xFF && data[data.len() - 1] == 0xD9;

    has_soi && has_eoi
}
