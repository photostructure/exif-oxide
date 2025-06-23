// EXIFTOOL-COMPAT: Integration tests for file type detection
// Validates against ExifTool's output

use exif_oxide::detection::{detect_file_type, FileType};
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use std::process::Command;

#[test]
fn test_jpeg_detection() {
    let mut file = File::open("test-images/canon/Canon_T3i.JPG").unwrap();
    let mut buffer = vec![0; 1024];
    let bytes_read = file.read(&mut buffer).unwrap();
    buffer.truncate(bytes_read);

    let info = detect_file_type(&buffer).unwrap();

    assert_eq!(info.file_type, FileType::JPEG);
    assert_eq!(info.mime_type, "image/jpeg");
    assert!(!info.weak_detection);
}

#[test]
fn test_cr2_detection() {
    let mut file = File::open("test-images/canon/Canon_T3i.CR2").unwrap();
    let mut buffer = vec![0; 1024];
    let bytes_read = file.read(&mut buffer).unwrap();
    buffer.truncate(bytes_read);

    let info = detect_file_type(&buffer).unwrap();

    assert_eq!(info.file_type, FileType::CR2);
    assert_eq!(info.mime_type, "image/x-canon-cr2");
}

#[test]
fn test_nrw_detection() {
    let mut file = File::open("test-images/nikon/nikon_z8_73.NEF").unwrap();
    let mut buffer = vec![0; 1024];
    let bytes_read = file.read(&mut buffer).unwrap();
    buffer.truncate(bytes_read);

    let info = detect_file_type(&buffer).unwrap();

    assert_eq!(info.file_type, FileType::NRW);
    assert_eq!(info.mime_type, "image/x-nikon-nrw");
}

#[test]
fn test_mime_type_compatibility() {
    // Test that our MIME types match ExifTool's output
    let test_files = vec![
        ("test-images/canon/Canon_T3i.JPG", "image/jpeg"),
        ("test-images/canon/Canon_T3i.CR2", "image/x-canon-cr2"),
        // Note: NEF file is actually NRW format according to ExifTool
        ("test-images/nikon/nikon_z8_73.NEF", "image/x-nikon-nrw"),
    ];

    for (file_path, expected_mime) in test_files {
        let mut file = File::open(file_path).unwrap();
        let mut buffer = vec![0; 1024];
        let bytes_read = file.read(&mut buffer).unwrap();
        buffer.truncate(bytes_read);

        let info = detect_file_type(&buffer).unwrap();
        assert_eq!(
            info.mime_type, expected_mime,
            "MIME type mismatch for {}",
            file_path
        );
    }
}

#[test]
#[ignore] // Run with --ignored to test against actual ExifTool
fn test_against_exiftool_output() {
    // Compare our detection with ExifTool's output
    let test_files = vec![
        "test-images/canon/Canon_T3i.JPG",
        "test-images/canon/Canon_T3i.CR2",
        "test-images/nikon/nikon_z8_73.NEF",
    ];

    for file_path in test_files {
        // Get ExifTool's detection
        let output = Command::new("exiftool")
            .args(["-j", "-FileType", "-MIMEType", file_path])
            .output()
            .expect("Failed to run exiftool");

        let exiftool_json: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
        let exiftool_data = &exiftool_json[0];

        let exiftool_mime = exiftool_data["MIMEType"].as_str().unwrap();

        // Get our detection
        let mut file = File::open(file_path).unwrap();
        let mut buffer = vec![0; 1024];
        let bytes_read = file.read(&mut buffer).unwrap();
        buffer.truncate(bytes_read);

        let info = detect_file_type(&buffer).unwrap();

        assert_eq!(
            info.mime_type, exiftool_mime,
            "MIME type mismatch for {} - ours: {}, ExifTool: {}",
            file_path, info.mime_type, exiftool_mime
        );
    }
}

#[test]
fn test_file_mime_type_field() {
    // Test that File:MIMEType field is accessible
    let mut file = File::open("test-images/canon/Canon_T3i.JPG").unwrap();
    let mut buffer = vec![0; 1024];
    let bytes_read = file.read(&mut buffer).unwrap();
    buffer.truncate(bytes_read);

    let info = detect_file_type(&buffer).unwrap();

    // Verify the File:MIMEType field accessor works
    assert_eq!(info.file_mime_type(), "image/jpeg");
    assert_eq!(info.file_mime_type(), info.mime_type);
}
