//! Tests for comprehensive APP segment parsing using table-driven identification

use exif_oxide::core::jpeg::find_metadata_segments;
use std::io::Cursor;

#[test]
fn test_comprehensive_app_segment_parsing() {
    // Create a JPEG with multiple APP segments
    let mut data = vec![];
    data.extend_from_slice(&[0xFF, 0xD8]); // SOI

    // APP0 JFIF segment
    data.extend_from_slice(&[0xFF, 0xE0]); // APP0 marker
    let jfif_data = b"JFIF\x00\x01\x01\x01\x00H\x00H\x00\x00";
    let length = (2 + jfif_data.len()) as u16;
    data.extend_from_slice(&length.to_be_bytes());
    data.extend_from_slice(jfif_data);

    // APP1 EXIF segment
    data.extend_from_slice(&[0xFF, 0xE1]); // APP1 marker
    data.extend_from_slice(&[0x00, 0x0E]); // Length = 14
    data.extend_from_slice(b"Exif\0\0"); // EXIF signature
    data.extend_from_slice(b"IIMM\0*"); // Fake TIFF header

    // APP1 XMP segment
    let xmp_sig = b"http://ns.adobe.com/xap/1.0/\0";
    let xmp_content = b"<x:xmpmeta>test</x:xmpmeta>";
    data.extend_from_slice(&[0xFF, 0xE1]); // APP1 marker
    let xmp_length = (2 + xmp_sig.len() + xmp_content.len()) as u16;
    data.extend_from_slice(&xmp_length.to_be_bytes());
    data.extend_from_slice(xmp_sig);
    data.extend_from_slice(xmp_content);

    // APP2 MPF segment
    data.extend_from_slice(&[0xFF, 0xE2]); // APP2 marker
    let mpf_data = b"MPF\x00MM\x00*\x00\x00\x00\x08";
    let mpf_length = (2 + mpf_data.len()) as u16;
    data.extend_from_slice(&mpf_length.to_be_bytes());
    data.extend_from_slice(mpf_data);

    // APP6 GoPro segment
    data.extend_from_slice(&[0xFF, 0xE6]); // APP6 marker
    let gopro_data = b"GoPro\x00DEVC";
    let gopro_length = (2 + gopro_data.len()) as u16;
    data.extend_from_slice(&gopro_length.to_be_bytes());
    data.extend_from_slice(gopro_data);

    data.extend_from_slice(&[0xFF, 0xD9]); // EOI

    // Parse the JPEG
    let mut cursor = Cursor::new(&data);
    let metadata = find_metadata_segments(&mut cursor).unwrap();

    // Verify backward compatibility
    assert!(metadata.exif.is_some());
    assert_eq!(metadata.exif.unwrap().data, b"IIMM\0*");

    assert_eq!(metadata.xmp.len(), 1);
    assert_eq!(metadata.xmp[0].data, xmp_content);
    assert!(!metadata.xmp[0].is_extended);

    assert!(metadata.mpf.is_some());
    assert_eq!(metadata.mpf.unwrap().data, b"MM\x00*\x00\x00\x00\x08");

    assert_eq!(metadata.gpmf.len(), 1);
    assert_eq!(metadata.gpmf[0].data, b"DEVC");

    // Verify comprehensive APP segment support
    assert_eq!(metadata.app_segments.len(), 5); // APP0, APP1 EXIF, APP1 XMP, APP2, APP6

    // Verify APP0 JFIF
    let app0_segment = metadata
        .app_segments
        .iter()
        .find(|s| s.segment_number == 0)
        .unwrap();
    assert!(app0_segment.rule.is_some());
    assert_eq!(app0_segment.rule.unwrap().name, "JFIF");

    // Verify APP1 EXIF
    let app1_exif = metadata
        .app_segments
        .iter()
        .find(|s| s.segment_number == 1 && s.rule.map_or(false, |r| r.name == "EXIF"))
        .unwrap();
    assert!(app1_exif.rule.is_some());
    assert_eq!(app1_exif.rule.unwrap().name, "EXIF");

    // Verify APP1 XMP
    let app1_xmp = metadata
        .app_segments
        .iter()
        .find(|s| s.segment_number == 1 && s.rule.map_or(false, |r| r.name == "XMP"))
        .unwrap();
    assert!(app1_xmp.rule.is_some());
    assert_eq!(app1_xmp.rule.unwrap().name, "XMP");

    // Verify APP2 MPF
    let app2_segment = metadata
        .app_segments
        .iter()
        .find(|s| s.segment_number == 2)
        .unwrap();
    assert!(app2_segment.rule.is_some());
    assert_eq!(app2_segment.rule.unwrap().name, "MPF");

    // Verify APP6 GoPro
    let app6_segment = metadata
        .app_segments
        .iter()
        .find(|s| s.segment_number == 6)
        .unwrap();
    assert!(app6_segment.rule.is_some());
    assert_eq!(app6_segment.rule.unwrap().name, "GoPro");
}

#[test]
fn test_unknown_app_segment() {
    // Create a JPEG with an unknown APP segment
    let mut data = vec![];
    data.extend_from_slice(&[0xFF, 0xD8]); // SOI

    // APP5 with unknown signature
    data.extend_from_slice(&[0xFF, 0xE5]); // APP5 marker
    let unknown_data = b"UNKNOWN\x00\x01\x02\x03";
    let length = (2 + unknown_data.len()) as u16;
    data.extend_from_slice(&length.to_be_bytes());
    data.extend_from_slice(unknown_data);

    data.extend_from_slice(&[0xFF, 0xD9]); // EOI

    // Parse the JPEG
    let mut cursor = Cursor::new(&data);
    let metadata = find_metadata_segments(&mut cursor).unwrap();

    // Verify the unknown segment is still captured
    assert_eq!(metadata.app_segments.len(), 1);
    let app5_segment = &metadata.app_segments[0];
    assert_eq!(app5_segment.segment_number, 5);
    assert!(app5_segment.rule.is_none()); // No rule should match
    assert_eq!(app5_segment.data, unknown_data);
}
