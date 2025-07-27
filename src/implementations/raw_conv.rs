//! RawConv implementations for special tag value conversions
//!
//! RawConv functions are applied to raw tag values before ValueConv/PrintConv,
//! typically used for decoding or special processing of raw data.

use crate::types::{Result, TagValue};
use tracing::debug;

/// Convert EXIF text with character encoding prefix to UTF-8 string
/// ExifTool: lib/Image/ExifTool/Exif.pm ConvertExifText()
///
/// UserComment and similar tags start with an 8-byte character code:
/// - "ASCII\0\0\0" - ASCII text
/// - "UNICODE\0" - UTF-16 text (byte order must be guessed)
/// - "JIS\0\0\0\0\0" - JIS encoding
/// - "\0\0\0\0\0\0\0\0" - Undefined encoding (treat as ASCII)
pub fn convert_exif_text(value: &TagValue) -> Result<TagValue> {
    match value {
        TagValue::Binary(data) => {
            if data.len() < 8 {
                // Too short to have encoding prefix
                return Ok(value.clone());
            }

            let id = &data[0..8];
            let text_data = &data[8..];

            // Check encoding type
            if id.starts_with(b"ASCII") || id == b"\0\0\0\0\0\0\0\0" {
                // ASCII encoding (or undefined)
                // Find null terminator if present
                let end = text_data
                    .iter()
                    .position(|&b| b == 0)
                    .unwrap_or(text_data.len());
                let text_slice = &text_data[..end];

                // Convert to UTF-8 string, replacing invalid sequences
                match String::from_utf8(text_slice.to_vec()) {
                    Ok(s) => Ok(TagValue::String(s.trim_end().to_string())),
                    Err(_) => {
                        // Fall back to lossy conversion
                        let s = String::from_utf8_lossy(text_slice);
                        Ok(TagValue::String(s.trim_end().to_string()))
                    }
                }
            } else if id.starts_with(b"UNICODE") {
                // UTF-16 encoding - need to detect byte order
                // ExifTool: MicrosoftPhoto writes little-endian even in big-endian EXIF

                // Try to detect byte order by checking for common patterns
                let is_little_endian = if text_data.len() >= 2 {
                    // Check if first character looks like ASCII in LE UTF-16
                    text_data[0] < 128 && text_data[1] == 0
                } else {
                    true // default to LE
                };

                // Convert UTF-16 to UTF-8
                let utf16_values: Vec<u16> = if is_little_endian {
                    text_data
                        .chunks_exact(2)
                        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                        .take_while(|&v| v != 0) // Stop at null terminator
                        .collect()
                } else {
                    text_data
                        .chunks_exact(2)
                        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
                        .take_while(|&v| v != 0) // Stop at null terminator
                        .collect()
                };

                match String::from_utf16(&utf16_values) {
                    Ok(s) => Ok(TagValue::String(s.trim_end().to_string())),
                    Err(_) => {
                        // Fall back to lossy conversion
                        let s = String::from_utf16_lossy(&utf16_values);
                        Ok(TagValue::String(s.trim_end().to_string()))
                    }
                }
            } else if id.starts_with(b"JIS") {
                // JIS encoding - not commonly used, return as-is for now
                debug!("JIS encoding not yet implemented for UserComment");
                Ok(value.clone())
            } else {
                // Unknown encoding - warn and return full data as string
                debug!("Unknown EXIF text encoding: {:?}", id);
                // Try to convert the whole thing as UTF-8
                match String::from_utf8(data.clone()) {
                    Ok(s) => Ok(TagValue::String(s.trim_end().to_string())),
                    Err(_) => Ok(value.clone()),
                }
            }
        }
        _ => Ok(value.clone()), // Not binary data, return as-is
    }
}

/// Placeholder for missing RawConv implementations
/// ExifTool: Various modules have RawConv functions we haven't implemented yet
pub fn missing_raw_conv(_tag_id: u16, raw_conv_expr: &str, value: &TagValue) -> Result<TagValue> {
    debug!(
        "Missing RawConv implementation for expression: {}",
        raw_conv_expr
    );
    Ok(value.clone())
}
