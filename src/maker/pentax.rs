//! Pentax maker note parser using table-driven approach
//!
//! Pentax maker notes use a standard IFD structure similar to standard EXIF,
//! making them one of the simplest manufacturer formats to parse.
//! The format is consistent across all Pentax and Ricoh cameras.
//!
//! This implementation uses auto-generated tag tables and print conversion
//! functions, eliminating the need to manually port ExifTool's Perl code.

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Pentax.pm"]

use crate::core::ifd::{IfdParser, TiffHeader};
use crate::core::print_conv::apply_print_conv;
use crate::core::{Endian, ExifValue};
use crate::error::Result;
use crate::maker::pentax::detection::{detect_pentax_maker_note, PENTAXDetectionResult};

pub mod detection;
use crate::maker::MakerNoteParser;
use crate::tables::pentax_tags::get_pentax_tag;
use std::collections::HashMap;

/// Parser for Pentax maker notes
pub struct PentaxMakerNoteParser;

impl MakerNoteParser for PentaxMakerNoteParser {
    fn parse(
        &self,
        data: &[u8],
        byte_order: Endian,
        _base_offset: usize,
    ) -> Result<HashMap<u16, ExifValue>> {
        if data.is_empty() {
            return Ok(HashMap::new());
        }

        // Use generated detection logic to identify Pentax maker note format
        let detection = match detect_pentax_maker_note(data) {
            Some(detection) => detection,
            None => {
                // Fallback: assume standard IFD at start of data
                PENTAXDetectionResult {
                    version: None,
                    ifd_offset: 0,
                    description: "Fallback Pentax parser".to_string(),
                }
            }
        };

        // Extract raw IFD data starting from detected offset
        let ifd_data = &data[detection.ifd_offset..];
        if ifd_data.is_empty() {
            return Ok(HashMap::new());
        }

        // Parse using table-driven approach
        parse_pentax_ifd_with_tables(ifd_data, byte_order)
    }

    fn manufacturer(&self) -> &'static str {
        "Pentax"
    }
}

/// Parse Pentax IFD using generated tag tables and print conversion
fn parse_pentax_ifd_with_tables(
    data: &[u8],
    byte_order: Endian,
) -> Result<HashMap<u16, ExifValue>> {
    // Create a fake TIFF header for IFD parsing
    // (Pentax maker notes don't have a TIFF header, they start directly with IFD)
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
            eprintln!("Warning: Pentax IFD parsing failed: {}", e);
            return Ok(HashMap::new());
        }
    };

    // Convert raw IFD entries to Pentax tags with print conversion
    let mut result = HashMap::new();

    for (tag_id, raw_value) in parsed_ifd.entries() {
        if let Some(pentax_tag) = get_pentax_tag(*tag_id) {
            // Apply print conversion to create human-readable value
            let converted_value = apply_print_conv(raw_value, pentax_tag.print_conv);

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

/// Pentax-specific tag IDs
pub mod tags {
    // Main Pentax tags from ExifTool
    pub const PENTAX_VERSION: u16 = 0x0000;
    pub const PENTAX_MODEL_TYPE: u16 = 0x0001;
    pub const PREVIEW_IMAGE_SIZE: u16 = 0x0002;
    pub const PREVIEW_IMAGE_LENGTH: u16 = 0x0003;
    pub const PREVIEW_IMAGE_START: u16 = 0x0004;
    pub const PENTAX_MODEL_ID: u16 = 0x0005;
    pub const DATE: u16 = 0x0006;
    pub const TIME: u16 = 0x0007;
    pub const QUALITY: u16 = 0x0008;
    pub const PENTAX_IMAGE_SIZE: u16 = 0x0009;
    pub const PICTURE_MODE: u16 = 0x000b;
    pub const FLASH_MODE: u16 = 0x000c;
    pub const FOCUS_MODE: u16 = 0x000d;
    pub const AF_POINT_SELECTED: u16 = 0x000e;
    pub const AF_POINT_IN_FOCUS: u16 = 0x000f;
    pub const EXPOSURE_TIME: u16 = 0x0012;
    pub const F_NUMBER: u16 = 0x0013;
    pub const ISO: u16 = 0x0014;
    pub const EXPOSURE_COMPENSATION: u16 = 0x0016;
    pub const METERING_MODE: u16 = 0x0017;
    pub const AUTO_BRACKETING: u16 = 0x0018;
    pub const WHITE_BALANCE: u16 = 0x0019;
    pub const WHITE_BALANCE_MODE: u16 = 0x001a;
    pub const FOCAL_LENGTH: u16 = 0x001d;
    pub const SATURATION: u16 = 0x001f;
    pub const CONTRAST: u16 = 0x0020;
    pub const SHARPNESS: u16 = 0x0021;
    pub const WORLD_TIME_LOCATION: u16 = 0x0022;
    pub const HOMETOWN_CITY: u16 = 0x0023;
    pub const DESTINATION_CITY: u16 = 0x0024;
    pub const HOMETOWN_DST: u16 = 0x0025;
    pub const DESTINATION_DST: u16 = 0x0026;
    pub const LANGUAGE: u16 = 0x002d;
    pub const COLOR_TEMPERATURE: u16 = 0x0032;
    pub const COLOR_SPACE: u16 = 0x0037;
    pub const LENS_TYPE: u16 = 0x003f;
    pub const LENS_INFO: u16 = 0x0207;
    pub const LENS_ID: u16 = 0x03fd;
}

// TODO: Implement ProcessBinaryData tables for Pentax video metadata
// Based on ExifTool's Pentax.pm ProcessBinaryData implementations
/*
pub mod binary_tables {
    use super::*;

    /// Create MOV video metadata table (Optio WP)
    /// ExifTool source: %Image::ExifTool::Pentax::MOV (line 5929)
    pub fn create_mov_table() -> BinaryDataTable {
        let mut table = BinaryDataTableBuilder::new("PentaxMOV", ExifFormat::U8);

        // Add string field with custom offset
        let mut make_field = BinaryField {
            name: "Make".to_string(),
            offset: 0x00,
            format: Some(ExifFormat::Ascii),
            count: 24,
            mask: None,
            shift: 0,
            print_conv: None,
        };
        table.table.add_field(0x00, make_field);

        let mut exposure_field = BinaryField {
            name: "ExposureTime".to_string(),
            offset: 0x26,
            format: Some(ExifFormat::U32),
            count: 1,
            mask: None,
            shift: 0,
            print_conv: Some("$val ? 10 / $val : 0".to_string()),
        };
        table.table.add_field(0x26, exposure_field);

        let mut fnumber_field = BinaryField {
            name: "FNumber".to_string(),
            offset: 0x2a,
            format: Some(ExifFormat::Rational),
            count: 1,
            mask: None,
            shift: 0,
            print_conv: None,
        };
        table.table.add_field(0x2a, fnumber_field);

        let mut iso_field = BinaryField {
            name: "ISO".to_string(),
            offset: 0xaf,
            format: Some(ExifFormat::U16),
            count: 1,
            mask: None,
            shift: 0,
            print_conv: None,
        };
        table.table.add_field(0xaf, iso_field);

        table.build()
    }

    /// Create PENT video metadata table (Optio WG-2 GPS)
    /// ExifTool source: %Image::ExifTool::Pentax::PENT (line 6044)
    pub fn create_pent_table() -> BinaryDataTable {
        let mut table = BinaryDataTableBuilder::new("PentaxPENT", ExifFormat::U8);

        // Core camera fields
        let mut make_field = BinaryField {
            name: "Make".to_string(),
            offset: 0x00,
            format: Some(ExifFormat::Ascii),
            count: 24,
            mask: None,
            shift: 0,
            print_conv: None,
        };
        table.table.add_field(0x00, make_field);

        let mut model_field = BinaryField {
            name: "Model".to_string(),
            offset: 0x1a,
            format: Some(ExifFormat::Ascii),
            count: 24,
            mask: None,
            shift: 0,
            print_conv: None,
        };
        table.table.add_field(0x1a, model_field);

        let mut iso_field = BinaryField {
            name: "ISO".to_string(),
            offset: 0xa7,
            format: Some(ExifFormat::U32),
            count: 1,
            mask: None,
            shift: 0,
            print_conv: None,
        };
        table.table.add_field(0xa7, iso_field);

        // GPS fields (basic subset)
        let mut gps_lat_ref_field = BinaryField {
            name: "GPSLatitudeRef".to_string(),
            offset: 0xcf,
            format: Some(ExifFormat::Ascii),
            count: 2,
            mask: None,
            shift: 0,
            print_conv: None,
        };
        table.table.add_field(0xcf, gps_lat_ref_field);

        let mut gps_lon_ref_field = BinaryField {
            name: "GPSLongitudeRef".to_string(),
            offset: 0xe9,
            format: Some(ExifFormat::Ascii),
            count: 2,
            mask: None,
            shift: 0,
            print_conv: None,
        };
        table.table.add_field(0xe9, gps_lon_ref_field);

        table.build()
    }

    /// Create AVI Junk metadata table (RS1000)
    /// ExifTool source: %Image::ExifTool::Pentax::Junk (line 6014)
    pub fn create_junk_table() -> BinaryDataTable {
        let mut table = BinaryDataTableBuilder::new("PentaxJunk", ExifFormat::U8);

        let mut model_field = BinaryField {
            name: "Model".to_string(),
            offset: 0x0c,
            format: Some(ExifFormat::Ascii),
            count: 32,
            mask: None,
            shift: 0,
            print_conv: None,
        };
        table.table.add_field(0x0c, model_field);

        table.build()
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::binary_data::BinaryDataTableBuilder;
    use crate::core::types::ExifFormat;

    #[test]
    fn test_pentax_parser_creation() {
        let parser = PentaxMakerNoteParser;
        assert_eq!(parser.manufacturer(), "Pentax");
    }

    #[test]
    fn test_empty_maker_note() {
        let parser = PentaxMakerNoteParser;
        let result = parser.parse(&[], Endian::Little, 0).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_pentax_processbinarydata_framework() {
        // Test that the ProcessBinaryData framework can be used for Pentax video metadata
        // This creates a simplified version of the MOV table
        let table = BinaryDataTableBuilder::new("PentaxMOVTest", ExifFormat::U8)
            .add_field(0x26, "ExposureTime", ExifFormat::U16, 1)
            .build();

        // Create test data with a known value at offset 0x26
        let mut test_data = vec![0u8; 0x30];
        test_data[0x26] = 0x34; // Low byte
        test_data[0x27] = 0x12; // High byte (little-endian = 0x1234)

        // Parse the binary data
        let result = table
            .parse(&test_data, crate::core::Endian::Little)
            .unwrap();

        // Verify we got the expected result
        assert_eq!(result.len(), 1);
        assert!(result.contains_key(&0x8026)); // 0x8000 + 0x26

        if let Some(crate::core::ExifValue::U16(value)) = result.get(&0x8026) {
            assert_eq!(*value, 0x1234);
        } else {
            panic!("Expected U16 value at tag 0x8026");
        }
    }

    #[test]
    fn test_pentax_binary_data_string_field() {
        // Test string field parsing for manufacturer name
        let table = BinaryDataTableBuilder::new("PentaxStringTest", ExifFormat::U8)
            .add_field(0x00, "Make", ExifFormat::Ascii, 8)
            .build();

        // Create test data with "PENTAX\0\0" at start
        let test_data = b"PENTAX\0\0extra_data";

        // Parse the binary data
        let result = table.parse(test_data, crate::core::Endian::Little).unwrap();

        // Verify we got the expected result
        assert_eq!(result.len(), 1);
        assert!(result.contains_key(&0x8000)); // 0x8000 + 0x00

        // For now, binary data returns U8Array for strings
        // In a full implementation, this would be converted to an Ascii string
        if let Some(crate::core::ExifValue::U8Array(bytes)) = result.get(&0x8000) {
            assert_eq!(bytes.len(), 8);
            assert_eq!(&bytes[0..6], b"PENTAX");
        } else {
            panic!(
                "Expected U8Array at tag 0x8000, got: {:?}",
                result.get(&0x8000)
            );
        }
    }
}
