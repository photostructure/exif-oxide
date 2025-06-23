//! Phase 2 tests for nested XMP elements

use exif_oxide::xmp::{parse_xmp, XmpArray, XmpValue};
use std::fs;

#[test]
fn test_parse_nested_xmp_from_file() {
    // Read the XMP test file that contains nested elements
    let jpeg_data = fs::read("exiftool/t/images/XMP.jpg").expect("Failed to read test image");

    // Extract XMP using our JPEG parser
    let mut file = std::io::Cursor::new(&jpeg_data);
    let metadata =
        exif_oxide::core::jpeg::find_metadata_segments(&mut file).expect("Failed to parse JPEG");

    assert!(!metadata.xmp.is_empty(), "Should find XMP segment");

    let xmp_segment = &metadata.xmp[0];

    // The XMP segment data already has the signature stripped by the JPEG parser
    let xmp_data = &xmp_segment.data;

    // Parse XMP
    let xmp = parse_xmp(xmp_data).expect("Failed to parse XMP");

    // Verify namespaces were extracted
    assert!(xmp.namespaces.contains_key("dc"));
    assert!(xmp.namespaces.contains_key("photoshop"));
    assert!(xmp.namespaces.contains_key("xapBJ"));

    // Check dc:creator (should be an ordered array/Seq)
    if let Some(dc_props) = xmp.properties.get("dc") {
        if let Some(creator) = dc_props.get("creator") {
            match creator {
                XmpValue::Array(XmpArray::Ordered(values)) => {
                    assert_eq!(values.len(), 1);
                    assert_eq!(values[0].as_str(), Some("Phil Harvey"));
                }
                _ => panic!("dc:creator should be an ordered array"),
            }
        } else {
            panic!("dc:creator not found");
        }

        // Check dc:subject (should be an unordered array/Bag)
        if let Some(subject) = dc_props.get("subject") {
            match subject {
                XmpValue::Array(XmpArray::Unordered(values)) => {
                    assert_eq!(values.len(), 3);
                    let subjects: Vec<&str> = values.iter().filter_map(|v| v.as_str()).collect();
                    assert!(subjects.contains(&"ExifTool"));
                    assert!(subjects.contains(&"Test"));
                    assert!(subjects.contains(&"XMP"));
                }
                _ => panic!("dc:subject should be an unordered array"),
            }
        }

        // Check dc:title (should be an Alt array)
        if let Some(title) = dc_props.get("title") {
            match title {
                XmpValue::Array(XmpArray::Alternative(alts)) => {
                    assert!(!alts.is_empty());
                    // Should have x-default language
                    let default_title = alts
                        .iter()
                        .find(|alt| alt.lang == "x-default")
                        .expect("Should have x-default title");
                    assert_eq!(default_title.value.as_str(), Some("Test IPTC picture"));
                }
                _ => panic!("dc:title should be an alternative array"),
            }
        }
    } else {
        panic!("dc namespace properties not found");
    }

    // Check photoshop properties (simple values)
    if let Some(ps_props) = xmp.properties.get("photoshop") {
        assert_eq!(
            ps_props.get("City").and_then(|v| v.as_str()),
            Some("Kingston")
        );
        assert_eq!(
            ps_props.get("Country").and_then(|v| v.as_str()),
            Some("Canada")
        );
        assert_eq!(
            ps_props.get("Credit").and_then(|v| v.as_str()),
            Some("My Credit")
        );
    }
}

#[test]
fn test_parse_simple_nested_xmp() {
    let xmp_data = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/">
            <dc:creator>
                <rdf:Seq>
                    <rdf:li>John Doe</rdf:li>
                    <rdf:li>Jane Smith</rdf:li>
                </rdf:Seq>
            </dc:creator>
            <dc:subject>
                <rdf:Bag>
                    <rdf:li>landscape</rdf:li>
                    <rdf:li>nature</rdf:li>
                </rdf:Bag>
            </dc:subject>
            <dc:description>
                <rdf:Alt>
                    <rdf:li xml:lang="x-default">A beautiful landscape</rdf:li>
                    <rdf:li xml:lang="en-US">A beautiful landscape</rdf:li>
                    <rdf:li xml:lang="fr-FR">Un beau paysage</rdf:li>
                </rdf:Alt>
            </dc:description>
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    let xmp = parse_xmp(xmp_data).expect("Failed to parse XMP");

    // Check dc:creator (Seq)
    let dc_props = xmp.properties.get("dc").expect("dc namespace not found");
    let creator = dc_props.get("creator").expect("creator not found");

    match creator {
        XmpValue::Array(XmpArray::Ordered(values)) => {
            assert_eq!(values.len(), 2);
            assert_eq!(values[0].as_str(), Some("John Doe"));
            assert_eq!(values[1].as_str(), Some("Jane Smith"));
        }
        _ => panic!("Expected ordered array for creator"),
    }

    // Check dc:subject (Bag)
    let subject = dc_props.get("subject").expect("subject not found");
    match subject {
        XmpValue::Array(XmpArray::Unordered(values)) => {
            assert_eq!(values.len(), 2);
            let subjects: Vec<&str> = values.iter().filter_map(|v| v.as_str()).collect();
            assert!(subjects.contains(&"landscape"));
            assert!(subjects.contains(&"nature"));
        }
        _ => panic!("Expected unordered array for subject"),
    }

    // Check dc:description (Alt)
    let description = dc_props.get("description").expect("description not found");
    match description {
        XmpValue::Array(XmpArray::Alternative(alts)) => {
            assert_eq!(alts.len(), 3);

            // Check each language alternative
            for alt in alts {
                match alt.lang.as_str() {
                    "x-default" | "en-US" => {
                        assert_eq!(alt.value.as_str(), Some("A beautiful landscape"));
                    }
                    "fr-FR" => {
                        assert_eq!(alt.value.as_str(), Some("Un beau paysage"));
                    }
                    _ => panic!("Unexpected language: {}", alt.lang),
                }
            }
        }
        _ => panic!("Expected alternative array for description"),
    }
}
