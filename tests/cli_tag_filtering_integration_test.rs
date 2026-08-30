//! CLI Tag Filtering Integration Tests
//!
//! Comprehensive test suite for CLI tag filtering and PrintConv/ValueConv control
//! Tests all features against real image files to ensure ExifTool compatibility

use exif_oxide::formats::extract_metadata;
use exif_oxide::types::{FilterOptions, TagValue};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

const TEST_IMAGE_CANON: &str = "test-images/canon/eos_rebel_t3i.jpg";
const TEST_IMAGE_RICOH: &str = "third-party/exiftool/t/images/Ricoh2.jpg";
/// Has an EXIF FileSource tag, which a `File*` *name* pattern also selects.
const TEST_IMAGE_NIKON: &str = "test-images/nikon/d3000.jpg";

/// Resolve a repo-relative test asset, searching the crate directory and its ancestors.
///
/// A `git worktree` created under `<repo>/.claude/worktrees/<name>` carries neither the
/// untracked `test-images/` tree nor a populated `third-party/exiftool` submodule, so the
/// asset only exists in the primary checkout further up the path. Returns `None` when the
/// asset is missing entirely, letting a test skip instead of panicking.
fn find_test_asset(relative: &str) -> Option<PathBuf> {
    let mut dir = Some(Path::new(env!("CARGO_MANIFEST_DIR")));
    while let Some(current) = dir {
        let candidate = current.join(relative);
        if candidate.exists() {
            return Some(candidate);
        }
        dir = current.parent();
    }
    None
}

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
        ..Default::default()
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
        ..Default::default()
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
        ..Default::default()
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
        ..Default::default()
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
        ..Default::default()
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
        ..Default::default()
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
        ..Default::default()
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
        ..Default::default()
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
        ..Default::default()
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
        ..Default::default()
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
        ..Default::default()
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
        ..Default::default()
    };

    let result = extract_metadata(Path::new(TEST_IMAGE_RICOH), false, false, Some(filter)).unwrap();

    // Should contain GPS tags
    assert!(result.tags.len() > 5, "Expected GPS tags");

    // All GPS tags should have appropriate values
    for tag in &result.tags {
        assert!(tag.name.starts_with("GPS"), "Non-GPS tag: {}", tag.name);
    }

    // The `#` applies to every tag the wildcard matches, not just an exact name.
    // ExifTool: lib/Image/ExifTool.pm:5364-5382 (SetFoundTags)
    // Reference: `exiftool -j -G "-GPS*#" Ricoh2.jpg` => "EXIF:GPSAltitudeRef": 0
    let altitude_ref = result
        .tags
        .iter()
        .find(|t| t.name == "GPSAltitudeRef")
        .expect("GPSAltitudeRef should be extracted by the GPS* pattern");
    match &altitude_ref.print {
        TagValue::U8(0) | TagValue::U16(0) => (),
        other => panic!("Expected numeric GPSAltitudeRef 0, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// CLI tag-request parity gaps (_todo/20260830-P2-cli-tag-request-parity.md),
// items 2, 4 and 5. Every expectation below was probed against vendored
// ExifTool 13.59 (third-party/exiftool/exiftool) before the test was written.
// ---------------------------------------------------------------------------

/// Item 2: a family-1 group name selects tags by their Group1 (subdirectory).
///
/// ExifTool hands the group portion of a request to `GroupMatches`
/// (lib/Image/ExifTool.pm:5398-5401), which compares a bare group name against
/// *every* group family returned by `GetGroup($tag, -1)`
/// (lib/Image/ExifTool.pm:5237-5253). "ExifIFD" therefore matches on family 1,
/// exactly as "EXIF" matches on family 0.
///
/// Probes (vendored ExifTool 13.59, canon/eos_rebel_t3i.jpg):
///   exiftool -j -G  "-ExifIFD:FNum?er#" => {"EXIF:FNumber": 4}
///   exiftool -j -G1 -FNumber -Make      => {"ExifIFD:FNumber": 4.0, "IFD0:Make": "Canon"}
#[test]
fn test_family1_group_request_selects_by_group1() {
    let Some(image) = find_test_asset(TEST_IMAGE_CANON) else {
        eprintln!("skipping: {TEST_IMAGE_CANON} not available");
        return;
    };

    let mut numeric_tags = HashSet::new();
    numeric_tags.insert("ExifIFD:FNum?er".to_string());
    let filter = FilterOptions {
        extract_all: false,
        glob_patterns: vec!["ExifIFD:FNum?er".to_string()],
        numeric_tags,
        ..Default::default()
    };

    let result = extract_metadata(&image, false, false, Some(filter)).unwrap();

    let names: Vec<&str> = result.tags.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(
        names,
        vec!["FNumber"],
        "-ExifIFD:FNum?er# must select the ExifIFD FNumber and nothing else"
    );
    let fnumber = &result.tags[0];
    assert_eq!(fnumber.group, "EXIF");
    assert_eq!(fnumber.group1, "ExifIFD");
    assert_eq!(
        fnumber.print, fnumber.value,
        "the `#` suffix must select the ValueConv result"
    );
}

/// Item 2: a family-1 group name that does not hold the tag matches nothing,
/// while the correct family-1 name does.
///
/// Probes (vendored ExifTool 13.59, canon/eos_rebel_t3i.jpg):
///   exiftool -j -G "-ExifIFD:Make" => no tags (Make lives in IFD0)
///   exiftool -j -G "-IFD0:Make"    => {"EXIF:Make": "Canon"}
#[test]
fn test_family1_group_request_rejects_wrong_subdirectory() {
    let Some(image) = find_test_asset(TEST_IMAGE_CANON) else {
        eprintln!("skipping: {TEST_IMAGE_CANON} not available");
        return;
    };

    let wrong_ifd = FilterOptions {
        extract_all: false,
        requested_tags: vec!["ExifIFD:Make".to_string()],
        ..Default::default()
    };
    let result = extract_metadata(&image, false, false, Some(wrong_ifd)).unwrap();
    assert!(
        result.tags.is_empty(),
        "Make is in IFD0, so -ExifIFD:Make must match nothing, got {:?}",
        result.tags.iter().map(|t| &t.name).collect::<Vec<_>>()
    );

    let right_ifd = FilterOptions {
        extract_all: false,
        requested_tags: vec!["IFD0:Make".to_string()],
        ..Default::default()
    };
    let result = extract_metadata(&image, false, false, Some(right_ifd)).unwrap();
    let names: Vec<&str> = result.tags.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, vec!["Make"], "-IFD0:Make must select EXIF:Make");
}

/// Item 4: `-all#` requests every tag with its ValueConv value.
///
/// ExifTool: lib/Image/ExifTool.pm:5367-5368 - a tag name of `*` or `all`
/// (case-insensitive) matches every extracted tag; :5364 strips the `#` first.
///
/// Probes (vendored ExifTool 13.59, canon/eos_rebel_t3i.jpg):
///   exiftool -j -G "-all#" => same 329 output lines as an unfiltered run, with
///                             "EXIF:Orientation": 8 (not "Rotate 270 CW")
///   exiftool -j -G "-ALL#" => identical (the match is case-insensitive)
#[test]
fn test_all_numeric_request_returns_every_tag_numerically() {
    let Some(image) = find_test_asset(TEST_IMAGE_CANON) else {
        eprintln!("skipping: {TEST_IMAGE_CANON} not available");
        return;
    };

    let unfiltered = extract_metadata(&image, false, false, None).unwrap();

    let mut numeric_tags = HashSet::new();
    numeric_tags.insert("all".to_string());
    let filter = FilterOptions {
        extract_all: false,
        requested_tags: vec!["all".to_string()],
        numeric_tags,
        ..Default::default()
    };
    let result = extract_metadata(&image, false, false, Some(filter)).unwrap();

    assert_eq!(
        result.tags.len(),
        unfiltered.tags.len(),
        "-all# must return the same tags as an unfiltered extraction"
    );

    let orientation = result
        .tags
        .iter()
        .find(|t| t.name == "Orientation")
        .expect("-all# must include EXIF:Orientation");
    match &orientation.print {
        TagValue::U8(8) | TagValue::U16(8) => (),
        other => panic!("-all# must give Orientation its numeric value 8, got {other:?}"),
    }
}

/// Item 4: `-*#` is the same request as `-all#`.
///
/// ExifTool: lib/Image/ExifTool.pm:5367 matches `/^(\*|all)$/i`.
///
/// Probe (vendored ExifTool 13.59, canon/eos_rebel_t3i.jpg):
///   exiftool -j -G "-*#" => byte-identical to `exiftool -j -G "-all#"`
#[test]
fn test_star_numeric_request_returns_every_tag_numerically() {
    let Some(image) = find_test_asset(TEST_IMAGE_CANON) else {
        eprintln!("skipping: {TEST_IMAGE_CANON} not available");
        return;
    };

    let unfiltered = extract_metadata(&image, false, false, None).unwrap();

    let mut numeric_tags = HashSet::new();
    numeric_tags.insert("*".to_string());
    let filter = FilterOptions {
        extract_all: false,
        glob_patterns: vec!["*".to_string()],
        numeric_tags,
        ..Default::default()
    };
    let result = extract_metadata(&image, false, false, Some(filter)).unwrap();

    assert_eq!(
        result.tags.len(),
        unfiltered.tags.len(),
        "-*# must return the same tags as an unfiltered extraction"
    );

    let orientation = result
        .tags
        .iter()
        .find(|t| t.name == "Orientation")
        .expect("-*# must include EXIF:Orientation");
    match &orientation.print {
        TagValue::U8(8) | TagValue::U16(8) => (),
        other => panic!("-*# must give Orientation its numeric value 8, got {other:?}"),
    }
}

/// Item 5: characters outside `[-\w*?]` are deleted from the tag portion of a
/// request before it is matched.
///
/// ExifTool: lib/Image/ExifTool.pm:5378 (`$tag =~ tr/-_A-Za-z0-9*?//dc;`) for
/// wildcard requests and :5386 (`tr/-_A-Za-z0-9//dc`) for plain ones. The `-j`
/// option turns on Duplicates (exiftool:949), so :5386 - not the "Invalid tag
/// name" branch at :5396 - is the one the JSON CLI reaches.
///
/// Probes (vendored ExifTool 13.59, canon/eos_rebel_t3i.jpg). Each warns
/// `Invalid TAG name` from exiftool:1445 and then matches anyway:
///   exiftool -j -G "-F*Num.ber" => EXIF:FNumber, MakerNotes:FlashGuideNumber,
///                                  MakerNotes:FNumber, Composite:FileNumber
///   exiftool -j -G "-FNum.ber"  => EXIF:FNumber, MakerNotes:FNumber
///   exiftool -j -G "-EX.IF:FNumber" => no tags, plus
///                                  "Warning: Invalid group name 'EX.IF'"
#[test]
fn test_illegal_characters_are_stripped_from_tag_requests() {
    let Some(image) = find_test_asset(TEST_IMAGE_CANON) else {
        eprintln!("skipping: {TEST_IMAGE_CANON} not available");
        return;
    };

    // Wildcard request with an illegal '.' - sterilized to "F*Number".
    let wildcard = FilterOptions {
        extract_all: false,
        glob_patterns: vec!["F*Num.ber".to_string()],
        ..Default::default()
    };
    let result = extract_metadata(&image, false, false, Some(wildcard)).unwrap();
    assert!(
        result.tags.iter().any(|t| t.name == "FNumber"),
        "-F*Num.ber must sterilize to F*Number and match FNumber, got {:?}",
        result.tags.iter().map(|t| &t.name).collect::<Vec<_>>()
    );

    // Plain request with an illegal '.' - sterilized to "FNumber".
    let plain = FilterOptions {
        extract_all: false,
        requested_tags: vec!["FNum.ber".to_string()],
        ..Default::default()
    };
    let result = extract_metadata(&image, false, false, Some(plain)).unwrap();
    let names: Vec<&str> = result.tags.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, vec!["FNumber"], "-FNum.ber must match FNumber");

    // An illegal character in the *group* portion is not sterilized: ExifTool
    // warns and replaces the group with 'invalid', which matches nothing.
    let bad_group = FilterOptions {
        extract_all: false,
        requested_tags: vec!["EX.IF:FNumber".to_string()],
        ..Default::default()
    };
    let result = extract_metadata(&image, false, false, Some(bad_group)).unwrap();
    assert!(
        result.tags.is_empty(),
        "-EX.IF:FNumber names an invalid group and must match nothing, got {:?}",
        result.tags.iter().map(|t| &t.name).collect::<Vec<_>>()
    );
}

/// A bare `File*` request is a tag-NAME pattern, not a group request, so it must not
/// trigger the File-group shortcut that skips EXIF parsing.
///
/// ExifTool matches a wildcard request against every extracted tag name regardless of
/// group (lib/Image/ExifTool.pm:5376-5382), which is why `File*` also returns the EXIF
/// FileSource tag. Probed with vendored ExifTool 13.59 on 2026-08-30:
///
/// ```console
/// $ third-party/exiftool/exiftool -j -G "-File*#" test-images/nikon/d3000.jpg
/// [{
///   "SourceFile": "test-images/nikon/d3000.jpg",
///   "File:FileName": "d3000.jpg",
///   "File:FileSize": 3043468,
///   "File:FileModifyDate": "...",
///   "File:FileAccessDate": "...",
///   "File:FileInodeChangeDate": "...",
///   "File:FilePermissions": 100664,
///   "File:FileType": "JPEG",
///   "File:FileTypeExtension": "JPG",
///   "EXIF:FileSource": 3
/// }]
/// ```
#[test]
fn test_bare_file_name_pattern_reaches_exif_tags() {
    let Some(image) = find_test_asset(TEST_IMAGE_NIKON) else {
        eprintln!("skipping: {TEST_IMAGE_NIKON} not available");
        return;
    };

    let mut numeric_tags = HashSet::new();
    numeric_tags.insert("File*".to_string());

    let filter = FilterOptions {
        extract_all: false,
        glob_patterns: vec!["File*".to_string()],
        numeric_tags,
        ..Default::default()
    };

    let result = extract_metadata(&image, false, false, Some(filter)).unwrap();

    assert!(
        result
            .tags
            .iter()
            .any(|t| t.group == "EXIF" && t.name == "FileSource"),
        "EXIF:FileSource should match the File* name pattern, got: {:?}",
        result
            .tags
            .iter()
            .map(|t| format!("{}:{}", t.group, t.name))
            .collect::<Vec<_>>()
    );

    // The File group tags ExifTool returns for the same request are still there,
    // including FileInodeChangeDate, which the stat-only shortcut never emitted.
    for expected in [
        "FileName",
        "FileSize",
        "FileModifyDate",
        "FileAccessDate",
        "FilePermissions",
        "FileType",
        "FileTypeExtension",
    ] {
        assert!(
            result
                .tags
                .iter()
                .any(|t| t.group == "File" && t.name == expected),
            "File:{expected} missing from -File* output"
        );
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    assert!(
        result
            .tags
            .iter()
            .any(|t| t.group == "File" && t.name == "FileInodeChangeDate"),
        "File:FileInodeChangeDate missing from -File* output"
    );

    // ExifTool's wildcards only ever apply to the tag name, so nothing whose name
    // fails to start with "File" may appear - notably no Directory or MIMEType.
    for tag in &result.tags {
        assert!(
            tag.name.to_lowercase().starts_with("file"),
            "{}:{} does not match the File* name pattern",
            tag.group,
            tag.name
        );
    }
}

/// The stat-only shortcut is a deliberate performance optimization: a request pinned
/// to the File *group* must still take it rather than parsing the whole file.
///
/// ```console
/// $ third-party/exiftool/exiftool -j -G "-File:all" test-images/nikon/d3000.jpg
/// [{ "File:FileName": ..., "File:MIMEType": "image/jpeg", ... }]   # File group only
/// ```
/// (vendored ExifTool 13.59, probed 2026-08-30)
#[test]
fn test_file_group_qualified_requests_keep_fast_path() {
    let group_all = FilterOptions {
        extract_all: false,
        group_all_patterns: vec!["File:all".to_string()],
        ..Default::default()
    };
    assert!(
        group_all.is_file_group_only(),
        "-File:all must keep the stat-only shortcut"
    );

    let qualified_glob = FilterOptions {
        extract_all: false,
        glob_patterns: vec!["File:*".to_string()],
        ..Default::default()
    };
    assert!(
        qualified_glob.is_file_group_only(),
        "-File:* must keep the stat-only shortcut"
    );

    let Some(image) = find_test_asset(TEST_IMAGE_NIKON) else {
        eprintln!("skipping: {TEST_IMAGE_NIKON} not available");
        return;
    };
    let result = extract_metadata(&image, false, false, Some(group_all)).unwrap();
    assert!(
        result.tags.iter().all(|t| t.group == "File"),
        "-File:all returned a non-File tag: {:?}",
        result
            .tags
            .iter()
            .find(|t| t.group != "File")
            .map(|t| format!("{}:{}", t.group, t.name))
    );
    assert!(
        result.tags.iter().any(|t| t.name == "MIMEType"),
        "-File:all should include File:MIMEType"
    );

    // The shortcut used to skip FileInodeChangeDate, so `-File:all` returned one tag
    // fewer than a full parse of the same file and one fewer than ExifTool.
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    assert!(
        result.tags.iter().any(|t| t.name == "FileInodeChangeDate"),
        "-File:all should include File:FileInodeChangeDate"
    );

    // Known gap, not asserted here: ExifTool's -File:all also returns the File group
    // tags that come from parsing the image (ImageWidth, ImageHeight, BitsPerSample,
    // ColorComponents, EncodingProcess, YCbCrSubSampling, ExifByteOrder,
    // CurrentIPTCDigest). The shortcut is stat-only by design and omits them.
}

/// The shortcut must return exactly what a full parse would return for the same tag
/// names - that is the only thing that makes skipping the parse safe.
#[test]
fn test_fast_path_system_tags_match_full_parse() {
    let file_tags: Vec<String> = [
        "FileName",
        "Directory",
        "FileSize",
        "FileModifyDate",
        "FileAccessDate",
        "FileInodeChangeDate",
        "FileCreateDate",
        "FilePermissions",
        "FileType",
        "FileTypeExtension",
        "MIMEType",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    let Some(path) = find_test_asset(TEST_IMAGE_NIKON) else {
        eprintln!("skipping: {TEST_IMAGE_NIKON} not available");
        return;
    };
    let describe = |filter: FilterOptions| -> Vec<String> {
        let mut described: Vec<String> = extract_metadata(&path, false, false, Some(filter))
            .unwrap()
            .tags
            .iter()
            .filter(|t| t.group == "File")
            .map(|t| format!("{}/{}={:?}/{:?}", t.group1, t.name, t.value, t.print))
            .collect();
        described.sort();
        described
    };

    // Plain requests, and the same requests with `#` - the numeric transformation has
    // to happen on both paths too.
    for numeric_tags in [
        HashSet::new(),
        HashSet::from(["FilePermissions".to_string(), "FileSize".to_string()]),
    ] {
        let shortcut = FilterOptions {
            requested_tags: file_tags.clone(),
            extract_all: false,
            numeric_tags: numeric_tags.clone(),
            ..Default::default()
        };
        assert!(
            shortcut.is_file_group_only(),
            "exact File tag names should take the shortcut"
        );

        // Adding one EXIF tag forces the full parse without changing which File tags match.
        let mut with_exif = file_tags.clone();
        with_exif.push("Orientation".to_string());
        let full = FilterOptions {
            requested_tags: with_exif,
            extract_all: false,
            numeric_tags: numeric_tags.clone(),
            ..Default::default()
        };
        assert!(!full.is_file_group_only());

        assert_eq!(
            describe(shortcut),
            describe(full),
            "the File-only shortcut disagrees with a full parse (numeric: {numeric_tags:?})"
        );
    }
}

/// FilePermissions carries two different values: ExifTool's ValueConv is the octal
/// file mode (`sprintf("%.3o", $val)`) and its PrintConv is the `rwx` string.
/// ExifTool: lib/Image/ExifTool.pm:1505-1536
///
/// ```console
/// $ third-party/exiftool/exiftool -j -G "-FilePermissions#" test-images/nikon/d3000.jpg
/// [{ "File:FilePermissions": 100664 }]
/// $ third-party/exiftool/exiftool -j -G -FilePermissions test-images/nikon/d3000.jpg
/// [{ "File:FilePermissions": "-rw-rw-r--" }]
/// ```
/// (vendored ExifTool 13.59, probed 2026-08-30)
#[cfg(unix)]
#[test]
fn test_file_permissions_value_is_octal_mode() {
    use std::os::unix::fs::PermissionsExt;

    let Some(path) = find_test_asset(TEST_IMAGE_NIKON) else {
        eprintln!("skipping: {TEST_IMAGE_NIKON} not available");
        return;
    };
    let mode = std::fs::metadata(&path).unwrap().permissions().mode();
    let expected_value = TagValue::U64(format!("{mode:03o}").parse::<u64>().unwrap());

    // The stat-only path (-File:all) and the full parse (-File*) must agree.
    let fast_path = FilterOptions {
        extract_all: false,
        group_all_patterns: vec!["File:all".to_string()],
        ..Default::default()
    };
    let full_path = FilterOptions {
        extract_all: false,
        glob_patterns: vec!["File*".to_string()],
        ..Default::default()
    };

    for (label, filter) in [("File:all", fast_path), ("File*", full_path)] {
        let result = extract_metadata(&path, false, false, Some(filter)).unwrap();
        let entry = result
            .tags
            .iter()
            .find(|t| t.name == "FilePermissions")
            .unwrap_or_else(|| panic!("-{label} should return FilePermissions"));

        assert_eq!(
            entry.value, expected_value,
            "-{label}: FilePermissions value should be the octal mode"
        );
        match &entry.print {
            TagValue::String(s) => assert_eq!(
                s.len(),
                10,
                "-{label}: FilePermissions print should be the rwx string, got {s:?}"
            ),
            other => panic!("-{label}: expected an rwx string, got {other:?}"),
        }
    }
}
