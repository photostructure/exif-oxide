//! Phase 2 XMP parser with proper empty tag handling

use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::collections::HashMap;

use crate::xmp::{LanguageAlternative, XmpArray, XmpError, XmpMetadata, XmpValue};

/// Parse XMP packet data into structured metadata
pub fn parse_xmp(data: &[u8]) -> Result<XmpMetadata, XmpError> {
    // Handle UTF-16 encoded XMP (some ExifTool test files use this)
    let xml_string = if data.len() >= 2 && data[0] == 0x00 {
        // Likely UTF-16 BE
        decode_utf16_be(data)?
    } else if data.len() >= 2 && data[1] == 0x00 {
        // Likely UTF-16 LE
        decode_utf16_le(data)?
    } else {
        // Assume UTF-8
        String::from_utf8_lossy(data).to_string()
    };

    let mut reader = Reader::from_reader(xml_string.as_bytes());
    reader.config_mut().trim_text(true);

    let mut metadata = XmpMetadata::new();
    let mut buf = Vec::new();

    // Parsing state
    let mut element_stack: Vec<ElementContext> = Vec::new();
    let mut current_array: Option<ArrayContext> = None;
    let mut pending_value: Option<String> = None;
    let mut pending_lang: Option<String> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag_name = reader
                    .decoder()
                    .decode(e.name().as_ref())
                    .map_err(|err| XmpError::XmlError(format!("Tag decode error: {}", err)))?
                    .to_string();

                handle_start_tag(
                    &reader,
                    &tag_name,
                    e,
                    &mut metadata,
                    &mut element_stack,
                    &mut current_array,
                    &mut pending_lang,
                )?;
            }

            Ok(Event::Empty(ref e)) => {
                // Handle self-closing tags like <rdf:Seq/>
                let tag_name = reader
                    .decoder()
                    .decode(e.name().as_ref())
                    .map_err(|err| XmpError::XmlError(format!("Tag decode error: {}", err)))?
                    .to_string();

                // Extract namespace declarations
                extract_namespace_declarations(&reader, e, &mut metadata.namespaces)?;

                match tag_name.as_ref() {
                    "rdf:Seq" | "rdf:Bag" | "rdf:Alt" => {
                        // Empty array
                        if let Some(elem) = element_stack.last() {
                            let value = match tag_name.as_ref() {
                                "rdf:Seq" => XmpValue::Array(XmpArray::Ordered(Vec::new())),
                                "rdf:Bag" => XmpValue::Array(XmpArray::Unordered(Vec::new())),
                                "rdf:Alt" => XmpValue::Array(XmpArray::Alternative(Vec::new())),
                                _ => unreachable!(),
                            };

                            let ns_props = metadata
                                .properties
                                .entry(elem.namespace.clone())
                                .or_default();
                            ns_props.insert(elem.property.clone(), value);
                        }
                    }
                    _ => {
                        // Other empty elements - treat as empty string
                        if element_stack.last().is_some() && tag_name.contains(':') {
                            if let Some((ns, prop)) = tag_name.split_once(':') {
                                let ns_props =
                                    metadata.properties.entry(ns.to_string()).or_default();
                                ns_props.insert(prop.to_string(), XmpValue::Simple(String::new()));
                            }
                        }
                    }
                }
            }

            Ok(Event::Text(e)) => {
                let text = reader
                    .decoder()
                    .decode(&e)
                    .map_err(|e| XmpError::XmlError(format!("UTF-8 error: {}", e)))?;
                let text = text.trim();

                if !text.is_empty() {
                    pending_value = Some(text.to_string());
                }
            }

            Ok(Event::End(ref e)) => {
                let tag_name = reader
                    .decoder()
                    .decode(e.name().as_ref())
                    .map_err(|err| XmpError::XmlError(format!("Tag decode error: {}", err)))?
                    .to_string();

                handle_end_tag(
                    &tag_name,
                    &mut metadata,
                    &mut element_stack,
                    &mut current_array,
                    &mut pending_value,
                )?;
            }

            Ok(Event::Eof) => break,

            Err(e) => return Err(XmpError::XmlError(format!("XML parsing error: {}", e))),

            _ => {} // Ignore other events
        }

        buf.clear();
    }

    Ok(metadata)
}

fn handle_start_tag(
    reader: &Reader<&[u8]>,
    tag_name: &str,
    e: &quick_xml::events::BytesStart,
    metadata: &mut XmpMetadata,
    element_stack: &mut Vec<ElementContext>,
    current_array: &mut Option<ArrayContext>,
    pending_lang: &mut Option<String>,
) -> Result<(), XmpError> {
    // Extract namespace declarations
    extract_namespace_declarations(reader, e, &mut metadata.namespaces)?;

    match tag_name {
        "rdf:RDF" | "x:xmpmeta" => {
            // Container elements - just continue
        }
        "rdf:Description" => {
            // Parse attributes as simple properties
            for attr in e.attributes() {
                let attr =
                    attr.map_err(|e| XmpError::XmlError(format!("Attribute error: {}", e)))?;
                let key = reader
                    .decoder()
                    .decode(attr.key.as_ref())
                    .map_err(|e| XmpError::XmlError(format!("UTF-8 error: {}", e)))?;
                let value = reader
                    .decoder()
                    .decode(&attr.value)
                    .map_err(|e| XmpError::XmlError(format!("UTF-8 error: {}", e)))?;

                if !key.starts_with("xmlns:") && key != "rdf:about" && key != "about" {
                    if let Some((ns, prop)) = key.split_once(':') {
                        let ns_props = metadata.properties.entry(ns.to_string()).or_default();
                        ns_props.insert(prop.to_string(), XmpValue::Simple(value.to_string()));
                    }
                }
            }
        }
        "rdf:Seq" => {
            if let Some(elem) = element_stack.last() {
                *current_array = Some(ArrayContext {
                    namespace: elem.namespace.clone(),
                    property: elem.property.clone(),
                    array_type: ArrayType::Seq,
                    values: Vec::new(),
                });
            }
        }
        "rdf:Bag" => {
            if let Some(elem) = element_stack.last() {
                *current_array = Some(ArrayContext {
                    namespace: elem.namespace.clone(),
                    property: elem.property.clone(),
                    array_type: ArrayType::Bag,
                    values: Vec::new(),
                });
            }
        }
        "rdf:Alt" => {
            if let Some(elem) = element_stack.last() {
                *current_array = Some(ArrayContext {
                    namespace: elem.namespace.clone(),
                    property: elem.property.clone(),
                    array_type: ArrayType::Alt,
                    values: Vec::new(),
                });
            }
        }
        "rdf:li" => {
            // List item - may have xml:lang attribute for Alt arrays
            *pending_lang = None;
            for attr in e.attributes() {
                let attr =
                    attr.map_err(|e| XmpError::XmlError(format!("Attribute error: {}", e)))?;
                let key = reader
                    .decoder()
                    .decode(attr.key.as_ref())
                    .map_err(|e| XmpError::XmlError(format!("UTF-8 error: {}", e)))?;

                if key == "xml:lang" {
                    *pending_lang = Some(
                        reader
                            .decoder()
                            .decode(&attr.value)
                            .map_err(|e| XmpError::XmlError(format!("UTF-8 error: {}", e)))?
                            .to_string(),
                    );
                }
            }

            element_stack.push(ElementContext {
                namespace: String::new(),
                property: String::new(),
                lang: pending_lang.clone(),
            });
        }
        _ => {
            // Property element
            if tag_name.contains(':') {
                if let Some((ns, prop)) = tag_name.split_once(':') {
                    element_stack.push(ElementContext {
                        namespace: ns.to_string(),
                        property: prop.to_string(),
                        lang: None,
                    });
                }
            }
        }
    }

    Ok(())
}

fn handle_end_tag(
    tag_name: &str,
    metadata: &mut XmpMetadata,
    element_stack: &mut Vec<ElementContext>,
    current_array: &mut Option<ArrayContext>,
    pending_value: &mut Option<String>,
) -> Result<(), XmpError> {
    match tag_name {
        "rdf:Seq" | "rdf:Bag" | "rdf:Alt" => {
            // End of array - store it
            if let Some(array_ctx) = current_array.take() {
                let value = match array_ctx.array_type {
                    ArrayType::Seq => XmpValue::Array(XmpArray::Ordered(array_ctx.values)),
                    ArrayType::Bag => XmpValue::Array(XmpArray::Unordered(array_ctx.values)),
                    ArrayType::Alt => {
                        // Convert to language alternatives
                        let mut alts = Vec::new();
                        for (i, value) in array_ctx.values.into_iter().enumerate() {
                            if let XmpValue::Struct(map) = value {
                                if let (
                                    Some(XmpValue::Simple(lang)),
                                    Some(XmpValue::Simple(text)),
                                ) = (map.get("_lang"), map.get("_value"))
                                {
                                    alts.push(LanguageAlternative {
                                        lang: lang.clone(),
                                        value: XmpValue::Simple(text.clone()),
                                    });
                                }
                            } else {
                                // Fallback for non-language alternatives
                                alts.push(LanguageAlternative {
                                    lang: if i == 0 {
                                        "x-default".to_string()
                                    } else {
                                        format!("item{}", i)
                                    },
                                    value,
                                });
                            }
                        }
                        XmpValue::Array(XmpArray::Alternative(alts))
                    }
                };

                let ns_props = metadata.properties.entry(array_ctx.namespace).or_default();
                ns_props.insert(array_ctx.property, value);
            }
        }
        "rdf:li" => {
            // End of list item
            if let Some(value) = pending_value.take() {
                if let Some(ref mut array) = current_array {
                    if matches!(array.array_type, ArrayType::Alt) {
                        // For Alt arrays, store value with language
                        let lang = element_stack
                            .last()
                            .and_then(|ctx| ctx.lang.clone())
                            .unwrap_or_else(|| "x-default".to_string());
                        array.values.push(XmpValue::Struct({
                            let mut m = HashMap::new();
                            m.insert("_lang".to_string(), XmpValue::Simple(lang));
                            m.insert("_value".to_string(), XmpValue::Simple(value));
                            m
                        }));
                    } else {
                        array.values.push(XmpValue::Simple(value));
                    }
                }
            }
            element_stack.pop();
        }
        _ => {
            // End of property element
            if tag_name.contains(':') {
                if let Some(ctx) = element_stack.pop() {
                    if current_array.is_none() {
                        // Not in an array - store as simple property
                        if let Some(value) = pending_value.take() {
                            let ns_props = metadata.properties.entry(ctx.namespace).or_default();
                            ns_props.insert(ctx.property, XmpValue::Simple(value));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Extract namespace declarations from element attributes
fn extract_namespace_declarations(
    reader: &Reader<&[u8]>,
    element: &quick_xml::events::BytesStart,
    namespaces: &mut HashMap<String, String>,
) -> Result<(), XmpError> {
    for attr in element.attributes() {
        let attr = attr.map_err(|e| XmpError::XmlError(format!("Attribute error: {}", e)))?;
        let key = reader
            .decoder()
            .decode(attr.key.as_ref())
            .map_err(|e| XmpError::XmlError(format!("UTF-8 error: {}", e)))?;

        if key.starts_with("xmlns:") {
            let prefix = key.strip_prefix("xmlns:").unwrap().to_string();
            let uri = reader
                .decoder()
                .decode(&attr.value)
                .map_err(|e| XmpError::XmlError(format!("UTF-8 error: {}", e)))?
                .to_string();
            namespaces.insert(prefix, uri);
        }
    }
    Ok(())
}

#[derive(Debug)]
struct ElementContext {
    namespace: String,
    property: String,
    lang: Option<String>,
}

#[derive(Debug)]
struct ArrayContext {
    namespace: String,
    property: String,
    array_type: ArrayType,
    values: Vec<XmpValue>,
}

#[derive(Debug, Clone, Copy)]
enum ArrayType {
    Seq,
    Bag,
    Alt,
}

/// Decode UTF-16 BE to String
fn decode_utf16_be(data: &[u8]) -> Result<String, XmpError> {
    if data.len() % 2 != 0 {
        return Err(XmpError::XmlError(
            "Odd number of bytes for UTF-16".to_string(),
        ));
    }

    let mut utf16_chars = Vec::new();
    for chunk in data.chunks_exact(2) {
        let code_unit = u16::from_be_bytes([chunk[0], chunk[1]]);
        utf16_chars.push(code_unit);
    }

    String::from_utf16(&utf16_chars)
        .map_err(|e| XmpError::XmlError(format!("UTF-16 decode error: {}", e)))
}

/// Decode UTF-16 LE to String
fn decode_utf16_le(data: &[u8]) -> Result<String, XmpError> {
    if data.len() % 2 != 0 {
        return Err(XmpError::XmlError(
            "Odd number of bytes for UTF-16".to_string(),
        ));
    }

    let mut utf16_chars = Vec::new();
    for chunk in data.chunks_exact(2) {
        let code_unit = u16::from_le_bytes([chunk[0], chunk[1]]);
        utf16_chars.push(code_unit);
    }

    String::from_utf16(&utf16_chars)
        .map_err(|e| XmpError::XmlError(format!("UTF-16 decode error: {}", e)))
}

/// Extract simple key-value pairs from XMP for Phase 1 compatibility
pub fn extract_simple_properties(xmp_data: &[u8]) -> Result<HashMap<String, String>, XmpError> {
    let metadata = parse_xmp(xmp_data)?;
    let mut simple_props = HashMap::new();

    // Flatten the namespace/property structure into simple strings
    for (namespace, properties) in metadata.properties {
        for (prop_name, value) in properties {
            if let XmpValue::Simple(text) = value {
                let full_name = format!("{}:{}", namespace, prop_name);
                simple_props.insert(full_name, text);
            }
        }
    }

    Ok(simple_props)
}

#[cfg(test)]
mod tests {
    use super::*;

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

        // Check empty Seq
        match dc.get("creator").unwrap() {
            XmpValue::Array(XmpArray::Ordered(values)) => {
                assert_eq!(values.len(), 0);
            }
            _ => panic!("Expected ordered array"),
        }

        // Check empty Bag
        match dc.get("subject").unwrap() {
            XmpValue::Array(XmpArray::Unordered(values)) => {
                assert_eq!(values.len(), 0);
            }
            _ => panic!("Expected unordered array"),
        }

        // Check empty Alt
        match dc.get("title").unwrap() {
            XmpValue::Array(XmpArray::Alternative(alts)) => {
                assert_eq!(alts.len(), 0);
            }
            _ => panic!("Expected alternative array"),
        }
    }

    #[test]
    fn test_nested_arrays() {
        let xmp = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/">
            <dc:creator>
                <rdf:Seq>
                    <rdf:li>Creator 1</rdf:li>
                    <rdf:li>Creator 2</rdf:li>
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
                assert_eq!(values.len(), 2);
                assert_eq!(values[0].as_str(), Some("Creator 1"));
                assert_eq!(values[1].as_str(), Some("Creator 2"));
            }
            _ => panic!("Expected ordered array"),
        }
    }
}
