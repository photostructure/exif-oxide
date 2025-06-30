//! Integration tests for exif-oxide
//!
//! These tests compare our output with ExifTool's output to ensure compatibility.
//! For Milestone 0a, we test the basic CLI structure and JSON format.

use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

/// Test CLI basic functionality
#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to run CLI with --help");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("exif-oxide"));
    assert!(stdout.contains("--show-missing"));
}

/// Test CLI with existing test image
#[test]
fn test_cli_with_test_image() {
    let output = Command::new("cargo")
        .args(["run", "--", "test-images/canon/Canon_T3i.JPG"])
        .output()
        .expect("Failed to run CLI with test image");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Parse as JSON
    let json: Value = serde_json::from_str(&stdout).expect("Output should be valid JSON");

    // Check required ExifTool fields
    assert!(json.get("SourceFile").is_some());
    assert!(json.get("ExifToolVersion").is_some());
    assert!(json.get("FileName").is_some());
    assert!(json.get("Directory").is_some());

    // Check that SourceFile matches input
    assert_eq!(
        json["SourceFile"].as_str().unwrap(),
        "test-images/canon/Canon_T3i.JPG"
    );
}

/// Test CLI with --show-missing flag
#[test]
fn test_cli_show_missing() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "--show-missing",
            "test-images/canon/Canon_T3i.JPG",
        ])
        .output()
        .expect("Failed to run CLI with --show-missing");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    let json: Value = serde_json::from_str(&stdout).expect("Output should be valid JSON");

    // Should include MissingImplementations
    assert!(json.get("MissingImplementations").is_some());
    let missing = json["MissingImplementations"].as_array().unwrap();
    assert!(!missing.is_empty());

    // Should contain expected missing items
    let missing_str = missing
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(missing_str
        .iter()
        .any(|s| s.contains("JPEG segment parsing")));
    assert!(missing_str
        .iter()
        .any(|s| s.contains("EXIF header parsing")));
}

/// Test CLI error handling for non-existent file
#[test]
fn test_cli_nonexistent_file() {
    let output = Command::new("cargo")
        .args(["run", "--", "nonexistent_file.jpg"])
        .output()
        .expect("Failed to run CLI with nonexistent file");

    // Should fail gracefully
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("not found") || stderr.contains("No such file"));
}

/// Test JSON structure compatibility with ExifTool format
#[test]
fn test_json_structure_compatibility() {
    let output = Command::new("cargo")
        .args(["run", "--", "test-images/canon/Canon_T3i.JPG"])
        .output()
        .expect("Failed to run CLI");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&stdout).unwrap();

    // Test that it matches ExifTool's basic JSON structure
    assert!(json.is_object());
    let obj = json.as_object().unwrap();

    // Should have string values for basic fields
    assert!(obj["SourceFile"].is_string());
    assert!(obj["ExifToolVersion"].is_string());
    assert!(obj["FileName"].is_string());
    assert!(obj["Directory"].is_string());

    // Should have numeric values for image dimensions
    assert!(obj["ImageWidth"].is_number());
    assert!(obj["ImageHeight"].is_number());

    // Test specific value types match ExifTool conventions
    if let Some(orientation) = obj.get("Orientation") {
        assert!(orientation.is_number());
    }
}

/// Compare basic structure with actual ExifTool (if available)
///
/// This test is optional and will be skipped if ExifTool is not installed
#[test]
fn test_compare_with_exiftool() {
    // Check if ExifTool is available
    let exiftool_check = Command::new("exiftool").arg("-ver").output();

    if exiftool_check.is_err() {
        println!("ExifTool not available, skipping comparison test");
        return;
    }

    // Get ExifTool output
    let exiftool_output = Command::new("exiftool")
        .args(["-j", "test-images/canon/Canon_T3i.JPG"])
        .output()
        .expect("Failed to run ExifTool");

    if !exiftool_output.status.success() {
        println!("ExifTool failed, skipping comparison");
        return;
    }

    let exiftool_json: Value =
        serde_json::from_str(&String::from_utf8(exiftool_output.stdout).unwrap())
            .expect("ExifTool should produce valid JSON");

    // ExifTool returns an array with one object
    let exiftool_data = &exiftool_json[0];

    // Get our output
    let our_output = Command::new("cargo")
        .args(["run", "--", "test-images/canon/Canon_T3i.JPG"])
        .output()
        .expect("Failed to run our CLI");

    let our_json: Value =
        serde_json::from_str(&String::from_utf8(our_output.stdout).unwrap()).unwrap();

    // Compare basic structure
    assert_eq!(
        our_json["SourceFile"].as_str().unwrap(),
        exiftool_data["SourceFile"].as_str().unwrap()
    );

    assert_eq!(
        our_json["FileName"].as_str().unwrap(),
        exiftool_data["FileName"].as_str().unwrap()
    );

    // Note: For Milestone 0a, we expect differences in actual metadata
    // since we're using mock data. Future milestones will make these match.
    println!("Basic structure comparison passed!");
    println!("Note: Metadata values differ because we're using mock data in Milestone 0a");
}

/// Test with different file formats
#[test]
fn test_different_file_formats() {
    let test_files = vec![
        "test-images/canon/Canon_T3i.JPG",
        "test-images/canon/Canon_T3i.CR2",
    ];

    for file in test_files {
        let path = PathBuf::from(file);
        if !path.exists() {
            continue; // Skip if test file doesn't exist
        }

        let output = Command::new("cargo")
            .args(["run", "--", file])
            .output()
            .unwrap_or_else(|_| panic!("Failed to run CLI with {file}"));

        if !output.status.success() {
            // Some formats might not be supported yet, that's ok for Milestone 0a
            let stderr = String::from_utf8(output.stderr).unwrap();
            if stderr.contains("Unsupported") {
                continue;
            } else {
                panic!("Unexpected error with {file}: {stderr}");
            }
        }

        let stdout = String::from_utf8(output.stdout).unwrap();
        let json: Value = serde_json::from_str(&stdout)
            .unwrap_or_else(|_| panic!("Output should be valid JSON for {file}"));

        assert_eq!(json["SourceFile"].as_str().unwrap(), file);
    }
}

/// Test registry functionality (basic smoke test)
#[test]
fn test_registry_functions() {
    use exif_oxide::registry::{apply_print_conv, clear_missing_tracking, register_print_conv};
    use exif_oxide::types::TagValue;

    // Clear any previous state
    clear_missing_tracking();

    // Register a test function
    fn test_converter(val: &TagValue) -> String {
        format!("Converted: {val}")
    }

    register_print_conv("test_function", test_converter);

    // Test it works
    let value = TagValue::U16(42);
    let result = apply_print_conv("test_function", &value);
    assert_eq!(result, "Converted: 42");

    // Test missing function fallback
    let missing_result = apply_print_conv("nonexistent_function", &value);
    assert_eq!(missing_result, "42"); // Should return raw value
}
