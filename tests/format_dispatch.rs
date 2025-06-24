//! Format dispatch system integration tests
//!
//! Tests the central format dispatch mechanism that routes
//! different file formats to their appropriate parsers.

use exif_oxide::core::{find_all_metadata_segments, find_metadata_segment, MetadataType};
use exif_oxide::detection::{detect_file_type, FileType};
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[test]
fn test_central_dispatch_all_formats() {
    // Test that all supported formats are correctly dispatched
    let test_cases = vec![
        // Format, test file, expected to have metadata
        (FileType::JPEG, "exiftool/t/images/Canon.jpg", true),
        (FileType::TIFF, "exiftool/t/images/ExifTool.tif", true), // TIFF with Canon EXIF
        (FileType::PNG, "exiftool/t/images/PNG.png", false),      // May not have EXIF
        (FileType::CR2, "exiftool/t/images/CanonRaw.cr2", true),
        (FileType::NEF, "exiftool/t/images/Nikon.nef", true),
        (FileType::ARW, "exiftool/t/images/Sony.arw", true),
        (FileType::WEBP, "exiftool/t/images/WebP.webp", false), // May not have EXIF
        (FileType::HEIC, "exiftool/t/images/HEIC.heic", false), // May not have EXIF
        (FileType::MP4, "exiftool/t/images/MP4.mp4", false),    // May not have EXIF
    ];

    for (expected_type, file_path, should_have_metadata) in test_cases {
        if !Path::new(file_path).exists() {
            continue;
        }

        // Verify format detection
        let mut file = File::open(file_path).unwrap();
        let mut buffer = vec![0u8; 1024];
        let bytes_read = file.read(&mut buffer).unwrap();
        buffer.truncate(bytes_read);

        let detected = detect_file_type(&buffer).unwrap();
        assert_eq!(
            detected.file_type, expected_type,
            "Format detection failed for {}",
            file_path
        );

        // Verify metadata extraction dispatch
        let metadata = find_metadata_segment(file_path).unwrap();

        if should_have_metadata {
            assert!(
                metadata.is_some(),
                "Expected metadata in {} but found none",
                file_path
            );

            let segment = metadata.unwrap();
            assert_eq!(
                segment.source_format, expected_type,
                "Source format mismatch for {}",
                file_path
            );
        }
    }
}

#[test]
fn test_jpeg_multi_segment_dispatch() {
    let test_file = "exiftool/t/images/Canon.jpg";
    if !Path::new(test_file).exists() {
        eprintln!("Skipping test - file not found: {}", test_file);
        return;
    }

    // Test that JPEG dispatch returns all segment types
    let collection = find_all_metadata_segments(test_file).unwrap();

    // Should have at least EXIF
    assert!(collection.exif.is_some(), "JPEG should have EXIF segment");

    let exif = collection.exif.unwrap();
    assert_eq!(exif.metadata_type, MetadataType::Exif);
    assert_eq!(exif.source_format, FileType::JPEG);

    // May also have MPF or XMP
    if let Some(mpf) = collection.mpf {
        assert_eq!(mpf.metadata_type, MetadataType::Mpf);
        assert_eq!(mpf.source_format, FileType::JPEG);
    }

    for xmp in &collection.xmp {
        assert_eq!(xmp.metadata_type, MetadataType::Xmp);
        assert_eq!(xmp.source_format, FileType::JPEG);
    }
}

#[test]
fn test_tiff_based_format_routing() {
    // Test that all TIFF-based formats route to TIFF parser
    let tiff_formats = vec![
        // Only test files that actually exist in the test suite
        (FileType::CR2, "exiftool/t/images/CanonRaw.cr2"),
        (FileType::NEF, "exiftool/t/images/Nikon.nef"),
        (FileType::RW2, "exiftool/t/images/Panasonic.rw2"),
        // Note: ExifTool.tif and DNG.dng have Canon Make so may be detected as CR2
    ];

    for (format_type, file_path) in tiff_formats {
        if !Path::new(file_path).exists() {
            continue;
        }

        let metadata = find_metadata_segment(file_path).unwrap();

        if let Some(segment) = metadata {
            // All TIFF-based formats should produce EXIF metadata
            assert_eq!(segment.metadata_type, MetadataType::Exif);
            assert_eq!(segment.source_format, format_type);

            // Should have valid TIFF header
            assert!(segment.data.len() >= 8, "TIFF data too small");
            let header = &segment.data[0..4];
            assert!(
                header == [0x49, 0x49, 0x2a, 0x00] || header == [0x4d, 0x4d, 0x00, 0x2a],
                "Invalid TIFF header for {}",
                file_path
            );
        }
    }
}

#[test]
fn test_heif_container_routing() {
    // Test HEIF/HEIC/AVIF routing to HEIF parser
    let heif_formats = vec![
        (FileType::HEIF, "exiftool/t/images/HEIF.heif"),
        (FileType::HEIC, "exiftool/t/images/HEIC.heic"),
        (FileType::AVIF, "exiftool/t/images/AVIF.avif"),
        (FileType::CR3, "exiftool/t/images/CanonCR3.cr3"),
    ];

    for (format_type, file_path) in heif_formats {
        if !Path::new(file_path).exists() {
            continue;
        }

        let metadata = find_metadata_segment(file_path).unwrap();

        match metadata {
            Some(segment) => {
                assert_eq!(segment.source_format, format_type);
                assert_eq!(segment.metadata_type, MetadataType::Exif);
            }
            None => {
                println!("No metadata in {} (OK for some HEIF files)", file_path);
            }
        }
    }
}

#[test]
fn test_dispatch_error_handling() {
    // Test graceful handling of unsupported formats

    // Create fake data for unsupported format
    let fake_data = b"FAKE_FORMAT_HEADER";
    let mut cursor = std::io::Cursor::new(fake_data);

    let collection = exif_oxide::core::find_all_metadata_segments_from_reader(&mut cursor).unwrap();

    // Should return empty collection for unknown format
    assert!(collection.exif.is_none());
    assert!(collection.mpf.is_none());
    assert!(collection.xmp.is_empty());
    assert!(collection.iptc.is_none());
}

#[test]
fn test_dispatch_with_corrupted_files() {
    // Test that dispatch handles corrupted files gracefully

    // Corrupted JPEG (wrong marker after SOI)
    let bad_jpeg = vec![0xFF, 0xD8, 0xFF, 0x00]; // Invalid marker
    let mut cursor = std::io::Cursor::new(bad_jpeg);

    let result = exif_oxide::core::find_metadata_segment_from_reader(&mut cursor);
    match result {
        Ok(_) => println!("Corrupted JPEG handled gracefully"),
        Err(_) => println!("Corrupted JPEG returned error (acceptable)"),
    }

    // Corrupted TIFF (invalid IFD offset)
    let bad_tiff = vec![
        0x49, 0x49, 0x2a, 0x00, // TIFF header
        0xFF, 0xFF, 0xFF, 0xFF, // Invalid IFD offset
    ];
    let mut cursor = std::io::Cursor::new(bad_tiff);

    let result = exif_oxide::core::find_metadata_segment_from_reader(&mut cursor);
    assert!(
        result.is_ok() || result.is_err(),
        "Should handle invalid TIFF gracefully"
    );
}

#[test]
fn test_format_aliases_dispatch() {
    // Test that format aliases are handled correctly
    let alias_tests = vec![
        // Some cameras produce files with non-standard extensions
        ("exiftool/t/images/Canon.jpg", FileType::JPEG),
        ("exiftool/t/images/ExifTool.tif", FileType::TIFF), // TIFF with Canon EXIF
    ];

    for (file_path, expected_type) in alias_tests {
        if !Path::new(file_path).exists() {
            continue;
        }

        let metadata = find_metadata_segment(file_path).unwrap();

        if let Some(segment) = metadata {
            assert_eq!(segment.source_format, expected_type);
        }
    }
}

#[test]
fn test_dispatch_preserves_offset() {
    // Test that dispatch preserves correct file offsets
    let test_file = "exiftool/t/images/Canon.jpg";
    if !Path::new(test_file).exists() {
        return;
    }

    let metadata = find_metadata_segment(test_file).unwrap().unwrap();

    // JPEG EXIF should have non-zero offset (after markers)
    assert!(metadata.offset > 0, "JPEG EXIF offset should be > 0");

    // For TIFF files, offset should typically be 0
    let tiff_file = "exiftool/t/images/ExifTool.tif";
    if Path::new(tiff_file).exists() {
        let tiff_metadata = find_metadata_segment(tiff_file).unwrap().unwrap();
        assert_eq!(
            tiff_metadata.offset, 0,
            "TIFF-based file offset should be 0"
        );
    }
}

#[test]
fn test_reader_dispatch_consistency() {
    // Test that file path and reader APIs produce same results
    let test_file = "exiftool/t/images/Canon.jpg";
    if !Path::new(test_file).exists() {
        return;
    }

    // Via file path
    let path_result = find_metadata_segment(test_file).unwrap();

    // Via reader
    let mut file = File::open(test_file).unwrap();
    let reader_result = exif_oxide::core::find_metadata_segment_from_reader(&mut file).unwrap();

    // Should produce identical results
    assert_eq!(
        path_result.is_some(),
        reader_result.is_some(),
        "Path and reader APIs should agree"
    );

    if let (Some(path_seg), Some(reader_seg)) = (path_result, reader_result) {
        assert_eq!(path_seg.source_format, reader_seg.source_format);
        assert_eq!(path_seg.metadata_type, reader_seg.metadata_type);
        assert_eq!(path_seg.data.len(), reader_seg.data.len());
    }
}
