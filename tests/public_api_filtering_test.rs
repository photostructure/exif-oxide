//! Test the public API filtering functionality
//!
//! Tests the new public API functions added for tag filtering support

use exif_oxide::{extract_metadata_json_with_filter, extract_metadata_with_filter, FilterOptions};
use std::collections::HashSet;
use std::path::Path;

const TEST_IMAGE: &str = "test-images/canon/eos_rebel_t3i.jpg";

#[test]
fn test_extract_metadata_json_with_filter_specific_tag() {
    // Test extracting only MIMEType tag via JSON API
    let filter = FilterOptions::tags_only(vec!["MIMEType".to_string()]);
    let result = extract_metadata_json_with_filter(TEST_IMAGE, Some(filter)).unwrap();

    // Should be a JSON object with only SourceFile and File:MIMEType
    let obj = result.as_object().unwrap();
    assert!(obj.contains_key("SourceFile"));
    assert!(obj.contains_key("File:MIMEType"));
    assert_eq!(
        obj.get("File:MIMEType").unwrap().as_str().unwrap(),
        "image/jpeg"
    );

    // Should not contain EXIF tags like Make, Model, etc.
    assert!(!obj.contains_key("EXIF:Make"));
    assert!(!obj.contains_key("EXIF:Model"));
}

#[test]
fn test_extract_metadata_json_with_filter_numeric_control() {
    // Test numeric control with # suffix
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

    let result = extract_metadata_json_with_filter(TEST_IMAGE, Some(filter)).unwrap();
    let obj = result.as_object().unwrap();

    // Should contain numeric Orientation value
    assert!(obj.contains_key("EXIF:Orientation"));
    let orientation = obj.get("EXIF:Orientation").unwrap();
    // Should be numeric (8) not string ("Rotate 270 CW")
    assert!(orientation.is_number());
    assert_eq!(orientation.as_u64().unwrap(), 8);
}

#[test]
fn test_extract_metadata_json_with_filter_glob_pattern() {
    // Test glob pattern support
    let filter = FilterOptions {
        requested_tags: vec![],
        requested_groups: vec![],
        group_all_patterns: vec![],
        extract_all: false,
        numeric_tags: HashSet::new(),
        glob_patterns: vec!["*Image*".to_string()],
    };

    let result = extract_metadata_json_with_filter(TEST_IMAGE, Some(filter)).unwrap();
    let obj = result.as_object().unwrap();

    // Should contain tags with "Image" in the name
    let image_tags: Vec<_> = obj.keys().filter(|k| k.contains("Image")).collect();

    assert!(!image_tags.is_empty(), "Expected tags containing 'Image'");

    // All returned tags should match the pattern
    for key in obj.keys() {
        if key != "SourceFile" {
            let tag_name = key.split(':').next_back().unwrap_or(key);
            assert!(tag_name.to_lowercase().contains("image"));
        }
    }
}

#[test]
fn test_extract_metadata_json_with_filter_middle_wildcard() {
    // Test middle wildcard pattern like -*Date*
    let filter = FilterOptions {
        requested_tags: vec![],
        requested_groups: vec![],
        group_all_patterns: vec![],
        extract_all: false,
        numeric_tags: HashSet::new(),
        glob_patterns: vec!["*Date*".to_string()],
    };

    let result = extract_metadata_json_with_filter(TEST_IMAGE, Some(filter)).unwrap();
    let obj = result.as_object().unwrap();

    // Should contain tags with "Date" in the name
    let date_tags: Vec<_> = obj
        .keys()
        .filter(|k| k.to_lowercase().contains("date"))
        .collect();

    assert!(!date_tags.is_empty(), "Expected tags containing 'Date'");
    assert!(
        date_tags.len() >= 6,
        "Expected multiple date tags, got {}",
        date_tags.len()
    );

    // Verify some specific expected date tags
    assert!(obj.contains_key("EXIF:CreateDate"));
    assert!(obj.contains_key("EXIF:DateTimeOriginal"));
    assert!(obj.contains_key("File:FileModifyDate"));

    // All returned tags should match the pattern
    for key in obj.keys() {
        if key != "SourceFile" {
            let tag_name = key.split(':').next_back().unwrap_or(key);
            assert!(
                tag_name.to_lowercase().contains("date"),
                "Non-date tag found: {}",
                key
            );
        }
    }
}

#[test]
fn test_extract_metadata_with_filter_structured_data() {
    // Test the structured data API (returns ExifData directly)
    let filter = FilterOptions::tags_only(vec!["MIMEType".to_string()]);
    let result = extract_metadata_with_filter(Path::new(TEST_IMAGE), Some(filter)).unwrap();

    // Should have exactly one tag (MIMEType)
    assert_eq!(result.tags.len(), 1);
    assert_eq!(result.tags[0].name, "MIMEType");
    assert_eq!(result.tags[0].group, "File");

    // Check the value
    match &result.tags[0].value {
        exif_oxide::TagValue::String(s) => assert_eq!(s, "image/jpeg"),
        other => panic!("Expected string value, got: {:?}", other),
    }
}

#[test]
fn test_extract_metadata_with_filter_no_filter() {
    // Test without filter (should extract all)
    let result = extract_metadata_with_filter(Path::new(TEST_IMAGE), None).unwrap();

    // Should have many tags
    assert!(
        result.tags.len() > 50,
        "Expected many tags, got {}",
        result.tags.len()
    );

    // Should contain common EXIF tags
    let tag_names: Vec<_> = result.tags.iter().map(|t| &t.name).collect();
    assert!(tag_names.contains(&&"Make".to_string()));
    assert!(tag_names.contains(&&"Model".to_string()));
    assert!(tag_names.contains(&&"MIMEType".to_string()));
}

#[test]
fn test_extract_metadata_json_with_filter_group_all() {
    // Test group:all pattern
    let filter = FilterOptions {
        requested_tags: vec![],
        requested_groups: vec![],
        group_all_patterns: vec!["File:all".to_string()],
        extract_all: false,
        numeric_tags: HashSet::new(),
        glob_patterns: vec![],
    };

    let result = extract_metadata_json_with_filter(TEST_IMAGE, Some(filter)).unwrap();
    let obj = result.as_object().unwrap();

    // Should contain File group tags
    assert!(obj.contains_key("File:MIMEType"));
    assert!(obj.contains_key("File:FileSize"));

    // Should not contain EXIF group tags
    assert!(!obj.contains_key("EXIF:Make"));
    assert!(!obj.contains_key("EXIF:Model"));

    // All tags except SourceFile should be from File group
    for key in obj.keys() {
        if key != "SourceFile" && key != "ExifToolVersion" {
            assert!(key.starts_with("File:"), "Non-File tag found: {}", key);
        }
    }
}
