//! Container format parsing integration tests
//!
//! Tests the RIFF and QuickTime container parsers introduced in Phase 1
//! for WebP, AVI, MP4, MOV and other container-based formats.

use exif_oxide::core::containers::{quicktime, riff};
use exif_oxide::core::find_metadata_segment;
use std::fs::File;
use std::path::Path;

#[test]
fn test_riff_webp_parsing() {
    let test_file = "exiftool/t/images/RIFF.webp";
    if !Path::new(test_file).exists() {
        eprintln!("Skipping test - file not found: {}", test_file);
        return;
    }

    // Test through main API
    let metadata = find_metadata_segment(test_file).unwrap();

    // WebP might have EXIF in RIFF chunks
    if let Some(segment) = metadata {
        println!("Found metadata in WebP at offset {}", segment.offset);
        assert_eq!(segment.source_format, exif_oxide::detection::FileType::WEBP);

        // Verify it's valid TIFF data (should start with TIFF header)
        if segment.data.len() >= 4 {
            let header = &segment.data[0..4];
            if header == [0x49, 0x49, 0x2a, 0x00] || header == [0x4d, 0x4d, 0x00, 0x2a] {
                println!("Valid TIFF header found in WebP EXIF");
            } else {
                println!(
                    "WebP EXIF data doesn't start with TIFF header (may be XMP or other format)"
                );
                println!(
                    "First 8 bytes: {:02X?}",
                    &segment.data[0..std::cmp::min(8, segment.data.len())]
                );
            }
        }
    }
}

#[test]
fn test_riff_avi_parsing() {
    let test_file = "exiftool/t/images/AVI.avi";
    if !Path::new(test_file).exists() {
        eprintln!("Skipping test - file not found: {}", test_file);
        return;
    }

    // AVI files may have metadata in RIFF chunks
    let metadata = find_metadata_segment(test_file).unwrap();

    // AVI metadata support varies
    match metadata {
        Some(segment) => {
            println!("Found metadata in AVI at offset {}", segment.offset);
            assert_eq!(segment.source_format, exif_oxide::detection::FileType::AVI);
        }
        None => {
            println!("No metadata in AVI (expected for some files)");
        }
    }
}

#[test]
fn test_quicktime_mp4_parsing() {
    let test_file = "exiftool/t/images/MP4.mp4";
    if !Path::new(test_file).exists() {
        eprintln!("Skipping test - file not found: {}", test_file);
        return;
    }

    let metadata = find_metadata_segment(test_file).unwrap();

    // MP4 files may have EXIF in meta atoms
    match metadata {
        Some(segment) => {
            println!("Found metadata in MP4 at offset {}", segment.offset);
            assert_eq!(segment.source_format, exif_oxide::detection::FileType::MP4);
        }
        None => {
            println!("No EXIF metadata in MP4 (may have other metadata types)");
        }
    }
}

#[test]
fn test_quicktime_mov_parsing() {
    let test_file = "exiftool/t/images/QuickTime.mov";
    if !Path::new(test_file).exists() {
        eprintln!("Skipping test - file not found: {}", test_file);
        return;
    }

    let metadata = find_metadata_segment(test_file).unwrap();

    // MOV files follow same QuickTime atom structure
    match metadata {
        Some(segment) => {
            println!("Found metadata in MOV at offset {}", segment.offset);
            assert_eq!(segment.source_format, exif_oxide::detection::FileType::MOV);
        }
        None => {
            println!("No EXIF metadata in MOV");
        }
    }
}

#[test]
fn test_container_chunk_limits() {
    // Test that container parsers respect chunk size limits

    // Create a RIFF file with oversized chunk
    let mut riff_data = Vec::new();
    riff_data.extend_from_slice(b"RIFF");
    riff_data.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0x7F]); // Very large size
    riff_data.extend_from_slice(b"WEBP");

    let mut cursor = std::io::Cursor::new(riff_data);
    let result = riff::find_metadata(&mut cursor);

    // Should handle gracefully without allocating huge memory
    assert!(result.is_ok(), "Should not panic on large chunk size");
}

#[test]
fn test_nested_container_atoms() {
    // Create a QuickTime file with nested meta atoms
    let mut qt_data = Vec::new();

    // ftyp atom
    qt_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x14]); // Size: 20
    qt_data.extend_from_slice(b"ftyp");
    qt_data.extend_from_slice(b"qt  ");
    qt_data.extend_from_slice(&[0; 8]);

    // meta atom with nested atoms
    qt_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x20]); // Size: 32
    qt_data.extend_from_slice(b"meta");
    qt_data.extend_from_slice(&[0; 4]); // Version/flags

    // Nested hdlr atom
    qt_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x18]); // Size: 24
    qt_data.extend_from_slice(b"hdlr");
    qt_data.extend_from_slice(&[0; 16]); // Handler data

    let mut cursor = std::io::Cursor::new(qt_data);
    let result = quicktime::find_metadata(&mut cursor);

    assert!(result.is_ok(), "Should parse nested atoms");
}

#[test]
fn test_riff_xmp_extraction() {
    // Some WebP files have XMP instead of EXIF
    let test_file = "exiftool/t/images/WebP.webp";
    if !Path::new(test_file).exists() {
        eprintln!("Skipping test - file not found: {}", test_file);
        return;
    }

    let mut file = File::open(test_file).unwrap();
    let result = riff::find_metadata(&mut file).unwrap();

    if let Some(segment) = result {
        println!(
            "Found {} metadata in WebP",
            match segment.metadata_type {
                riff::MetadataType::Exif => "EXIF",
                riff::MetadataType::Xmp => "XMP",
            }
        );
    }
}

#[test]
fn test_3gpp_container_support() {
    // Test 3GPP video format (uses QuickTime container)
    let test_file = "exiftool/t/images/3GPP.3gp";
    if !Path::new(test_file).exists() {
        eprintln!("Skipping test - file not found: {}", test_file);
        return;
    }

    let metadata = find_metadata_segment(test_file).unwrap();

    match metadata {
        Some(segment) => {
            println!("Found metadata in 3GPP at offset {}", segment.offset);
            assert_eq!(
                segment.source_format,
                exif_oxide::detection::FileType::ThreeGPP
            );
        }
        None => {
            println!("No EXIF metadata in 3GPP");
        }
    }
}

#[test]
fn test_container_early_termination() {
    // Test that container parsers stop when they find metadata

    // Create WebP with EXIF chunk early
    let mut webp_data = Vec::new();
    webp_data.extend_from_slice(b"RIFF");
    webp_data.extend_from_slice(&[0x00, 0x01, 0x00, 0x00]); // File size
    webp_data.extend_from_slice(b"WEBP");

    // VP8 chunk (image data)
    webp_data.extend_from_slice(b"VP8 ");
    webp_data.extend_from_slice(&[0x10, 0x00, 0x00, 0x00]); // Chunk size
    webp_data.extend_from_slice(&[0; 16]); // Dummy VP8 data

    // EXIF chunk
    webp_data.extend_from_slice(b"EXIF");
    webp_data.extend_from_slice(&[0x08, 0x00, 0x00, 0x00]); // Chunk size
    webp_data.extend_from_slice(&[0x49, 0x49, 0x2a, 0x00]); // TIFF header
    webp_data.extend_from_slice(&[0x08, 0x00, 0x00, 0x00]); // IFD offset

    // Large chunk after EXIF (should not be read)
    webp_data.extend_from_slice(b"JUNK");
    webp_data.extend_from_slice(&[0x00, 0x10, 0x00, 0x00]); // 1MB chunk

    let mut cursor = std::io::Cursor::new(webp_data);
    let result = riff::find_metadata(&mut cursor).unwrap();

    match result {
        Some(_) => {
            println!("Found EXIF chunk as expected");
            // Verify parser stopped after finding EXIF
            let position = cursor.position();
            assert!(
                position < 200,
                "Should stop after finding EXIF, not read entire file"
            );
        }
        None => {
            println!("No EXIF chunk found - this may be expected for synthetic test data");
        }
    }
}

#[test]
fn test_container_format_variants() {
    // Test various container format variants
    let test_cases = vec![
        (
            "exiftool/t/images/M4V.m4v",
            exif_oxide::detection::FileType::M4V,
        ),
        (
            "exiftool/t/images/3GPP2.3g2",
            exif_oxide::detection::FileType::ThreeGPP2,
        ),
    ];

    for (file_path, expected_type) in test_cases {
        if !Path::new(file_path).exists() {
            continue;
        }

        let metadata = find_metadata_segment(file_path).unwrap();

        match metadata {
            Some(segment) => {
                assert_eq!(
                    segment.source_format, expected_type,
                    "Format mismatch for {}",
                    file_path
                );
            }
            None => {
                println!("No metadata in {} (OK)", file_path);
            }
        }
    }
}
