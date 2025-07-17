//! File detection integration tests
//!
//! These tests verify file type detection against real-world sample files.
//!
//! Note: These tests require the `integration-tests` feature to be enabled and
//! external test assets to be available. They are automatically skipped in published crates.

#![cfg(feature = "integration-tests")]

use exif_oxide::file_detection::FileTypeDetector;
use std::fs::File;
use std::path::Path;

#[test]
fn test_real_file_detection() {
    let detector = FileTypeDetector::new();

    // Test HEIC file
    let heic_path = Path::new("test-images/apple/IMG_9757.heic");
    if heic_path.exists() {
        let mut file = File::open(heic_path).unwrap();
        let result = detector.detect_file_type(heic_path, &mut file).unwrap();
        assert_eq!(result.file_type, "HEIC");
        assert_eq!(result.format, "MOV");
        assert_eq!(result.mime_type, "image/heic");
        assert_eq!(
            result.description,
            "High Efficiency Image Format still image"
        );
        println!("✓ HEIC detection successful");
    }

    // Test JPEG file
    let jpeg_path = Path::new("test-images/jpeg/olympus_c960.jpg");
    if jpeg_path.exists() {
        let mut file = File::open(jpeg_path).unwrap();
        let result = detector.detect_file_type(jpeg_path, &mut file).unwrap();
        assert_eq!(result.file_type, "JPEG");
        assert_eq!(result.format, "JPEG");
        assert_eq!(result.mime_type, "image/jpeg");
        println!("✓ JPEG detection successful");
    }

    // Test TIFF file
    let tiff_path = Path::new("test-images/tiff/little_endian_example.tif");
    if tiff_path.exists() {
        let mut file = File::open(tiff_path).unwrap();
        let result = detector.detect_file_type(tiff_path, &mut file).unwrap();
        assert_eq!(result.file_type, "TIFF");
        assert_eq!(result.format, "TIFF");
        assert_eq!(result.mime_type, "image/tiff");
        println!("✓ TIFF detection successful");
    }
}

#[test]
fn test_quicktime_heic_is_heif() {
    // Test that QuickTime.heic with mif1 brand is correctly detected as HEIF, not HEIC
    // This matches ExifTool's behavior: mif1 brand maps to HEIF
    let detector = FileTypeDetector::new();
    let heic_path = Path::new("third-party/exiftool/t/images/QuickTime.heic");

    if heic_path.exists() {
        let mut file = File::open(heic_path).unwrap();
        match detector.detect_file_type(heic_path, &mut file) {
            Ok(result) => {
                // Even though the extension is .heic, ExifTool detects it as HEIF due to mif1 brand
                assert_eq!(
                    result.file_type, "HEIF",
                    "File type should be HEIF (not HEIC) for mif1 brand"
                );
                assert_eq!(result.format, "MOV");
                assert_eq!(result.mime_type, "image/heif");
                assert_eq!(result.description, "High Efficiency Image Format");
                println!("✓ QuickTime.heic correctly detected as HEIF (mif1 brand)");
            }
            Err(e) => {
                panic!("Failed to detect QuickTime.heic: {e:?}");
            }
        }
    } else {
        println!(
            "⚠️  QuickTime.heic test file not found at: {}",
            heic_path.display()
        );
    }
}
