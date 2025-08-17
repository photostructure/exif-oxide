use exif_oxide::exif::ExifReader;
use exif_oxide::*;

#[test]
fn test_canon_lens_lookup() {
    // P07: Unified completion - see docs/todo/P07-unified-codegen-completion.md
    // This test validates that canonLensTypes lookup table is properly populated
    // and can handle mixed numeric/string keys like "2.1"

    // This should fail until SimpleTableStrategy is fixed to handle mixed keys
    use exif_oxide::generated::Canon_pm::canon_lens_types::lookup_canon_lens_types;

    // Test numeric key
    assert_eq!(lookup_canon_lens_types("1"), Some("Canon EF 50mm f/1.8"));

    // Test decimal string key (this is what currently fails)
    assert_eq!(
        lookup_canon_lens_types("2.1"),
        Some("Sigma 24mm f/2.8 Super Wide II")
    );

    // Test that the table is actually populated (not empty)
    // canonLensTypes should have 526+ entries
    // If the strategy fix worked, this lookup should exist
    assert!(lookup_canon_lens_types("4.1").is_some());
}

#[test]
fn test_compilation_succeeds() {
    // P07: Ensure the codebase compiles after all fixes
    // This test will pass when all import issues are resolved

    // Just creating an ExifReader should work if imports are fixed
    let _reader = ExifReader::new();

    // If we can construct it without panicking, compilation worked
    assert!(true);
}
