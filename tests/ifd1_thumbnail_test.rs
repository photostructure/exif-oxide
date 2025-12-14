//! IFD1 Thumbnail Extraction Tests
//!
//! Tests for parsing IFD1 (thumbnail directory) and extracting ThumbnailOffset/ThumbnailLength tags.
//!
//! ## Background
//!
//! TIFF/EXIF structure contains an IFD chain:
//! - IFD0: Main image metadata (Make, Model, ExifIFD pointer, GPS pointer)
//! - IFD1: Thumbnail image metadata (ThumbnailOffset, ThumbnailLength)
//!
//! Each IFD ends with a 4-byte "next IFD offset" pointer. IFD0 points to IFD1.
//!
//! ExifTool Reference: lib/Image/ExifTool/Exif.pm IFD chain processing
//! TPP: docs/tpp/P0-IFD1-THUMBNAIL-EXTRACTION.md

#![cfg(feature = "integration-tests")]

use exif_oxide::formats::extract_metadata;
use std::path::Path;

mod common;
use common::APPLE_IMG_3755_JPG;

/// Test that IFD1 thumbnail tags are extracted
///
/// This test verifies that ThumbnailOffset (0x0201) and ThumbnailLength (0x0202)
/// are extracted from IFD1 when present.
///
/// Test image: test-images/apple/IMG_3755.JPG
/// - Has IFD1 with 6 entries
/// - ThumbnailOffset = 3014 (0x0bc6)
/// - ThumbnailLength = 8106 (0x1faa)
#[test]
fn test_ifd1_thumbnail_tags_extracted() {
    let exif_data = extract_metadata(Path::new(APPLE_IMG_3755_JPG), false, false, None)
        .expect("Failed to extract metadata from test image");

    // Check that ThumbnailOffset was extracted from IFD1
    let has_thumbnail_offset = exif_data
        .tags
        .iter()
        .any(|t| t.name.contains("ThumbnailOffset"));

    // Check that ThumbnailLength was extracted from IFD1
    let has_thumbnail_length = exif_data
        .tags
        .iter()
        .any(|t| t.name.contains("ThumbnailLength"));

    assert!(
        has_thumbnail_offset,
        "ThumbnailOffset should be extracted from IFD1. \
         Available tags: {:?}",
        exif_data.tags.iter().map(|t| &t.name).collect::<Vec<_>>()
    );

    assert!(
        has_thumbnail_length,
        "ThumbnailLength should be extracted from IFD1. \
         Available tags: {:?}",
        exif_data.tags.iter().map(|t| &t.name).collect::<Vec<_>>()
    );
}

/// Test that ThumbnailOffset and ThumbnailLength have correct values
///
/// Based on ExifTool verbose output for test-images/apple/IMG_3755.JPG:
/// - ThumbnailOffset = 3014
/// - ThumbnailLength = 8106
#[test]
fn test_ifd1_thumbnail_values() {
    let exif_data = extract_metadata(Path::new(APPLE_IMG_3755_JPG), false, false, None)
        .expect("Failed to extract metadata from test image");

    // Find ThumbnailOffset tag
    let thumbnail_offset = exif_data
        .tags
        .iter()
        .find(|t| t.name.contains("ThumbnailOffset"))
        .expect("ThumbnailOffset tag not found");

    // Find ThumbnailLength tag
    let thumbnail_length = exif_data
        .tags
        .iter()
        .find(|t| t.name.contains("ThumbnailLength"))
        .expect("ThumbnailLength tag not found");

    // Verify values match ExifTool output
    // ThumbnailOffset is an IsOffset tag - the raw IFD value (3014) is adjusted by adding
    // the TIFF base offset (34) to get the absolute file offset (3048).
    // ExifTool: Exif.pm:7052-7066 applies base offset to IsOffset tags during extraction.
    let offset_val = thumbnail_offset
        .value
        .as_u32()
        .expect("ThumbnailOffset should be a u32");
    let length_val = thumbnail_length
        .value
        .as_u32()
        .expect("ThumbnailLength should be a u32");

    // Raw IFD value is 3014, TIFF header starts at file offset 34 (0x22),
    // so absolute file offset = 3014 + 34 = 3048
    assert_eq!(
        offset_val, 3048,
        "ThumbnailOffset should be 3048 (absolute file offset), got {}",
        offset_val
    );
    assert_eq!(
        length_val, 8106,
        "ThumbnailLength should be 8106, got {}",
        length_val
    );
}

/// Test that IFD1 tags have correct group assignment
///
/// ExifTool assigns IFD1 tags to:
/// - Group0: "EXIF"
/// - Group1: "IFD1"
#[test]
fn test_ifd1_group_assignment() {
    let exif_data = extract_metadata(Path::new(APPLE_IMG_3755_JPG), false, false, None)
        .expect("Failed to extract metadata from test image");

    // Find ThumbnailOffset tag
    let thumbnail_offset = exif_data
        .tags
        .iter()
        .find(|t| t.name.contains("ThumbnailOffset"))
        .expect("ThumbnailOffset tag not found");

    // Verify group assignment
    assert_eq!(
        thumbnail_offset.group, "EXIF",
        "ThumbnailOffset should have group0='EXIF', got '{}'",
        thumbnail_offset.group
    );

    assert_eq!(
        thumbnail_offset.group1, "IFD1",
        "ThumbnailOffset should have group1='IFD1', got '{}'",
        thumbnail_offset.group1
    );
}
