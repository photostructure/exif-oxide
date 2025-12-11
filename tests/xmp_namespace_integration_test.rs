//! Integration tests for XMP namespace tag extraction
//!
//! Tests that XMP2.pl and MWG.pm namespace tables are correctly generated
//! and that tags from these namespaces are properly extracted at runtime.
//!
//! TPP: docs/todo/P03g-xmp2-mwg-codegen.md

#![cfg(feature = "integration-tests")]

use exif_oxide::formats;
use std::path::Path;

/// Test Creative Commons (cc) namespace tag extraction
/// Source: XMP2.pl
#[test]
fn test_cc_namespace_attribution_name() {
    let path = Path::new("third-party/exiftool/t/images/XMP3.xmp");
    if !path.exists() {
        eprintln!("Test file not found: {:?}", path);
        return;
    }

    let result = formats::extract_metadata(path, false, false, None);
    assert!(
        result.is_ok(),
        "Failed to extract metadata: {:?}",
        result.err()
    );

    let exif_data = result.unwrap();
    let entries = &exif_data.tags;

    // Find AttributionName tag from cc namespace
    let attribution = entries
        .iter()
        .find(|e| e.name == "AttributionName" || e.name.contains("Attribution"));

    assert!(
        attribution.is_some(),
        "AttributionName tag should be extracted from cc namespace. Available XMP tags: {:?}",
        entries
            .iter()
            .filter(|e| e.group.starts_with("XMP"))
            .map(|e| format!("{}:{}", e.group, e.name))
            .collect::<Vec<_>>()
    );

    // Verify the value matches ExifTool output
    let attr = attribution.unwrap();
    let value_str = format!("{}", attr.value);
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
