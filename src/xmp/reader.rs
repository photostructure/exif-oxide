//! XMP reading from JPEG files

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek};
use std::path::Path;

use crate::core::jpeg::{find_metadata_segments, XmpSegment};
use crate::xmp::{extract_simple_properties, ExtendedXmp, XmpError, XmpPacket};

/// Read XMP metadata from a JPEG file
pub fn read_xmp_from_jpeg<P: AsRef<Path>>(path: P) -> Result<Option<XmpPacket>, XmpError> {
    let mut file = File::open(path)?;
    read_xmp_from_reader(&mut file)
}

/// Read XMP metadata from a reader containing JPEG data
pub fn read_xmp_from_reader<R: Read + Seek>(reader: &mut R) -> Result<Option<XmpPacket>, XmpError> {
    let metadata = find_metadata_segments(reader)
        .map_err(|e| XmpError::XmlError(format!("JPEG parsing error: {}", e)))?;

    if metadata.xmp.is_empty() {
        return Ok(None);
    }

    // Separate standard and extended XMP segments
    let (standard_segments, extended_segments): (Vec<_>, Vec<_>) =
        metadata.xmp.into_iter().partition(|seg| !seg.is_extended);

    // Process standard XMP (should only be one)
    if standard_segments.is_empty() {
        return Ok(None);
    }

    let mut packet = XmpPacket::new(standard_segments[0].data.clone());

    // Process extended XMP if present
    if !extended_segments.is_empty() {
        packet.extended = Some(assemble_extended_xmp(extended_segments)?);
    }

    Ok(Some(packet))
}

/// Assemble extended XMP from multiple segments
fn assemble_extended_xmp(segments: Vec<XmpSegment>) -> Result<ExtendedXmp, XmpError> {
    if segments.is_empty() {
        return Err(XmpError::ExtendedXmpError(
            "No extended XMP segments".to_string(),
        ));
    }

    // For Phase 1, we'll just concatenate the data
    // In Phase 3, we'll properly parse GUID, offsets, and validate MD5
    let mut data = Vec::new();
    for segment in segments {
        data.extend_from_slice(&segment.data);
    }

    Ok(ExtendedXmp {
        guid: String::new(), // Will parse in Phase 3
        total_length: data.len() as u32,
        md5: None,
        data,
    })
}

/// Extract simple properties from a JPEG file's XMP data
pub fn extract_xmp_properties<P: AsRef<Path>>(
    path: P,
) -> Result<HashMap<String, String>, XmpError> {
    let xmp = read_xmp_from_jpeg(path)?;

    match xmp {
        Some(packet) => extract_simple_properties(&packet.standard),
        None => Ok(HashMap::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_xmp_from_jpeg_no_xmp() {
        // Minimal JPEG with no XMP
        let data = vec![
            0xFF, 0xD8, // SOI
            0xFF, 0xD9, // EOI
        ];

        let mut cursor = Cursor::new(data);
        let result = read_xmp_from_reader(&mut cursor).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_read_xmp_from_jpeg_with_xmp() {
        let xmp_sig = b"http://ns.adobe.com/xap/1.0/\0";
        let xmp_data = br#"<?xpacket begin="" id="test"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/"
            dc:title="Test Image">
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#;

        let mut data = vec![];
        data.extend_from_slice(&[0xFF, 0xD8]); // SOI
        data.extend_from_slice(&[0xFF, 0xE1]); // APP1 marker
        let length = (2 + xmp_sig.len() + xmp_data.len()) as u16;
        data.extend_from_slice(&length.to_be_bytes()); // Length
        data.extend_from_slice(xmp_sig); // XMP signature
        data.extend_from_slice(xmp_data); // XMP data
        data.extend_from_slice(&[0xFF, 0xD9]); // EOI

        let mut cursor = Cursor::new(data);
        let result = read_xmp_from_reader(&mut cursor).unwrap();
        assert!(result.is_some());

        let packet = result.unwrap();
        assert_eq!(packet.standard, xmp_data);
        assert!(packet.extended.is_none());
    }
}
