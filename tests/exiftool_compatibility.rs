//! ExifTool compatibility tests
//!
//! These tests compare exif-oxide output with ExifTool output
//! to ensure compatibility across all supported formats.

use exif_oxide::{core::find_metadata_segment, read_basic_exif};
use serde_json::Value;
use std::path::Path;
use std::process::Command;

/// Run ExifTool and get JSON output for comparison
fn get_exiftool_output(path: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let output = Command::new("./exiftool/exiftool")
        .arg("-json")
        .arg("-g")
        .arg("-n") // Numeric output for proper value comparison
        .arg(path)
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "ExifTool failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    let json_str = String::from_utf8(output.stdout)?;
    let json: Vec<Value> = serde_json::from_str(&json_str)?;

    Ok(json.into_iter().next().unwrap_or(Value::Null))
}

/// Compare basic EXIF fields between exif-oxide and ExifTool
#[test]
fn test_jpeg_compatibility() {
    let test_file = "exiftool/t/images/Canon.jpg";
    if !Path::new(test_file).exists() {
        eprintln!("Skipping test - file not found: {}", test_file);
        return;
    }

    // Get exif-oxide output
    let exif_oxide_result = read_basic_exif(test_file).unwrap();

    // Get ExifTool output
    let exiftool_json = get_exiftool_output(test_file).unwrap();

    // Compare Make
    if let Some(make) = exif_oxide_result.make {
        let exiftool_make = exiftool_json["EXIF"]["Make"]
            .as_str()
            .or_else(|| exiftool_json["IFD0"]["Make"].as_str())
            .unwrap_or("");
        assert_eq!(make, exiftool_make, "Make mismatch");
    }

    // Compare Model
    if let Some(model) = exif_oxide_result.model {
        let exiftool_model = exiftool_json["EXIF"]["Model"]
            .as_str()
            .or_else(|| exiftool_json["IFD0"]["Model"].as_str())
            .unwrap_or("");
        assert_eq!(model, exiftool_model, "Model mismatch");
    }

    // Compare Orientation
    if let Some(orientation) = exif_oxide_result.orientation {
        let exiftool_orientation = exiftool_json["EXIF"]["Orientation"]
            .as_u64()
            .or_else(|| exiftool_json["IFD0"]["Orientation"].as_u64())
            .unwrap_or(0) as u16;
        assert_eq!(orientation, exiftool_orientation, "Orientation mismatch");
    }
}

/// Test TIFF format compatibility
#[test]
fn test_tiff_compatibility() {
    let test_file = "exiftool/t/images/ExifTool.tif";
    if !Path::new(test_file).exists() {
        eprintln!("Skipping test - file not found: {}", test_file);
        return;
    }

    // Verify we can read TIFF files
    let metadata = find_metadata_segment(test_file).unwrap();
    assert!(metadata.is_some(), "Should find metadata in TIFF file");

    let basic_exif = read_basic_exif(test_file).unwrap();
    assert!(
        basic_exif.make.is_some() || basic_exif.model.is_some(),
        "Should extract basic EXIF from TIFF"
    );
}

/// Test PNG format compatibility  
#[test]
fn test_png_compatibility() {
    // Test PNG with eXIf chunk
    let test_file = "exiftool/t/images/PNG.png";
    if !Path::new(test_file).exists() {
        eprintln!("Skipping test - file not found: {}", test_file);
        return;
    }

    let metadata = find_metadata_segment(test_file).unwrap();
    // PNG.png might not have EXIF, which is OK
    match metadata {
        Some(_) => println!("Found EXIF in PNG"),
        None => println!("No EXIF in PNG (expected)"),
    }
}

/// Test RAW format compatibility (CR2)
#[test]
fn test_cr2_compatibility() {
    let test_file = "exiftool/t/images/CanonRaw.cr2";
    if !Path::new(test_file).exists() {
        eprintln!("Skipping test - file not found: {}", test_file);
        return;
    }

    let metadata = find_metadata_segment(test_file).unwrap();
    assert!(metadata.is_some(), "Should find metadata in CR2 file");

    let basic_exif = read_basic_exif(test_file).unwrap();
    assert_eq!(basic_exif.make, Some("Canon".to_string()));
}

/// Test WebP format support
#[test]
fn test_webp_format() {
    let test_file = "exiftool/t/images/WebP.webp";
    if !Path::new(test_file).exists() {
        eprintln!("Skipping test - file not found: {}", test_file);
        return;
    }

    // WebP may or may not have EXIF
    let _ = find_metadata_segment(test_file).unwrap();
}

/// Test video format support (MP4)
#[test]
fn test_mp4_format() {
    let test_file = "exiftool/t/images/MP4.mp4";
    if !Path::new(test_file).exists() {
        eprintln!("Skipping test - file not found: {}", test_file);
        return;
    }

    // MP4 may have metadata in various atoms
    let _ = find_metadata_segment(test_file).unwrap();
}

/// Test format detection matches ExifTool
#[test]
fn test_format_detection_compatibility() {
    use exif_oxide::detection::{detect_file_type, FileType};
    use std::fs::File;
    use std::io::Read;

    let test_cases = vec![
        ("exiftool/t/images/Canon.jpg", FileType::JPEG),
        ("exiftool/t/images/ExifTool.tif", FileType::TIFF),
        ("exiftool/t/images/PNG.png", FileType::PNG),
        ("exiftool/t/images/CanonRaw.cr2", FileType::CR2),
        ("exiftool/t/images/Nikon.nef", FileType::NEF),
    ];

    for (file_path, expected_type) in test_cases {
        if !Path::new(file_path).exists() {
            continue;
        }

        let mut file = File::open(file_path).unwrap();
        let mut buffer = vec![0u8; 1024];
        let bytes_read = file.read(&mut buffer).unwrap();
        buffer.truncate(bytes_read);

        let detected = detect_file_type(&buffer).unwrap();
        assert_eq!(
            detected.file_type, expected_type,
            "Format detection mismatch for {}",
            file_path
        );
    }
}

/// Test that we handle missing EXIF gracefully like ExifTool
#[test]
fn test_no_exif_handling() {
    // GIF files typically have no EXIF
    let test_file = "exiftool/t/images/GIF.gif";
    if Path::new(test_file).exists() {
        let metadata = find_metadata_segment(test_file).unwrap();
        assert!(metadata.is_none(), "GIF should have no EXIF metadata");
    }
}
