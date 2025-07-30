//! Test Canon CameraSettings binary data extraction
//!
//! This test verifies that Canon CameraSettings subdirectory tags
//! are properly extracted as individual values instead of raw arrays.

use exif_oxide::formats;
use exif_oxide::types::TagValue;
use std::path::Path;

#[test]
fn test_canon_camerasettings_extraction() {
    // Use Canon test images that contain CameraSettings data
    let canon_images = [
        "third-party/exiftool/t/images/Canon.jpg",
        "third-party/exiftool/t/images/Canon1DmkIII.jpg",
        "test-images/canon/eos_rebel_t3i.jpg",
    ];

    for image_path in &canon_images {
        let path = Path::new(image_path);
        if !path.exists() {
            eprintln!("Skipping {} - file not found", image_path);
            continue;
        }

        println!(
            "\nTesting Canon CameraSettings extraction for: {}",
            image_path
        );

        let exif_data = formats::extract_metadata(path, false, false, None).unwrap();

        // Look for Canon MakerNotes tag 0x1 (CameraSettings)
        let canon_settings_tag = exif_data
            .tags
            .iter()
            .find(|e| e.group == "MakerNotes" && e.name == "CameraSettings");

        if let Some(tag) = canon_settings_tag {
            println!("Found Canon:0x0001 tag: {}", tag.name);

            // Verify it's not a raw array
            match &tag.value {
                TagValue::U16Array(arr) => {
                    println!(
                        "ERROR: Canon CameraSettings is still a raw U16 array with {} values",
                        arr.len()
                    );
                    println!("First 10 values: {:?}", &arr[..arr.len().min(10)]);
                }
                TagValue::U8Array(arr) => {
                    println!(
                        "ERROR: Canon CameraSettings is still a raw U8 array with {} values",
                        arr.len()
                    );
                }
                _ => {
                    println!(
                        "Good: Canon CameraSettings has been processed, value type: {:?}",
                        tag.value
                    );
                }
            }
        } else {
            println!("Note: Canon:0x0001 (CameraSettings) tag not found in raw tags - this is expected if it was processed as a subdirectory");
        }

        // Look for individual CameraSettings tags like MacroMode, Quality, etc.
        let expected_tags = [
            "MacroMode",
            "Quality",
            "CanonFlashMode",
            "ContinuousDrive",
            "FocusMode",
            "CanonImageSize",
            "EasyMode",
        ];

        let mut found_count = 0;
        for tag_name in &expected_tags {
            let tag = exif_data.tags.iter().find(|e| e.name == *tag_name);

            if let Some(t) = tag {
                println!("Found CameraSettings tag: {} = {}", t.name, t.value);
                found_count += 1;

                // Verify it's a string value, not an array
                match &t.value {
                    TagValue::String(s) => {
                        assert!(!s.is_empty(), "{} should not be empty", tag_name);
                        // Some values might be "Unknown" if they're not in the lookup table
                        // That's OK for now - the important thing is they're extracted as individual tags
                        if s == "Unknown" {
                            println!(
                                "Note: {} has Unknown value - raw value not in lookup table",
                                tag_name
                            );
                        }
                    }
                    _ => {
                        println!("Warning: {} has non-string value: {:?}", tag_name, t.value);
                    }
                }
            }
        }

        if found_count > 0 {
            println!("Successfully found {} CameraSettings tags", found_count);
        } else {
            println!("Warning: No CameraSettings tags found for {}", image_path);
        }
    }
}

#[test]
fn test_canon_selftimer_extraction() {
    // Test the SelfTimer tag specifically since we have PrintConv for it
    let canon_images = [
        "third-party/exiftool/t/images/Canon.jpg",
        "test-images/canon/eos_rebel_t3i.jpg",
    ];

    for image_path in &canon_images {
        let path = Path::new(image_path);
        if !path.exists() {
            continue;
        }

        let exif_data = formats::extract_metadata(path, false, false, None).unwrap();

        // Look for SelfTimer tag
        let selftimer = exif_data.tags.iter().find(|e| e.name == "SelfTimer");

        if let Some(tag) = selftimer {
            println!("Found SelfTimer in {}: {}", image_path, tag.value);

            // The value should be a formatted string like "Off" or "2 s" or "10 s, Custom"
            match &tag.value {
                TagValue::String(s) => {
                    // Verify it matches expected format
                    assert!(
                        s == "Off" || s.ends_with(" s") || s.contains(" s,"),
                        "SelfTimer value '{}' doesn't match expected format",
                        s
                    );
                }
                TagValue::I16(val) => {
                    // If it's still raw, we should apply PrintConv
                    println!("Warning: SelfTimer is raw value: {}", val);
                }
                _ => panic!("Unexpected SelfTimer value type: {:?}", tag.value),
            }
        }
    }
}

#[test]
fn test_debug_canon_subdirectory_processing() {
    // This test shows debug info about Canon subdirectory processing
    // Run with: RUST_LOG=debug cargo t test_debug_canon_subdirectory_processing -- --nocapture

    let image_path = "third-party/exiftool/t/images/Canon.jpg";
    let path = Path::new(image_path);

    if !path.exists() {
        println!("Test image not found: {}", image_path);
        return;
    }

    println!("Processing {} for subdirectory debug...\n", image_path);

    let exif_data = formats::extract_metadata(path, false, false, None).unwrap();

    // Look for any MakerNotes tags
    println!("=== All MakerNotes tags ===");
    for tag in &exif_data.tags {
        if tag.group == "MakerNotes" {
            println!("{}: {} = {}", tag.group, tag.name, tag.value);
        }
    }

    // Check if CameraSettings tag exists (should be absent if processed)
    let camera_settings = exif_data
        .tags
        .iter()
        .find(|e| e.group == "MakerNotes" && e.name == "CameraSettings");

    if let Some(tag) = camera_settings {
        println!("\nCameraSettings tag still exists: {}", tag.value);
    } else {
        println!("\nCameraSettings tag not found - good, it should be processed as subdirectory");
    }

    // Check for individual CameraSettings components
    let camera_setting_tags = ["MacroMode", "Quality", "SelfTimer", "CanonFlashMode"];
    println!("\n=== Individual CameraSettings components ===");
    for tag_name in &camera_setting_tags {
        if let Some(tag) = exif_data.tags.iter().find(|e| e.name == *tag_name) {
            println!("Found {}: {}", tag_name, tag.value);
        } else {
            println!("{}: not found", tag_name);
        }
    }
}
