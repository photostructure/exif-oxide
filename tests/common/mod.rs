//! Common test utilities shared across integration tests
//!
//! This module provides helper functions that are used by multiple integration tests
//! to set up test data and scenarios.

use exif_oxide::exif::ExifReader;
use exif_oxide::types::TagValue;

/// Helper to create an ExifReader with test data for integration tests
/// This allows us to simulate extracted EXIF data without accessing private fields
pub fn create_test_reader_with_tags(test_data: Vec<(u16, TagValue, &str, &str)>) -> ExifReader {
    let mut reader = ExifReader::new();

    // Since we can't access private fields from integration tests,
    // we need to use the public API. However, for now we'll expose
    // a test helper method that's only available during testing.
    #[cfg(feature = "test-helpers")]
    {
        for (tag_id, value, namespace, ifd_name) in test_data {
            reader.add_test_tag(tag_id, value, namespace, ifd_name);
        }
    }

    #[cfg(not(feature = "test-helpers"))]
    {
        // When test-helpers feature is not available, we still create a reader
        // but note that it won't have the test data. This ensures the module
        // compiles but tests may not pass.
        let _ = test_data; // Suppress unused variable warning
    }

    reader
}

/// Create a test reader with typical camera EXIF data
pub fn create_camera_test_reader() -> ExifReader {
    create_test_reader_with_tags(vec![
        (
            0x829a,
            TagValue::String("50".to_string()),
            "EXIF",
            "ExifIFD",
        ), // FocalLength
        (
            0xa405,
            TagValue::String("75".to_string()),
            "EXIF",
            "ExifIFD",
        ), // FocalLengthIn35mmFormat
        (
            0x829d,
            TagValue::String("2.8".to_string()),
            "EXIF",
            "ExifIFD",
        ), // FNumber
        (0xa002, TagValue::U32(1920), "EXIF", "ExifIFD"), // ExifImageWidth
        (0xa003, TagValue::U32(1280), "EXIF", "ExifIFD"), // ExifImageHeight
        (0x8827, TagValue::U32(400), "EXIF", "ExifIFD"),  // ISO
        (
            0x829a,
            TagValue::String("1/60".to_string()),
            "EXIF",
            "ExifIFD",
        ), // ExposureTime
    ])
}

/// Create a test reader with GPS EXIF data
pub fn create_gps_test_reader() -> ExifReader {
    create_test_reader_with_tags(vec![
        (
            0x0002,
            TagValue::String("37 46 29.7".to_string()),
            "GPS",
            "GPS",
        ), // GPSLatitude
        (0x0001, TagValue::String("N".to_string()), "GPS", "GPS"), // GPSLatitudeRef
        (
            0x0004,
            TagValue::String("122 25 9.8".to_string()),
            "GPS",
            "GPS",
        ), // GPSLongitude
        (0x0003, TagValue::String("W".to_string()), "GPS", "GPS"), // GPSLongitudeRef
        (0x0006, TagValue::String("10".to_string()), "GPS", "GPS"), // GPSAltitude
        (0x0005, TagValue::U8(0), "GPS", "GPS"), // GPSAltitudeRef (0 = above sea level)
        (
            0x001d,
            TagValue::String("2024:01:15".to_string()),
            "GPS",
            "GPS",
        ), // GPSDateStamp
        (
            0x0007,
            TagValue::String("14 30 25".to_string()),
            "GPS",
            "GPS",
        ), // GPSTimeStamp
    ])
}

/// Create a test reader with minimal EXIF data for testing missing dependencies
pub fn create_minimal_test_reader() -> ExifReader {
    create_test_reader_with_tags(vec![
        (0xa002, TagValue::U32(1920), "EXIF", "ExifIFD"), // ExifImageWidth
        (0xa003, TagValue::U32(1280), "EXIF", "ExifIFD"), // ExifImageHeight
    ])
}

/// Create a test reader with comprehensive EXIF data for performance testing
pub fn create_comprehensive_test_reader() -> ExifReader {
    create_test_reader_with_tags(vec![
        (
            0x010f,
            TagValue::String("Canon".to_string()),
            "EXIF",
            "ExifIFD",
        ), // Make
        (
            0x0110,
            TagValue::String("Canon EOS R5".to_string()),
            "EXIF",
            "ExifIFD",
        ), // Model
        (
            0x829a,
            TagValue::String("85".to_string()),
            "EXIF",
            "ExifIFD",
        ), // FocalLength
        (
            0xa405,
            TagValue::String("85".to_string()),
            "EXIF",
            "ExifIFD",
        ), // FocalLengthIn35mmFormat
        (
            0x829d,
            TagValue::String("1.4".to_string()),
            "EXIF",
            "ExifIFD",
        ), // FNumber
        (
            0x9201,
            TagValue::String("1.4".to_string()),
            "EXIF",
            "ExifIFD",
        ), // ApertureValue
        (
            0x829a,
            TagValue::String("1/200".to_string()),
            "EXIF",
            "ExifIFD",
        ), // ExposureTime
        (
            0x9201,
            TagValue::String("7.64".to_string()),
            "EXIF",
            "ExifIFD",
        ), // ShutterSpeedValue
        (0x8827, TagValue::U32(100), "EXIF", "ExifIFD"), // ISO
        (0xa002, TagValue::U32(8192), "EXIF", "ExifIFD"), // ExifImageWidth
        (0xa003, TagValue::U32(5464), "EXIF", "ExifIFD"), // ExifImageHeight
        (0x0100, TagValue::U32(8192), "EXIF", "ExifIFD"), // ImageWidth
        (0x0101, TagValue::U32(5464), "EXIF", "ExifIFD"), // ImageHeight
    ])
}

/// Create a test reader for dependency resolution testing
pub fn create_dependency_test_reader() -> ExifReader {
    create_test_reader_with_tags(vec![
        (
            0x829a,
            TagValue::String("50".to_string()),
            "EXIF",
            "ExifIFD",
        ), // FocalLength
        (
            0xa405,
            TagValue::String("75".to_string()),
            "EXIF",
            "ExifIFD",
        ), // FocalLengthIn35mmFormat
        (
            0x829d,
            TagValue::String("2.8".to_string()),
            "EXIF",
            "ExifIFD",
        ), // FNumber
        (0xa002, TagValue::U32(1920), "EXIF", "ExifIFD"), // ExifImageWidth
        (0xa003, TagValue::U32(1280), "EXIF", "ExifIFD"), // ExifImageHeight
    ])
}
