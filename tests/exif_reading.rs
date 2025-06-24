//! Integration tests for Spike 1

use exif_oxide::read_basic_exif;

#[test]
fn test_exiftool_sample_image() {
    let exif = read_basic_exif("exiftool/t/images/ExifTool.jpg").unwrap();

    // The actual EXIF data contains FUJIFILM, even though ExifTool reports Canon
    // (ExifTool may be using other metadata sources)
    assert_eq!(exif.make, Some("FUJIFILM".to_string()));
    assert_eq!(exif.model, Some("FinePix2400Zoom".to_string()));
    assert_eq!(exif.orientation, Some(1)); // Normal orientation
}

#[test]
fn test_canon_image() {
    let exif = read_basic_exif("exiftool/t/images/Canon.jpg").unwrap();

    // Test that we can read Canon images
    assert_eq!(exif.make, Some("Canon".to_string()));
    assert_eq!(exif.model, Some("Canon EOS DIGITAL REBEL".to_string()));
}

#[test]
fn test_nikon_image() {
    let exif = read_basic_exif("exiftool/t/images/Nikon.jpg").unwrap();

    // Test that we can read Nikon images
    assert_eq!(exif.make, Some("NIKON".to_string()));
    assert!(exif.model.is_some());
}

#[test]
fn test_no_exif() {
    use exif_oxide::error::Error;

    let result = read_basic_exif("exiftool/t/images/PNG.png");

    // Should fail because this PNG has no EXIF data
    assert!(matches!(result, Err(Error::NoExif)));
}
