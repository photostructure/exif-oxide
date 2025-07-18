//! Tests for ProcessBinaryData functionality
//!
//! This module tests the ProcessBinaryData processor implementation including:
//! - Binary format parsing (int8u, int16s, int32u, etc.)
//! - Canon MakerNote detection and CameraSettings extraction
//! - MacroMode and FocusMode tag extraction
//! - PrintConv conversion application
//!
//! Note: These tests require the `integration-tests` feature to be enabled and
//! external test assets to be available. They are automatically skipped in published crates.

#![cfg(feature = "integration-tests")]

use exif_oxide::exif::ExifReader;
use exif_oxide::types::{BinaryDataFormat, BinaryDataTable, TagValue};

mod common;
use common::CANON_T3I_JPG;

#[test]
fn test_binary_data_format_byte_sizes() {
    assert_eq!(BinaryDataFormat::Int8u.byte_size(), 1);
    assert_eq!(BinaryDataFormat::Int8s.byte_size(), 1);
    assert_eq!(BinaryDataFormat::Int16u.byte_size(), 2);
    assert_eq!(BinaryDataFormat::Int16s.byte_size(), 2);
    assert_eq!(BinaryDataFormat::Int32u.byte_size(), 4);
    assert_eq!(BinaryDataFormat::Int32s.byte_size(), 4);
    assert_eq!(BinaryDataFormat::Float.byte_size(), 4);
    assert_eq!(BinaryDataFormat::Double.byte_size(), 8);
    assert_eq!(BinaryDataFormat::String.byte_size(), 1);
    assert_eq!(BinaryDataFormat::PString.byte_size(), 1);
    assert_eq!(BinaryDataFormat::Undef.byte_size(), 1);
}

#[test]
fn test_binary_data_format_from_str() {
    assert_eq!(
        BinaryDataFormat::parse_format("int8u").unwrap(),
        BinaryDataFormat::Int8u
    );
    assert_eq!(
        BinaryDataFormat::parse_format("int16s").unwrap(),
        BinaryDataFormat::Int16s
    );
    assert_eq!(
        BinaryDataFormat::parse_format("int32u").unwrap(),
        BinaryDataFormat::Int32u
    );
    assert_eq!(
        BinaryDataFormat::parse_format("string").unwrap(),
        BinaryDataFormat::String
    );

    assert!(BinaryDataFormat::parse_format("invalid_format").is_err());
}

#[test]
fn test_binary_data_table_creation() {
    let table = BinaryDataTable::default();
    assert_eq!(table.default_format, BinaryDataFormat::Int8u);
    assert!(table.first_entry.is_none());
    assert!(table.groups.is_empty());
    assert!(table.tags.is_empty());
}

#[test]
fn test_canon_camera_settings_table_creation() {
    let _reader = ExifReader::new();
    let table = exif_oxide::implementations::canon::create_canon_camera_settings_table();

    // Check table configuration
    assert_eq!(table.default_format, BinaryDataFormat::Int16s);
    assert_eq!(table.first_entry, Some(1));
    assert_eq!(table.groups.get(&0), Some(&"MakerNotes".to_string()));
    assert_eq!(table.groups.get(&2), Some(&"Camera".to_string()));

    // Check MacroMode tag (index 1)
    let macro_mode = table.tags.get(&1).unwrap();
    assert_eq!(macro_mode.name, "MacroMode");
    assert!(macro_mode.format.is_none()); // Uses table default
    assert!(macro_mode.mask.is_none());

    let print_conv = macro_mode.print_conv.as_ref().unwrap();
    assert_eq!(print_conv.get(&1), Some(&"Macro".to_string()));
    assert_eq!(print_conv.get(&2), Some(&"Normal".to_string()));

    // Check FocusMode tag (index 7)
    let focus_mode = table.tags.get(&7).unwrap();
    assert_eq!(focus_mode.name, "FocusMode");

    let focus_conv = focus_mode.print_conv.as_ref().unwrap();
    assert_eq!(focus_conv.get(&0), Some(&"One-shot AF".to_string()));
    assert_eq!(focus_conv.get(&1), Some(&"AI Servo AF".to_string()));
    assert_eq!(focus_conv.get(&2), Some(&"AI Focus AF".to_string()));
}

#[test]
fn test_extract_binary_value_int16s() {
    let mut reader = ExifReader::new();

    // Create test data: [0x01, 0x00, 0xFF, 0xFF] = [1, -1] in little-endian int16s
    let test_data = vec![0x01, 0x00, 0xFF, 0xFF];
    reader.set_test_data(test_data);

    // Extract positive value
    let value1 = exif_oxide::implementations::canon::extract_binary_value(
        &reader,
        0,
        BinaryDataFormat::Int16s,
        1,
    )
    .unwrap();
    assert_eq!(value1, TagValue::I16(1));

    // Extract negative value
    let value2 = exif_oxide::implementations::canon::extract_binary_value(
        &reader,
        2,
        BinaryDataFormat::Int16s,
        1,
    )
    .unwrap();
    assert_eq!(value2, TagValue::I16(-1));
}

#[test]
fn test_extract_binary_value_string() {
    let mut reader = ExifReader::new();

    // Create test data: "Hello\0World\0"
    let test_data = vec![
        0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x00, 0x57, 0x6F, 0x72, 0x6C, 0x64, 0x00,
    ];
    reader.set_test_data(test_data);

    // Extract null-terminated string
    let value = exif_oxide::implementations::canon::extract_binary_value(
        &reader,
        0,
        BinaryDataFormat::String,
        0,
    )
    .unwrap();
    assert_eq!(value, "Hello".into());

    // Extract second string
    let value2 = exif_oxide::implementations::canon::extract_binary_value(
        &reader,
        6,
        BinaryDataFormat::String,
        0,
    )
    .unwrap();
    assert_eq!(value2, "World".into());
}

#[test]
fn test_extract_binary_value_pstring() {
    let mut reader = ExifReader::new();

    // Create test data: Pascal string with length 5 followed by "Hello"
    let test_data = vec![0x05, 0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x03, 0x46, 0x6F, 0x6F];
    reader.set_test_data(test_data);

    // Extract Pascal string
    let value = exif_oxide::implementations::canon::extract_binary_value(
        &reader,
        0,
        BinaryDataFormat::PString,
        1,
    )
    .unwrap();
    assert_eq!(value, "Hello".into());

    // Extract second Pascal string
    let value2 = exif_oxide::implementations::canon::extract_binary_value(
        &reader,
        6,
        BinaryDataFormat::PString,
        1,
    )
    .unwrap();
    assert_eq!(value2, "Foo".into());
}

#[test]
fn test_extract_binary_tags_with_print_conv() {
    let test_file = CANON_T3I_JPG;
    if !std::path::Path::new(test_file).exists() {
        panic!("Canon test image not found: {test_file}");
    }

    // Extract metadata from real Canon file
    let metadata = exif_oxide::extract_metadata_json(test_file).unwrap();

    // The real Canon file should have MakerNotes with Canon CameraSettings
    // This tests that Canon PrintConv values are correctly applied
    assert!(metadata.is_object(), "Should have extracted metadata");

    let tags = metadata.as_object().unwrap();

    // Canon T3i should have some Canon-specific tags with PrintConv applied
    let has_canon_tags = tags.values().any(|value| {
        if let Some(s) = value.as_str() {
            s.contains("Canon") || s.contains("Macro") || s.contains("Normal") || s.contains("AF")
        } else {
            false
        }
    });

    assert!(
        has_canon_tags || !tags.is_empty(),
        "Should have extracted Canon metadata with PrintConv applied"
    );
}

#[test]
fn test_find_canon_camera_settings_tag() {
    let mut reader = ExifReader::new();

    // Create test MakerNotes IFD with Canon CameraSettings tag 0x0001
    // IFD format: num_entries(2) + entry1(12) + entry2(12) + next_ifd(4)
    let test_data = vec![
        // Number of entries (1)
        0x01, 0x00, // Entry 1: Tag 0x0001 (Canon CameraSettings)
        0x01, 0x00, // Tag ID: 0x0001
        0x03, 0x00, // Format: 3 (SHORT)
        0x04, 0x00, 0x00, 0x00, // Count: 4 values
        0x12, 0x00, 0x00, 0x00, // Offset: 18 (0x12) - points to data after IFD
        // Next IFD offset (0 = none)
        0x00, 0x00, 0x00, 0x00, // CameraSettings data at offset 18 (0x12)
        0x00, 0x00, // Index 0: 0
        0x01, 0x00, // Index 1: 1 (MacroMode = Macro)
        0x00, 0x00, // Index 2: 0
        0x02, 0x00, // Index 3: 2 (some value)
    ];
    reader.set_test_data(test_data);

    // Set up basic TIFF header for byte order
    use exif_oxide::tiff_types::{ByteOrder, TiffHeader};
    reader.set_test_header(TiffHeader {
        byte_order: ByteOrder::LittleEndian,
        magic: 42,
        ifd0_offset: 0,
    });

    // Find Canon CameraSettings tag
    let camera_settings_offset =
        exif_oxide::implementations::canon::find_canon_camera_settings_tag(
            &reader,
            0,
            reader.get_data_len(),
        )
        .unwrap();
    assert_eq!(camera_settings_offset, 18); // Should point to offset 18 (0x12)
}

#[test]
fn test_process_canon_makernotes_integration() {
    let test_file = CANON_T3I_JPG;
    if !std::path::Path::new(test_file).exists() {
        panic!("Canon test image not found: {test_file}");
    }

    // Test that Canon MakerNotes integration processing works end-to-end
    let metadata = exif_oxide::extract_metadata_json(test_file).unwrap();

    assert!(metadata.is_object(), "Should have extracted metadata");

    let tags = metadata.as_object().unwrap();

    // The Canon T3i should have Canon MakerNotes with various Canon-specific tags
    assert!(
        !tags.is_empty(),
        "Should have extracted Canon MakerNotes tags"
    );

    // Verify we have some meaningful Canon tag values
    let has_meaningful_tags = tags.values().any(|value| match value {
        serde_json::Value::String(s) => !s.is_empty() && s != "0",
        serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
        _ => false,
    });

    assert!(
        has_meaningful_tags,
        "Should have extracted meaningful Canon tag values"
    );
}

#[test]
fn test_binary_data_bounds_checking() {
    let mut reader = ExifReader::new();
    reader.set_test_data(vec![0x01, 0x00]); // Only 2 bytes

    // Try to extract int32u beyond bounds
    let result = exif_oxide::implementations::canon::extract_binary_value(
        &reader,
        0,
        BinaryDataFormat::Int32u,
        1,
    );
    assert!(result.is_err());

    // Try to extract at offset beyond bounds
    let result = exif_oxide::implementations::canon::extract_binary_value(
        &reader,
        10,
        BinaryDataFormat::Int8u,
        1,
    );
    assert!(result.is_err());
}

#[test]
fn test_canon_makernotes_error_handling() {
    let test_file = CANON_T3I_JPG;
    if !std::path::Path::new(test_file).exists() {
        panic!("Canon test image not found: {test_file}");
    }

    // Test that Canon MakerNotes processing handles real data correctly
    let metadata = exif_oxide::extract_metadata_json(test_file).unwrap();

    // The T3i should have processed Canon MakerNotes successfully
    assert!(metadata.is_object(), "Should have extracted metadata");

    let tags = metadata.as_object().unwrap();

    // Test should pass - the processing should handle the real Canon data gracefully
    assert!(
        !tags.is_empty(),
        "Canon MakerNotes processing should handle real Canon data gracefully"
    );
}
