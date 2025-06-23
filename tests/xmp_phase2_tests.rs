//! Comprehensive tests for Phase 2 XMP functionality

use exif_oxide::xmp::{parse_xmp, XmpArray, XmpValue};
use std::collections::HashMap;

#[test]
fn test_ordered_array_seq() {
    let xmp = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/">
            <dc:creator>
                <rdf:Seq>
                    <rdf:li>Alice</rdf:li>
                    <rdf:li>Bob</rdf:li>
                    <rdf:li>Charlie</rdf:li>
                </rdf:Seq>
            </dc:creator>
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    let metadata = parse_xmp(xmp).unwrap();
    let dc = metadata.properties.get("dc").unwrap();
    let creator = dc.get("creator").unwrap();

    match creator {
        XmpValue::Array(XmpArray::Ordered(values)) => {
            assert_eq!(values.len(), 3);
            assert_eq!(values[0].as_str(), Some("Alice"));
            assert_eq!(values[1].as_str(), Some("Bob"));
            assert_eq!(values[2].as_str(), Some("Charlie"));
        }
        _ => panic!("Expected ordered array"),
    }
}

#[test]
fn test_unordered_array_bag() {
    let xmp = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/">
            <dc:subject>
                <rdf:Bag>
                    <rdf:li>nature</rdf:li>
                    <rdf:li>landscape</rdf:li>
                    <rdf:li>mountains</rdf:li>
                    <rdf:li>sunset</rdf:li>
                </rdf:Bag>
            </dc:subject>
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    let metadata = parse_xmp(xmp).unwrap();
    let dc = metadata.properties.get("dc").unwrap();
    let subject = dc.get("subject").unwrap();

    match subject {
        XmpValue::Array(XmpArray::Unordered(values)) => {
            assert_eq!(values.len(), 4);
            let subjects: Vec<&str> = values.iter().filter_map(|v| v.as_str()).collect();
            assert!(subjects.contains(&"nature"));
            assert!(subjects.contains(&"landscape"));
            assert!(subjects.contains(&"mountains"));
            assert!(subjects.contains(&"sunset"));
        }
        _ => panic!("Expected unordered array"),
    }
}

#[test]
fn test_language_alternatives() {
    let xmp = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/">
            <dc:title>
                <rdf:Alt>
                    <rdf:li xml:lang="x-default">Default Title</rdf:li>
                    <rdf:li xml:lang="en-US">American Title</rdf:li>
                    <rdf:li xml:lang="en-GB">British Title</rdf:li>
                    <rdf:li xml:lang="fr-FR">Titre Francais</rdf:li>
                    <rdf:li xml:lang="de-DE">Deutscher Titel</rdf:li>
                    <rdf:li xml:lang="es-ES">Titulo Espanol</rdf:li>
                </rdf:Alt>
            </dc:title>
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    let metadata = parse_xmp(xmp).unwrap();
    let dc = metadata.properties.get("dc").unwrap();
    let title = dc.get("title").unwrap();

    match title {
        XmpValue::Array(XmpArray::Alternative(alts)) => {
            assert_eq!(alts.len(), 6);

            // Check specific languages
            let lang_map: HashMap<String, String> = alts
                .iter()
                .map(|alt| (alt.lang.clone(), alt.value.as_str().unwrap().to_string()))
                .collect();

            assert_eq!(
                lang_map.get("x-default"),
                Some(&"Default Title".to_string())
            );
            assert_eq!(lang_map.get("en-US"), Some(&"American Title".to_string()));
            assert_eq!(lang_map.get("en-GB"), Some(&"British Title".to_string()));
            assert_eq!(lang_map.get("fr-FR"), Some(&"Titre Francais".to_string()));
            assert_eq!(lang_map.get("de-DE"), Some(&"Deutscher Titel".to_string()));
            assert_eq!(lang_map.get("es-ES"), Some(&"Titulo Espanol".to_string()));
        }
        _ => panic!("Expected alternative array"),
    }
}

#[test]
fn test_mixed_content_types() {
    let xmp = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/"
            xmlns:xmp="http://ns.adobe.com/xap/1.0/"
            xmp:Rating="5"
            xmp:CreateDate="2024-01-15">
            <dc:creator>
                <rdf:Seq>
                    <rdf:li>John Doe</rdf:li>
                </rdf:Seq>
            </dc:creator>
            <dc:rights>
                <rdf:Alt>
                    <rdf:li xml:lang="x-default">(C) 2024 John Doe</rdf:li>
                </rdf:Alt>
            </dc:rights>
            <dc:subject>
                <rdf:Bag>
                    <rdf:li>test</rdf:li>
                    <rdf:li>example</rdf:li>
                </rdf:Bag>
            </dc:subject>
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    let metadata = parse_xmp(xmp).unwrap();

    // Check simple attributes
    let xmp_props = metadata.properties.get("xmp").unwrap();
    assert_eq!(xmp_props.get("Rating").and_then(|v| v.as_str()), Some("5"));
    assert_eq!(
        xmp_props.get("CreateDate").and_then(|v| v.as_str()),
        Some("2024-01-15")
    );

    // Check arrays
    let dc = metadata.properties.get("dc").unwrap();

    // Creator (Seq)
    match dc.get("creator").unwrap() {
        XmpValue::Array(XmpArray::Ordered(values)) => {
            assert_eq!(values.len(), 1);
            assert_eq!(values[0].as_str(), Some("John Doe"));
        }
        _ => panic!("Expected ordered array for creator"),
    }

    // Rights (Alt)
    match dc.get("rights").unwrap() {
        XmpValue::Array(XmpArray::Alternative(alts)) => {
            assert_eq!(alts.len(), 1);
            assert_eq!(alts[0].lang, "x-default");
            assert_eq!(alts[0].value.as_str(), Some("(C) 2024 John Doe"));
        }
        _ => panic!("Expected alternative array for rights"),
    }

    // Subject (Bag)
    match dc.get("subject").unwrap() {
        XmpValue::Array(XmpArray::Unordered(values)) => {
            assert_eq!(values.len(), 2);
        }
        _ => panic!("Expected unordered array for subject"),
    }
}

#[test]
fn test_empty_arrays() {
    let xmp = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/">
            <dc:creator>
                <rdf:Seq/>
            </dc:creator>
            <dc:subject>
                <rdf:Bag/>
            </dc:subject>
            <dc:title>
                <rdf:Alt/>
            </dc:title>
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    let metadata = parse_xmp(xmp).unwrap();
    let dc = metadata.properties.get("dc").unwrap();

    // Empty Seq
    match dc.get("creator").unwrap() {
        XmpValue::Array(XmpArray::Ordered(values)) => {
            assert_eq!(values.len(), 0);
        }
        _ => panic!("Expected ordered array"),
    }

    // Empty Bag
    match dc.get("subject").unwrap() {
        XmpValue::Array(XmpArray::Unordered(values)) => {
            assert_eq!(values.len(), 0);
        }
        _ => panic!("Expected unordered array"),
    }

    // Empty Alt
    match dc.get("title").unwrap() {
        XmpValue::Array(XmpArray::Alternative(alts)) => {
            assert_eq!(alts.len(), 0);
        }
        _ => panic!("Expected alternative array"),
    }
}

#[test]
fn test_multiple_namespaces() {
    let xmp = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/"
            xmlns:photoshop="http://ns.adobe.com/photoshop/1.0/"
            xmlns:xmp="http://ns.adobe.com/xap/1.0/"
            xmlns:tiff="http://ns.adobe.com/tiff/1.0/"
            xmlns:exif="http://ns.adobe.com/exif/1.0/">
            <dc:format>image/jpeg</dc:format>
            <photoshop:City>New York</photoshop:City>
            <xmp:CreatorTool>Test Tool</xmp:CreatorTool>
            <tiff:Make>Test Camera</tiff:Make>
            <exif:DateTimeOriginal>2024-01-15T12:00:00</exif:DateTimeOriginal>
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    let metadata = parse_xmp(xmp).unwrap();

    // Check namespaces are registered
    assert!(metadata.namespaces.contains_key("dc"));
    assert!(metadata.namespaces.contains_key("photoshop"));
    assert!(metadata.namespaces.contains_key("xmp"));
    assert!(metadata.namespaces.contains_key("tiff"));
    assert!(metadata.namespaces.contains_key("exif"));

    // Check properties
    assert_eq!(
        metadata
            .properties
            .get("dc")
            .unwrap()
            .get("format")
            .and_then(|v| v.as_str()),
        Some("image/jpeg")
    );
    assert_eq!(
        metadata
            .properties
            .get("photoshop")
            .unwrap()
            .get("City")
            .and_then(|v| v.as_str()),
        Some("New York")
    );
    assert_eq!(
        metadata
            .properties
            .get("xmp")
            .unwrap()
            .get("CreatorTool")
            .and_then(|v| v.as_str()),
        Some("Test Tool")
    );
    assert_eq!(
        metadata
            .properties
            .get("tiff")
            .unwrap()
            .get("Make")
            .and_then(|v| v.as_str()),
        Some("Test Camera")
    );
    assert_eq!(
        metadata
            .properties
            .get("exif")
            .unwrap()
            .get("DateTimeOriginal")
            .and_then(|v| v.as_str()),
        Some("2024-01-15T12:00:00")
    );
}

#[test]
fn test_special_characters_in_values() {
    let xmp = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/">
            <dc:description>
                <rdf:Alt>
                    <rdf:li xml:lang="x-default">Special chars: &lt;&gt;&amp;&quot;&apos;</rdf:li>
                    <rdf:li xml:lang="en">Line 1
Line 2
Line 3</rdf:li>
                </rdf:Alt>
            </dc:description>
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    let metadata = parse_xmp(xmp).unwrap();
    let dc = metadata.properties.get("dc").unwrap();
    let description = dc.get("description").unwrap();

    match description {
        XmpValue::Array(XmpArray::Alternative(alts)) => {
            assert_eq!(alts.len(), 2);

            // Check XML entity handling (entities preserved in quick-xml)
            let default_desc = alts.iter().find(|a| a.lang == "x-default").unwrap();
            assert_eq!(
                default_desc.value.as_str(),
                Some("Special chars: &lt;&gt;&amp;&quot;&apos;")
            );

            // Check multiline handling
            let en_desc = alts.iter().find(|a| a.lang == "en").unwrap();
            assert!(en_desc.value.as_str().unwrap().contains("Line 1"));
            assert!(en_desc.value.as_str().unwrap().contains("Line 3"));
        }
        _ => panic!("Expected alternative array"),
    }
}

#[test]
fn test_utf16_encoded_xmp() {
    // Create UTF-16 BE encoded XMP
    let xmp_str = r#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/">
            <dc:title>
                <rdf:Alt>
                    <rdf:li xml:lang="x-default">UTF-16 Test</rdf:li>
                </rdf:Alt>
            </dc:title>
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    // Convert to UTF-16 BE
    let mut utf16_be = Vec::new();
    for ch in xmp_str.encode_utf16() {
        utf16_be.extend_from_slice(&ch.to_be_bytes());
    }

    let metadata = parse_xmp(&utf16_be).unwrap();
    let dc = metadata.properties.get("dc").unwrap();
    let title = dc.get("title").unwrap();

    match title {
        XmpValue::Array(XmpArray::Alternative(alts)) => {
            assert_eq!(alts.len(), 1);
            assert_eq!(alts[0].value.as_str(), Some("UTF-16 Test"));
        }
        _ => panic!("Expected alternative array"),
    }
}

#[test]
fn test_nested_elements_within_arrays() {
    // Some XMP uses nested structures within array items
    let xmp = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:xmpMM="http://ns.adobe.com/xap/1.0/mm/">
            <xmpMM:History>
                <rdf:Seq>
                    <rdf:li>Event 1</rdf:li>
                    <rdf:li>Event 2</rdf:li>
                    <rdf:li>Event 3</rdf:li>
                </rdf:Seq>
            </xmpMM:History>
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    let metadata = parse_xmp(xmp).unwrap();
    let xmp_mm = metadata.properties.get("xmpMM").unwrap();
    let history = xmp_mm.get("History").unwrap();

    match history {
        XmpValue::Array(XmpArray::Ordered(values)) => {
            assert_eq!(values.len(), 3);
            assert_eq!(values[0].as_str(), Some("Event 1"));
            assert_eq!(values[1].as_str(), Some("Event 2"));
            assert_eq!(values[2].as_str(), Some("Event 3"));
        }
        _ => panic!("Expected ordered array"),
    }
}

#[test]
fn test_whitespace_handling() {
    let xmp = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/">
            <dc:creator>
                <rdf:Seq>
                    <rdf:li>   Spaces Before and After   </rdf:li>
                    <rdf:li>
                        Text with newlines
                    </rdf:li>
                    <rdf:li>	Tabs	Around	</rdf:li>
                </rdf:Seq>
            </dc:creator>
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    let metadata = parse_xmp(xmp).unwrap();
    let dc = metadata.properties.get("dc").unwrap();
    let creator = dc.get("creator").unwrap();

    match creator {
        XmpValue::Array(XmpArray::Ordered(values)) => {
            assert_eq!(values.len(), 3);
            // Whitespace should be trimmed
            assert_eq!(values[0].as_str(), Some("Spaces Before and After"));
            assert_eq!(values[1].as_str(), Some("Text with newlines"));
            assert_eq!(values[2].as_str(), Some("Tabs	Around"));
        }
        _ => panic!("Expected ordered array"),
    }
}

#[test]
fn test_rdf_about_variations() {
    // Test different ways rdf:about can appear
    let xmp = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about="uuid:12345678-1234-1234-1234-123456789012"
            xmlns:dc="http://purl.org/dc/elements/1.1/"
            dc:format="image/jpeg">
        </rdf:Description>
        <rdf:Description about=""
            xmlns:xmp="http://ns.adobe.com/xap/1.0/"
            xmp:Rating="5">
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    let metadata = parse_xmp(xmp).unwrap();

    // Both descriptions should be parsed
    assert_eq!(
        metadata
            .properties
            .get("dc")
            .unwrap()
            .get("format")
            .and_then(|v| v.as_str()),
        Some("image/jpeg")
    );
    assert_eq!(
        metadata
            .properties
            .get("xmp")
            .unwrap()
            .get("Rating")
            .and_then(|v| v.as_str()),
        Some("5")
    );
}
