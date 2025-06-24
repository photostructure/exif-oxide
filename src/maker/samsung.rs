//! Samsung maker note parser
//!
//! Samsung maker notes use two different formats:
//! 1. "STMN" binary format (older models) - requires ProcessBinaryData
//! 2. Standard EXIF IFD format (newer models) - similar to Canon/Pentax
//!
//! This implementation focuses on the EXIF format (Type2) maker notes
//! found in newer Samsung models.

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Samsung.pm"]

use crate::core::ifd::{IfdParser, TiffHeader};
use crate::core::{Endian, ExifValue};
use crate::error::Result;
use crate::maker::MakerNoteParser;
use std::collections::HashMap;

/// Parser for Samsung maker notes
pub struct SamsungMakerNoteParser;

impl MakerNoteParser for SamsungMakerNoteParser {
    fn parse(
        &self,
        data: &[u8],
        byte_order: Endian,
        _base_offset: usize,
    ) -> Result<HashMap<u16, ExifValue>> {
        // Samsung maker notes have two main formats:
        // 1. "STMN" binary format (older Samsung models)
        // 2. Standard EXIF IFD format (newer Samsung models)

        if data.is_empty() {
            return Ok(HashMap::new());
        }

        // Check for STMN signature (binary format)
        if data.len() >= 4 && &data[0..4] == b"STMN" {
            // TODO: Implement ProcessBinaryData framework for STMN format
            // For now, return empty results for STMN format
            eprintln!("Warning: Samsung STMN binary format not yet supported");
            return Ok(HashMap::new());
        }

        // Check for standard EXIF format (Type2 format)
        // Samsung Type2 maker notes start directly with an IFD (no special header)
        // They use the same byte order as the main EXIF data

        // Create a fake TIFF header for IFD parsing
        // (Samsung Type2 maker notes don't have a TIFF header, they start directly with IFD)
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
                eprintln!("Warning: Samsung maker note parsing failed: {}", e);
                Ok(HashMap::new())
            }
        }
    }

    fn manufacturer(&self) -> &'static str {
        "Samsung"
    }
}

/// Samsung-specific tag IDs from Type2 format
pub mod tags {
    // Main Samsung Type2 tags (from Samsung.pm lines 134-300+)
    pub const MAKER_NOTE_VERSION: u16 = 0x0001;
    pub const DEVICE_TYPE: u16 = 0x0002;
    pub const SAMSUNG_MODEL_ID: u16 = 0x0003;
    pub const ORIENTATION_INFO: u16 = 0x0011;
    pub const SMART_ALBUM_COLOR: u16 = 0x0020;
    pub const PICTURE_WIZARD: u16 = 0x0021;
    pub const LOCAL_LOCATION_NAME: u16 = 0x0030;
    pub const LOCATION_NAME: u16 = 0x0031;
    pub const FACE_DETECT: u16 = 0x0035;
    pub const FACE_RECOGNITION: u16 = 0x0036;
    pub const FACE_NAME: u16 = 0x0037;
    pub const LENS_TYPE: u16 = 0xa003;
    pub const LENS_FIRMWARE: u16 = 0xa004;
    pub const INTERNAL_LENS_SERIAL: u16 = 0xa005;
    pub const LENS_ID: u16 = 0xa019;

    // Preview image tags from STMN format (for future ProcessBinaryData support)
    pub const PREVIEW_IMAGE_START: u16 = 0x0002; // In STMN format
    pub const PREVIEW_IMAGE_LENGTH: u16 = 0x0003; // In STMN format
}

/// Samsung device types (from DeviceType tag 0x0002)
#[allow(dead_code)]
pub mod device_types {
    pub const COMPACT_DIGITAL_CAMERA: u32 = 0x1000;
    pub const HIGH_END_NX_CAMERA: u32 = 0x2000;
    pub const HXM_VIDEO_CAMERA: u32 = 0x3000;
    pub const CELL_PHONE: u32 = 0x12000;
    pub const SMX_VIDEO_CAMERA: u32 = 0x300000;
}

/// Well-known Samsung Model IDs (from SamsungModelID tag 0x0003)
#[allow(dead_code)]
pub mod model_ids {
    pub const NX10: u32 = 0x100101c;
    pub const NX100: u32 = 0x100130c;
    pub const NX11: u32 = 0x1001327;
    pub const NX20: u32 = 0x5000000; // Note: this ID is shared by multiple models
    pub const NX200: u32 = 0x5000000; // Note: this ID is shared by multiple models
    pub const NX1000: u32 = 0x5000000; // Note: this ID is shared by multiple models
    pub const NX1: u32 = 0x5001038;
    pub const NX2000: u32 = 0x5001038; // Note: this ID is shared by multiple models
    pub const NX30: u32 = 0x5001038; // Note: this ID is shared by multiple models
    pub const NX300: u32 = 0x5001038; // Note: this ID is shared by multiple models
    pub const NX500: u32 = 0x5001038; // Note: this ID is shared by multiple models
    pub const EX1: u32 = 0x6001036;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_samsung_parser_creation() {
        let parser = SamsungMakerNoteParser;
        assert_eq!(parser.manufacturer(), "Samsung");
    }

    #[test]
    fn test_empty_maker_note() {
        let parser = SamsungMakerNoteParser;
        let result = parser.parse(&[], Endian::Little, 0).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_stmn_format_detection() {
        let parser = SamsungMakerNoteParser;
        let stmn_data = b"STMN\x00\x00\x00\x01test data";
        let result = parser.parse(stmn_data, Endian::Little, 0).unwrap();
        // Should return empty for now since STMN format is not yet implemented
        assert!(result.is_empty());
    }
}
