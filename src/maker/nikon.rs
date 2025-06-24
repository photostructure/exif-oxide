//! Nikon maker note parser
//!
//! Nikon maker notes use an IFD structure similar to standard EXIF,
//! but with Nikon-specific tags and some encrypted sections.
//! This parser also supports ProcessBinaryData for complex binary structures.

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Nikon.pm"]

use crate::core::ifd::{IfdParser, TiffHeader};
use crate::core::print_conv::apply_print_conv;
use crate::core::{Endian, ExifValue};
use crate::error::Result;
use crate::maker::MakerNoteParser;
use crate::tables::nikon_tags::get_nikon_tag;
use std::collections::HashMap;

pub mod binary;
pub mod detection;

/// Parser for Nikon maker notes
pub struct NikonMakerNoteParser;

impl MakerNoteParser for NikonMakerNoteParser {
    fn parse(
        &self,
        data: &[u8],
        byte_order: Endian,
        _base_offset: usize,
    ) -> Result<HashMap<u16, ExifValue>> {
        // Use generated detection logic to identify Nikon maker note format
        if data.is_empty() {
            return Ok(HashMap::new());
        }

        // Detect the specific Nikon maker note format and version
        let (parsing_data, _detected_offset) =
            if let Some(detection_result) = detection::detect_nikon_maker_note(data) {
                // Use detected IFD offset for parsing
                let offset = detection_result.ifd_offset;
                if offset < data.len() {
                    (&data[offset..], offset)
                } else {
                    (data, 0)
                }
            } else {
                // Fallback: assume standard IFD structure starting at beginning
                (data, 0)
            };

        // Create a fake TIFF header for IFD parsing
        // (Nikon maker notes don't have a TIFF header, they start directly with IFD)
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

        let mut results = HashMap::new();

        match IfdParser::parse_ifd(&tiff_data, &header, 8) {
            Ok(parsed) => {
                let entries = parsed.entries();

                // First, apply table-driven PrintConv to all IFD entries (following Pentax pattern)
                for (tag_id, raw_value) in entries {
                    if let Some(nikon_tag) = get_nikon_tag(*tag_id) {
                        // Apply print conversion to create human-readable value
                        let converted_value = apply_print_conv(raw_value, nikon_tag.print_conv);

                        // Store both raw and converted values
                        // Raw value for programmatic access
                        results.insert(*tag_id, raw_value.clone());

                        // Converted value as string (following ExifTool pattern)
                        // Use a high bit pattern to distinguish converted values
                        let converted_tag_id = 0x8000 | tag_id;
                        results.insert(converted_tag_id, ExifValue::Ascii(converted_value));
                    } else {
                        // Keep unknown tags as-is
                        results.insert(*tag_id, raw_value.clone());
                    }
                }

                // Process binary data tags that require ProcessBinaryData
                for (tag, value) in entries {
                    if let Some(binary_data) = self.extract_binary_data_for_tag(*tag, value) {
                        // Extract version and model for conditional processing
                        let version = self.extract_version_string(entries);
                        let model = self.extract_model_string(entries);

                        match binary::NikonBinaryDataProcessor::process_binary_data(
                            *tag,
                            &binary_data,
                            byte_order,
                            version.as_deref(),
                            model.as_deref(),
                        ) {
                            Ok(binary_results) => {
                                // Merge binary data results into main results
                                for (binary_tag, binary_value) in binary_results {
                                    results.insert(binary_tag, binary_value);
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "Warning: Binary data processing failed for tag 0x{:04x}: {}",
                                    tag, e
                                );
                            }
                        }
                    }
                }

                Ok(results)
            }
            Err(e) => {
                // Log the error but return empty results
                // Nikon maker notes may have encrypted sections that cause parsing errors
                eprintln!("Warning: Nikon maker note parsing failed: {}", e);
                Ok(HashMap::new())
            }
        }
    }

    fn manufacturer(&self) -> &'static str {
        "Nikon"
    }
}

impl NikonMakerNoteParser {
    /// Extract binary data from specific tags that use ProcessBinaryData
    fn extract_binary_data_for_tag(&self, tag: u16, value: &ExifValue) -> Option<Vec<u8>> {
        match tag {
            // Known Nikon binary data tags
            0x001f => {
                // VRInfo (Vibration Reduction Info)
                if let ExifValue::Undefined(data) = value {
                    Some(data.clone())
                } else {
                    None
                }
            }
            0x0010 => {
                // ShotInfo (various camera models)
                if let ExifValue::Undefined(data) = value {
                    Some(data.clone())
                } else {
                    None
                }
            }
            0x0097 => {
                // ColorBalance
                if let ExifValue::Undefined(data) = value {
                    Some(data.clone())
                } else {
                    None
                }
            }
            0x0098 => {
                // LensData
                if let ExifValue::Undefined(data) = value {
                    Some(data.clone())
                } else {
                    None
                }
            }
            0x00a8 => {
                // FlashInfo
                if let ExifValue::Undefined(data) = value {
                    Some(data.clone())
                } else {
                    None
                }
            }
            _ => None, // Not a binary data tag
        }
    }

    /// Extract version string from maker notes (if available)
    fn extract_version_string(&self, entries: &HashMap<u16, ExifValue>) -> Option<String> {
        // Check common version tags
        for version_tag in [0x0001, 0x0002, 0x0003] {
            if let Some(ExifValue::Ascii(version)) = entries.get(&version_tag) {
                return Some(version.clone());
            }
            if let Some(ExifValue::Undefined(data)) = entries.get(&version_tag) {
                if data.len() >= 4 {
                    if let Ok(version) = std::str::from_utf8(&data[..4]) {
                        return Some(version.to_string());
                    }
                }
            }
        }
        None
    }

    /// Extract model string for conditional processing
    fn extract_model_string(&self, _entries: &HashMap<u16, ExifValue>) -> Option<String> {
        // In real implementation, this would come from main EXIF data
        // For now, we'll try to infer from maker notes or return None

        // Check if there's any model-specific tag that might give us a hint
        // In practice, this would be passed from the main IFD parser
        None // Simplified for now
    }
}

/// Nikon-specific tag IDs
pub mod tags {
    // Main Nikon tags from the Main table
    pub const MAKER_NOTE_VERSION: u16 = 0x0001;
    pub const ISO: u16 = 0x0002;
    pub const COLOR_MODE: u16 = 0x0003;
    pub const QUALITY: u16 = 0x0004;
    pub const WHITE_BALANCE: u16 = 0x0005;
    pub const SHARPNESS: u16 = 0x0006;
    pub const FOCUS_MODE: u16 = 0x0007;
    pub const FLASH_SETTING: u16 = 0x0008;
    pub const FLASH_TYPE: u16 = 0x0009;
    pub const WHITE_BALANCE_FINE_TUNE: u16 = 0x000b;
    pub const WB_RB_LEVELS: u16 = 0x000c;
    pub const PROGRAM_SHIFT: u16 = 0x000d;
    pub const EXPOSURE_DIFFERENCE: u16 = 0x000e;
    pub const ISO_SELECTION: u16 = 0x000f;
    pub const DATA_DUMP: u16 = 0x0010;
    pub const PREVIEW_IFD: u16 = 0x0011;
    pub const AUTO_FLASH_COMPENSATION: u16 = 0x0012;
    pub const AUTO_EXPOSURE_BRACKET_VALUE: u16 = 0x0013;
    pub const EXPOSURE_BRACKET_VALUE: u16 = 0x0018;
    pub const IMAGE_BOUNDARY: u16 = 0x001a;
    pub const FLASH_EXPOSURE_COMPENSATION: u16 = 0x001b;
    pub const FLASH_BRACKET_COMPENSATION: u16 = 0x001c;
    pub const AE_BRACKET_COMPENSATION: u16 = 0x001d;
    pub const AUTO_FOCUS_POSITION: u16 = 0x001e;
    pub const DIGITAL_ZOOM: u16 = 0x001f;
    pub const LENS_TYPE: u16 = 0x0020;
    pub const CAMERA_TONE_COMPENSATION: u16 = 0x0081;
    pub const CAMERA_COLOR_MODE: u16 = 0x0082;
    pub const CAMERA_HUE_ADJUSTMENT: u16 = 0x0085;
    pub const NEF_COMPRESSION: u16 = 0x0093;
    pub const SATURATION: u16 = 0x0094;
    pub const NOISE_REDUCTION: u16 = 0x0095;
    pub const LINEAR_IZATION_TABLE: u16 = 0x0096;
    pub const COLOR_BALANCE_A: u16 = 0x0097;
    pub const LENS_DATA: u16 = 0x0098;
    pub const RAW_IMAGE_CENTER: u16 = 0x0099;
    pub const SENSOR_PIXEL_SIZE: u16 = 0x009a;
    pub const SCENE_ASSIST: u16 = 0x009c;
    pub const DATE_STAMP_MODE: u16 = 0x009d;
    pub const RETAIL_INFO: u16 = 0x009e;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nikon_parser_creation() {
        let parser = NikonMakerNoteParser;
        assert_eq!(parser.manufacturer(), "Nikon");
    }

    #[test]
    fn test_empty_maker_note() {
        let parser = NikonMakerNoteParser;
        let result = parser.parse(&[], Endian::Little, 0).unwrap();
        assert!(result.is_empty());
    }
}
