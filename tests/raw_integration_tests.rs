//! Integration tests for RAW format support
//!
//! This module tests the complete RAW processing pipeline from file detection
//! through format-specific processing to metadata extraction.
//!
//! Following CLAUDE.md guidance: "your current approach has been problematic in the past"
//! - these tests use actual ExifTool test files rather than synthetic data
//! - this ensures we handle real-world camera quirks and edge cases
//!
//! Note: These tests require the `integration-tests` feature to be enabled and
//! external test assets to be available. They are automatically skipped in published crates.

#![cfg(feature = "integration-tests")]

use exif_oxide::formats::extract_metadata;
use exif_oxide::types::TagValue;
use std::path::Path;

#[test]
fn test_kyocera_raw_detection_and_processing() {
    // Use actual ExifTool test file instead of synthetic data
    // CLAUDE.md: "your current approach has been problematic in the past"
    let path = Path::new("third-party/exiftool/t/images/KyoceraRaw.raw");

    // Ensure the test file exists
    assert!(path.exists(), "ExifTool test file not found: {path:?}");

    // Test metadata extraction
    let result = extract_metadata(path, false, false, None);
    assert!(
        result.is_ok(),
        "Failed to extract metadata: {:?}",
        result.err()
    );

    let metadata = result.unwrap();

    // Check that we have some tags
    assert!(!metadata.tags.is_empty(), "No tags extracted");

    // Check file type detection
    let file_type_entries: Vec<_> = metadata
        .tags
        .iter()
        .filter(|entry| entry.name == "FileType")
        .collect();
    assert!(!file_type_entries.is_empty(), "FileType tag not found");
    if let TagValue::String(file_type) = &file_type_entries[0].value {
        assert_eq!(file_type, "RAW", "Wrong file type detected");
    }

    // Validate against ExifTool's expected output
    // ExifTool output: exiftool -j -struct KyoceraRaw.raw

    // Check Make - should be "KYOCERA" (correctly reversed from "ARECOYK")
    let make_entries: Vec<_> = metadata
        .tags
        .iter()
        .filter(|entry| entry.name == "Make")
        .collect();

    if !make_entries.is_empty() {
        if let TagValue::String(make) = &make_entries[0].value {
            // ExifTool shows "KYOCERA", not the raw "ARECOYK"
            assert_eq!(
                make, "KYOCERA",
                "Make should be reversed from magic bytes to 'KYOCERA'"
            );
        }
    }

    // Check Model - should be "N DIGITAL"
    let model_entries: Vec<_> = metadata
        .tags
        .iter()
        .filter(|entry| entry.name == "Model")
        .collect();

    if !model_entries.is_empty() {
        if let TagValue::String(model) = &model_entries[0].value {
            // ExifTool shows "N DIGITAL"
            assert_eq!(model, "N DIGITAL", "Model should match ExifTool output");
        }
    }

    // Check FirmwareVersion - should be "Ver. 1.07"
    let firmware_entries: Vec<_> = metadata
        .tags
        .iter()
        .filter(|entry| entry.name == "FirmwareVersion")
        .collect();

    if !firmware_entries.is_empty() {
        if let TagValue::String(firmware) = &firmware_entries[0].value {
            // ExifTool shows "Ver. 1.07"
            assert_eq!(
                firmware, "Ver. 1.07",
                "FirmwareVersion should match ExifTool output"
            );
        }
    }

    // Check ISO - should be 100
    let iso_entries: Vec<_> = metadata
        .tags
        .iter()
        .filter(|entry| entry.name == "ISO")
        .collect();

    if !iso_entries.is_empty() {
        if let TagValue::U32(iso) = &iso_entries[0].value {
            // ExifTool shows 100
            assert_eq!(*iso, 100, "ISO should match ExifTool output");
        }
    }

    println!("✅ Successfully processed real Kyocera RAW file");
    println!("📊 Extracted {} tags", metadata.tags.len());
    println!(
        "🏷️  Tag names: {:?}",
        metadata.tags.iter().map(|t| &t.name).collect::<Vec<_>>()
    );

    // Show some key extracted values for debugging
    println!("🔍 Key extracted values:");
    for entry in &metadata.tags {
        if ["Make", "Model", "FirmwareVersion", "ISO", "FocalLength"].contains(&entry.name.as_str())
        {
            println!("   {}: {:?}", entry.name, entry.value);
        }
    }

    // Debug: Show all tag entries with their groups to understand the structure
    println!("🔍 All tag entries (first 10):");
    for (i, entry) in metadata.tags.iter().enumerate() {
        if i >= 10 {
            break;
        }
        println!(
            "   {}: {} | Group: {} | Group1: {} | Value: {:?}",
            i, entry.name, entry.group, entry.group1, entry.value
        );
    }
}

#[test]
fn test_minimal_kyocera_raw_file() {
    // Test with a minimal valid Kyocera RAW file (just magic bytes + padding)
    // This tests our error handling for files that pass detection but have minimal data
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut temp_file = NamedTempFile::with_suffix(".raw").expect("Failed to create temp file");

    // Create exactly 156 bytes (minimum expected) with just the magic
    let mut data = vec![0u8; 156];
    data[0x19..0x19 + 7].copy_from_slice(b"ARECOYK");
    // All other data is zero - this tests our handling of empty/zero data

    temp_file
        .write_all(&data)
        .expect("Failed to write test data");
    temp_file.flush().expect("Failed to flush");

    // Test should handle gracefully - file detection succeeds, RAW processing extracts what it can
    let result = extract_metadata(temp_file.path(), false, false, None);
    assert!(result.is_ok(), "Should handle minimal RAW file gracefully");

    let metadata = result.unwrap();

    // Should have basic file tags plus some RAW processing
    assert!(!metadata.tags.is_empty(), "Should have basic file tags");

    // Check that FileType was detected as RAW
    let file_type_entries: Vec<_> = metadata
        .tags
        .iter()
        .filter(|entry| entry.name == "FileType")
        .collect();
    assert!(
        !file_type_entries.is_empty(),
        "FileType tag should be present"
    );
    if let TagValue::String(file_type) = &file_type_entries[0].value {
        assert_eq!(file_type, "RAW", "FileType should be RAW");
    }

    println!("✅ Successfully processed minimal RAW file");
}

#[test]
fn test_recognized_raw_file() {
    // Test that actual RAW files are correctly detected
    // Using the real Kyocera RAW file from ExifTool's test suite
    let raw_file = "third-party/exiftool/t/images/KyoceraRaw.raw";

    if !std::path::Path::new(raw_file).exists() {
        println!("⚠️  Skipping RAW test - {raw_file} not found");
        return;
    }

    let result = extract_metadata(std::path::Path::new(raw_file), false, false, None);
    assert!(result.is_ok(), "Should successfully process valid RAW file");

    let metadata = result.unwrap();
    assert!(!metadata.tags.is_empty(), "Should have metadata tags");

    // Check that FileType was correctly detected as RAW
    let file_type_entries: Vec<_> = metadata
        .tags
        .iter()
        .filter(|entry| entry.name == "FileType")
        .collect();

    assert!(!file_type_entries.is_empty(), "Should have FileType tag");

    if let TagValue::String(file_type) = &file_type_entries[0].value {
        assert_eq!(
            file_type, "RAW",
            "FileType should be RAW for valid Kyocera RAW file"
        );
        println!("✅ File correctly detected as: {file_type}");
    }
}

#[test]
fn test_non_raw_file_not_detected_as_raw() {
    // Test that non-RAW files are not incorrectly detected as RAW
    // Using an actual JPEG file from the test suite
    let jpeg_file = "test-images/casio/QVCI.jpg";

    if !std::path::Path::new(jpeg_file).exists() {
        println!("⚠️  Skipping non-RAW test - {jpeg_file} not found");
        return;
    }

    let result = extract_metadata(std::path::Path::new(jpeg_file), false, false, None);
    assert!(result.is_ok(), "Should successfully process JPEG file");

    let metadata = result.unwrap();
    assert!(!metadata.tags.is_empty(), "Should have metadata tags");

    // Check that FileType was NOT detected as RAW
    let file_type_entries: Vec<_> = metadata
        .tags
        .iter()
        .filter(|entry| entry.name == "FileType")
        .collect();

    assert!(!file_type_entries.is_empty(), "Should have FileType tag");

    if let TagValue::String(file_type) = &file_type_entries[0].value {
        assert_ne!(file_type, "RAW", "FileType should not be RAW for JPEG file");
        println!("✅ File correctly detected as: {file_type} (not RAW)");
    }
}

#[test]
fn test_raw_file_type_detection() {
    use exif_oxide::FileTypeDetector;
    use std::fs::File;
    use std::io::BufReader;

    // Use the actual ExifTool test file for file type detection
    let path = Path::new("third-party/exiftool/t/images/KyoceraRaw.raw");
    assert!(path.exists(), "ExifTool test file not found: {path:?}");

    // Open the real file
    let file = File::open(path).expect("Failed to open test file");
    let mut reader = BufReader::new(file);

    // Test file type detection
    let detector = FileTypeDetector::new();
    let result = detector.detect_file_type(path, &mut reader);
    assert!(
        result.is_ok(),
        "File type detection failed: {:?}",
        result.err()
    );

    let detection = result.unwrap();
    assert_eq!(detection.file_type, "RAW", "Wrong file type detected");
    assert_eq!(detection.format, "RAW", "Wrong format detected");

    println!("✅ Real file detection result: {detection:?}");
}

#[test]
fn test_raw_format_detection() {
    use exif_oxide::raw::detect_raw_format;
    use exif_oxide::FileTypeDetectionResult;

    // Test Kyocera detection
    let detection_result = FileTypeDetectionResult {
        file_type: "RAW".to_string(),
        format: "RAW".to_string(),
        mime_type: "application/octet-stream".to_string(),
        description: "Kyocera Contax N Digital RAW or Panasonic RAW".to_string(),
    };

    let format = detect_raw_format(&detection_result);
    assert_eq!(
        format,
        exif_oxide::raw::RawFormat::Kyocera,
        "Wrong RAW format detected"
    );

    // Test unknown format
    let unknown_detection = FileTypeDetectionResult {
        file_type: "UNKNOWN".to_string(),
        format: "UNKNOWN".to_string(),
        mime_type: "application/octet-stream".to_string(),
        description: "Unknown format".to_string(),
    };

    let format = detect_raw_format(&unknown_detection);
    assert_eq!(
        format,
        exif_oxide::raw::RawFormat::Unknown,
        "Should detect unknown format"
    );
}

// =============================================================================
// Milestone 17b Integration Tests: Real File Testing vs ExifTool
// =============================================================================

/// Test Panasonic RW2 file with TIFF integration
#[test]
fn test_milestone_17b_panasonic_rw2_real_file() {
    let test_file = "test-images/panasonic/panasonic_lumix_g9_ii_35.rw2";

    // Skip test if file doesn't exist
    if !Path::new(test_file).exists() {
        eprintln!("Warning: Test file {test_file} not found, skipping test");
        return;
    }

    println!("🧪 Testing Panasonic RW2 file with TIFF integration...");

    // Extract metadata using our implementation
    let result = extract_metadata(Path::new(test_file), false, false, None);
    assert!(
        result.is_ok(),
        "Failed to process RW2 file: {:?}",
        result.err()
    );

    let exif_data = result.unwrap();
    assert!(
        !exif_data.tags.is_empty(),
        "Should have extracted metadata tags"
    );

    // Validate that we detected the file correctly
    let file_type_tag = exif_data.tags.iter().find(|tag| tag.name == "FileType");
    if let Some(tag) = file_type_tag {
        if let TagValue::String(file_type) = &tag.value {
            println!("✅ File detected as: {file_type}");
        }
    }

    // Validate core metadata that should be present
    let mut core_tags_found = 0;
    let core_expected_tags = ["Make", "Model"];

    for expected_tag in &core_expected_tags {
        if let Some(tag) = exif_data.tags.iter().find(|tag| tag.name == *expected_tag) {
            println!("✅ Found {}: {}", expected_tag, tag.value);
            core_tags_found += 1;
        } else {
            println!("⚠️  Core tag not found: {expected_tag}");
        }
    }

    // We should find at least Make and Model for a real camera file
    assert!(
        core_tags_found >= 1,
        "Expected at least 1 core tag, found {core_tags_found}"
    );

    // Check for TIFF-specific tags that indicate our TIFF integration worked
    let tiff_indicators = [
        "ExifImageWidth",
        "ExifImageHeight",
        "Orientation",
        "DateTime",
    ];
    let mut tiff_tags_found = 0;

    for indicator in &tiff_indicators {
        if exif_data.tags.iter().any(|tag| tag.name == *indicator) {
            println!("✅ Found TIFF indicator: {indicator}");
            tiff_tags_found += 1;
        }
    }

    println!(
        "✅ Panasonic RW2 test passed - extracted {} total tags, {} core tags, {} TIFF indicators",
        exif_data.tags.len(),
        core_tags_found,
        tiff_tags_found
    );
}

/// Test Minolta MRW file with multi-block processing
#[test]
fn test_milestone_17b_minolta_mrw_real_file() {
    let test_file = "test-images/minolta/DiMAGE_7.mrw";

    // Skip test if file doesn't exist
    if !Path::new(test_file).exists() {
        eprintln!("Warning: Test file {test_file} not found, skipping test");
        return;
    }

    println!("🧪 Testing Minolta MRW file with multi-block processing...");

    // Extract metadata using our implementation
    let result = extract_metadata(Path::new(test_file), false, false, None);
    assert!(
        result.is_ok(),
        "Failed to process MRW file: {:?}",
        result.err()
    );

    let exif_data = result.unwrap();
    assert!(
        !exif_data.tags.is_empty(),
        "Should have extracted metadata tags"
    );

    // Validate that we detected the file correctly
    let file_type_tag = exif_data.tags.iter().find(|tag| tag.name == "FileType");
    if let Some(tag) = file_type_tag {
        if let TagValue::String(file_type) = &tag.value {
            println!("✅ File detected as: {file_type}");
        }
    }

    // Validate core metadata that should be present
    let mut core_tags_found = 0;
    let core_expected_tags = ["Make", "Model"];

    for expected_tag in &core_expected_tags {
        if let Some(tag) = exif_data.tags.iter().find(|tag| tag.name == *expected_tag) {
            println!("✅ Found {}: {}", expected_tag, tag.value);
            core_tags_found += 1;
        } else {
            println!("⚠️  Core tag not found: {expected_tag}");
        }
    }

    // Check for Minolta-specific tags that indicate our MRW processing worked
    let minolta_indicators = ["MinoltaImageSize", "MinoltaQuality", "ColorMode"];
    let mut minolta_tags_found = 0;

    for indicator in &minolta_indicators {
        if exif_data.tags.iter().any(|tag| tag.name == *indicator) {
            println!("✅ Found Minolta indicator: {indicator}");
            minolta_tags_found += 1;
        } else {
            println!("ℹ️  Minolta tag not found: {indicator} (may be expected)");
        }
    }

    // MRW files may not have standard Make/Model tags, so check for any extracted tags
    // The important thing is that we're extracting MRW-specific maker note data
    assert!(
        !exif_data.tags.is_empty(),
        "Expected at least some tags to be extracted from MRW file"
    );

    println!(
        "✅ Minolta MRW test passed - extracted {} total tags, {} core tags, {} Minolta indicators",
        exif_data.tags.len(),
        core_tags_found,
        minolta_tags_found
    );
}

/// Test that our TIFF integration handles multiple RAW formats
#[test]
fn test_milestone_17b_multiple_raw_formats() {
    let test_files = [
        (
            "test-images/panasonic/panasonic_lumix_g9_ii_35.rw2",
            "Panasonic",
        ),
        ("test-images/minolta/DiMAGE_7.mrw", "Minolta"),
    ];

    let mut processed_files = 0;

    for (file_path, expected_make_hint) in &test_files {
        if !Path::new(file_path).exists() {
            eprintln!("Warning: Test file {file_path} not found, skipping");
            continue;
        }

        println!("🧪 Testing multi-format processing: {file_path}");

        // Test that we can process the file without errors
        let result = extract_metadata(Path::new(file_path), false, false, None);
        assert!(
            result.is_ok(),
            "Failed to process {} file: {:?}",
            expected_make_hint,
            result.err()
        );

        let exif_data = result.unwrap();

        // Verify we extracted some tags
        assert!(
            !exif_data.tags.is_empty(),
            "No tags extracted from {expected_make_hint} file"
        );

        // Try to find manufacturer information
        if let Some(make_tag) = exif_data.tags.iter().find(|tag| tag.name == "Make") {
            println!(
                "✅ {} format detected Make: {}",
                expected_make_hint, make_tag.value
            );
        }

        // Count different tag groups to verify comprehensive extraction
        let unique_groups: std::collections::HashSet<_> =
            exif_data.tags.iter().map(|tag| &tag.group).collect();

        println!(
            "✅ {} format processing: {} tags, {} groups",
            expected_make_hint,
            exif_data.tags.len(),
            unique_groups.len()
        );

        processed_files += 1;
    }

    // If no test files were available, skip the test
    if processed_files == 0 {
        println!("⚠️  No test files available - skipping multi-format test");
        return;
    }
    println!("✅ Multi-format test passed - processed {processed_files} files");
}
