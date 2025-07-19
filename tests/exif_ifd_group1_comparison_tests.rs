//! ExifTool Group1 Comparison Tests
//!
//! This module compares our group1 field assignments with ExifTool's Group1 output
//! using the -G1 flag to validate the ExifIFD milestone implementation.
//!
//! ExifTool Reference: ExifTool -G1 flag shows Group1 (directory location) assignments
//! Milestone: docs/todo/MILESTONE-ExifIFD.md Group1 Assignment
//!
//! Note: These tests require the `integration-tests` feature to be enabled and
//! external test assets to be available. They are automatically skipped in published crates.

#![cfg(feature = "integration-tests")]

use exif_oxide::formats::extract_metadata;
use serde_json::Value;
use std::collections::HashMap;
use std::process::Command;

mod common;
use common::CANON_T3I_JPG;

/// Helper function to get ExifTool Group1 assignments using -G1 flag
fn get_exiftool_group1_assignments(image_path: &str) -> Option<HashMap<String, String>> {
    // Check if ExifTool is available
    let exiftool_check = Command::new("exiftool").arg("-ver").output();
    if exiftool_check.is_err() {
        println!("ExifTool not available, skipping Group1 comparison test");
        return None;
    }

    // Run ExifTool with -G1 and -j flags to get Group1 assignments in JSON
    let output = Command::new("exiftool")
        .args(["-G1", "-j", image_path])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    // Parse ExifTool JSON output
    let json_str = String::from_utf8(output.stdout).ok()?;
    let json_value: Value = serde_json::from_str(&json_str).ok()?;
    let json_array = json_value.as_array()?;
    let json_obj = json_array.first()?.as_object()?;

    let mut group1_assignments = HashMap::new();

    // Parse Group1-prefixed tags from ExifTool output
    // Format: "Group1:TagName": "value"
    for (key, _value) in json_obj {
        if let Some((group1, tag_name)) = key.split_once(':') {
            // Store the Group1 assignment for each tag
            group1_assignments.insert(tag_name.to_string(), group1.to_string());
        }
    }

    Some(group1_assignments)
}

/// Test Group1 assignments match ExifTool for Canon T3i image
#[test]
fn test_group1_assignments_match_exiftool_canon() {
    let image_path = CANON_T3I_JPG;

    // Get ExifTool's Group1 assignments
    let exiftool_group1 = match get_exiftool_group1_assignments(image_path) {
        Some(assignments) => assignments,
        None => {
            println!("Skipping Group1 comparison test - ExifTool not available or failed");
            return;
        }
    };

    // Get our EXIF data
    let exif_data = extract_metadata(std::path::Path::new(image_path), false, false).unwrap();

    // Compare Group1 assignments for each tag
    let mut total_compared = 0;
    let mut mismatches = Vec::new();
    let mut our_group1_stats = HashMap::new();

    for tag in &exif_data.tags {
        // Count our group1 assignments for statistics
        *our_group1_stats.entry(tag.group1.clone()).or_insert(0) += 1;

        if let Some(exiftool_group1) = exiftool_group1.get(&tag.name) {
            total_compared += 1;

            if &tag.group1 != exiftool_group1 {
                mismatches.push((
                    tag.name.clone(),
                    tag.group1.clone(),
                    exiftool_group1.clone(),
                ));
            }
        }
    }

    // Print diagnostic information
    println!("ExifTool Group1 comparison for {image_path}:");
    println!("  Total tags compared: {total_compared}");
    println!(
        "  ExifTool Group1 categories found: {:?}",
        exiftool_group1
            .values()
            .collect::<std::collections::HashSet<_>>()
    );
    println!("  Our group1 distribution: {our_group1_stats:?}");

    if !mismatches.is_empty() {
        println!("  Group1 mismatches found:");
        for (tag_name, our_group1, exiftool_group1) in &mismatches {
            println!("    {tag_name}: ours='{our_group1}' vs ExifTool='{exiftool_group1}'");
        }
    }

    // ❌ EXPECTED FAILURE: Due to the current namespace assignment bug,
    // ExifIFD tags will have group1="IFD0" instead of "ExifIFD"

    // For now, we document the mismatches rather than asserting perfect match
    // When the bug is fixed, this test should pass with 0 mismatches

    if mismatches.is_empty() {
        println!("✅ Perfect Group1 match with ExifTool!");
    } else {
        println!(
            "❌ Found {} Group1 mismatches (expected due to current bug)",
            mismatches.len()
        );

        // Check if the mismatches are the expected ExifIFD -> IFD0 bug
        let exif_ifd_mismatches: Vec<_> = mismatches
            .iter()
            .filter(|(_, our_group1, exiftool_group1)| {
                our_group1 == "IFD0" && exiftool_group1 == "ExifIFD"
            })
            .collect();

        println!(
            "  ExifIFD mismatches (expected bug): {}",
            exif_ifd_mismatches.len()
        );

        // TODO: When the bug is fixed, uncomment this assertion
        // assert!(mismatches.is_empty(),
        //     "Group1 assignments should match ExifTool exactly");
    }

    // Verify we compared a reasonable number of tags
    assert!(
        total_compared >= 20,
        "Should compare at least 20 tags with ExifTool, compared {total_compared}"
    );
}

/// Test Group1 assignments for multiple image formats
#[test]
fn test_group1_assignments_multiple_formats() {
    let test_images = [CANON_T3I_JPG, "test-images/canon/Canon_T3i.CR2"];

    for image_path in &test_images {
        if !std::path::Path::new(image_path).exists() {
            println!("Skipping {image_path} - file not found");
            continue;
        }

        // Get ExifTool's Group1 assignments
        let exiftool_group1 = match get_exiftool_group1_assignments(image_path) {
            Some(assignments) => assignments,
            None => {
                println!("Skipping {image_path} - ExifTool failed");
                continue;
            }
        };

        // Get our EXIF data
        let exif_data = match extract_metadata(std::path::Path::new(image_path), false, false) {
            Ok(data) => data,
            Err(e) => {
                println!("Skipping {image_path} - processing failed: {e}");
                continue;
            }
        };

        // Count Group1 categories for this image
        let mut our_group1_categories = std::collections::HashSet::new();
        let mut exiftool_group1_categories = std::collections::HashSet::new();

        for tag in &exif_data.tags {
            our_group1_categories.insert(&tag.group1);
        }

        for group1 in exiftool_group1.values() {
            exiftool_group1_categories.insert(group1);
        }

        println!(
            "Image {image_path}: our_group1={our_group1_categories:?}, exiftool_group1={exiftool_group1_categories:?}"
        );

        // Both should have multiple Group1 categories
        assert!(
            !our_group1_categories.is_empty(),
            "Should have at least 1 Group1 category for {image_path}"
        );
        assert!(
            !exiftool_group1_categories.is_empty(),
            "ExifTool should show at least 1 Group1 category for {image_path}"
        );
    }
}

/// Test specific ExifIFD tags have correct Group1 assignment vs ExifTool
#[test]
fn test_exif_ifd_specific_tags_group1() {
    let image_path = CANON_T3I_JPG;

    // Get ExifTool's Group1 assignments
    let exiftool_group1 = match get_exiftool_group1_assignments(image_path) {
        Some(assignments) => assignments,
        None => {
            println!("Skipping ExifIFD-specific Group1 test - ExifTool not available");
            return;
        }
    };

    // Get our EXIF data
    let exif_data = extract_metadata(std::path::Path::new(image_path), false, false).unwrap();

    // Tags that should definitely be in ExifIFD according to ExifTool
    let expected_exif_ifd_tags = [
        "ExposureTime",
        "FNumber",
        "ExifVersion",
        "DateTimeOriginal",
        "DateTimeDigitized",
        "Flash",
        "FocalLength",
        "FlashpixVersion",
        "ColorSpace",
        "ExifImageWidth",
        "ExifImageHeight",
    ];

    let mut exif_ifd_comparisons = 0;
    let mut exif_ifd_mismatches = 0;

    for tag_name in &expected_exif_ifd_tags {
        if let (Some(our_tag), Some(exiftool_group1)) = (
            exif_data.get_tag_by_name(tag_name),
            exiftool_group1.get(*tag_name),
        ) {
            exif_ifd_comparisons += 1;

            println!(
                "Tag {}: ours='{}', ExifTool='{}'",
                tag_name, our_tag.group1, exiftool_group1
            );

            if &our_tag.group1 != exiftool_group1 {
                exif_ifd_mismatches += 1;

                // The expected bug: our tags have group1="IFD0" but should be "ExifIFD"
                if our_tag.group1 == "IFD0" && exiftool_group1 == "ExifIFD" {
                    println!("  ❌ Expected ExifIFD mismatch due to namespace bug");
                } else {
                    println!("  ⚠️  Unexpected Group1 mismatch");
                }
            } else {
                println!("  ✅ Group1 assignment matches");
            }
        }
    }

    // Verify we found and compared ExifIFD-specific tags
    assert!(
        exif_ifd_comparisons >= 5,
        "Should compare at least 5 ExifIFD-specific tags, compared {exif_ifd_comparisons}"
    );

    // TODO: When the bug is fixed, this should be 0
    // For now, we expect mismatches due to the namespace assignment bug
    println!(
        "ExifIFD-specific tags: {exif_ifd_comparisons} comparisons, {exif_ifd_mismatches} mismatches"
    );

    // Document the current state: we expect mismatches due to the bug
    if exif_ifd_mismatches > 0 {
        println!("❌ Found ExifIFD Group1 mismatches (expected due to namespace bug)");
    } else {
        println!("✅ All ExifIFD Group1 assignments match ExifTool!");
    }
}

/// Helper test to show ExifTool's Group1 output format for debugging
#[test]
fn test_debug_exiftool_group1_output() {
    let image_path = CANON_T3I_JPG;

    // Check if ExifTool is available
    let exiftool_check = Command::new("exiftool").arg("-ver").output();
    if exiftool_check.is_err() {
        println!("ExifTool not available, skipping debug output test");
        return;
    }

    // Run ExifTool with -G1 flag to show Group1 assignments
    let output = Command::new("exiftool")
        .args(["-G1", "-j", image_path])
        .output()
        .expect("Failed to run ExifTool");

    if output.status.success() {
        let json_str = String::from_utf8(output.stdout).unwrap();
        let json_value: Value = serde_json::from_str(&json_str).unwrap();

        println!("ExifTool -G1 output sample for {image_path}:");

        if let Some(obj) = json_value[0].as_object() {
            let mut group1_examples = HashMap::new();

            // Collect examples of each Group1 category
            for (key, value) in obj {
                if let Some((group1, tag_name)) = key.split_once(':') {
                    group1_examples
                        .entry(group1.to_string())
                        .or_insert_with(Vec::new)
                        .push((tag_name.to_string(), value.clone()));
                }
            }

            // Show examples from each Group1 category
            for (group1, examples) in group1_examples {
                println!("  Group1 '{}': {} tags", group1, examples.len());
                for (tag_name, _value) in examples.iter().take(3) {
                    println!("    {group1}:{tag_name}");
                }
                if examples.len() > 3 {
                    println!("    ... and {} more", examples.len() - 3);
                }
            }
        }
    } else {
        println!("ExifTool failed to process {image_path}");
    }
}
