use exif_oxide::core::Endian;
use exif_oxide::maker::{parse_maker_notes, Manufacturer};

#[test]
fn test_sony_manufacturer_detection() {
    assert_eq!(Manufacturer::from_make("SONY"), Manufacturer::Sony);
    assert_eq!(Manufacturer::from_make("Sony"), Manufacturer::Sony);
    assert_eq!(Manufacturer::from_make("SONY ILCE-7M3"), Manufacturer::Sony);
    assert_eq!(
        Manufacturer::from_make("SONY Corporation"),
        Manufacturer::Sony
    );
}

#[test]
fn test_sony_parser_available() {
    let manufacturer = Manufacturer::from_make("SONY");
    let parser = manufacturer.parser();
    assert!(parser.is_some());
}

#[test]
fn test_sony_maker_note_parsing() {
    // Test empty maker note
    let result = parse_maker_notes(&[], "SONY", Endian::Little, 0);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());

    // Test with minimal IFD data
    let ifd_data = vec![
        0x00, 0x00, // 0 entries (valid empty IFD)
        0xFF, 0xFF, 0xFF, 0xFF, // No next IFD
    ];

    let result = parse_maker_notes(&ifd_data, "SONY ILCE-7M3", Endian::Little, 0);
    assert!(result.is_ok());
}

#[test]
fn test_sony_maker_note_with_tags() {
    // Create a minimal IFD with one tag (Quality = 0x0102)
    let ifd_data = vec![
        0x01, 0x00, // 1 entry
        // Tag entry: tag=0x0102, type=LONG (4), count=1, value=1
        0x02, 0x01, // Tag ID 0x0102 (Quality)
        0x04, 0x00, // Type 4 (LONG)
        0x01, 0x00, 0x00, 0x00, // Count 1
        0x01, 0x00, 0x00, 0x00, // Value 1 (Super Fine)
        0xFF, 0xFF, 0xFF, 0xFF, // No next IFD
    ];

    let result = parse_maker_notes(&ifd_data, "SONY", Endian::Little, 0);
    assert!(result.is_ok());

    let tags = result.unwrap();
    // Sony tags are prefixed with 0x534F
    let expected_tag_id = 0x534F + 0x0102;
    assert!(
        tags.contains_key(&expected_tag_id),
        "Should contain prefixed Quality tag"
    );
}

#[test]
fn test_sony_endianness() {
    // Test with big-endian data
    let ifd_data = vec![
        0x00, 0x00, // 0 entries (big-endian)
        0xFF, 0xFF, 0xFF, 0xFF, // No next IFD
    ];

    let result = parse_maker_notes(&ifd_data, "SONY", Endian::Big, 0);
    assert!(result.is_ok());
}

#[test]
fn test_invalid_sony_maker_note() {
    // Test with invalid data (too short)
    let invalid_data = vec![0x00];

    let result = parse_maker_notes(&invalid_data, "SONY", Endian::Little, 0);
    // Should return Ok with empty HashMap due to error handling
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}
