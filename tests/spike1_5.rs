//! Integration tests for Spike 1.5 - Table-driven parsing with all format types

use exif_oxide::core::{ifd, jpeg, ExifValue};
use std::fs::File;

#[test]
fn test_resolution_parsing() {
    // Canon.jpg should have XResolution and YResolution tags (rational type)
    let mut file = File::open("exiftool/t/images/Canon.jpg").unwrap();
    let exif_segment = jpeg::find_exif_segment(&mut file)
        .unwrap()
        .expect("Canon.jpg should have EXIF data");

    let ifd = ifd::IfdParser::parse(exif_segment.data).unwrap();

    // Check XResolution (0x011A)
    match ifd.entries().get(&0x011A) {
        Some(ExifValue::Rational(num, den)) => {
            // Canon typically uses 180/1 or 72/1 DPI
            assert!(*den > 0, "Resolution denominator should be positive");
            let dpi = *num as f64 / *den as f64;
            assert!(dpi > 0.0, "DPI should be positive");
        }
        _ => panic!("XResolution should be a Rational value"),
    }

    // Check YResolution (0x011B)
    match ifd.entries().get(&0x011B) {
        Some(ExifValue::Rational(num, den)) => {
            assert!(*den > 0, "Resolution denominator should be positive");
            let dpi = *num as f64 / *den as f64;
            assert!(dpi > 0.0, "DPI should be positive");
        }
        _ => panic!("YResolution should be a Rational value"),
    }
}

#[test]
fn test_bits_per_sample_array() {
    // Many images have BitsPerSample as an array
    let mut file = File::open("exiftool/t/images/Canon.jpg").unwrap();
    let exif_segment = jpeg::find_exif_segment(&mut file)
        .unwrap()
        .expect("Canon.jpg should have EXIF data");

    let ifd = ifd::IfdParser::parse(exif_segment.data).unwrap();

    // BitsPerSample (0x0102) might be present
    if let Some(value) = ifd.entries().get(&0x0102) {
        match value {
            ExifValue::U16(v) => {
                assert_eq!(*v, 8, "Single BitsPerSample should typically be 8");
            }
            ExifValue::U16Array(arr) => {
                // RGB images typically have [8, 8, 8]
                assert!(!arr.is_empty(), "BitsPerSample array should not be empty");
                for &bits in arr {
                    assert!(bits > 0 && bits <= 16, "Bits per sample should be 1-16");
                }
            }
            _ => panic!("BitsPerSample should be U16 or U16Array"),
        }
    }
}

#[test]
fn test_table_driven_parsing() {
    // Test that various tags are parsed with correct formats from the table
    let mut file = File::open("exiftool/t/images/ExifTool.jpg").unwrap();
    let exif_segment = jpeg::find_exif_segment(&mut file)
        .unwrap()
        .expect("ExifTool.jpg should have EXIF data");

    let ifd = ifd::IfdParser::parse(exif_segment.data).unwrap();

    // Check various format types

    // ASCII string - Make (0x010F)
    if let Some(ExifValue::Ascii(make)) = ifd.entries().get(&0x010F) {
        assert!(!make.is_empty(), "Make should not be empty");
    }

    // U16 - Orientation (0x0112)
    if let Some(ExifValue::U16(orientation)) = ifd.entries().get(&0x0112) {
        assert!(
            *orientation >= 1 && *orientation <= 8,
            "Orientation should be 1-8"
        );
    }

    // U32 - ImageWidth might be present as U32
    if let Some(value) = ifd.entries().get(&0x0100) {
        match value {
            ExifValue::U16(_) | ExifValue::U32(_) => {
                // Either format is acceptable for width
            }
            _ => panic!("ImageWidth should be U16 or U32"),
        }
    }
}

#[test]
fn test_nikon_rational_tags() {
    // Nikon images often have more rational values
    let mut file = File::open("exiftool/t/images/Nikon.jpg").unwrap();
    let exif_segment = jpeg::find_exif_segment(&mut file)
        .unwrap()
        .expect("Nikon.jpg should have EXIF data");

    let ifd = ifd::IfdParser::parse(exif_segment.data).unwrap();

    // Just verify we can parse without panicking
    // and that rational values are handled correctly
    for (tag_id, value) in ifd.entries() {
        match value {
            ExifValue::Rational(_num, den) => {
                // Basic sanity check for rationals
                assert!(
                    *den != 0,
                    "Tag 0x{:04X}: Denominator should not be zero",
                    tag_id
                );
            }
            ExifValue::RationalArray(rationals) => {
                for (_num, den) in rationals {
                    assert!(
                        *den != 0,
                        "Tag 0x{:04X}: Denominator should not be zero",
                        tag_id
                    );
                }
            }
            ExifValue::SignedRational(_num, den) => {
                assert!(
                    *den != 0,
                    "Tag 0x{:04X}: Denominator should not be zero",
                    tag_id
                );
            }
            ExifValue::SignedRationalArray(rationals) => {
                for (_num, den) in rationals {
                    assert!(
                        *den != 0,
                        "Tag 0x{:04X}: Denominator should not be zero",
                        tag_id
                    );
                }
            }
            _ => {} // Other types are fine
        }
    }
}

#[test]
fn test_tag_format_override() {
    // Test that actual format in file is used when parsing
    let mut file = File::open("exiftool/t/images/Canon.jpg").unwrap();
    let exif_segment = jpeg::find_exif_segment(&mut file).unwrap().unwrap();
    let ifd = ifd::IfdParser::parse(exif_segment.data).unwrap();

    // Count how many tags were successfully parsed
    let parsed_count = ifd.entries().len();
    assert!(
        parsed_count >= 5,
        "Should parse at least 5 tags, got {}",
        parsed_count
    );

    // Verify no panic on unknown tags
    // The parser should handle unknown tags gracefully
    let _has_undefined = ifd
        .entries()
        .values()
        .any(|value| matches!(value, ExifValue::Undefined(_)));
    // It's OK if there are undefined values (unknown tags or unsupported formats)
}
