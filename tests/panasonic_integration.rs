use exif_oxide::core::Endian;
use exif_oxide::maker::{parse_maker_notes, Manufacturer};

#[test]
fn test_panasonic_manufacturer_detection() {
    assert_eq!(
        Manufacturer::from_make("Panasonic"),
        Manufacturer::Panasonic
    );
    assert_eq!(
        Manufacturer::from_make("PANASONIC"),
        Manufacturer::Panasonic
    );
    assert_eq!(
        Manufacturer::from_make("Panasonic DMC-GH5"),
        Manufacturer::Panasonic
    );
    assert_eq!(
        Manufacturer::from_make("Panasonic Corporation"),
        Manufacturer::Panasonic
    );
    assert_eq!(
        Manufacturer::from_make("panasonic lumix"),
        Manufacturer::Panasonic
    );
}

#[test]
fn test_panasonic_parser_available() {
    let manufacturer = Manufacturer::from_make("Panasonic");
    let parser = manufacturer.parser();
    assert!(parser.is_some());
}

#[test]
fn test_panasonic_maker_note_parsing() {
    // Test empty maker note
    let result = parse_maker_notes(&[], "Panasonic", Endian::Little, 0);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());

    // Test with minimal IFD data
    let ifd_data = vec![
        0x00, 0x00, // 0 entries (valid empty IFD)
        0xFF, 0xFF, 0xFF, 0xFF, // No next IFD
    ];

    let result = parse_maker_notes(&ifd_data, "Panasonic DMC-GH5", Endian::Little, 0);
    assert!(result.is_ok());
}

#[test]
fn test_panasonic_maker_note_with_tags() {
    // Create a minimal IFD with one tag (ImageQuality = 0x0001)
    let ifd_data = vec![
        0x01, 0x00, // 1 entry
        // Tag entry: tag=0x0001, type=LONG (4), count=1, value=1
        0x01, 0x00, // Tag ID 0x0001 (ImageQuality)
        0x04, 0x00, // Type 4 (LONG)
        0x01, 0x00, 0x00, 0x00, // Count 1
        0x01, 0x00, 0x00, 0x00, // Value 1
        0xFF, 0xFF, 0xFF, 0xFF, // No next IFD
    ];

    let result = parse_maker_notes(&ifd_data, "Panasonic", Endian::Little, 0);
    assert!(result.is_ok());

    let tags = result.unwrap();

    // Panasonic uses standard tag IDs without prefixing (based on ExifTool Panasonic.pm)
    // Tag 0x0001 is ImageQuality in ExifTool's Panasonic table
    let quality_tag_id = 0x0001;
    assert!(
        tags.contains_key(&quality_tag_id),
        "Should contain ImageQuality tag 0x0001, found keys: {:?}",
        tags.keys().collect::<Vec<_>>()
    );
}

#[test]
fn test_panasonic_signature_handling() {
    // Test with Panasonic signature + minimal IFD
    let signature_data = [
        // Panasonic signature: "Panasonic\0\0\0" (12 bytes)
        0x50, 0x61, 0x6e, 0x61, 0x73, 0x6f, 0x6e, 0x69, 0x63, 0x00, 0x00, 0x00,
        // Minimal IFD after signature
        0x00, 0x00, // 0 entries
        0xFF, 0xFF, 0xFF, 0xFF, // No next IFD
    ];

    let result = parse_maker_notes(&signature_data, "Panasonic", Endian::Little, 0);
    assert!(result.is_ok());
}

#[test]
fn test_panasonic_endianness() {
    // Test with big-endian data
    let ifd_data = vec![
        0x00, 0x00, // 0 entries (big-endian)
        0xFF, 0xFF, 0xFF, 0xFF, // No next IFD
    ];

    let result = parse_maker_notes(&ifd_data, "Panasonic", Endian::Big, 0);
    assert!(result.is_ok());
}

#[test]
fn test_invalid_panasonic_maker_note() {
    // Test with invalid data (too short)
    let invalid_data = vec![0x00];

    let result = parse_maker_notes(&invalid_data, "Panasonic", Endian::Little, 0);
    // Should return Ok with empty HashMap due to error handling
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_panasonic_multiple_tags() {
    // Create IFD with multiple tags (ImageQuality and FirmwareVersion)
    let ifd_data = vec![
        0x02, 0x00, // 2 entries
        // First tag entry: ImageQuality (0x0001)
        0x01, 0x00, // Tag ID 0x0001
        0x04, 0x00, // Type 4 (LONG)
        0x01, 0x00, 0x00, 0x00, // Count 1
        0x02, 0x00, 0x00, 0x00, // Value 2 (High quality)
        // Second tag entry: FirmwareVersion (0x0002)
        0x02, 0x00, // Tag ID 0x0002
        0x02, 0x00, // Type 2 (ASCII)
        0x08, 0x00, 0x00, 0x00, // Count 8
        0x26, 0x00, 0x00, 0x00, // Offset to string data (0x26 = 38)
        0xFF, 0xFF, 0xFF, 0xFF, // No next IFD
        // String data at offset 38 (0x26)
        0x56, 0x31, 0x2e, 0x30, 0x2e, 0x30, 0x30, 0x00, // "V1.0.00\0"
    ];

    let result = parse_maker_notes(&ifd_data, "Panasonic", Endian::Little, 0);
    assert!(result.is_ok());

    let tags = result.unwrap();

    // Should have both tags
    assert!(
        tags.contains_key(&0x0001),
        "Should contain ImageQuality tag"
    );
    assert!(
        tags.contains_key(&0x0002),
        "Should contain FirmwareVersion tag"
    );
}
