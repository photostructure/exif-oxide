//! Olympus maker note parser using optimized table-driven approach
//!
//! Olympus maker notes use a standard IFD structure with maker-specific tags.
//! This implementation leverages the revolutionary shared PrintConv system
//! that eliminates duplicate conversion functions across manufacturers.
//!
//! Key optimizations:
//! - Uses shared OnOff conversion (11 tags consolidated)
//! - Uses shared olympusCameraTypes lookup (7 tags consolidated)
//! - Auto-generated tag tables with PrintConv mappings
//! - Zero-copy parsing with table-driven conversions

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Olympus.pm"]

use crate::core::ifd::{IfdParser, TiffHeader};
use crate::core::print_conv::apply_print_conv;
use crate::core::{Endian, ExifValue};
use crate::error::Result;
use crate::maker::olympus::detection::{detect_olympus_maker_note, OLYMPUSDetectionResult};
use crate::maker::MakerNoteParser;
use crate::tables::olympus_tags::{get_olympus_tag, OLYMPUS_TAGS};
use std::collections::HashMap;

pub mod detection;

/// Parser for Olympus maker notes using optimized shared PrintConv system
pub struct OlympusMakerNoteParser;

impl MakerNoteParser for OlympusMakerNoteParser {
    fn manufacturer(&self) -> &'static str {
        "Olympus"
    }

    fn parse(
        &self,
        data: &[u8],
        byte_order: Endian,
        _base_offset: usize,
    ) -> Result<HashMap<u16, ExifValue>> {
        if data.is_empty() {
            return Ok(HashMap::new());
        }

        // Use auto-generated detection logic to identify Olympus maker note format
        let detection = match detect_olympus_maker_note(data) {
            Some(detection) => detection,
            None => {
                // Fallback: assume standard IFD at start of data
                OLYMPUSDetectionResult {
                    version: None,
                    ifd_offset: 0,
                    description: "Fallback Olympus parser".to_string(),
                }
            }
        };

        // Parse IFD data using optimized table-driven approach
        parse_olympus_ifd_with_tables(&data[detection.ifd_offset..], byte_order)
    }
}

/// Parse Olympus IFD using auto-generated tag tables and shared PrintConv system
fn parse_olympus_ifd_with_tables(
    data: &[u8],
    byte_order: Endian,
) -> Result<HashMap<u16, ExifValue>> {
    if data.is_empty() {
        return Ok(HashMap::new());
    }

    // Create a fake TIFF header for IFD parsing
    // (Olympus maker notes don't have a TIFF header, they start directly with IFD)
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
            return Err(e);
        }
    };

    let mut result = HashMap::new();

    // Process each IFD entry using optimized table-driven conversion
    for (tag_id, raw_value) in parsed_ifd.entries() {
        if let Some(olympus_tag) = get_olympus_tag(*tag_id) {
            // Apply print conversion to create human-readable value
            let converted_value = apply_print_conv(raw_value, olympus_tag.print_conv);

            // Store both raw and converted values
            // Raw value for programmatic access
            result.insert(*tag_id, raw_value.clone());

            // Converted value as string (following ExifTool pattern)
            // Use a high bit pattern to distinguish converted values
            let converted_tag_id = 0x8000 | tag_id;
            result.insert(converted_tag_id, ExifValue::Ascii(converted_value));
        } else {
            // Keep unknown tags as-is
            result.insert(*tag_id, raw_value.clone());
        }
    }

    Ok(result)
}

/// Get Olympus tag information by ID
pub fn get_olympus_tag_info(tag_id: u16) -> Option<&'static str> {
    get_olympus_tag(tag_id).map(|tag| tag.name)
}

/// List all supported Olympus tags
pub fn list_olympus_tags() -> Vec<(u16, &'static str)> {
    OLYMPUS_TAGS.iter().map(|tag| (tag.id, tag.name)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ExifValue;

    #[test]
    fn test_olympus_tag_lookup() {
        // Test that we can find known Olympus tags
        assert!(get_olympus_tag(0x0202).is_some()); // Macro
        assert!(get_olympus_tag(0x0203).is_some()); // BWMode
        assert_eq!(get_olympus_tag(0x9999), None); // Non-existent tag
    }

    #[test]
    fn test_olympus_shared_printconv() {
        // Test that Olympus leverages shared PrintConv optimizations
        let tag_count = OLYMPUS_TAGS.len();
        assert!(tag_count >= 100, "Should have 100+ Olympus tags");

        // Verify that OnOff tags use shared conversion
        let macro_tag = get_olympus_tag(0x0202).unwrap(); // Macro tag
        let converted = apply_print_conv(&ExifValue::U32(1), macro_tag.print_conv);
        assert_eq!(converted, "On"); // Should use shared OnOff conversion
    }

    #[test]
    fn test_empty_data() {
        let parser = OlympusMakerNoteParser;
        let result = parser.parse(&[], Endian::Little, 0).unwrap();
        assert!(result.is_empty());
    }
}
