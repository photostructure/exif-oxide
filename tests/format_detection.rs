// EXIFTOOL-COMPAT: Integration tests for file type detection
// Validates against ExifTool's output

use exif_oxide::detection::{detect_file_type, FileType};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::process::Command;

#[test]
fn test_jpeg_detection() {
    let mut file = File::open("test-images/canon/Canon_T3i.JPG").unwrap();
    let mut buffer = vec![0; 1024];
    let bytes_read = file.read(&mut buffer).unwrap();
    buffer.truncate(bytes_read);

    let info = detect_file_type(&buffer).unwrap();

    assert_eq!(info.file_type, FileType::JPEG);
    assert_eq!(info.mime_type, "image/jpeg");
    assert!(!info.weak_detection);
}

#[test]
fn test_cr2_detection() {
    let mut file = File::open("test-images/canon/Canon_T3i.CR2").unwrap();
    let mut buffer = vec![0; 1024];
    let bytes_read = file.read(&mut buffer).unwrap();
    buffer.truncate(bytes_read);

    let info = detect_file_type(&buffer).unwrap();

    assert_eq!(info.file_type, FileType::CR2);
    assert_eq!(info.mime_type, "image/x-canon-cr2");
}

#[test]
fn test_nrw_detection() {
    let mut file = File::open("test-images/nikon/nikon_z8_73.NEF").unwrap();
    let mut buffer = vec![0; 1024];
    let bytes_read = file.read(&mut buffer).unwrap();
    buffer.truncate(bytes_read);

    let info = detect_file_type(&buffer).unwrap();

    assert_eq!(info.file_type, FileType::NRW);
    assert_eq!(info.mime_type, "image/x-nikon-nrw");
}

#[test]
fn test_mime_type_compatibility() {
    // Test that our MIME types match ExifTool's output
    let test_files = vec![
        ("test-images/canon/Canon_T3i.JPG", "image/jpeg"),
        ("test-images/canon/Canon_T3i.CR2", "image/x-canon-cr2"),
        // Note: NEF file is actually NRW format according to ExifTool
        ("test-images/nikon/nikon_z8_73.NEF", "image/x-nikon-nrw"),
    ];

    for (file_path, expected_mime) in test_files {
        let mut file = File::open(file_path).unwrap();
        let mut buffer = vec![0; 1024];
        let bytes_read = file.read(&mut buffer).unwrap();
        buffer.truncate(bytes_read);

        let info = detect_file_type(&buffer).unwrap();
        assert_eq!(
            info.mime_type, expected_mime,
            "MIME type mismatch for {}",
            file_path
        );
    }
}

#[test]
#[ignore] // Run with --ignored to test against actual ExifTool
fn test_against_exiftool_output() {
    // Compare our detection with ExifTool's output
    let test_files = vec![
        "test-images/canon/Canon_T3i.JPG",
        "test-images/canon/Canon_T3i.CR2",
        "test-images/nikon/nikon_z8_73.NEF",
    ];

    for file_path in test_files {
        // Get ExifTool's detection
        let output = Command::new("exiftool")
            .args(["-j", "-FileType", "-MIMEType", file_path])
            .output()
            .expect("Failed to run exiftool");

        let exiftool_json: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
        let exiftool_data = &exiftool_json[0];

        let exiftool_mime = exiftool_data["MIMEType"].as_str().unwrap();

        // Get our detection
        let mut file = File::open(file_path).unwrap();
        let mut buffer = vec![0; 1024];
        let bytes_read = file.read(&mut buffer).unwrap();
        buffer.truncate(bytes_read);

        let info = detect_file_type(&buffer).unwrap();

        assert_eq!(
            info.mime_type, exiftool_mime,
            "MIME type mismatch for {} - ours: {}, ExifTool: {}",
            file_path, info.mime_type, exiftool_mime
        );
    }
}

#[test]
fn test_file_mime_type_field() {
    // Test that File:MIMEType field is accessible
    let mut file = File::open("test-images/canon/Canon_T3i.JPG").unwrap();
    let mut buffer = vec![0; 1024];
    let bytes_read = file.read(&mut buffer).unwrap();
    buffer.truncate(bytes_read);

    let info = detect_file_type(&buffer).unwrap();

    // Verify the File:MIMEType field accessor works
    assert_eq!(info.file_mime_type(), "image/jpeg");
    assert_eq!(info.file_mime_type(), info.mime_type);
}

// Video format detection tests
#[test]
fn test_canon_crm_detection() {
    // Canon CRM file: QuickTime container with "crx " brand + video atoms
    let crm_data = [
        // QuickTime header with size and ftyp
        0x00, 0x00, 0x00, 0x20, // box size (32 bytes)
        b'f', b't', b'y', b'p', // ftyp box
        b'c', b'r', b'x', b' ', // brand: "crx "
        b'c', b'r', b'x', b' ', // compatible brand
        // Add some padding and video-like atoms
        0x00, 0x00, 0x00, 0x10, // next box size
        b't', b'r', b'a', b'k', // trak atom (indicates video track)
        0x00, 0x00, 0x00, 0x00, // padding
        0x00, 0x00, 0x00, 0x00,
    ];

    let info = detect_file_type(&crm_data).unwrap();
    assert_eq!(info.file_type, FileType::CRM);
    assert_eq!(info.mime_type, "video/x-canon-crm");
    assert!(!info.weak_detection);
}

#[test]
fn test_canon_cr3_detection() {
    // Canon CR3 file: QuickTime container with "crx " brand but no video atoms
    let cr3_data = [
        // QuickTime header with size and ftyp
        0x00, 0x00, 0x00, 0x20, // box size (32 bytes)
        b'f', b't', b'y', b'p', // ftyp box
        b'c', b'r', b'x', b' ', // brand: "crx "
        b'c', b'r', b'x', b' ', // compatible brand
        // Add some padding but no video atoms
        0x00, 0x00, 0x00, 0x10, // next box size
        b'm', b'd', b'a', b't', // mdat atom (media data, but no video track)
        0x00, 0x00, 0x00, 0x00, // padding
        0x00, 0x00, 0x00, 0x00,
    ];

    let info = detect_file_type(&cr3_data).unwrap();
    assert_eq!(info.file_type, FileType::CR3);
    assert_eq!(info.mime_type, "image/x-canon-cr3");
    assert!(!info.weak_detection);
}

#[test]
fn test_3gpp_detection() {
    // 3GPP file: QuickTime container with "3gp4" brand
    let threegpp_data = [
        // QuickTime header with size and ftyp
        0x00, 0x00, 0x00, 0x20, // box size (32 bytes)
        b'f', b't', b'y', b'p', // ftyp box
        b'3', b'g', b'p', b'4', // brand: "3gp4"
        b'3', b'g', b'p', b'4', // compatible brand
        // Add some padding
        0x00, 0x00, 0x00, 0x10, // next box size
        b'm', b'd', b'a', b't', // mdat atom
        0x00, 0x00, 0x00, 0x00, // padding
        0x00, 0x00, 0x00, 0x00,
    ];

    let info = detect_file_type(&threegpp_data).unwrap();
    assert_eq!(info.file_type, FileType::ThreeGPP);
    assert_eq!(info.mime_type, "video/3gpp");
    assert!(!info.weak_detection);
}

#[test]
fn test_3gpp2_detection() {
    // 3GPP2 file: QuickTime container with "3g2a" brand
    let threegpp2_data = [
        // QuickTime header with size and ftyp
        0x00, 0x00, 0x00, 0x20, // box size (32 bytes)
        b'f', b't', b'y', b'p', // ftyp box
        b'3', b'g', b'2', b'a', // brand: "3g2a"
        b'3', b'g', b'2', b'a', // compatible brand
        // Add some padding
        0x00, 0x00, 0x00, 0x10, // next box size
        b'm', b'd', b'a', b't', // mdat atom
        0x00, 0x00, 0x00, 0x00, // padding
        0x00, 0x00, 0x00, 0x00,
    ];

    let info = detect_file_type(&threegpp2_data).unwrap();
    assert_eq!(info.file_type, FileType::ThreeGPP2);
    assert_eq!(info.mime_type, "video/3gpp2");
    assert!(!info.weak_detection);
}

#[test]
fn test_m4v_detection() {
    // M4V file: QuickTime container with "M4V " brand
    let m4v_data = [
        // QuickTime header with size and ftyp
        0x00, 0x00, 0x00, 0x20, // box size (32 bytes)
        b'f', b't', b'y', b'p', // ftyp box
        b'M', b'4', b'V', b' ', // brand: "M4V "
        b'M', b'4', b'V', b' ', // compatible brand
        // Add some padding
        0x00, 0x00, 0x00, 0x10, // next box size
        b'm', b'd', b'a', b't', // mdat atom
        0x00, 0x00, 0x00, 0x00, // padding
        0x00, 0x00, 0x00, 0x00,
    ];

    let info = detect_file_type(&m4v_data).unwrap();
    assert_eq!(info.file_type, FileType::M4V);
    assert_eq!(info.mime_type, "video/x-m4v");
    assert!(!info.weak_detection);
}

#[test]
fn test_heif_sequence_detection() {
    // HEIF sequence file: QuickTime container with "msf1" brand
    let heifs_data = [
        // QuickTime header with size and ftyp
        0x00, 0x00, 0x00, 0x20, // box size (32 bytes)
        b'f', b't', b'y', b'p', // ftyp box
        b'm', b's', b'f', b'1', // brand: "msf1" (HEIF sequence)
        b'm', b's', b'f', b'1', // compatible brand
        // Add some padding
        0x00, 0x00, 0x00, 0x10, // next box size
        b'm', b'd', b'a', b't', // mdat atom
        0x00, 0x00, 0x00, 0x00, // padding
        0x00, 0x00, 0x00, 0x00,
    ];

    let info = detect_file_type(&heifs_data).unwrap();
    assert_eq!(info.file_type, FileType::HEIFS);
    assert_eq!(info.mime_type, "image/heif-sequence");
    assert!(!info.weak_detection);
}

#[test]
fn test_heic_sequence_detection() {
    // HEIC sequence file: QuickTime container with "hevc" brand
    let heics_data = [
        // QuickTime header with size and ftyp
        0x00, 0x00, 0x00, 0x20, // box size (32 bytes)
        b'f', b't', b'y', b'p', // ftyp box
        b'h', b'e', b'v', b'c', // brand: "hevc" (HEIC sequence)
        b'h', b'e', b'v', b'c', // compatible brand
        // Add some padding
        0x00, 0x00, 0x00, 0x10, // next box size
        b'm', b'd', b'a', b't', // mdat atom
        0x00, 0x00, 0x00, 0x00, // padding
        0x00, 0x00, 0x00, 0x00,
    ];

    let info = detect_file_type(&heics_data).unwrap();
    assert_eq!(info.file_type, FileType::HEICS);
    assert_eq!(info.mime_type, "image/heic-sequence");
    assert!(!info.weak_detection);
}

#[test]
fn test_file_extension_detection() {
    // Test extension-based detection for video formats
    use exif_oxide::detection::detect_by_extension;

    let crm_info = detect_by_extension("crm").unwrap();
    assert_eq!(crm_info.file_type, FileType::CRM);
    assert_eq!(crm_info.mime_type, "video/x-canon-crm");
    assert!(crm_info.weak_detection);

    let threegpp_info = detect_by_extension("3gp").unwrap();
    assert_eq!(threegpp_info.file_type, FileType::ThreeGPP);
    assert_eq!(threegpp_info.mime_type, "video/3gpp");

    let m4v_info = detect_by_extension("m4v").unwrap();
    assert_eq!(m4v_info.file_type, FileType::M4V);
    assert_eq!(m4v_info.mime_type, "video/x-m4v");
}

// Expanded format detection tests
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
