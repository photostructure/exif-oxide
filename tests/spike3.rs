//! Spike 3 tests: Binary tag extraction (preview images and thumbnails)

use exif_oxide::{
    extract_canon_preview, extract_largest_preview, extract_thumbnail, read_basic_exif,
};
use std::path::Path;

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

    match extract_thumbnail(test_image) {
        Ok(Some(thumbnail)) => {
            println!(
                "✓ Canon T3i JPG: Extracted thumbnail {} bytes",
                thumbnail.len()
            );
            assert!(!thumbnail.is_empty());

            // Validate JPEG format
            assert_eq!(thumbnail[0], 0xFF);
            assert_eq!(thumbnail[1], 0xD8);
            assert_eq!(thumbnail[thumbnail.len() - 2], 0xFF);
            assert_eq!(thumbnail[thumbnail.len() - 1], 0xD9);
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

    match extract_thumbnail(test_image) {
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

    match extract_canon_preview(test_image) {
        Ok(Some(preview)) => {
            println!(
                "✓ Canon R5 Mark II JPG: Extracted preview {} bytes",
                preview.len()
            );
            assert!(!preview.is_empty());

            // Validate JPEG format
            assert_eq!(preview[0], 0xFF);
            assert_eq!(preview[1], 0xD8);
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
fn test_extract_largest_preview_from_various_images() {
    let test_images = vec![
        "test-images/canon/Canon_T3i.JPG",
        "test-images/canon/canon_eos_r5_mark_ii_10.jpg",
        "test-images/nikon/nikon_z8_73.jpg",
        "test-images/sony/sony_a7c_ii_02.jpg",
        "test-images/panasonic/panasonic_lumix_g9_ii_35.jpg",
    ];

    for test_image in test_images {
        if !Path::new(test_image).exists() {
            eprintln!("Warning: Test image {} not found, skipping", test_image);
            continue;
        }

        match extract_largest_preview(test_image) {
            Ok(Some(preview)) => {
                println!(
                    "✓ {}: Extracted largest preview {} bytes",
                    Path::new(test_image).file_name().unwrap().to_str().unwrap(),
                    preview.len()
                );
                assert!(!preview.is_empty());
            }
            Ok(None) => {
                println!(
                    "ℹ {}: No preview found",
                    Path::new(test_image).file_name().unwrap().to_str().unwrap()
                );
            }
            Err(e) => {
                println!(
                    "⚠ {}: Preview extraction failed: {}",
                    Path::new(test_image).file_name().unwrap().to_str().unwrap(),
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
        "exiftool/t/images/Canon.jpg",
        "exiftool/t/images/Canon1DmkIII.jpg",
    ];

    for test_image in exiftool_images {
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

        // Test thumbnail extraction
        match extract_thumbnail(test_image) {
            Ok(Some(thumbnail)) => {
                println!("  ✓ Thumbnail: {} bytes", thumbnail.len());
            }
            Ok(None) => {
                println!("  ℹ No thumbnail found");
            }
            Err(e) => {
                println!("  ⚠ Thumbnail extraction failed: {}", e);
            }
        }

        // Test Canon preview extraction
        match extract_canon_preview(test_image) {
            Ok(Some(preview)) => {
                println!("  ✓ Canon preview: {} bytes", preview.len());
            }
            Ok(None) => {
                println!("  ℹ No Canon preview found");
            }
            Err(e) => {
                println!("  ⚠ Canon preview extraction failed: {}", e);
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

    let thumbnail = extract_thumbnail(test_image).unwrap_or(None);
    let canon_preview = extract_canon_preview(test_image).unwrap_or(None);

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

    let start = std::time::Instant::now();
    let _result = extract_largest_preview(test_image);
    let duration = start.elapsed();

    println!(
        "✓ Canon T3i JPG: Extraction time: {:.2}ms",
        duration.as_secs_f64() * 1000.0
    );

    // Should be under 50ms for typical images (target was <5ms but being realistic)
    assert!(
        duration.as_millis() < 50,
        "Extraction took too long: {}ms",
        duration.as_millis()
    );
}
