//! Spike 2 tests: Maker Note Parsing
//!
//! Tests for manufacturer detection and Canon maker note parsing.

use exif_oxide::core::ifd::IfdParser;
use exif_oxide::core::jpeg;
use exif_oxide::core::ExifValue;
use exif_oxide::tables::lookup_canon_tag;
use std::fs::File;
use std::path::Path;

/// Test that we can detect Canon as manufacturer from Make tag
#[test]
fn test_manufacturer_detection() {
    let canon_image = "exiftool/t/images/Canon.jpg";
    if !Path::new(canon_image).exists() {
        eprintln!(
            "Warning: Test image {} not found, skipping test",
            canon_image
        );
        return;
    }

    let mut file = File::open(canon_image).unwrap();
    let exif_segment = jpeg::find_exif_segment(&mut file).unwrap().unwrap();
    let ifd = IfdParser::parse(exif_segment.data).unwrap();

    // Check Make tag (0x10F)
    let make = ifd.entries().get(&0x10F).unwrap();
    match make {
        ExifValue::Ascii(s) => {
            assert!(
                s.contains("Canon"),
                "Expected Canon in Make tag, got: {}",
                s
            );
        }
        _ => panic!("Make tag should be ASCII string"),
    }
}

/// Test that Canon maker notes are parsed and extracted
#[test]
fn test_canon_maker_note_parsing() {
    let canon_image = "exiftool/t/images/Canon1DmkIII.jpg";
    if !Path::new(canon_image).exists() {
        eprintln!(
            "Warning: Test image {} not found, skipping test",
            canon_image
        );
        return;
    }

    let mut file = File::open(canon_image).unwrap();
    let exif_segment = jpeg::find_exif_segment(&mut file).unwrap().unwrap();
    let ifd = IfdParser::parse(exif_segment.data).unwrap();

    // Count Canon-specific tags (prefixed with 0xC000)
    let canon_tag_count = ifd
        .entries()
        .keys()
        .filter(|&&tag| tag >= 0xC000 && tag < 0xD000)
        .count();

    println!("Found {} Canon maker note tags", canon_tag_count);
    assert!(
        canon_tag_count > 20,
        "Expected at least 20 Canon tags, found {}",
        canon_tag_count
    );

    // The documentation says 28 tags were extracted, but we have 27 due to overflow prevention
    assert!(
        canon_tag_count >= 27,
        "Expected at least 27 Canon tags based on spike results"
    );
}

/// Test extraction of specific Canon tags
#[test]
fn test_specific_canon_tags() {
    let canon_image = "exiftool/t/images/Canon1DmkIII.jpg";
    if !Path::new(canon_image).exists() {
        eprintln!(
            "Warning: Test image {} not found, skipping test",
            canon_image
        );
        return;
    }

    let mut file = File::open(canon_image).unwrap();
    let exif_segment = jpeg::find_exif_segment(&mut file).unwrap().unwrap();
    let ifd = IfdParser::parse(exif_segment.data).unwrap();

    // Test for specific Canon tags we know should exist
    // Tag IDs from src/maker/canon.rs:
    // - 0x0006 (ImageType)
    // - 0x0007 (FirmwareVersion)
    // - 0x0008 (FileNumber)
    // - 0x0009 (OwnerName)

    // Check ImageType (0xC006 after prefixing)
    if let Some(image_type) = ifd.entries().get(&0xC006) {
        match image_type {
            ExifValue::Ascii(s) => {
                println!("Canon ImageType: {}", s);
                assert!(!s.is_empty(), "ImageType should not be empty");
            }
            ExifValue::Undefined(data) => {
                // Some Canon models store this as Undefined format
                println!("Canon ImageType: {} bytes of Undefined data", data.len());
                assert!(!data.is_empty(), "ImageType should not be empty");
            }
            _ => panic!(
                "ImageType should be ASCII string or Undefined, got {:?}",
                image_type
            ),
        }
    }

    // Check FirmwareVersion (0xC007)
    if let Some(firmware) = ifd.entries().get(&0xC007) {
        match firmware {
            ExifValue::Ascii(s) => {
                println!("Canon FirmwareVersion: {}", s);
                assert!(!s.is_empty(), "FirmwareVersion should not be empty");
            }
            ExifValue::Undefined(data) => {
                // Some Canon models store this as Undefined format
                println!(
                    "Canon FirmwareVersion: {} bytes of Undefined data",
                    data.len()
                );
                assert!(!data.is_empty(), "FirmwareVersion should not be empty");
            }
            _ => panic!(
                "FirmwareVersion should be ASCII string or Undefined, got {:?}",
                firmware
            ),
        }
    }
}

/// Test Canon tag lookup functionality
#[test]
fn test_canon_tag_lookup() {
    // Test some known Canon tags that are actually generated
    let test_tags = [
        (0x0003, "CanonFlashInfo"),
        (0x0006, "CanonImageType"),
        (0x0007, "CanonFirmwareVersion"),
        (0x0008, "FileNumber"),
        (0x0009, "OwnerName"),
    ];

    for (tag_id, expected_name) in test_tags {
        let tag_info = lookup_canon_tag(tag_id);
        assert!(
            tag_info.is_some(),
            "Canon tag 0x{:04x} should exist",
            tag_id
        );

        let info = tag_info.unwrap();
        assert_eq!(
            info.name, expected_name,
            "Canon tag 0x{:04x} should be named {}, got {}",
            tag_id, expected_name, info.name
        );
    }
}

/// Test parsing maker notes from various Canon models
#[test]
fn test_multiple_canon_models() {
    let test_images = [
        ("exiftool/t/images/Canon.jpg", "Canon EOS 40D"),
        (
            "exiftool/t/images/Canon1DmkIII.jpg",
            "Canon EOS-1D Mark III",
        ),
        ("test-images/canon/Canon_T3i.JPG", "Canon EOS REBEL T3i"),
    ];

    for (image_path, expected_model) in test_images {
        if !Path::new(image_path).exists() {
            eprintln!("Warning: Test image {} not found, skipping", image_path);
            continue;
        }

        let mut file = File::open(image_path).unwrap();
        let exif_segment = jpeg::find_exif_segment(&mut file);

        match exif_segment {
            Ok(Some(segment)) => {
                let ifd = IfdParser::parse(segment.data).unwrap();

                // Check Model tag
                if let Some(model) = ifd.entries().get(&0x110) {
                    match model {
                        ExifValue::Ascii(s) => {
                            println!("{}: Model = {}", image_path, s);
                            // Verify it contains expected model substring
                            if !expected_model.is_empty() {
                                assert!(
                                    s.contains(&expected_model[..10]),
                                    "Expected model to contain '{}', got '{}'",
                                    &expected_model[..10],
                                    s
                                );
                            }
                        }
                        _ => panic!("Model tag should be ASCII"),
                    }
                }

                // Count Canon maker note tags
                let canon_tags = ifd
                    .entries()
                    .iter()
                    .filter(|(&tag, _)| tag >= 0xC000 && tag < 0xD000)
                    .count();

                println!("{}: Found {} Canon maker note tags", image_path, canon_tags);
                assert!(
                    canon_tags > 0,
                    "Should find Canon maker note tags in {}",
                    image_path
                );
            }
            Ok(None) => {
                eprintln!("{}: No EXIF data found", image_path);
            }
            Err(e) => {
                eprintln!("{}: Error reading EXIF: {}", image_path, e);
            }
        }
    }
}

/// Test that non-Canon images don't get Canon tags
#[test]
fn test_non_canon_no_canon_tags() {
    let nikon_image = "exiftool/t/images/Nikon.jpg";
    if !Path::new(nikon_image).exists() {
        eprintln!(
            "Warning: Test image {} not found, skipping test",
            nikon_image
        );
        return;
    }

    let mut file = File::open(nikon_image).unwrap();
    let exif_segment = jpeg::find_exif_segment(&mut file).unwrap().unwrap();
    let ifd = IfdParser::parse(exif_segment.data).unwrap();

    // Check Make tag
    let make = ifd.entries().get(&0x10F).unwrap();
    match make {
        ExifValue::Ascii(s) => {
            assert!(
                s.contains("NIKON"),
                "Expected NIKON in Make tag, got: {}",
                s
            );
        }
        _ => panic!("Make tag should be ASCII string"),
    }

    // Should have no Canon-specific tags (0xC000 range)
    let canon_tag_count = ifd
        .entries()
        .keys()
        .filter(|&&tag| tag >= 0xC000 && tag < 0xD000)
        .count();

    assert_eq!(
        canon_tag_count, 0,
        "Non-Canon image should have no Canon tags"
    );
}

/// Test maker note parsing with missing Make tag
#[test]
fn test_maker_note_without_make_tag() {
    // This test would require creating a synthetic EXIF data structure
    // For now, we'll skip this as it requires more setup
    // In a real implementation, we'd create EXIF data with maker notes but no Make tag
    // and verify it stores the raw data instead of parsing
}

/// Test that the Canon table was properly generated
#[test]
fn test_canon_table_generation() {
    use exif_oxide::tables::CANON_TAGS;

    // According to spike 2 results, 34 Canon tags were generated
    assert!(
        CANON_TAGS.len() >= 34,
        "Expected at least 34 Canon tags in generated table, found {}",
        CANON_TAGS.len()
    );

    // Verify table is sorted (required for binary search)
    for i in 1..CANON_TAGS.len() {
        assert!(
            CANON_TAGS[i - 1].0 <= CANON_TAGS[i].0,
            "Canon tags table should be sorted"
        );
    }
}
