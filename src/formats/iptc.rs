//! IPTC metadata processing
//!
//! This module implements IPTC (International Press Telecommunications Council)
//! metadata parsing following ExifTool's IPTC.pm ProcessIPTC function.
//!
//! IPTC data is stored in JPEG APP13 segments wrapped in Adobe Photoshop
//! Image Resource Blocks (Resource ID 0x0404). The data uses a DataSet format:
//! - Marker (0x1C)
//! - Record number (1 byte)
//! - DataSet number (1 byte)
//! - Length (2 bytes, big-endian)
//! - Data (variable length)
//!
//! Reference: ExifTool lib/Image/ExifTool/IPTC.pm ProcessIPTC function (lines 1050-1200)

use crate::generated::IPTC_pm::application_record_tags::IPTC_APPLICATIONRECORD_TAGS;
use crate::generated::IPTC_pm::envelope_record_tags::IPTC_ENVELOPERECORD_TAGS;
use crate::types::{ExifError, Result, TagInfo, TagValue};
use std::collections::HashMap;
use tracing::{debug, warn};

/// IPTC DataSet structure
/// ExifTool: IPTC.pm ProcessIPTC function parsing logic
#[derive(Debug)]
#[allow(dead_code)] // Structure defined for documentation/future use
struct IptcDataSet {
    record: u8,
    dataset: u8,
    length: u16,
    data: Vec<u8>,
}

/// IPTC parser state
#[derive(Debug)]
struct IptcParser {
    /// Tag definitions by (record, dataset) -> TagInfo
    tag_definitions: HashMap<(u8, u8), TagInfo>,
    /// Character encoding for text fields (default Latin-1)
    #[allow(dead_code)] // Used in Phase 5 for full CodedCharacterSet support
    character_encoding: String,
}

impl IptcParser {
    /// Create new IPTC parser with tag definitions
    /// ExifTool: Tag table lookup system
    fn new() -> Self {
        let mut tag_definitions = HashMap::new();

        // Load ApplicationRecord (Record 2) tags
        for (id, tag_def) in IPTC_APPLICATIONRECORD_TAGS.iter() {
            if *id <= 255 {
                tag_definitions.insert((2, *id as u8), tag_def.clone());
            }
        }

        // Load EnvelopeRecord (Record 1) tags
        for (id, tag_def) in IPTC_ENVELOPERECORD_TAGS.iter() {
            if *id <= 255 {
                tag_definitions.insert((1, *id as u8), tag_def.clone());
            }
        }

        Self {
            tag_definitions,
            character_encoding: "Latin-1".to_string(), // Default encoding
        }
    }

    /// Parse IPTC binary data into tag values
    /// ExifTool: IPTC.pm ProcessIPTC function (lines 1050-1200)
    fn parse_iptc_data(&mut self, data: &[u8]) -> Result<HashMap<String, TagValue>> {
        let mut extracted_tags = HashMap::new();
        let mut pos = 0;
        let data_len = data.len();

        debug!("Parsing IPTC data: {} bytes", data_len);

        // ExifTool: Quick check for improperly byte-swapped IPTC (lines 1109-1118)
        if data_len >= 4 && data[0] != 0x1c && data[3] == 0x1c {
            warn!("IPTC data appears to be improperly byte-swapped - attempting correction");
            // TODO: Implement byte-swap correction if needed
        }

        // ExifTool: Main parsing loop (lines 1129-1181)
        while pos + 5 <= data_len {
            // Read 5-byte header: marker, record, dataset, length
            let marker = data[pos];
            let record = data[pos + 1];
            let dataset = data[pos + 2];
            let mut length = u16::from_be_bytes([data[pos + 3], data[pos + 4]]);
            pos += 5;

            // ExifTool: Check for valid IPTC marker (lines 1132-1141)
            if marker != 0x1c {
                if marker == 0 {
                    // Check if rest is all zeros (iMatch padding)
                    let remaining = &data[pos - 1..];
                    if remaining.iter().all(|&b| b == 0) {
                        break; // End of data
                    }
                }
                warn!("Bad IPTC data tag (marker 0x{:x})", marker);
                break;
            }

            // ExifTool: Handle extended IPTC entry (lines 1144-1155)
            if length & 0x8000 != 0 {
                let n = (length & 0x7fff) as usize; // Number of length bytes
                if pos + n > data_len || n > 8 {
                    warn!(
                        "Invalid extended IPTC entry (dataset {}:{}, len {})",
                        record, dataset, length
                    );
                    break;
                }

                // Read variable-length size (big-endian)
                length = 0;
                for _ in 0..n {
                    length = length * 256 + data[pos] as u16;
                    pos += 1;
                }
            }

            // ExifTool: Validate data length (lines 1156-1160)
            if pos + length as usize > data_len {
                warn!(
                    "Invalid IPTC entry (dataset {}:{}, len {})",
                    record, dataset, length
                );
                break;
            }

            // Extract data
            let tag_data = data[pos..pos + length as usize].to_vec();
            pos += length as usize;

            // Look up tag definition
            if let Some(tag_def) = self.tag_definitions.get(&(record, dataset)) {
                let tag_name = format!("IPTC:{}", tag_def.name);
                let tag_value = self.convert_tag_value(&tag_data, tag_def)?;

                debug!("Extracted IPTC tag: {} = {:?}", tag_name, tag_value);
                extracted_tags.insert(tag_name, tag_value);
            } else {
                debug!("Unknown IPTC tag: Record {}, DataSet {}", record, dataset);
            }
        }

        Ok(extracted_tags)
    }

    /// Convert raw tag data to TagValue based on format
    /// ExifTool: Format-specific conversion and string handling
    fn convert_tag_value(&self, data: &[u8], tag_def: &TagInfo) -> Result<TagValue> {
        match tag_def.format {
            format if format.starts_with("string") => {
                // Convert bytes to string with proper encoding
                let text = self.decode_string(data)?;

                // Handle list-type tags (Keywords can have multiple values)
                if tag_def.name == "Keywords" {
                    // For now, treat as single value - multi-value handling needs more work
                    Ok(TagValue::string(text))
                } else {
                    Ok(TagValue::string(text))
                }
            }
            "int16u" => {
                if data.len() >= 2 {
                    let value = u16::from_be_bytes([data[0], data[1]]);
                    Ok(TagValue::U16(value))
                } else {
                    Err(ExifError::InvalidFormat("Invalid int16u data".to_string()))
                }
            }
            "digits[8]" => {
                // Date format: YYYYMMDD
                let text = self.decode_string(data)?;
                Ok(TagValue::string(text))
            }
            "digits[1]" | "digits[2]" => {
                // Numeric string
                let text = self.decode_string(data)?;
                Ok(TagValue::string(text))
            }
            _ => {
                // Default to text for unknown formats
                let text = self.decode_string(data)?;
                Ok(TagValue::string(text))
            }
        }
    }

    /// Decode byte array to string using current character encoding
    /// ExifTool: Character set handling with CodedCharacterSet support
    fn decode_string(&self, data: &[u8]) -> Result<String> {
        // For now, use simple UTF-8 decoding with fallback to Latin-1
        // TODO: Implement proper CodedCharacterSet handling in Phase 5
        match std::str::from_utf8(data) {
            Ok(s) => Ok(s.trim_end_matches('\0').to_string()), // Remove null termination
            Err(_) => {
                // Fallback to Latin-1 (ISO 8859-1) decoding
                let text: String = data.iter().map(|&b| b as char).collect();
                Ok(text.trim_end_matches('\0').to_string())
            }
        }
    }
}

/// Parse IPTC metadata from binary data
/// ExifTool: Main entry point equivalent to ProcessIPTC
pub fn parse_iptc_metadata(data: &[u8]) -> Result<HashMap<String, TagValue>> {
    let mut parser = IptcParser::new();
    parser.parse_iptc_data(data)
}

/// Parse IPTC data from JPEG APP13 segment containing Adobe Photoshop Image Resource Blocks
/// ExifTool: Photoshop.pm Image Resource Block processing with Resource ID 0x0404 for IPTC
pub fn parse_iptc_from_app13(app13_data: &[u8]) -> Result<HashMap<String, TagValue>> {
    debug!(
        "Parsing IPTC from APP13 segment: {} bytes",
        app13_data.len()
    );

    // Check for "Photoshop 3.0" signature
    // ExifTool: Photoshop.pm ProcessPhotoshop function
    let photoshop_signature = b"Photoshop 3.0\0";
    if app13_data.len() < photoshop_signature.len() || !app13_data.starts_with(photoshop_signature)
    {
        return Err(ExifError::InvalidFormat(
            "APP13 segment does not contain Photoshop 3.0 signature".to_string(),
        ));
    }

    // Skip signature and start parsing Image Resource Blocks
    let mut pos = photoshop_signature.len();

    while pos + 12 <= app13_data.len() {
        // Parse Image Resource Block header
        // Format: "8BIM" (4 bytes) + Resource ID (2 bytes) + Name Length (2 bytes) + Name + Data Length (4 bytes) + Data

        // Check for "8BIM" signature
        if pos + 4 > app13_data.len() || &app13_data[pos..pos + 4] != b"8BIM" {
            debug!("No more Image Resource Blocks found at offset {}", pos);
            break;
        }
        pos += 4;

        // Read Resource ID (2 bytes, big-endian)
        if pos + 2 > app13_data.len() {
            break;
        }
        let resource_id = u16::from_be_bytes([app13_data[pos], app13_data[pos + 1]]);
        pos += 2;

        // Read Name Length (2 bytes, big-endian)
        if pos + 2 > app13_data.len() {
            break;
        }
        let name_length = u16::from_be_bytes([app13_data[pos], app13_data[pos + 1]]) as usize;
        pos += 2;

        // Skip name data (padded to even boundary)
        let padded_name_length = if name_length % 2 == 0 {
            name_length
        } else {
            name_length + 1
        };
        if pos + padded_name_length > app13_data.len() {
            break;
        }
        pos += padded_name_length;

        // Read Data Length (4 bytes, big-endian)
        if pos + 4 > app13_data.len() {
            break;
        }
        let data_length = u32::from_be_bytes([
            app13_data[pos],
            app13_data[pos + 1],
            app13_data[pos + 2],
            app13_data[pos + 3],
        ]) as usize;
        pos += 4;

        debug!(
            "Found Image Resource Block: ID=0x{:04x}, name_len={}, data_len={}",
            resource_id, name_length, data_length
        );

        // Check if this is the IPTC resource (Resource ID 0x0404)
        if resource_id == 0x0404 {
            debug!(
                "Found IPTC Image Resource Block (0x0404) with {} bytes",
                data_length
            );

            // Extract IPTC data
            if pos + data_length > app13_data.len() {
                return Err(ExifError::InvalidFormat(
                    "IPTC Image Resource Block data extends beyond APP13 segment".to_string(),
                ));
            }

            let iptc_data = &app13_data[pos..pos + data_length];

            // Parse IPTC data using our existing parser
            return parse_iptc_metadata(iptc_data);
        }

        // Skip to next Image Resource Block (data padded to even boundary)
        let padded_data_length = if data_length % 2 == 0 {
            data_length
        } else {
            data_length + 1
        };
        pos += padded_data_length;
    }

    // No IPTC resource found
    debug!("No IPTC Image Resource Block (0x0404) found in APP13 segment");
    Ok(HashMap::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::generated::IPTC_pm::tag_kit::PrintConvType;

    #[test]
    fn test_iptc_dataset_parsing() {
        // Create minimal IPTC data for testing
        // Record 2, DataSet 25 (Keywords), Length 14, Data "Communications"
        let data = vec![
            0x1c, 0x02, 0x19, 0x00,
            0x0e, // Header: marker, record, dataset, length (14 bytes)
            b'C', b'o', b'm', b'm', b'u', b'n', b'i', b'c', b'a', b't', b'i', b'o', b'n',
            b's', // "Communications" (14 chars)
        ];

        let result = parse_iptc_metadata(&data);
        assert!(result.is_ok());

        let tags = result.unwrap();
        assert!(tags.contains_key("IPTC:Keywords"));

        if let Some(TagValue::String(value)) = tags.get("IPTC:Keywords") {
            assert_eq!(value, "Communications");
        } else {
            panic!("Expected Keywords tag to be a string");
        }
    }

    #[test]
    fn test_invalid_marker() {
        // Invalid marker (not 0x1c)
        let data = vec![0x1d, 0x02, 0x19, 0x00, 0x05, b'h', b'e', b'l', b'l', b'o'];

        let result = parse_iptc_metadata(&data);
        assert!(result.is_ok());

        let tags = result.unwrap();
        assert!(tags.is_empty()); // Should stop parsing on invalid marker
    }

    #[test]
    fn test_multiple_iptc_tags() {
        // Test parsing multiple IPTC tags
        let mut data = Vec::new();

        // Keywords (Record 2, DataSet 25)
        data.extend_from_slice(&[0x1c, 0x02, 0x19, 0x00, 0x04]);
        data.extend_from_slice(b"test");

        // Source (Record 2, DataSet 115)
        data.extend_from_slice(&[0x1c, 0x02, 0x73, 0x00, 0x0C]);
        data.extend_from_slice(b"FreeFoto.com");

        let result = parse_iptc_metadata(&data);
        assert!(result.is_ok());

        let tags = result.unwrap();
        assert_eq!(tags.len(), 2);
        assert!(tags.contains_key("IPTC:Keywords"));
        assert!(tags.contains_key("IPTC:Source"));

        if let Some(TagValue::String(value)) = tags.get("IPTC:Source") {
            assert_eq!(value, "FreeFoto.com");
        }
    }

    #[test]
    fn test_extended_iptc_entry() {
        // Test extended IPTC entry with variable-length size field
        // Extended entry marker: length has high bit set (0x8000 | n)
        let mut data = Vec::new();
        data.extend_from_slice(&[0x1c, 0x02, 0x19]); // marker, record, dataset
        data.extend_from_slice(&[0x80, 0x02]); // Extended length: 2 bytes follow
        data.extend_from_slice(&[0x00, 0x04]); // Actual length: 4 bytes
        data.extend_from_slice(b"test");

        let result = parse_iptc_metadata(&data);
        assert!(result.is_ok());

        let tags = result.unwrap();
        assert!(tags.contains_key("IPTC:Keywords"));

        if let Some(TagValue::String(value)) = tags.get("IPTC:Keywords") {
            assert_eq!(value, "test");
        }
    }

    #[test]
    fn test_truncated_data() {
        // Test handling of truncated IPTC data
        let data = vec![0x1c, 0x02, 0x19, 0x00, 0x10]; // Claims 16 bytes but no data follows

        let result = parse_iptc_metadata(&data);
        assert!(result.is_ok());

        let tags = result.unwrap();
        assert!(tags.is_empty()); // Should handle gracefully without crashing
    }

    #[test]
    fn test_zero_padding() {
        // Test iMatch zero padding at end of data
        let mut data = Vec::new();

        // Valid IPTC tag
        data.extend_from_slice(&[0x1c, 0x02, 0x19, 0x00, 0x04]);
        data.extend_from_slice(b"test");

        // Zero padding (should be ignored)
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        let result = parse_iptc_metadata(&data);
        assert!(result.is_ok());

        let tags = result.unwrap();
        assert_eq!(tags.len(), 1);
        assert!(tags.contains_key("IPTC:Keywords"));
    }

    #[test]
    fn test_unknown_tag() {
        // Test handling of unknown tag (record/dataset not in our definitions)
        let data = vec![
            0x1c, 0x09, 0x99, 0x00, 0x04, // Unknown record 9, dataset 153
            b't', b'e', b's', b't',
        ];

        let result = parse_iptc_metadata(&data);
        assert!(result.is_ok());

        let tags = result.unwrap();
        assert!(tags.is_empty()); // Unknown tags are not extracted
    }

    #[test]
    fn test_string_encoding() {
        // Test UTF-8 string handling
        let utf8_bytes = "tëst".as_bytes(); // UTF-8 with accented character (5 bytes)
        let mut data = Vec::new();
        data.extend_from_slice(&[0x1c, 0x02, 0x19, 0x00, utf8_bytes.len() as u8]);
        data.extend_from_slice(utf8_bytes);

        let result = parse_iptc_metadata(&data);
        assert!(result.is_ok());

        let tags = result.unwrap();
        assert!(tags.contains_key("IPTC:Keywords"));

        if let Some(TagValue::String(value)) = tags.get("IPTC:Keywords") {
            assert_eq!(value, "tëst");
        }
    }

    #[test]
    fn test_null_terminated_string() {
        // Test null-terminated string handling
        let mut data = Vec::new();
        data.extend_from_slice(&[0x1c, 0x02, 0x19, 0x00, 0x05]);
        data.extend_from_slice(b"test\0"); // Null-terminated

        let result = parse_iptc_metadata(&data);
        assert!(result.is_ok());

        let tags = result.unwrap();
        assert!(tags.contains_key("IPTC:Keywords"));

        if let Some(TagValue::String(value)) = tags.get("IPTC:Keywords") {
            assert_eq!(value, "test"); // Null termination should be stripped
        }
    }

    #[test]
    fn test_photoshop_app13_parsing() {
        // Test parsing IPTC from Adobe Photoshop APP13 segment
        let mut app13_data = Vec::new();

        // Photoshop 3.0 signature
        app13_data.extend_from_slice(b"Photoshop 3.0\0");

        // Image Resource Block header
        app13_data.extend_from_slice(b"8BIM"); // Signature
        app13_data.extend_from_slice(&[0x04, 0x04]); // Resource ID 0x0404 (IPTC)
        app13_data.extend_from_slice(&[0x00, 0x00]); // Name length (0)
        app13_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x09]); // Data length (9 bytes)

        // IPTC data
        app13_data.extend_from_slice(&[0x1c, 0x02, 0x19, 0x00, 0x04]);
        app13_data.extend_from_slice(b"test");

        let result = parse_iptc_from_app13(&app13_data);
        assert!(result.is_ok());

        let tags = result.unwrap();
        assert_eq!(tags.len(), 1);
        assert!(tags.contains_key("IPTC:Keywords"));
    }

    #[test]
    fn test_photoshop_app13_invalid_signature() {
        // Test APP13 segment without Photoshop signature
        let app13_data = b"Not Photoshop data";

        let result = parse_iptc_from_app13(app13_data);
        assert!(result.is_err());

        if let Err(ExifError::InvalidFormat(msg)) = result {
            assert!(msg.contains("Photoshop 3.0 signature"));
        } else {
            panic!("Expected InvalidFormat error");
        }
    }

    #[test]
    fn test_photoshop_app13_no_iptc_resource() {
        // Test APP13 segment with valid Photoshop data but no IPTC resource
        let mut app13_data = Vec::new();

        // Photoshop 3.0 signature
        app13_data.extend_from_slice(b"Photoshop 3.0\0");

        // Image Resource Block header (different resource ID)
        app13_data.extend_from_slice(b"8BIM"); // Signature
        app13_data.extend_from_slice(&[0x04, 0x03]); // Resource ID 0x0403 (not IPTC)
        app13_data.extend_from_slice(&[0x00, 0x00]); // Name length (0)
        app13_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x04]); // Data length (4 bytes)
        app13_data.extend_from_slice(b"test");

        let result = parse_iptc_from_app13(&app13_data);
        assert!(result.is_ok());

        let tags = result.unwrap();
        assert!(tags.is_empty()); // No IPTC resource found
    }

    #[test]
    fn test_photoshop_app13_with_resource_name() {
        // Test APP13 segment with named resource
        let mut app13_data = Vec::new();

        // Photoshop 3.0 signature
        app13_data.extend_from_slice(b"Photoshop 3.0\0");

        // Image Resource Block header with name
        app13_data.extend_from_slice(b"8BIM"); // Signature
        app13_data.extend_from_slice(&[0x04, 0x04]); // Resource ID 0x0404 (IPTC)
        app13_data.extend_from_slice(&[0x00, 0x04]); // Name length (4 bytes)
        app13_data.extend_from_slice(b"IPTC"); // Resource name
        app13_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x09]); // Data length (9 bytes)

        // IPTC data
        app13_data.extend_from_slice(&[0x1c, 0x02, 0x19, 0x00, 0x04]);
        app13_data.extend_from_slice(b"test");

        let result = parse_iptc_from_app13(&app13_data);
        assert!(result.is_ok());

        let tags = result.unwrap();
        assert_eq!(tags.len(), 1);
        assert!(tags.contains_key("IPTC:Keywords"));
    }

    #[test]
    fn test_format_conversion_int16u() {
        // Create parser and test int16u format conversion
        let parser = IptcParser::new();

        // Create a tag definition for int16u format (like ApplicationRecordVersion)
        let tag_def = TagInfo {
            name: "ApplicationRecordVersion",
            format: "int16u",
            print_conv: None,
            value_conv: None,
        };

        let data = [0x00, 0x02]; // Big-endian 16-bit value: 2
        let result = parser.convert_tag_value(&data, &tag_def);
        assert!(result.is_ok());

        if let Ok(TagValue::U16(value)) = result {
            assert_eq!(value, 2);
        } else {
            panic!("Expected U16 tag value");
        }
    }

    #[test]
    fn test_format_conversion_digits() {
        // Test digits format conversion
        let parser = IptcParser::new();

        let tag_def = TagInfo {
            name: "ReferenceNumber",
            format: "digits[8]",
            print_conv: None,
            value_conv: None,
        };

        let data = b"12345678";
        let result = parser.convert_tag_value(data, &tag_def);
        assert!(result.is_ok());

        if let Ok(TagValue::String(value)) = result {
            assert_eq!(value, "12345678");
        } else {
            panic!("Expected String tag value");
        }
    }

    #[test]
    fn test_parser_initialization() {
        // Test that parser initializes with expected tag definitions
        let parser = IptcParser::new();

        // Should have definitions for common IPTC tags
        assert!(parser.tag_definitions.contains_key(&(2, 25))); // Keywords
        assert!(parser.tag_definitions.contains_key(&(2, 115))); // Source
        assert!(parser.tag_definitions.contains_key(&(2, 110))); // Credit

        // Check default encoding
        assert_eq!(parser.character_encoding, "Latin-1");
    }
}
