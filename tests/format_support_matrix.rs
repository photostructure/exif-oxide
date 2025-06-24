//! Format support matrix test
//!
//! This test generates a compatibility matrix showing which formats
//! are successfully parsed by exif-oxide.

#![allow(clippy::all)]
#![allow(unused_variables)]
#![allow(dead_code)]

use exif_oxide::core::find_metadata_segment;
use exif_oxide::detection::{detect_file_type, FileType};
use std::fs;
use std::path::Path;

#[derive(Debug)]
struct FormatTestResult {
    format: FileType,
    file_name: String,
    detection_ok: bool,
    metadata_found: bool,
    error: Option<String>,
}

/// Test all formats in the ExifTool test suite
#[test]
fn generate_format_support_matrix() {
    let test_dir = "exiftool/t/images";
    if !Path::new(test_dir).exists() {
        eprintln!("ExifTool test images not found at {}", test_dir);
        return;
    }

    let mut results = Vec::new();

    // Read all files in the test directory
    if let Ok(entries) = fs::read_dir(test_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Detect format
                let format = if let Ok(mut file) = fs::File::open(&path) {
                    let mut buffer = vec![0u8; 1024];
                    if let Ok(bytes_read) = std::io::Read::read(&mut file, &mut buffer) {
                        buffer.truncate(bytes_read);
                        detect_file_type(&buffer).ok().map(|info| info.file_type)
                    } else {
                        None
                    }
                } else {
                    None
                };

                let format = format.unwrap_or(FileType::Unknown);

                // Try to extract metadata
                let (metadata_found, error) = match find_metadata_segment(&path) {
                    Ok(Some(_)) => (true, None),
                    Ok(None) => (false, None),
                    Err(e) => (false, Some(e.to_string())),
                };

                results.push(FormatTestResult {
                    format,
                    file_name,
                    detection_ok: format != FileType::Unknown,
                    metadata_found,
                    error,
                });
            }
        }
    }

    // Sort results by format
    results.sort_by_key(|r| format!("{:?}", r.format));

    // Print summary
    println!("\n=== Format Support Matrix ===\n");
    println!(
        "{:<20} {:<10} {:<10} {:<30}",
        "Format", "Detected", "Metadata", "File"
    );
    println!("{:-<70}", "");

    let mut format_stats = std::collections::HashMap::new();

    for result in &results {
        let format_name = format!("{:?}", result.format);
        let detected = if result.detection_ok { "✓" } else { "✗" };
        let metadata = if result.metadata_found { "✓" } else { "✗" };

        println!(
            "{:<20} {:<10} {:<10} {:<30}",
            format_name, detected, metadata, result.file_name
        );

        // Track statistics
        let stat = format_stats.entry(result.format).or_insert((0, 0, 0));
        stat.0 += 1; // Total files
        if result.detection_ok {
            stat.1 += 1; // Detected
        }
        if result.metadata_found {
            stat.2 += 1; // Metadata extracted
        }
    }

    // Print statistics
    println!("\n=== Format Statistics ===\n");
    println!(
        "{:<20} {:<10} {:<10} {:<10}",
        "Format", "Files", "Detected", "Metadata"
    );
    println!("{:-<50}", "");

    let mut sorted_stats: Vec<_> = format_stats.into_iter().collect();
    sorted_stats.sort_by_key(|(format, _)| format!("{:?}", format));

    let mut total_files = 0;
    let mut total_detected = 0;
    let mut total_metadata = 0;

    for (format, (files, detected, metadata)) in sorted_stats {
        if files > 0 {
            println!(
                "{:<20} {:<10} {:<10} {:<10}",
                format!("{:?}", format),
                files,
                detected,
                metadata
            );
            total_files += files;
            total_detected += detected;
            total_metadata += metadata;
        }
    }

    println!("{:-<50}", "");
    println!(
        "{:<20} {:<10} {:<10} {:<10}",
        "TOTAL", total_files, total_detected, total_metadata
    );

    // Calculate success rates
    let detection_rate = (total_detected as f32 / total_files as f32 * 100.0) as u32;
    let metadata_rate = (total_metadata as f32 / total_detected as f32 * 100.0) as u32;

    println!("\nDetection Success Rate: {}%", detection_rate);
    println!(
        "Metadata Extraction Rate: {}% (of detected files)",
        metadata_rate
    );
}

/// Test specific format edge cases
#[test]
fn test_format_edge_cases() {
    // Test TIFF with multiple IFDs
    if Path::new("exiftool/t/images/ExifTool.tif").exists() {
        let metadata = find_metadata_segment("exiftool/t/images/ExifTool.tif").unwrap();
        assert!(metadata.is_some(), "Multi-IFD TIFF should be supported");
    }

    // Test JPEG with multiple APP1 segments
    if Path::new("exiftool/t/images/Canon.jpg").exists() {
        let metadata = find_metadata_segment("exiftool/t/images/Canon.jpg").unwrap();
        assert!(metadata.is_some(), "Multi-segment JPEG should be supported");
    }
}

/// Test format aliases and variations
#[test]
fn test_format_aliases() {
    use exif_oxide::detection::detect_file_type;

    // Test that JPEG variations are recognized
    let jpeg_soi = vec![0xFF, 0xD8, 0xFF];
    let detected = detect_file_type(&jpeg_soi).unwrap();
    assert_eq!(detected.file_type, FileType::JPEG);

    // Test TIFF variations (II and MM)
    let tiff_le = vec![0x49, 0x49, 0x2A, 0x00];
    let detected = detect_file_type(&tiff_le).unwrap();
    assert_eq!(detected.file_type, FileType::TIFF);

    let tiff_be = vec![0x4D, 0x4D, 0x00, 0x2A];
    let detected = detect_file_type(&tiff_be).unwrap();
    assert_eq!(detected.file_type, FileType::TIFF);
}
