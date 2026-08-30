//! ExifTool-style filtering for comparison tools
//!
//! This module provides functions to parse and apply ExifTool-style tag filters
//! like `-EXIF:all`, `-Orientation#`, `-GPS*`, etc.

use crate::types::{FilterOptions, TagRequest};
use serde_json::Value;
use std::collections::HashMap;

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
    let tag_requests = args
        .iter()
        .filter_map(|arg| arg.strip_prefix('-').filter(|rest| !rest.is_empty()))
        .map(TagRequest::parse)
        .collect();

    FilterOptions::from_requests(tag_requests)
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
        assert_eq!(
            filter.tag_requests,
            vec![TagRequest::new("Orientation", true)]
        );
    }

    #[test]
    fn test_parse_glob_pattern() {
        let filter = parse_exiftool_filters(&["-GPS*"]);
        assert!(!filter.extract_all);
        assert_eq!(filter.glob_patterns, vec!["GPS*"]);
    }

    /// `-Group:*` must stay a glob. `should_extract_tag`'s group-all branch matches
    /// the literal "all", so routing `EXIF:*` there would drop every EXIF tag.
    ///
    /// Probed (ExifTool 13.59, test-images/canon/eos_5d_mark_iii.jpg):
    /// `exiftool -j -G "-EXIF:*"` => the whole EXIF group
    #[test]
    fn test_parse_group_qualified_glob_is_not_group_all() {
        let filter = parse_exiftool_filters(&["-EXIF:*"]);
        assert_eq!(filter.glob_patterns, vec!["EXIF:*"]);
        assert!(filter.group_all_patterns.is_empty());
        assert!(filter.should_extract_tag("Make", "EXIF"));
        assert!(!filter.should_extract_tag("MIMEType", "File"));
    }

    /// `--all` is a spelling of `-all`; any other `--TAG` is ExifTool's exclusion
    /// syntax, which we do not implement. Such a request must match nothing rather
    /// than silently become the opposite (inclusion) request.
    ///
    /// Probed (ExifTool 13.59, eos_5d_mark_iii.jpg):
    /// `exiftool -j -G "--GPS*"` => every tag EXCEPT the GPS* ones
    #[test]
    fn test_double_dash_request_is_not_an_inclusion() {
        let excluded = parse_exiftool_filters(&["--GPS*"]);
        assert!(
            !excluded.should_extract_tag("GPSVersionID", "EXIF"),
            "--GPS* must not be read as -GPS*"
        );

        let all_alias = parse_exiftool_filters(&["--all"]);
        assert!(all_alias.extract_all);
    }

    #[test]
    fn test_parse_multiple_filters() {
        let filter = parse_exiftool_filters(&["-EXIF:all", "-GPS*", "-Orientation#"]);
        assert!(!filter.extract_all);
        assert_eq!(filter.group_all_patterns, vec!["EXIF:all"]);
        assert_eq!(filter.glob_patterns, vec!["GPS*"]);
        assert_eq!(filter.requested_tags, vec!["Orientation"]);
        assert_eq!(
            filter.tag_requests,
            vec![
                TagRequest::new("EXIF:all", false),
                TagRequest::new("GPS*", false),
                TagRequest::new("Orientation", true),
            ],
            "the request list keeps command-line order across all request kinds"
        );
    }

    /// ExifTool walks its requests in command-line order and appends each request's
    /// matches to the found-tag list; the JSON writer prints the first entry for a
    /// tag name and skips every later one. So the *first* request that matches a tag
    /// decides whether that tag prints its ValueConv (`#`) or PrintConv value.
    ///
    /// ExifTool: lib/Image/ExifTool.pm:5345-5437 (SetFoundTags), 3266-3290 (GetInfo),
    /// exiftool:2947-2953 (JSON `%noDups`).
    ///
    /// Probed against vendored ExifTool 13.59 with test-images/apple/IMG_3755.MOV:
    ///
    /// ```text
    /// exiftool -j -Duration "-*Duration*#" IMG_3755.MOV  => "Duration": "2.96 s"
    /// exiftool -j "-*Duration*#" -Duration IMG_3755.MOV  => "Duration": 2.965
    /// ```
    #[test]
    fn test_numeric_request_order_first_match_wins() {
        let print_first = parse_exiftool_filters(&["-Duration", "-*Duration*#"]);
        assert!(
            !print_first.should_use_numeric("Duration", "QuickTime"),
            "-Duration comes first, so Duration keeps its PrintConv value"
        );
        assert!(
            print_first.should_use_numeric("TrackDuration", "QuickTime"),
            "TrackDuration is only matched by the numeric wildcard"
        );

        let numeric_first = parse_exiftool_filters(&["-*Duration*#", "-Duration"]);
        assert!(
            numeric_first.should_use_numeric("Duration", "QuickTime"),
            "the numeric wildcard comes first, so Duration prints its ValueConv value"
        );
    }

    /// A request whose group does not match is not a match at all, so the decision
    /// falls through to the next request.
    ///
    /// Probed (ExifTool 13.59, IMG_3755.MOV):
    /// `exiftool -j "-EXIF:Duration#" -Duration` => `"Duration": "2.96 s"`
    /// `exiftool -j "-QuickTime:Duration#" -Duration` => `"Duration": 2.965`
    #[test]
    fn test_numeric_request_order_skips_group_mismatch() {
        let wrong_group = parse_exiftool_filters(&["-EXIF:Duration#", "-Duration"]);
        assert!(
            !wrong_group.should_use_numeric("Duration", "QuickTime"),
            "EXIF:Duration# does not match a QuickTime tag, so -Duration decides"
        );

        let right_group = parse_exiftool_filters(&["-QuickTime:Duration#", "-Duration"]);
        assert!(
            right_group.should_use_numeric("Duration", "QuickTime"),
            "QuickTime:Duration# matches first"
        );
    }

    /// `-all` and `-Group:all` are ordinary requests in ExifTool's list, so they take
    /// part in first-match-wins just like any other pattern.
    ///
    /// Probed (ExifTool 13.59, test-images/canon/eos_5d_mark_iii.jpg):
    /// `exiftool -j -all -Orientation#`      => `"Orientation": "Horizontal (normal)"`
    /// `exiftool -j -Orientation# -all`      => `"Orientation": 1`
    /// `exiftool -j "-EXIF:all" -Orientation#` => `"Orientation": "Horizontal (normal)"`
    /// `exiftool -j -Orientation# "-EXIF:all"` => `"Orientation": 1`
    #[test]
    fn test_all_requests_participate_in_numeric_order() {
        let all_first = parse_exiftool_filters(&["-all", "-Orientation#"]);
        assert!(
            !all_first.should_use_numeric("Orientation", "EXIF"),
            "-all matches every tag first, so Orientation keeps its PrintConv value"
        );

        let numeric_first = parse_exiftool_filters(&["-Orientation#", "-all"]);
        assert!(
            numeric_first.should_use_numeric("Orientation", "EXIF"),
            "-Orientation# matches before -all"
        );

        let group_all_first = parse_exiftool_filters(&["-EXIF:all", "-Orientation#"]);
        assert!(
            !group_all_first.should_use_numeric("Orientation", "EXIF"),
            "-EXIF:all matches the EXIF group first"
        );
        assert!(
            group_all_first.should_use_numeric("Orientation", "MakerNotes"),
            "-EXIF:all does not match a MakerNotes tag, so -Orientation# decides"
        );
    }

    /// `-Group:all#` selects the whole group by value; the `#` must be stripped
    /// before the request is classified, or `EXIF:all` is mistaken for a tag name.
    ///
    /// Probed (ExifTool 13.59, eos_5d_mark_iii.jpg):
    /// `exiftool -j -G "-EXIF:all#"` => 54 EXIF tags, `"EXIF:Orientation": 1`
    #[test]
    fn test_group_all_numeric_request() {
        let filter = parse_exiftool_filters(&["-EXIF:all#"]);
        assert_eq!(
            filter.group_all_patterns,
            vec!["EXIF:all"],
            "the `#` must be stripped before classifying the request"
        );
        assert!(filter.should_extract_tag("Orientation", "EXIF"));
        assert!(filter.should_use_numeric("Orientation", "EXIF"));
        assert!(!filter.should_use_numeric("Duration", "QuickTime"));
    }

    /// `-all#` extracts everything by value.
    ///
    /// Probed (ExifTool 13.59, eos_5d_mark_iii.jpg):
    /// `exiftool -j -all#` => `"Orientation": 1`, `"ExposureTime": 0.004`
    #[test]
    fn test_extract_all_numeric_request() {
        let filter = parse_exiftool_filters(&["-all#"]);
        assert!(
            !filter.extract_all,
            "-all# must stay a filtered request so the numeric override still runs"
        );
        assert!(filter.should_extract_tag("Orientation", "EXIF"));
        assert!(filter.should_extract_tag("MIMEType", "File"));
        assert!(filter.should_use_numeric("Orientation", "EXIF"));
        assert!(filter.should_use_numeric("ExposureTime", "EXIF"));
    }

    /// `-all#` mixed with an exact tag request: the exact request still wins when it
    /// comes first, and only for the tag it names.
    ///
    /// Probed (ExifTool 13.59, test-images/canon/eos_rebel_t3i.jpg):
    /// `exiftool -j -G -Orientation -all#`
    ///   => `"EXIF:Orientation": "Rotate 270 CW"`, `"EXIF:ExposureTime": 0.0005`
    /// `exiftool -j -G -all# -Orientation` => `"EXIF:Orientation": 8`
    /// `exiftool -j -G -Orientation -Orientation#` => `"EXIF:Orientation": "Rotate 270 CW"`
    /// `exiftool -j -G -Orientation# -Orientation` => `"EXIF:Orientation": 8`
    #[test]
    fn test_extract_all_numeric_mixed_with_exact_request() {
        let exact_first = parse_exiftool_filters(&["-Orientation", "-all#"]);
        assert!(exact_first.should_extract_tag("MIMEType", "File"));
        assert!(
            !exact_first.should_use_numeric("Orientation", "EXIF"),
            "-Orientation matched before -all#"
        );
        assert!(
            exact_first.should_use_numeric("ExposureTime", "EXIF"),
            "every other tag is only matched by -all#"
        );

        let all_first = parse_exiftool_filters(&["-all#", "-Orientation"]);
        assert!(all_first.should_use_numeric("Orientation", "EXIF"));

        // Same tag requested twice, differing only in the `#`
        let print_first = parse_exiftool_filters(&["-Orientation", "-Orientation#"]);
        assert!(!print_first.should_use_numeric("Orientation", "EXIF"));
        let numeric_first = parse_exiftool_filters(&["-Orientation#", "-Orientation"]);
        assert!(numeric_first.should_use_numeric("Orientation", "EXIF"));
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
