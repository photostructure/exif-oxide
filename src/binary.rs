//! Binary tag extraction module
//!
//! Simple module for extracting binary data from EXIF tags.

use crate::core::ifd::ParsedIfd;
use crate::core::mpf::{MpfImageType, ParsedMpf};
use crate::core::ExifValue;
use crate::error::{Error, Result};

/// Extract binary data for a specific tag ID
///
/// This function extracts the raw binary data associated with a tag.
/// For offset-based tags (like ThumbnailImage or PreviewImage), it reads
/// the data from the original file using the offset and length tags.
///
/// # Arguments
/// * `ifd` - The parsed IFD containing tag entries
/// * `tag_id` - The tag ID to extract
/// * `original_data` - The original file data (needed for offset-based tags)
///
/// # Returns
/// * `Ok(Some(data))` - The binary data if found
/// * `Ok(None)` - If the tag doesn't exist
/// * `Err` - If the tag exists but extraction failed
pub fn extract_binary_tag(
    ifd: &ParsedIfd,
    tag_id: u16,
    original_data: &[u8],
) -> Result<Option<Vec<u8>>> {
    // Check if this is an offset-based image tag
    match tag_id {
        // IFD1 ThumbnailImage (uses actual IFD1 tags 0x201, 0x202)
        0x1201 => extract_offset_based_tag(ifd, 0x1201, 0x1202, original_data),
        // IFD0 PreviewImage (in some formats)
        0x111 => extract_offset_based_tag(ifd, 0x111, 0x117, original_data),
        // Canon PreviewImage
        0xB605 => extract_offset_based_tag(ifd, 0xB605, 0xB602, original_data),
        // Default: try to extract directly from the tag value
        _ => extract_direct_binary(ifd, tag_id),
    }
}

/// Extract binary data that's stored directly in the tag value
fn extract_direct_binary(ifd: &ParsedIfd, tag_id: u16) -> Result<Option<Vec<u8>>> {
    let value = match ifd.entries().get(&tag_id) {
        Some(v) => v,
        None => return Ok(None),
    };

    match value {
        ExifValue::Undefined(data) => Ok(Some(data.clone())),
        ExifValue::U8Array(data) => Ok(Some(data.clone())),
        _ => Err(Error::InvalidExif(format!(
            "Tag 0x{:04X} does not contain binary data",
            tag_id
        ))),
    }
}

/// Extract binary data using offset/length tag pairs
fn extract_offset_based_tag(
    ifd: &ParsedIfd,
    offset_tag: u16,
    length_tag: u16,
    original_data: &[u8],
) -> Result<Option<Vec<u8>>> {
    // Get offset and length
    let offset = match ifd.get_numeric_u32(offset_tag) {
        Some(o) => o as usize,
        None => return Ok(None),
    };

    let length = match ifd.get_numeric_u32(length_tag) {
        Some(l) => l as usize,
        None => return Ok(None),
    };

    // Validate bounds
    if offset + length > original_data.len() {
        return Err(Error::InvalidExif(format!(
            "Binary data offset/length extends beyond file: offset={}, length={}, file_size={}",
            offset,
            length,
            original_data.len()
        )));
    }

    if length == 0 {
        return Ok(None);
    }

    // The offset is relative to the TIFF header, not the file start
    // For JPEG files, the TIFF header starts after the APP1 marker header
    // We need to find where the EXIF data starts in the file

    // Look for "Exif\0\0" marker to find TIFF header position
    let tiff_offset = if original_data.len() > 12 && &original_data[0..2] == b"\xFF\xD8" {
        // This is a JPEG file, find the TIFF header offset
        if let Some(exif_pos) = original_data
            .windows(6)
            .position(|window| window == b"Exif\x00\x00")
        {
            // TIFF header starts 6 bytes after "Exif\0\0"
            exif_pos + 6
        } else {
            // Fallback: assume standard JPEG structure
            12
        }
    } else {
        // For non-JPEG files (RAW, TIFF), offset is from file start
        0
    };

    let file_offset = tiff_offset + offset;

    // Validate bounds
    if file_offset + length > original_data.len() {
        return Err(Error::InvalidExif(format!(
            "Binary data extends beyond file: file_offset={}, length={}, file_size={}",
            file_offset,
            length,
            original_data.len()
        )));
    }

    // Extract the data
    let data = original_data[file_offset..file_offset + length].to_vec();

    Ok(Some(data))
}

/// Extract an image from MPF data
///
/// MPF (Multi-Picture Format) stores multiple images in a single JPEG file.
/// This function extracts a specific image by index.
///
/// # Arguments
/// * `mpf` - The parsed MPF data
/// * `image_index` - The index of the image to extract (0-based)
/// * `original_data` - The original file data
/// * `mpf_offset` - The offset of the MPF segment in the file
///
/// # Returns
/// * `Ok(Some(data))` - The image data if found
/// * `Ok(None)` - If the image index doesn't exist
/// * `Err` - If extraction failed
pub fn extract_mpf_image(
    mpf: &ParsedMpf,
    image_index: usize,
    original_data: &[u8],
    mpf_offset: usize,
) -> Result<Option<Vec<u8>>> {
    // Check if the image index is valid
    if image_index >= mpf.images.len() {
        return Ok(None);
    }

    let image_entry = &mpf.images[image_index];

    // MPF offsets are relative to the start of the MPF segment (after "MPF\0")
    let file_offset = mpf_offset + image_entry.offset as usize;
    let length = image_entry.length as usize;

    // Validate bounds
    if file_offset + length > original_data.len() {
        return Err(Error::InvalidExif(format!(
            "MPF image extends beyond file: offset={}, length={}, file_size={}",
            file_offset,
            length,
            original_data.len()
        )));
    }

    if length == 0 {
        return Ok(None);
    }

    // Extract the data
    let data = original_data[file_offset..file_offset + length].to_vec();

    Ok(Some(data))
}

/// Find and extract the first large thumbnail from MPF data
///
/// This searches for the first image with type LargeThumbnailVGA or LargeThumbnailFullHD.
///
/// # Arguments
/// * `mpf` - The parsed MPF data
/// * `original_data` - The original file data
/// * `mpf_offset` - The offset of the MPF segment in the file
///
/// # Returns
/// * `Ok(Some(data))` - The thumbnail data if found
/// * `Ok(None)` - If no large thumbnail exists
/// * `Err` - If extraction failed
pub fn extract_mpf_preview(
    mpf: &ParsedMpf,
    original_data: &[u8],
    mpf_offset: usize,
) -> Result<Option<Vec<u8>>> {
    // Find the first large thumbnail
    for (index, image) in mpf.images.iter().enumerate() {
        match image.image_type {
            MpfImageType::LargeThumbnailVGA | MpfImageType::LargeThumbnailFullHD => {
                return extract_mpf_image(mpf, index, original_data, mpf_offset);
            }
            _ => continue,
        }
    }

    Ok(None)
}
