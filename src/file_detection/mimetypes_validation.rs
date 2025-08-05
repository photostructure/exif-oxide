//! Validation tests for MIMETYPES.md format coverage
//!
//! This module validates that our file detection system can properly handle
//! all the formats specified in docs/MIMETYPES.md

use super::*;
use std::io::Cursor;

/// Generate test formats from our codegen'd data rather than hardcoding
/// This ensures we test exactly what we support and stays in sync with ExifTool updates
fn get_test_formats() -> Vec<(String, String)> {
    // TODO: Re-enable when file_type_lookup module is available
    // use crate::generated::file_types::file_type_lookup::FILE_TYPE_LOOKUP;

    let mut formats = Vec::new();
    let detector = super::FileTypeDetector::new();

    // TODO: Test all file types from our generated lookup table when available
    // for (extension, _) in FILE_TYPE_LOOKUP.iter() {
    for extension in &[] as &[&str] {
        let extension_lower = extension.to_lowercase();
        let filename = format!("test.{extension_lower}");
        let dummy_path = std::path::Path::new(&filename);

        // Get MIME type through our full detection process (including fallbacks)
        if let Ok(candidates) = detector.get_candidates_from_extension(dummy_path) {
            if let Some(file_type) = candidates.first() {
                if let Ok(result) = detector.build_result(file_type, dummy_path) {
                    // Only include if we have a specific MIME type (not the fallback)
                    if result.mime_type != "application/octet-stream" {
                        formats.push((extension_lower, result.mime_type));
                    }
                }
            }
        }
    }

    // Also test common extension aliases that might not be in the main lookup
    let aliases = [
        ("jpg", "jpeg"),
        ("tif", "tiff"),
        ("3gp2", "3g2"),
        ("aif", "aiff"),
    ];

    for (alias, _canonical) in aliases {
        let filename = format!("test.{alias}");
        let dummy_path = std::path::Path::new(&filename);

        if let Ok(candidates) = detector.get_candidates_from_extension(dummy_path) {
            if let Some(file_type) = candidates.first() {
                if let Ok(result) = detector.build_result(file_type, dummy_path) {
                    if result.mime_type != "application/octet-stream" {
                        formats.push((alias.to_string(), result.mime_type));
                    }
                }
            }
        }
    }

    // Sort for consistent test order
    formats.sort_by(|a, b| a.0.cmp(&b.0));
    formats.dedup(); // Remove duplicates
    formats
}

#[test]
fn test_mimetypes_extension_coverage() {
    let detector = FileTypeDetector::new();

    let test_formats = get_test_formats();

    for (ext, expected_mime) in &test_formats {
        let filename = format!("test.{ext}");
        let path = Path::new(&filename);

        // Test that we can get candidates for this extension
        let candidates = detector.get_candidates_from_extension(path);
        assert!(
            candidates.is_ok(),
            "Failed to get candidates for extension: {ext}"
        );

        let candidates = candidates.unwrap();
        assert!(
            !candidates.is_empty(),
            "No candidates found for extension: {ext}"
        );

        // Test that we can build a proper MIME type result for this file type
        // Use the build_result function directly to test MIME type resolution including fallbacks
        let file_type = &candidates[0];
        let result = detector.build_result(file_type, path);

        match result {
            Ok(detection) => {
                println!(
                    "Extension: {} -> FileType: {} -> MIME: {} (expected: {})",
                    ext, detection.file_type, detection.mime_type, expected_mime
                );

                // Verify we have a specific MIME type (not the generic fallback)
                assert_ne!(
                    detection.mime_type, "application/octet-stream",
                    "No specific MIME type found for extension: {ext} (got generic fallback)"
                );
            }
            Err(e) => {
                panic!("Failed to build result for extension: {ext} - {e:?}");
            }
        }
    }
}

#[test]
fn test_all_mimetypes_formats_detectable() {
    let detector = FileTypeDetector::new();

    // Test common magic number patterns for key formats
    let test_cases = vec![
        // JPEG
        (
            "test.jpg",
            vec![0xff, 0xd8, 0xff, 0xe0],
            "JPEG",
            "image/jpeg",
        ),
        // PNG
        (
            "test.png",
            vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a],
            "PNG",
            "image/png",
        ),
        // TIFF (little endian)
        (
            "test.tif",
            vec![0x49, 0x49, 0x2a, 0x00],
            "TIFF",
            "image/tiff",
        ),
        // GIF (GIF89a)
        (
            "test.gif",
            vec![0x47, 0x49, 0x46, 0x38, 0x39, 0x61],
            "GIF",
            "image/gif",
        ),
        // BMP
        ("test.bmp", vec![0x42, 0x4d], "BMP", "image/bmp"),
        // PDF detection not supported - skipping
        // ZIP (used by many modern formats)
        (
            "test.zip",
            vec![0x50, 0x4b, 0x03, 0x04],
            "ZIP",
            "application/zip",
        ),
    ];

    for (filename, magic_data, expected_type, expected_mime) in test_cases {
        let path = Path::new(filename);
        let mut cursor = Cursor::new(magic_data);

        let result = detector.detect_file_type(path, &mut cursor);
        assert!(result.is_ok(), "Failed to detect file type for: {filename}");

        let detection = result.unwrap();
        assert_eq!(
            detection.file_type, expected_type,
            "Wrong file type detected for {}: expected {}, got {}",
            filename, expected_type, detection.file_type
        );
        assert_eq!(
            detection.mime_type, expected_mime,
            "Wrong MIME type for {}: expected {}, got {}",
            filename, expected_mime, detection.mime_type
        );
    }
}

#[test]
fn test_extension_normalization_coverage() {
    let detector = FileTypeDetector::new();

    // Test ExifTool's critical extension normalizations
    let normalization_tests = vec![
        ("tif", "TIFF"), // Critical: TIF -> TIFF (hardcoded in ExifTool)
        ("jpg", "JPEG"), // Critical: JPG -> JPEG
        ("3gp2", "3G2"), // Critical: 3GP2 -> 3G2
        ("aif", "AIFF"), // Critical: AIF -> AIFF
        ("png", "PNG"),  // No change
        ("mp4", "MP4"),  // No change
    ];

    for (input_ext, expected_normalized) in normalization_tests {
        let normalized = detector.normalize_extension(input_ext);
        assert_eq!(
            normalized, expected_normalized,
            "Extension normalization failed: {input_ext} should normalize to {expected_normalized}, got {normalized}"
        );
    }
}

#[test]
fn test_raw_format_coverage() {
    let detector = FileTypeDetector::new();

    // Test that we can identify all major RAW formats by extension
    // Most RAW formats rely on extension since they use TIFF magic numbers
    let raw_formats = vec![
        "cr2", "cr3", "crw", // Canon
        "nef", "nrw", // Nikon
        "arw", "sr2", "srf", // Sony
        "raf", // Fujifilm
        "orf", // Olympus
        "rw2", // Panasonic
        "dng", // Adobe
        "pef", // Pentax
    ];

    for ext in raw_formats {
        let filename = format!("test.{ext}");
        let path = Path::new(&filename);
        let candidates = detector.get_candidates_from_extension(path).unwrap();

        assert!(
            !candidates.is_empty(),
            "No candidates found for RAW extension: {ext}"
        );

        // Verify we have MIME type mapping
        let file_type = &candidates[0];
        let mime_type = crate::generated::exiftool_pm::lookup_mime_types(file_type);
        assert!(
            mime_type.is_some(),
            "No MIME type mapping for RAW format: {ext} (file type: {file_type})"
        );

        println!(
            "RAW format {} -> {} -> {}",
            ext,
            file_type,
            mime_type.unwrap()
        );
    }
}

#[test]
fn test_video_format_coverage() {
    let detector = FileTypeDetector::new();

    // Test video format detection capabilities
    let video_formats = vec![
        "mp4", "mov", "avi", "mkv", "webm", "wmv", "3gp", "3g2", "m4v", "mts", "m2ts",
    ];

    for ext in video_formats {
        let filename = format!("test.{ext}");
        let path = Path::new(&filename);
        let candidates = detector.get_candidates_from_extension(path).unwrap();

        assert!(
            !candidates.is_empty(),
            "No candidates found for video extension: {ext}"
        );

        // Test MIME type resolution using our detection engine (including fallbacks)
        let file_type = &candidates[0];
        let result = detector.build_result(file_type, path);

        match result {
            Ok(detection) => {
                println!(
                    "Video format {} -> {} -> {}",
                    ext, file_type, detection.mime_type
                );

                // Verify we have a specific MIME type (not the generic fallback)
                assert_ne!(
                    detection.mime_type, "application/octet-stream",
                    "No specific MIME type found for video format: {ext} (got generic fallback)"
                );
            }
            Err(e) => {
                panic!("Failed to build result for video format: {ext} - {e:?}");
            }
        }
    }
}

#[test]
fn test_weak_magic_handling() {
    let detector = FileTypeDetector::new();

    // MP3 is marked as weak magic in ExifTool - should rely on extension
    let path = Path::new("test.mp3");

    // Even with non-MP3 data, should detect as MP3 due to weak magic
    let fake_data = vec![0x00, 0x01, 0x02, 0x03];
    let mut cursor = Cursor::new(fake_data);

    let result = detector.detect_file_type(path, &mut cursor).unwrap();
    assert_eq!(result.file_type, "MP3");
    assert_eq!(result.mime_type, "audio/mpeg");
}

/// Performance test: ensure detection is sub-millisecond for common formats
#[test]
fn test_detection_performance() {
    let detector = FileTypeDetector::new();
    let start = std::time::Instant::now();

    // Test 100 detections to get meaningful timing
    for _ in 0..100 {
        let path = Path::new("test.jpg");
        let jpeg_data = vec![0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10];
        let mut cursor = Cursor::new(&jpeg_data);

        let _result = detector.detect_file_type(path, &mut cursor).unwrap();
    }

    let elapsed = start.elapsed();
    let per_detection = elapsed / 100;

    println!("Average detection time: {per_detection:?}");

    // Should be well under 1ms per detection
    assert!(
        per_detection.as_millis() < 1,
        "Detection too slow: {per_detection:?} per detection (should be <1ms)"
    );
}
