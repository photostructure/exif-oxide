//! Video format detection tests for Phase 1 video containers
//! Tests QuickTime-based video formats with different brand codes

use exif_oxide::detection::{detect_file_type, FileType};

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
