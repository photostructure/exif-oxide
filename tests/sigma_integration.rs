//! Integration tests for Sigma maker notes
//!
//! Tests Sigma manufacturer detection and maker note parsing functionality

use exif_oxide::core::Endian;
use exif_oxide::maker::{parse_maker_notes, Manufacturer};

#[test]
fn test_sigma_manufacturer_detection() {
    // Test various Sigma manufacturer names
    assert_eq!(Manufacturer::from_make("SIGMA"), Manufacturer::Sigma);
    assert_eq!(Manufacturer::from_make("sigma"), Manufacturer::Sigma);
    assert_eq!(
        Manufacturer::from_make("Sigma Corporation"),
        Manufacturer::Sigma
    );
    assert_eq!(
        Manufacturer::from_make("SIGMA CORPORATION"),
        Manufacturer::Sigma
    );
    assert_eq!(Manufacturer::from_make("Sigma"), Manufacturer::Sigma);
}

#[test]
fn test_sigma_parser_instantiation() {
    let manufacturer = Manufacturer::Sigma;
    let parser = manufacturer.parser();
    assert!(parser.is_some());

    if let Some(parser) = parser {
        assert_eq!(parser.manufacturer(), "Sigma");
    }
}

#[test]
fn test_sigma_empty_maker_notes() {
    // Test with empty maker note data
    let result = parse_maker_notes(&[], "SIGMA", Endian::Little, 0);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_sigma_minimal_maker_notes() {
    // Test with minimal valid IFD data (0 entries)
    let minimal_ifd = vec![
        0x00, 0x00, // 0 entries
        0xFF, 0xFF, 0xFF, 0xFF, // No next IFD (-1)
    ];

    let result = parse_maker_notes(&minimal_ifd, "SIGMA", Endian::Little, 0);
    assert!(result.is_ok());
    // Should be empty since there are no entries
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_sigma_unknown_manufacturer() {
    // Test that unknown manufacturers don't trigger Sigma parsing
    assert_eq!(Manufacturer::from_make("FakeCamera"), Manufacturer::Unknown);
    assert_eq!(
        Manufacturer::from_make("NonExistentBrand"),
        Manufacturer::Unknown
    );
    assert_ne!(Manufacturer::from_make("FakeCamera"), Manufacturer::Sigma);

    // Test that known non-Sigma manufacturers are correctly identified (not as Unknown)
    assert_eq!(Manufacturer::from_make("Apple"), Manufacturer::Apple);
    assert_eq!(Manufacturer::from_make("Samsung"), Manufacturer::Samsung);
    assert_ne!(Manufacturer::from_make("Apple"), Manufacturer::Sigma);
}

#[test]
fn test_sigma_parser_error_handling() {
    // Test with malformed IFD data (too short)
    let malformed_data = vec![0x01]; // Too short to be valid IFD

    let result = parse_maker_notes(&malformed_data, "SIGMA", Endian::Little, 0);
    assert!(result.is_ok()); // Should return empty map, not error
    assert!(result.unwrap().is_empty());
}
