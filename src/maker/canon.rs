//! Canon maker note parser
//!
//! Canon maker notes use an IFD structure similar to standard EXIF,
//! but with Canon-specific tags and sometimes special offset handling.

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Canon.pm"]
#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/CanonRaw.pm"]

use crate::core::ifd::{IfdParser, TiffHeader};
use crate::core::print_conv::apply_print_conv;
use crate::core::{Endian, ExifValue};
use crate::error::Result;
use crate::maker::MakerNoteParser;
use crate::tables::canon_tags::get_canon_tag;
use std::collections::HashMap;

/// Parser for Canon maker notes
pub struct CanonMakerNoteParser;

impl MakerNoteParser for CanonMakerNoteParser {
    fn parse(
        &self,
        data: &[u8],
        byte_order: Endian,
        _base_offset: usize,
    ) -> Result<HashMap<u16, ExifValue>> {
        // Canon maker notes start directly with an IFD (no header)
        // They use the same byte order as the main EXIF data

        if data.is_empty() {
            return Ok(HashMap::new());
        }

        // Canon maker notes may have a footer with offset information
        // Check if there's an 8-byte footer at the end
        let mut actual_length = data.len();
        if data.len() >= 8 {
            let footer_start = data.len() - 8;
            let footer = &data[footer_start..];

            // Check for TIFF-like footer (II*\0 or MM\0*)
            if (footer[0..2] == [b'I', b'I'] || footer[0..2] == [b'M', b'M'])
                && (footer[2..4] == [0x2a, 0x00] || footer[2..4] == [0x00, 0x2a])
            {
                // This is a Canon footer, exclude it from parsing
                actual_length = footer_start;
            }
        }

        // Parse as a standard IFD
        let parsing_data = &data[..actual_length];

        // Create a fake TIFF header for IFD parsing
        // (Canon maker notes don't have a TIFF header, they start directly with IFD)
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

        // Parse using table-driven approach with PrintConv
        parse_canon_ifd_with_tables(&tiff_data, &header)
    }

    fn manufacturer(&self) -> &'static str {
        "Canon"
    }
}

/// Parse Canon IFD using generated tag tables and print conversion
fn parse_canon_ifd_with_tables(
    tiff_data: &[u8],
    header: &TiffHeader,
) -> Result<HashMap<u16, ExifValue>> {
    let parsed_ifd = match IfdParser::parse_ifd(tiff_data, header, 8) {
        Ok(parsed) => parsed,
        Err(e) => {
            eprintln!("Warning: Canon IFD parsing failed: {}", e);
            return Ok(HashMap::new());
        }
    };

    // Convert raw IFD entries to Canon tags with print conversion
    let mut result = HashMap::new();

    for (tag_id, raw_value) in parsed_ifd.entries() {
        if let Some(canon_tag) = get_canon_tag(*tag_id) {
            // Apply print conversion to create human-readable value
            let converted_value = apply_print_conv(raw_value, canon_tag.print_conv);

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

/// Canon-specific tag IDs
pub mod tags {
    // Main Canon tags
    pub const CAMERA_SETTINGS: u16 = 0x0001;
    pub const FOCAL_LENGTH: u16 = 0x0002;
    pub const SHOT_INFO: u16 = 0x0004;
    pub const PANORAMA: u16 = 0x0005;
    pub const IMAGE_TYPE: u16 = 0x0006;
    pub const FIRMWARE_VERSION: u16 = 0x0007;
    pub const FILE_NUMBER: u16 = 0x0008;
    pub const OWNER_NAME: u16 = 0x0009;
    pub const SERIAL_NUMBER: u16 = 0x000c;
    pub const CAMERA_INFO: u16 = 0x000d;
    pub const CUSTOM_FUNCTIONS: u16 = 0x000f;
    pub const MODEL_ID: u16 = 0x0010;
    pub const AF_INFO: u16 = 0x0012;

    // CameraSettings sub-tags (tag 0x0001)
    pub const MACRO_MODE: u16 = 0x0001;
    pub const SELF_TIMER: u16 = 0x0002;
    pub const QUALITY: u16 = 0x0003;
    pub const FLASH_MODE: u16 = 0x0004;
    pub const DRIVE_MODE: u16 = 0x0005;
    pub const FOCUS_MODE: u16 = 0x0007;
    pub const IMAGE_SIZE: u16 = 0x000a;
    pub const EASY_MODE: u16 = 0x000b;
    pub const DIGITAL_ZOOM: u16 = 0x000c;
    pub const CONTRAST: u16 = 0x000d;
    pub const SATURATION: u16 = 0x000e;
    pub const SHARPNESS: u16 = 0x000f;
    pub const ISO: u16 = 0x0010;
    pub const METERING_MODE: u16 = 0x0011;
    pub const FOCUS_TYPE: u16 = 0x0012;
    pub const AF_POINT: u16 = 0x0013;
    pub const EXPOSURE_MODE: u16 = 0x0014;
    pub const LENS_ID: u16 = 0x0016;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canon_parser_creation() {
        let parser = CanonMakerNoteParser;
        assert_eq!(parser.manufacturer(), "Canon");
    }

    #[test]
    fn test_empty_maker_note() {
        let parser = CanonMakerNoteParser;
        let result = parser.parse(&[], Endian::Little, 0).unwrap();
        assert!(result.is_empty());
    }
}
