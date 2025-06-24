//! Integration tests for container format support

use exif_oxide::core::find_metadata_segment;
use std::path::Path;

#[test]
fn test_webp_format_detection() {
    // Test that WebP files are recognized but may not have EXIF
    if Path::new("exiftool/t/images/WebP.webp").exists() {
        let result = find_metadata_segment("exiftool/t/images/WebP.webp");

        // WebP may or may not have EXIF data
        match result {
            Ok(Some(segment)) => {
                // If it has metadata, verify it's valid
                assert!(!segment.data.is_empty());
            }
            Ok(None) => {
                // No metadata is also valid for WebP
            }
            Err(e) => {
                panic!("Error reading WebP file: {}", e);
            }
        }
    }
}

#[test]
fn test_mp4_format_detection() {
    // Test that MP4 files are recognized
    if Path::new("exiftool/t/images/MP4.mp4").exists() {
        let result = find_metadata_segment("exiftool/t/images/MP4.mp4");

        // MP4 may or may not have EXIF data
        match result {
            Ok(_) => {
                // Either Some or None is valid
            }
            Err(e) => {
                panic!("Error reading MP4 file: {}", e);
            }
        }
    }
}

#[test]
fn test_avi_format_detection() {
    // Test that AVI files are recognized
    if Path::new("exiftool/t/images/AVI.avi").exists() {
        let result = find_metadata_segment("exiftool/t/images/AVI.avi");

        // AVI rarely has EXIF data
        match result {
            Ok(_) => {
                // Either Some or None is valid
            }
            Err(e) => {
                panic!("Error reading AVI file: {}", e);
            }
        }
    }
}

#[test]
fn test_mov_format_detection() {
    // Test that MOV files are recognized
    if Path::new("exiftool/t/images/MOV.mov").exists() {
        let result = find_metadata_segment("exiftool/t/images/MOV.mov");

        // MOV files often have metadata
        match result {
            Ok(_) => {
                // Either Some or None is valid
            }
            Err(e) => {
                panic!("Error reading MOV file: {}", e);
            }
        }
    }
}

#[test]
fn test_3gp_format_detection() {
    // Test that 3GP files are recognized
    if Path::new("exiftool/t/images/3GP.3gp").exists() {
        let result = find_metadata_segment("exiftool/t/images/3GP.3gp");

        // 3GP files use QuickTime container
        match result {
            Ok(_) => {
                // Either Some or None is valid
            }
            Err(e) => {
                panic!("Error reading 3GP file: {}", e);
            }
        }
    }
}
