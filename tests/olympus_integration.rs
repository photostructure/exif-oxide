use exif_oxide::core::Endian;
use exif_oxide::maker::{parse_maker_notes, Manufacturer};

#[test]
fn test_olympus_manufacturer_detection() {
    assert_eq!(Manufacturer::from_make("OLYMPUS"), Manufacturer::Olympus);
    assert_eq!(
        Manufacturer::from_make("OLYMPUS CORPORATION"),
        Manufacturer::Olympus
    );
    assert_eq!(
        Manufacturer::from_make("OLYMPUS IMAGING CORP."),
        Manufacturer::Olympus
    );
    assert_eq!(
        Manufacturer::from_make("OLYMPUS OPTICAL CO.,LTD"),
        Manufacturer::Olympus
    );
}

#[test]
fn test_olympus_parser_available() {
    let manufacturer = Manufacturer::from_make("OLYMPUS");
    let parser = manufacturer.parser();
    assert!(parser.is_some());
}

#[test]
fn test_olympus_maker_note_parsing() {
    // Test empty maker note
    let result = parse_maker_notes(&[], "OLYMPUS", Endian::Little, 0);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());

    // Test with minimal IFD data (no signature)
    let ifd_data = vec![
        0x00, 0x00, // 0 entries (valid empty IFD)
        0xFF, 0xFF, 0xFF, 0xFF, // No next IFD
    ];

    let result = parse_maker_notes(&ifd_data, "OLYMPUS", Endian::Little, 0);
    assert!(result.is_ok());
}

#[test]
fn test_olympus_maker_note_with_signature() {
    // Test with Olympus signature "OLYMPUS\x00II"
    let mut data = Vec::new();
    data.extend_from_slice(b"OLYMPUS\x00"); // 8-byte signature
    data.extend_from_slice(b"II"); // Little endian marker
    data.extend_from_slice(&[0x00, 0x00]); // 0 entries
    data.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]); // No next IFD

    let result = parse_maker_notes(&data, "OLYMPUS", Endian::Little, 0);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty()); // Empty because 0 entries
}

#[test]
fn test_olympus_maker_note_with_big_endian() {
    // Test with Olympus signature "OLYMPUS\x00MM"
    let mut data = Vec::new();
    data.extend_from_slice(b"OLYMPUS\x00"); // 8-byte signature
    data.extend_from_slice(b"MM"); // Big endian marker
    data.extend_from_slice(&[0x00, 0x00]); // 0 entries
    data.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]); // No next IFD

    let result = parse_maker_notes(&data, "OLYMPUS", Endian::Big, 0);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty()); // Empty because 0 entries
}

// Integration test with real Olympus test images (when available)
#[test]
fn test_olympus_real_image() {
    use exif_oxide::read_basic_exif;
    use std::path::Path;

    // Test with ExifTool's Olympus test image
    let test_image = Path::new("third-party/exiftool/t/images/Olympus.jpg");
    if test_image.exists() {
        let result = read_basic_exif(test_image);
        assert!(result.is_ok());

        let exif = result.unwrap();
        assert!(exif.make.is_some());
        assert!(exif.make.unwrap().contains("OLYMPUS"));
    }
}
