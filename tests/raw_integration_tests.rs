//! Integration tests for RAW format support
//!
//! This module tests the complete RAW processing pipeline from file detection
//! through format-specific processing to metadata extraction.
//!
//! Following CLAUDE.md guidance: "your current approach has been problematic in the past"
//! - these tests use actual ExifTool test files rather than synthetic data
//! - this ensures we handle real-world camera quirks and edge cases

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
    let result = extract_metadata(path, false);
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
    let result = extract_metadata(temp_file.path(), false);
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
fn test_unrecognized_raw_file() {
    // Create file with .raw extension but no valid magic bytes
    // This should fail at file detection stage, not RAW processing
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut temp_file = NamedTempFile::with_suffix(".raw").expect("Failed to create temp file");
    let mut data = vec![0u8; 156];

    // Add invalid magic at offset 0x19 (not ARECOYK, II, or MM)
    data[0x19..0x19 + 7].copy_from_slice(b"WRONGXY");

    temp_file
        .write_all(&data)
        .expect("Failed to write test data");
    temp_file.flush().expect("Failed to flush");

    // This should be handled as an unknown file type, not as RAW
    let result = extract_metadata(temp_file.path(), false);
    if result.is_err() {
        println!(
            "✅ Error processing unrecognized .raw file: {:?}",
            result.err()
        );
        // Expected: our file detection is strict and rejects invalid magic
        return;
    }

    // If we get here, the file was processed successfully (shouldn't happen with our strict detection)
    let metadata = result.unwrap();

    // Should have basic file tags
    assert!(!metadata.tags.is_empty(), "Should have basic file tags");

    // Check that FileType was NOT detected as RAW
    let file_type_entries: Vec<_> = metadata
        .tags
        .iter()
        .filter(|entry| entry.name == "FileType")
        .collect();

    if !file_type_entries.is_empty() {
        if let TagValue::String(file_type) = &file_type_entries[0].value {
            // Should be detected as unknown, not RAW
            assert_ne!(
                file_type, "RAW",
                "FileType should not be RAW for invalid magic"
            );
            println!("File correctly detected as: {file_type}");
        }
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
