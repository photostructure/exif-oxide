//! ExifTool-style filtering for comparison tools
//!
//! This module provides functions to parse and apply ExifTool-style tag filters
//! like `-EXIF:all`, `-Orientation#`, `-GPS*`, etc.

use crate::hash::ImageHashType;
use crate::types::FilterOptions;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// Parse ExifTool-style filter expressions into FilterOptions
///
/// Supports all the patterns that the main CLI supports:
/// - `-TagName` - extract specific tag
/// - `-TagName#` - extract tag with numeric value (ValueConv)  
/// - `-GroupName:all` - extract all tags from group
/// - `-GPS*` - glob patterns
/// - `-all` - extract all tags
///
/// # Examples
///
/// ```
/// use exif_oxide::compat::parse_exiftool_filters;
///
/// // Parse EXIF group filter
/// let filter = parse_exiftool_filters(&["-EXIF:all"]);
///
/// // Parse specific tag with numeric output
/// let filter = parse_exiftool_filters(&["-Orientation#"]);
///
/// // Parse glob pattern
/// let filter = parse_exiftool_filters(&["-GPS*"]);
/// ```
pub fn parse_exiftool_filters(args: &[&str]) -> FilterOptions {
    let mut requested_tags = Vec::new();
    let requested_groups = Vec::new();
    let mut group_all_patterns = Vec::new();
    let mut glob_patterns = Vec::new();
    let mut numeric_tags = HashSet::new();
    let mut extract_all = false;

    for arg in args {
        if *arg == "-all" || *arg == "--all" {
            // Special case: extract all tags
            extract_all = true;
        } else if arg.starts_with('-') && arg.len() > 1 {
            // Process tag/group filters
            let filter_arg = &arg[1..]; // Remove leading '-'

            if filter_arg.ends_with('#') && filter_arg.len() > 1 {
                // Numeric tag: -TagName# or -Pattern#
                let tag_name = &filter_arg[..filter_arg.len() - 1];
                if tag_name.contains('*') {
                    // Glob pattern with numeric: -GPS*#
                    glob_patterns.push(tag_name.to_string());
                    numeric_tags.insert(tag_name.to_string());
                } else {
                    // Regular numeric tag: -TagName#
                    requested_tags.push(tag_name.to_string());
                    numeric_tags.insert(tag_name.to_string());
                }
            } else if filter_arg.ends_with(":all") {
                // Group all pattern: -GroupName:all
                group_all_patterns.push(filter_arg.to_string());
            } else if filter_arg.contains('*') {
                // Glob pattern: -GPS*, -*tude, -*Date*, -EXIF:*
                glob_patterns.push(filter_arg.to_string());
            } else if filter_arg.contains(':') {
                // Group:tag pattern (future extension)
                // For now, treat as specific tag request
                requested_tags.push(filter_arg.to_string());
            } else {
                // Simple tag name: -TagName
                requested_tags.push(filter_arg.to_string());
            }
        }
    }

    // Build FilterOptions based on parsed arguments
    if extract_all {
        // -all flag overrides everything else
        FilterOptions {
            requested_tags: Vec::new(),
            requested_groups: Vec::new(),
            group_all_patterns: Vec::new(),
            extract_all: true,
            numeric_tags,
            glob_patterns: Vec::new(),
            compute_image_hash: false,
            image_hash_type: ImageHashType::default(),
        }
    } else if requested_tags.is_empty()
        && requested_groups.is_empty()
        && group_all_patterns.is_empty()
        && glob_patterns.is_empty()
    {
        // No filters specified - extract all tags (backward compatibility)
        FilterOptions {
            requested_tags: Vec::new(),
            requested_groups: Vec::new(),
            group_all_patterns: Vec::new(),
            extract_all: true,
            numeric_tags,
            glob_patterns: Vec::new(),
            compute_image_hash: false,
            image_hash_type: ImageHashType::default(),
        }
    } else {
        // Specific filters requested
        FilterOptions {
            requested_tags,
            requested_groups,
            group_all_patterns,
            extract_all: false,
            numeric_tags,
            glob_patterns,
            compute_image_hash: false,
            image_hash_type: ImageHashType::default(),
        }
    }
}

/// Apply ExifTool-style filtering to JSON output
///
/// This filters a JSON object containing tag data to only include tags
/// that match the FilterOptions criteria.
pub fn apply_exiftool_filter(data: &Value, filter: &FilterOptions) -> Value {
    if filter.extract_all {
        return data.clone();
    }

    if let Some(obj) = data.as_object() {
        let filtered: HashMap<String, Value> = obj
            .iter()
            .filter(|(key, _)| {
                // Always include SourceFile
                if key.as_str() == "SourceFile" {
                    return true;
                }

                // Parse group and tag from key (e.g., "EXIF:Orientation")
                let (group, tag) = if let Some((g, t)) = key.split_once(':') {
                    (g, t)
                } else {
                    // No group prefix, treat as tag name only
                    ("", key.as_str())
                };

                // Also check if the full key matches any requested tags
                // This handles cases like -EXIF:Orientation where the user specifies the full key
                if filter
                    .requested_tags
                    .iter()
                    .any(|req_tag| req_tag.to_lowercase() == key.to_lowercase())
                {
                    return true;
                }

                filter.should_extract_tag(tag, group)
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        serde_json::to_value(filtered).unwrap()
    } else {
        data.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_group_all_filter() {
        let filter = parse_exiftool_filters(&["-EXIF:all"]);
        assert!(!filter.extract_all);
        assert_eq!(filter.group_all_patterns, vec!["EXIF:all"]);
        assert!(filter.requested_tags.is_empty());
    }

    #[test]
    fn test_parse_numeric_tag_filter() {
        let filter = parse_exiftool_filters(&["-Orientation#"]);
        assert!(!filter.extract_all);
        assert_eq!(filter.requested_tags, vec!["Orientation"]);
        assert!(filter.numeric_tags.contains("Orientation"));
    }

    #[test]
    fn test_parse_glob_pattern() {
        let filter = parse_exiftool_filters(&["-GPS*"]);
        assert!(!filter.extract_all);
        assert_eq!(filter.glob_patterns, vec!["GPS*"]);
    }

    #[test]
    fn test_parse_multiple_filters() {
        let filter = parse_exiftool_filters(&["-EXIF:all", "-GPS*", "-Orientation#"]);
        assert!(!filter.extract_all);
        assert_eq!(filter.group_all_patterns, vec!["EXIF:all"]);
        assert_eq!(filter.glob_patterns, vec!["GPS*"]);
        assert_eq!(filter.requested_tags, vec!["Orientation"]);
        assert!(filter.numeric_tags.contains("Orientation"));
    }

    #[test]
    fn test_apply_filter_to_json() {
        let json_data = serde_json::json!({
            "SourceFile": "test.jpg",
            "EXIF:Orientation": 1,
            "EXIF:Make": "Canon",
            "GPS:Latitude": 37.7749,
            "File:MIMEType": "image/jpeg"
        });

        let filter = parse_exiftool_filters(&["-EXIF:all"]);
        let filtered = apply_exiftool_filter(&json_data, &filter);

        // Should include SourceFile and all EXIF tags, but not GPS or File tags
        assert!(filtered.get("SourceFile").is_some());
        assert!(filtered.get("EXIF:Orientation").is_some());
        assert!(filtered.get("EXIF:Make").is_some());
        assert!(filtered.get("GPS:Latitude").is_none());
        assert!(filtered.get("File:MIMEType").is_none());
    }
}
