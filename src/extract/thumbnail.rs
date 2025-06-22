//! EXIF thumbnail extraction from IFD1

use crate::core::ifd::ParsedIfd;
use crate::error::{Error, Result};

/// Extract thumbnail image from IFD1
///
/// EXIF thumbnails are stored in IFD1 with:
/// - Tag 0x201 (ThumbnailOffset): Offset to thumbnail data
/// - Tag 0x202 (ThumbnailLength): Length of thumbnail data
pub fn extract_thumbnail(ifd: &ParsedIfd, original_data: &[u8]) -> Result<Option<Vec<u8>>> {
    // Get thumbnail offset and length from IFD1 using flexible numeric parsing
    let parsed_offset = match ifd.get_numeric_u32(0x1000 + 0x201) {
        Some(offset) => offset as usize,
        None => return Ok(None), // No thumbnail offset available
    };

    let length = match ifd.get_numeric_u32(0x1000 + 0x202) {
        Some(length) => length as usize,
        None => return Ok(None), // No thumbnail length available
    };

    // The parsed offset might point to a data structure containing the JPEG
    // Let's first try the parsed offset, then check for JPEG header adjustments
    let mut offset = parsed_offset;

    // Check if there's a JPEG header at this offset
    if offset + 2 < original_data.len()
        && original_data[offset] == 0xFF
        && original_data[offset + 1] == 0xD8
    {
        // Direct JPEG, use as-is
    } else {
        // Look for JPEG header within the first 20 bytes
        let search_end = std::cmp::min(offset + 20, original_data.len());
        if let Some(jpeg_offset) = original_data[offset..search_end]
            .windows(2)
            .position(|window| window == [0xFF, 0xD8])
        {
            offset += jpeg_offset;
        }
    }

    // Validate bounds
    if offset + length > original_data.len() {
        return Err(Error::InvalidExif(
            "Thumbnail offset/length extends beyond file".into(),
        ));
    }

    if length == 0 {
        return Ok(None);
    }

    // Extract thumbnail data directly (we've already adjusted the offset)
    let raw_data = &original_data[offset..offset + length];

    // Find JPEG end (EOI marker) to trim any trailing data
    let jpeg_end = raw_data
        .windows(2)
        .position(|window| window == [0xFF, 0xD9])
        .map(|pos| pos + 2) // +2 to include the EOI marker
        .unwrap_or(raw_data.len());

    let thumbnail_data = raw_data[..jpeg_end].to_vec();

    // Validate it's a JPEG
    if super::validate_jpeg(&thumbnail_data) {
        Ok(Some(thumbnail_data))
    } else {
        Err(Error::InvalidExif(
            "Thumbnail data is not valid JPEG".into(),
        ))
    }
}

// Unit tests are commented out due to ParsedIfd private fields
// Integration tests with real images are in tests/spike3.rs
