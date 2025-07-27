//! Tests for RAW processing utilities

use super::utils::*;

#[test]
fn test_reverse_string() {
    let input = b"KYOCERA\0";
    let result = reverse_string(input);
    assert_eq!(result, "ARECOYK");

    // Test with null termination
    let input2 = b"TEST\0\0\0";
    let result2 = reverse_string(input2);
    assert_eq!(result2, "TSET");
}

#[test]
fn test_kyocera_exposure_time() {
    // Test some known values
    assert!((kyocera_exposure_time(0) - 0.0).abs() < f64::EPSILON);

    // Test calculation: 2^(val/8) / 16000
    let val = 64; // Should be 2^8 / 16000 = 256 / 16000 = 0.016
    let expected = 2_f64.powf(64.0 / 8.0) / 16000.0;
    assert!((kyocera_exposure_time(val) - expected).abs() < 0.0001);
}

#[test]
fn test_kyocera_fnumber() {
    // Test some known values
    assert!((kyocera_fnumber(0) - 0.0).abs() < f64::EPSILON);

    // Test calculation: 2^(val/16)
    let val = 32; // Should be 2^2 = 4.0
    let expected = 2_f64.powf(32.0 / 16.0);
    assert!((kyocera_fnumber(val) - expected).abs() < 0.0001);
}

#[test]
fn test_kyocera_iso_lookup() {
    // Test known mappings
    assert_eq!(kyocera_iso_lookup(7), Some(25));
    assert_eq!(kyocera_iso_lookup(13), Some(100));
    assert_eq!(kyocera_iso_lookup(19), Some(400));

    // Test invalid values
    assert_eq!(kyocera_iso_lookup(6), None);
    assert_eq!(kyocera_iso_lookup(20), None);
    assert_eq!(kyocera_iso_lookup(100), None);
}

#[test]
fn test_rw2_magic_bytes_detection() {
    use crate::exif::ExifReader;

    // Test minimal RW2 file with correct magic bytes and minimal IFD
    // Focus on testing that RW2 magic bytes are recognized, not full parsing
    let rw2_data = [
        0x49, 0x49, 0x55, 0x00, // IIU\0 - RW2 magic bytes
        0x08, 0x00, 0x00, 0x00, // IFD0 offset = 0x08 (minimal offset)
        // Minimal IFD0 with 0 entries (just test magic byte recognition)
        0x00, 0x00, // Number of entries = 0
        0x00, 0x00, 0x00, 0x00, // Next IFD = 0
    ];

    let mut reader = ExifReader::new();
    let result = extract_tiff_dimensions(&mut reader, &rw2_data);

    // The key test: RW2 magic bytes should be recognized and not cause early return
    // If magic bytes weren't recognized, the function would return early with debug message
    assert!(result.is_ok());

    // For this simple test, we just verify the function processes the file without error
    // The actual sensor border parsing is tested separately
}

#[test]
fn test_standard_tiff_magic_bytes() {
    use crate::exif::ExifReader;

    // Test little-endian TIFF (II*\0)
    let le_tiff_data = [
        0x49, 0x49, 0x2A, 0x00, // II*\0 - little-endian TIFF
        0x08, 0x00, 0x00, 0x00, // IFD0 offset = 0x08
        // IFD0 with 0 entries
        0x00, 0x00, // Number of entries = 0
        0x00, 0x00, 0x00, 0x00, // Next IFD = 0
    ];

    let mut reader = ExifReader::new();
    let result = extract_tiff_dimensions(&mut reader, &le_tiff_data);
    assert!(result.is_ok());

    // Test big-endian TIFF (MM\0*)
    let be_tiff_data = [
        0x4D, 0x4D, 0x00, 0x2A, // MM\0* - big-endian TIFF
        0x00, 0x00, 0x00, 0x08, // IFD0 offset = 0x08 (big-endian)
        // IFD0 with 0 entries
        0x00, 0x00, // Number of entries = 0
        0x00, 0x00, 0x00, 0x00, // Next IFD = 0
    ];

    let mut reader2 = ExifReader::new();
    let result2 = extract_tiff_dimensions(&mut reader2, &be_tiff_data);
    assert!(result2.is_ok());
}

#[test]
fn test_invalid_magic_bytes() {
    use crate::exif::ExifReader;

    // Test invalid magic bytes should return early without error
    let invalid_data = [
        0x00, 0x00, 0x00, 0x00, // Invalid magic bytes
        0x08, 0x00, 0x00, 0x00,
    ];

    let mut reader = ExifReader::new();
    let result = extract_tiff_dimensions(&mut reader, &invalid_data);
    assert!(result.is_ok()); // Should return Ok(()) but not process anything
    assert!(reader.extracted_tags.is_empty());
}

#[test]
fn test_sensor_border_calculation() {
    use crate::exif::ExifReader;
    use crate::types::TagValue;

    // Create mock RW2 data with all sensor border tags
    // Values match the test file: left=4, right=5780, top=4, bottom=4340
    let rw2_data = create_mock_rw2_with_sensor_borders(4, 5780, 4, 4340);

    let mut reader = ExifReader::new();
    let result = extract_tiff_dimensions(&mut reader, &rw2_data);
    assert!(result.is_ok());

    // Verify all sensor border tags were extracted
    assert_eq!(
        reader.extracted_tags.get(&(0x04, "EXIF".to_string())),
        Some(&TagValue::U16(4))
    ); // SensorTopBorder
    assert_eq!(
        reader.extracted_tags.get(&(0x05, "EXIF".to_string())),
        Some(&TagValue::U16(4))
    ); // SensorLeftBorder
    assert_eq!(
        reader.extracted_tags.get(&(0x06, "EXIF".to_string())),
        Some(&TagValue::U16(4340))
    ); // SensorBottomBorder
    assert_eq!(
        reader.extracted_tags.get(&(0x07, "EXIF".to_string())),
        Some(&TagValue::U16(5780))
    ); // SensorRightBorder

    // Verify calculated dimensions (ExifTool: PanasonicRaw.pm:675-690)
    // ImageWidth = RightBorder - LeftBorder = 5780 - 4 = 5776
    // ImageHeight = BottomBorder - TopBorder = 4340 - 4 = 4336
    assert_eq!(
        reader.extracted_tags.get(&(0x0100, "EXIF".to_string())),
        Some(&TagValue::U32(5776))
    ); // ImageWidth
    assert_eq!(
        reader.extracted_tags.get(&(0x0101, "EXIF".to_string())),
        Some(&TagValue::U32(4336))
    ); // ImageHeight
}

// Helper function to create mock RW2 data with sensor border tags
fn create_mock_rw2_with_sensor_borders(left: u16, right: u16, top: u16, bottom: u16) -> Vec<u8> {
    let mut data = vec![
        // RW2 header
        0x49, 0x49, 0x55, 0x00, // IIU\0 - RW2 magic bytes
        0x18, 0x00, 0x00, 0x00, // IFD0 offset = 0x18
        // Padding
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ];

    // IFD0 with 4 sensor border entries
    data.extend_from_slice(&[0x04, 0x00]); // 4 entries

    // SensorTopBorder (0x04)
    data.extend_from_slice(&[0x04, 0x00, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00]);
    data.extend_from_slice(&top.to_le_bytes());
    data.extend_from_slice(&[0x00, 0x00]);

    // SensorLeftBorder (0x05)
    data.extend_from_slice(&[0x05, 0x00, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00]);
    data.extend_from_slice(&left.to_le_bytes());
    data.extend_from_slice(&[0x00, 0x00]);

    // SensorBottomBorder (0x06)
    data.extend_from_slice(&[0x06, 0x00, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00]);
    data.extend_from_slice(&bottom.to_le_bytes());
    data.extend_from_slice(&[0x00, 0x00]);

    // SensorRightBorder (0x07)
    data.extend_from_slice(&[0x07, 0x00, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00]);
    data.extend_from_slice(&right.to_le_bytes());
    data.extend_from_slice(&[0x00, 0x00]);

    // Next IFD = 0
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

    data
}
