//! String measurement functions for Rust code generation
//!
//! This module provides functions for measuring string properties like length,
//! following Perl's exact behavior for compatibility with ExifTool expressions.

use crate::core::TagValue;

/// Perl length() function - returns the length of a string
///
/// In Perl, length() returns the number of characters in a string.
/// For non-string values, Perl converts them to strings first.
///
/// # Arguments
/// * `val` - The TagValue to get the length of
///
/// # Returns
/// * For PrintConv: TagValue::String containing the length as a string
/// * For ValueConv: TagValue::I32 containing the length as integer
///
/// # Example
/// ```rust
/// # use exif_oxide::core::{TagValue, length_string, length_i32};
/// assert_eq!(length_string(TagValue::String("hello".to_string())), TagValue::String("5".to_string()));
/// assert_eq!(length_i32(TagValue::String("hello".to_string())), TagValue::I32(5));
/// ```
pub fn length_string(val: TagValue) -> TagValue {
    TagValue::String(match val {
        TagValue::String(s) => s.len().to_string(),
        _ => val.to_string().len().to_string(),
    })
}

/// Perl length() function returning integer (for ValueConv)
pub fn length_i32(val: TagValue) -> TagValue {
    TagValue::I32(match val {
        TagValue::String(s) => s.len() as i32,
        _ => val.to_string().len() as i32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length_functions() {
        // String length
        assert_eq!(
            length_string(TagValue::String("hello".to_string())),
            TagValue::String("5".to_string())
        );
        assert_eq!(
            length_i32(TagValue::String("hello".to_string())),
            TagValue::I32(5)
        );

        // Empty string
        assert_eq!(
            length_string(TagValue::String("".to_string())),
            TagValue::String("0".to_string())
        );

        // Non-string values get converted to string first
        assert_eq!(
            length_string(TagValue::I32(123)),
            TagValue::String("3".to_string()) // "123" has length 3
        );
    }
}
