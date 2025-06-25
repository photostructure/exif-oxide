//! DJI maker note parser using table-driven approach
//!
//! DJI drones use a different approach compared to traditional cameras:
//! - Main metadata is in XMP tags (drone-dji namespace)
//! - Some thermal data in APP4 segments
//! - Protobuf format for timed metadata
//! - Limited traditional maker note usage
//!
//! This implementation focuses on the traditional maker notes when present,
//! which contain basic drone flight information.

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/DJI.pm"]

use crate::core::ifd::{IfdParser, TiffHeader};
use crate::core::print_conv::apply_print_conv;
use crate::core::{Endian, ExifValue};
use crate::error::Result;
use crate::maker::MakerNoteParser;
use crate::tables::dji_tags::get_dji_tag;
use std::collections::HashMap;

/// Parser for DJI maker notes
pub struct DJIMakerNoteParser;

impl MakerNoteParser for DJIMakerNoteParser {
    fn parse(
        &self,
        data: &[u8],
        byte_order: Endian,
        _base_offset: usize,
    ) -> Result<HashMap<u16, ExifValue>> {
        if data.is_empty() {
            return Ok(HashMap::new());
        }

        // DJI maker notes don't have a special signature like other manufacturers
        // They use standard IFD format when present
        // EXIFTOOL-SOURCE: lib/Image/ExifTool/DJI.pm - DJI::Main table uses standard EXIF format

        // Parse using table-driven approach starting from beginning of data
        parse_dji_ifd_with_tables(data, byte_order)
    }

    fn manufacturer(&self) -> &'static str {
        "DJI"
    }
}

/// Parse DJI IFD using generated tag tables and print conversion
fn parse_dji_ifd_with_tables(data: &[u8], byte_order: Endian) -> Result<HashMap<u16, ExifValue>> {
    // Create a fake TIFF header for IFD parsing
    let mut tiff_data = Vec::with_capacity(8 + data.len());

    // Add TIFF header
    match byte_order {
        Endian::Little => {
            tiff_data.extend_from_slice(b"II");
            tiff_data.extend_from_slice(&[0x2a, 0x00]); // 42 in little-endian
            tiff_data.extend_from_slice(&[0x08, 0x00, 0x00, 0x00]); // offset 8
        }
        Endian::Big => {
            tiff_data.extend_from_slice(b"MM");
            tiff_data.extend_from_slice(&[0x00, 0x2a]); // 42 in big-endian
            tiff_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x08]); // offset 8
        }
    }

    // Add the actual IFD data
    tiff_data.extend_from_slice(data);

    // Parse the IFD
    let header = TiffHeader {
        byte_order,
        ifd0_offset: 8,
    };

    let parsed_ifd = match IfdParser::parse_ifd(&tiff_data, &header, 8) {
        Ok(parsed) => parsed,
        Err(e) => {
            eprintln!("Warning: DJI IFD parsing failed: {}", e);
            return Ok(HashMap::new());
        }
    };

    // Convert raw IFD entries to DJI tags with print conversion
    let mut result = HashMap::new();

    for (tag_id, raw_value) in parsed_ifd.entries() {
        if let Some(dji_tag) = get_dji_tag(*tag_id) {
            // Apply print conversion to create human-readable value
            let converted_value = apply_print_conv(raw_value, dji_tag.print_conv);

            // Store both raw and converted values
            // Raw value for programmatic access
            result.insert(*tag_id, raw_value.clone());

            // Converted value for human-readable display
            // Use high bit (0x8000) to distinguish converted tags
            let converted_tag_id = 0x8000 | *tag_id;
            result.insert(converted_tag_id, ExifValue::Ascii(converted_value));
        } else {
            // Store unknown tags as raw values
            result.insert(*tag_id, raw_value.clone());
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Endian;

    #[test]
    fn test_dji_parser_creation() {
        let parser = DJIMakerNoteParser;
        assert_eq!(parser.manufacturer(), "DJI");
    }

    #[test]
    fn test_dji_empty_data() {
        let parser = DJIMakerNoteParser;
        let result = parser.parse(&[], Endian::Little, 0);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_dji_basic_ifd_parsing() {
        let parser = DJIMakerNoteParser;

        // Create a minimal IFD with one entry (DJI Make tag 0x01)
        let mut data = Vec::new();
        data.extend_from_slice(&[0x01, 0x00]); // 1 entry
        data.extend_from_slice(&[0x01, 0x00]); // tag 0x01 (Make)
        data.extend_from_slice(&[0x02, 0x00]); // type ASCII
        data.extend_from_slice(&[0x04, 0x00, 0x00, 0x00]); // count 4
        data.extend_from_slice(b"DJI\0"); // value "DJI"
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // next IFD offset (none)

        let result = parser.parse(&data, Endian::Little, 0);
        assert!(result.is_ok());

        let tags = result.unwrap();
        assert!(!tags.is_empty());

        // Should have raw tag 0x01
        assert!(tags.contains_key(&0x01));

        // Should have converted tag 0x8001 (0x8000 | 0x01)
        assert!(tags.contains_key(&0x8001));
    }
}
