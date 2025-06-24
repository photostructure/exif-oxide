//! Panasonic maker note parser using table-driven approach
//!
//! Panasonic maker notes use a proprietary format that starts with "Panasonic"
//! signature followed by IFD-like structures. This format is consistent
//! across Panasonic cameras and contains camera-specific settings including
//! video recording parameters, lens corrections, and image processing settings.
//!
//! This implementation uses auto-generated tag tables and print conversion
//! functions, eliminating the need to manually port ExifTool's Perl code.

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Panasonic.pm"]

use crate::core::ifd::{IfdParser, TiffHeader};
use crate::core::print_conv::apply_print_conv;
use crate::core::{Endian, ExifValue};
use crate::error::Result;
use crate::maker::panasonic::detection::{detect_panasonic_maker_note, PANASONICDetectionResult};

pub mod detection;
use crate::maker::MakerNoteParser;
use crate::tables::panasonic_tags::get_panasonic_tag;
use std::collections::HashMap;

/// Parser for Panasonic maker notes
pub struct PanasonicMakerNoteParser;

impl MakerNoteParser for PanasonicMakerNoteParser {
    fn parse(
        &self,
        data: &[u8],
        byte_order: Endian,
        _base_offset: usize,
    ) -> Result<HashMap<u16, ExifValue>> {
        if data.is_empty() {
            return Ok(HashMap::new());
        }

        // Use generated detection logic to identify Panasonic maker note format
        let detection = match detect_panasonic_maker_note(data) {
            Some(detection) => detection,
            None => {
                // Fallback: assume standard IFD at start of data
                PANASONICDetectionResult {
                    version: None,
                    ifd_offset: 0,
                    description: "Fallback Panasonic parser".to_string(),
                }
            }
        };

        // Panasonic maker notes start with "Panasonic\0\0\0" signature (12 bytes)
        // The IFD starts immediately after the signature
        let ifd_offset = if data.len() >= 12
            && data.starts_with(&[
                0x50, 0x61, 0x6e, 0x61, 0x73, 0x6f, 0x6e, 0x69, 0x63, 0x00, 0x00, 0x00,
            ]) {
            12 // Skip "Panasonic\0\0\0" signature
        } else {
            detection.ifd_offset
        };

        // Extract raw IFD data starting from detected offset
        let ifd_data = &data[ifd_offset..];
        if ifd_data.is_empty() {
            return Ok(HashMap::new());
        }

        // Parse using table-driven approach
        parse_panasonic_ifd_with_tables(ifd_data, byte_order)
    }

    fn manufacturer(&self) -> &'static str {
        "Panasonic"
    }
}

/// Parse Panasonic IFD using generated tag tables and print conversion
fn parse_panasonic_ifd_with_tables(
    data: &[u8],
    byte_order: Endian,
) -> Result<HashMap<u16, ExifValue>> {
    // Create a fake TIFF header for IFD parsing
    // (Panasonic maker notes don't have a TIFF header, they start directly with IFD)
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
            eprintln!("Warning: Panasonic IFD parsing failed: {}", e);
            return Ok(HashMap::new());
        }
    };

    // Convert raw IFD entries to Panasonic tags with print conversion
    let mut result = HashMap::new();

    for (tag_id, raw_value) in parsed_ifd.entries() {
        if let Some(panasonic_tag) = get_panasonic_tag(*tag_id) {
            // Apply print conversion to create human-readable value
            let converted_value = apply_print_conv(raw_value, panasonic_tag.print_conv);

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

/// Panasonic-specific tag IDs
pub mod tags {
    // Main Panasonic tags from ExifTool
    pub const PANASONIC_VERSION: u16 = 0x0000;
    pub const QUALITY: u16 = 0x0001;
    pub const FIRMWARE_VERSION: u16 = 0x0002;
    pub const WHITE_BALANCE: u16 = 0x0003;
    pub const FOCUS_MODE: u16 = 0x0007;
    pub const AF_AREA_MODE: u16 = 0x000f;
    pub const IMAGE_STABILIZATION: u16 = 0x001a;
    pub const MACRO_MODE: u16 = 0x001c;
    pub const RECORD_MODE: u16 = 0x001f;
    pub const AUDIO: u16 = 0x0020;
    pub const DATA_DUMP: u16 = 0x0021;
    pub const WHITE_BALANCE_BIAS: u16 = 0x0023;
    pub const FLASH_BIAS: u16 = 0x0024;
    pub const INTERNAL_SERIAL_NUMBER: u16 = 0x0025;
    pub const PANASONIC_EXIF_VERSION: u16 = 0x0026;
    pub const COLOR_EFFECT: u16 = 0x0028;
    pub const TIME_SINCE_POWER_ON: u16 = 0x0029;
    pub const BURST_MODE: u16 = 0x002a;
    pub const SEQUENCE_NUMBER: u16 = 0x002b;
    pub const CONTRAST_MODE: u16 = 0x002c;
    pub const NOISE_REDUCTION: u16 = 0x002d;
    pub const SELF_TIMER: u16 = 0x002e;
    pub const ROTATION: u16 = 0x0030;
    pub const AF_ASSIST_LAMP: u16 = 0x0031;
    pub const COLOR_MODE: u16 = 0x0032;
    pub const BABY_AGE: u16 = 0x0033;
    pub const OPTICAL_ZOOM_MODE: u16 = 0x0034;
    pub const CONVERSION_LENS: u16 = 0x0035;
    pub const TRAVEL_DAY: u16 = 0x0036;
    pub const CONTRAST: u16 = 0x0039;
    pub const WORLD_TIME_LOCATION: u16 = 0x003a;
    pub const TEXT_STAMP: u16 = 0x003b;
    pub const PROGRAM_ISO: u16 = 0x003c;
    pub const ADVANCED_SCENE_TYPE: u16 = 0x003d;
    pub const TEXT_STAMP_2: u16 = 0x003e;
    pub const FACES_DETECTED: u16 = 0x003f;
    pub const SATURATION: u16 = 0x0040;
    pub const SHARPNESS: u16 = 0x0041;
    pub const FILM_MODE: u16 = 0x0042;
    pub const COLOR_TEMP_KELVIN: u16 = 0x0044;
    pub const BRACKET_SETTINGS: u16 = 0x0045;
    pub const WB_ADJUST_AB: u16 = 0x0046;
    pub const WB_ADJUST_GM: u16 = 0x0047;
    pub const FLASH_CURTAIN: u16 = 0x0048;
    pub const LONG_EXPOSURE_NOISE_REDUCTION: u16 = 0x0049;
    pub const PANASONIC_IMAGE_WIDTH: u16 = 0x004b;
    pub const PANASONIC_IMAGE_HEIGHT: u16 = 0x004c;
    pub const AF_POINT_POSITION: u16 = 0x004d;
    pub const FACE_DETECTION_INFO: u16 = 0x004e;
    pub const LENS_TYPE: u16 = 0x0051;
    pub const LENS_SERIAL_NUMBER: u16 = 0x0052;
    pub const ACCESSORY_TYPE: u16 = 0x0053;
    pub const ACCESSORY_SERIAL_NUMBER: u16 = 0x0054;
    pub const TRANSFORM: u16 = 0x0059;
    pub const INTELLIGENT_EXPOSURE: u16 = 0x005d;
    pub const LENS_FIRMWARE_VERSION: u16 = 0x0060;
    pub const FACE_RECOGNITION_INFO: u16 = 0x0061;
    pub const FLASH_WARNING: u16 = 0x0062;
    pub const INTELLIGENT_RESOLUTION: u16 = 0x0070;
    pub const TOUCH_AE: u16 = 0x0077;
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::core::binary_data::BinaryDataTableBuilder;
    // use crate::core::types::ExifFormat;

    #[test]
    fn test_panasonic_parser_creation() {
        let parser = PanasonicMakerNoteParser;
        assert_eq!(parser.manufacturer(), "Panasonic");
    }

    #[test]
    fn test_empty_maker_note() {
        let parser = PanasonicMakerNoteParser;
        let result = parser.parse(&[], Endian::Little, 0).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_panasonic_signature_detection() {
        let parser = PanasonicMakerNoteParser;

        // Create test data with Panasonic signature
        let mut test_data = vec![0u8; 20];
        test_data[0..9].copy_from_slice(b"Panasonic");

        // Parser should handle this without error
        let _result = parser.parse(&test_data, Endian::Little, 0).unwrap();

        // Result may be empty due to invalid IFD after signature, but should not panic
        // Length check not needed since len() is always >= 0
    }

    #[test]
    fn test_panasonic_detection_pattern() {
        // Test the detection function directly using the correct Panasonic maker note format
        // Format: "Panasonic\0\0\0" (12 bytes total) per ExifTool lib/Image/ExifTool/MakerNotes.pm line 725-729
        let test_data = &[
            0x50, 0x61, 0x6e, 0x61, 0x73, 0x6f, 0x6e, 0x69, 0x63, 0x00, 0x00, 0x00, 0x01, 0x02,
            0x03,
        ];
        let detection = detect_panasonic_maker_note(test_data);

        assert!(detection.is_some());
        let detection = detection.unwrap();
        assert_eq!(detection.ifd_offset, 12); // IFD starts at offset 12 per ExifTool
        assert_eq!(detection.description, "Panasonic maker note");
    }
}
