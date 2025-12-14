//! Integration tests for XMP namespace tag extraction
//!
//! Tests that XMP2.pl and MWG.pm namespace tables are correctly generated
//! and that tags from these namespaces are properly extracted at runtime.
//!
//! TPP: docs/todo/P03d-unknown-tags-research.md - Extraction tests for all researched tags

#![cfg(feature = "integration-tests")]

use exif_oxide::formats;
use std::path::Path;

use exif_oxide::types::TagEntry;

/// Helper to find a tag by name in extracted metadata
fn find_tag<'a>(tags: &'a [TagEntry], name: &str) -> Option<&'a TagEntry> {
    tags.iter().find(|e| e.name == name)
}

/// Helper to extract metadata from a path, panicking on failure
fn extract_or_panic(path: &Path) -> exif_oxide::ExifData {
    if !path.exists() {
        panic!("Test file not found: {:?}", path);
    }
    formats::extract_metadata(path, false, false, None)
        .unwrap_or_else(|e| panic!("Failed to extract metadata from {:?}: {:?}", path, e))
}

// =============================================================================
// Creative Commons (CC) Namespace Tags - XMP2.pl
// =============================================================================

/// Test all Creative Commons tags extraction from test-resources/cc-license-tags.xmp
/// Tags: License, AttributionName, AttributionURL, UseGuidelines, Jurisdiction, Permits, Requires, Prohibits
///
/// BLOCKED: XMP parser doesn't extract rdf:resource attributes (License, AttributionURL, etc.)
/// or Bag structures with resource items (Permits, Requires, Prohibits)
#[test]
#[ignore = "XMP parser needs rdf:resource attribute and Bag structure support"]
fn test_cc_namespace_all_tags_extraction() {
    let path = Path::new("test-resources/cc-license-tags.xmp");
    let data = extract_or_panic(path);

    // License
    let license = find_tag(&data.tags, "License");
    assert!(license.is_some(), "License tag should be extracted");
    assert!(
        format!("{}", license.unwrap().value).contains("creativecommons.org"),
        "License should contain CC URL"
    );

    // AttributionName
    let attr_name = find_tag(&data.tags, "AttributionName");
    assert!(
        attr_name.is_some(),
        "AttributionName tag should be extracted"
    );
    assert_eq!(
        format!("{}", attr_name.unwrap().value),
        "Test Author",
        "AttributionName value mismatch"
    );

    // AttributionURL
    let attr_url = find_tag(&data.tags, "AttributionURL");
    assert!(attr_url.is_some(), "AttributionURL tag should be extracted");
    assert!(
        format!("{}", attr_url.unwrap().value).contains("example.com/author"),
        "AttributionURL should contain expected URL"
    );

    // UseGuidelines
    let guidelines = find_tag(&data.tags, "UseGuidelines");
    assert!(
        guidelines.is_some(),
        "UseGuidelines tag should be extracted"
    );
    assert!(
        format!("{}", guidelines.unwrap().value).contains("example.com/guidelines"),
        "UseGuidelines should contain expected URL"
    );

    // Jurisdiction
    let jurisdiction = find_tag(&data.tags, "Jurisdiction");
    assert!(
        jurisdiction.is_some(),
        "Jurisdiction tag should be extracted"
    );
    assert!(
        format!("{}", jurisdiction.unwrap().value).contains("creativecommons.org"),
        "Jurisdiction should contain CC URL"
    );

    // Permits (Bag type)
    let permits = find_tag(&data.tags, "Permits");
    assert!(permits.is_some(), "Permits tag should be extracted");
    let permits_val = format!("{}", permits.unwrap().value);
    assert!(
        permits_val.contains("Reproduction") || permits_val.contains("Distribution"),
        "Permits should contain license permissions, got: {}",
        permits_val
    );

    // Requires (Bag type)
    let requires = find_tag(&data.tags, "Requires");
    assert!(requires.is_some(), "Requires tag should be extracted");
    let requires_val = format!("{}", requires.unwrap().value);
    assert!(
        requires_val.contains("Attribution") || requires_val.contains("Share"),
        "Requires should contain license requirements, got: {}",
        requires_val
    );

    // Prohibits (Bag type)
    let prohibits = find_tag(&data.tags, "Prohibits");
    assert!(prohibits.is_some(), "Prohibits tag should be extracted");
    let prohibits_val = format!("{}", prohibits.unwrap().value);
    assert!(
        prohibits_val.contains("Commercial"),
        "Prohibits should contain commercial use restriction, got: {}",
        prohibits_val
    );
}

/// Test Creative Commons (cc) namespace tag extraction from ExifTool test file
/// Source: XMP2.pl
#[test]
fn test_cc_namespace_attribution_name() {
    let path = Path::new("third-party/exiftool/t/images/XMP3.xmp");
    let data = extract_or_panic(path);

    // Find AttributionName tag from cc namespace
    let attribution = find_tag(&data.tags, "AttributionName");

    assert!(
        attribution.is_some(),
        "AttributionName tag should be extracted from cc namespace. Available XMP tags: {:?}",
        data.tags
            .iter()
            .filter(|e| e.group.starts_with("XMP"))
            .map(|e| format!("{}:{}", e.group, e.name))
            .collect::<Vec<_>>()
    );

    // Verify the value matches ExifTool output
    let value_str = format!("{}", attribution.unwrap().value);
    assert!(
        value_str.contains("some attr"),
        "AttributionName should contain 'some attr', got: {}",
        value_str
    );
}

/// Test that xmp_lookup returns correct tag info for cc namespace
#[test]
fn test_cc_namespace_lookup_works() {
    use exif_oxide::xmp::xmp_lookup::lookup_xmp_tag;

    // License tag
    let license = lookup_xmp_tag("cc", "license");
    assert!(license.is_some(), "cc:license should be in lookup tables");
    assert_eq!(license.unwrap().name, "License");

    // AttributionName tag
    let attr = lookup_xmp_tag("cc", "attributionName");
    assert!(
        attr.is_some(),
        "cc:attributionName should be in lookup tables"
    );
    assert_eq!(attr.unwrap().name, "AttributionName");

    // Permits tag (has PrintConv)
    let permits = lookup_xmp_tag("cc", "permits");
    assert!(permits.is_some(), "cc:permits should be in lookup tables");
    assert!(
        permits.unwrap().print_conv.is_some(),
        "cc:permits should have PrintConv"
    );
}

/// Test mediapro namespace lookup
#[test]
fn test_mediapro_namespace_lookup_works() {
    use exif_oxide::xmp::xmp_lookup::lookup_xmp_tag;

    let people = lookup_xmp_tag("mediapro", "People");
    assert!(
        people.is_some(),
        "mediapro:People should be in lookup tables"
    );
    assert_eq!(people.unwrap().name, "People");
}

/// Test Iptc4xmpExt namespace lookup
#[test]
fn test_iptc_ext_namespace_lookup_works() {
    use exif_oxide::xmp::xmp_lookup::lookup_xmp_tag;

    let person = lookup_xmp_tag("Iptc4xmpExt", "PersonInImage");
    assert!(
        person.is_some(),
        "Iptc4xmpExt:PersonInImage should be in lookup tables"
    );
    assert_eq!(person.unwrap().name, "PersonInImage");

    // Also test lowercase variant
    let person_lower = lookup_xmp_tag("iptc4xmpExt", "PersonInImage");
    assert!(
        person_lower.is_some(),
        "iptc4xmpExt (lowercase) should also work"
    );
}

/// Test MWG namespace lookups
#[test]
fn test_mwg_namespace_lookup_works() {
    use exif_oxide::xmp::xmp_lookup::lookup_xmp_tag;

    // MWG Regions (mwg-rs)
    let regions = lookup_xmp_tag("mwg-rs", "RegionsRegionList");
    assert!(
        regions.is_some(),
        "mwg-rs:RegionsRegionList should be in lookup tables"
    );
    assert_eq!(regions.unwrap().name, "RegionList");

    // MWG Keywords (mwg-kw)
    let keywords = lookup_xmp_tag("mwg-kw", "Keywords");
    assert!(
        keywords.is_some(),
        "mwg-kw:Keywords should be in lookup tables"
    );
    assert_eq!(keywords.unwrap().name, "KeywordInfo");
}

// =============================================================================
// MWG Region and Keyword Tags - MWG.pm
// =============================================================================

/// Test MWG Region data extraction from XMP5.xmp
/// With struct flattening, we get RegionsRegionListName, RegionsRegionListType, etc.
/// Note: ExifTool uses FlatName="Region" to shorten to RegionName, RegionType
/// which requires additional lookup table entries (future enhancement)
#[test]
fn test_mwg_region_data_extraction() {
    let path = Path::new("third-party/exiftool/t/images/XMP5.xmp");
    let data = extract_or_panic(path);

    // With struct flattening, we get the full path name (FlatName not yet implemented)
    let region_name = find_tag(&data.tags, "RegionsRegionListName");
    assert!(
        region_name.is_some(),
        "RegionsRegionListName tag should be extracted. Available Region tags: {:?}",
        data.tags
            .iter()
            .filter(|e| e.name.contains("Region"))
            .map(|e| &e.name)
            .collect::<Vec<_>>()
    );

    let name_val = format!("{}", region_name.unwrap().value);
    assert!(
        name_val.contains("Region 1"),
        "RegionsRegionListName should contain 'Region 1', got: {}",
        name_val
    );

    let region_type = find_tag(&data.tags, "RegionsRegionListType");
    assert!(
        region_type.is_some(),
        "RegionsRegionListType should be extracted"
    );
    let type_val = format!("{}", region_type.unwrap().value);
    assert!(
        type_val.contains("Face"),
        "RegionsRegionListType should contain 'Face', got: {}",
        type_val
    );
}

/// Test ExifTool-compatible MWG Region tag names
/// ExifTool flattens mwg-rs:Regions/RegionList/Name to XMP:RegionName
///
/// BLOCKED: Requires XMP struct flattening with FlatName support
/// See: third-party/exiftool/doc/concepts/XMP_STRUCT_FLATTENING.md (to be created)
#[test]
#[ignore = "XMP struct flattening needed - ExifTool uses GetXMPTagID() + AddFlattenedTags()"]
fn test_mwg_region_exiftool_compatible_names() {
    let path = Path::new("third-party/exiftool/t/images/XMP5.xmp");
    let data = extract_or_panic(path);

    // ExifTool extracts these as flattened tags:
    let region_name = find_tag(&data.tags, "RegionName");
    assert!(
        region_name.is_some(),
        "RegionName should be extracted (ExifTool-compatible)"
    );
    assert_eq!(format!("{}", region_name.unwrap().value), "Region 1");

    let region_type = find_tag(&data.tags, "RegionType");
    assert!(
        region_type.is_some(),
        "RegionType should be extracted (ExifTool-compatible)"
    );
    assert_eq!(format!("{}", region_type.unwrap().value), "Face");
}

/// Test MWG keyword data extraction from XMP5.xmp
/// With struct flattening, we now get HierarchicalKeywords1, HierarchicalKeywords2, etc.
/// This test is now redundant with test_mwg_hierarchical_keywords_exiftool_compatible_names
/// but kept for compatibility verification
#[test]
fn test_mwg_keyword_data_extraction() {
    let path = Path::new("third-party/exiftool/t/images/XMP5.xmp");
    let data = extract_or_panic(path);

    // With struct flattening, we now get HierarchicalKeywords1 instead of raw Hierarchy
    let hk1 = find_tag(&data.tags, "HierarchicalKeywords1");
    assert!(
        hk1.is_some(),
        "HierarchicalKeywords1 tag should be extracted. Available keyword tags: {:?}",
        data.tags
            .iter()
            .filter(|e| e.name.contains("Keyword") || e.name.contains("Hierarch"))
            .map(|e| &e.name)
            .collect::<Vec<_>>()
    );
    let hk1_val = format!("{}", hk1.unwrap().value);
    assert!(
        hk1_val.contains("A-1") || hk1_val.contains("B-1") || hk1_val.contains("C-1"),
        "HierarchicalKeywords1 should contain level 1 keywords, got: {}",
        hk1_val
    );

    // HierarchicalKeywords2 contains level 2 keywords
    let hk2 = find_tag(&data.tags, "HierarchicalKeywords2");
    assert!(hk2.is_some(), "HierarchicalKeywords2 should be extracted");
    let hk2_val = format!("{}", hk2.unwrap().value);
    assert!(
        hk2_val.contains("A-2") || hk2_val.contains("B-2"),
        "HierarchicalKeywords2 should contain level 2 keywords, got: {}",
        hk2_val
    );
}

/// Test ExifTool-compatible MWG HierarchicalKeywords tag names
/// ExifTool manually pre-defines HierarchicalKeywords1-6 in MWG.pm
/// Now working with struct flattening implementation!
#[test]
fn test_mwg_hierarchical_keywords_exiftool_compatible_names() {
    let path = Path::new("third-party/exiftool/t/images/XMP5.xmp");
    let data = extract_or_panic(path);

    // ExifTool extracts these as pre-defined flattened tags:
    let hk1 = find_tag(&data.tags, "HierarchicalKeywords1");
    assert!(
        hk1.is_some(),
        "HierarchicalKeywords1 should be extracted. Available tags: {:?}",
        data.tags
            .iter()
            .filter(|e| e.name.contains("Keyword") || e.name.contains("Hierarch"))
            .map(|e| &e.name)
            .collect::<Vec<_>>()
    );
    let hk1_val = format!("{}", hk1.unwrap().value);
    assert!(
        hk1_val.contains("A-1") || hk1_val.contains("B-1") || hk1_val.contains("C-1"),
        "HierarchicalKeywords1 should contain level 1 keywords, got: {}",
        hk1_val
    );

    let hk2 = find_tag(&data.tags, "HierarchicalKeywords2");
    assert!(hk2.is_some(), "HierarchicalKeywords2 should be extracted");
    let hk2_val = format!("{}", hk2.unwrap().value);
    assert!(
        hk2_val.contains("A-2") || hk2_val.contains("B-2"),
        "HierarchicalKeywords2 should contain level 2 keywords, got: {}",
        hk2_val
    );
}

// =============================================================================
// IPTC Extension Tags - XMP2.pl
// =============================================================================

/// Test IPTC PersonInImage extraction from test-resources/iptc-person.xmp
/// Tags: PersonInImage
#[test]
fn test_iptc_person_in_image_extraction() {
    let path = Path::new("test-resources/iptc-person.xmp");
    let data = extract_or_panic(path);

    let person = find_tag(&data.tags, "PersonInImage");
    assert!(
        person.is_some(),
        "PersonInImage tag should be extracted. Available IPTC tags: {:?}",
        data.tags
            .iter()
            .filter(|e| e.group.contains("IPTC") || e.name.contains("Person"))
            .map(|e| format!("{}:{}", e.group, e.name))
            .collect::<Vec<_>>()
    );
    let person_val = format!("{}", person.unwrap().value);
    assert!(
        person_val.contains("John Smith") || person_val.contains("Jane Doe"),
        "PersonInImage should contain person names, got: {}",
        person_val
    );
}

// =============================================================================
// XMP Media Management Tags - XMP.pm
// =============================================================================

/// Test History data extraction from test-resources/xmp-history.xmp
/// With struct flattening, we now get HistoryWhen, HistoryAction, etc.
#[test]
fn test_history_data_extraction() {
    let path = Path::new("test-resources/xmp-history.xmp");
    let data = extract_or_panic(path);

    // With struct flattening, History is now extracted as individual flattened tags
    let history_when = find_tag(&data.tags, "HistoryWhen");
    assert!(
        history_when.is_some(),
        "HistoryWhen tag should be extracted. Available History tags: {:?}",
        data.tags
            .iter()
            .filter(|e| e.name.contains("History"))
            .map(|e| format!("{}:{}", e.group, e.name))
            .collect::<Vec<_>>()
    );

    let when_val = format!("{}", history_when.unwrap().value);
    assert!(
        when_val.contains("2024"),
        "HistoryWhen should contain datetime, got: {}",
        when_val
    );

    let history_action = find_tag(&data.tags, "HistoryAction");
    assert!(
        history_action.is_some(),
        "HistoryAction should be extracted"
    );
    let action_val = format!("{}", history_action.unwrap().value);
    assert!(
        action_val.contains("created"),
        "HistoryAction should contain 'created', got: {}",
        action_val
    );
}

/// Test ExifTool-compatible xmpMM:History tag names
/// ExifTool flattens xmpMM:History/stEvt:when to HistoryWhen using GetXMPTagID()
/// Now working with struct flattening implementation!
#[test]
fn test_history_exiftool_compatible_names() {
    let path = Path::new("test-resources/xmp-history.xmp");
    let data = extract_or_panic(path);

    // ExifTool extracts these as flattened tags:
    let history_when = find_tag(&data.tags, "HistoryWhen");
    assert!(
        history_when.is_some(),
        "HistoryWhen should be extracted (ExifTool-compatible)"
    );
    let when_val = format!("{}", history_when.unwrap().value);
    assert!(
        when_val.contains("2024"),
        "HistoryWhen should contain datetime"
    );

    let history_action = find_tag(&data.tags, "HistoryAction");
    assert!(
        history_action.is_some(),
        "HistoryAction should be extracted (ExifTool-compatible)"
    );
    let action_val = format!("{}", history_action.unwrap().value);
    assert!(
        action_val.contains("created"),
        "HistoryAction should contain 'created', got: {}",
        action_val
    );
}

// =============================================================================
// MediaPro Tags - XMP2.pl
// =============================================================================

/// Test People tag extraction from ExifTool.jpg
/// Tags: People (mediapro namespace)
///
/// BLOCKED: mediapro namespace not being extracted from embedded XMP
#[test]
#[ignore = "mediapro namespace extraction needs investigation"]
fn test_people_tag_extraction() {
    let path = Path::new("third-party/exiftool/t/images/ExifTool.jpg");
    let data = extract_or_panic(path);

    let people = find_tag(&data.tags, "People");
    assert!(
        people.is_some(),
        "People tag should be extracted. Available mediapro tags: {:?}",
        data.tags
            .iter()
            .filter(|e| e.group.contains("mediapro") || e.name == "People")
            .map(|e| format!("{}:{}", e.group, e.name))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        format!("{}", people.unwrap().value),
        "Santa",
        "People value mismatch"
    );
}

// =============================================================================
// DNG/EXIF Tags - Exif.pm
// =============================================================================

/// Test DNGLensInfo extraction from DNG.dng
/// Tags: DNGLensInfo (EXIF tag 0xC630)
#[test]
fn test_dng_lens_info_extraction() {
    let path = Path::new("third-party/exiftool/t/images/DNG.dng");
    let data = extract_or_panic(path);

    let dng_lens = find_tag(&data.tags, "DNGLensInfo");
    assert!(
        dng_lens.is_some(),
        "DNGLensInfo tag should be extracted. Available lens tags: {:?}",
        data.tags
            .iter()
            .filter(|e| e.name.contains("Lens"))
            .map(|e| format!("{}:{}", e.group, e.name))
            .collect::<Vec<_>>()
    );
    let lens_val = format!("{}", dng_lens.unwrap().value);
    // ExifTool shows: 18-55mm f/?
    assert!(
        lens_val.contains("18") || lens_val.contains("55"),
        "DNGLensInfo should contain lens focal length info, got: {}",
        lens_val
    );
}
