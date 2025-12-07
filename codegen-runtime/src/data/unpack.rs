/// Binary unpacking utilities for Perl-style unpack operations
///
/// This module provides functions to unpack binary data according to
/// Perl unpack format specifications.
use crate::TagValue;

/// Unpack binary data according to a Perl unpack specification
///
/// Common formats:
/// - `C` - unsigned char (u8)
/// - `n` - network (big-endian) short (u16)
/// - `N` - network (big-endian) long (u32)
/// - `H2` - hex string, 2 digits
///
/// # Examples
/// ```
/// use codegen_runtime::{TagValue, data::unpack_binary};
///
/// let result = unpack_binary("nC2", &TagValue::Binary(vec![0x20, 0x24, 0x0A, 0x14]));
/// // Returns: [TagValue::U16(8228), TagValue::U8(10), TagValue::U8(20)]
/// ```
pub fn unpack_binary(spec: &str, val: &TagValue) -> Vec<TagValue> {
    let bytes = match val {
        TagValue::Binary(b) => b.clone(),
        TagValue::String(s) => s.as_bytes().to_vec(),
        _ => return vec![TagValue::I32(0)], // fallback
    };

    unpack_bytes(spec, &bytes)
}

/// Unpack bytes according to specification
fn unpack_bytes(spec: &str, bytes: &[u8]) -> Vec<TagValue> {
    let mut results = Vec::new();
    let mut byte_index = 0;
    let mut spec_chars = spec.chars().peekable();

    while let Some(ch) = spec_chars.next() {
        // Handle special cases that need to look ahead for their own count
        match ch {
            'H' => {
                // Hex string - next char should be digit count
                let hex_count = if let Some(&next) = spec_chars.peek() {
                    if next.is_ascii_digit() {
                        spec_chars.next().unwrap().to_digit(10).unwrap_or(2) as usize
                    } else {
                        2 // Default to 2 hex digits
                    }
                } else {
                    2 // Default to 2 hex digits
                };

                let byte_count = hex_count.div_ceil(2);
                let mut hex_str = String::new();
                for _ in 0..byte_count {
                    if byte_index < bytes.len() {
                        let b = bytes[byte_index];
                        hex_str.push_str(&format!("{b:02x}"));
                        byte_index += 1;
                    } else {
                        hex_str.push_str("00");
                    }
                }
                // Truncate to exact hex digit count
                hex_str.truncate(hex_count);
                results.push(TagValue::String(hex_str));
            }
            _ => {
                // Get repeat count if present for other format characters
                let mut count_str = String::new();
                while let Some(&next) = spec_chars.peek() {
                    if next.is_ascii_digit() {
                        count_str.push(spec_chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                let count = if count_str.is_empty() {
                    1
                } else {
                    count_str.parse::<usize>().unwrap_or(1)
                };

                // Process the format character
                for _ in 0..count {
                    if byte_index >= bytes.len() {
                        // Not enough bytes - pad with zeros
                        results.push(TagValue::I32(0));
                        continue;
                    }

                    match ch {
                        'C' => {
                            // Unsigned char
                            results.push(TagValue::U8(bytes[byte_index]));
                            byte_index += 1;
                        }
                        'c' => {
                            // Signed char
                            results.push(TagValue::I32(bytes[byte_index] as i8 as i32));
                            byte_index += 1;
                        }
                        'n' => {
                            // Network (big-endian) unsigned short
                            if byte_index + 1 < bytes.len() {
                                let value = ((bytes[byte_index] as u16) << 8)
                                    | (bytes[byte_index + 1] as u16);
                                results.push(TagValue::U16(value));
                                byte_index += 2;
                            } else {
                                results.push(TagValue::I32(0));
                                byte_index = bytes.len();
                            }
                        }
                        'N' => {
                            // Network (big-endian) unsigned long
                            if byte_index + 3 < bytes.len() {
                                let value = ((bytes[byte_index] as u32) << 24)
                                    | ((bytes[byte_index + 1] as u32) << 16)
                                    | ((bytes[byte_index + 2] as u32) << 8)
                                    | (bytes[byte_index + 3] as u32);
                                results.push(TagValue::U32(value));
                                byte_index += 4;
                            } else {
                                results.push(TagValue::I32(0));
                                byte_index = bytes.len();
                            }
                        }
                        'v' => {
                            // Little-endian unsigned short
                            if byte_index + 1 < bytes.len() {
                                let value = (bytes[byte_index] as u16)
                                    | ((bytes[byte_index + 1] as u16) << 8);
                                results.push(TagValue::U16(value));
                                byte_index += 2;
                            } else {
                                results.push(TagValue::I32(0));
                                byte_index = bytes.len();
                            }
                        }
                        'V' => {
                            // Little-endian unsigned long
                            if byte_index + 3 < bytes.len() {
                                let value = (bytes[byte_index] as u32)
                                    | ((bytes[byte_index + 1] as u32) << 8)
                                    | ((bytes[byte_index + 2] as u32) << 16)
                                    | ((bytes[byte_index + 3] as u32) << 24);
                                results.push(TagValue::U32(value));
                                byte_index += 4;
                            } else {
                                results.push(TagValue::I32(0));
                                byte_index = bytes.len();
                            }
                        }
                        _ => {
                            // Unknown format - skip
                            byte_index += 1;
                        }
                    }
                }
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unpack_basic() {
        let bytes = TagValue::Binary(vec![0x12, 0x34, 0x56]);
        let result = unpack_binary("C3", &bytes);
        assert_eq!(
            result,
            vec![TagValue::U8(0x12), TagValue::U8(0x34), TagValue::U8(0x56)]
        );
    }

    #[test]
    fn test_unpack_network_short() {
        let bytes = TagValue::Binary(vec![0x12, 0x34, 0x56, 0x78]);
        let result = unpack_binary("n2", &bytes);
        assert_eq!(result, vec![TagValue::U16(0x1234), TagValue::U16(0x5678)]);
    }

    #[test]
    fn test_unpack_mixed() {
        let bytes = TagValue::Binary(vec![0x20, 0x24, 0x0A, 0x14]);
        let result = unpack_binary("nC2", &bytes);
        assert_eq!(
            result,
            vec![
                TagValue::U16(0x2024),
                TagValue::U8(0x0A),
                TagValue::U8(0x14)
            ]
        );
    }

    #[test]
    fn test_unpack_hex() {
        let bytes = TagValue::Binary(vec![0xAB, 0xCD, 0xEF]);
        let result = unpack_binary("H2H2H2", &bytes);
        assert_eq!(
            result,
            vec![
                TagValue::String("ab".to_string()),
                TagValue::String("cd".to_string()),
                TagValue::String("ef".to_string())
            ]
        );
    }
}
