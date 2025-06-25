//! Ricoh maker note parser using table-driven approach
//!
//! Ricoh maker notes use a proprietary format that starts with "RICOH"
//! signature followed by IFD-like structures. This format is consistent
//! across Ricoh cameras and contains camera-specific settings including
//! film simulation modes, X-Trans sensor parameters, and lens information.
//!
//! This implementation uses auto-generated tag tables and print conversion
//! functions, eliminating the need to manually port ExifTool's Perl code.

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Ricoh.pm"]

use crate::core::ifd::{IfdParser, TiffHeader};
use crate::core::print_conv::apply_print_conv;
use crate::core::{Endian, ExifValue};
use crate::error::Result;
use crate::maker::ricoh::detection::{detect_ricoh_maker_note, RICOHDetectionResult};

pub mod detection;
use crate::maker::MakerNoteParser;
use crate::tables::ricoh_tags::get_ricoh_tag;
use std::collections::HashMap;

/// Parser for Ricoh maker notes
pub struct RicohMakerNoteParser;

impl MakerNoteParser for RicohMakerNoteParser {
    fn parse(
        &self,
        data: &[u8],
        byte_order: Endian,
        _base_offset: usize,
    ) -> Result<HashMap<u16, ExifValue>> {
        if data.is_empty() {
            return Ok(HashMap::new());
        }

        // Use generated detection logic to identify Ricoh maker note format
        let detection = match detect_ricoh_maker_note(data) {
            Some(detection) => detection,
            None => {
                // Fallback: assume standard IFD at start of data
                RICOHDetectionResult {
                    version: None,
                    ifd_offset: 0,
                    description: "Fallback Ricoh parser".to_string(),
                }
            }
        };

        // Ricoh maker notes start with "RICOH" signature (8 bytes)
        // The IFD starts immediately after the signature
        let ifd_offset = if data.len() >= 8 && data.starts_with(b"RICOH") {
            8 // Skip "RICOH" signature
        } else {
            detection.ifd_offset
        };

        // Extract raw IFD data starting from detected offset
        let ifd_data = &data[ifd_offset..];
        if ifd_data.is_empty() {
            return Ok(HashMap::new());
        }

        // Parse using table-driven approach
        parse_ricoh_ifd_with_tables(ifd_data, byte_order)
    }

    fn manufacturer(&self) -> &'static str {
        "Ricoh"
    }
}

/// Parse Ricoh IFD using generated tag tables and print conversion
fn parse_ricoh_ifd_with_tables(data: &[u8], byte_order: Endian) -> Result<HashMap<u16, ExifValue>> {
    // Create a fake TIFF header for IFD parsing
    // (Ricoh maker notes don't have a TIFF header, they start directly with IFD)
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
            eprintln!("Warning: Ricoh IFD parsing failed: {}", e);
            return Ok(HashMap::new());
        }
    };

    // Convert raw IFD entries to Ricoh tags with print conversion
    let mut result = HashMap::new();

    for (tag_id, raw_value) in parsed_ifd.entries() {
        if let Some(ricoh_tag) = get_ricoh_tag(*tag_id) {
            // Apply print conversion to create human-readable value
            let converted_value = apply_print_conv(raw_value, ricoh_tag.print_conv);

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

/// Ricoh-specific tag IDs
pub mod tags {
    // Main Ricoh tags from ExifTool
    pub const RICOH_VERSION: u16 = 0x0000;
    pub const INTERNAL_SERIAL_NUMBER: u16 = 0x0010;
    pub const QUALITY: u16 = 0x1000;
    pub const SHARPNESS: u16 = 0x1001;
    pub const WHITE_BALANCE: u16 = 0x1002;
    pub const SATURATION: u16 = 0x1003;
    pub const CONTRAST: u16 = 0x1004;
    pub const COLOR_TEMPERATURE: u16 = 0x1005;
    pub const WHITE_BALANCE_FINE_TUNE: u16 = 0x100a;
    pub const NOISE_REDUCTION: u16 = 0x100b;
    pub const CLARITY: u16 = 0x100f;
    pub const FUJI_FLASH_MODE: u16 = 0x1010;
    pub const FLASH_EXPOSURE_COMP: u16 = 0x1011;
    pub const MACRO: u16 = 0x1020;
    pub const FOCUS_MODE: u16 = 0x1021;
    pub const AF_MODE: u16 = 0x1022;
    pub const FOCUS_PIXEL: u16 = 0x1023;
    pub const PRIORITY_SETTINGS: u16 = 0x102b;
    pub const FOCUS_SETTINGS: u16 = 0x102d;
    pub const AFC_SETTINGS: u16 = 0x102e;
    pub const SLOW_SYNC: u16 = 0x1030;
    pub const PICTURE_MODE: u16 = 0x1031;
    pub const EXPOSURE_COUNT: u16 = 0x1032;
    pub const EXR_AUTO: u16 = 0x1033;
    pub const EXR_MODE: u16 = 0x1034;
    pub const SHADOW_TONE: u16 = 0x1040;
    pub const HIGHLIGHT_TONE: u16 = 0x1041;
    pub const DIGITAL_ZOOM: u16 = 0x1044;
    pub const LENS_MODULATION_OPTIMIZER: u16 = 0x1045;
    pub const GRAIN_EFFECT_ROUGHNESS: u16 = 0x1047;
    pub const COLOR_CHROME_EFFECT: u16 = 0x1048;
    pub const BW_ADJUSTMENT: u16 = 0x1049;
    pub const BW_MAGENTA_GREEN: u16 = 0x104b;
    pub const CROP_MODE: u16 = 0x104d;
    pub const COLOR_CHROME_FX_BLUE: u16 = 0x104e;
    pub const SHUTTER_TYPE: u16 = 0x1050;
    pub const CROP_FLAG: u16 = 0x1051;
    pub const CROP_TOP_LEFT: u16 = 0x1052;
    pub const CROP_SIZE: u16 = 0x1053;
    pub const SEQUENCE_NUMBER: u16 = 0x1101;
    pub const DRIVE_SETTINGS: u16 = 0x1103;
    pub const PIXEL_SHIFT_SHOTS: u16 = 0x1105;
    pub const PIXEL_SHIFT_OFFSET: u16 = 0x1106;
    pub const PANORAMA_ANGLE: u16 = 0x1153;
    pub const PANORAMA_DIRECTION: u16 = 0x1154;
    pub const ADVANCED_FILTER: u16 = 0x1201;
    pub const COLOR_MODE: u16 = 0x1210;
    pub const BLUR_WARNING: u16 = 0x1300;
    pub const FOCUS_WARNING: u16 = 0x1301;
    pub const EXPOSURE_WARNING: u16 = 0x1302;
    pub const GE_IMAGE_SIZE: u16 = 0x1304;
    pub const DYNAMIC_RANGE: u16 = 0x1400;
    pub const FILM_MODE: u16 = 0x1401;
    pub const DYNAMIC_RANGE_SETTING: u16 = 0x1402;
    pub const DEVELOPMENT_DYNAMIC_RANGE: u16 = 0x1403;
    pub const MIN_FOCAL_LENGTH: u16 = 0x1404;
    pub const MAX_FOCAL_LENGTH: u16 = 0x1405;
    pub const MAX_APERTURE_AT_MIN_FOCAL: u16 = 0x1406;
    pub const MAX_APERTURE_AT_MAX_FOCAL: u16 = 0x1407;
    pub const AUTO_DYNAMIC_RANGE: u16 = 0x140b;
    pub const IMAGE_STABILIZATION: u16 = 0x1422;
    pub const SCENE_RECOGNITION: u16 = 0x1425;
    pub const RATING: u16 = 0x1431;
    pub const IMAGE_GENERATION: u16 = 0x1436;
    pub const IMAGE_COUNT: u16 = 0x1438;
    pub const DRANGE_PRIORITY: u16 = 0x1443;
    pub const DRANGE_PRIORITY_AUTO: u16 = 0x1444;
    pub const DRANGE_PRIORITY_FIXED: u16 = 0x1445;
    pub const FLICKER_REDUCTION: u16 = 0x1446;
    pub const FUJI_MODEL: u16 = 0x1447;
    pub const FUJI_MODEL2: u16 = 0x1448;
    pub const ROLL_ANGLE: u16 = 0x144d;
    pub const VIDEO_RECORDING_MODE: u16 = 0x3803;
    pub const PERIPHERAL_LIGHTING: u16 = 0x3804;
    pub const VIDEO_COMPRESSION: u16 = 0x3806;
    pub const FRAME_RATE: u16 = 0x3820;
    pub const FRAME_WIDTH: u16 = 0x3821;
    pub const FRAME_HEIGHT: u16 = 0x3822;
    pub const FULL_HD_HIGH_SPEED_REC: u16 = 0x3824;
    pub const FACE_ELEMENT_SELECTED: u16 = 0x4005;
    pub const FACES_DETECTED: u16 = 0x4100;
    pub const FACE_POSITIONS: u16 = 0x4103;
    pub const NUM_FACE_ELEMENTS: u16 = 0x4200;
    pub const FACE_ELEMENT_TYPES: u16 = 0x4201;
    pub const FACE_ELEMENT_POSITIONS: u16 = 0x4203;
    pub const FACE_REC_INFO: u16 = 0x4282;
    pub const FILE_SOURCE: u16 = 0x8000;
    pub const ORDER_NUMBER: u16 = 0x8002;
    pub const FRAME_NUMBER: u16 = 0x8003;
    pub const PARALLAX: u16 = 0xb211;
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::core::binary_data::BinaryDataTableBuilder;
    // use crate::core::types::ExifFormat;

    #[test]
    fn test_ricoh_parser_creation() {
        let parser = RicohMakerNoteParser;
        assert_eq!(parser.manufacturer(), "Ricoh");
    }

    #[test]
    fn test_empty_maker_note() {
        let parser = RicohMakerNoteParser;
        let result = parser.parse(&[], Endian::Little, 0).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_ricoh_signature_detection() {
        let parser = RicohMakerNoteParser;

        // Create test data with RICOH signature
        let mut test_data = vec![0u8; 20];
        test_data[0..5].copy_from_slice(b"RICOH");

        // Parser should handle this without error
        let _result = parser.parse(&test_data, Endian::Little, 0).unwrap();

        // Result may be empty due to invalid IFD after signature, but should not panic
        // Length check not needed since len() is always >= 0
    }

    #[test]
    fn test_ricoh_detection_pattern() {
        // Test the detection function directly
        let test_data = b"RICOH_test_data";
        let detection = detect_ricoh_maker_note(test_data);

        assert!(detection.is_some());
        let detection = detection.unwrap();
        assert_eq!(detection.ifd_offset, 0);
        assert_eq!(detection.description, "ricoh maker note signature");
    }
}
