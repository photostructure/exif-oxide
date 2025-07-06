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
        (0x829a, "50".into(), "EXIF", "ExifIFD"),  // FocalLength
        (0xa405, "75".into(), "EXIF", "ExifIFD"),  // FocalLengthIn35mmFormat
        (0x829d, "2.8".into(), "EXIF", "ExifIFD"), // FNumber
        (0xa002, TagValue::U32(1920), "EXIF", "ExifIFD"), // ExifImageWidth
        (0xa003, TagValue::U32(1280), "EXIF", "ExifIFD"), // ExifImageHeight
        (0x8827, TagValue::U32(400), "EXIF", "ExifIFD"), // ISO
        (0x829a, "1/60".into(), "EXIF", "ExifIFD"), // ExposureTime
    ])
}

/// Create a test reader with GPS EXIF data
pub fn create_gps_test_reader() -> ExifReader {
    create_test_reader_with_tags(vec![
        (0x0002, "37 46 29.7".into(), "GPS", "GPS"), // GPSLatitude
        (0x0001, "N".into(), "GPS", "GPS"),          // GPSLatitudeRef
        (0x0004, "122 25 9.8".into(), "GPS", "GPS"), // GPSLongitude
        (0x0003, "W".into(), "GPS", "GPS"),          // GPSLongitudeRef
        (0x0006, "10".into(), "GPS", "GPS"),         // GPSAltitude
        (0x0005, TagValue::U8(0), "GPS", "GPS"),     // GPSAltitudeRef (0 = above sea level)
        (0x001d, "2024:01:15".into(), "GPS", "GPS"), // GPSDateStamp
        (0x0007, "14 30 25".into(), "GPS", "GPS"),   // GPSTimeStamp
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
        (0x010f, "Canon".into(), "EXIF", "ExifIFD"), // Make
        (0x0110, "Canon EOS R5".into(), "EXIF", "ExifIFD"), // Model
        (0x829a, "85".into(), "EXIF", "ExifIFD"),    // FocalLength
        (0xa405, "85".into(), "EXIF", "ExifIFD"),    // FocalLengthIn35mmFormat
        (0x829d, "1.4".into(), "EXIF", "ExifIFD"),   // FNumber
        (0x9201, "1.4".into(), "EXIF", "ExifIFD"),   // ApertureValue
        (0x829a, "1/200".into(), "EXIF", "ExifIFD"), // ExposureTime
        (0x9201, "7.64".into(), "EXIF", "ExifIFD"),  // ShutterSpeedValue
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
        (0x829a, "50".into(), "EXIF", "ExifIFD"),  // FocalLength
        (0xa405, "75".into(), "EXIF", "ExifIFD"),  // FocalLengthIn35mmFormat
        (0x829d, "2.8".into(), "EXIF", "ExifIFD"), // FNumber
        (0xa002, TagValue::U32(1920), "EXIF", "ExifIFD"), // ExifImageWidth
        (0xa003, TagValue::U32(1280), "EXIF", "ExifIFD"), // ExifImageHeight
    ])
}
