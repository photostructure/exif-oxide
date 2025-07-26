//! Integration tests for exif-oxide
//!
//! These tests compare our output with ExifTool's output to ensure compatibility.
//! For Milestone 0a, we test the basic CLI structure and JSON format.
//!
//! Note: These tests require the `integration-tests` feature to be enabled and
//! external test assets to be available. They are automatically skipped in published crates.

#![cfg(feature = "integration-tests")]

use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

mod common;
use common::{CANON_T3I_CR2, CANON_T3I_JPG};

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
        .args(["run", "--", CANON_T3I_JPG])
        .output()
        .expect("Failed to run CLI with test image");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Parse as JSON array (ExifTool format)
    let json: Value = serde_json::from_str(&stdout).expect("Output should be valid JSON");
    assert!(json.is_array());
    let array = json.as_array().unwrap();
    assert_eq!(array.len(), 1);
    let obj = &array[0];

    // Check required fields that exif-oxide provides
    assert!(obj.get("SourceFile").is_some());
    assert!(obj.get("ExifToolVersion").is_some());

    // Check that SourceFile matches input
    assert_eq!(obj["SourceFile"].as_str().unwrap(), CANON_T3I_JPG);
}

/// Test CLI with --show-missing flag
#[test]
fn test_cli_show_missing() {
    let output = Command::new("cargo")
        .args(["run", "--", "--show-missing", CANON_T3I_JPG])
        .output()
        .expect("Failed to run CLI with --show-missing");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    let json: Value = serde_json::from_str(&stdout).expect("Output should be valid JSON");
    assert!(json.is_array());
    let array = json.as_array().unwrap();
    assert_eq!(array.len(), 1);
    let obj = &array[0];

    // Should include MissingImplementations
    assert!(obj.get("MissingImplementations").is_some());
    let missing = obj["MissingImplementations"].as_array().unwrap();
    assert!(!missing.is_empty());

    // Should contain expected missing items (tag IDs and conversion functions)
    let missing_str = missing
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect::<Vec<_>>();

    // Current implementation tracks missing PrintConv/ValueConv expressions with tag context
    assert!(
        missing_str
            .iter()
            .any(|s| s.contains("PrintConv:") || s.contains("ValueConv:")),
        "Should contain missing PrintConv or ValueConv expressions"
    );
    assert!(
        missing_str.iter().any(|s| s.contains("[used by tags:")),
        "Should show which tags use the missing conversions"
    );
}

/// Test CLI error handling for non-existent file
#[test]
fn test_cli_nonexistent_file() {
    let output = Command::new("cargo")
        .args(["run", "--", "nonexistent_file.jpg"])
        .output()
        .expect("Failed to run CLI with nonexistent file");

    // Our CLI returns success (0) with error info in JSON array for graceful batch processing
    // This differs from ExifTool but allows continuous processing of multiple files
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&stdout).expect("Output should be valid JSON");
    assert!(json.is_array());
    let array = json.as_array().unwrap();
    assert_eq!(array.len(), 1);
    let obj = &array[0];

    // Should have error information
    assert!(obj.get("errors").is_some());
    let errors = obj["errors"].as_array().unwrap();
    assert!(!errors.is_empty());
    let error_msg = errors[0].as_str().unwrap();
    assert!(error_msg.contains("File not found") || error_msg.contains("not found"));
}

/// Test JSON structure compatibility with ExifTool format
#[test]
fn test_json_structure_compatibility() {
    let output = Command::new("cargo")
        .args(["run", "--", CANON_T3I_JPG])
        .output()
        .expect("Failed to run CLI");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&stdout).unwrap();

    // Test that it matches ExifTool's JSON array structure (even for single files)
    assert!(json.is_array());
    let array = json.as_array().unwrap();
    assert_eq!(array.len(), 1);

    let obj = array[0].as_object().unwrap();

    // Should have string values for basic fields
    assert!(obj["SourceFile"].is_string());
    assert!(obj["ExifToolVersion"].is_string());

    // Milestone 2 implements real EXIF parsing for ASCII and numeric tags
    // Should extract Make and Model from Canon image
    assert!(obj.contains_key("EXIF:Make"));
    assert!(obj.contains_key("EXIF:Model"));
    assert_eq!(obj["EXIF:Make"], "Canon");
    assert_eq!(obj["EXIF:Model"], "Canon EOS REBEL T3i");

    // Milestone 4: Should extract PrintConv converted values like Orientation
    if let Some(orientation) = obj.get("EXIF:Orientation") {
        assert!(orientation.is_string());
        // Should match ExifTool's PrintConv value for this Canon image
        assert_eq!(orientation, "Rotate 270 CW");
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
        .args(["-j", CANON_T3I_JPG])
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
        .args(["run", "--", CANON_T3I_JPG])
        .output()
        .expect("Failed to run our CLI");

    let our_json: Value =
        serde_json::from_str(&String::from_utf8(our_output.stdout).unwrap()).unwrap();

    // Our output is also an array, get the first element
    assert!(our_json.is_array());
    let our_array = our_json.as_array().unwrap();
    assert!(!our_array.is_empty());
    let our_data = &our_array[0];

    // Compare basic structure
    assert_eq!(
        our_data["SourceFile"].as_str().unwrap(),
        exiftool_data["SourceFile"].as_str().unwrap()
    );

    // Both should have these basic fields
    assert!(our_data.get("SourceFile").is_some());
    assert!(our_data.get("ExifToolVersion").is_some());

    // ExifTool has FileName, we derive it from SourceFile
    assert!(exiftool_data.get("FileName").is_some());

    // Note: For Milestone 0a, we expect differences in actual metadata
    // since we're using mock data. Future milestones will make these match.
    println!("Basic structure comparison passed!");
    println!("Note: Metadata values differ because we're using mock data in Milestone 0a");
}

/// Test with different file formats
#[test]
fn test_different_file_formats() {
    let test_files = vec![CANON_T3I_JPG, CANON_T3I_CR2];

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

        // JSON is now an array format
        assert!(json.is_array());
        let array = json.as_array().unwrap();
        assert_eq!(array.len(), 1);
        let obj = &array[0];
        assert_eq!(obj["SourceFile"].as_str().unwrap(), file);
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
    fn test_converter(val: &TagValue) -> TagValue {
        TagValue::string(format!("Converted: {val}"))
    }

    register_print_conv("test_function", test_converter);

    // Test it works
    let value = TagValue::U16(42);
    let result = apply_print_conv("test_function", &value);
    assert_eq!(result, "Converted: 42".into());

    // Test missing function fallback
    let missing_result = apply_print_conv("nonexistent_function", &value);
    assert_eq!(missing_result, TagValue::U16(42)); // Should return raw value
}
