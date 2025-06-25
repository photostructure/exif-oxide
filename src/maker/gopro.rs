//! GoPro maker note parser using GPMF integration
//!
//! GoPro cameras use GPMF (GoPro Metadata Format) instead of traditional
//! EXIF maker notes. GPMF uses 4-byte tag IDs and nested containers with
//! a completely different structure than standard IFD-based maker notes.
//!
//! This parser integrates with the existing GPMF parser to provide maker note
//! compatibility while leveraging the comprehensive GPMF implementation.

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/GoPro.pm"]

use crate::core::{Endian, ExifValue};
use crate::error::Result;
use crate::gpmf::parse_gpmf;
use crate::maker::MakerNoteParser;
use std::collections::HashMap;

/// Parser for GoPro maker notes (GPMF format)
pub struct GoProMakerNoteParser;

impl MakerNoteParser for GoProMakerNoteParser {
    fn parse(
        &self,
        data: &[u8],
        _byte_order: Endian, // GPMF uses big-endian format, so we ignore the input endian
        _base_offset: usize,
    ) -> Result<HashMap<u16, ExifValue>> {
        if data.is_empty() {
            return Ok(HashMap::new());
        }

        // Parse GPMF data using the existing comprehensive parser
        let gpmf_results = parse_gpmf(data)?;

        // Convert GPMF string-based results to u16-based HashMap for maker note compatibility
        let mut result = HashMap::new();
        let mut tag_counter: u16 = 0x1000; // Start at 0x1000 to avoid conflicts with standard EXIF tags

        for (tag_id, value) in gpmf_results {
            // Map each GPMF tag to a unique u16 ID
            // This allows the maker note system to access GPMF tags
            result.insert(tag_counter, value);

            // Store the original GPMF tag ID as a string value with a special prefix
            // This allows lookup of the original tag name
            let tag_name_key = 0x8000 | tag_counter; // Use high bit to indicate tag name
            result.insert(tag_name_key, ExifValue::Ascii(format!("GPMF:{}", tag_id)));

            tag_counter += 1;
        }

        Ok(result)
    }

    fn manufacturer(&self) -> &'static str {
        "GoPro"
    }
}

/// Detect if data contains GPMF format
///
/// GPMF format detection (based on ExifTool's ProcessGoPro):
/// - Looks for GPMF 4-byte tag structure
/// - First 4 bytes should be a valid GPMF tag (like "DEVC")
/// - Followed by format code, size, and count
pub fn detect_gpmf_format(data: &[u8]) -> bool {
    if data.len() < 8 {
        return false;
    }

    // Check if first 4 bytes form a valid GPMF tag
    let tag_bytes = &data[0..4];

    // Check for common GPMF container tags
    let tag_str = std::str::from_utf8(tag_bytes).unwrap_or("");

    // DEVC is the main container tag in GPMF format
    if tag_str == "DEVC" {
        return true;
    }

    // Check for other common GPMF tags that might appear at the start
    let common_tags = ["STRM", "GPS5", "GYRO", "ACCL", "TMPC"];
    if common_tags.contains(&tag_str) {
        return true;
    }

    // Additional validation: check if the structure looks like GPMF
    // Format: 4-byte tag + 1-byte format + 1-byte size + 2-byte count
    if data.len() >= 8 {
        let format_code = data[4];
        let size_per_sample = data[5];
        let count = u16::from_be_bytes([data[6], data[7]]);

        // Basic validation of GPMF structure
        if format_code <= 0x6a && size_per_sample > 0 && count > 0 {
            return true;
        }
    }

    false
}

/// Convert GPMF tag to maker note tag ID
///
/// This provides a mapping between 4-byte GPMF tags and 16-bit maker note tag IDs
/// for compatibility with the existing maker note system.
pub fn gpmf_tag_to_maker_note_id(gpmf_tag: &str) -> Option<u16> {
    // Map common GPMF tags to specific maker note IDs
    match gpmf_tag {
        "DEVC" => Some(0x1001), // Device Container
        "DVNM" => Some(0x1002), // Device Name
        "FMWR" => Some(0x1003), // Firmware Version
        "CASN" => Some(0x1004), // Camera Serial Number
        "GPS5" => Some(0x1005), // GPS Info
        "GYRO" => Some(0x1006), // Gyroscope
        "ACCL" => Some(0x1007), // Accelerometer
        "TMPC" => Some(0x1008), // Temperature
        "SHUT" => Some(0x1009), // Exposure Times
        "ISOE" => Some(0x100A), // ISO Speeds
        _ => None,              // Unknown tags get dynamic assignment
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gopro_parser_creation() {
        let parser = GoProMakerNoteParser;
        assert_eq!(parser.manufacturer(), "GoPro");
    }

    #[test]
    fn test_detect_gpmf_format() {
        // Test with DEVC container tag
        let gpmf_data = b"DEVC\x00\x04\x00\x01";
        assert!(detect_gpmf_format(gpmf_data));

        // Test with GPS5 tag
        let gps_data = b"GPS5\x6c\x04\x00\x05";
        assert!(detect_gpmf_format(gps_data));

        // Test with non-GPMF data
        let invalid_data = b"ABCD\xff\xff\xff\xff";
        assert!(!detect_gpmf_format(invalid_data));

        // Test with empty data
        assert!(!detect_gpmf_format(b""));
    }

    #[test]
    fn test_gpmf_tag_mapping() {
        assert_eq!(gpmf_tag_to_maker_note_id("DEVC"), Some(0x1001));
        assert_eq!(gpmf_tag_to_maker_note_id("GPS5"), Some(0x1005));
        assert_eq!(gpmf_tag_to_maker_note_id("UNKN"), None);
    }

    #[test]
    fn test_parse_empty_data() {
        let parser = GoProMakerNoteParser;
        let result = parser.parse(&[], Endian::Little, 0);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
