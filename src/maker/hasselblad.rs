//! Hasselblad maker note parser
//!
//! Hasselblad maker notes use a standard IFD structure similar to standard EXIF.
//! Unlike Canon or Nikon, Hasselblad doesn't have complex binary structures or encryption,
//! making them relatively straightforward to parse.

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/MakerNotes.pm"]

use crate::core::ifd::{IfdParser, TiffHeader};
use crate::core::{Endian, ExifValue};
use crate::error::Result;
use crate::maker::MakerNoteParser;
use std::collections::HashMap;

/// Parser for Hasselblad maker notes
pub struct HasselbladMakerNoteParser;

impl MakerNoteParser for HasselbladMakerNoteParser {
    fn parse(
        &self,
        data: &[u8],
        byte_order: Endian,
        _base_offset: usize,
    ) -> Result<HashMap<u16, ExifValue>> {
        // Hasselblad maker notes start directly with an IFD (no header)
        // They use the same byte order as the main EXIF data

        if data.is_empty() {
            return Ok(HashMap::new());
        }

        // Hasselblad maker notes are simpler than Canon - no footer handling needed
        // Parse as a standard IFD directly

        // Create a fake TIFF header for IFD parsing
        // (Hasselblad maker notes don't have a TIFF header, they start directly with IFD)
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

        match IfdParser::parse_ifd(&tiff_data, &header, 8) {
            Ok(parsed) => Ok(parsed.entries().clone()),
            Err(e) => {
                // Log the error but return empty results
                // Many maker notes have quirks that might cause parsing errors
                eprintln!("Warning: Hasselblad maker note parsing failed: {}", e);
                Ok(HashMap::new())
            }
        }
    }

    fn manufacturer(&self) -> &'static str {
        "Hasselblad"
    }
}

/// Hasselblad-specific tag IDs
/// Based on comments from ExifTool's MakerNotes.pm
pub mod tags {
    // Known Hasselblad tags from ExifTool comments
    pub const SENSOR_CODE: u16 = 0x0011;
    pub const CAMERA_MODEL_ID: u16 = 0x0012;
    pub const CAMERA_MODEL_NAME: u16 = 0x0015;
    pub const COATING_CODE: u16 = 0x0016;
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
}
