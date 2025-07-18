//! Enhanced ExifTool Group Compatibility Tests
//!
//! This module extends the existing ExifTool compatibility testing framework
//! to specifically validate Group1 vs Group0 assignments against ExifTool's
//! group assignment behavior using the -G, -G1, and -G2 flags.
//!
//! This complements the existing exiftool_compatibility_tests.rs by focusing
//! specifically on group assignment correctness rather than value accuracy.
//!
//! Note: These tests require the `integration-tests` feature to be enabled and
//! external test assets to be available. They are automatically skipped in published crates.

#![cfg(feature = "integration-tests")]
//!
//! ExifTool Reference: ExifTool -G/-G1/-G2 flags for group hierarchies
//! Milestone: docs/milestones/MILESTONE-ExifIFD.md Group Assignment Validation

use exif_oxide::formats::extract_metadata;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::process::Command;

mod common;
use common::CANON_T3I_JPG;

/// Helper to get ExifTool group assignments using various -G flags
#[derive(Debug)]
struct ExifToolGroupData {
    /// Group0 assignments from -G flag (format family)
    group0_assignments: HashMap<String, String>,
    /// Group1 assignments from -G1 flag (subdirectory location)  
    group1_assignments: HashMap<String, String>,
    /// Combined group assignments from -G2 flag (full hierarchy)
    group2_assignments: HashMap<String, String>,
}

impl ExifToolGroupData {
    /// Extract group assignments from ExifTool using multiple -G flags
    fn extract_from_exiftool(image_path: &str) -> Option<Self> {
        // Check if ExifTool is available
        let exiftool_check = Command::new("exiftool").arg("-ver").output();
        if exiftool_check.is_err() {
            println!("ExifTool not available, skipping group compatibility test");
            return None;
        }

        // Get Group0 assignments (-G flag)
        let group0_assignments = Self::get_group_assignments(image_path, "-G")?;

        // Get Group1 assignments (-G1 flag)
        let group1_assignments = Self::get_group_assignments(image_path, "-G1")?;

        // Get Group2 assignments (-G2 flag)
        let group2_assignments = Self::get_group_assignments(image_path, "-G2")?;

        Some(ExifToolGroupData {
            group0_assignments,
            group1_assignments,
            group2_assignments,
        })
    }

    /// Helper to run ExifTool with specific group flag and parse assignments
    fn get_group_assignments(
        image_path: &str,
        group_flag: &str,
    ) -> Option<HashMap<String, String>> {
        let output = Command::new("exiftool")
            .args([group_flag, "-j", image_path])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let json_str = String::from_utf8(output.stdout).ok()?;
        let json_value: Value = serde_json::from_str(&json_str).ok()?;
        let json_array = json_value.as_array()?;
        let json_obj = json_array.first()?.as_object()?;

        let mut assignments = HashMap::new();

        // Parse group-prefixed tags from ExifTool output
        // Format: "Group:TagName": "value" (for -G and -G1)
        // Format: "Group0:Group1:TagName": "value" (for -G2)
        for (key, _value) in json_obj {
            if let Some(colon_pos) = key.find(':') {
                let (group_part, tag_name) = key.split_at(colon_pos);
                let tag_name = &tag_name[1..]; // Remove the colon

                // For -G2, we need to handle the full hierarchy
                if group_flag == "-G2" {
                    // Group2 format: "Group0:Group1:TagName"
                    assignments.insert(tag_name.to_string(), group_part.to_string());
                } else {
                    // For -G and -G1, it's just "Group:TagName"
                    assignments.insert(tag_name.to_string(), group_part.to_string());
                }
            }
        }

        Some(assignments)
    }
}

/// Test Group0 (format family) assignments match ExifTool
#[test]
fn test_group0_assignments_match_exiftool() {
    let image_path = CANON_T3I_JPG;

    let exiftool_groups = match ExifToolGroupData::extract_from_exiftool(image_path) {
        Some(data) => data,
        None => {
            println!("Skipping Group0 compatibility test - ExifTool not available");
            return;
        }
    };

    let exif_data = extract_metadata(std::path::Path::new(image_path), false, false).unwrap();

    println!("Group0 (format family) compatibility test:");

    let mut total_compared = 0;
    let mut group0_matches = 0;
    let mut group0_mismatches = Vec::new();

    for tag in &exif_data.tags {
        if let Some(exiftool_group0) = exiftool_groups.group0_assignments.get(&tag.name) {
            total_compared += 1;

            if &tag.group == exiftool_group0 {
                group0_matches += 1;
            } else {
                group0_mismatches.push((
                    tag.name.clone(),
                    tag.group.clone(),
                    exiftool_group0.clone(),
                ));
            }
        }
    }

    println!("  Total Group0 comparisons: {total_compared}");
    println!("  Group0 matches: {group0_matches}");
    println!("  Group0 mismatches: {}", group0_mismatches.len());

    if !group0_mismatches.is_empty() {
        println!("  Group0 mismatch details:");
        for (tag_name, our_group0, exiftool_group0) in &group0_mismatches {
            println!("    {tag_name}: ours='{our_group0}' vs ExifTool='{exiftool_group0}'");
        }
    }

    // Group0 assignments should be mostly correct (these are stable)
    let group0_accuracy = group0_matches as f64 / total_compared as f64;
    println!("  Group0 accuracy: {:.1}%", group0_accuracy * 100.0);

    assert!(
        total_compared >= 20,
        "Should compare at least 20 tags for Group0, compared {total_compared}"
    );

    // Group0 should have high accuracy since it's format family (mostly EXIF)
    assert!(
        group0_accuracy >= 0.8,
        "Group0 accuracy should be at least 80%, got {:.1}%",
        group0_accuracy * 100.0
    );
}

/// Test Group1 (subdirectory location) assignments match ExifTool  
#[test]
fn test_group1_assignments_match_exiftool() {
    let image_path = CANON_T3I_JPG;

    let exiftool_groups = match ExifToolGroupData::extract_from_exiftool(image_path) {
        Some(data) => data,
        None => {
            println!("Skipping Group1 compatibility test - ExifTool not available");
            return;
        }
    };

    let exif_data = extract_metadata(std::path::Path::new(image_path), false, false).unwrap();

    println!("Group1 (subdirectory location) compatibility test:");

    let mut total_compared = 0;
    let mut group1_matches = 0;
    let mut group1_mismatches = Vec::new();
    let mut exif_ifd_mismatches = 0;

    for tag in &exif_data.tags {
        if let Some(exiftool_group1) = exiftool_groups.group1_assignments.get(&tag.name) {
            total_compared += 1;

            if &tag.group1 == exiftool_group1 {
                group1_matches += 1;
            } else {
                group1_mismatches.push((
                    tag.name.clone(),
                    tag.group1.clone(),
                    exiftool_group1.clone(),
                ));

                // Count specific ExifIFD mismatches (the known bug)
                if tag.group1 == "IFD0" && exiftool_group1 == "ExifIFD" {
                    exif_ifd_mismatches += 1;
                }
            }
        }
    }

    println!("  Total Group1 comparisons: {total_compared}");
    println!("  Group1 matches: {group1_matches}");
    println!("  Group1 mismatches: {}", group1_mismatches.len());
    println!("  ExifIFD-specific mismatches (known bug): {exif_ifd_mismatches}");

    if !group1_mismatches.is_empty() && group1_mismatches.len() <= 10 {
        println!("  Group1 mismatch details (first 10):");
        for (tag_name, our_group1, exiftool_group1) in group1_mismatches.iter().take(10) {
            println!("    {tag_name}: ours='{our_group1}' vs ExifTool='{exiftool_group1}'");
        }
    }

    let group1_accuracy = group1_matches as f64 / total_compared as f64;
    println!("  Group1 accuracy: {:.1}%", group1_accuracy * 100.0);

    assert!(
        total_compared >= 20,
        "Should compare at least 20 tags for Group1, compared {total_compared}"
    );

    // ✅ FIXED: ExifIFD namespace bug is now resolved, Group1 accuracy should be high

    if group1_accuracy >= 0.9 {
        println!("✅ Excellent Group1 accuracy - namespace bug is fixed!");
    } else if group1_accuracy >= 0.7 {
        println!("✅ Good Group1 accuracy - most ExifIFD tags correctly assigned");
    } else {
        println!("❌ Low Group1 accuracy - may indicate remaining issues");

        // Show some mismatches for debugging
        if !group1_mismatches.is_empty() {
            println!("  Sample mismatches:");
            for (tag_name, our_group1, exiftool_group1) in group1_mismatches.iter().take(5) {
                println!("    {tag_name}: ours='{our_group1}' vs ExifTool='{exiftool_group1}'");
            }
        }
    }

    // With the bug fixed, we should have high Group1 accuracy
    assert!(
        group1_accuracy >= 0.8,
        "Group1 accuracy should be at least 80% with bug fixed, got {:.1}%",
        group1_accuracy * 100.0
    );
}

/// Test Group2 (full hierarchy) assignments for complex cases
#[test]
fn test_group2_full_hierarchy_compatibility() {
    let image_path = CANON_T3I_JPG;

    let exiftool_groups = match ExifToolGroupData::extract_from_exiftool(image_path) {
        Some(data) => data,
        None => {
            println!("Skipping Group2 compatibility test - ExifTool not available");
            return;
        }
    };

    let exif_data = extract_metadata(std::path::Path::new(image_path), false, false).unwrap();

    println!("Group2 (full hierarchy) compatibility test:");

    // Group2 shows full hierarchy like "EXIF:IFD0" or "EXIF:ExifIFD"
    let mut hierarchy_patterns = HashMap::new();

    for tag in &exif_data.tags {
        if let Some(exiftool_group2) = exiftool_groups.group2_assignments.get(&tag.name) {
            let our_hierarchy = format!("{}:{}", tag.group, tag.group1);

            *hierarchy_patterns
                .entry((exiftool_group2.clone(), our_hierarchy))
                .or_insert(0) += 1;
        }
    }

    println!("  Group2 hierarchy patterns found:");
    for ((exiftool_hierarchy, our_hierarchy), count) in &hierarchy_patterns {
        println!("    ExifTool:'{exiftool_hierarchy}' vs Ours:'{our_hierarchy}' ({count} tags)");
    }

    // Verify we have reasonable hierarchy patterns
    assert!(
        !hierarchy_patterns.is_empty(),
        "Should find Group2 hierarchy patterns"
    );

    // Look for the expected EXIF:ExifIFD pattern (when bug is fixed)
    let has_exif_exif_ifd = hierarchy_patterns
        .keys()
        .any(|(exiftool, _)| exiftool.contains("EXIF") && exiftool.contains("ExifIFD"));

    println!("  ExifTool shows EXIF:ExifIFD hierarchy: {has_exif_exif_ifd}");

    // Note: ExifTool's Group2 uses different category names (Camera, Image, Time, etc.)
    // rather than the raw Group0:Group1 format we expected.
    // The presence of tags assigned to our "EXIF:ExifIFD" hierarchy indicates
    // correct processing even if ExifTool's Group2 uses semantic categories.

    let our_exif_exif_ifd_count = hierarchy_patterns
        .iter()
        .filter(|((_, our_hierarchy), _)| our_hierarchy == "EXIF:ExifIFD")
        .map(|(_, count)| count)
        .sum::<i32>();

    println!("  Our EXIF:ExifIFD tags: {our_exif_exif_ifd_count}");

    // With the bug fixed, we should have ExifIFD tags in our hierarchy
    assert!(
        our_exif_exif_ifd_count > 0,
        "Should have tags in EXIF:ExifIFD hierarchy"
    );
}

/// Test group assignment distribution and categories
#[test]
fn test_group_assignment_distribution() {
    let image_path = CANON_T3I_JPG;

    let exiftool_groups = match ExifToolGroupData::extract_from_exiftool(image_path) {
        Some(data) => data,
        None => {
            println!("Skipping group distribution test - ExifTool not available");
            return;
        }
    };

    let exif_data = extract_metadata(std::path::Path::new(image_path), false, false).unwrap();

    // Analyze ExifTool's group distribution
    let mut exiftool_group0_categories = HashSet::new();
    let mut exiftool_group1_categories = HashSet::new();

    for group0 in exiftool_groups.group0_assignments.values() {
        exiftool_group0_categories.insert(group0);
    }

    for group1 in exiftool_groups.group1_assignments.values() {
        exiftool_group1_categories.insert(group1);
    }

    // Analyze our group distribution
    let mut our_group0_categories = HashSet::new();
    let mut our_group1_categories = HashSet::new();

    for tag in &exif_data.tags {
        our_group0_categories.insert(&tag.group);
        our_group1_categories.insert(&tag.group1);
    }

    println!("Group distribution comparison:");
    println!("  ExifTool Group0 categories: {exiftool_group0_categories:?}");
    println!("  Our Group0 categories: {our_group0_categories:?}");
    println!("  ExifTool Group1 categories: {exiftool_group1_categories:?}");
    println!("  Our Group1 categories: {our_group1_categories:?}");

    // Verify we have similar diversity of group categories
    assert!(
        our_group0_categories.len() >= 2,
        "Should have at least 2 Group0 categories, found {}",
        our_group0_categories.len()
    );

    assert!(
        !our_group1_categories.is_empty(),
        "Should have at least 1 Group1 category, found {}",
        our_group1_categories.len()
    );

    // ExifTool should show ExifIFD as a Group1 category for Canon T3i
    assert!(
        exiftool_group1_categories
            .iter()
            .any(|s| s.as_str() == "ExifIFD"),
        "ExifTool should show ExifIFD as Group1 category for Canon T3i"
    );

    // ✅ FIXED: We should also have ExifIFD
    assert!(
        our_group1_categories
            .iter()
            .any(|s| s.as_str() == "ExifIFD"),
        "We should also have ExifIFD as Group1 category"
    );

    // Both should have "EXIF" as primary Group0
    assert!(
        exiftool_group0_categories
            .iter()
            .any(|s| s.as_str() == "EXIF"),
        "ExifTool should have EXIF as Group0 category"
    );
    assert!(
        our_group0_categories.iter().any(|s| s.as_str() == "EXIF"),
        "We should have EXIF as Group0 category"
    );
}

/// Test specific ExifIFD tags that are key indicators of correct grouping
#[test]
fn test_key_exif_ifd_tag_grouping() {
    let image_path = CANON_T3I_JPG;

    let exiftool_groups = match ExifToolGroupData::extract_from_exiftool(image_path) {
        Some(data) => data,
        None => {
            println!("Skipping key ExifIFD tag test - ExifTool not available");
            return;
        }
    };

    let exif_data = extract_metadata(std::path::Path::new(image_path), false, false).unwrap();

    // Key ExifIFD tags that are reliable indicators
    let key_exif_ifd_tags = [
        "ExifVersion",     // 0x9000 - Always in ExifIFD
        "FlashpixVersion", // 0xA000 - Always in ExifIFD
        "ColorSpace",      // 0xA001 - Always in ExifIFD
        "ExifImageWidth",  // 0xA002 - Always in ExifIFD
        "ExifImageHeight", // 0xA003 - Always in ExifIFD
    ];

    println!("Key ExifIFD tag grouping verification:");

    let mut key_tags_found = 0;
    let mut correct_group1_assignments = 0;

    for tag_name in &key_exif_ifd_tags {
        if let (Some(our_tag), Some(exiftool_group1)) = (
            exif_data.get_tag_by_name(tag_name),
            exiftool_groups.group1_assignments.get(*tag_name),
        ) {
            key_tags_found += 1;

            println!(
                "  {}: ours='{}' vs ExifTool='{}'",
                tag_name, our_tag.group1, exiftool_group1
            );

            if &our_tag.group1 == exiftool_group1 {
                correct_group1_assignments += 1;
            }

            // Verify ExifTool assigns these to ExifIFD (sanity check)
            assert_eq!(
                exiftool_group1, "ExifIFD",
                "ExifTool should assign {tag_name} to ExifIFD Group1"
            );
        }
    }

    assert!(
        key_tags_found >= 3,
        "Should find at least 3 key ExifIFD tags, found {key_tags_found}"
    );

    let key_tag_accuracy = correct_group1_assignments as f64 / key_tags_found as f64;
    println!(
        "  Key ExifIFD tag Group1 accuracy: {:.1}%",
        key_tag_accuracy * 100.0
    );

    // ✅ FIXED: Should be 100% with bug resolved
    if key_tag_accuracy >= 1.0 {
        println!("✅ Key ExifIFD tags have correct Group1 - bug is fixed!");
    } else {
        println!("❌ Key ExifIFD tags have incorrect Group1 (unexpected - bug should be fixed)");
    }

    // With the bug fixed, all key ExifIFD tags should have correct Group1
    assert!(
        key_tag_accuracy >= 1.0,
        "All key ExifIFD tags should have correct Group1 with bug fixed, got {:.1}%",
        key_tag_accuracy * 100.0
    );
}
