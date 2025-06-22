//! Integration tests for Spike 4: XMP Support

use exif_oxide::{extract_xmp_properties, xmp};
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

/// Create a test JPEG file with XMP data
fn create_test_jpeg_with_xmp(dir: &TempDir, filename: &str) -> std::path::PathBuf {
    let path = dir.path().join(filename);
    let mut file = File::create(&path).unwrap();

    let xmp_sig = b"http://ns.adobe.com/xap/1.0/\0";
    let xmp_data = br#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/"
            xmlns:xmp="http://ns.adobe.com/xap/1.0/"
            xmlns:photoshop="http://ns.adobe.com/photoshop/1.0/"
            dc:title="Test Image"
            dc:creator="John Doe"
            dc:format="image/jpeg"
            xmp:CreatorTool="Adobe Photoshop CS6"
            xmp:CreateDate="2024-01-15T10:30:00"
            photoshop:Credit="Test Credit">
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#;

    let mut data = vec![];
    data.extend_from_slice(&[0xFF, 0xD8]); // SOI

    // Add XMP APP1 segment
    data.extend_from_slice(&[0xFF, 0xE1]); // APP1 marker
    let length = (2 + xmp_sig.len() + xmp_data.len()) as u16;
    data.extend_from_slice(&length.to_be_bytes()); // Length
    data.extend_from_slice(xmp_sig); // XMP signature
    data.extend_from_slice(xmp_data); // XMP data

    // Add minimal image data
    data.extend_from_slice(&[0xFF, 0xDA]); // SOS marker
    data.extend_from_slice(&[0x00, 0x0C]); // Length
    data.extend_from_slice(&[0x03, 0x01, 0x00, 0x02, 0x11, 0x03, 0x11, 0x00, 0x3F, 0x00]); // Minimal scan header
    data.extend_from_slice(&[0xFF, 0xD9]); // EOI

    file.write_all(&data).unwrap();
    path
}

#[test]
fn test_extract_xmp_properties() {
    let temp_dir = TempDir::new().unwrap();
    let jpeg_path = create_test_jpeg_with_xmp(&temp_dir, "test_xmp.jpg");

    // Extract XMP properties
    let props = extract_xmp_properties(&jpeg_path).unwrap();

    // Verify expected properties
    assert_eq!(props.get("dc:title"), Some(&"Test Image".to_string()));
    assert_eq!(props.get("dc:creator"), Some(&"John Doe".to_string()));
    assert_eq!(props.get("dc:format"), Some(&"image/jpeg".to_string()));
    assert_eq!(
        props.get("xmp:CreatorTool"),
        Some(&"Adobe Photoshop CS6".to_string())
    );
    assert_eq!(
        props.get("xmp:CreateDate"),
        Some(&"2024-01-15T10:30:00".to_string())
    );
    assert_eq!(
        props.get("photoshop:Credit"),
        Some(&"Test Credit".to_string())
    );
}

#[test]
fn test_read_xmp_packet() {
    let temp_dir = TempDir::new().unwrap();
    let jpeg_path = create_test_jpeg_with_xmp(&temp_dir, "test_packet.jpg");

    // Read XMP packet directly
    let packet = xmp::read_xmp_from_jpeg(&jpeg_path).unwrap();
    assert!(packet.is_some());

    let packet = packet.unwrap();
    assert!(packet.as_str().unwrap().contains("Test Image"));
    assert!(packet.extended.is_none());
}

#[test]
fn test_jpeg_without_xmp() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("no_xmp.jpg");
    let mut file = File::create(&path).unwrap();

    // Create minimal JPEG without XMP
    let data = vec![
        0xFF, 0xD8, // SOI
        0xFF, 0xD9, // EOI
    ];

    file.write_all(&data).unwrap();

    // Try to extract XMP
    let props = extract_xmp_properties(&path).unwrap();
    assert!(props.is_empty());
}

#[test]
fn test_multiple_namespaces() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("multi_ns.jpg");
    let mut file = File::create(&path).unwrap();

    let xmp_sig = b"http://ns.adobe.com/xap/1.0/\0";
    let xmp_data = br#"<?xpacket begin="" id="test"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/"
            xmlns:xmp="http://ns.adobe.com/xap/1.0/"
            xmlns:tiff="http://ns.adobe.com/tiff/1.0/"
            xmlns:exif="http://ns.adobe.com/exif/1.0/"
            dc:subject="Test Subject"
            xmp:Rating="5"
            tiff:Make="Canon"
            tiff:Model="EOS R5"
            exif:ISO="400">
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#;

    let mut data = vec![];
    data.extend_from_slice(&[0xFF, 0xD8]); // SOI
    data.extend_from_slice(&[0xFF, 0xE1]); // APP1 marker
    let length = (2 + xmp_sig.len() + xmp_data.len()) as u16;
    data.extend_from_slice(&length.to_be_bytes());
    data.extend_from_slice(xmp_sig);
    data.extend_from_slice(xmp_data);
    data.extend_from_slice(&[0xFF, 0xD9]); // EOI

    file.write_all(&data).unwrap();

    let props = extract_xmp_properties(&path).unwrap();

    // Verify properties from different namespaces
    assert_eq!(props.get("dc:subject"), Some(&"Test Subject".to_string()));
    assert_eq!(props.get("xmp:Rating"), Some(&"5".to_string()));
    assert_eq!(props.get("tiff:Make"), Some(&"Canon".to_string()));
    assert_eq!(props.get("tiff:Model"), Some(&"EOS R5".to_string()));
    assert_eq!(props.get("exif:ISO"), Some(&"400".to_string()));
}

#[cfg(test)]
mod xmp_parser_advanced {
    use exif_oxide::xmp::{parse_xmp, XmpValue};

    #[test]
    fn test_parse_metadata_structure() {
        let xmp_data = br#"<?xpacket begin="" id="test"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/"
            xmlns:xmp="http://ns.adobe.com/xap/1.0/"
            dc:title="Test"
            xmp:Rating="5">
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#;

        let metadata = parse_xmp(xmp_data).unwrap();

        // Verify namespace registry
        assert_eq!(
            metadata.namespaces.get("dc"),
            Some(&"http://purl.org/dc/elements/1.1/".to_string())
        );
        assert_eq!(
            metadata.namespaces.get("xmp"),
            Some(&"http://ns.adobe.com/xap/1.0/".to_string())
        );

        // Verify properties
        let dc_props = metadata.properties.get("dc").unwrap();
        assert_eq!(
            dc_props.get("title"),
            Some(&XmpValue::Simple("Test".to_string()))
        );

        let xmp_props = metadata.properties.get("xmp").unwrap();
        assert_eq!(
            xmp_props.get("Rating"),
            Some(&XmpValue::Simple("5".to_string()))
        );
    }
}
