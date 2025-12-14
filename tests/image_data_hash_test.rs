//! ImageDataHash integration tests
//!
//! These tests verify that exif-oxide computes ImageDataHash identically to ExifTool.
//! The hash is computed on the actual image data (excluding metadata), allowing
//! detection of image content changes while ignoring metadata edits.
//!
//! ExifTool reference: lib/Image/ExifTool/Writer.pl (ImageDataHash function)
//!                     lib/Image/ExifTool/PNG.pm (IDAT chunk hashing)
//!
//! Note: These tests require the `integration-tests` feature and external test images.

#![cfg(feature = "integration-tests")]

use exif_oxide::formats::extract_metadata;
use exif_oxide::hash::ImageHashType;
use exif_oxide::types::FilterOptions;
use std::path::Path;
use std::process::Command;

/// Path to ExifTool binary
const EXIFTOOL_PATH: &str = "third-party/exiftool/exiftool";

/// Test images for hash verification
const TEST_JPEG: &str = "../test-images/Canon/CanonCanoScanLiDE100.jpg";
const TEST_PNG: &str = "../test-images/exif-oxide-test-images/example.png";

/// Get ImageDataHash from ExifTool for comparison
fn exiftool_image_hash(path: &str, hash_type: &str) -> Option<String> {
    let hash_type_lower = hash_type.to_lowercase();
    let output = Command::new(EXIFTOOL_PATH)
        .args([
            "-api",
            "requesttags=imagedatahash",
            "-api",
            &format!("imagehashtype={}", hash_type_lower),
            "-ImageDataHash",
            "-s",
            "-s",
            "-s",
            path,
        ])
        .output()
        .ok()?;

    if output.status.success() {
        let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if hash.is_empty() {
            None
        } else {
            Some(hash)
        }
    } else {
        None
    }
}

/// Get ImageDataHash from exif-oxide
fn exif_oxide_image_hash(path: &str, hash_type: ImageHashType) -> Option<String> {
    let filter_options = FilterOptions {
        compute_image_hash: true,
        image_hash_type: hash_type,
        ..FilterOptions::default()
    };

    let metadata = extract_metadata(Path::new(path), false, false, Some(filter_options)).ok()?;

    // Find ImageDataHash in tags
    for tag in &metadata.tags {
        if tag.name == "ImageDataHash" {
            if let exif_oxide::types::TagValue::String(hash) = &tag.value {
                return Some(hash.clone());
            }
        }
    }
    None
}

#[test]
fn test_jpeg_image_hash_md5_matches_exiftool() {
    let path = TEST_JPEG;

    // Skip if test image not available
    if !Path::new(path).exists() {
        eprintln!("Skipping test: {} not found", path);
        return;
    }

    let exiftool_hash = exiftool_image_hash(path, "md5");
    let oxide_hash = exif_oxide_image_hash(path, ImageHashType::Md5);

    assert!(
        exiftool_hash.is_some(),
        "ExifTool should produce a hash for {}",
        path
    );
    assert!(
        oxide_hash.is_some(),
        "exif-oxide should produce a hash for {}",
        path
    );

    assert_eq!(
        exiftool_hash.unwrap(),
        oxide_hash.unwrap(),
        "JPEG MD5 hash should match ExifTool for {}",
        path
    );
}

#[test]
fn test_jpeg_image_hash_sha256_matches_exiftool() {
    let path = TEST_JPEG;

    // Skip if test image not available
    if !Path::new(path).exists() {
        eprintln!("Skipping test: {} not found", path);
        return;
    }

    let exiftool_hash = exiftool_image_hash(path, "sha256");
    let oxide_hash = exif_oxide_image_hash(path, ImageHashType::Sha256);

    assert!(
        exiftool_hash.is_some(),
        "ExifTool should produce a SHA256 hash for {}",
        path
    );
    assert!(
        oxide_hash.is_some(),
        "exif-oxide should produce a SHA256 hash for {}",
        path
    );

    assert_eq!(
        exiftool_hash.unwrap(),
        oxide_hash.unwrap(),
        "JPEG SHA256 hash should match ExifTool for {}",
        path
    );
}

#[test]
fn test_png_image_hash_md5_matches_exiftool() {
    let path = TEST_PNG;

    // Skip if test image not available
    if !Path::new(path).exists() {
        eprintln!("Skipping test: {} not found", path);
        return;
    }

    let exiftool_hash = exiftool_image_hash(path, "md5");
    let oxide_hash = exif_oxide_image_hash(path, ImageHashType::Md5);

    assert!(
        exiftool_hash.is_some(),
        "ExifTool should produce a hash for {}",
        path
    );
    assert!(
        oxide_hash.is_some(),
        "exif-oxide should produce a hash for {}",
        path
    );

    assert_eq!(
        exiftool_hash.unwrap(),
        oxide_hash.unwrap(),
        "PNG MD5 hash should match ExifTool for {}",
        path
    );
}

#[test]
fn test_png_image_hash_sha256_matches_exiftool() {
    let path = TEST_PNG;

    // Skip if test image not available
    if !Path::new(path).exists() {
        eprintln!("Skipping test: {} not found", path);
        return;
    }

    let exiftool_hash = exiftool_image_hash(path, "sha256");
    let oxide_hash = exif_oxide_image_hash(path, ImageHashType::Sha256);

    assert!(
        exiftool_hash.is_some(),
        "ExifTool should produce a SHA256 hash for {}",
        path
    );
    assert!(
        oxide_hash.is_some(),
        "exif-oxide should produce a SHA256 hash for {}",
        path
    );

    assert_eq!(
        exiftool_hash.unwrap(),
        oxide_hash.unwrap(),
        "PNG SHA256 hash should match ExifTool for {}",
        path
    );
}

#[test]
fn test_hash_not_computed_when_not_requested() {
    let path = TEST_JPEG;

    // Skip if test image not available
    if !Path::new(path).exists() {
        eprintln!("Skipping test: {} not found", path);
        return;
    }

    // Default filter options - no hash computation
    let filter_options = FilterOptions::default();
    let metadata = extract_metadata(Path::new(path), false, false, Some(filter_options))
        .expect("Should extract metadata");

    // Should NOT have ImageDataHash tag
    let has_hash = metadata.tags.iter().any(|t| t.name == "ImageDataHash");
    assert!(
        !has_hash,
        "ImageDataHash should not be present when not requested"
    );
}

#[test]
fn test_sha512_hash_length() {
    let path = TEST_JPEG;

    // Skip if test image not available
    if !Path::new(path).exists() {
        eprintln!("Skipping test: {} not found", path);
        return;
    }

    let hash = exif_oxide_image_hash(path, ImageHashType::Sha512);
    assert!(hash.is_some(), "Should produce SHA512 hash");

    // SHA512 produces 128 hex characters
    let hash_str = hash.unwrap();
    assert_eq!(
        hash_str.len(),
        128,
        "SHA512 hash should be 128 hex characters"
    );
}

/// Test that hashes are consistent across multiple runs
#[test]
fn test_hash_consistency() {
    let path = TEST_JPEG;

    // Skip if test image not available
    if !Path::new(path).exists() {
        eprintln!("Skipping test: {} not found", path);
        return;
    }

    let hash1 = exif_oxide_image_hash(path, ImageHashType::Md5);
    let hash2 = exif_oxide_image_hash(path, ImageHashType::Md5);

    assert_eq!(
        hash1, hash2,
        "Hash should be consistent across multiple runs"
    );
}

/// Test with SHA512 against ExifTool
#[test]
fn test_jpeg_image_hash_sha512_matches_exiftool() {
    let path = TEST_JPEG;

    // Skip if test image not available
    if !Path::new(path).exists() {
        eprintln!("Skipping test: {} not found", path);
        return;
    }

    let exiftool_hash = exiftool_image_hash(path, "sha512");
    let oxide_hash = exif_oxide_image_hash(path, ImageHashType::Sha512);

    assert!(
        exiftool_hash.is_some(),
        "ExifTool should produce a SHA512 hash for {}",
        path
    );
    assert!(
        oxide_hash.is_some(),
        "exif-oxide should produce a SHA512 hash for {}",
        path
    );

    assert_eq!(
        exiftool_hash.unwrap(),
        oxide_hash.unwrap(),
        "JPEG SHA512 hash should match ExifTool for {}",
        path
    );
}
