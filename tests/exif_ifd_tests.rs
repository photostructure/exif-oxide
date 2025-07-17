//! ExifIFD Group Assignment and Context-Aware Processing Tests
//!
//! This module tests the ExifIFD milestone requirements:
//! 1. Correct group1 assignment for ExifIFD tags vs main IFD tags
//! 2. Group-based API access methods
//! 3. Context-aware processing and validation
//!
//! ExifTool Reference: lib/Image/ExifTool/Exif.pm ExifIFD group assignment
//! Milestone: docs/milestones/MILESTONE-ExifIFD.md
//!
//! Note: These tests require the `integration-tests` feature to be enabled and
//! external test assets to be available. They are automatically skipped in published crates.

#![cfg(feature = "integration-tests")]

use exif_oxide::formats::extract_metadata;
use std::collections::HashMap;

/// Test that ExifIFD tags get correct group1 assignment
/// This is the core requirement from the milestone specification
#[test]
fn test_exif_ifd_group_assignment() {
    let exif_data = extract_metadata(
        std::path::Path::new("test-images/canon/Canon_T3i.JPG"),
        false,
    )
    .unwrap();

    // Find tags that should be in ExifIFD based on their tag IDs
    // Common ExifIFD tags from Canon T3i image
    let exif_ifd_tag_names = [
        "ExposureTime",
        "FNumber",
        "ExposureProgram",
        "ISO",
        "ExifVersion",
        "DateTimeOriginal",
        "DateTimeDigitized",
        "ShutterSpeedValue",
        "ApertureValue",
        "Flash",
        "FocalLength",
        "MakerNote",
        "UserComment",
        "FlashpixVersion",
        "ColorSpace",
        "ExifImageWidth",
        "ExifImageHeight",
        "FocalPlaneXResolution",
        "FocalPlaneYResolution",
        "FocalPlaneResolutionUnit",
        "CustomRendered",
        "ExposureMode",
        "WhiteBalance",
        "SceneCaptureType",
    ];

    // Test that ExifIFD tags have correct group1 assignment
    let mut found_exif_ifd_tags = 0;
    for tag_name in &exif_ifd_tag_names {
        if let Some(tag) = exif_data.get_tag_by_name(tag_name) {
            // ✅ FIXED: ExifIFD tags now correctly have group1 = "ExifIFD"
            println!(
                "Tag {}: group={}, group1={}",
                tag_name, tag.group, tag.group1
            );

            // Verify correct group1 assignment
            assert_eq!(
                tag.group1, "ExifIFD",
                "Tag {} should have group1='ExifIFD' but has '{}'",
                tag_name, tag.group1
            );

            // All these tags should have Group0 = "EXIF" (this is correct)
            assert_eq!(
                tag.group, "EXIF",
                "Tag {} should have group='EXIF' but has '{}'",
                tag_name, tag.group
            );

            found_exif_ifd_tags += 1;
        }
    }

    // Verify we found multiple ExifIFD tags (Canon T3i has 20+ ExifIFD tags)
    assert!(
        found_exif_ifd_tags >= 10,
        "Should find at least 10 ExifIFD tags, found {found_exif_ifd_tags}"
    );
}

/// Test that main IFD tags maintain correct group1 assignment
#[test]
fn test_main_ifd_group_assignment() {
    let exif_data = extract_metadata(
        std::path::Path::new("test-images/canon/Canon_T3i.JPG"),
        false,
    )
    .unwrap();

    // Tags that should be in main IFD (IFD0)
    let main_ifd_tag_names = [
        "Make",
        "Model",
        "Orientation",
        "XResolution",
        "YResolution",
        "ResolutionUnit",
        "Software",
        "DateTime",
        "YCbCrPositioning",
    ];

    let mut found_main_ifd_tags = 0;
    for tag_name in &main_ifd_tag_names {
        if let Some(tag) = exif_data.get_tag_by_name(tag_name) {
            // Main IFD tags should have group1 = "IFD0"
            assert_eq!(
                tag.group1, "IFD0",
                "Main IFD tag {} should have group1='IFD0' but has '{}'",
                tag_name, tag.group1
            );

            // Group0 should be "EXIF" for main EXIF tags
            assert_eq!(
                tag.group, "EXIF",
                "Main IFD tag {} should have group='EXIF' but has '{}'",
                tag_name, tag.group
            );

            found_main_ifd_tags += 1;
        }
    }

    // Verify we found multiple main IFD tags
    assert!(
        found_main_ifd_tags >= 5,
        "Should find at least 5 main IFD tags, found {found_main_ifd_tags}"
    );
}

/// Test GPS tags get correct group1 assignment
#[test]
fn test_gps_group_assignment() {
    // Canon T3i may not have GPS data, so use a GPS-enabled image or skip
    let exif_data = match extract_metadata(
        std::path::Path::new("test-images/canon/Canon_T3i.JPG"),
        false,
    ) {
        Ok(data) => data,
        Err(_) => return, // Skip if file not available
    };

    // Look for any GPS tags
    let gps_tags: Vec<_> = exif_data
        .tags
        .iter()
        .filter(|tag| tag.name.starts_with("GPS"))
        .collect();

    // If GPS tags exist, verify group assignment
    for tag in gps_tags {
        assert_eq!(
            tag.group1, "GPS",
            "GPS tag {} should have group1='GPS' but has '{}'",
            tag.name, tag.group1
        );

        // GPS tags have Group0 = "EXIF" in ExifTool
        assert_eq!(
            tag.group, "EXIF",
            "GPS tag {} should have group='EXIF' but has '{}'",
            tag.name, tag.group
        );
    }
}

/// Test group-based API access methods work correctly
#[test]
fn test_group_based_api_access() {
    let exif_data = extract_metadata(
        std::path::Path::new("test-images/canon/Canon_T3i.JPG"),
        false,
    )
    .unwrap();

    // Test get_exif_ifd_tags() method
    let exif_ifd_tags = exif_data.get_exif_ifd_tags();

    // Should now find ExifIFD tags since the bug is fixed
    assert!(!exif_ifd_tags.is_empty(), "Should find ExifIFD tags");

    for tag in &exif_ifd_tags {
        assert_eq!(
            tag.group1, "ExifIFD",
            "get_exif_ifd_tags() returned tag with wrong group1: {}",
            tag.group1
        );
    }

    // Test get_tags_by_group1() method
    let ifd0_tags = exif_data.get_tags_by_group1("IFD0");
    assert!(!ifd0_tags.is_empty(), "Should find IFD0 tags");

    for tag in &ifd0_tags {
        assert_eq!(
            tag.group1, "IFD0",
            "get_tags_by_group1('IFD0') returned tag with wrong group1: {}",
            tag.group1
        );
    }

    // Test get_tag_by_group() method
    if let Some(make_tag) = exif_data.get_tag_by_group("EXIF", "Make") {
        assert_eq!(make_tag.name, "Make");
        assert_eq!(make_tag.group, "EXIF");
    }

    // Test ExifTool-style qualified access
    if let Some(make_tag) = exif_data.get_tag_exiftool_style("EXIF:Make") {
        assert_eq!(make_tag.name, "Make");
    }

    // Test ExifIFD-qualified access now that bug is fixed
    if let Some(exposure_tag) = exif_data.get_tag_exiftool_style("ExifIFD:ExposureTime") {
        assert_eq!(exposure_tag.name, "ExposureTime");
        assert_eq!(exposure_tag.group1, "ExifIFD");
    }
}

/// Test that ExifIFD and main IFD tags are properly distinguished
#[test]
fn test_exif_ifd_vs_main_ifd_distinction() {
    let exif_data = extract_metadata(
        std::path::Path::new("test-images/canon/Canon_T3i.JPG"),
        false,
    )
    .unwrap();

    // Group tags by their group1 assignment
    let mut tags_by_group1: HashMap<String, Vec<&str>> = HashMap::new();

    for tag in &exif_data.tags {
        tags_by_group1
            .entry(tag.group1.clone())
            .or_default()
            .push(&tag.name);
    }

    // Print current group1 distribution for debugging
    println!("Current group1 distribution:");
    for (group1, tag_names) in &tags_by_group1 {
        println!(
            "  {}: {} tags - {:?}",
            group1,
            tag_names.len(),
            &tag_names[..tag_names.len().min(5)]
        );
    }

    // Verify we have multiple group1 categories
    assert!(
        tags_by_group1.len() >= 2,
        "Should have multiple group1 categories, found: {:?}",
        tags_by_group1.keys().collect::<Vec<_>>()
    );

    // Should have IFD0 tags (main IFD)
    assert!(tags_by_group1.contains_key("IFD0"), "Should have IFD0 tags");

    // Should also have ExifIFD tags now that bug is fixed
    assert!(
        tags_by_group1.contains_key("ExifIFD"),
        "Should have ExifIFD tags"
    );

    // Verify specific tags are in the right groups
    let empty_vec = vec![];
    let ifd0_tags = tags_by_group1.get("IFD0").unwrap_or(&empty_vec);
    assert!(ifd0_tags.contains(&"Make"), "Make should be in IFD0");
    assert!(ifd0_tags.contains(&"Model"), "Model should be in IFD0");
}

/// Test that the namespace assignment bug has been fixed
/// This test verifies that ExifIFD tags now have correct group1 assignment
#[test]
fn test_namespace_assignment_bug_fixed() {
    let exif_data = extract_metadata(
        std::path::Path::new("test-images/canon/Canon_T3i.JPG"),
        false,
    )
    .unwrap();

    // Find a tag that should be in ExifIFD (like ExposureTime)
    if let Some(exposure_time) = exif_data.get_tag_by_name("ExposureTime") {
        // ✅ FIXED: ExifIFD tags now correctly have group1="ExifIFD"
        println!(
            "ExposureTime: group={}, group1={}",
            exposure_time.group, exposure_time.group1
        );

        // Verify the fix: ExifIFD tags should have group1="ExifIFD"
        assert_eq!(
            exposure_time.group1, "ExifIFD",
            "FIXED: ExposureTime should have group1='ExifIFD', got '{}'",
            exposure_time.group1
        );

        // Also verify correct group0 assignment
        assert_eq!(
            exposure_time.group, "EXIF",
            "ExposureTime should have group='EXIF', got '{}'",
            exposure_time.group
        );
    } else {
        panic!("Canon T3i image should have ExposureTime tag");
    }
}

/// Test ExifIFD-specific tags are present and accessible
#[test]
fn test_exif_ifd_specific_tags() {
    let exif_data = extract_metadata(
        std::path::Path::new("test-images/canon/Canon_T3i.JPG"),
        false,
    )
    .unwrap();

    // Tags that only exist in ExifIFD subdirectory
    let exif_ifd_specific_tags = [
        "ExifVersion",     // 0x9000 - Mandatory in ExifIFD
        "FlashpixVersion", // 0xA000 - ExifIFD specific
        "ColorSpace",      // 0xA001 - ExifIFD specific
        "ExifImageWidth",  // 0xA002 - ExifIFD specific
        "ExifImageHeight", // 0xA003 - ExifIFD specific
    ];

    let mut found_specific_tags = 0;
    for tag_name in &exif_ifd_specific_tags {
        if let Some(tag) = exif_data.get_tag_by_name(tag_name) {
            // These tags should only exist in ExifIFD context
            println!(
                "ExifIFD-specific tag {}: group={}, group1={}",
                tag_name, tag.group, tag.group1
            );

            // Verify correct group1 assignment now that bug is fixed
            assert_eq!(
                tag.group1, "ExifIFD",
                "ExifIFD-specific tag {tag_name} should have group1='ExifIFD'"
            );

            found_specific_tags += 1;
        }
    }

    assert!(
        found_specific_tags >= 3,
        "Should find at least 3 ExifIFD-specific tags, found {found_specific_tags}"
    );
}
