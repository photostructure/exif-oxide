//! Hasselblad maker note parser
//!
//! Hasselblad maker notes use a standard IFD structure similar to standard EXIF.
//! Detection is based solely on Make field being "Hasselblad".
//! Uses simple IFD parsing with no special headers or complex structures.
//!
//! Based on ExifTool's MakerNotes.pm implementation:
//! - Condition: $$self{Make} eq "Hasselblad"
//! - TagTable: Image::ExifTool::Unknown::Main
//! - ByteOrder: Unknown (auto-detected)
//! - Start: $valuePtr (no offset)
//! - Base: 0 (not self-contained)

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/MakerNotes.pm"]

use crate::core::ifd::{IfdParser, TiffHeader};
use crate::core::print_conv::apply_print_conv;
use crate::core::{Endian, ExifValue};
use crate::error::Result;
use crate::maker::MakerNoteParser;
use crate::tables::hasselblad_tags::{get_hasselblad_tag, HASSELBLAD_TAGS};
use std::collections::HashMap;

/// Parser for Hasselblad maker notes
///
/// Follows ExifTool's simple implementation for Hasselblad maker notes
pub struct HasselbladMakerNoteParser;

impl MakerNoteParser for HasselbladMakerNoteParser {
    fn parse(
        &self,
        data: &[u8],
        byte_order: Endian,
        _base_offset: usize,
    ) -> Result<HashMap<u16, ExifValue>> {
        if data.is_empty() {
            return Ok(HashMap::new());
        }

        // Parse using table-driven approach like other manufacturers
        parse_hasselblad_ifd_with_tables(data, byte_order)
    }

    fn manufacturer(&self) -> &'static str {
        "Hasselblad"
    }
}

/// Parse Hasselblad IFD data using tag tables
///
/// Matches ExifTool's processing of Hasselblad maker notes:
/// - Direct IFD parsing (no special header)
/// - Uses main EXIF byte order
/// - Base offset 0 to avoid warnings
fn parse_hasselblad_ifd_with_tables(
    data: &[u8],
    byte_order: Endian,
) -> Result<HashMap<u16, ExifValue>> {
    // Create a fake TIFF header for IFD parsing
    // Hasselblad maker notes start directly with IFD data
    let mut tiff_data = Vec::with_capacity(8 + data.len());

    // Add TIFF header matching the detected byte order
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

    let parsed = match IfdParser::parse_ifd(&tiff_data, &header, 8) {
        Ok(parsed) => parsed,
        Err(e) => {
            // Log the error but return empty results
            // Many maker notes have quirks that might cause parsing errors
            eprintln!("Warning: Hasselblad maker note parsing failed: {}", e);
            return Ok(HashMap::new());
        }
    };

    let mut result = HashMap::new();

    // Process entries using tag table
    for (tag_id, raw_value) in parsed.entries() {
        // Store raw value
        result.insert(*tag_id, raw_value.clone());

        // Apply PrintConv if tag is known
        if let Some(tag_def) = get_hasselblad_tag(*tag_id) {
            let converted = apply_print_conv(raw_value, tag_def.print_conv);
            // Store converted value with high bit set (following other parsers)
            let converted_tag_id = 0x8000 | tag_id;
            result.insert(converted_tag_id, ExifValue::Ascii(converted));
        }
    }

    Ok(result)
}

/// Known Hasselblad tag IDs from ExifTool comments
/// These match the tag definitions in hasselblad_tags.rs
pub mod tags {
    /// Sensor code (reference IB)
    pub const SENSOR_CODE: u16 = 0x0011;

    /// Camera model ID (uncertain)
    pub const CAMERA_MODEL_ID: u16 = 0x0012;

    /// Camera model name
    pub const CAMERA_MODEL_NAME: u16 = 0x0015;

    /// Coating code (reference IB)
    pub const COATING_CODE: u16 = 0x0016;
}

/// Check if a tag ID is a known Hasselblad tag
pub fn is_known_hasselblad_tag(tag_id: u16) -> bool {
    get_hasselblad_tag(tag_id).is_some()
}

/// Get count of known Hasselblad tags
pub fn hasselblad_tag_count() -> usize {
    HASSELBLAD_TAGS.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hasselblad_parser_creation() {
        let parser = HasselbladMakerNoteParser;
        assert_eq!(parser.manufacturer(), "Hasselblad");
    }

    #[test]
    fn test_empty_maker_note() {
        let parser = HasselbladMakerNoteParser;
        let result = parser.parse(&[], Endian::Little, 0).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_hasselblad_tag_constants() {
        assert_eq!(tags::SENSOR_CODE, 0x0011);
        assert_eq!(tags::CAMERA_MODEL_ID, 0x0012);
        assert_eq!(tags::CAMERA_MODEL_NAME, 0x0015);
        assert_eq!(tags::COATING_CODE, 0x0016);
    }

    #[test]
    fn test_known_tag_detection() {
        assert!(is_known_hasselblad_tag(0x0011));
        assert!(is_known_hasselblad_tag(0x0012));
        assert!(is_known_hasselblad_tag(0x0015));
        assert!(is_known_hasselblad_tag(0x0016));
        assert!(!is_known_hasselblad_tag(0x9999));
    }

    #[test]
    fn test_tag_count() {
        assert_eq!(hasselblad_tag_count(), 4);
    }

    #[test]
    fn test_table_consistency() {
        // Verify our constants match the tag table
        use crate::tables::hasselblad_tags::get_hasselblad_tag;

        assert!(get_hasselblad_tag(tags::SENSOR_CODE).is_some());
        assert!(get_hasselblad_tag(tags::CAMERA_MODEL_ID).is_some());
        assert!(get_hasselblad_tag(tags::CAMERA_MODEL_NAME).is_some());
        assert!(get_hasselblad_tag(tags::COATING_CODE).is_some());

        assert_eq!(
            get_hasselblad_tag(tags::SENSOR_CODE).unwrap().name,
            "SensorCode"
        );
        assert_eq!(
            get_hasselblad_tag(tags::CAMERA_MODEL_ID).unwrap().name,
            "CameraModelID"
        );
        assert_eq!(
            get_hasselblad_tag(tags::CAMERA_MODEL_NAME).unwrap().name,
            "CameraModelName"
        );
        assert_eq!(
            get_hasselblad_tag(tags::COATING_CODE).unwrap().name,
            "CoatingCode"
        );
    }
}
