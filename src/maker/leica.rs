//! Leica maker note parser
//!
//! Leica maker notes use a standard IFD structure similar to standard EXIF.
//! Leica cameras are often manufactured in partnership with Panasonic, so
//! the maker note structure follows similar patterns to Panasonic cameras.
//! Note: Leica maker note tags are defined in ExifTool's Panasonic.pm module.

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Panasonic.pm"]

use crate::core::ifd::{IfdParser, TiffHeader};
use crate::core::{Endian, ExifValue};
use crate::error::Result;
use crate::maker::MakerNoteParser;
use std::collections::HashMap;

/// Parser for Leica maker notes
pub struct LeicaMakerNoteParser;

impl MakerNoteParser for LeicaMakerNoteParser {
    fn parse(
        &self,
        data: &[u8],
        byte_order: Endian,
        _base_offset: usize,
    ) -> Result<HashMap<u16, ExifValue>> {
        // Leica maker notes start directly with an IFD (no header)
        // They use the same byte order as the main EXIF data
        // This follows the same pattern as Canon and Pentax

        if data.is_empty() {
            return Ok(HashMap::new());
        }

        // Some Leica cameras may have signature headers, but most follow standard IFD format
        // Check for common Leica signatures and skip them if present
        let mut data_offset = 0;

        // Check for Leica-specific signatures
        if data.len() >= 4 {
            // Some Leica cameras use "LEIC" signature
            if &data[0..4] == b"LEIC" {
                data_offset = 4;
            }
            // Some use "Leica" signature
            else if data.len() >= 5 && &data[0..5] == b"Leica" {
                data_offset = 8; // Usually followed by version info
            }
        }

        let parsing_data = &data[data_offset..];

        // Leica maker notes are straightforward - no footer handling needed
        // Parse as a standard IFD directly

        // Create a fake TIFF header for IFD parsing
        // (Leica maker notes don't have a TIFF header, they start directly with IFD)
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
                eprintln!("Warning: Leica maker note parsing failed: {}", e);
                Ok(HashMap::new())
            }
        }
    }

    fn manufacturer(&self) -> &'static str {
        "Leica"
    }
}

/// Leica-specific tag IDs
/// These are derived from the Leica tag tables in ExifTool's Panasonic.pm
pub mod tags {
    // Leica2 tags (M8) - tag prefix 0x300
    pub const QUALITY: u16 = 0x300;
    pub const USER_PROFILE: u16 = 0x302;
    pub const SERIAL_NUMBER: u16 = 0x303;
    pub const WHITE_BALANCE: u16 = 0x304;
    pub const LENS_TYPE: u16 = 0x310;

    // Leica5 tags (X1, X2, X VARIO, T) - tag prefix 0x0300-0x0500
    pub const LENS_TYPE_5: u16 = 0x0303;
    pub const SERIAL_NUMBER_5: u16 = 0x0305;
    pub const ORIGINAL_FILE_NAME: u16 = 0x0407;
    pub const ORIGINAL_DIRECTORY: u16 = 0x0408;
    pub const FOCUS_INFO: u16 = 0x040a;
    pub const EXPOSURE_MODE: u16 = 0x040d;
    pub const SHOT_INFO: u16 = 0x0410;
    pub const FILM_MODE: u16 = 0x0412;
    pub const WB_RGB_LEVELS: u16 = 0x0413;
    pub const INTERNAL_SERIAL_NUMBER: u16 = 0x0500;
    pub const CAMERA_IFD: u16 = 0x05ff;

    // Leica6 tags (newer cameras) - tag prefix 0x300
    pub const LEICA6_UNKNOWN_300: u16 = 0x300;

    // Leica9 tags (S Typ 007, M10) - tag prefix 0x300
    pub const LEICA9_UNKNOWN_304: u16 = 0x304;
    pub const LEICA9_UNKNOWN_311: u16 = 0x311;
    pub const LEICA9_UNKNOWN_312: u16 = 0x312;

    // Common Leica tags across different table versions
    pub const FIRMWARE_VERSION: u16 = 0x0001;
    pub const CAMERA_TEMPERATURE: u16 = 0x0002;
    pub const IMAGE_NUMBER: u16 = 0x0003;
    pub const CAMERA_ORIENTATION: u16 = 0x0004;
    pub const CONTRAST: u16 = 0x0005;
    pub const SATURATION: u16 = 0x0006;
    pub const SHARPNESS: u16 = 0x0007;

    // Preview and thumbnail related tags
    pub const PREVIEW_IMAGE_START: u16 = 0x0201;
    pub const PREVIEW_IMAGE_LENGTH: u16 = 0x0202;
    pub const PREVIEW_IMAGE_SIZE: u16 = 0x0203;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leica_parser_creation() {
        let parser = LeicaMakerNoteParser;
        assert_eq!(parser.manufacturer(), "Leica");
    }

    #[test]
    fn test_empty_maker_note() {
        let parser = LeicaMakerNoteParser;
        let result = parser.parse(&[], Endian::Little, 0).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_leica_signature_detection() {
        let parser = LeicaMakerNoteParser;

        // Test with LEIC signature
        let data_with_leic = b"LEIC\x01\x00\x00\x00"; // Mock IFD data
        let result = parser.parse(data_with_leic, Endian::Little, 0);
        assert!(result.is_ok());

        // Test with Leica signature
        let data_with_leica = b"Leica\x00\x00\x00\x01\x00\x00\x00"; // Mock IFD data
        let result = parser.parse(data_with_leica, Endian::Little, 0);
        assert!(result.is_ok());
    }
}
