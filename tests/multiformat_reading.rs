//! Multi-format metadata reading integration tests
//!
//! Tests the core functionality of reading metadata from various file formats
//! including JPEG, TIFF, PNG, HEIF, WebP, and video containers.

use exif_oxide::detection::{detect_file_type, FileType};
use exif_oxide::{core::find_metadata_segment, read_basic_exif};
use std::path::Path;

#[test]
fn test_jpeg_metadata_extraction() {
    let test_files = vec![
        "exiftool/t/images/Canon.jpg",
        "exiftool/t/images/Nikon.jpg",
        "exiftool/t/images/Sony.jpg",
        "exiftool/t/images/FujiFilm.jpg",
    ];

    for file_path in test_files {
        if !Path::new(file_path).exists() {
            eprintln!("Skipping test - file not found: {}", file_path);
            continue;
        }

        // Verify format detection
        let format = detect_format(file_path);
        assert_eq!(
            format,
            FileType::JPEG,
            "Should detect as JPEG: {}",
            file_path
        );

        // Verify metadata extraction
        let metadata = find_metadata_segment(file_path).unwrap();
        assert!(
            metadata.is_some(),
            "Should find metadata in JPEG: {}",
            file_path
        );

        // Verify basic EXIF extraction
        let exif = read_basic_exif(file_path).unwrap();
        assert!(
            exif.make.is_some() || exif.model.is_some(),
            "Should extract make/model from {}",
            file_path
        );
    }
}

#[test]
fn test_raw_format_metadata_extraction() {
    let raw_formats = vec![
        ("exiftool/t/images/CanonRaw.cr2", FileType::CR2, "Canon"),
        ("exiftool/t/images/Nikon.nef", FileType::NEF, "NIKON"),
        ("exiftool/t/images/DNG.dng", FileType::TIFF, "Canon"), // DNG with Canon EXIF
    ];

    for (file_path, expected_type, expected_make) in raw_formats {
        if !Path::new(file_path).exists() {
            continue;
        }

        // Verify format detection
        let format = detect_format(file_path);
        assert_eq!(format, expected_type, "Format detection for {}", file_path);

        // Verify metadata extraction
        let metadata = find_metadata_segment(file_path).unwrap();
        assert!(metadata.is_some(), "Should find metadata in {}", file_path);

        // Verify make extraction
        let exif = read_basic_exif(file_path).unwrap();
        // Some formats may have variations in Make strings
        let actual_make = exif.make.as_deref();
        if expected_make == "NIKON" {
            // Nikon files may have "NIKON CORPORATION" or just "NIKON"
            assert!(
                actual_make == Some("NIKON") || actual_make == Some("NIKON CORPORATION"),
                "Make extraction from {}: got {:?}",
                file_path,
                actual_make
            );
        } else {
            assert_eq!(
                actual_make,
                Some(expected_make),
                "Make extraction from {}",
                file_path
            );
        }
    }
}

#[test]
fn test_png_metadata_support() {
    let test_file = "exiftool/t/images/PNG.png";
    if !Path::new(test_file).exists() {
        return;
    }

    let format = detect_format(test_file);
    assert_eq!(format, FileType::PNG);

    // PNG may or may not have eXIf chunk
    let metadata = find_metadata_segment(test_file).unwrap();
    match metadata {
        Some(segment) => {
            println!("Found {} bytes of metadata in PNG", segment.data.len());
            assert!(!segment.data.is_empty());
        }
        None => {
            println!("No eXIf chunk in PNG (valid case)");
        }
    }
}

#[test]
fn test_webp_container_format() {
    let test_file = "exiftool/t/images/RIFF.webp";
    if !Path::new(test_file).exists() {
        return;
    }

    let format = detect_format(test_file);
    assert_eq!(format, FileType::WEBP);

    let metadata = find_metadata_segment(test_file).unwrap();
    if let Some(segment) = metadata {
        // WebP may contain EXIF (starts with TIFF header) or XMP
        assert!(segment.data.len() >= 4);
        let header = &segment.data[0..4];

        if header == [0x49, 0x49, 0x2a, 0x00] || header == [0x4d, 0x4d, 0x00, 0x2a] {
            println!("Found EXIF data in WebP");
        } else {
            println!("Found other metadata type in WebP (possibly XMP)");
        }
    }
}

#[test]
fn test_video_container_formats() {
    let video_formats = vec![
        ("exiftool/t/images/MP4.mp4", FileType::MP4),
        ("exiftool/t/images/QuickTime.mov", FileType::MOV),
        ("exiftool/t/images/RIFF.avi", FileType::AVI),
    ];

    for (file_path, expected_type) in video_formats {
        if !Path::new(file_path).exists() {
            continue;
        }

        let format = detect_format(file_path);
        assert_eq!(format, expected_type, "Format detection for {}", file_path);

        // Video formats may or may not have EXIF
        let _ = find_metadata_segment(file_path).unwrap();
        println!("Processed {} successfully", file_path);
    }
}

#[test]
fn test_format_dispatch_consistency() {
    // Test that the same file always produces the same result
    let test_file = "exiftool/t/images/Canon.jpg";
    if !Path::new(test_file).exists() {
        return;
    }

    // Read multiple times
    let result1 = find_metadata_segment(test_file).unwrap();
    let result2 = find_metadata_segment(test_file).unwrap();

    match (result1, result2) {
        (Some(seg1), Some(seg2)) => {
            assert_eq!(seg1.data.len(), seg2.data.len());
            assert_eq!(seg1.offset, seg2.offset);
            assert_eq!(seg1.source_format, seg2.source_format);
        }
        (None, None) => {
            // Both found no metadata - consistent
        }
        _ => panic!("Inconsistent results from same file"),
    }
}

#[test]
fn test_unsupported_format_handling() {
    let unsupported_files = vec!["exiftool/t/images/GIF.gif", "exiftool/t/images/BMP.bmp"];

    for file_path in unsupported_files {
        if !Path::new(file_path).exists() {
            continue;
        }

        // Should detect format
        let format = detect_format(file_path);
        assert_ne!(
            format,
            FileType::Unknown,
            "Should detect format of {}",
            file_path
        );

        // But no metadata extraction
        let metadata = find_metadata_segment(file_path).unwrap();
        assert!(
            metadata.is_none(),
            "Should not find metadata in {}",
            file_path
        );
    }
}

#[test]
fn test_tiff_variants() {
    let tiff_files = vec![
        "exiftool/t/images/ExifTool.tif",
        "exiftool/t/images/GeoTiff.tif",
    ];

    for file_path in tiff_files {
        if !Path::new(file_path).exists() {
            continue;
        }

        let format = detect_format(file_path);
        assert_eq!(format, FileType::TIFF);

        let metadata = find_metadata_segment(file_path).unwrap();
        assert!(metadata.is_some(), "Should find metadata in TIFF");

        // TIFF files should have data starting at offset 0
        if let Some(segment) = metadata {
            assert_eq!(
                segment.offset, 0,
                "TIFF data should start at file beginning"
            );
        }
    }
}

// Helper function to detect format
fn detect_format(path: &str) -> FileType {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(path).unwrap();
    let mut buffer = vec![0u8; 1024];
    let bytes_read = file.read(&mut buffer).unwrap();
    buffer.truncate(bytes_read);

    detect_file_type(&buffer).unwrap().file_type
}
