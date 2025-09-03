//! String extraction functions for Rust code generation
//!
//! This module provides functions for extracting substrings from strings,
//! following Perl's exact behavior for compatibility with ExifTool expressions.

use crate::TagValue;

/// Perl substr() function - extract substring
///
/// In Perl, substr(string, offset, [length]) extracts a substring.
/// - Negative offsets count from the end
/// - If length is omitted, extract to end of string
/// - If offset is beyond string length, return empty string
///
/// # Arguments
/// * `string` - The source string TagValue
/// * `offset` - Starting position (TagValue that converts to i32)
/// * `length` - Optional length (TagValue that converts to i32)
///
/// # Returns
/// * TagValue::String containing the extracted substring
///
/// # Examples
/// ```rust
/// # use codegen_runtime::{TagValue, substr_2arg, substr_3arg};
/// assert_eq!(
///     substr_3arg(
///         TagValue::String("hello".to_string()),
///         TagValue::I32(1),
///         TagValue::I32(3)
///     ),
///     TagValue::String("ell".to_string())
/// );
/// assert_eq!(
///     substr_2arg(
///         TagValue::String("hello".to_string()),
///         TagValue::I32(-2)
///     ),
///     TagValue::String("lo".to_string())
/// );
/// ```
pub fn substr_2arg<S: Into<TagValue>, O: Into<TagValue>>(string: S, offset: O) -> TagValue {
    let string_val = string.into();
    let offset_val = offset.into();
    let string_str = string_val.to_string();
    let offset_i32 = match offset_val {
        TagValue::I32(i) => i,
        TagValue::String(s) => s.parse::<i32>().unwrap_or(0),
        _ => offset_val.to_string().parse::<i32>().unwrap_or(0),
    };

    let len = string_str.len() as i32;

    // Calculate starting position
    let start = if offset_i32 < 0 {
        // Negative offset counts from end
        std::cmp::max(0, len + offset_i32) as usize
    } else {
        // Positive offset from beginning
        if offset_i32 >= len {
            return TagValue::String("".to_string());
        }
        offset_i32 as usize
    };

    // Extract from start to end
    let result = if start < string_str.len() {
        string_str.chars().skip(start).collect()
    } else {
        "".to_string()
    };

    TagValue::String(result)
}

/// Perl substr() function with 3 arguments - extract substring with length limit
pub fn substr_3arg<S: Into<TagValue>, O: Into<TagValue>, L: Into<TagValue>>(
    string: S,
    offset: O,
    length: L,
) -> TagValue {
    let string_val = string.into();
    let offset_val = offset.into();
    let length_val = length.into();
    let string_str = string_val.to_string();
    let offset_i32 = match offset_val {
        TagValue::I32(i) => i,
        TagValue::String(s) => s.parse::<i32>().unwrap_or(0),
        _ => offset_val.to_string().parse::<i32>().unwrap_or(0),
    };
    let length_i32 = match length_val {
        TagValue::I32(i) => i,
        TagValue::String(s) => s.parse::<i32>().unwrap_or(0),
        _ => length_val.to_string().parse::<i32>().unwrap_or(0),
    };

    if length_i32 <= 0 {
        return TagValue::String("".to_string());
    }

    let len = string_str.len() as i32;

    // Calculate starting position
    let start = if offset_i32 < 0 {
        // Negative offset counts from end
        std::cmp::max(0, len + offset_i32) as usize
    } else {
        // Positive offset from beginning
        if offset_i32 >= len {
            return TagValue::String("".to_string());
        }
        offset_i32 as usize
    };

    // Extract substring with length limit
    let result = if start < string_str.len() {
        string_str
            .chars()
            .skip(start)
            .take(length_i32 as usize)
            .collect()
    } else {
        "".to_string()
    };

    TagValue::String(result)
}

/// Perl index() function - find substring position
///
/// In Perl, index(string, substring, [position]) finds the position of a substring.
/// Returns -1 if not found, similar to Perl's behavior.
///
/// # Arguments
/// * `haystack` - The string to search in
/// * `needle` - The substring to search for
/// * `start_pos` - Optional starting position (default 0)
///
/// # Returns
/// * TagValue::I32 containing the position (0-based) or -1 if not found
pub fn index_2arg<H: Into<TagValue>, N: Into<TagValue>>(haystack: H, needle: N) -> TagValue {
    index_3arg(haystack, needle, 0)
}

/// Perl index() function with 3 arguments
pub fn index_3arg<H: Into<TagValue>, N: Into<TagValue>, S: Into<TagValue>>(
    haystack: H,
    needle: N,
    start_pos: S,
) -> TagValue {
    let haystack_val = haystack.into();
    let needle_val = needle.into();
    let start_pos_val = start_pos.into();
    let haystack_str = haystack_val.to_string();
    let needle_str = needle_val.to_string();
    let start = match start_pos_val {
        TagValue::I32(i) => std::cmp::max(0, i) as usize,
        TagValue::String(s) => s.parse::<i32>().unwrap_or(0).max(0) as usize,
        _ => 0,
    };

    // Handle empty needle - Perl returns 0 for empty string
    if needle_str.is_empty() {
        return TagValue::I32(start.min(haystack_str.len()) as i32);
    }

    // Search from starting position
    if start < haystack_str.len() {
        if let Some(pos) = haystack_str[start..].find(&needle_str) {
            TagValue::I32((start + pos) as i32)
        } else {
            TagValue::I32(-1)
        }
    } else {
        TagValue::I32(-1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substr_2arg() {
        // Basic substring extraction
        assert_eq!(
            substr_2arg(
                TagValue::String("hello world".to_string()),
                TagValue::I32(6)
            ),
            TagValue::String("world".to_string())
        );

        // Negative offset
        assert_eq!(
            substr_2arg(
                TagValue::String("hello world".to_string()),
                TagValue::I32(-5)
            ),
            TagValue::String("world".to_string())
        );

        // Offset beyond string
        assert_eq!(
            substr_2arg(TagValue::String("hello".to_string()), TagValue::I32(10)),
            TagValue::String("".to_string())
        );

        // Offset at beginning
        assert_eq!(
            substr_2arg(TagValue::String("hello".to_string()), TagValue::I32(0)),
            TagValue::String("hello".to_string())
        );
    }

    #[test]
    fn test_substr_3arg() {
        // Basic substring with length
        assert_eq!(
            substr_3arg(
                TagValue::String("hello world".to_string()),
                TagValue::I32(0),
                TagValue::I32(5)
            ),
            TagValue::String("hello".to_string())
        );

        // Negative offset with length
        assert_eq!(
            substr_3arg(
                TagValue::String("hello world".to_string()),
                TagValue::I32(-5),
                TagValue::I32(3)
            ),
            TagValue::String("wor".to_string())
        );

        // Length beyond string end
        assert_eq!(
            substr_3arg(
                TagValue::String("hello".to_string()),
                TagValue::I32(3),
                TagValue::I32(10)
            ),
            TagValue::String("lo".to_string())
        );

        // Zero length
        assert_eq!(
            substr_3arg(
                TagValue::String("hello".to_string()),
                TagValue::I32(0),
                TagValue::I32(0)
            ),
            TagValue::String("".to_string())
        );

        // Negative length
        assert_eq!(
            substr_3arg(
                TagValue::String("hello".to_string()),
                TagValue::I32(0),
                TagValue::I32(-1)
            ),
            TagValue::String("".to_string())
        );
    }

    #[test]
    fn test_index_2arg() {
        // Basic search
        assert_eq!(
            index_2arg(
                TagValue::String("hello world hello".to_string()),
                TagValue::String("hello".to_string())
            ),
            TagValue::I32(0)
        );

        // Not found
        assert_eq!(
            index_2arg(
                TagValue::String("hello world".to_string()),
                TagValue::String("xyz".to_string())
            ),
            TagValue::I32(-1)
        );

        // Empty needle
        assert_eq!(
            index_2arg(
                TagValue::String("hello".to_string()),
                TagValue::String("".to_string())
            ),
            TagValue::I32(0)
        );
    }

    #[test]
    fn test_index_3arg() {
        // Search with starting position
        assert_eq!(
            index_3arg(
                TagValue::String("hello world hello".to_string()),
                TagValue::String("hello".to_string()),
                TagValue::I32(1)
            ),
            TagValue::I32(12)
        );

        // Search single character
        assert_eq!(
            index_3arg(
                TagValue::String("hello world hello".to_string()),
                TagValue::String("o".to_string()),
                TagValue::I32(0)
            ),
            TagValue::I32(4)
        );

        // Starting position beyond string
        assert_eq!(
            index_3arg(
                TagValue::String("hello".to_string()),
                TagValue::String("h".to_string()),
                TagValue::I32(10)
            ),
            TagValue::I32(-1)
        );
    }
}
