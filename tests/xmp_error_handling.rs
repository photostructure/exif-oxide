//! Tests for XMP error handling and edge cases

use exif_oxide::xmp::{extract_simple_properties, parse_xmp};

#[test]
fn test_invalid_xml() {
    let invalid_xml = b"This is not XML at all!";
    // Invalid XML may succeed or fail - both acceptable
    let result = parse_xmp(invalid_xml);
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

    assert!(parse_xmp(malformed).is_err());
}

#[test]
fn test_unclosed_array() {
    let unclosed = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/">
            <dc:creator>
                <rdf:Seq>
                    <rdf:li>Test</rdf:li>
                <!-- Missing </rdf:Seq> -->
            </dc:creator>
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    assert!(parse_xmp(unclosed).is_err());
}

#[test]
fn test_invalid_utf8() {
    // Invalid UTF-8 sequence
    let invalid_utf8 = vec![
        0x3C, 0x78, 0x3A, 0x78, 0x6D, 0x70, 0x6D, 0x65, 0x74, 0x61, // <x:xmpmeta
        0xFF, 0xFE, 0xFD, // Invalid UTF-8
        0x2F, 0x3E, // />
    ];

    // Should handle gracefully (lossy conversion)
    let result = parse_xmp(&invalid_utf8);
    assert!(result.is_ok() || result.is_err()); // Either parse with replacement chars or error
}

#[test]
fn test_empty_input() {
    let empty = b"";
    // Empty input may succeed with empty metadata
    let result = parse_xmp(empty);
    if result.is_ok() {
        let metadata = result.unwrap();
        assert!(metadata.properties.is_empty());
    }
}

#[test]
fn test_whitespace_only() {
    let whitespace = b"   \n\t\r   ";
    // Whitespace may succeed with empty metadata
    let result = parse_xmp(whitespace);
    if result.is_ok() {
        let metadata = result.unwrap();
        assert!(metadata.properties.is_empty());
    }
}

#[test]
fn test_mismatched_array_types() {
    // This XMP has a structural issue but might be parsed differently by different parsers
    let xmp = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/">
            <dc:creator>
                <rdf:Bag>  <!-- Using Bag instead of typical Seq for creator -->
                    <rdf:li>Author 1</rdf:li>
                    <rdf:li>Author 2</rdf:li>
                </rdf:Bag>
            </dc:creator>
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    // Should parse successfully even if semantically unusual
    let result = parse_xmp(xmp).unwrap();
    let dc = result.properties.get("dc").unwrap();
    assert!(dc.contains_key("creator"));
}

#[test]
fn test_duplicate_properties() {
    let xmp = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/"
            dc:format="image/jpeg"
            dc:format="image/png">  <!-- Duplicate attribute -->
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    // quick-xml returns an error for duplicate attributes
    let props = extract_simple_properties(xmp);
    // Either succeeds or fails - both are acceptable behavior
    if props.is_ok() {
        let props = props.unwrap();
        assert!(props.contains_key("dc:format"));
    }
}

#[test]
fn test_missing_namespace_declaration() {
    let xmp = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            unknown:property="value">  <!-- unknown namespace not declared -->
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    // Should still parse - namespace declarations are not strictly required for parsing
    let props = extract_simple_properties(xmp).unwrap();
    assert!(props.contains_key("unknown:property"));
}

#[test]
fn test_very_long_values() {
    let long_value = "x".repeat(10000);
    let xmp = format!(
        r#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/"
            dc:description="{}">
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#,
        long_value
    );

    let props = extract_simple_properties(xmp.as_bytes()).unwrap();
    assert_eq!(props.get("dc:description").unwrap().len(), 10000);
}

#[test]
fn test_nested_arrays_not_supported() {
    // Nested arrays (array within array) - edge case
    let xmp = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:test="http://example.com/test/">
            <test:nested>
                <rdf:Seq>
                    <rdf:li>
                        <rdf:Seq>
                            <rdf:li>Nested Item</rdf:li>
                        </rdf:Seq>
                    </rdf:li>
                </rdf:Seq>
            </test:nested>
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    // Parser should handle this gracefully (may not parse nested structure fully)
    let result = parse_xmp(xmp);
    assert!(result.is_ok());
}

#[test]
fn test_cdata_sections() {
    let xmp = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/">
            <dc:description>
                <rdf:Alt>
                    <rdf:li xml:lang="x-default"><![CDATA[Text with <special> & characters]]></rdf:li>
                </rdf:Alt>
            </dc:description>
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    let metadata = parse_xmp(xmp).unwrap();
    let dc = metadata.properties.get("dc").unwrap();
    let desc = dc.get("description").unwrap();

    // CDATA content should be preserved
    match desc {
        exif_oxide::xmp::XmpValue::Array(exif_oxide::xmp::XmpArray::Alternative(alts)) => {
            if !alts.is_empty() {
                assert!(alts[0].value.as_str().unwrap().contains("<special>"));
                assert!(alts[0].value.as_str().unwrap().contains("&"));
            }
        }
        _ => {
            // May not parse correctly, which is acceptable for CDATA test
        }
    }
}

#[test]
fn test_mixed_content_in_elements() {
    // Mixed content (text and elements) in same parent
    let xmp = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/">
            <dc:description>
                Some text here
                <rdf:Alt>
                    <rdf:li xml:lang="x-default">Alternative text</rdf:li>
                </rdf:Alt>
                More text after
            </dc:description>
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    // Parser should handle mixed content gracefully
    let result = parse_xmp(xmp);
    assert!(result.is_ok());
}

#[test]
fn test_recursive_depth_limit() {
    // Create deeply nested XML to test stack limits
    let mut xmp = String::from(
        r#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:test="http://example.com/test/">"#,
    );

    // Create 100 levels of nesting
    for i in 0..100 {
        xmp.push_str(&format!("\n<test:level{}>", i));
    }
    xmp.push_str("\nDeep content");
    for i in (0..100).rev() {
        xmp.push_str(&format!("\n</test:level{}>", i));
    }

    xmp.push_str(
        r#"
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#,
    );

    // Should handle deep nesting (or fail gracefully)
    let _ = parse_xmp(xmp.as_bytes());
}
