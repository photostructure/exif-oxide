//! Apple maker note parser using table-driven approach
//!
//! Apple maker notes use a standard IFD structure similar to standard EXIF,
//! but with Apple-specific tags. Apple devices (iPhone, iPad) use relatively
//! simple maker note structures compared to traditional camera manufacturers.
//!
//! This implementation uses auto-generated tag tables and print conversion
//! functions, following the established table-driven pattern.

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Apple.pm"]

use crate::core::ifd::{IfdParser, TiffHeader};
use crate::core::print_conv::apply_print_conv;
use crate::core::{Endian, ExifValue};
use crate::error::Result;
use crate::maker::apple::detection::{detect_apple_maker_note, APPLEDetectionResult};

pub mod detection;
use crate::maker::MakerNoteParser;
use crate::tables::apple_tags::get_apple_tag;
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
        if data.is_empty() {
            return Ok(HashMap::new());
        }

        // Use generated detection logic to identify Apple maker note format
        let detection = match detect_apple_maker_note(data) {
            Some(detection) => detection,
            None => {
                // Fallback: assume standard IFD at start of data
                APPLEDetectionResult {
                    version: None,
                    ifd_offset: 0,
                    description: "Fallback Apple parser".to_string(),
                }
            }
        };

        // Apple maker notes typically start directly with IFD (no special header)
        let ifd_offset = detection.ifd_offset;

        // Extract raw IFD data starting from detected offset
        let ifd_data = &data[ifd_offset..];

        // Parse raw IFD data first
        let raw_entries = parse_apple_raw_ifd(ifd_data, byte_order)?;
        let mut result = HashMap::new();

        // Apply table-driven conversion using Apple tag table
        for (tag_id, raw_value) in raw_entries {
            // Store raw value
            result.insert(tag_id, raw_value.clone());

            // Apply PrintConv if tag is known
            if let Some(tag_def) = get_apple_tag(tag_id) {
                let converted = apply_print_conv(&raw_value, tag_def.print_conv);
                // Store converted value with high bit set to distinguish from raw
                let converted_tag_id = 0x8000 | tag_id;
                result.insert(converted_tag_id, ExifValue::Ascii(converted));
            }
        }

        Ok(result)
    }

    fn manufacturer(&self) -> &'static str {
        "Apple"
    }
}

/// Parse Apple raw IFD data  
fn parse_apple_raw_ifd(data: &[u8], byte_order: Endian) -> Result<HashMap<u16, ExifValue>> {
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
    fn test_apple_tag_lookup() {
        use crate::tables::apple_tags::get_apple_tag;

        // Verify some key tags from the generated table
        assert!(get_apple_tag(0x0001).is_some()); // MakerNoteVersion
        assert!(get_apple_tag(0x0004).is_some()); // AEStable
        assert!(get_apple_tag(0x000a).is_some()); // HDRImageType
        assert!(get_apple_tag(0x0014).is_some()); // ImageCaptureType
        assert!(get_apple_tag(0x002e).is_some()); // CameraType

        // Verify tag names
        let ae_stable_tag = get_apple_tag(0x0004).unwrap();
        assert_eq!(ae_stable_tag.name, "AEStable");

        let camera_type_tag = get_apple_tag(0x002e).unwrap();
        assert_eq!(camera_type_tag.name, "CameraType");
    }

    #[test]
    fn test_apple_tag_count() {
        use crate::tables::apple_tags::APPLE_TAGS;
        // Verify we have the expected number of tags
        assert_eq!(APPLE_TAGS.len(), 42);
    }

    #[test]
    fn test_apple_printconv_functions() {
        use crate::core::print_conv::apply_print_conv;
        use crate::core::print_conv::PrintConvId;

        // Test HDRImageType conversion
        assert_eq!(
            apply_print_conv(&ExifValue::U32(3), PrintConvId::AppleHDRImageType),
            "HDR Image"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(4), PrintConvId::AppleHDRImageType),
            "Original Image"
        );

        // Test ImageCaptureType conversion
        assert_eq!(
            apply_print_conv(&ExifValue::U32(1), PrintConvId::AppleImageCaptureType),
            "ProRAW"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(10), PrintConvId::AppleImageCaptureType),
            "Photo"
        );

        // Test CameraType conversion
        assert_eq!(
            apply_print_conv(&ExifValue::U32(0), PrintConvId::AppleCameraType),
            "Back Wide Angle"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(6), PrintConvId::AppleCameraType),
            "Front"
        );
    }
}
