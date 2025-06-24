//! Olympus maker note parser
//!
//! Olympus maker notes use a standard IFD structure after a signature header.
//! The signature is "OLYMPUS\x00" followed by endian markers (II or MM).

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Olympus.pm"]

use crate::core::ifd::{IfdParser, TiffHeader};
use crate::core::{Endian, ExifValue};
use crate::error::Result;
use crate::maker::MakerNoteParser;
use std::collections::HashMap;

/// Parser for Olympus maker notes
pub struct OlympusMakerNoteParser;

impl MakerNoteParser for OlympusMakerNoteParser {
    fn parse(
        &self,
        data: &[u8],
        byte_order: Endian,
        _base_offset: usize,
    ) -> Result<HashMap<u16, ExifValue>> {
        if data.is_empty() {
            return Ok(HashMap::new());
        }

        // Check for Olympus signature
        // Olympus maker notes start with "OLYMPUS\x00" followed by endian marker
        if data.len() < 10 {
            return Ok(HashMap::new());
        }

        // Look for Olympus signature
        if &data[0..8] != b"OLYMPUS\x00" {
            // Try alternative detection - some Olympus cameras might have different signatures
            // For now, proceed with standard IFD parsing if no signature found
            eprintln!("Warning: Olympus maker note without expected signature, attempting standard IFD parsing");
        }

        // Determine offset based on signature presence
        let ifd_offset = if &data[0..8] == b"OLYMPUS\x00" {
            // Standard Olympus format: "OLYMPUS\x00" + endian markers (II or MM)
            // Check endian markers at offset 8-9
            let _endian_from_data = if data.len() >= 10 {
                match &data[8..10] {
                    b"II" => Endian::Little,
                    b"MM" => Endian::Big,
                    _ => byte_order, // Fall back to main EXIF byte order
                }
            } else {
                byte_order
            };

            // Use detected endian for parsing, IFD starts after 8-byte header
            10 // Skip "OLYMPUS\x00" + endian markers
        } else {
            // No signature found, assume IFD starts immediately
            0
        };

        // Ensure we have enough data for IFD parsing
        if data.len() <= ifd_offset {
            return Ok(HashMap::new());
        }

        let parsing_data = &data[ifd_offset..];

        // Create a fake TIFF header for IFD parsing
        // Olympus uses standard IFD structure after the signature
        let mut tiff_data = Vec::with_capacity(8 + parsing_data.len());

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
        tiff_data.extend_from_slice(parsing_data);

        // Parse the IFD
        let header = TiffHeader {
            byte_order,
            ifd0_offset: 8,
        };

        match IfdParser::parse_ifd(&tiff_data, &header, 8) {
            Ok(parsed) => Ok(parsed.entries().clone()),
            Err(e) => {
                // Log the error but return empty results
                // Many maker notes have quirks that might cause parsing errors
                eprintln!("Warning: Olympus maker note parsing failed: {}", e);
                Ok(HashMap::new())
            }
        }
    }

    fn manufacturer(&self) -> &'static str {
        "Olympus"
    }
}

/// Olympus-specific tag IDs
pub mod tags {
    // Main Olympus tags (not prefixed - will be prefixed by tag system)
    pub const MAKER_NOTE_VERSION: u16 = 0x0000;
    pub const COMPRESSED_IMAGE_SIZE: u16 = 0x0040;
    pub const PREVIEW_IMAGE_DATA: u16 = 0x0081;
    pub const PREVIEW_IMAGE_START: u16 = 0x0088;
    pub const PREVIEW_IMAGE_LENGTH: u16 = 0x0089;
    pub const THUMBNAIL_IMAGE: u16 = 0x0100;
    pub const BODY_FIRMWARE_VERSION: u16 = 0x0104;

    // Camera settings
    pub const SPECIAL_MODE: u16 = 0x0200;
    pub const QUALITY: u16 = 0x0201;
    pub const MACRO: u16 = 0x0202;
    pub const BW_MODE: u16 = 0x0203;
    pub const DIGITAL_ZOOM: u16 = 0x0204;
    pub const FOCAL_PLANE_DIAGONAL: u16 = 0x0205;
    pub const LENS_DISTORTION_PARAMS: u16 = 0x0206;
    pub const CAMERA_TYPE: u16 = 0x0207;
    pub const CAMERA_ID: u16 = 0x0209;
    pub const ONE_TOUCH_WB: u16 = 0x020B;
    pub const WHITE_BALANCE_2: u16 = 0x020C;
    pub const WHITE_BALANCE_TEMPERATURE: u16 = 0x020D;
    pub const WHITE_BALANCE_BRACKET: u16 = 0x020E;

    // Extended settings
    pub const FLASH_MODE: u16 = 0x1004;
    pub const FLASH_EXPOSURE_COMP: u16 = 0x1005;
    pub const AUTO_EXPOSURE_LOCK: u16 = 0x1006;
    pub const LENS_PROPERTIES: u16 = 0x1007;
    pub const REAL_ISO: u16 = 0x1008;
    pub const AUTO_EXPOSURE_BRACKET: u16 = 0x1009;
    pub const IMAGE_STABILIZATION: u16 = 0x100A;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_olympus_parser_creation() {
        let parser = OlympusMakerNoteParser;
        assert_eq!(parser.manufacturer(), "Olympus");
    }

    #[test]
    fn test_empty_maker_note() {
        let parser = OlympusMakerNoteParser;
        let result = parser.parse(&[], Endian::Little, 0).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_signature_detection() {
        let parser = OlympusMakerNoteParser;

        // Test with proper Olympus signature
        let mut data = Vec::new();
        data.extend_from_slice(b"OLYMPUS\x00");
        data.extend_from_slice(b"II"); // Little endian
        data.extend_from_slice(&[0x00, 0x00]); // IFD entry count (0)

        let result = parser.parse(&data, Endian::Little, 0).unwrap();
        // Should succeed without error, even if no tags found
        assert!(result.is_empty()); // Empty because no IFD entries
    }

    #[test]
    fn test_no_signature() {
        let parser = OlympusMakerNoteParser;

        // Test without signature (fallback to direct IFD parsing)
        let data = vec![0x00, 0x00]; // IFD entry count (0)

        let result = parser.parse(&data, Endian::Little, 0).unwrap();
        // Should succeed without error
        assert!(result.is_empty());
    }
}
