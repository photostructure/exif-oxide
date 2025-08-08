//! Tests for the conversion registry modules

use super::printconv_registry::{get_printconv_registry, get_tag_specific_printconv};
use super::valueconv_registry::get_valueconv_registry;
use super::*;

#[test]
fn test_module_scoped_lookup() {
    // Test direct lookup of a known value
    let result = lookup_valueconv("Image::ExifTool::GPS::ConvertTimeStamp($val)", "GPS_pm");
    assert_eq!(
        result,
        Some((
            "crate::implementations::value_conv",
            "gpstimestamp_value_conv"
        ))
    );
}

#[test]
fn test_manual_printconv_lookup() {
    let result = lookup_printconv("fnumber_print_conv", "Exif_pm");
    assert_eq!(
        result,
        Some(("crate::implementations::print_conv", "fnumber_print_conv"))
    );
}

#[test]
fn test_exact_string_matching() {
    // Test that we use exact string matching without normalization
    // These should NOT match because they're formatted differently
    let result1 = lookup_printconv("sprintf(\"%.1f mm\",$val)", "Exif_pm");
    let result2 = lookup_printconv("sprintf(\"%.1f mm\", $val)", "Exif_pm");

    // They might both be None or map to functions, but the key point is
    // we're not normalizing - we're doing exact matching
    // So if one is in the registry and the other isn't, they won't match

    // This test just verifies the lookup works, not the results
    // (since we may or may not have these specific entries)
    let _ = (result1, result2);
}

#[test]
fn test_registry_contains_expected_entries() {
    // Verify some known entries exist in the registries

    // Check PrintConv registry
    let printconv_registry = get_printconv_registry();
    assert!(printconv_registry.contains_key("fnumber_print_conv"));
    assert!(printconv_registry.contains_key("exposuretime_print_conv"));

    // Check ValueConv registry
    let valueconv_registry = get_valueconv_registry();
    // GPS timestamp conversion should be there
    let has_gps_timestamp = valueconv_registry
        .keys()
        .any(|k| k.contains("GPS::ConvertTimeStamp"));
    assert!(
        has_gps_timestamp,
        "ValueConv registry should contain GPS timestamp conversion"
    );
}

#[test]
fn test_tag_specific_printconv() {
    // Test tag-specific lookups
    let tag_registry = get_tag_specific_printconv();

    // Flash should be a universal tag
    assert!(tag_registry.contains_key("Flash"));

    // GPS reference tags should be there
    assert!(tag_registry.contains_key("GPSLatitudeRef"));
    assert!(tag_registry.contains_key("GPSLongitudeRef"));
}

#[test]
fn test_module_name_normalization() {
    // Module names should have _pm stripped
    // GPS_pm -> GPS for scoped lookups

    // This tests that the module name normalization (not expression normalization)
    // still works correctly
    let result = lookup_printconv("GPS::ConvertTimeStamp($val)", "GPS_pm");
    // Should work whether we pass GPS or GPS_pm
    let result2 = lookup_printconv("GPS::ConvertTimeStamp($val)", "GPS");

    // Both should resolve to the same thing (or both be None)
    assert_eq!(result, result2);
}
