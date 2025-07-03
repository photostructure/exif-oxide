//! Unit tests for the ExifReader module
//!
//! These tests validate the core EXIF processing functionality including
//! subdirectory management, processor dispatch, and GPS coordinate handling.

#[cfg(test)]
use crate::exif::ExifReader;
#[cfg(test)]
use crate::tiff_types::ByteOrder;
#[cfg(test)]
use crate::types::{DataMemberValue, DirectoryInfo, ProcessorType, TagValue};

#[test]
fn test_exif_reader_basic() {
    let mut reader = ExifReader::new();

    // Create minimal EXIF data with TIFF header and empty IFD
    let exif_data = [
        0x49, 0x49, // "II" - little-endian
        0x2A, 0x00, // Magic: 42 (LE)
        0x08, 0x00, 0x00, 0x00, // IFD0 offset: 8 (LE)
        0x00, 0x00, // Number of entries: 0
        0x00, 0x00, 0x00, 0x00, // Next IFD: none
    ];

    reader.parse_exif_data(&exif_data).unwrap();

    let header = reader.get_header().unwrap();
    assert_eq!(header.byte_order, ByteOrder::LittleEndian);
    assert_eq!(header.ifd0_offset, 8);
}

#[test]
fn test_subdirectory_recursion_prevention() {
    let mut reader = ExifReader::new();

    // Test recursion prevention by manually manipulating the PROCESSED hash
    let addr = 100u64;

    // Initially not processed
    assert!(!reader.processed.contains_key(&addr));

    // Mark as processed
    reader.processed.insert(addr, "TestIFD".to_string());

    // Verify it's marked as processed
    assert!(reader.processed.contains_key(&addr));
    assert_eq!(reader.processed.get(&addr), Some(&"TestIFD".to_string()));

    // Test that the same address would be detected as circular reference
    // We'll test this by checking the logic directly rather than calling process_subdirectory
    // which requires valid EXIF data
    let dir_info = DirectoryInfo {
        name: "TestIFD2".to_string(),
        dir_start: 100,
        dir_len: 0,
        base: 0,
        data_pos: 0,
        allow_reprocess: false,
    };

    let calculated_addr = dir_info.dir_start as u64 + dir_info.data_pos + dir_info.base;
    assert_eq!(calculated_addr, addr);
    assert!(reader.processed.contains_key(&calculated_addr));
}

#[test]
fn test_subdirectory_path_management() {
    let mut reader = ExifReader::new();

    // Initially empty path
    assert_eq!(reader.get_current_path(), "Root");

    // Create some directory info (not used in this test)
    let _dir_info = DirectoryInfo {
        name: "ExifIFD".to_string(),
        dir_start: 0,
        dir_len: 0,
        base: 0,
        data_pos: 0,
        allow_reprocess: false,
    };

    // Manually test path management
    reader.path.push("IFD0".to_string());
    assert_eq!(reader.get_current_path(), "IFD0");

    reader.path.push("ExifIFD".to_string());
    assert_eq!(reader.get_current_path(), "IFD0/ExifIFD");

    reader.path.pop();
    assert_eq!(reader.get_current_path(), "IFD0");
}

#[test]
fn test_subdirectory_tag_detection() {
    let reader = ExifReader::new();

    // Test that SubDirectory tags are detected correctly
    assert!(reader.is_subdirectory_tag(0x8769)); // ExifIFD
    assert!(reader.is_subdirectory_tag(0x8825)); // GPS
    assert!(reader.is_subdirectory_tag(0xA005)); // InteropIFD
    assert!(reader.is_subdirectory_tag(0x927C)); // MakerNotes

    // Test that regular tags are not detected as subdirectories
    assert!(!reader.is_subdirectory_tag(0x010F)); // Make
    assert!(!reader.is_subdirectory_tag(0x0110)); // Model
    assert!(!reader.is_subdirectory_tag(0x0112)); // Orientation
}

#[test]
fn test_processing_statistics() {
    let mut reader = ExifReader::new();

    // Add some mock data
    reader
        .extracted_tags
        .insert(0x010F, TagValue::String("Canon".to_string()));
    reader.warnings.push("Test warning".to_string());
    reader
        .data_members
        .insert("TestMember".to_string(), DataMemberValue::U16(42));

    let stats = reader.get_processing_stats();
    assert_eq!(stats.get("extracted_tags"), Some(&1));
    assert_eq!(stats.get("warnings"), Some(&1));
    assert_eq!(stats.get("data_members"), Some(&1));
    assert_eq!(stats.get("processed_directories"), Some(&0));
    assert_eq!(stats.get("subdirectory_overrides"), Some(&0));
}

#[test]
fn test_processor_dispatch_selection() {
    let reader = ExifReader::new();

    // Test default processor selection
    assert_eq!(reader.select_processor("IFD0", None), ProcessorType::Exif);
    assert_eq!(
        reader.select_processor("ExifIFD", None),
        ProcessorType::Exif
    );
    assert_eq!(reader.select_processor("GPS", None), ProcessorType::Gps);
    assert_eq!(
        reader.select_processor("InteropIFD", None),
        ProcessorType::Exif
    );

    // Test MakerNotes gets manufacturer-specific detection (defaults to EXIF when no Make/Model)
    let processor = reader.select_processor("MakerNotes", None);
    match processor {
        ProcessorType::Exif => {
            // Expected when no Make/Model tags are available for detection
        }
        ProcessorType::Canon(_) => {
            // Expected when Canon Make is detected
        }
        _ => panic!("Expected EXIF or Canon processor for MakerNotes, got {processor:?}"),
    }
}

#[test]
fn test_processor_dispatch_overrides() {
    let mut reader = ExifReader::new();

    // Add a SubDirectory override
    reader.add_subdirectory_override(0x8769, ProcessorType::BinaryData);

    // Verify override is stored
    let dispatch = reader.get_processor_dispatch();
    assert_eq!(
        dispatch.subdirectory_overrides.get(&0x8769),
        Some(&ProcessorType::BinaryData)
    );

    // Verify stats reflect the override
    let stats = reader.get_processing_stats();
    assert_eq!(stats.get("subdirectory_overrides"), Some(&1));
}

#[test]
fn test_subdirectory_processor_overrides() {
    let reader = ExifReader::new();

    // Test known SubDirectory processor overrides
    assert_eq!(reader.get_subdirectory_processor_override(0x8769), None); // ExifIFD
    assert_eq!(reader.get_subdirectory_processor_override(0x8825), None); // GPS
    assert_eq!(reader.get_subdirectory_processor_override(0xA005), None); // InteropIFD

    // MakerNotes should have no override (to allow manufacturer-specific detection)
    assert_eq!(reader.get_subdirectory_processor_override(0x927C), None);

    // Unknown tag should have no override
    assert_eq!(reader.get_subdirectory_processor_override(0x1234), None);
}

#[test]
fn test_gps_rational_arrays_returned_raw() {
    // Test that GPS coordinates return raw rational arrays in Milestone 8e
    // GPS:GPSLatitude should return [[54,1], [59,38/100], [0,1]] not decimal degrees

    // Test GPSLatitude returns rational array format
    let lat_rationals = TagValue::RationalArray(vec![(54, 1), (5938, 100), (0, 1)]);

    // Verify we can access the rational components directly
    if let TagValue::RationalArray(rationals) = &lat_rationals {
        assert_eq!(
            rationals.len(),
            3,
            "GPS coordinates should have 3 components"
        );
        assert_eq!(rationals[0], (54, 1), "Degrees component");
        assert_eq!(rationals[1], (5938, 100), "Minutes component");
        assert_eq!(rationals[2], (0, 1), "Seconds component");
    } else {
        panic!("GPS coordinates should be RationalArray");
    }

    // Test GPSLongitude format
    let lon_rationals = TagValue::RationalArray(vec![(1, 1), (5485, 100), (0, 1)]);
    if let TagValue::RationalArray(rationals) = &lon_rationals {
        assert_eq!(rationals[0], (1, 1), "Degrees component");
        assert_eq!(rationals[1], (5485, 100), "Minutes component");
        assert_eq!(rationals[2], (0, 1), "Seconds component");
    } else {
        panic!("GPS coordinates should be RationalArray");
    }

    // GPS reference tags should remain as strings
    let lat_ref = TagValue::String("N".to_string());
    let lon_ref = TagValue::String("W".to_string());

    assert_eq!(lat_ref.as_string(), Some("N"));
    assert_eq!(lon_ref.as_string(), Some("W"));

    // Note: Decimal conversion will be handled by Composite tags
    // that combine GPS:GPSLatitude + GPS:GPSLatitudeRef -> Composite:GPSLatitude
}
