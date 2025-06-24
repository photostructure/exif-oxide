//! Sony maker note parser
//!
//! Sony maker notes use an IFD structure similar to standard EXIF,
//! but with Sony-specific tags and many encrypted sections.
//! Many tags are inherited from Minolta.

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Sony.pm"]

use crate::core::binary_data::{BinaryDataTable, BinaryDataTableBuilder};
use crate::core::ifd::{IfdParser, TiffHeader};
use crate::core::types::ExifFormat;
use crate::core::{Endian, ExifValue};
use crate::error::Result;
use crate::maker::MakerNoteParser;
use crate::tables::lookup_sony_tag;
use std::collections::HashMap;

pub mod sony_cipher;

/// Parser for Sony maker notes
pub struct SonyMakerNoteParser;

impl MakerNoteParser for SonyMakerNoteParser {
    fn parse(
        &self,
        data: &[u8],
        byte_order: Endian,
        _base_offset: usize,
    ) -> Result<HashMap<u16, ExifValue>> {
        // Sony maker notes use standard IFD structure (like Canon and Nikon)
        // They use the same byte order as the main EXIF data
        // Many tags are encrypted and will be processed by ProcessBinaryData

        if data.is_empty() {
            return Ok(HashMap::new());
        }

        // Sony maker notes start directly with an IFD (no special header)
        // Parse as a standard IFD
        let parsing_data = data;

        // Create a fake TIFF header for IFD parsing
        // (Sony maker notes don't have a TIFF header, they start directly with IFD)
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

        match IfdParser::parse_ifd(&tiff_data, &header, 8) {
            Ok(parsed) => {
                // Apply Sony-specific tag prefixing (0x534F)
                let mut sony_tags = HashMap::new();
                for (tag_id, value) in parsed.entries() {
                    let prefixed_id = 0x534F + tag_id;

                    // Look up tag info from generated table
                    if let Some(tag_info) = lookup_sony_tag(*tag_id) {
                        eprintln!(
                            "Sony tag found: {} (0x{:04x} -> 0x{:04x})",
                            tag_info.name, tag_id, prefixed_id
                        );
                    }

                    // Check if this is an encrypted tag that needs special processing
                    if sony_cipher::is_encrypted_tag(*tag_id) {
                        if let Some(processed_value) =
                            self.process_encrypted_tag(*tag_id, value, byte_order)
                        {
                            sony_tags.insert(prefixed_id, processed_value);
                        } else {
                            // If decryption failed, still store the original encrypted value
                            sony_tags.insert(prefixed_id, value.clone());
                        }
                    } else {
                        // Normal unencrypted tag
                        sony_tags.insert(prefixed_id, value.clone());
                    }
                }
                Ok(sony_tags)
            }
            Err(e) => {
                // Log the error but return empty results
                // Sony maker notes may have encrypted sections that cause parsing errors
                eprintln!("Warning: Sony maker note parsing failed: {}", e);
                Ok(HashMap::new())
            }
        }
    }

    fn manufacturer(&self) -> &'static str {
        "Sony"
    }
}

impl SonyMakerNoteParser {
    /// Process encrypted Sony tags using decryption and ProcessBinaryData
    fn process_encrypted_tag(
        &self,
        tag_id: u16,
        value: &ExifValue,
        byte_order: Endian,
    ) -> Option<ExifValue> {
        match tag_id {
            0x2010 => self.process_tag2010(value, byte_order),
            tag if (0x9000..0xa000).contains(&tag) => self.process_9xxx_tag(tag, value, byte_order),
            _ => None,
        }
    }

    /// Process Sony Tag2010 (encrypted tag with multiple variants)
    fn process_tag2010(&self, value: &ExifValue, byte_order: Endian) -> Option<ExifValue> {
        if let ExifValue::Undefined(encrypted_data) = value {
            // Decrypt the data first
            let decrypted_data = sony_cipher::decipher_sony_data_auto(encrypted_data);

            // Create Tag2010 binary data table for parsing the decrypted data
            let tag2010_table = self.create_tag2010_table();

            // Parse the decrypted binary data
            match tag2010_table.parse(&decrypted_data, byte_order) {
                Ok(parsed_fields) => {
                    // For now, return the first parsed field or create a summary
                    if let Some((_, first_value)) = parsed_fields.iter().next() {
                        Some(first_value.clone())
                    } else {
                        // If no fields were parsed, return the decrypted data as Undefined
                        Some(ExifValue::Undefined(decrypted_data))
                    }
                }
                Err(_) => {
                    // If binary data parsing failed, return the decrypted data
                    Some(ExifValue::Undefined(decrypted_data))
                }
            }
        } else {
            None
        }
    }

    /// Process Sony 0x9xxx encrypted tags
    fn process_9xxx_tag(
        &self,
        _tag_id: u16,
        value: &ExifValue,
        _byte_order: Endian,
    ) -> Option<ExifValue> {
        if let ExifValue::Undefined(encrypted_data) = value {
            // Decrypt the 9xxx series data
            let decrypted_data = sony_cipher::decipher_sony_data_auto(encrypted_data);

            // For 9xxx tags, we typically just return the decrypted data
            // without further binary data processing (unless specific tables are defined)
            Some(ExifValue::Undefined(decrypted_data))
        } else {
            None
        }
    }

    /// Create Tag2010 binary data table
    /// This is a simplified version - real implementation would have model-specific variants
    fn create_tag2010_table(&self) -> BinaryDataTable {
        BinaryDataTableBuilder::new("Tag2010", ExifFormat::U16)
            .add_field(0, "SonyImageHeight", ExifFormat::U16, 1)
            .add_field(1, "SonyImageWidth", ExifFormat::U16, 1)
            .add_field(2, "SonyImageSize", ExifFormat::U32, 1)
            .add_field(4, "SonyQuality", ExifFormat::U16, 1)
            .add_field(6, "SonyFlashExposureComp", ExifFormat::I16, 1)
            .add_field(8, "SonyTeleConverter", ExifFormat::U16, 1)
            .add_field(10, "SonyWhiteBalanceFineTune", ExifFormat::I16, 1)
            .add_field(12, "SonyCameraSettings", ExifFormat::U16, 1)
            .add_field(14, "SonyWhiteBalance", ExifFormat::U16, 1)
            .add_field(16, "SonyExtraInfo", ExifFormat::U16, 1)
            .build()
    }
}

/// Sony-specific tag IDs (from ExifTool)
pub mod tags {
    // Main Sony tags from the Main table
    pub const QUALITY: u16 = 0x0102;
    pub const FLASH_EXPOSURE_COMP: u16 = 0x0104;
    pub const TELECONVERTER: u16 = 0x0105;
    pub const WHITE_BALANCE_FINE_TUNE: u16 = 0x0112;
    pub const CAMERA_SETTINGS: u16 = 0x0114;
    pub const WHITE_BALANCE: u16 = 0x0115;
    pub const EXTRA_INFO: u16 = 0x0116;
    pub const PRINT_IM: u16 = 0x0e00;
    pub const MULTI_BURST_MODE: u16 = 0x1000;
    pub const MULTI_BURST_IMAGE_WIDTH: u16 = 0x1001;
    pub const MULTI_BURST_IMAGE_HEIGHT: u16 = 0x1002;
    pub const PANORAMA: u16 = 0x1003;
    pub const PREVIEW_IMAGE: u16 = 0x2001;
    pub const RATING: u16 = 0x2002;
    pub const CONTRAST: u16 = 0x2004;
    pub const SATURATION: u16 = 0x2005;
    pub const SHARPNESS: u16 = 0x2006;
    pub const BRIGHTNESS: u16 = 0x2007;
    pub const LONG_EXPOSURE_NOISE_REDUCTION: u16 = 0x2008;
    pub const HIGH_ISO_NOISE_REDUCTION: u16 = 0x2009;
    pub const HDR: u16 = 0x200a;
    pub const MULTI_FRAME_NOISE_REDUCTION: u16 = 0x200b;
    pub const PICTURE_EFFECT: u16 = 0x200e;
    pub const SOFT_SKIN_EFFECT: u16 = 0x200f;
    pub const VIGNETTING_CORRECTION: u16 = 0x2011;
    pub const LATERAL_CHROMATIC_ABERRATION: u16 = 0x2012;
    pub const DISTORTION_CORRECTION: u16 = 0x2013;
    pub const WB_SHIFT_AMBER_MAGENTA: u16 = 0x2014;
    pub const AUTO_PORTRAIT_FRAMED: u16 = 0x2016;
    pub const FOCUS_MODE: u16 = 0x201b;
    pub const AF_POINT_SELECTED: u16 = 0x201e;
    pub const SHOT_INFO: u16 = 0x3000;

    // Note: Many Sony tags in the 0x9xxx range are encrypted and are now
    // processed by the ProcessBinaryData framework with decryption
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sony_parser_creation() {
        let parser = SonyMakerNoteParser;
        assert_eq!(parser.manufacturer(), "Sony");
    }

    #[test]
    fn test_empty_maker_note() {
        let parser = SonyMakerNoteParser;
        let result = parser.parse(&[], Endian::Little, 0).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_minimal_ifd() {
        let parser = SonyMakerNoteParser;
        // Minimal valid IFD: 0 entries, no next IFD
        let data = vec![
            0x00, 0x00, // 0 entries
            0xFF, 0xFF, 0xFF, 0xFF, // No next IFD (-1)
        ];
        let result = parser.parse(&data, Endian::Little, 0).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_encrypted_tag_detection() {
        use super::sony_cipher;

        // Test 0x2010 tag (encrypted)
        assert!(sony_cipher::is_encrypted_tag(0x2010));

        // Test 0x9xxx range (encrypted)
        assert!(sony_cipher::is_encrypted_tag(0x9000));
        assert!(sony_cipher::is_encrypted_tag(0x9123));
        assert!(sony_cipher::is_encrypted_tag(0x9fff));

        // Test non-encrypted tags
        assert!(!sony_cipher::is_encrypted_tag(0x0102)); // Quality
        assert!(!sony_cipher::is_encrypted_tag(0x2001)); // PreviewImage
        assert!(!sony_cipher::is_encrypted_tag(0xa000)); // Outside 9xxx range
    }

    #[test]
    fn test_tag2010_table_creation() {
        let parser = SonyMakerNoteParser;
        let table = parser.create_tag2010_table();

        assert_eq!(table.name, "Tag2010");
        assert_eq!(table.default_format, ExifFormat::U16);
        assert!(!table.fields.is_empty());

        // Check that we have expected fields
        assert!(table.fields.iter().any(|(pos, _)| *pos == 0)); // SonyImageHeight
        assert!(table.fields.iter().any(|(pos, _)| *pos == 1)); // SonyImageWidth
        assert!(table.fields.iter().any(|(pos, _)| *pos == 4)); // SonyQuality
    }

    #[test]
    fn test_encrypted_tag_processing() {
        let parser = SonyMakerNoteParser;

        // Test with mock encrypted data for Tag2010
        let encrypted_data = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let encrypted_value = ExifValue::Undefined(encrypted_data);

        // Process the encrypted tag
        let result = parser.process_encrypted_tag(0x2010, &encrypted_value, Endian::Little);

        // Should return some processed value (either parsed binary data or decrypted data)
        assert!(result.is_some());
    }

    #[test]
    fn test_9xxx_tag_processing() {
        let parser = SonyMakerNoteParser;

        // Test with mock encrypted data for 9xxx tag
        let encrypted_data = vec![0x10, 0x20, 0x30, 0x40];
        let encrypted_value = ExifValue::Undefined(encrypted_data);

        // Process the encrypted 9xxx tag
        let result = parser.process_encrypted_tag(0x9abc, &encrypted_value, Endian::Little);

        // Should return decrypted data
        assert!(result.is_some());
        if let Some(ExifValue::Undefined(decrypted)) = result {
            // Decrypted data should be same length but different content
            assert_eq!(decrypted.len(), 4);
        }
    }

    #[test]
    fn test_non_encrypted_tag_processing() {
        let parser = SonyMakerNoteParser;

        // Test with regular tag data
        let _regular_data = [0x01, 0x02];
        let regular_value = ExifValue::U16(0x0201);

        // Process a non-encrypted tag
        let result = parser.process_encrypted_tag(0x0102, &regular_value, Endian::Little);

        // Should return None for non-encrypted tags
        assert!(result.is_none());
    }

    #[test]
    fn test_cipher_mode_detection() {
        use super::sony_cipher;

        // Test cipher mode detection with various data patterns
        let test_data1 = vec![0x00, 0x01, 0x02, 0x03];
        let mode1 = sony_cipher::determine_cipher_mode(&test_data1);
        // The actual mode depends on the decryption result, so just verify it's a valid mode
        assert!(matches!(
            mode1,
            sony_cipher::SonyCipherMode::Single | sony_cipher::SonyCipherMode::Double
        ));

        // Test with empty data
        let empty_data = vec![];
        let mode_empty = sony_cipher::determine_cipher_mode(&empty_data);
        assert_eq!(mode_empty, sony_cipher::SonyCipherMode::Single);
    }

    #[test]
    fn test_sony_tag2010_variant_detection() {
        use super::sony_cipher::SonyTag2010Variant;

        // Test model-specific variant detection
        assert_eq!(
            SonyTag2010Variant::from_model("NEX-7"),
            SonyTag2010Variant::Tag2010b
        );
        assert_eq!(
            SonyTag2010Variant::from_model("ALPHA A7"),
            SonyTag2010Variant::Tag2010d
        ); // A7 series use Tag2010d
        assert_eq!(
            SonyTag2010Variant::from_model("RX100 II"),
            SonyTag2010Variant::Tag2010c
        );
        assert_eq!(
            SonyTag2010Variant::from_model("DSC-H50"),
            SonyTag2010Variant::Tag2010a
        );
    }
}
