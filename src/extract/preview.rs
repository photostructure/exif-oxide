//! Canon maker note preview image extraction

use crate::core::ifd::ParsedIfd;
use crate::error::{Error, Result};

/// Extract Canon preview image from maker notes
///
/// Canon stores preview images in maker notes with:
/// - Tag 0xB605 (PreviewImageStart): Offset to preview data  
/// - Tag 0xB602 (PreviewImageLength): Length of preview data
///
/// These tags are stored directly in maker notes.
pub fn extract_canon_preview(ifd: &ParsedIfd, original_data: &[u8]) -> Result<Option<Vec<u8>>> {
    // Canon preview tags from our generated table
    let preview_start_tag = 0xB605; // PreviewImageStart
    let preview_length_tag = 0xB602; // PreviewImageLength

    // Get preview offset and length from Canon maker notes using flexible parsing
    let offset = match ifd.get_numeric_u32(preview_start_tag) {
        Some(offset) => offset as usize,
        None => return Ok(None), // No preview available
    };

    let length = match ifd.get_numeric_u32(preview_length_tag) {
        Some(length) => length as usize,
        None => return Ok(None), // No preview length
    };

    // Validate bounds
    if offset + length > original_data.len() {
        return Err(Error::InvalidExif(
            "Preview offset/length extends beyond file".into(),
        ));
    }

    if length == 0 {
        return Ok(None);
    }

    // Extract preview data
    let preview_data = original_data[offset..offset + length].to_vec();

    // Validate it's a JPEG
    if super::validate_jpeg(&preview_data) {
        Ok(Some(preview_data))
    } else {
        Err(Error::InvalidExif("Preview data is not valid JPEG".into()))
    }
}

/// Get Canon preview dimensions if available
pub fn get_canon_preview_dimensions(ifd: &ParsedIfd) -> Result<Option<(u32, u32)>> {
    let width_tag = 0xB603; // PreviewImageWidth
    let height_tag = 0xB604; // PreviewImageHeight

    let width = ifd.get_numeric_u32(width_tag);
    let height = ifd.get_numeric_u32(height_tag);

    match (width, height) {
        (Some(w), Some(h)) => Ok(Some((w, h))),
        _ => Ok(None),
    }
}

// Unit tests are commented out due to ParsedIfd private fields
// Integration tests with real images are in tests/spike3.rs
