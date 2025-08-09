//! Integration test for P07 unified codegen completion
//!
//! P07: Unified completion - see docs/todo/P07-unified-codegen-completion.md
//!
//! This test specifically validates that canonLensTypes lookup functionality works,
//! which was identified as the critical blocking issue that needed to be resolved.

use std::collections::HashMap;

#[test]
fn test_canon_lens_lookup_functionality() {
    // Test that canonLensTypes lookup works for the specific example mentioned in TPP
    // TPP states: Lens ID 2.1 should return "Sigma 24mm f/2.8 Super Wide II"

    // Try to access the canon lens lookup function
    // This will fail compilation if the module/function isn't properly generated and exported
    let result = exif_oxide::generated::canon::canon_lens_types::lookup_canon_lens_types("2.1");

    // Validate the expected result from the TPP
    assert_eq!(result, Some("Sigma 24mm f/2.8 Super Wide II"));

    // Test a few more entries to ensure the lookup table is properly populated
    assert_eq!(
        exif_oxide::generated::canon::canon_lens_types::lookup_canon_lens_types("2"),
        Some("Canon EF 28mm f/2.8 or 28mm f/2.8 IS USM")
    );

    // Test that unknown keys return None
    assert_eq!(
        exif_oxide::generated::canon::canon_lens_types::lookup_canon_lens_types("999.999"),
        None
    );
}

#[test]
fn test_canon_lens_types_module_accessible() {
    // Test that the module structure is properly set up
    // This ensures the module declarations were correctly generated

    use exif_oxide::generated::canon::canon_lens_types;

    // Test that we can call the lookup function at all
    let _result = canon_lens_types::lookup_canon_lens_types("1");

    // If we get here, the module structure is working
    assert!(true);
}

#[test]
fn test_canon_lens_types_populated() {
    // Test that the canonical lens types table has reasonable number of entries
    // TPP mentioned 526+ entries expected

    use exif_oxide::generated::canon::canon_lens_types;

    // Test some known entries to ensure table is populated
    let test_keys = vec!["1", "2", "3", "4", "5", "2.1", "4.1"];
    let mut found_entries = 0;

    for key in test_keys {
        if canon_lens_types::lookup_canon_lens_types(key).is_some() {
            found_entries += 1;
        }
    }

    // We should find at least several entries if the table is properly populated
    assert!(
        found_entries >= 3,
        "Expected at least 3 entries in canonLensTypes, found {}",
        found_entries
    );
}
