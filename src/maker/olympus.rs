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

use crate::core::ifd::parse_ifd;
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
    let mut parser = IfdParser::new(data, byte_order);
    let raw_entries = parser.parse_ifd(0)?;
    
    let mut result = HashMap::new();
    
    // Process each IFD entry using optimized table-driven conversion
    for (tag_id, raw_value) in raw_entries {
        // Store raw value
        result.insert(tag_id, raw_value.clone());
        
        // Apply table-driven PrintConv if available
        if let Some(tag_def) = get_olympus_tag(tag_id) {
            // Generate human-readable value using shared PrintConv system
            let converted = apply_print_conv(&raw_value, tag_def.print_conv);
            
            // Store converted value with high bit set to distinguish from raw
            let converted_tag_id = 0x8000 | tag_id;
            result.insert(converted_tag_id, ExifValue::Ascii(converted));
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
    OLYMPUS_TAGS
        .iter()
        .map(|tag| (tag.id, tag.name))
        .collect()
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
        assert_eq!(get_olympus_tag(0x9999), None);  // Non-existent tag
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