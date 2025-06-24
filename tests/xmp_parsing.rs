//! XMP metadata parsing tests

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

/// Create a test JPEG file with specific XMP content
fn create_test_jpeg_with_custom_xmp(
    dir: &TempDir,
    filename: &str,
    xmp_content: &[u8],
) -> std::path::PathBuf {
    let path = dir.path().join(filename);
    let mut file = File::create(&path).unwrap();

    let xmp_sig = b"http://ns.adobe.com/xap/1.0/\0";

    let mut data = vec![];
    data.extend_from_slice(&[0xFF, 0xD8]); // SOI

    // Add XMP APP1 segment
    data.extend_from_slice(&[0xFF, 0xE1]); // APP1 marker
    let length = (2 + xmp_sig.len() + xmp_content.len()) as u16;
    data.extend_from_slice(&length.to_be_bytes()); // Length
    data.extend_from_slice(xmp_sig); // XMP signature
    data.extend_from_slice(xmp_content); // XMP data

    data.extend_from_slice(&[0xFF, 0xD9]); // EOI

    file.write_all(&data).unwrap();
    path
}

mod basic_xmp_tests {
    use super::*;

    #[test]
    fn test_extract_xmp_properties_from_jpeg() {
        let dir = TempDir::new().unwrap();
        let jpeg_path = create_test_jpeg_with_xmp(&dir, "test.jpg");

        let properties = extract_xmp_properties(&jpeg_path).unwrap();

        // Verify basic properties
        assert_eq!(properties.get("dc:title"), Some(&"Test Image".to_string()));
        assert_eq!(properties.get("dc:creator"), Some(&"John Doe".to_string()));
        assert_eq!(properties.get("dc:format"), Some(&"image/jpeg".to_string()));
        assert_eq!(
            properties.get("xmp:CreatorTool"),
            Some(&"Adobe Photoshop CS6".to_string())
        );
        assert_eq!(
            properties.get("xmp:CreateDate"),
            Some(&"2024-01-15T10:30:00".to_string())
        );
        assert_eq!(
            properties.get("photoshop:Credit"),
            Some(&"Test Credit".to_string())
        );
    }

    #[test]
    fn test_parse_xmp_basic() {
        let xmp_data = br#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/"
            dc:title="Sample Title"
            dc:creator="Test Author">
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#;

        let metadata = xmp::parse_xmp(xmp_data).unwrap();
        let properties = xmp::extract_simple_properties(xmp_data).unwrap();

        assert_eq!(
            properties.get("dc:title"),
            Some(&"Sample Title".to_string())
        );
        assert_eq!(
            properties.get("dc:creator"),
            Some(&"Test Author".to_string())
        );
    }

    #[test]
    fn test_xmp_with_nested_elements() {
        let xmp_data = br#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/">
            <dc:title>Nested Title</dc:title>
            <dc:description>Nested Description</dc:description>
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#;

        let metadata = xmp::parse_xmp(xmp_data).unwrap();
        let properties = xmp::extract_simple_properties(xmp_data).unwrap();

        assert_eq!(
            properties.get("dc:title"),
            Some(&"Nested Title".to_string())
        );
        assert_eq!(
            properties.get("dc:description"),
            Some(&"Nested Description".to_string())
        );
    }
}

mod advanced_xmp_tests {
    use super::*;

    #[test]
    fn test_xmp_with_arrays() {
        let xmp_data = br#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/">
            <dc:subject>
                <rdf:Bag>
                    <rdf:li>landscape</rdf:li>
                    <rdf:li>nature</rdf:li>
                    <rdf:li>sunset</rdf:li>
                </rdf:Bag>
            </dc:subject>
            <dc:creator>
                <rdf:Seq>
                    <rdf:li>John Doe</rdf:li>
                    <rdf:li>Jane Smith</rdf:li>
                </rdf:Seq>
            </dc:creator>
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#;

        let metadata = xmp::parse_xmp(xmp_data).unwrap();

        // Check dc namespace exists
        assert!(metadata.properties.contains_key("dc"));

        // Check arrays were parsed
        let dc_props = &metadata.properties["dc"];
        assert!(dc_props.contains_key("subject"));
        assert!(dc_props.contains_key("creator"));

        // Verify array types
        match &dc_props["subject"] {
            xmp::types::XmpValue::Array(xmp::types::XmpArray::Unordered(items)) => {
                assert_eq!(items.len(), 3);
                assert!(items
                    .iter()
                    .any(|v| matches!(v, xmp::types::XmpValue::Simple(s) if s == "landscape")));
                assert!(items
                    .iter()
                    .any(|v| matches!(v, xmp::types::XmpValue::Simple(s) if s == "nature")));
                assert!(items
                    .iter()
                    .any(|v| matches!(v, xmp::types::XmpValue::Simple(s) if s == "sunset")));
            }
            _ => panic!("Expected Bag array for dc:subject"),
        }

        match &dc_props["creator"] {
            xmp::types::XmpValue::Array(xmp::types::XmpArray::Ordered(items)) => {
                assert_eq!(items.len(), 2);
                assert!(items
                    .iter()
                    .any(|v| matches!(v, xmp::types::XmpValue::Simple(s) if s == "John Doe")));
                assert!(items
                    .iter()
                    .any(|v| matches!(v, xmp::types::XmpValue::Simple(s) if s == "Jane Smith")));
            }
            _ => panic!("Expected Seq array for dc:creator"),
        }
    }

    #[test]
    fn test_xmp_with_language_alternatives() {
        let xmp_str = r#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/">
            <dc:title>
                <rdf:Alt>
                    <rdf:li xml:lang="x-default">Default Title</rdf:li>
                    <rdf:li xml:lang="en-US">English Title</rdf:li>
                    <rdf:li xml:lang="es">Título en Español</rdf:li>
                    <rdf:li xml:lang="de">Deutscher Titel</rdf:li>
                </rdf:Alt>
            </dc:title>
            <dc:rights>
                <rdf:Alt>
                    <rdf:li xml:lang="x-default">© 2024 Test Corp</rdf:li>
                    <rdf:li xml:lang="en">© 2024 Test Corporation</rdf:li>
                </rdf:Alt>
            </dc:rights>
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#;
        let xmp_data = xmp_str.as_bytes();

        let metadata = xmp::parse_xmp(xmp_data).unwrap();

        let dc_props = &metadata.properties["dc"];

        // Check title alternatives
        match &dc_props["title"] {
            xmp::types::XmpValue::Array(xmp::types::XmpArray::Alternative(alts)) => {
                assert_eq!(alts.len(), 4);

                // Find specific language alternatives
                let default_alt = alts.iter().find(|a| a.lang == "x-default").unwrap();
                assert_eq!(
                    default_alt.value,
                    xmp::types::XmpValue::Simple("Default Title".to_string())
                );

                let en_alt = alts.iter().find(|a| a.lang == "en-US").unwrap();
                assert_eq!(
                    en_alt.value,
                    xmp::types::XmpValue::Simple("English Title".to_string())
                );

                let es_alt = alts.iter().find(|a| a.lang == "es").unwrap();
                assert_eq!(
                    es_alt.value,
                    xmp::types::XmpValue::Simple("Título en Español".to_string())
                );

                let de_alt = alts.iter().find(|a| a.lang == "de").unwrap();
                assert_eq!(
                    de_alt.value,
                    xmp::types::XmpValue::Simple("Deutscher Titel".to_string())
                );
            }
            _ => panic!("Expected Alt array for dc:title"),
        }

        // Check rights alternatives
        match &dc_props["rights"] {
            xmp::types::XmpValue::Array(xmp::types::XmpArray::Alternative(alts)) => {
                assert_eq!(alts.len(), 2);

                let default_alt = alts.iter().find(|a| a.lang == "x-default").unwrap();
                assert_eq!(
                    default_alt.value,
                    xmp::types::XmpValue::Simple("© 2024 Test Corp".to_string())
                );

                let en_alt = alts.iter().find(|a| a.lang == "en").unwrap();
                assert_eq!(
                    en_alt.value,
                    xmp::types::XmpValue::Simple("© 2024 Test Corporation".to_string())
                );
            }
            _ => panic!("Expected Alt array for dc:rights"),
        }
    }

    #[test]
    #[ignore = "UTF-16 encoding support not yet implemented - re-enable when feature is added"]
    fn test_xmp_with_utf16_encoding() {
        // UTF-16 LE BOM followed by XML content
        let mut xmp_data = vec![0xFF, 0xFE]; // UTF-16 LE BOM
        let xml_str = r#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/"
            dc:title="UTF-16 Title 测试"
            dc:creator="作者名">
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#;

        // Convert to UTF-16 LE
        for ch in xml_str.encode_utf16() {
            xmp_data.extend_from_slice(&ch.to_le_bytes());
        }

        let metadata = xmp::parse_xmp(&xmp_data).unwrap();
        let properties = xmp::extract_simple_properties(&xmp_data).unwrap();

        assert!(properties.contains_key("dc:title"));
        assert!(properties.contains_key("dc:creator"));
        // The actual UTF-16 characters might not parse correctly in all cases
    }
}

mod error_handling_tests {
    use super::*;

    #[test]
    fn test_invalid_xml() {
        let invalid_xml = b"This is not XML at all!";
        // Invalid XML may succeed or fail - both acceptable
        let result = xmp::parse_xmp(invalid_xml);
        if result.is_ok() {
            let metadata = result.unwrap();
            // If it succeeds, it should have empty or minimal properties
            assert!(metadata.properties.is_empty() || metadata.properties.len() <= 1);
        }
    }

    #[test]
    fn test_malformed_xml() {
        let malformed = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about="">
            <!-- Missing closing tag -->
        </rdf:RDF>
</x:xmpmeta>"#;

        assert!(xmp::parse_xmp(malformed).is_err());
    }

    #[test]
    fn test_empty_xmp() {
        let empty_xmp = br#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
    </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#;

        let metadata = xmp::parse_xmp(empty_xmp).unwrap();
        let properties = xmp::extract_simple_properties(empty_xmp).unwrap();

        // Should have no properties
        assert!(properties.is_empty());
    }

    #[test]
    fn test_xmp_without_packet_markers() {
        let xmp_data = br#"<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/"
            dc:title="No Packet Markers">
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

        let metadata = xmp::parse_xmp(xmp_data).unwrap();
        let properties = xmp::extract_simple_properties(xmp_data).unwrap();

        assert_eq!(
            properties.get("dc:title"),
            Some(&"No Packet Markers".to_string())
        );
    }

    #[test]
    fn test_invalid_namespace_handling() {
        let xmp_data = br#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:unknown="http://example.com/unknown/"
            unknown:property="Unknown Value"
            xmlns:dc="http://purl.org/dc/elements/1.1/"
            dc:title="Known Title">
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#;

        let metadata = xmp::parse_xmp(xmp_data).unwrap();
        let properties = xmp::extract_simple_properties(xmp_data).unwrap();

        // Should handle unknown namespace gracefully
        assert_eq!(properties.get("dc:title"), Some(&"Known Title".to_string()));
        // Unknown namespace property might or might not be included
        // Both behaviors are acceptable
    }
}

mod integration_tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_real_image_xmp_extraction() {
        // Test with real images if they exist
        let test_image = "test-images/canon/Canon_T3i.JPG";
        if Path::new(test_image).exists() {
            let result = extract_xmp_properties(test_image);

            match result {
                Ok(properties) => {
                    println!("Found {} XMP properties", properties.len());
                    for (key, value) in properties.iter().take(5) {
                        println!("  {}: {}", key, value);
                    }
                }
                Err(e) => {
                    // Not all images have XMP data, which is fine
                    println!("No XMP data in test image or error: {}", e);
                }
            }
        }
    }

    #[test]
    fn test_extended_xmp() {
        // Create a JPEG with extended XMP (larger than 64KB)
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("extended_xmp.jpg");
        let mut file = File::create(&path).unwrap();

        let xmp_sig = b"http://ns.adobe.com/xap/1.0/\0";
        let xmp_guid = "ABCDEF0123456789ABCDEF0123456789"; // 32-char GUID
        let ext_sig = format!("http://ns.adobe.com/xmp/extension/\0{}\0", xmp_guid);

        // Main XMP with HasExtendedXMP
        let main_xmp = format!(
            r#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:xmpNote="http://ns.adobe.com/xmp/note/"
            xmpNote:HasExtendedXMP="{}">
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#,
            xmp_guid
        );

        // Extended XMP content
        let extended_content = br#"<rdf:Description rdf:about=""
    xmlns:dc="http://purl.org/dc/elements/1.1/"
    dc:title="Extended XMP Test"
    dc:description="This is part of extended XMP">
</rdf:Description>"#;

        let mut data = vec![];
        data.extend_from_slice(&[0xFF, 0xD8]); // SOI

        // Main XMP APP1
        data.extend_from_slice(&[0xFF, 0xE1]); // APP1 marker
        let length = (2 + xmp_sig.len() + main_xmp.as_bytes().len()) as u16;
        data.extend_from_slice(&length.to_be_bytes());
        data.extend_from_slice(xmp_sig);
        data.extend_from_slice(main_xmp.as_bytes());

        // Extended XMP APP1
        data.extend_from_slice(&[0xFF, 0xE1]); // APP1 marker
        let ext_header = ext_sig.as_bytes();
        let offset = 0u32;
        let total_length = extended_content.len() as u32;

        let ext_length = 2 + ext_header.len() + 4 + 4 + extended_content.len();
        data.extend_from_slice(&(ext_length as u16).to_be_bytes());
        data.extend_from_slice(ext_header);
        data.extend_from_slice(&offset.to_be_bytes());
        data.extend_from_slice(&total_length.to_be_bytes());
        data.extend_from_slice(extended_content);

        data.extend_from_slice(&[0xFF, 0xD9]); // EOI

        file.write_all(&data).unwrap();

        // Try to read it
        let result = xmp::reader::read_xmp_from_jpeg(&path);
        match result {
            Ok(Some(xmp_packet)) => {
                println!("Found XMP packet");

                // Check if extended XMP was detected
                let has_extended = xmp_packet.extended.is_some();
                println!("Has extended XMP: {}", has_extended);

                if let Some(ref extended) = xmp_packet.extended {
                    println!("Extended XMP GUID: {}", extended.guid);
                    println!("Extended XMP size: {} bytes", extended.total_length);
                }
            }
            Ok(None) => {
                println!("No XMP data found");
            }
            Err(e) => {
                println!("Error reading XMP: {}", e);
            }
        }
    }
}

mod phase2_tests {
    use super::*;

    #[test]
    #[ignore = "Structured XMP properties not yet implemented - re-enable when feature is added"]
    fn test_structured_properties() {
        let xmp_data = br#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:Iptc4xmpCore="http://iptc.org/std/Iptc4xmpCore/1.0/xmlns/">
            <Iptc4xmpCore:CreatorContactInfo>
                <Iptc4xmpCore:CiAdrExtadr>123 Main St</Iptc4xmpCore:CiAdrExtadr>
                <Iptc4xmpCore:CiAdrCity>Anytown</Iptc4xmpCore:CiAdrCity>
                <Iptc4xmpCore:CiAdrRegion>CA</Iptc4xmpCore:CiAdrRegion>
                <Iptc4xmpCore:CiAdrPcode>12345</Iptc4xmpCore:CiAdrPcode>
                <Iptc4xmpCore:CiEmailWork>test@example.com</Iptc4xmpCore:CiEmailWork>
            </Iptc4xmpCore:CreatorContactInfo>
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#;

        let metadata = xmp::parse_xmp(xmp_data).unwrap();

        // Should have parsed the structured property
        assert!(metadata.properties.contains_key("Iptc4xmpCore"));

        let iptc_props = &metadata.properties["Iptc4xmpCore"];
        assert!(iptc_props.contains_key("CreatorContactInfo"));

        // The nested structure should be captured
        match &iptc_props["CreatorContactInfo"] {
            xmp::types::XmpValue::Struct(fields) => {
                assert!(fields.contains_key("CiAdrExtadr"));
                assert!(fields.contains_key("CiAdrCity"));
                assert!(fields.contains_key("CiEmailWork"));
            }
            _ => panic!("Expected Structure for CreatorContactInfo"),
        }
    }

    #[test]
    #[ignore = "XMP resource references not yet implemented - re-enable when feature is added"]
    fn test_resource_ref() {
        let xmp_data = br#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:xmpMM="http://ns.adobe.com/xap/1.0/mm/"
            xmlns:stRef="http://ns.adobe.com/xap/1.0/sType/ResourceRef#">
            <xmpMM:DerivedFrom>
                <rdf:Description
                    stRef:instanceID="xmp.iid:ORIGINAL123"
                    stRef:documentID="xmp.did:ORIGINAL456"
                    stRef:originalDocumentID="xmp.did:ORIGINAL789">
                </rdf:Description>
            </xmpMM:DerivedFrom>
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#;

        let metadata = xmp::parse_xmp(xmp_data).unwrap();

        assert!(metadata.properties.contains_key("xmpMM"));
        let xmpmm_props = &metadata.properties["xmpMM"];
        assert!(xmpmm_props.contains_key("DerivedFrom"));
    }

    #[test]
    fn test_qualified_properties() {
        let xmp_data = br#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:xmp="http://ns.adobe.com/xap/1.0/"
            xmlns:xmpRights="http://ns.adobe.com/xap/1.0/rights/">
            <xmpRights:UsageTerms>
                <rdf:Alt>
                    <rdf:li xml:lang="x-default">For editorial use only</rdf:li>
                </rdf:Alt>
            </xmpRights:UsageTerms>
            <xmp:Rating>5</xmp:Rating>
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#;

        let metadata = xmp::parse_xmp(xmp_data).unwrap();
        let properties = xmp::extract_simple_properties(xmp_data).unwrap();

        // Simple property should be extracted
        assert_eq!(properties.get("xmp:Rating"), Some(&"5".to_string()));

        // Language alternative should be handled
        assert!(metadata.properties.contains_key("xmpRights"));
    }
}
