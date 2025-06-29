//! Integration tests for exiftool_sync extraction tools

use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
#[ignore] // Run with: cargo test --test exiftool_sync_integration -- --ignored
fn test_binary_formats_extraction_integration() {
    // Check if third-party/exiftool exists
    if !Path::new("third-party/exiftool").exists() {
        eprintln!("Skipping test: third-party/exiftool not found");
        return;
    }

    // Clean up any existing generated files
    let formats_dir = Path::new("src/binary/formats");
    if formats_dir.exists() {
        fs::remove_dir_all(formats_dir).ok();
    }

    // Run the extraction
    let output = Command::new("cargo")
        .args(["run", "--bin", "exiftool_sync", "extract", "binary-formats"])
        .output()
        .expect("Failed to run exiftool_sync");

    // Check that it succeeded
    assert!(
        output.status.success(),
        "Extraction failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify that files were generated
    assert!(formats_dir.exists(), "formats directory not created");

    // Check for expected manufacturer files
    let expected_files = ["canon.rs", "nikon.rs", "sony.rs", "olympus.rs"];
    for file in &expected_files {
        let path = formats_dir.join(file);
        assert!(path.exists(), "Expected file {} not found", file);

        // Verify file content
        let content =
            fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read {}", file));

        // Check for required headers
        assert!(
            content.contains("AUTO-GENERATED"),
            "{} missing AUTO-GENERATED header",
            file
        );
        assert!(
            content.contains("EXIFTOOL-SOURCE"),
            "{} missing EXIFTOOL-SOURCE",
            file
        );
        assert!(
            content.contains("BinaryDataTable"),
            "{} missing BinaryDataTable imports",
            file
        );

        // Check for at least one table function
        assert!(
            content.contains("pub fn create_"),
            "{} missing table creation functions",
            file
        );
    }

    // Verify Canon specific content
    let canon_content =
        fs::read_to_string(formats_dir.join("canon.rs")).expect("Failed to read canon.rs");
    assert!(
        canon_content.contains("SensorInfo"),
        "Canon file missing SensorInfo table"
    );

    // Verify Nikon specific content
    let nikon_content =
        fs::read_to_string(formats_dir.join("nikon.rs")).expect("Failed to read nikon.rs");
    assert!(
        nikon_content.contains("ShotInfo"),
        "Nikon file missing ShotInfo tables"
    );

    println!("Binary formats extraction test passed!");
}

#[test]
fn test_exiftool_sync_help() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "exiftool_sync", "help"])
        .output()
        .expect("Failed to run exiftool_sync help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Check for clap-generated help content
    assert!(stdout.contains("Tool to synchronize exif-oxide with ExifTool updates"));
    assert!(stdout.contains("extract"));
    assert!(stdout.contains("Extract algorithms from ExifTool source"));
    assert!(stdout.contains("Commands:"));
}

#[test]
fn test_exiftool_sync_status() {
    // This test should work even without exiftool-sync.toml
    let output = Command::new("cargo")
        .args(["run", "--bin", "exiftool_sync", "status"])
        .output()
        .expect("Failed to run exiftool_sync status");

    // Should either succeed or fail gracefully with config not found
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("Failed to read config") || stderr.contains("exiftool-sync.toml"),
            "Unexpected error: {}",
            stderr
        );
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("ExifTool Synchronization Status"));
    }
}
