//! Sigma maker note parser
//!
//! Sigma maker notes use a standard IFD structure similar to standard EXIF,
//! making them relatively straightforward to parse. The format is consistent
//! across most Sigma/Foveon cameras.

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Sigma.pm"]

use crate::core::ifd::{IfdParser, TiffHeader};
use crate::core::{Endian, ExifValue};
use crate::error::Result;
use crate::maker::MakerNoteParser;
use std::collections::HashMap;

/// Parser for Sigma maker notes
pub struct SigmaMakerNoteParser;

impl MakerNoteParser for SigmaMakerNoteParser {
    fn parse(
        &self,
        data: &[u8],
        byte_order: Endian,
        _base_offset: usize,
    ) -> Result<HashMap<u16, ExifValue>> {
        // Sigma maker notes start directly with an IFD (no header)
        // They use the same byte order as the main EXIF data

        if data.is_empty() {
            return Ok(HashMap::new());
        }

        // Sigma maker notes are straightforward - no footer handling needed
        // Parse as a standard IFD directly

        // Create a fake TIFF header for IFD parsing
        // (Sigma maker notes don't have a TIFF header, they start directly with IFD)
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
                eprintln!("Warning: Sigma maker note parsing failed: {}", e);
                Ok(HashMap::new())
            }
        }
    }

    fn manufacturer(&self) -> &'static str {
        "Sigma"
    }
}

/// Sigma-specific tag IDs
pub mod tags {
    // Main Sigma tags from ExifTool Sigma.pm
    pub const SERIAL_NUMBER: u16 = 0x0002;
    pub const DRIVE_MODE: u16 = 0x0003;
    pub const RESOLUTION_MODE: u16 = 0x0004;
    pub const AF_MODE: u16 = 0x0005;
    pub const FOCUS_SETTING: u16 = 0x0006;
    pub const WHITE_BALANCE: u16 = 0x0007;
    pub const EXPOSURE_MODE: u16 = 0x0008;
    pub const METERING_MODE: u16 = 0x0009;
    pub const LENS_FOCAL_RANGE: u16 = 0x000a;
    pub const COLOR_SPACE: u16 = 0x000b;
    pub const EXPOSURE_COMPENSATION: u16 = 0x000c;
    pub const CONTRAST: u16 = 0x000d;
    pub const SHADOW: u16 = 0x000e;
    pub const HIGHLIGHT: u16 = 0x000f;
    pub const SATURATION: u16 = 0x0010;
    pub const SHARPNESS: u16 = 0x0011;
    pub const X3_FILL_LIGHT: u16 = 0x0012;
    pub const COLOR_ADJUSTMENT: u16 = 0x0014;
    pub const ADJUSTMENT_MODE: u16 = 0x0015;
    pub const QUALITY: u16 = 0x0016;
    pub const FIRMWARE: u16 = 0x0017;
    pub const SOFTWARE: u16 = 0x0018;
    pub const AUTO_BRACKET: u16 = 0x0019;
    pub const PREVIEW_IMAGE_START: u16 = 0x001a;
    pub const PREVIEW_IMAGE_LENGTH: u16 = 0x001b;
    pub const PREVIEW_IMAGE_SIZE: u16 = 0x001c;
    pub const MAKER_NOTE_VERSION: u16 = 0x001d;
    pub const AF_POINT: u16 = 0x001f;
    pub const FILE_FORMAT: u16 = 0x0022;
    pub const CALIBRATION: u16 = 0x0024;
    pub const LENS_TYPE: u16 = 0x0027;
    pub const LENS_FOCAL_RANGE_2: u16 = 0x002a;
    pub const LENS_MAX_APERTURE_RANGE: u16 = 0x002b;
    pub const COLOR_MODE: u16 = 0x002c;
    pub const LENS_APERTURE_RANGE: u16 = 0x0030;
    pub const F_NUMBER: u16 = 0x0031;
    pub const EXPOSURE_TIME: u16 = 0x0032;
    pub const EXPOSURE_TIME_2: u16 = 0x0033;
    pub const BURST_SHOT: u16 = 0x0034;
    pub const EXPOSURE_COMPENSATION_2: u16 = 0x0035;
    pub const SENSOR_TEMPERATURE: u16 = 0x0039;
    pub const FLASH_EXPOSURE_COMP: u16 = 0x003a;
    pub const FIRMWARE_2: u16 = 0x003b;
    pub const WHITE_BALANCE_2: u16 = 0x003c;
    pub const PICTURE_MODE: u16 = 0x003d;
    pub const LENS_APERTURE_RANGE_2: u16 = 0x0048;
    pub const F_NUMBER_2: u16 = 0x0049;
    pub const EXPOSURE_TIME_3: u16 = 0x004a;
    pub const EXPOSURE_TIME_4: u16 = 0x004b;
    pub const EXPOSURE_COMPENSATION_3: u16 = 0x004d;
    pub const SENSOR_TEMPERATURE_2: u16 = 0x0055;
    pub const FLASH_EXPOSURE_COMP_2: u16 = 0x0056;
    pub const FIRMWARE_3: u16 = 0x0057;
    pub const WHITE_BALANCE_3: u16 = 0x0058;
    pub const DIGITAL_FILTER: u16 = 0x0059;
    pub const MODEL: u16 = 0x0084;
    pub const ISO: u16 = 0x0086;
    pub const RESOLUTION_MODE_2: u16 = 0x0087;
    pub const WHITE_BALANCE_4: u16 = 0x0088;
    pub const FIRMWARE_4: u16 = 0x008c;
    pub const CAMERA_CALIBRATION: u16 = 0x011f;
    pub const WB_SETTINGS: u16 = 0x0120;
    pub const WB_SETTINGS_2: u16 = 0x0121;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sigma_parser_creation() {
        let parser = SigmaMakerNoteParser;
        assert_eq!(parser.manufacturer(), "Sigma");
    }

    #[test]
    fn test_empty_maker_note() {
        let parser = SigmaMakerNoteParser;
        let result = parser.parse(&[], Endian::Little, 0).unwrap();
        assert!(result.is_empty());
    }
}
