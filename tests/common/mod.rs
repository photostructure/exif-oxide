//! Common test utilities shared across integration tests
//!
//! This module provides helper functions that are used by multiple integration tests
//! to set up test data and scenarios.

#![allow(dead_code)] // Test constants and helpers are intended for future use

use exif_oxide::exif::ExifReader;
use exif_oxide::types::TagValue;

/// Test image path constants to avoid duplication across test files
///
/// These constants provide a central location for test image paths, ensuring consistency
/// and making it easy to update paths when test images are moved or renamed.
///
/// ## Usage
///
/// ```rust
/// use common::CANON_T3I_JPG;
///
/// let exif_data = extract_metadata(
///     std::path::Path::new(CANON_T3I_JPG),
///     false,
///     false,
/// ).unwrap();
/// ```
///
/// ## Guidelines
///
/// - Use these constants instead of hardcoding paths in individual test files
/// - Add new constants when introducing new test images
/// - Keep the naming convention: `MANUFACTURER_MODEL_FORMAT`
/// - Document any special properties of the test images (e.g., has GPS, MakerNotes, etc.)
///
/// Canon T3i JPEG - Primary test image with comprehensive EXIF data including ExifIFD
pub const CANON_T3I_JPG: &str = "test-images/canon/eos_rebel_t3i.jpg";

/// Canon T3i RAW file - Companion RAW format for testing raw processing
pub const CANON_T3I_CR2: &str = "test-images/canon/eos_rebel_t3i.cr2";

/// Olympus ORF test file - Tests Olympus-specific RAW format and MakerNotes
pub const OLYMPUS_TEST_ORF: &str = "test-images/olympus/test.orf";

/// Apple iPhone image - Tests Apple/iOS-specific metadata and HEIC conversion
pub const APPLE_IMG_3755_JPG: &str = "test-images/apple/IMG_3755.JPG";

/// Nikon Z8 image - Tests modern Nikon mirrorless camera metadata
pub const NIKON_Z8_JPG: &str = "test-images/nikon/z_8_73.jpg";

/// Sony A7C II image - Tests Sony mirrorless camera metadata
pub const SONY_A7C_JPG: &str = "test-images/sony/a7c_ii.jpg";

/// Canon EOS R50V image - Tests newer Canon mirrorless format
pub const CANON_EOS_R50V_JPG: &str = "test-images/canon/eos_r50v_01.jpg";

/// Canon EOS R5 Mark II image - Tests latest Canon high-resolution format
pub const CANON_EOS_R5_MARK_II_JPG: &str = "test-images/canon/canon_eos_r5_mark_ii_10.jpg";

/// Fujifilm X-E5 image - Tests Fujifilm X-series metadata
pub const FUJIFILM_XE5_JPG: &str = "test-images/fujifilm/xe5_02.jpg";

/// Panasonic G9 II image - Tests Panasonic Lumix metadata
pub const PANASONIC_G9_II_JPG: &str = "test-images/panasonic/dc-g9m2.jpg";

/// Olympus C2000Z image - Tests Olympus camera metadata  
pub const OLYMPUS_C2000Z_JPG: &str = "test-images/olympus/c2020z.jpg";

/// Nikon Z8 NEF RAW file - Tests Nikon RAW format
pub const NIKON_Z8_NEF: &str = "test-images/nikon/z_8_73.nef";

/// Sony A7C II ARW RAW file - Tests Sony RAW format
pub const SONY_A7C_ARW: &str = "test-images/sony/a7c_ii.arw";

/// Panasonic G9 II RW2 RAW file - Tests Panasonic RAW format
pub const PANASONIC_G9_II_RW2: &str = "test-images/panasonic/dc-g9m2.rw2";

/// Minolta DiMAGE 7 MRW RAW file - Tests Minolta RAW format
pub const MINOLTA_DIMAGE_7_MRW: &str = "test-images/minolta/dimage_7.mrw";

/// Apple IMG_9757 HEIC file - Tests Apple HEIC format
pub const APPLE_IMG_9757_HEIC: &str = "test-images/apple/IMG_9757.heic";

/// Helper to create an ExifReader with test data for integration tests
/// This allows us to simulate extracted EXIF data without accessing private fields
#[allow(unused_mut)] // mut needed only when test-helpers feature is enabled
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
