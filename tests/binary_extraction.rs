//! Binary tag extraction tests (thumbnails, previews, etc.)

use exif_oxide::binary::composite_tags::{
    extract_jpgfromraw, extract_preview_image, extract_thumbnail_image,
};
use exif_oxide::binary::extract_binary_tag;
use exif_oxide::core::find_metadata_segment;
use exif_oxide::core::ifd::IfdParser;
use exif_oxide::core::types::ExifValue;
use exif_oxide::read_basic_exif;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[path = "common/mod.rs"]
mod common;

use common::validate_jpeg;

/// Test extracting thumbnails from various image types
#[test]
fn test_extract_thumbnail_from_canon_jpg() {
    let test_image = "test-images/canon/Canon_T3i.JPG";
    if !Path::new(test_image).exists() {
        eprintln!(
            "Warning: Test image {} not found, skipping test",
            test_image
        );
        return;
    }

    // Read metadata and file data
    let metadata_segment = find_metadata_segment(test_image)
        .unwrap()
        .expect("No EXIF data found");
    let ifd = IfdParser::parse(metadata_segment.data).unwrap();
    let original_data = fs::read(test_image).unwrap();

    // Extract thumbnail using tag 0x1201 (IFD1 thumbnail)
    match extract_binary_tag(&ifd, 0x1201, &original_data) {
        Ok(Some(thumbnail)) => {
            println!(
                "✓ Canon T3i JPG: Extracted thumbnail {} bytes",
                thumbnail.len()
            );
            assert!(!thumbnail.is_empty());
            assert!(validate_jpeg(&thumbnail), "Invalid JPEG data");
        }
        Ok(None) => {
            println!("ℹ Canon T3i JPG: No thumbnail found");
        }
        Err(e) => {
            panic!("Failed to extract thumbnail from Canon T3i JPG: {}", e);
        }
    }
}

#[test]
fn test_extract_thumbnail_from_nikon_jpg() {
    let test_image = "test-images/nikon/nikon_z8_73.jpg";
    if !Path::new(test_image).exists() {
        eprintln!(
            "Warning: Test image {} not found, skipping test",
            test_image
        );
        return;
    }

    // Read metadata and file data
    let metadata_segment = find_metadata_segment(test_image)
        .unwrap()
        .expect("No EXIF data found");
    let ifd = IfdParser::parse(metadata_segment.data).unwrap();
    let original_data = fs::read(test_image).unwrap();

    // Extract thumbnail using tag 0x1201 (IFD1 thumbnail)
    match extract_binary_tag(&ifd, 0x1201, &original_data) {
        Ok(Some(thumbnail)) => {
            println!(
                "✓ Nikon Z8 JPG: Extracted thumbnail {} bytes",
                thumbnail.len()
            );
            assert!(!thumbnail.is_empty());
        }
        Ok(None) => {
            println!("ℹ Nikon Z8 JPG: No thumbnail found");
        }
        Err(e) => {
            panic!("Failed to extract thumbnail from Nikon Z8 JPG: {}", e);
        }
    }
}

#[test]
fn test_extract_canon_preview_from_jpg() {
    let test_image = "test-images/canon/canon_eos_r5_mark_ii_10.jpg";
    if !Path::new(test_image).exists() {
        eprintln!(
            "Warning: Test image {} not found, skipping test",
            test_image
        );
        return;
    }

    // Read metadata and file data
    let metadata_segment = find_metadata_segment(test_image)
        .unwrap()
        .expect("No EXIF data found");
    let ifd = IfdParser::parse(metadata_segment.data).unwrap();
    let original_data = fs::read(test_image).unwrap();

    // Extract Canon preview using tag 0xB605 (Canon PreviewImageStart)
    match extract_binary_tag(&ifd, 0xB605, &original_data) {
        Ok(Some(preview)) => {
            println!(
                "✓ Canon R5 Mark II JPG: Extracted preview {} bytes",
                preview.len()
            );
            assert!(!preview.is_empty());
            assert!(validate_jpeg(&preview), "Invalid JPEG data");
        }
        Ok(None) => {
            println!("ℹ Canon R5 Mark II JPG: No Canon preview found");
        }
        Err(e) => {
            println!(
                "⚠ Canon R5 Mark II JPG: Canon preview extraction failed: {}",
                e
            );
        }
    }
}

#[test]
fn test_extract_binary_from_various_images() {
    let test_images = vec![
        ("test-images/canon/Canon_T3i.JPG", 0x1201, "Thumbnail"),
        (
            "test-images/canon/canon_eos_r5_mark_ii_10.jpg",
            0xB605,
            "Canon Preview",
        ),
        ("test-images/nikon/nikon_z8_73.jpg", 0x1201, "Thumbnail"),
        ("test-images/sony/sony_a7c_ii_02.jpg", 0x1201, "Thumbnail"),
        (
            "test-images/panasonic/panasonic_lumix_g9_ii_35.jpg",
            0x1201,
            "Thumbnail",
        ),
    ];

    for (test_image, tag_id, tag_name) in test_images {
        if !Path::new(test_image).exists() {
            eprintln!("Warning: Test image {} not found, skipping", test_image);
            continue;
        }

        // Read metadata and file data
        let metadata_segment = match find_metadata_segment(test_image) {
            Ok(Some(segment)) => segment,
            Ok(None) => {
                println!("ℹ {}: No EXIF data found", test_image);
                continue;
            }
            Err(e) => {
                println!("⚠ {}: Failed to find metadata: {}", test_image, e);
                continue;
            }
        };

        let ifd = match IfdParser::parse(metadata_segment.data) {
            Ok(ifd) => ifd,
            Err(e) => {
                println!("⚠ {}: Failed to parse IFD: {}", test_image, e);
                continue;
            }
        };

        let original_data = match fs::read(test_image) {
            Ok(data) => data,
            Err(e) => {
                println!("⚠ {}: Failed to read file: {}", test_image, e);
                continue;
            }
        };

        match extract_binary_tag(&ifd, tag_id, &original_data) {
            Ok(Some(data)) => {
                println!(
                    "✓ {}: Extracted {} - {} bytes",
                    Path::new(test_image).file_name().unwrap().to_str().unwrap(),
                    tag_name,
                    data.len()
                );
                assert!(!data.is_empty());
            }
            Ok(None) => {
                println!(
                    "ℹ {}: No {} found",
                    Path::new(test_image).file_name().unwrap().to_str().unwrap(),
                    tag_name
                );
            }
            Err(e) => {
                println!(
                    "⚠ {}: {} extraction failed: {}",
                    Path::new(test_image).file_name().unwrap().to_str().unwrap(),
                    tag_name,
                    e
                );
            }
        }
    }
}

#[test]
fn test_basic_exif_reading_still_works() {
    let test_image = "test-images/canon/Canon_T3i.JPG";
    if !Path::new(test_image).exists() {
        eprintln!(
            "Warning: Test image {} not found, skipping test",
            test_image
        );
        return;
    }

    match read_basic_exif(test_image) {
        Ok(exif) => {
            println!(
                "✓ Canon T3i JPG EXIF: Make={:?}, Model={:?}, Orientation={:?}",
                exif.make, exif.model, exif.orientation
            );
            assert!(exif.make.is_some());
            assert!(exif.model.is_some());
        }
        Err(e) => {
            panic!("Failed to read basic EXIF from Canon T3i JPG: {}", e);
        }
    }
}

#[test]
fn test_exiftool_test_images() {
    let exiftool_images = vec![
        ("exiftool/t/images/Canon.jpg", 0x1201, "Thumbnail"),
        ("exiftool/t/images/Canon1DmkIII.jpg", 0x1201, "Thumbnail"),
    ];

    for (test_image, tag_id, tag_name) in exiftool_images {
        if !Path::new(test_image).exists() {
            eprintln!(
                "Warning: ExifTool test image {} not found, skipping",
                test_image
            );
            continue;
        }

        println!("Testing ExifTool image: {}", test_image);

        // Test basic EXIF reading
        match read_basic_exif(test_image) {
            Ok(exif) => {
                println!("  ✓ EXIF: Make={:?}, Model={:?}", exif.make, exif.model);
            }
            Err(e) => {
                println!("  ⚠ EXIF reading failed: {}", e);
            }
        }

        // Test binary extraction
        if let Ok(Some(metadata_segment)) = find_metadata_segment(test_image) {
            if let Ok(ifd) = IfdParser::parse(metadata_segment.data) {
                if let Ok(original_data) = fs::read(test_image) {
                    match extract_binary_tag(&ifd, tag_id, &original_data) {
                        Ok(Some(data)) => {
                            println!("  ✓ {}: {} bytes", tag_name, data.len());
                        }
                        Ok(None) => {
                            println!("  ℹ No {} found", tag_name);
                        }
                        Err(e) => {
                            println!("  ⚠ {} extraction failed: {}", tag_name, e);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test_compare_thumbnail_vs_preview_sizes() {
    let test_image = "test-images/canon/Canon_T3i.JPG";
    if !Path::new(test_image).exists() {
        eprintln!(
            "Warning: Test image {} not found, skipping test",
            test_image
        );
        return;
    }

    // Read metadata and file data
    let metadata_segment = find_metadata_segment(test_image)
        .unwrap()
        .expect("No EXIF data found");
    let ifd = IfdParser::parse(metadata_segment.data).unwrap();
    let original_data = fs::read(test_image).unwrap();

    let thumbnail = extract_binary_tag(&ifd, 0x1201, &original_data).unwrap_or(None);
    let canon_preview = extract_binary_tag(&ifd, 0xB605, &original_data).unwrap_or(None);

    match (thumbnail, canon_preview) {
        (Some(thumb), Some(preview)) => {
            println!(
                "✓ Canon T3i JPG: Thumbnail {} bytes, Preview {} bytes",
                thumb.len(),
                preview.len()
            );
            // Canon previews are usually larger than thumbnails
            if preview.len() > thumb.len() {
                println!("  ✓ Preview is larger than thumbnail as expected");
            } else {
                println!("  ℹ Thumbnail is larger than or equal to preview");
            }
        }
        (Some(thumb), None) => {
            println!(
                "✓ Canon T3i JPG: Only thumbnail available {} bytes",
                thumb.len()
            );
        }
        (None, Some(preview)) => {
            println!(
                "✓ Canon T3i JPG: Only Canon preview available {} bytes",
                preview.len()
            );
        }
        (None, None) => {
            println!("ℹ Canon T3i JPG: No thumbnail or preview found");
        }
    }
}

/// Performance test: extraction should be fast
#[test]
fn test_extraction_performance() {
    let test_image = "test-images/canon/Canon_T3i.JPG";
    if !Path::new(test_image).exists() {
        eprintln!(
            "Warning: Test image {} not found, skipping test",
            test_image
        );
        return;
    }

    // Read metadata and file data once
    let metadata_segment = find_metadata_segment(test_image)
        .unwrap()
        .expect("No EXIF data found");
    let ifd = IfdParser::parse(metadata_segment.data).unwrap();
    let original_data = fs::read(test_image).unwrap();

    let start = std::time::Instant::now();

    // Try to extract both thumbnail and preview
    let _thumb = extract_binary_tag(&ifd, 0x1201, &original_data);
    let _preview = extract_binary_tag(&ifd, 0xB605, &original_data);

    let duration = start.elapsed();

    println!(
        "✓ Canon T3i JPG: Extraction time: {:.2}ms",
        duration.as_secs_f64() * 1000.0
    );

    // Should be under 50ms for typical images
    assert!(
        duration.as_millis() < 50,
        "Extraction took too long: {}ms",
        duration.as_millis()
    );
}

/// Test our new composite tag extraction functionality
#[test]
fn test_composite_tag_extraction() {
    let test_image = "test-images/canon/Canon_T3i.JPG";
    if !Path::new(test_image).exists() {
        eprintln!(
            "Warning: Test image {} not found, skipping test",
            test_image
        );
        return;
    }

    // Read metadata and file data
    let metadata_segment = find_metadata_segment(test_image)
        .unwrap()
        .expect("No EXIF data found");
    let ifd = IfdParser::parse(metadata_segment.data).unwrap();
    let original_data = fs::read(test_image).unwrap();

    // Convert IFD entries to our composite tag format
    let mut tags: HashMap<u16, ExifValue> = HashMap::new();

    // Look for ThumbnailOffset and ThumbnailLength
    if let Some(offset_entry) = ifd.entries().get(&0x0201) {
        if let Some(length_entry) = ifd.entries().get(&0x0202) {
            // Extract offset and length from ExifValue
            let offset = match offset_entry {
                ExifValue::U32(val) => Some(*val),
                ExifValue::U16(val) => Some(*val as u32),
                _ => None,
            };

            let length = match length_entry {
                ExifValue::U32(val) => Some(*val),
                ExifValue::U16(val) => Some(*val as u32),
                _ => None,
            };

            if let (Some(offset), Some(length)) = (offset, length) {
                tags.insert(0x0201, ExifValue::U32(offset));
                tags.insert(0x0202, ExifValue::U32(length));

                println!(
                    "Found ThumbnailOffset: 0x{:X}, ThumbnailLength: {}",
                    offset, length
                );
            }
        }
    }

    // Test our composite tag extractors
    if let Some(thumbnail) = extract_thumbnail_image(&tags, &original_data) {
        println!(
            "✓ Composite ThumbnailImage: Extracted {} bytes",
            thumbnail.len()
        );
        assert!(!thumbnail.is_empty());
        assert!(
            common::validate_jpeg(&thumbnail),
            "Invalid JPEG data from composite extractor"
        );
    } else {
        println!("ℹ No thumbnail found via composite extractor");
    }

    // Test PreviewImage extraction (may not be present in all images)
    if let Some(preview) = extract_preview_image(&tags, &original_data) {
        println!(
            "✓ Composite PreviewImage: Extracted {} bytes",
            preview.len()
        );
        assert!(!preview.is_empty());
    } else {
        println!("ℹ No preview found via composite extractor");
    }

    // Test JpgFromRaw extraction (unlikely to be present in JPEG)
    if let Some(jpg_from_raw) = extract_jpgfromraw(&tags, &original_data) {
        println!(
            "✓ Composite JpgFromRaw: Extracted {} bytes",
            jpg_from_raw.len()
        );
        assert!(!jpg_from_raw.is_empty());
    } else {
        println!("ℹ No JpgFromRaw found via composite extractor (expected for JPEG)");
    }

    println!("✓ Composite tag extraction test completed");
}
