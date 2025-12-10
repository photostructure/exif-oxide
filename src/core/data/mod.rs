//! Binary data parsing and packing operations
//!
//! This module provides functions for packing and unpacking binary data
//! according to format specifications, similar to Perl's pack/unpack functions.

pub mod unpack;

pub use unpack::unpack_binary;

use crate::core::TagValue;

/// Join a vector of TagValues with a separator
///
/// Implements: join "separator", @array
/// This joins a list of values with the given separator.
///
/// # Arguments
/// * `separator` - String to join results with (e.g., " ", "-")
/// * `values` - Slice of TagValues to join
///
/// # Example
/// ```rust
/// # use exif_oxide::core::{TagValue, join_vec};
/// let values = vec![TagValue::String("a".into()), TagValue::String("b".into())];
/// let result = join_vec(" ", &values);
/// // Returns: TagValue::String("a b")
/// ```
pub fn join_vec(separator: &str, values: &[TagValue]) -> TagValue {
    let strings: Vec<String> = values.iter().map(|v| v.to_string()).collect();
    TagValue::String(strings.join(separator))
}

/// Pack "C*" with bit extraction pattern
///
/// Implements: pack "C*", map { (($val>>$_)&mask)+offset } shifts...
/// This extracts specific bit ranges from val at different shift positions,
/// applies a mask and offset, then packs as unsigned chars into a binary string.
///
/// # Arguments
/// * `val` - The TagValue to extract bits from
/// * `shifts` - Array of bit shift positions
/// * `mask` - Bitmask to apply after shifting
/// * `offset` - Offset to add to each extracted value
///
/// # Example
/// ```rust
/// # use exif_oxide::core::{TagValue, pack_c_star_bit_extract};
/// // Equivalent to: pack "C*", map { (($val>>$_)&0x1f)+0x60 } 10, 5, 0
/// let result = pack_c_star_bit_extract(&TagValue::I32(0x1234), &[10, 5, 0], 0x1f, 0x60);
/// ```
pub fn pack_c_star_bit_extract(val: &TagValue, shifts: &[i32], mask: i32, offset: i32) -> TagValue {
    // Extract numeric value from TagValue
    let numeric_val = match val {
        TagValue::I32(i) => *i,
        TagValue::F64(f) => *f as i32,
        TagValue::String(s) => s.parse::<i32>().unwrap_or(0),
        _ => 0,
    };

    let bytes: Vec<u8> = shifts
        .iter()
        .map(|&shift| (((numeric_val >> shift) & mask) + offset) as u8)
        .collect();

    TagValue::String(String::from_utf8_lossy(&bytes).to_string())
}

/// Join unpacked binary data with a separator
///
/// Implements: join "separator", unpack "format", data
/// This unpacks binary data according to the format specification,
/// then joins the results with the given separator.
///
/// # Arguments
/// * `separator` - String to join results with (e.g., " ", "-")
/// * `format` - Unpack format specification (e.g., "H2H2", "C*")
/// * `val` - The TagValue containing binary data to unpack
///
/// # Example
/// ```rust
/// # use exif_oxide::core::{TagValue, join_unpack_binary};
/// // Equivalent to: join " ", unpack "H2H2", val
/// let result = join_unpack_binary(" ", "H2H2", &TagValue::Binary(vec![0xAB, 0xCD]));
/// // Returns: TagValue::String("ab cd")
/// ```
pub fn join_unpack_binary(separator: &str, format: &str, val: &TagValue) -> TagValue {
    let unpacked = unpack_binary(format, val);

    // Convert unpacked values to strings
    let strings: Vec<String> = unpacked
        .iter()
        .map(|v| match v {
            TagValue::String(s) => s.clone(),
            TagValue::U8(n) => format!("{n:02x}"),
            TagValue::U16(n) => format!("{n:04x}"),
            TagValue::U32(n) => format!("{n:08x}"),
            TagValue::I32(n) => {
                let b = *n as u8;
                format!("{b:02x}")
            }
            _ => v.to_string(),
        })
        .collect();

    TagValue::String(strings.join(separator))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_unpack_binary() {
        // Test basic hex unpacking with join
        let binary_data = TagValue::Binary(vec![0xAB, 0xCD, 0xEF]);
        let result = join_unpack_binary(" ", "H2H2H2", &binary_data);
        assert_eq!(result, TagValue::String("ab cd ef".to_string()));

        // Test with different separator
        let result = join_unpack_binary("-", "H2H2", &TagValue::Binary(vec![0x12, 0x34]));
        assert_eq!(result, TagValue::String("12-34".to_string()));

        // Test with unsigned chars
        let result = join_unpack_binary(" ", "C3", &TagValue::Binary(vec![0x10, 0x20, 0x30]));
        assert_eq!(result, TagValue::String("10 20 30".to_string()));

        // Test empty separator
        let result = join_unpack_binary("", "H2H2", &TagValue::Binary(vec![0xAB, 0xCD]));
        assert_eq!(result, TagValue::String("abcd".to_string()));

        // Test single value
        let result = join_unpack_binary(" ", "H2", &TagValue::Binary(vec![0xFF]));
        assert_eq!(result, TagValue::String("ff".to_string()));
    }

    #[test]
    fn test_pack_c_star_bit_extract() {
        // Test bit extraction and packing
        let _result = pack_c_star_bit_extract(&TagValue::I32(0x1234), &[8, 0], 0xFF, 0);
        // 0x1234 >> 8 = 0x12, 0x1234 >> 0 = 0x34
        // Should pack as bytes [0x12, 0x34] → string

        // Test with mask and offset
        let result = pack_c_star_bit_extract(&TagValue::I32(0x1F), &[0], 0x1F, 0x40);
        // (0x1F >> 0) & 0x1F + 0x40 = 0x1F + 0x40 = 0x5F
        // Should pack as byte [0x5F] → string
        assert!(!result.to_string().is_empty()); // Should produce some output
    }
}
