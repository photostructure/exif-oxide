//! XMP XML parsing implementation

use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::collections::HashMap;

use crate::xmp::{XmpError, XmpMetadata, XmpValue};

/// Parse XMP packet data into structured metadata
pub fn parse_xmp(data: &[u8]) -> Result<XmpMetadata, XmpError> {
    let mut reader = Reader::from_reader(data);
    reader.config_mut().trim_text(true);

    let mut metadata = XmpMetadata::new();
    let mut buf = Vec::new();

    // Track current parsing context
    let mut current_namespace: Option<String> = None;
    let mut _in_rdf = false;
    let mut in_description = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                match tag_name.as_ref() {
                    "rdf:RDF" => {
                        _in_rdf = true;
                        // Extract namespace declarations
                        for attr in e.attributes() {
                            let attr = attr.map_err(|e| {
                                XmpError::XmlError(format!("Attribute error: {}", e))
                            })?;
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
                                metadata.namespaces.insert(prefix, uri);
                            }
                        }
                    }
                    "rdf:Description" => {
                        in_description = true;
                        // Parse simple attributes as properties
                        for attr in e.attributes() {
                            let attr = attr.map_err(|e| {
                                XmpError::XmlError(format!("Attribute error: {}", e))
                            })?;
                            let key = reader
                                .decoder()
                                .decode(attr.key.as_ref())
                                .map_err(|e| XmpError::XmlError(format!("UTF-8 error: {}", e)))?;
                            let value = reader
                                .decoder()
                                .decode(&attr.value)
                                .map_err(|e| XmpError::XmlError(format!("UTF-8 error: {}", e)))?;

                            // Handle namespace declarations
                            if key.starts_with("xmlns:") {
                                let prefix = key.strip_prefix("xmlns:").unwrap().to_string();
                                metadata.namespaces.insert(prefix, value.to_string());
                            }
                            // Skip rdf:about attribute
                            else if key != "rdf:about" {
                                // Parse namespace:property format
                                if let Some((ns, prop)) = key.split_once(':') {
                                    let ns_props = metadata
                                        .properties
                                        .entry(ns.to_string())
                                        .or_insert_with(HashMap::new);
                                    ns_props.insert(
                                        prop.to_string(),
                                        XmpValue::Simple(value.to_string()),
                                    );
                                }
                            }
                        }
                    }
                    _ => {
                        // Handle property elements
                        if in_description && tag_name.contains(':') {
                            if let Some((ns, _prop)) = tag_name.split_once(':') {
                                current_namespace = Some(ns.to_string());
                            }
                        }
                    }
                }
            }

            Ok(Event::Text(e)) => {
                // Handle text content within property elements
                if let Some(ref _ns) = current_namespace {
                    let text = reader
                        .decoder()
                        .decode(&e)
                        .map_err(|e| XmpError::XmlError(format!("UTF-8 error: {}", e)))?;

                    if !text.trim().is_empty() {
                        // This is a simplified approach - we'll enhance it in Phase 2
                        // For now, just store the text as a simple value
                    }
                }
            }

            Ok(Event::End(ref e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                match tag_name.as_ref() {
                    "rdf:RDF" => _in_rdf = false,
                    "rdf:Description" => in_description = false,
                    _ => {
                        if tag_name.contains(':') {
                            current_namespace = None;
                        }
                    }
                }
            }

            Ok(Event::Eof) => break,

            Err(e) => return Err(XmpError::XmlError(format!("XML parsing error: {}", e))),

            _ => {} // Ignore other events
        }

        buf.clear();
    }

    Ok(metadata)
}

/// Extract simple key-value pairs from XMP for Phase 1
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
    fn test_parse_simple_xmp() {
        let xmp_data = br#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/"
            xmlns:xmp="http://ns.adobe.com/xap/1.0/"
            dc:format="image/jpeg"
            xmp:CreatorTool="Adobe Photoshop CS6">
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#;

        let props = extract_simple_properties(xmp_data).unwrap();
        assert_eq!(props.get("dc:format"), Some(&"image/jpeg".to_string()));
        assert_eq!(
            props.get("xmp:CreatorTool"),
            Some(&"Adobe Photoshop CS6".to_string())
        );
    }

    #[test]
    fn test_parse_empty_xmp() {
        let xmp_data = br#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about="">
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#;

        let props = extract_simple_properties(xmp_data).unwrap();
        assert!(props.is_empty());
    }
}
