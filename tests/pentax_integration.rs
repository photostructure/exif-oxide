use exif_oxide::core::Endian;
use exif_oxide::maker::{parse_maker_notes, Manufacturer};

#[test]
fn test_pentax_manufacturer_detection() {
    assert_eq!(Manufacturer::from_make("PENTAX"), Manufacturer::Pentax);
    assert_eq!(Manufacturer::from_make("PENTAX K-3"), Manufacturer::Pentax);
    assert_eq!(
        Manufacturer::from_make("RICOH IMAGING COMPANY, LTD."),
        Manufacturer::Pentax
    );
    assert_eq!(Manufacturer::from_make("Ricoh"), Manufacturer::Pentax);
}

#[test]
fn test_pentax_parser_available() {
    let manufacturer = Manufacturer::from_make("PENTAX");
    let parser = manufacturer.parser();
    assert!(parser.is_some());
}

#[test]
fn test_pentax_maker_note_parsing() {
    // Test empty maker note
    let result = parse_maker_notes(&[], "PENTAX", Endian::Little, 0);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());

    // Test with minimal IFD data
    let ifd_data = vec![
        0x00, 0x00, // 0 entries (valid empty IFD)
        0xFF, 0xFF, 0xFF, 0xFF, // No next IFD
    ];

    let result = parse_maker_notes(&ifd_data, "PENTAX K-3", Endian::Little, 0);
    assert!(result.is_ok());
}
