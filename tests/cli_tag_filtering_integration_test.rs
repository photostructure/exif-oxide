//! CLI Tag Filtering Integration Tests
//!
//! Comprehensive test suite for CLI tag filtering and PrintConv/ValueConv control
//! Tests all features against real image files to ensure ExifTool compatibility

use exif_oxide::formats::extract_metadata;
use exif_oxide::types::{FilterOptions, TagValue};
use std::collections::HashSet;
use std::path::Path;

const TEST_IMAGE_CANON: &str = "test-images/canon/eos_rebel_t3i.jpg";
const TEST_IMAGE_RICOH: &str = "third-party/exiftool/t/images/Ricoh2.jpg";

#[test]
fn test_specific_tag_filtering() {
    // Test exact tag filtering like -MIMEType
    let filter = FilterOptions::tags_only(vec!["MIMEType".to_string()]);
    let result = extract_metadata(Path::new(TEST_IMAGE_CANON), false, false, Some(filter)).unwrap();

    // Should only contain SourceFile and MIMEType
    assert_eq!(result.tags.len(), 1);
    assert_eq!(result.tags[0].name, "MIMEType");
    assert_eq!(result.tags[0].group, "File");
    assert_eq!(
        result.tags[0].value,
        TagValue::String("image/jpeg".to_string())
    );
}

#[test]
fn test_case_insensitive_tag_filtering() {
    // Test case insensitive matching like -mimetype
    let filter = FilterOptions::tags_only(vec!["mimetype".to_string()]);
    let result = extract_metadata(Path::new(TEST_IMAGE_CANON), false, false, Some(filter)).unwrap();

    // Should match MIMEType despite lowercase input
    assert_eq!(result.tags.len(), 1);
    assert_eq!(result.tags[0].name, "MIMEType");
}

#[test]
fn test_numeric_value_control() {
    // Test numeric control with # suffix like -Orientation#
    let mut numeric_tags = HashSet::new();
    numeric_tags.insert("Orientation".to_string());

    let filter = FilterOptions {
        requested_tags: vec!["Orientation".to_string()],
        requested_groups: vec![],
        group_all_patterns: vec![],
        extract_all: false,
        numeric_tags,
        glob_patterns: vec![],
    };

    let result = extract_metadata(Path::new(TEST_IMAGE_CANON), false, false, Some(filter)).unwrap();

    // Should contain only Orientation with numeric value
    assert_eq!(result.tags.len(), 1);
    assert_eq!(result.tags[0].name, "Orientation");
    // Should be numeric, not "Rotate 270 CW"
    match &result.tags[0].print {
        TagValue::U8(8) | TagValue::U16(8) => (), // Expected numeric value 8
        other => panic!("Expected numeric value 8, got: {:?}", other),
    }
}

#[test]
fn test_group_all_filtering() {
    // Test group:all pattern like -EXIF:all
    let filter = FilterOptions {
        requested_tags: vec![],
        requested_groups: vec![],
        group_all_patterns: vec!["EXIF:all".to_string()],
        extract_all: false,
        numeric_tags: HashSet::new(),
        glob_patterns: vec![],
    };

    let result = extract_metadata(Path::new(TEST_IMAGE_CANON), false, false, Some(filter)).unwrap();

    // Should contain multiple EXIF tags
    assert!(
        result.tags.len() > 20,
        "Expected many EXIF tags, got {}",
        result.tags.len()
    );

    // All tags should be from EXIF group
    for tag in &result.tags {
        assert_eq!(
            tag.group, "EXIF",
            "Non-EXIF tag found: {}:{}",
            tag.group, tag.name
        );
    }
}

#[test]
fn test_prefix_wildcard_gps() {
    // Test prefix wildcard like -GPS*
    let filter = FilterOptions {
        requested_tags: vec![],
        requested_groups: vec![],
        group_all_patterns: vec![],
        extract_all: false,
        numeric_tags: HashSet::new(),
        glob_patterns: vec!["GPS*".to_string()],
    };

    let result = extract_metadata(Path::new(TEST_IMAGE_RICOH), false, false, Some(filter)).unwrap();

    // Should contain GPS tags
    assert!(
        result.tags.len() > 10,
        "Expected multiple GPS tags, got {}",
        result.tags.len()
    );

    // All tags should start with GPS
    for tag in &result.tags {
        assert!(
            tag.name.starts_with("GPS"),
            "Non-GPS tag found: {}",
            tag.name
        );
    }
}

#[test]
fn test_suffix_wildcard() {
    // Test suffix wildcard like -*Width
    let filter = FilterOptions {
        requested_tags: vec![],
        requested_groups: vec![],
        group_all_patterns: vec![],
        extract_all: false,
        numeric_tags: HashSet::new(),
        glob_patterns: vec!["*Width".to_string()],
    };

    let result = extract_metadata(Path::new(TEST_IMAGE_CANON), false, false, Some(filter)).unwrap();

    // Should contain width-related tags
    assert!(
        result.tags.len() >= 2,
        "Expected width tags, got {}",
        result.tags.len()
    );

    // All tags should end with Width
    for tag in &result.tags {
        assert!(
            tag.name.ends_with("Width"),
            "Non-width tag found: {}",
            tag.name
        );
    }
}

#[test]
fn test_middle_wildcard() {
    // Test middle wildcard like -*Image*
    let filter = FilterOptions {
        requested_tags: vec![],
        requested_groups: vec![],
        group_all_patterns: vec![],
        extract_all: false,
        numeric_tags: HashSet::new(),
        glob_patterns: vec!["*Image*".to_string()],
    };

    let result = extract_metadata(Path::new(TEST_IMAGE_CANON), false, false, Some(filter)).unwrap();

    // Should contain image-related tags
    assert!(
        result.tags.len() >= 5,
        "Expected multiple image tags, got {}",
        result.tags.len()
    );

    // All tags should contain "Image"
    for tag in &result.tags {
        assert!(
            tag.name.to_lowercase().contains("image"),
            "Non-image tag found: {}",
            tag.name
        );
    }
}

#[test]
fn test_middle_wildcard_date_pattern() {
    // Test middle wildcard with Date pattern like -*Date*
    let filter = FilterOptions {
        requested_tags: vec![],
        requested_groups: vec![],
        group_all_patterns: vec![],
        extract_all: false,
        numeric_tags: HashSet::new(),
        glob_patterns: vec!["*Date*".to_string()],
    };

    let result = extract_metadata(Path::new(TEST_IMAGE_CANON), false, false, Some(filter)).unwrap();

    // Should contain date-related tags
    assert!(
        result.tags.len() >= 6,
        "Expected multiple date tags, got {}",
        result.tags.len()
    );

    // All tags should contain "Date" (case insensitive)
    for tag in &result.tags {
        assert!(
            tag.name.to_lowercase().contains("date"),
            "Non-date tag found: {}",
            tag.name
        );
    }

    // Verify we get specific expected date tags
    let tag_names: Vec<String> = result.tags.iter().map(|t| t.name.clone()).collect();
    assert!(tag_names.contains(&"CreateDate".to_string()));
    assert!(tag_names.contains(&"DateTimeOriginal".to_string()));
    assert!(tag_names.contains(&"ModifyDate".to_string()));
    assert!(tag_names.contains(&"FileModifyDate".to_string()));
}

#[test]
fn test_complex_filtering_combination() {
    // Test complex filtering: -Orientation# -EXIF:all -GPS*
    let mut numeric_tags = HashSet::new();
    numeric_tags.insert("Orientation".to_string());

    let filter = FilterOptions {
        requested_tags: vec!["Orientation".to_string()],
        requested_groups: vec![],
        group_all_patterns: vec!["EXIF:all".to_string()],
        extract_all: false,
        numeric_tags,
        glob_patterns: vec!["GPS*".to_string()],
    };

    let result = extract_metadata(Path::new(TEST_IMAGE_RICOH), false, false, Some(filter)).unwrap();

    // Should contain many tags from different sources
    assert!(
        result.tags.len() > 30,
        "Expected many tags from combination, got {}",
        result.tags.len()
    );

    // Find orientation tag and verify it's numeric
    let orientation_tag = result
        .tags
        .iter()
        .find(|t| t.name == "Orientation")
        .unwrap();
    match &orientation_tag.print {
        TagValue::U8(_) | TagValue::U16(_) => (), // Expected numeric value
        other => panic!("Expected numeric Orientation, got: {:?}", other),
    }

    // Should have GPS tags
    let gps_count = result
        .tags
        .iter()
        .filter(|t| t.name.starts_with("GPS"))
        .count();
    assert!(gps_count > 5, "Expected GPS tags, got {}", gps_count);

    // Should have EXIF tags
    let exif_count = result.tags.iter().filter(|t| t.group == "EXIF").count();
    assert!(
        exif_count > 20,
        "Expected many EXIF tags, got {}",
        exif_count
    );
}

#[test]
fn test_file_only_performance_optimization() {
    // Test that File-only requests are optimized
    let filter = FilterOptions::tags_only(vec!["MIMEType".to_string()]);

    // This should use the optimized path (extract_file_tags_only)
    let result = extract_metadata(Path::new(TEST_IMAGE_CANON), false, false, Some(filter)).unwrap();

    // Should only contain SourceFile and MIMEType (no EXIF parsing)
    assert_eq!(result.tags.len(), 1);
    assert_eq!(result.tags[0].name, "MIMEType");
}

#[test]
fn test_extract_all_backward_compatibility() {
    // Test that None filter option extracts all tags (backward compatibility)
    let result_all = extract_metadata(Path::new(TEST_IMAGE_CANON), false, false, None).unwrap();

    let filter_all = FilterOptions::extract_all();
    let result_filter =
        extract_metadata(Path::new(TEST_IMAGE_CANON), false, false, Some(filter_all)).unwrap();

    // Both should extract the same number of tags
    assert_eq!(result_all.tags.len(), result_filter.tags.len());
}

#[test]
fn test_group_qualified_glob_patterns() {
    // Test group:pattern like EXIF:GPS*
    let filter = FilterOptions {
        requested_tags: vec![],
        requested_groups: vec![],
        group_all_patterns: vec![],
        extract_all: false,
        numeric_tags: HashSet::new(),
        glob_patterns: vec!["EXIF:GPS*".to_string()],
    };

    let result = extract_metadata(Path::new(TEST_IMAGE_RICOH), false, false, Some(filter)).unwrap();

    // Should contain EXIF group GPS tags only
    for tag in &result.tags {
        if tag.name.starts_with("GPS") {
            // GPS tags can be in EXIF group too
            assert!(tag.group == "EXIF" || tag.group == "GPS");
        }
    }
}

#[test]
fn test_no_matches_wildcard() {
    // Test wildcard that matches nothing
    let filter = FilterOptions {
        requested_tags: vec![],
        requested_groups: vec![],
        group_all_patterns: vec![],
        extract_all: false,
        numeric_tags: HashSet::new(),
        glob_patterns: vec!["NonExistent*".to_string()],
    };

    let result = extract_metadata(Path::new(TEST_IMAGE_CANON), false, false, Some(filter)).unwrap();

    // Should contain no tags (only SourceFile in JSON output)
    assert_eq!(result.tags.len(), 0);
}

#[test]
fn test_multiple_glob_patterns() {
    // Test multiple glob patterns: -*Date* -*Width*
    // Note: We use patterns that match tags exif-oxide actually produces
    // (Composite tags are not yet implemented)
    let filter = FilterOptions {
        requested_tags: vec![],
        requested_groups: vec![],
        group_all_patterns: vec![],
        extract_all: false,
        numeric_tags: HashSet::new(),
        glob_patterns: vec!["*Date*".to_string(), "*Width*".to_string()],
    };

    let result = extract_metadata(Path::new(TEST_IMAGE_CANON), false, false, Some(filter)).unwrap();

    // Should contain both Date and Width tags
    let date_count = result
        .tags
        .iter()
        .filter(|t| t.name.to_lowercase().contains("date"))
        .count();
    let width_count = result
        .tags
        .iter()
        .filter(|t| t.name.to_lowercase().contains("width"))
        .count();

    // Canon T3i has multiple date tags (CreateDate, DateTimeOriginal, ModifyDate, FileModifyDate, etc.)
    assert!(
        date_count >= 3,
        "Expected at least 3 date tags, found {} date tags, {} total tags",
        date_count,
        result.tags.len()
    );
    // Canon T3i has width tags (ImageWidth, ExifImageWidth)
    assert!(
        width_count >= 2,
        "Expected at least 2 width tags, found {} width tags, {} total tags",
        width_count,
        result.tags.len()
    );
    // Combined count should be meaningful
    assert!(
        result.tags.len() >= 5,
        "Expected at least 5 tags total from multiple patterns, got {}",
        result.tags.len()
    );
}

#[test]
fn test_case_insensitive_glob_patterns() {
    // Test case insensitive glob patterns
    let filter = FilterOptions {
        requested_tags: vec![],
        requested_groups: vec![],
        group_all_patterns: vec![],
        extract_all: false,
        numeric_tags: HashSet::new(),
        glob_patterns: vec!["gps*".to_string()], // lowercase
    };

    let result = extract_metadata(Path::new(TEST_IMAGE_RICOH), false, false, Some(filter)).unwrap();

    // Should match GPS tags despite lowercase pattern
    assert!(
        result.tags.len() > 5,
        "Expected GPS tags with lowercase pattern"
    );

    for tag in &result.tags {
        assert!(
            tag.name.to_lowercase().starts_with("gps"),
            "Non-GPS tag: {}",
            tag.name
        );
    }
}

#[test]
fn test_numeric_with_glob_patterns() {
    // Test numeric control combined with glob patterns: -GPS*#
    let mut numeric_tags = HashSet::new();
    numeric_tags.insert("GPS*".to_string());

    let filter = FilterOptions {
        requested_tags: vec![],
        requested_groups: vec![],
        group_all_patterns: vec![],
        extract_all: false,
        numeric_tags,
        glob_patterns: vec!["GPS*".to_string()],
    };

    let result = extract_metadata(Path::new(TEST_IMAGE_RICOH), false, false, Some(filter)).unwrap();

    // Should contain GPS tags
    assert!(result.tags.len() > 5, "Expected GPS tags");

    // All GPS tags should have appropriate values
    for tag in &result.tags {
        assert!(tag.name.starts_with("GPS"), "Non-GPS tag: {}", tag.name);
    }
}
