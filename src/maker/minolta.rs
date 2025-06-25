//! Minolta maker note parser using table-driven approach
//!
//! Minolta maker notes use multiple formats based on camera model:
//! - Type 1: Standard IFD format for most Konica Minolta/Minolta cameras
//! - Type 2: MINOL/CAMER signature (uses Olympus tag table)
//! - Type 3: Binary formats (MLY0, KC, +M+M, 0xd7) for specific models
//!
//! This implementation uses auto-generated tag tables and print conversion
//! functions, eliminating the need to manually port ExifTool's Perl code.

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Minolta.pm"]

use crate::core::ifd::{IfdParser, TiffHeader};
use crate::core::print_conv::apply_print_conv;
use crate::core::{Endian, ExifValue};
use crate::error::Result;
use crate::maker::minolta::detection::{detect_minolta_maker_note, MINOLTADetectionResult};

pub mod detection;
use crate::maker::MakerNoteParser;
use crate::tables::minolta_tags::get_minolta_tag;
use std::collections::HashMap;

/// Parser for Minolta maker notes
pub struct MinoltaMakerNoteParser;

impl MakerNoteParser for MinoltaMakerNoteParser {
    fn parse(
        &self,
        data: &[u8],
        byte_order: Endian,
        _base_offset: usize,
    ) -> Result<HashMap<u16, ExifValue>> {
        if data.is_empty() {
            return Ok(HashMap::new());
        }

        // Use generated detection logic to identify Minolta maker note format
        let detection = match detect_minolta_maker_note(data) {
            Ok(detection) => {
                if !detection.detected {
                    return Ok(HashMap::new());
                }
                detection
            }
            Err(_) => {
                // Fallback: assume standard IFD at start of data
                MINOLTADetectionResult {
                    detected: true,
                    ifd_offset: 0,
                    byte_order: None,
                    description: "Fallback Minolta parser".to_string(),
                }
            }
        };

        // Handle different Minolta maker note types based on detection
        if detection.description.contains("Type 3") || detection.description.contains("binary") {
            // Type 3: Binary format - not IFD-based, limited parsing
            let mut result = HashMap::new();
            result.insert(0x0000, ExifValue::Ascii(detection.description.clone()));
            return Ok(result);
        }

        // Type 1 & 2: IFD-based formats
        let ifd_offset = detection.ifd_offset;

        // Extract raw IFD data starting from detected offset
        let ifd_data = &data[ifd_offset..];
        if ifd_data.is_empty() {
            return Ok(HashMap::new());
        }

        // Parse using table-driven approach
        parse_minolta_ifd_with_tables(ifd_data, byte_order)
    }

    fn manufacturer(&self) -> &'static str {
        "Minolta"
    }
}

/// Parse Minolta IFD using generated tag tables and print conversion
fn parse_minolta_ifd_with_tables(
    data: &[u8],
    byte_order: Endian,
) -> Result<HashMap<u16, ExifValue>> {
    // Create a fake TIFF header for IFD parsing
    // (Minolta maker notes don't have a TIFF header, they start directly with IFD)
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
            eprintln!("Warning: Minolta IFD parsing failed: {}", e);
            return Ok(HashMap::new());
        }
    };

    // Convert raw IFD entries to Minolta tags with print conversion
    let mut result = HashMap::new();

    for (tag_id, raw_value) in parsed_ifd.entries() {
        if let Some(minolta_tag) = get_minolta_tag(*tag_id) {
            // Apply print conversion to create human-readable value
            let converted_value = apply_print_conv(raw_value, minolta_tag.print_conv);

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

/// Minolta-specific tag IDs from ExifTool
pub mod tags {
    // Main Minolta tags from ExifTool Minolta.pm
    pub const MAKER_NOTE_VERSION: u16 = 0x0000;
    pub const CAMERA_SETTINGS_1: u16 = 0x0001;
    pub const CAMERA_SETTINGS_2: u16 = 0x0003;
    pub const COMPRESSED_IMAGE_SIZE: u16 = 0x0040;
    pub const PREVIEW_IMAGE_START: u16 = 0x0088;
    pub const PREVIEW_IMAGE_LENGTH: u16 = 0x0089;
    pub const SCENE_MODE: u16 = 0x0100;
    pub const COLOR_MODE: u16 = 0x0101;
    pub const MINOLTA_QUALITY: u16 = 0x0102;
    pub const MINOLTA_IMAGE_SIZE: u16 = 0x0103;
    pub const FLASH_EXPOSURE_COMP: u16 = 0x0104;
    pub const TELECONVERTER: u16 = 0x0105;
    pub const WHITE_BALANCE: u16 = 0x0112;
    pub const LENS_ID: u16 = 0x0113;
    pub const AF_POINT: u16 = 0x0114;
    pub const DRIVE_MODE: u16 = 0x0115;
    pub const COLOR_SPACE: u16 = 0x0116;
    pub const SHARPNESS: u16 = 0x0118;
    pub const SUBJECT_PROGRAM: u16 = 0x0119;
    pub const FLASH_EXPOSURE_COMP2: u16 = 0x011a;
    pub const METER_MODE: u16 = 0x011b;
    pub const ISO_SETTING: u16 = 0x011c;
    pub const MODEL: u16 = 0x011d;
    pub const INTERVAL_MODE: u16 = 0x011e;
    pub const FOLDER_NAME: u16 = 0x011f;
    pub const COLOR_TEMPERATURE: u16 = 0x0120;
    pub const WHITE_BALANCE_BRACKETING: u16 = 0x0121;
    pub const PROGRAM_MODE: u16 = 0x0137;
    pub const IMAGE_NUMBER: u16 = 0x0138;
    pub const WB_INFO_A100: u16 = 0x0020;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minolta_parser_creation() {
        let parser = MinoltaMakerNoteParser;
        assert_eq!(parser.manufacturer(), "Minolta");
    }

    #[test]
    fn test_empty_maker_note() {
        let parser = MinoltaMakerNoteParser;
        let result = parser.parse(&[], Endian::Little, 0).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_minolta_type2_signature_detection() {
        let parser = MinoltaMakerNoteParser;

        // Create test data with MINOL signature
        let mut test_data = vec![0u8; 20];
        test_data[0..6].copy_from_slice(b"MINOL\0");

        // Parser should handle this without error
        let _result = parser.parse(&test_data, Endian::Little, 0).unwrap();

        // Result may be empty due to invalid IFD after signature, but should not panic
    }

    #[test]
    fn test_minolta_type3_binary_detection() {
        let parser = MinoltaMakerNoteParser;

        // Create test data with MLY0 signature (Type 3 binary)
        let mut test_data = vec![0u8; 20];
        test_data[0..4].copy_from_slice(b"MLY0");

        let result = parser.parse(&test_data, Endian::Little, 0).unwrap();

        // Type 3 binary formats return description only
        assert!(!result.is_empty());
        if let Some(ExifValue::Ascii(desc)) = result.get(&0x0000) {
            assert!(desc.contains("Type 3"));
        }
    }

    #[test]
    fn test_minolta_detection_patterns() {
        // Test the detection function directly

        // Test MINOL signature
        let test_data = b"MINOL\0test_data_here";
        let detection = detect_minolta_maker_note(test_data).unwrap();
        assert!(detection.detected);
        assert_eq!(detection.ifd_offset, 8);
        assert!(detection.description.contains("MINOL"));

        // Test MLY0 signature
        let test_data = b"MLY0test_data";
        let detection = detect_minolta_maker_note(test_data).unwrap();
        assert!(detection.detected);
        assert_eq!(detection.ifd_offset, 0);
        assert!(detection.description.contains("MLY0"));

        // Test +M+M signature
        let test_data = b"+M+Mtest_data";
        let detection = detect_minolta_maker_note(test_data).unwrap();
        assert!(detection.detected);
        assert_eq!(detection.ifd_offset, 0);
        assert!(detection.description.contains("+M+M"));
    }
}
