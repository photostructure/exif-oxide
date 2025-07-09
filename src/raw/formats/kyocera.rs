//! Kyocera RAW format handler
//!
//! This module implements ExifTool's KyoceraRaw.pm processing logic exactly.
//! Kyocera Contax N Digital cameras store RAW metadata in a simple 156-byte
//! binary header with string values stored in byte-reversed format.
//!
//! ExifTool Reference: lib/Image/ExifTool/KyoceraRaw.pm (173 lines total)
//! Processing: ProcessBinaryData with fixed offsets and string reversal

use crate::exif::ExifReader;
use crate::raw::{utils, RawFormatHandler};
use crate::types::{ExifError, Result, TagSourceInfo, TagValue};
use std::collections::HashMap;

/// Kyocera RAW format handler
/// ExifTool: lib/Image/ExifTool/KyoceraRaw.pm - Simple ProcessBinaryData format
pub struct KyoceraRawHandler {
    /// Tag definitions with offsets and formats
    /// ExifTool: %Image::ExifTool::KyoceraRaw::Main hash (lines 25-106)
    tag_definitions: HashMap<u16, KyoceraTagDef>,
}

/// Kyocera tag definition
/// ExifTool: Individual entries in %Image::ExifTool::KyoceraRaw::Main
#[derive(Debug, Clone)]
struct KyoceraTagDef {
    /// Tag name
    /// ExifTool: Key in tag hash
    #[allow(dead_code)] // Used for debugging and future reference
    name: String,
    /// Offset in binary data
    /// ExifTool: Tag number/offset
    offset: usize,
    /// Data format
    /// ExifTool: Format field
    format: KyoceraFormat,
    /// Optional conversion function
    /// ExifTool: ValueConv/PrintConv fields
    conversion: Option<KyoceraConversion>,
}

/// Kyocera data formats
/// ExifTool: Format specifications in tag definitions
#[derive(Debug, Clone)]
enum KyoceraFormat {
    /// Fixed-length string (with byte count)
    /// ExifTool: string[N] format
    String(usize),
    /// 32-bit unsigned integer
    /// ExifTool: int32u format
    Int32u,
    /// Array of 32-bit unsigned integers
    /// ExifTool: int32u[N] format
    Int32uArray(usize),
}

/// Kyocera conversion types
/// ExifTool: ValueConv and PrintConv logic
#[derive(Debug, Clone)]
enum KyoceraConversion {
    /// Reverse string bytes (Kyocera-specific)
    /// ExifTool: \&ReverseString function reference
    ReverseString,
    /// Convert to exposure time
    /// ExifTool: '2**($val / 8) / 16000' expression
    ExposureTime,
    /// Convert to F-number
    /// ExifTool: '2**($val / 16)' expression
    FNumber,
    /// Convert to max aperture
    /// ExifTool: '2**($val / 16)' expression (same as FNumber)
    MaxAperture,
    /// Convert to datetime
    /// ExifTool: '$self->ConvertDateTime($val)' expression
    DateTime,
    /// Convert to focal length with "mm" suffix
    /// ExifTool: '"$val mm"' expression
    FocalLength,
    /// Convert using ISO lookup table
    /// ExifTool: PrintConv hash with ISO values (lines 56-70)
    IsoLookup,
}

impl Default for KyoceraRawHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl KyoceraRawHandler {
    /// Create new Kyocera RAW handler with tag definitions
    /// ExifTool: %Image::ExifTool::KyoceraRaw::Main hash construction
    pub fn new() -> Self {
        let mut tag_definitions = HashMap::new();

        // ExifTool: lib/Image/ExifTool/KyoceraRaw.pm lines 26-106
        // Each tag definition is translated exactly from the Perl hash

        // FirmwareVersion at offset 0x01
        // ExifTool: line 26 - 1 => { Name => 'FirmwareVersion', Format => 'string[10]', ValueConv => \&ReverseString }
        tag_definitions.insert(
            0x01,
            KyoceraTagDef {
                name: "FirmwareVersion".to_string(),
                offset: 0x01,
                format: KyoceraFormat::String(10),
                conversion: Some(KyoceraConversion::ReverseString),
            },
        );

        // Model at offset 0x0c
        // ExifTool: line 27 - 0x0c => { Name => 'Model', Format => 'string[12]', ValueConv => \&ReverseString }
        tag_definitions.insert(
            0x0c,
            KyoceraTagDef {
                name: "Model".to_string(),
                offset: 0x0c,
                format: KyoceraFormat::String(12),
                conversion: Some(KyoceraConversion::ReverseString),
            },
        );

        // Make at offset 0x19
        // ExifTool: line 28 - 0x19 => { Name => 'Make', Format => 'string[7]', ValueConv => \&ReverseString }
        tag_definitions.insert(
            0x19,
            KyoceraTagDef {
                name: "Make".to_string(),
                offset: 0x19,
                format: KyoceraFormat::String(7),
                conversion: Some(KyoceraConversion::ReverseString),
            },
        );

        // DateTimeOriginal at offset 0x21
        // ExifTool: line 29-33 - 0x21 => { Name => 'DateTimeOriginal', Format => 'string[20]', Groups => { 2 => 'Time' }, ValueConv => \&ReverseString, PrintConv => '$self->ConvertDateTime($val)' }
        tag_definitions.insert(
            0x21,
            KyoceraTagDef {
                name: "DateTimeOriginal".to_string(),
                offset: 0x21,
                format: KyoceraFormat::String(20),
                conversion: Some(KyoceraConversion::DateTime),
            },
        );

        // ISO at offset 0x34
        // ExifTool: lines 34-71 - 0x34 => { Name => 'ISO', Format => 'int32u', PrintConv => { hash with ISO values } }
        tag_definitions.insert(
            0x34,
            KyoceraTagDef {
                name: "ISO".to_string(),
                offset: 0x34,
                format: KyoceraFormat::Int32u,
                conversion: Some(KyoceraConversion::IsoLookup),
            },
        );

        // ExposureTime at offset 0x38
        // ExifTool: lines 72-75 - 0x38 => { Name => 'ExposureTime', Format => 'int32u', ValueConv => '2**($val / 8) / 16000', PrintConv => 'Image::ExifTool::Exif::PrintExposureTime($val)' }
        tag_definitions.insert(
            0x38,
            KyoceraTagDef {
                name: "ExposureTime".to_string(),
                offset: 0x38,
                format: KyoceraFormat::Int32u,
                conversion: Some(KyoceraConversion::ExposureTime),
            },
        );

        // WB_RGGBLevels at offset 0x3c
        // ExifTool: line 76 - 0x3c => { Name => 'WB_RGGBLevels', Format => 'int32u[4]' }
        tag_definitions.insert(
            0x3c,
            KyoceraTagDef {
                name: "WB_RGGBLevels".to_string(),
                offset: 0x3c,
                format: KyoceraFormat::Int32uArray(4),
                conversion: None,
            },
        );

        // FNumber at offset 0x58
        // ExifTool: lines 77-79 - 0x58 => { Name => 'FNumber', Format => 'int32u', ValueConv => '2**($val / 16)', PrintConv => 'sprintf("%.1f",$val)' }
        tag_definitions.insert(
            0x58,
            KyoceraTagDef {
                name: "FNumber".to_string(),
                offset: 0x58,
                format: KyoceraFormat::Int32u,
                conversion: Some(KyoceraConversion::FNumber),
            },
        );

        // MaxAperture at offset 0x68
        // ExifTool: lines 80-82 - 0x68 => { Name => 'MaxAperture', Format => 'int32u', ValueConv => '2**($val / 16)', PrintConv => 'sprintf("%.1f",$val)' }
        tag_definitions.insert(
            0x68,
            KyoceraTagDef {
                name: "MaxAperture".to_string(),
                offset: 0x68,
                format: KyoceraFormat::Int32u,
                conversion: Some(KyoceraConversion::MaxAperture),
            },
        );

        // FocalLength at offset 0x70
        // ExifTool: lines 83-84 - 0x70 => { Name => 'FocalLength', Format => 'int32u', PrintConv => '"$val mm"' }
        tag_definitions.insert(
            0x70,
            KyoceraTagDef {
                name: "FocalLength".to_string(),
                offset: 0x70,
                format: KyoceraFormat::Int32u,
                conversion: Some(KyoceraConversion::FocalLength),
            },
        );

        // Lens at offset 0x7c
        // ExifTool: line 85 - 0x7c => { Name => 'Lens', Format => 'string[32]' }
        tag_definitions.insert(
            0x7c,
            KyoceraTagDef {
                name: "Lens".to_string(),
                offset: 0x7c,
                format: KyoceraFormat::String(32),
                conversion: None,
            },
        );

        Self { tag_definitions }
    }

    /// Extract value from binary data based on format
    /// ExifTool: ProcessBinaryData value extraction logic
    fn extract_value(
        &self,
        data: &[u8],
        offset: usize,
        format: &KyoceraFormat,
    ) -> Result<TagValue> {
        match format {
            KyoceraFormat::String(length) => {
                if offset + length > data.len() {
                    return Err(ExifError::ParseError(format!(
                        "String at offset {offset:#x} extends beyond data bounds"
                    )));
                }
                let string_bytes = &data[offset..offset + length];
                let string_value = String::from_utf8_lossy(string_bytes).to_string();
                Ok(TagValue::String(string_value))
            }
            KyoceraFormat::Int32u => {
                if offset + 4 > data.len() {
                    return Err(ExifError::ParseError(format!(
                        "Int32u at offset {offset:#x} extends beyond data bounds"
                    )));
                }
                // Kyocera uses big-endian byte order (MM)
                // ExifTool: KyoceraRaw.pm line 23 - ByteOrder => 'MM'
                let value = u32::from_be_bytes([
                    data[offset],
                    data[offset + 1],
                    data[offset + 2],
                    data[offset + 3],
                ]);
                Ok(TagValue::U32(value))
            }
            KyoceraFormat::Int32uArray(count) => {
                let total_bytes = count * 4;
                if offset + total_bytes > data.len() {
                    return Err(ExifError::ParseError(format!(
                        "Int32u array at offset {offset:#x} extends beyond data bounds"
                    )));
                }

                let mut values = Vec::new();
                for i in 0..*count {
                    let item_offset = offset + i * 4;
                    let value = u32::from_be_bytes([
                        data[item_offset],
                        data[item_offset + 1],
                        data[item_offset + 2],
                        data[item_offset + 3],
                    ]);
                    values.push(value);
                }
                Ok(TagValue::U32Array(values))
            }
        }
    }

    /// Apply conversion to extracted value
    /// ExifTool: ValueConv and PrintConv logic from tag definitions
    fn apply_conversion(&self, value: &TagValue, conversion: &KyoceraConversion) -> TagValue {
        match conversion {
            KyoceraConversion::ReverseString => {
                if let TagValue::String(s) = value {
                    TagValue::String(utils::reverse_string(s.as_bytes()))
                } else {
                    value.clone()
                }
            }
            KyoceraConversion::ExposureTime => {
                if let TagValue::U32(val) = value {
                    let exposure_time = utils::kyocera_exposure_time(*val);
                    TagValue::F64(exposure_time)
                } else {
                    value.clone()
                }
            }
            KyoceraConversion::FNumber | KyoceraConversion::MaxAperture => {
                if let TagValue::U32(val) = value {
                    let fnumber = utils::kyocera_fnumber(*val);
                    TagValue::F64(fnumber)
                } else {
                    value.clone()
                }
            }
            KyoceraConversion::DateTime => {
                if let TagValue::String(s) = value {
                    // First reverse the string, then format as datetime
                    let reversed = utils::reverse_string(s.as_bytes());
                    // ExifTool ConvertDateTime logic would go here
                    // For now, just return the reversed string
                    TagValue::String(reversed)
                } else {
                    value.clone()
                }
            }
            KyoceraConversion::FocalLength => {
                if let TagValue::U32(val) = value {
                    TagValue::String(format!("{val} mm"))
                } else {
                    value.clone()
                }
            }
            KyoceraConversion::IsoLookup => {
                if let TagValue::U32(val) = value {
                    if let Some(iso_speed) = utils::kyocera_iso_lookup(*val) {
                        TagValue::U32(iso_speed)
                    } else {
                        value.clone() // Keep original value if not in lookup
                    }
                } else {
                    value.clone()
                }
            }
        }
    }
}

impl RawFormatHandler for KyoceraRawHandler {
    /// Process Kyocera RAW data
    /// ExifTool: ProcessBinaryData call with KyoceraRaw::Main table
    fn process_raw(&self, reader: &mut ExifReader, data: &[u8]) -> Result<()> {
        // Validate minimum data size
        // ExifTool: KyoceraRaw.pm processes exactly 156 bytes
        if data.len() < 156 {
            return Err(ExifError::ParseError(format!(
                "Kyocera RAW data too short: {} bytes (expected 156)",
                data.len()
            )));
        }

        // Process each tag definition
        for (&tag_id, tag_def) in &self.tag_definitions {
            // Extract raw value
            let raw_value = self.extract_value(data, tag_def.offset, &tag_def.format)?;

            // Apply conversion if specified
            let final_value = if let Some(conversion) = &tag_def.conversion {
                self.apply_conversion(&raw_value, conversion)
            } else {
                raw_value
            };

            // Store the tag with source info
            // ExifTool: Groups => { 0 => 'MakerNotes', 2 => 'Camera' }
            // Individual tags may override group 2 (e.g., DateTimeOriginal => 'Time', ISO => 'Image')
            let group2 = match tag_def.name.as_str() {
                "DateTimeOriginal" => "Time",
                "ISO" | "ExposureTime" | "FNumber" | "MaxAperture" | "FocalLength"
                | "WB_RGGBLevels" => "Image",
                _ => "Camera", // Default
            };

            let source_info = TagSourceInfo::new(
                "MakerNotes".to_string(),
                "KyoceraRaw".to_string(),
                group2.to_string(),
            );

            reader.extracted_tags.insert(tag_id, final_value);
            reader.tag_sources.insert(tag_id, source_info);
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "KyoceraRaw"
    }

    fn validate_format(&self, data: &[u8]) -> bool {
        // ExifTool: KyoceraRaw.pm validation logic
        // Check for minimum size and magic bytes
        super::super::detector::validate_kyocera_magic(data)
    }
}

/// Get Kyocera RAW tag name by ID
/// ExifTool: lib/Image/ExifTool/KyoceraRaw.pm tag definitions
pub fn get_kyocera_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        0x01 => Some("FirmwareVersion"),
        0x0c => Some("Model"),
        0x19 => Some("Make"),
        0x21 => Some("DateTimeOriginal"),
        0x34 => Some("ISO"),
        0x38 => Some("ExposureTime"),
        0x3c => Some("WB_RGGBLevels"),
        0x58 => Some("FNumber"),
        0x68 => Some("MaxAperture"),
        0x70 => Some("FocalLength"),
        0x7c => Some("Lens"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kyocera_handler_creation() {
        let handler = KyoceraRawHandler::new();
        assert_eq!(handler.name(), "KyoceraRaw");

        // Check that we have the expected number of tag definitions
        assert_eq!(handler.tag_definitions.len(), 11);

        // Check some key tag definitions
        assert!(handler.tag_definitions.contains_key(&0x19)); // Make
        assert!(handler.tag_definitions.contains_key(&0x34)); // ISO
        assert!(handler.tag_definitions.contains_key(&0x38)); // ExposureTime
    }

    #[test]
    fn test_get_kyocera_tag_name() {
        // Test known tag names
        assert_eq!(get_kyocera_tag_name(0x19), Some("Make"));
        assert_eq!(get_kyocera_tag_name(0x34), Some("ISO"));
        assert_eq!(get_kyocera_tag_name(0x38), Some("ExposureTime"));
        assert_eq!(get_kyocera_tag_name(0x01), Some("FirmwareVersion"));
        assert_eq!(get_kyocera_tag_name(0x0c), Some("Model"));

        // Test unknown tag
        assert_eq!(get_kyocera_tag_name(0x99), None);
    }

    #[test]
    fn test_value_extraction() {
        let handler = KyoceraRawHandler::new();

        // Test string extraction
        let mut data = vec![0u8; 20];
        data[10..17].copy_from_slice(b"ARECOYK");

        let result = handler
            .extract_value(&data, 10, &KyoceraFormat::String(7))
            .unwrap();
        if let TagValue::String(s) = result {
            assert!(s.starts_with("ARECOYK"));
        } else {
            panic!("Expected String value");
        }

        // Test int32u extraction (big-endian)
        let mut data = vec![0u8; 10];
        data[4..8].copy_from_slice(&[0x00, 0x00, 0x01, 0x00]); // 256 in big-endian

        let result = handler
            .extract_value(&data, 4, &KyoceraFormat::Int32u)
            .unwrap();
        if let TagValue::U32(val) = result {
            assert_eq!(val, 256);
        } else {
            panic!("Expected U32 value");
        }
    }

    #[test]
    fn test_conversions() {
        let handler = KyoceraRawHandler::new();

        // Test string reversal
        let input = TagValue::String("KYOCERA\0".to_string());
        let result = handler.apply_conversion(&input, &KyoceraConversion::ReverseString);
        if let TagValue::String(s) = result {
            assert!(s.starts_with("ARECOYK"));
        } else {
            panic!("Expected String value");
        }

        // Test ISO lookup
        let input = TagValue::U32(13); // Should map to ISO 100
        let result = handler.apply_conversion(&input, &KyoceraConversion::IsoLookup);
        if let TagValue::U32(iso) = result {
            assert_eq!(iso, 100);
        } else {
            panic!("Expected U32 value");
        }

        // Test focal length formatting
        let input = TagValue::U32(50);
        let result = handler.apply_conversion(&input, &KyoceraConversion::FocalLength);
        if let TagValue::String(s) = result {
            assert_eq!(s, "50 mm");
        } else {
            panic!("Expected String value");
        }
    }

    #[test]
    fn test_format_validation() {
        let handler = KyoceraRawHandler::new();

        // Create valid test data with magic bytes
        let mut valid_data = vec![0u8; 156];
        valid_data[0x19..0x19 + 7].copy_from_slice(b"ARECOYK");

        assert!(handler.validate_format(&valid_data));

        // Test invalid data
        let invalid_data = vec![0u8; 100]; // Too short
        assert!(!handler.validate_format(&invalid_data));

        let mut wrong_magic = vec![0u8; 156];
        wrong_magic[0x19..0x19 + 7].copy_from_slice(b"WRONGXY");
        assert!(!handler.validate_format(&wrong_magic));
    }
}
