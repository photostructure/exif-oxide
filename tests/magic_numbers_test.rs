//! Tests for file type detection

#[test]
fn test_magic_number_detection() {
    use exif_oxide::detection::{detect_file_type, FileType};

    // Test JPEG detection
    let jpeg_data = b"\xFF\xD8\xFF\xE0\x00\x10JFIF";
    let result = detect_file_type(jpeg_data).unwrap();
    assert_eq!(result.file_type, FileType::JPEG);
    assert_eq!(result.mime_type, "image/jpeg");

    // Test PNG detection
    let png_data = b"\x89PNG\r\n\x1A\n";
    let result = detect_file_type(png_data).unwrap();
    assert_eq!(result.file_type, FileType::PNG);
    assert_eq!(result.mime_type, "image/png");

    // Test TIFF (little endian)
    let tiff_le = b"II\x2A\x00";
    let result = detect_file_type(tiff_le).unwrap();
    assert_eq!(result.file_type, FileType::TIFF);
    assert_eq!(result.mime_type, "image/tiff");

    // Test TIFF (big endian)
    let tiff_be = b"MM\x00\x2A";
    let result = detect_file_type(tiff_be).unwrap();
    assert_eq!(result.file_type, FileType::TIFF);
    assert_eq!(result.mime_type, "image/tiff");

    // Test GIF
    let gif89a = b"GIF89a";
    let result = detect_file_type(gif89a).unwrap();
    assert_eq!(result.file_type, FileType::GIF);

    // Test unknown format
    let unknown = b"UNKNOWN_FORMAT";
    let result = detect_file_type(unknown).unwrap();
    assert_eq!(result.file_type, FileType::Unknown);
    assert_eq!(result.mime_type, "application/octet-stream");
}

#[test]
fn test_file_type_enum() {
    use exif_oxide::detection::FileType;

    // Test that basic formats are defined
    let _jpeg = FileType::JPEG;
    let _png = FileType::PNG;
    let _tiff = FileType::TIFF;
    let _cr2 = FileType::CR2;
    let _nef = FileType::NEF;
}
