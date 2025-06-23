// Tests for expanded file type detection covering formats from TODO-SUPPORTED.md

use exif_oxide::detection::{detect_file_type, FileType};
use std::collections::HashMap;

/// Test data for various file formats
fn get_test_signatures() -> HashMap<FileType, Vec<u8>> {
    let mut signatures = HashMap::new();

    // GIF89a signature
    signatures.insert(
        FileType::GIF,
        b"GIF89a\x01\x00\x01\x00\x00\x00\x00".to_vec(),
    );

    // BMP signature
    signatures.insert(
        FileType::BMP,
        b"BM\x1e\x00\x00\x00\x00\x00\x00\x00\x1a\x00".to_vec(),
    );

    // WebP signature (RIFF + WEBP)
    signatures.insert(FileType::WEBP, b"RIFF\x20\x00\x00\x00WEBP".to_vec());

    // AVI signature (RIFF + AVI )
    signatures.insert(FileType::AVI, b"RIFF\x20\x00\x00\x00AVI ".to_vec());

    // RAF signature (Fujifilm)
    signatures.insert(FileType::RAF, b"FUJIFILM\x01\x00\x00\x00".to_vec());

    // MP4 signature (ftyp box)
    signatures.insert(FileType::MP4, b"\x00\x00\x00\x20ftypisom".to_vec());

    // MOV signature (moov box)
    signatures.insert(FileType::MOV, b"\x00\x00\x00\x20moov".to_vec());

    signatures
}

/// Expected MIME types from TODO-SUPPORTED.md
fn get_expected_mime_types() -> HashMap<FileType, &'static str> {
    let mut expected = HashMap::new();

    // Common image formats
    expected.insert(FileType::GIF, "image/gif");
    expected.insert(FileType::BMP, "image/bmp");
    expected.insert(FileType::WEBP, "image/webp");
    expected.insert(FileType::AVIF, "image/avif");

    // Canon RAW formats
    expected.insert(FileType::CRW, "image/x-canon-crw");

    // Sony RAW formats
    expected.insert(FileType::SR2, "image/x-sony-sr2");

    // Other manufacturer RAW formats
    expected.insert(FileType::RAF, "image/x-fujifilm-raf");
    expected.insert(FileType::ORF, "image/x-olympus-orf");
    expected.insert(FileType::PEF, "image/x-pentax-pef");

    // Video formats
    expected.insert(FileType::MP4, "video/mp4");
    expected.insert(FileType::MOV, "video/quicktime");
    expected.insert(FileType::AVI, "video/x-msvideo");

    expected
}

#[test]
fn test_new_format_detection() {
    let signatures = get_test_signatures();
    let expected_mimes = get_expected_mime_types();

    for (file_type, signature) in signatures {
        let info = detect_file_type(&signature).unwrap();

        assert_eq!(
            info.file_type, file_type,
            "Wrong file type detected for {:?}",
            file_type
        );

        if let Some(expected_mime) = expected_mimes.get(&file_type) {
            assert_eq!(
                info.mime_type, *expected_mime,
                "Wrong MIME type for {:?}: got {}, expected {}",
                file_type, info.mime_type, expected_mime
            );
        }
    }
}

#[test]
fn test_mime_type_compliance() {
    // Test that all MIME types match TODO-SUPPORTED.md requirements
    let _expected_mimes = get_expected_mime_types();

    // Test a few key formats with their expected MIME types
    let test_cases = vec![
        (FileType::GIF, "image/gif"),
        (FileType::BMP, "image/bmp"),
        (FileType::WEBP, "image/webp"),
        (FileType::RAF, "image/x-fujifilm-raf"),
        (FileType::MP4, "video/mp4"),
        (FileType::MOV, "video/quicktime"),
        (FileType::AVI, "video/x-msvideo"),
    ];

    for (file_type, expected_mime) in test_cases {
        let signatures = get_test_signatures();
        if let Some(signature) = signatures.get(&file_type) {
            let info = detect_file_type(signature).unwrap();
            assert_eq!(
                info.mime_type, expected_mime,
                "MIME type mismatch for {:?}",
                file_type
            );
        }
    }
}

#[test]
fn test_riff_container_disambiguation() {
    // Test that we can distinguish between WebP and AVI (both use RIFF)

    // WebP: RIFF + WEBP
    let webp_data = b"RIFF\x20\x00\x00\x00WEBP";
    let webp_info = detect_file_type(webp_data).unwrap();
    assert_eq!(webp_info.file_type, FileType::WEBP);
    assert_eq!(webp_info.mime_type, "image/webp");

    // AVI: RIFF + AVI
    let avi_data = b"RIFF\x20\x00\x00\x00AVI ";
    let avi_info = detect_file_type(avi_data).unwrap();
    assert_eq!(avi_info.file_type, FileType::AVI);
    assert_eq!(avi_info.mime_type, "video/x-msvideo");
}

#[test]
fn test_quicktime_container_detection() {
    // Test MP4/MOV detection via different fourcc codes

    // MP4 with ftyp box
    let mp4_data = b"\x00\x00\x00\x20ftypisom\x00\x00\x02\x00";
    let mp4_info = detect_file_type(mp4_data).unwrap();
    assert_eq!(mp4_info.file_type, FileType::MP4);
    assert_eq!(mp4_info.mime_type, "video/mp4");

    // MOV with moov box
    let mov_data = b"\x00\x00\x00\x20moov";
    let mov_info = detect_file_type(mov_data).unwrap();
    assert_eq!(mov_info.file_type, FileType::MOV);
    assert_eq!(mov_info.mime_type, "video/quicktime");
}

#[test]
fn test_file_mime_type_accessor() {
    // Test File:MIMEType field compatibility for new formats
    let signatures = get_test_signatures();

    for (_file_type, signature) in signatures {
        let info = detect_file_type(&signature).unwrap();

        // File:MIMEType should match mime_type
        assert_eq!(info.file_mime_type(), info.mime_type);

        // Should not be empty
        assert!(!info.file_mime_type().is_empty());
    }
}
