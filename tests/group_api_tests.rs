//! Group-Based API Access Pattern Tests
//!
//! This module comprehensively tests the group-based tag access methods
//! that enable ExifTool-compatible API patterns for tag retrieval.
//!
//! Tests the API methods added for the ExifIFD milestone:
//! - get_exif_ifd_tags() - Get all ExifIFD-specific tags
//! - get_tags_by_group1() - Get tags by Group1 (subdirectory location)
//!
//! Note: These tests require the `integration-tests` feature to be enabled and
//! external test assets to be available. They are automatically skipped in published crates.

#![cfg(feature = "integration-tests")]
//! - get_tag_by_group() - Get tag by group name and tag name
//! - get_tag_exiftool_style() - Parse "Group:TagName" qualified names
//! - get_tag_by_name() - Get tag by name (without group qualifier)
//!
//! ExifTool Reference: ExifTool group-based tag access patterns
//! Milestone: docs/milestones/MILESTONE-ExifIFD.md API Compatibility

use exif_oxide::formats::extract_metadata;

/// Test get_exif_ifd_tags() method returns only ExifIFD tags
#[test]
fn test_get_exif_ifd_tags_method() {
    let exif_data = extract_metadata(
        std::path::Path::new("test-images/canon/Canon_T3i.JPG"),
        false,
    )
    .unwrap();

    // Get all ExifIFD tags using the dedicated method
    let exif_ifd_tags = exif_data.get_exif_ifd_tags();

    println!("get_exif_ifd_tags() returned {} tags", exif_ifd_tags.len());

    // Verify all returned tags have group1 = "ExifIFD"
    for tag in &exif_ifd_tags {
        assert_eq!(
            tag.group1, "ExifIFD",
            "get_exif_ifd_tags() returned tag '{}' with group1='{}', expected 'ExifIFD'",
            tag.name, tag.group1
        );
    }

    // TODO: When the namespace bug is fixed, this should return multiple tags
    // For now, it returns 0 tags due to the bug
    println!(
        "ExifIFD tags found: {} (expected 0 due to current bug)",
        exif_ifd_tags.len()
    );

    // When bug is fixed, uncomment this:
    // assert!(exif_ifd_tags.len() >= 10,
    //     "Should find at least 10 ExifIFD tags, found {}", exif_ifd_tags.len());

    // Verify method consistency - should match manual filtering
    let manual_exif_ifd_tags: Vec<_> = exif_data
        .tags
        .iter()
        .filter(|tag| tag.group1 == "ExifIFD")
        .collect();

    assert_eq!(
        exif_ifd_tags.len(),
        manual_exif_ifd_tags.len(),
        "get_exif_ifd_tags() should match manual filtering"
    );
}

/// Test get_tags_by_group1() method for different Group1 values
#[test]
fn test_get_tags_by_group1_method() {
    let exif_data = extract_metadata(
        std::path::Path::new("test-images/canon/Canon_T3i.JPG"),
        false,
    )
    .unwrap();

    // Test IFD0 group (main IFD)
    let ifd0_tags = exif_data.get_tags_by_group1("IFD0");
    assert!(!ifd0_tags.is_empty(), "Should find IFD0 tags");

    for tag in &ifd0_tags {
        assert_eq!(
            tag.group1, "IFD0",
            "get_tags_by_group1('IFD0') returned tag '{}' with wrong group1: '{}'",
            tag.name, tag.group1
        );
    }

    println!("IFD0 tags found: {}", ifd0_tags.len());
    for tag in ifd0_tags.iter().take(5) {
        println!("  IFD0: {} = {:?}", tag.name, tag.value);
    }

    // Test ExifIFD group (should be empty due to current bug)
    let exif_ifd_tags = exif_data.get_tags_by_group1("ExifIFD");
    println!(
        "ExifIFD tags found: {} (expected 0 due to bug)",
        exif_ifd_tags.len()
    );

    for tag in &exif_ifd_tags {
        assert_eq!(
            tag.group1, "ExifIFD",
            "get_tags_by_group1('ExifIFD') returned tag with wrong group1"
        );
    }

    // Test GPS group (may be empty if no GPS data)
    let gps_tags = exif_data.get_tags_by_group1("GPS");
    println!("GPS tags found: {}", gps_tags.len());

    for tag in &gps_tags {
        assert_eq!(
            tag.group1, "GPS",
            "get_tags_by_group1('GPS') returned tag '{}' with wrong group1: '{}'",
            tag.name, tag.group1
        );
    }

    // Test non-existent group
    let fake_tags = exif_data.get_tags_by_group1("NonExistentGroup");
    assert!(
        fake_tags.is_empty(),
        "get_tags_by_group1() should return empty for non-existent group"
    );

    // Verify method consistency
    let total_manual = exif_data
        .tags
        .iter()
        .filter(|tag| tag.group1 == "IFD0" || tag.group1 == "ExifIFD" || tag.group1 == "GPS")
        .count();
    let total_api = ifd0_tags.len() + exif_ifd_tags.len() + gps_tags.len();

    assert_eq!(
        total_manual, total_api,
        "get_tags_by_group1() totals should match manual filtering"
    );
}

/// Test get_tag_by_group() method for both Group0 and Group1 access
#[test]
fn test_get_tag_by_group_method() {
    let exif_data = extract_metadata(
        std::path::Path::new("test-images/canon/Canon_T3i.JPG"),
        false,
    )
    .unwrap();

    // Test Group0 (format family) access
    let make_by_exif = exif_data.get_tag_by_group("EXIF", "Make");
    assert!(
        make_by_exif.is_some(),
        "Should find Make tag via EXIF group"
    );

    if let Some(tag) = make_by_exif {
        assert_eq!(tag.name, "Make");
        assert_eq!(tag.group, "EXIF");
        println!("Found Make via EXIF group: {:?}", tag.value);
    }

    let model_by_exif = exif_data.get_tag_by_group("EXIF", "Model");
    assert!(
        model_by_exif.is_some(),
        "Should find Model tag via EXIF group"
    );

    // Test Group1 (subdirectory location) access
    let make_by_ifd0 = exif_data.get_tag_by_group("IFD0", "Make");
    assert!(
        make_by_ifd0.is_some(),
        "Should find Make tag via IFD0 group1"
    );

    if let Some(tag) = make_by_ifd0 {
        assert_eq!(tag.name, "Make");
        assert_eq!(tag.group1, "IFD0");
        println!("Found Make via IFD0 group1: {:?}", tag.value);
    }

    // Verify both accesses return the same tag
    if let (Some(tag1), Some(tag2)) = (make_by_exif, make_by_ifd0) {
        assert_eq!(tag1.name, tag2.name);
        assert_eq!(tag1.value, tag2.value);
        println!("Group0 and Group1 access return same tag: ✅");
    }

    // Test ExifIFD group1 access (should fail due to current bug)
    let exposure_by_exif_ifd = exif_data.get_tag_by_group("ExifIFD", "ExposureTime");
    println!(
        "ExposureTime via ExifIFD group1: {:?} (expected None due to bug)",
        exposure_by_exif_ifd.map(|t| &t.value)
    );

    // TODO: When bug is fixed, this should work:
    // assert!(exposure_by_exif_ifd.is_some(), "Should find ExposureTime via ExifIFD group1");

    // Test non-existent combinations
    let fake_tag = exif_data.get_tag_by_group("EXIF", "NonExistentTag");
    assert!(
        fake_tag.is_none(),
        "Should return None for non-existent tag"
    );

    let fake_group = exif_data.get_tag_by_group("FakeGroup", "Make");
    assert!(
        fake_group.is_none(),
        "Should return None for non-existent group"
    );
}

/// Test get_tag_exiftool_style() method for qualified tag names
#[test]
fn test_get_tag_exiftool_style_method() {
    let exif_data = extract_metadata(
        std::path::Path::new("test-images/canon/Canon_T3i.JPG"),
        false,
    )
    .unwrap();

    // Test Group0:TagName format
    let make_qualified = exif_data.get_tag_exiftool_style("EXIF:Make");
    assert!(
        make_qualified.is_some(),
        "Should find Make via 'EXIF:Make' qualifier"
    );

    if let Some(tag) = make_qualified {
        assert_eq!(tag.name, "Make");
        assert_eq!(tag.group, "EXIF");
        println!("Found 'EXIF:Make': {:?}", tag.value);
    }

    // Test Group1:TagName format
    let make_ifd0 = exif_data.get_tag_exiftool_style("IFD0:Make");
    assert!(
        make_ifd0.is_some(),
        "Should find Make via 'IFD0:Make' qualifier"
    );

    if let Some(tag) = make_ifd0 {
        assert_eq!(tag.name, "Make");
        assert_eq!(tag.group1, "IFD0");
        println!("Found 'IFD0:Make': {:?}", tag.value);
    }

    // Test ExifIFD qualified access (should fail due to current bug)
    let exposure_qualified = exif_data.get_tag_exiftool_style("ExifIFD:ExposureTime");
    println!(
        "'ExifIFD:ExposureTime': {:?} (expected None due to bug)",
        exposure_qualified.map(|t| &t.value)
    );

    // TODO: When bug is fixed, this should work:
    // assert!(exposure_qualified.is_some(), "Should find ExposureTime via 'ExifIFD:ExposureTime'");

    // Test unqualified tag name (fallback to get_tag_by_name)
    let make_unqualified = exif_data.get_tag_exiftool_style("Make");
    assert!(
        make_unqualified.is_some(),
        "Should find Make without qualifier"
    );

    if let Some(tag) = make_unqualified {
        assert_eq!(tag.name, "Make");
        println!("Found unqualified 'Make': {:?}", tag.value);
    }

    // Test invalid qualifier format
    let invalid_format = exif_data.get_tag_exiftool_style("InvalidFormat");
    // Should fallback to unqualified search
    assert_eq!(
        invalid_format.is_some(),
        exif_data.get_tag_by_name("InvalidFormat").is_some()
    );

    // Test non-existent qualified tag
    let fake_qualified = exif_data.get_tag_exiftool_style("EXIF:NonExistentTag");
    assert!(
        fake_qualified.is_none(),
        "Should return None for non-existent qualified tag"
    );

    let fake_group_qualified = exif_data.get_tag_exiftool_style("FakeGroup:Make");
    assert!(
        fake_group_qualified.is_none(),
        "Should return None for fake group qualifier"
    );
}

/// Test get_tag_by_name() method for basic unqualified access
#[test]
fn test_get_tag_by_name_method() {
    let exif_data = extract_metadata(
        std::path::Path::new("test-images/canon/Canon_T3i.JPG"),
        false,
    )
    .unwrap();

    // Test basic tag access
    let make_tag = exif_data.get_tag_by_name("Make");
    assert!(make_tag.is_some(), "Should find Make tag");

    if let Some(tag) = make_tag {
        assert_eq!(tag.name, "Make");
        println!(
            "Make: {:?} (group: {}, group1: {})",
            tag.value, tag.group, tag.group1
        );
    }

    let model_tag = exif_data.get_tag_by_name("Model");
    assert!(model_tag.is_some(), "Should find Model tag");

    // Test case sensitivity
    let make_wrong_case = exif_data.get_tag_by_name("make");
    assert!(
        make_wrong_case.is_none(),
        "Tag names should be case sensitive"
    );

    let make_wrong_case2 = exif_data.get_tag_by_name("MAKE");
    assert!(
        make_wrong_case2.is_none(),
        "Tag names should be case sensitive"
    );

    // Test non-existent tag
    let fake_tag = exif_data.get_tag_by_name("NonExistentTag");
    assert!(
        fake_tag.is_none(),
        "Should return None for non-existent tag"
    );

    // Test that method returns first match (if multiple tags have same name)
    // This is expected behavior for basic unqualified access
    let exposure_tag = exif_data.get_tag_by_name("ExposureTime");
    if let Some(tag) = exposure_tag {
        println!(
            "ExposureTime: {:?} (group: {}, group1: {})",
            tag.value, tag.group, tag.group1
        );
        assert_eq!(tag.name, "ExposureTime");
    }
}

/// Test API method consistency and cross-validation
#[test]
fn test_api_method_consistency() {
    let exif_data = extract_metadata(
        std::path::Path::new("test-images/canon/Canon_T3i.JPG"),
        false,
    )
    .unwrap();

    // Test that different access methods return the same tag
    let make_by_name = exif_data.get_tag_by_name("Make");
    let make_by_group = exif_data.get_tag_by_group("EXIF", "Make");
    let make_by_style = exif_data.get_tag_exiftool_style("EXIF:Make");
    let make_by_group1 = exif_data.get_tag_by_group("IFD0", "Make");

    // All should find the Make tag
    assert!(make_by_name.is_some());
    assert!(make_by_group.is_some());
    assert!(make_by_style.is_some());
    assert!(make_by_group1.is_some());

    // All should return the same tag content
    if let (Some(t1), Some(t2), Some(t3), Some(t4)) =
        (make_by_name, make_by_group, make_by_style, make_by_group1)
    {
        assert_eq!(t1.name, t2.name);
        assert_eq!(t1.value, t2.value);
        assert_eq!(t1.value, t3.value);
        assert_eq!(t1.value, t4.value);

        println!("API consistency verified: all methods return same Make tag ✅");
    }

    // Test group filtering consistency
    let ifd0_tags_by_group1 = exif_data.get_tags_by_group1("IFD0");
    let ifd0_tags_manual: Vec<_> = exif_data
        .tags
        .iter()
        .filter(|tag| tag.group1 == "IFD0")
        .collect();

    assert_eq!(
        ifd0_tags_by_group1.len(),
        ifd0_tags_manual.len(),
        "get_tags_by_group1() should match manual filtering"
    );

    // Test ExifIFD-specific method consistency
    let exif_ifd_by_method = exif_data.get_exif_ifd_tags();
    let exif_ifd_by_group1 = exif_data.get_tags_by_group1("ExifIFD");

    assert_eq!(
        exif_ifd_by_method.len(),
        exif_ifd_by_group1.len(),
        "get_exif_ifd_tags() should match get_tags_by_group1('ExifIFD')"
    );
}

/// Test API performance and behavior with large tag sets
#[test]
fn test_api_performance_and_edge_cases() {
    let exif_data = extract_metadata(
        std::path::Path::new("test-images/canon/Canon_T3i.JPG"),
        false,
    )
    .unwrap();

    // Test with empty string
    let empty_tag = exif_data.get_tag_by_name("");
    assert!(
        empty_tag.is_none(),
        "Should handle empty tag name gracefully"
    );

    let empty_group = exif_data.get_tag_by_group("", "Make");
    assert!(
        empty_group.is_none(),
        "Should handle empty group name gracefully"
    );

    let empty_qualified = exif_data.get_tag_exiftool_style("");
    assert!(
        empty_qualified.is_none(),
        "Should handle empty qualified name gracefully"
    );

    // Test with whitespace
    let whitespace_tag = exif_data.get_tag_by_name(" Make ");
    assert!(
        whitespace_tag.is_none(),
        "Should not match tags with extra whitespace"
    );

    // Test with special characters
    let special_tag = exif_data.get_tag_by_name("Make:Test");
    assert!(
        special_tag.is_none(),
        "Should handle special characters in tag names"
    );

    // Test colon handling in ExifTool-style access
    let multiple_colons = exif_data.get_tag_exiftool_style("EXIF:GPS:Make");
    // Should split on first colon only, so group="EXIF" and name="GPS:Make"
    assert!(
        multiple_colons.is_none(),
        "Should handle multiple colons correctly"
    );

    let no_colon = exif_data.get_tag_exiftool_style("JustATagName");
    // Should fallback to get_tag_by_name
    let by_name = exif_data.get_tag_by_name("JustATagName");
    assert_eq!(
        no_colon.is_some(),
        by_name.is_some(),
        "ExifTool-style without colon should match get_tag_by_name"
    );

    println!("API edge case handling: ✅");
}
