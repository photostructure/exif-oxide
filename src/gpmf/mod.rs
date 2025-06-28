//! GoPro GPMF (GoPro Metadata Format) Parser
//!
//! EXIFTOOL-SOURCE: lib/Image/ExifTool/GoPro.pm
//!
//! GPMF is GoPro's custom metadata format used in MP4 videos and JPEG files.
//! Unlike traditional EXIF maker notes, GPMF uses a completely different
//! binary structure with 4-byte tag IDs and nested containers.

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/GoPro.pm"]

use crate::core::ExifValue;
use crate::error::Result;
use std::collections::HashMap;

pub mod format;
pub mod tags;

#[cfg(test)]
mod test_integration;

pub use format::{
    get_default_format_size, get_gpmf_format, get_gpmf_size, GpmfFormat, GPMF_FORMAT_COUNT,
    GPMF_FORMAT_MAP, GPMF_SIZE_MAP,
};
pub use tags::*;

/// GPMF parser for extracting GoPro metadata
pub struct GpmfParser;

impl GpmfParser {
    /// Create a new GPMF parser
    pub fn new() -> Self {
        Self
    }

    /// Parse GPMF data from a binary stream
    ///
    /// GPMF structure (from ExifTool ProcessGoPro):
    /// - 4-byte tag ID (like "DEVC", "ACCL")
    /// - 1-byte format code (0x62=int8s, 0x42=int8u, etc.)
    /// - 1-byte size per sample
    /// - 2-byte count (number of samples)
    /// - Data payload (size * count bytes)
    /// - Padding to 4-byte boundary
    pub fn parse(&self, data: &[u8]) -> Result<HashMap<String, ExifValue>> {
        let mut result = HashMap::new();
        let mut pos = 0;

        self.parse_gpmf_stream(data, &mut pos, &mut result)?;

        Ok(result)
    }

    /// Parse a GPMF stream recursively
    fn parse_gpmf_stream(
        &self,
        data: &[u8],
        pos: &mut usize,
        result: &mut HashMap<String, ExifValue>,
    ) -> Result<()> {
        while *pos + 8 <= data.len() {
            // Read GPMF header (8 bytes)
            let tag_bytes = &data[*pos..*pos + 4];
            let tag_id = std::str::from_utf8(tag_bytes)
                .map_err(|_| crate::error::Error::InvalidData("Invalid GPMF tag ID".to_string()))?
                .to_string();

            let format_code = data[*pos + 4];
            let size_per_sample = data[*pos + 5] as usize;
            let count = u16::from_be_bytes([data[*pos + 6], data[*pos + 7]]) as usize;

            *pos += 8;

            let payload_size = size_per_sample * count;

            // Check bounds
            if *pos + payload_size > data.len() {
                break;
            }

            // Stop at null tag
            if tag_id == "\0\0\0\0" {
                break;
            }

            // Skip empty values unless in verbose mode
            if payload_size == 0 {
                continue;
            }

            // Get tag definition
            let tag_def = get_gpmf_tag(&tag_id);

            // Parse payload based on format
            let payload = &data[*pos..*pos + payload_size];

            if format_code == 0x00 {
                // Container format - recurse into subdirectory
                if tag_def.map(|t| t.subdirectory).unwrap_or(false) {
                    let mut sub_pos = 0;
                    self.parse_gpmf_stream(payload, &mut sub_pos, result)?;
                }
            } else {
                // Parse data value based on format
                let value = self.parse_gpmf_value(payload, format_code, size_per_sample, count)?;

                // Apply PrintConv if available
                let final_value = if let Some(tag) = tag_def {
                    if !tag.binary {
                        // Apply PrintConv conversion for non-binary tags
                        // TODO: Integrate with PrintConv system
                        value
                    } else {
                        value
                    }
                } else {
                    value
                };

                result.insert(tag_id, final_value);
            }

            // Advance position with 4-byte alignment
            *pos += (payload_size + 3) & !3;
        }

        Ok(())
    }

    /// Parse a GPMF value based on format code
    fn parse_gpmf_value(
        &self,
        data: &[u8],
        format_code: u8,
        size_per_sample: usize,
        count: usize,
    ) -> Result<ExifValue> {
        // Get format information
        let _format = get_gpmf_format(format_code);

        if count == 1 {
            // Single value
            self.parse_single_value(data, format_code, size_per_sample)
        } else {
            // Multiple values - create array
            let mut values = Vec::new();
            for i in 0..count {
                let start = i * size_per_sample;
                let end = start + size_per_sample;
                if end <= data.len() {
                    let value =
                        self.parse_single_value(&data[start..end], format_code, size_per_sample)?;
                    values.push(value);
                }
            }

            // Convert to space-separated string for compatibility
            let value_strings: Vec<String> = values
                .iter()
                .map(|v| self.exif_value_to_string(v).to_string())
                .collect();

            Ok(ExifValue::Ascii(value_strings.join(" ")))
        }
    }

    /// Parse a single GPMF value
    fn parse_single_value(&self, data: &[u8], format_code: u8, _size: usize) -> Result<ExifValue> {
        match format_code {
            // int8s - signed 8-bit
            0x62 => {
                if !data.is_empty() {
                    Ok(ExifValue::I16(data[0] as i8 as i16))
                } else {
                    Ok(ExifValue::I16(0))
                }
            }

            // int8u - unsigned 8-bit
            0x42 => {
                if !data.is_empty() {
                    Ok(ExifValue::U8(data[0]))
                } else {
                    Ok(ExifValue::U8(0))
                }
            }

            // string - null-terminated or fixed length
            0x63 => {
                let string_data = if let Some(null_pos) = data.iter().position(|&b| b == 0) {
                    &data[..null_pos]
                } else {
                    data
                };
                let s = String::from_utf8_lossy(string_data).to_string();
                Ok(ExifValue::Ascii(s))
            }

            // int16s - signed 16-bit big-endian
            0x73 => {
                if data.len() >= 2 {
                    let value = i16::from_be_bytes([data[0], data[1]]);
                    Ok(ExifValue::I16(value))
                } else {
                    Ok(ExifValue::I16(0))
                }
            }

            // int16u - unsigned 16-bit big-endian
            0x53 => {
                if data.len() >= 2 {
                    let value = u16::from_be_bytes([data[0], data[1]]);
                    Ok(ExifValue::U16(value))
                } else {
                    Ok(ExifValue::U16(0))
                }
            }

            // int32s - signed 32-bit big-endian
            0x6c => {
                if data.len() >= 4 {
                    let value = i32::from_be_bytes([data[0], data[1], data[2], data[3]]);
                    Ok(ExifValue::I32(value))
                } else {
                    Ok(ExifValue::I32(0))
                }
            }

            // int32u - unsigned 32-bit big-endian
            0x4c => {
                if data.len() >= 4 {
                    let value = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
                    Ok(ExifValue::U32(value))
                } else {
                    Ok(ExifValue::U32(0))
                }
            }

            // float - 32-bit IEEE 754 big-endian
            0x66 => {
                if data.len() >= 4 {
                    // Store as Undefined since ExifValue doesn't have Float
                    Ok(ExifValue::Undefined(data[..4].to_vec()))
                } else {
                    Ok(ExifValue::Undefined(vec![0, 0, 0, 0]))
                }
            }

            // double - 64-bit IEEE 754 big-endian
            0x64 => {
                if data.len() >= 8 {
                    // Store as Undefined since ExifValue doesn't have Double
                    Ok(ExifValue::Undefined(data[..8].to_vec()))
                } else {
                    Ok(ExifValue::Undefined(vec![0, 0, 0, 0, 0, 0, 0, 0]))
                }
            }

            // int64s - signed 64-bit big-endian
            0x6a => {
                if data.len() >= 8 {
                    // Store as Undefined since ExifValue doesn't have 64-bit types
                    Ok(ExifValue::Undefined(data[..8].to_vec()))
                } else {
                    Ok(ExifValue::Undefined(vec![0, 0, 0, 0, 0, 0, 0, 0]))
                }
            }

            // int64u - unsigned 64-bit big-endian
            0x4a => {
                if data.len() >= 8 {
                    // Store as Undefined since ExifValue doesn't have 64-bit types
                    Ok(ExifValue::Undefined(data[..8].to_vec()))
                } else {
                    Ok(ExifValue::Undefined(vec![0, 0, 0, 0, 0, 0, 0, 0]))
                }
            }

            // fixed32s, fixed64s, undef, and other formats
            _ => {
                // For unknown formats, store as binary data
                Ok(ExifValue::Undefined(data.to_vec()))
            }
        }
    }

    /// Convert ExifValue to string representation
    fn exif_value_to_string(&self, value: &ExifValue) -> String {
        match value {
            ExifValue::Ascii(s) => s.clone(),
            ExifValue::U8(n) => n.to_string(),
            ExifValue::U8Array(arr) => arr
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(" "),
            ExifValue::U16(n) => n.to_string(),
            ExifValue::U16Array(arr) => arr
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(" "),
            ExifValue::U32(n) => n.to_string(),
            ExifValue::U32Array(arr) => arr
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(" "),
            ExifValue::I16(n) => n.to_string(),
            ExifValue::I16Array(arr) => arr
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(" "),
            ExifValue::I32(n) => n.to_string(),
            ExifValue::I32Array(arr) => arr
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(" "),
            ExifValue::Rational(num, den) => {
                if *den == 1 {
                    num.to_string()
                } else {
                    format!("{}/{}", num, den)
                }
            }
            ExifValue::RationalArray(arr) => arr
                .iter()
                .map(|(num, den)| {
                    if *den == 1 {
                        num.to_string()
                    } else {
                        format!("{}/{}", num, den)
                    }
                })
                .collect::<Vec<_>>()
                .join(" "),
            ExifValue::SignedRational(num, den) => {
                if *den == 1 {
                    num.to_string()
                } else {
                    format!("{}/{}", num, den)
                }
            }
            ExifValue::SignedRationalArray(arr) => arr
                .iter()
                .map(|(num, den)| {
                    if *den == 1 {
                        num.to_string()
                    } else {
                        format!("{}/{}", num, den)
                    }
                })
                .collect::<Vec<_>>()
                .join(" "),
            ExifValue::Undefined(v) => format!("(Binary data, {} bytes)", v.len()),
            ExifValue::BinaryData(len) => format!("(Binary data, {} bytes)", len),
        }
    }
}

impl Default for GpmfParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse GPMF data from a byte array
///
/// This is the main entry point for parsing GoPro GPMF metadata
/// from MP4 GPMF boxes or JPEG APP6 segments.
pub fn parse_gpmf(data: &[u8]) -> Result<HashMap<String, ExifValue>> {
    let parser = GpmfParser::new();
    parser.parse(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpmf_parser_creation() {
        let parser = GpmfParser::new();
        assert_eq!(std::mem::size_of_val(&parser), 0); // Zero-sized struct
    }

    #[test]
    fn test_parse_empty_data() {
        let parser = GpmfParser::new();
        let result = parser.parse(&[]);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_parse_null_tag() {
        let parser = GpmfParser::new();
        // Create minimal GPMF data with null tag
        let data = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let result = parser.parse(&data);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
