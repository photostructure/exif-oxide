//! Apple maker note parser
//!
//! Apple maker notes use a standard IFD structure similar to standard EXIF,
//! but with Apple-specific tags. Apple devices (iPhone, iPad) use relatively
//! simple maker note structures compared to traditional camera manufacturers.

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Apple.pm"]

use crate::core::ifd::{IfdParser, TiffHeader};
use crate::core::{Endian, ExifValue};
use crate::error::Result;
use crate::maker::MakerNoteParser;
use std::collections::HashMap;

/// Parser for Apple maker notes
pub struct AppleMakerNoteParser;

impl MakerNoteParser for AppleMakerNoteParser {
    fn parse(
        &self,
        data: &[u8],
        byte_order: Endian,
        _base_offset: usize,
    ) -> Result<HashMap<u16, ExifValue>> {
        // Apple maker notes start directly with an IFD (no header)
        // They use the same byte order as the main EXIF data

        if data.is_empty() {
            return Ok(HashMap::new());
        }

        // Apple maker notes are similar to Pentax - straightforward IFD structure
        // No special handling needed for footers or headers

        // Create a fake TIFF header for IFD parsing
        // (Apple maker notes don't have a TIFF header, they start directly with IFD)
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
                eprintln!("Warning: Apple maker note parsing failed: {}", e);
                Ok(HashMap::new())
            }
        }
    }

    fn manufacturer(&self) -> &'static str {
        "Apple"
    }
}

/// Apple-specific tag IDs (from Apple.pm)
pub mod tags {
    // Main Apple tags from ExifTool
    pub const MAKER_NOTE_VERSION: u16 = 0x0001;
    pub const AE_MATRIX: u16 = 0x0002;
    pub const RUN_TIME: u16 = 0x0003;
    pub const AE_STABLE: u16 = 0x0004;
    pub const AE_TARGET: u16 = 0x0005;
    pub const AE_AVERAGE: u16 = 0x0006;
    pub const AF_STABLE: u16 = 0x0007;
    pub const ACCELERATION_VECTOR: u16 = 0x0008;
    pub const HDR_IMAGE_TYPE: u16 = 0x000a;
    pub const BURST_UUID: u16 = 0x000b;
    pub const FOCUS_DISTANCE_RANGE: u16 = 0x000c;
    pub const OIS_MODE: u16 = 0x000f;
    pub const CONTENT_IDENTIFIER: u16 = 0x0011;
    pub const IMAGE_CAPTURE_TYPE: u16 = 0x0014;
    pub const IMAGE_UNIQUE_ID: u16 = 0x0015;
    pub const LIVE_PHOTO_VIDEO_INDEX: u16 = 0x0017;
    pub const IMAGE_PROCESSING_FLAGS: u16 = 0x0019;
    pub const QUALITY_HINT: u16 = 0x001a;
    pub const LUMINANCE_NOISE_AMPLITUDE: u16 = 0x001d;
    pub const PHOTOS_APP_FEATURE_FLAGS: u16 = 0x001f;
    pub const IMAGE_CAPTURE_REQUEST_ID: u16 = 0x0020;
    pub const HDR_HEADROOM: u16 = 0x0021;
    pub const AF_PERFORMANCE: u16 = 0x0023;
    pub const SCENE_FLAGS: u16 = 0x0025;
    pub const SIGNAL_TO_NOISE_RATIO_TYPE: u16 = 0x0026;
    pub const SIGNAL_TO_NOISE_RATIO: u16 = 0x0027;
    pub const PHOTO_IDENTIFIER: u16 = 0x002b;
    pub const COLOR_TEMPERATURE: u16 = 0x002d;
    pub const CAMERA_TYPE: u16 = 0x002e;
    pub const FOCUS_POSITION: u16 = 0x002f;
    pub const HDR_GAIN: u16 = 0x0030;
    pub const AF_MEASURED_DEPTH: u16 = 0x0038;
    pub const AF_CONFIDENCE: u16 = 0x003d;
    pub const COLOR_CORRECTION_MATRIX: u16 = 0x003e;
    pub const GREEN_GHOST_MITIGATION_STATUS: u16 = 0x003f;
    pub const SEMANTIC_STYLE: u16 = 0x0040;
    pub const SEMANTIC_STYLE_RENDERING_VER: u16 = 0x0041;
    pub const SEMANTIC_STYLE_PRESET: u16 = 0x0042;

    // Unknown tags for research
    pub const APPLE_0X004E: u16 = 0x004e;
    pub const APPLE_0X004F: u16 = 0x004f;
    pub const APPLE_0X0054: u16 = 0x0054;
    pub const APPLE_0X005A: u16 = 0x005a;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apple_parser_creation() {
        let parser = AppleMakerNoteParser;
        assert_eq!(parser.manufacturer(), "Apple");
    }

    #[test]
    fn test_empty_maker_note() {
        let parser = AppleMakerNoteParser;
        let result = parser.parse(&[], Endian::Little, 0).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_apple_tag_constants() {
        // Verify some key tag constants
        assert_eq!(tags::MAKER_NOTE_VERSION, 0x0001);
        assert_eq!(tags::ACCELERATION_VECTOR, 0x0008);
        assert_eq!(tags::CONTENT_IDENTIFIER, 0x0011);
        assert_eq!(tags::IMAGE_CAPTURE_TYPE, 0x0014);
        assert_eq!(tags::CAMERA_TYPE, 0x002e);
    }
}
