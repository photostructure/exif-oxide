//! Fujifilm maker note parser
//!
//! Fujifilm maker notes use a standard IFD structure similar to standard EXIF,
//! making them straightforward to parse. The format is consistent across all
//! Fujifilm models.
//!
//! Key characteristics:
//! - Standard IFD structure (no special header like Olympus)
//! - Uses the same byte order as the main EXIF data
//! - No encryption or obfuscation
//! - Supports ProcessBinaryData for advanced settings

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/FujiFilm.pm"]

use crate::core::ifd::{IfdParser, TiffHeader};
use crate::core::{Endian, ExifValue};
use crate::error::Result;
use crate::maker::MakerNoteParser;
use std::collections::HashMap;

// Include the binary data module
pub mod fujifilm_binary;
use fujifilm_binary::{create_fujifilm_binary_tables, get_fujifilm_binary_tag_mapping};

/// Parser for Fujifilm maker notes
pub struct FujifilmMakerNoteParser;

impl MakerNoteParser for FujifilmMakerNoteParser {
    fn parse(
        &self,
        data: &[u8],
        byte_order: Endian,
        _base_offset: usize,
    ) -> Result<HashMap<u16, ExifValue>> {
        // Fujifilm maker notes start directly with an IFD (no header)
        // They use the same byte order as the main EXIF data

        if data.is_empty() {
            return Ok(HashMap::new());
        }

        // First parse standard IFD data
        let mut results = self.parse_standard_ifd(data, byte_order)?;

        // Then check for ProcessBinaryData tags and parse them
        let binary_results = self.parse_binary_data_tags(&results, byte_order)?;

        // Merge binary data results into main results
        for (tag_id, value) in binary_results {
            results.insert(tag_id, value);
        }

        Ok(results)
    }

    fn manufacturer(&self) -> &'static str {
        "Fujifilm"
    }
}

impl FujifilmMakerNoteParser {
    /// Parse standard IFD data (original implementation)
    fn parse_standard_ifd(
        &self,
        data: &[u8],
        byte_order: Endian,
    ) -> Result<HashMap<u16, ExifValue>> {
        // Fujifilm maker notes are similar to Pentax - no header, just IFD data
        // Parse as a standard IFD directly

        // Create a fake TIFF header for IFD parsing
        // (Fujifilm maker notes don't have a TIFF header, they start directly with IFD)
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
                eprintln!("Warning: Fujifilm maker note parsing failed: {}", e);
                Ok(HashMap::new())
            }
        }
    }

    /// Parse ProcessBinaryData tags
    fn parse_binary_data_tags(
        &self,
        ifd_results: &HashMap<u16, ExifValue>,
        byte_order: Endian,
    ) -> Result<HashMap<u16, ExifValue>> {
        let mut binary_results = HashMap::new();

        // Create binary data tables
        let binary_tables = create_fujifilm_binary_tables();
        let tag_mapping = get_fujifilm_binary_tag_mapping();

        // Check each tag in the IFD results to see if it has binary data to process
        for (&tag_id, value) in ifd_results {
            if let Some(&table_name) = tag_mapping.get(&tag_id) {
                if let Some(table) = binary_tables.get(table_name) {
                    // Extract binary data from the tag value
                    if let Some(binary_data) = self.extract_binary_data(value) {
                        // Parse the binary data using the appropriate table
                        match table.parse(&binary_data, byte_order) {
                            Ok(parsed_data) => {
                                // Add parsed data with offset to avoid conflicts
                                for (binary_tag_id, binary_value) in parsed_data {
                                    // Combine original tag ID with binary field offset
                                    let combined_tag_id = tag_id + binary_tag_id - 0x8000;
                                    binary_results.insert(combined_tag_id, binary_value);
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "Warning: Failed to parse binary data for tag 0x{:04x} ({}): {}",
                                    tag_id, table_name, e
                                );
                            }
                        }
                    }
                }
            }
        }

        Ok(binary_results)
    }

    /// Extract binary data from an ExifValue
    fn extract_binary_data(&self, value: &ExifValue) -> Option<Vec<u8>> {
        match value {
            ExifValue::Undefined(data) => Some(data.clone()),
            ExifValue::U8Array(data) => Some(data.clone()),
            ExifValue::U16Array(data) => {
                // Convert u16 array to u8 array (little-endian)
                let mut bytes = Vec::new();
                for &val in data {
                    bytes.extend_from_slice(&val.to_le_bytes());
                }
                Some(bytes)
            }
            ExifValue::U32Array(data) => {
                // Convert u32 array to u8 array (little-endian)
                let mut bytes = Vec::new();
                for &val in data {
                    bytes.extend_from_slice(&val.to_le_bytes());
                }
                Some(bytes)
            }
            // For single values, convert to byte array
            ExifValue::U8(val) => Some(vec![*val]),
            ExifValue::U16(val) => Some(val.to_le_bytes().to_vec()),
            ExifValue::U32(val) => Some(val.to_le_bytes().to_vec()),
            _ => {
                // For other types (ASCII, etc.), we don't process as binary data
                None
            }
        }
    }
}

/// Fujifilm-specific tag IDs (from ExifTool)
pub mod tags {
    // Main Fujifilm tags from ExifTool
    pub const VERSION: u16 = 0x0000;
    pub const INTERNAL_SERIAL_NUMBER: u16 = 0x0010;
    pub const QUALITY: u16 = 0x1000;
    pub const SHARPNESS: u16 = 0x1001;
    pub const WHITE_BALANCE: u16 = 0x1002;
    pub const SATURATION: u16 = 0x1003;
    pub const CONTRAST: u16 = 0x1004;
    pub const COLOR_TEMPERATURE: u16 = 0x1005;
    pub const CONTRAST_HIGHLIGHT_SHADOW_ADJUST: u16 = 0x1006;
    pub const NOISE_REDUCTION: u16 = 0x100b;
    pub const FUJI_FLASH_MODE: u16 = 0x1010;
    pub const FLASH_EXPOSURE_COMP: u16 = 0x1011;
    pub const MACRO: u16 = 0x1020;
    pub const FOCUS_MODE: u16 = 0x1021;
    pub const AF_MODE: u16 = 0x1022;
    pub const FOCUS_PIXEL: u16 = 0x1023;
    pub const SLOW_SYNC: u16 = 0x1030;
    pub const PICTURE_MODE: u16 = 0x1031;
    pub const EXR_AUTO: u16 = 0x1033;
    pub const EXR_MODE: u16 = 0x1034;
    pub const SHUTTER_TYPE: u16 = 0x1050;
    pub const CONTINUOUS_MODE: u16 = 0x1100;
    pub const SEQUENCE_NUMBER: u16 = 0x1101;
    pub const BLUR_WARNING: u16 = 0x1300;
    pub const FOCUS_WARNING: u16 = 0x1301;
    pub const EXPOSURE_WARNING: u16 = 0x1302;
    pub const DYNAMIC_RANGE: u16 = 0x1400;
    pub const FILM_MODE: u16 = 0x1401;
    pub const DYNAMIC_RANGE_SETTING: u16 = 0x1402;
    pub const DEVELOPMENT_DYNAMIC_RANGE: u16 = 0x1403;
    pub const MIN_FOCAL_LENGTH: u16 = 0x1404;
    pub const MAX_FOCAL_LENGTH: u16 = 0x1405;
    pub const MAX_APERTURE_AT_MIN_FOCAL: u16 = 0x1406;
    pub const MAX_APERTURE_AT_MAX_FOCAL: u16 = 0x1407;
    pub const FACE_POSITIONS: u16 = 0x4100;
    pub const NUM_DETECTED_FACES: u16 = 0x4103;
    pub const FACE_ELEMENT_TYPES: u16 = 0x4201;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::ExifFormat;

    #[test]
    fn test_fujifilm_parser_creation() {
        let parser = FujifilmMakerNoteParser;
        assert_eq!(parser.manufacturer(), "Fujifilm");
    }

    #[test]
    fn test_empty_maker_note() {
        let parser = FujifilmMakerNoteParser;
        let result = parser.parse(&[], Endian::Little, 0).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_binary_data_extraction() {
        let parser = FujifilmMakerNoteParser;

        // Test with undefined data
        let undefined_data = ExifValue::Undefined(vec![0x12, 0x34, 0x56, 0x78]);
        let result = parser.extract_binary_data(&undefined_data);
        assert_eq!(result, Some(vec![0x12, 0x34, 0x56, 0x78]));

        // Test with U8 array
        let u8_array = ExifValue::U8Array(vec![0x12, 0x34]);
        let result = parser.extract_binary_data(&u8_array);
        assert_eq!(result, Some(vec![0x12, 0x34]));

        // Test with U16 value
        let u16_val = ExifValue::U16(0x1234);
        let result = parser.extract_binary_data(&u16_val);
        assert_eq!(result, Some(vec![0x34, 0x12])); // Little-endian

        // Test with ASCII (should return None)
        let ascii_val = ExifValue::Ascii("test".to_string());
        let result = parser.extract_binary_data(&ascii_val);
        assert_eq!(result, None);
    }

    #[test]
    fn test_fujifilm_binary_tables() {
        let tables = create_fujifilm_binary_tables();

        // Should have all expected tables
        assert!(tables.contains_key("PrioritySettings"));
        assert!(tables.contains_key("FocusSettings"));
        assert!(tables.contains_key("AFCSettings"));
        assert!(tables.contains_key("DriveSettings"));

        // Test a specific table
        let focus_table = tables.get("FocusSettings").unwrap();
        assert_eq!(focus_table.name, "FocusSettings");
        assert_eq!(focus_table.default_format, ExifFormat::U32);
    }

    #[test]
    fn test_fujifilm_tag_mapping() {
        let mapping = get_fujifilm_binary_tag_mapping();

        // Check specific mappings
        assert_eq!(mapping.get(&0x102b), Some(&"PrioritySettings"));
        assert_eq!(mapping.get(&0x102d), Some(&"FocusSettings"));
        assert_eq!(mapping.get(&0x102e), Some(&"AFCSettings"));
        assert_eq!(mapping.get(&0x1103), Some(&"DriveSettings"));
    }
}
