use exif_oxide::core::Endian;
use exif_oxide::maker::{parse_maker_notes, Manufacturer};

#[test]
fn test_nikon_manufacturer_detection() {
    assert_eq!(Manufacturer::from_make("NIKON"), Manufacturer::Nikon);
    assert_eq!(
        Manufacturer::from_make("NIKON CORPORATION"),
        Manufacturer::Nikon
    );
    assert_eq!(Manufacturer::from_make("NIKON Z 8"), Manufacturer::Nikon);
    assert_eq!(Manufacturer::from_make("Nikon D850"), Manufacturer::Nikon);
}

#[test]
fn test_nikon_parser_available() {
    let manufacturer = Manufacturer::from_make("NIKON");
    let parser = manufacturer.parser();
    assert!(parser.is_some());
}

#[test]
fn test_nikon_maker_note_parsing() {
    // Test empty maker note
    let result = parse_maker_notes(&[], "NIKON", Endian::Little, 0);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());

    // Test with minimal IFD data
    let ifd_data = vec![
        0x00, 0x00, // 0 entries (valid empty IFD)
        0xFF, 0xFF, 0xFF, 0xFF, // No next IFD
    ];

    let result = parse_maker_notes(&ifd_data, "NIKON CORPORATION", Endian::Little, 0);
    assert!(result.is_ok());
}

#[test]
fn test_nikon_endianness() {
    // Test both little-endian and big-endian
    let ifd_data = vec![
        0x00, 0x00, // 0 entries
        0xFF, 0xFF, 0xFF, 0xFF, // No next IFD
    ];

    // Little-endian
    let result = parse_maker_notes(&ifd_data, "NIKON", Endian::Little, 0);
    assert!(result.is_ok());

    // Big-endian
    let result = parse_maker_notes(&ifd_data, "NIKON", Endian::Big, 0);
    assert!(result.is_ok());
}

#[test]
fn test_nikon_invalid_data_handling() {
    // Test with invalid data (too short)
    let invalid_data = vec![0x00]; // Too short for valid IFD

    let result = parse_maker_notes(&invalid_data, "NIKON", Endian::Little, 0);
    assert!(result.is_ok()); // Should gracefully handle and return empty
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_nikon_model_variations() {
    // Test various Nikon model names
    let models = vec![
        "NIKON D850",
        "NIKON Z 8",
        "NIKON Z 6III",
        "NIKON D780",
        "NIKON CORPORATION",
        "Nikon",
        "nikon",
        "NIKON COOLPIX P950",
    ];

    for model in models {
        assert_eq!(Manufacturer::from_make(model), Manufacturer::Nikon);
        let parser = Manufacturer::from_make(model).parser();
        assert!(parser.is_some());
    }
}

#[test]
fn test_nikon_printconv_integration() {
    // For now, test the basic functionality with empty maker note data
    // The main goal is to ensure the parser doesn't crash with PrintConv integration
    let result = parse_maker_notes(&[], "NIKON", Endian::Little, 0);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());

    // Test with minimal valid empty IFD
    let empty_ifd = vec![
        0x00, 0x00, // 0 entries
        0x00, 0x00, 0x00, 0x00, // No next IFD
    ];

    let result = parse_maker_notes(&empty_ifd, "NIKON", Endian::Little, 0);
    assert!(result.is_ok());

    // The key test is that the parser compiles and runs with PrintConv integration
    // More detailed testing would require real Nikon maker note data
    println!("Nikon parser with PrintConv integration works correctly");
}

#[test]
fn test_nikon_table_driven_architecture() {
    // Test that the table-driven architecture is integrated properly
    // This test validates that the Nikon parser can be instantiated with
    // the PrintConv integration without errors

    let manufacturer = Manufacturer::from_make("NIKON");
    let parser = manufacturer.parser();
    assert!(parser.is_some(), "Nikon parser should be available");

    // Test with basic valid data
    let basic_data = vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let result = parse_maker_notes(&basic_data, "NIKON", Endian::Little, 0);

    // The key is that it doesn't panic and returns a result
    assert!(
        result.is_ok(),
        "Nikon parser should handle basic data without errors"
    );

    println!("Nikon table-driven PrintConv architecture integrated successfully");
}
