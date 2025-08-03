//! Integration test for P15: Sony TAG_PREFIX Implementation
//!
//! This test validates that Sony unknown tags receive manufacturer-specific
//! TAG_PREFIX naming (Sony_0xXXXX) instead of generic naming (Tag_XXXX).
//!
//! P15: Sony TAG_PREFIX Implementation for Unknown Tag Naming  
//! see docs/todo/P15-TAG_PREFIX.md

use exif_oxide::{extract_metadata_with_filter, FilterOptions};
use std::path::Path;

#[cfg(feature = "integration-tests")]
#[test]
fn test_sony_tag_prefix_behavior() {
    // P15: Sony TAG_PREFIX Implementation - see docs/todo/P15-TAG_PREFIX.md
    // Sony unknown tags should show "MakerNotes:Sony_0xXXXX" format instead of "MakerNotes:Tag_XXXX"
    // ExifTool reference: ./exiftool -u -j -G third-party/exiftool/t/images/Sony.jpg

    const TEST_IMAGE: &str = "third-party/exiftool/t/images/Sony.jpg";

    // Extract all metadata to capture unknown tags
    let filter = FilterOptions::extract_all();
    let result = extract_metadata_with_filter(Path::new(TEST_IMAGE), Some(filter));

    assert!(
        result.is_ok(),
        "Failed to read Sony test image: {:?}",
        result.err()
    );

    let exif_data = result.unwrap();

    // Create helper functions for tag analysis
    let find_tags_with_prefix = |prefix: &str| -> Vec<String> {
        exif_data
            .tags
            .iter()
            .filter(|tag| tag.name.starts_with(prefix))
            .map(|tag| format!("{}:{}", tag.group, tag.name))
            .collect()
    };

    let find_tag = |name: &str| -> Option<String> {
        exif_data
            .tags
            .iter()
            .find(|tag| format!("{}:{}", tag.group, tag.name) == name)
            .map(|tag| tag.print.to_string())
    };

    // Test 1: Count generic Tag_ entries
    let generic_tag_entries = find_tags_with_prefix("Tag_");
    println!("Found generic Tag_ entries: {:?}", generic_tag_entries);

    // Test 2: Count Sony_0x entries (what we expect)
    let sony_prefix_entries = find_tags_with_prefix("Sony_0x");
    println!("Found Sony_0x entries: {:?}", sony_prefix_entries);

    // Test 3: Verify specific Sony unknown tags that should use TAG_PREFIX
    // Based on ExifTool output: MakerNotes:Sony_0x2000, Sony_0x9001, etc.
    let expected_sony_tags = [
        "MakerNotes:Sony_0x2000",
        "MakerNotes:Sony_0x9001",
        "MakerNotes:Sony_0x9002",
        "MakerNotes:Sony_0x9005",
        "MakerNotes:Sony_0x9006",
        "MakerNotes:Sony_0x9007",
        "MakerNotes:Sony_0x9008",
    ];

    let mut found_sony_prefix = 0;
    for expected_tag in &expected_sony_tags {
        if let Some(value) = find_tag(expected_tag) {
            found_sony_prefix += 1;
            println!(
                "✓ Found expected Sony TAG_PREFIX tag: {} = {:?}",
                expected_tag, value
            );
        } else {
            // Look for equivalent Tag_ version (current broken behavior)
            let tag_name = expected_tag.replace("Sony_0x", "Tag_");
            if let Some(value) = find_tag(&tag_name) {
                println!(
                    "✗ Found generic tag instead of Sony TAG_PREFIX: {} = {:?}",
                    tag_name, value
                );
            }
        }
    }

    // Test 4: Ensure EXIF:Tag_C4A5 remains (this is correct - not a MakerNotes tag)
    let exif_tag_c4a5 = find_tag("EXIF:Tag_C4A5");
    assert!(
        exif_tag_c4a5.is_some(),
        "EXIF:Tag_C4A5 should remain as generic tag (not manufacturer-specific)"
    );

    // MAIN ASSERTION: Sony unknown tags should use TAG_PREFIX format
    // Current behavior: 7 MakerNotes:Tag_XXXX entries
    // Expected behavior: 7 MakerNotes:Sony_0xXXXX entries
    assert!(
        found_sony_prefix >= 7,
        "Expected at least 7 Sony TAG_PREFIX tags (Sony_0xXXXX format), but found {}. \
         Current behavior shows generic Tag_XXXX format instead of manufacturer-specific Sony_0xXXXX. \
         This indicates hardcoded fallbacks are bypassing the TAG_PREFIX mechanism. \
         Sony TAG_PREFIX requires fixing hardcoded format!(\"Tag_{{tag_id:04X}}\") calls.",
        found_sony_prefix
    );

    // Test 5: Verify TAG_PREFIX behavior is consistent
    // Should have exactly 1 generic Tag_ entry (EXIF:Tag_C4A5)
    // and 0 generic MakerNotes:Tag_ entries (all should be Sony_0xXXXX)
    let makernotes_generic_tags: Vec<String> = exif_data
        .tags
        .iter()
        .filter(|tag| tag.group == "MakerNotes" && tag.name.starts_with("Tag_"))
        .map(|tag| format!("{}:{}", tag.group, tag.name))
        .collect();

    assert!(
        makernotes_generic_tags.is_empty(),
        "MakerNotes should not contain generic Tag_ entries when TAG_PREFIX is working. \
         Found: {:?}. These should be Sony_0xXXXX format instead.",
        makernotes_generic_tags
    );

    println!("✓ Sony TAG_PREFIX behavior validated successfully");
}
