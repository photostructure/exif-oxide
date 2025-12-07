//! String transformation functions for Rust code generation
//!
//! This module provides functions for transforming strings, including
//! regex operations that follow Perl's exact behavior for compatibility
//! with ExifTool expressions.

use crate::TagValue;

/// Simple regex replace function (wrapper around regex_substitute_perl)
///
/// This is a convenience function for simple regex replacements.
/// For full Perl semantics use regex_substitute_perl which returns success status.
///
/// # Arguments
/// * `pattern` - The regex pattern to match
/// * `replacement` - The replacement string
/// * `input` - The input string to operate on
///
/// # Returns
/// String with all matches replaced, or original string if pattern is invalid
pub fn regex_replace(pattern: &str, replacement: &str, input: &str) -> String {
    let (_, result) =
        regex_substitute_perl(pattern, replacement, &TagValue::String(input.to_string()));
    match result {
        TagValue::String(s) => s,
        _ => result.to_string(),
    }
}

/// Regex substitution with Perl semantics: $val =~ s/pattern/replacement/
///
/// In Perl, this operation both modifies the variable AND returns a boolean.
/// This function returns (success: bool, modified_value: TagValue) to capture both semantics.
///
/// # Arguments
/// * `pattern` - The regex pattern to match
/// * `replacement` - The replacement string  
/// * `val` - The TagValue to operate on
///
/// # Returns
/// * `(true, modified_val)` if substitution occurred
/// * `(false, original_val)` if no match found
///
/// # Example
/// ```rust
/// # use codegen_runtime::{TagValue, regex_substitute_perl};
/// let (success, result) = regex_substitute_perl(r" 1$", "", &TagValue::String("123 1".to_string()));
/// assert_eq!(success, true);
/// assert_eq!(result, TagValue::String("123".to_string()));
/// ```
pub fn regex_substitute_perl(pattern: &str, replacement: &str, val: &TagValue) -> (bool, TagValue) {
    let input = val.to_string();

    // Create regex - if pattern is invalid, no substitution occurs
    let regex = match regex::Regex::new(pattern) {
        Ok(r) => r,
        Err(_) => return (false, val.clone()),
    };

    // Check if pattern matches
    if regex.is_match(&input) {
        // Substitution occurred - return modified value
        let modified = regex.replace(&input, replacement).to_string();
        (true, TagValue::String(modified))
    } else {
        // No match - return original value unchanged
        (false, val.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_substitute_perl() {
        // Test successful substitution
        let (success, result) =
            regex_substitute_perl(r" 1$", "", &TagValue::String("123 1".to_string()));
        assert_eq!(success, true);
        assert_eq!(result, TagValue::String("123".to_string()));

        // Test no match
        let (success, result) =
            regex_substitute_perl(r"xyz", "abc", &TagValue::String("hello".to_string()));
        assert_eq!(success, false);
        assert_eq!(result, TagValue::String("hello".to_string()));

        // Test invalid regex pattern
        let (success, result) =
            regex_substitute_perl(r"[", "abc", &TagValue::String("hello".to_string()));
        assert_eq!(success, false);
        assert_eq!(result, TagValue::String("hello".to_string()));
    }

    #[test]
    fn test_regex_substitute_perl_direct() {
        // Debug what's happening with regex_substitute_perl directly
        let (success, result) =
            regex_substitute_perl("123", "X", &TagValue::String("hello 123 world".to_string()));
        assert_eq!(success, true);
        assert_eq!(result, TagValue::String("hello X world".to_string()));
    }

    #[test]
    fn test_regex_replace() {
        // Test successful replacement
        let result = regex_replace(r"123", "X", "hello 123 world");
        assert_eq!(result, "hello X world");

        // Test no match - should return original string
        let result = regex_replace(r"\d+", "X", "hello world");
        assert_eq!(result, "hello world");

        // Test invalid pattern - should return original string
        let result = regex_replace(r"[", "X", "hello world");
        assert_eq!(result, "hello world");
    }
}

/// Convert ASCII value to character (Perl chr function)
///
/// In Perl: chr(65) returns 'A', chr(0) returns null byte
/// Values outside 0-255 are truncated to 8-bit range
///
/// # Arguments
/// * `code_point` - ASCII/byte value to convert to character
///
/// # Returns
/// TagValue::String containing the character (or empty for invalid input)
pub fn chr<T: Into<TagValue>>(code_point: T) -> TagValue {
    let val = code_point.into();

    match val.to_numeric() {
        Some(num) => {
            let byte_val = (num as u32) & 0xFF; // Truncate to 8-bit
            let s = if byte_val <= 127 {
                // ASCII range
                char::from(byte_val as u8).to_string()
            } else {
                // Extended ASCII - convert byte to string
                vec![byte_val as u8].iter().map(|&b| b as char).collect()
            };
            TagValue::String(s)
        }
        None => TagValue::String(String::new()), // Invalid input returns empty string
    }
}

/// Convert string to uppercase (Perl uc function)
///
/// In Perl: uc("hello") returns "HELLO", uc(undef) returns ""
///
/// # Arguments
/// * `input` - String value to convert to uppercase
///
/// # Returns
/// TagValue::String containing uppercase version of the string
pub fn uc<T: Into<TagValue>>(input: T) -> TagValue {
    let val = input.into();

    let s = match val {
        TagValue::String(s) => s.to_uppercase(),
        TagValue::Empty => String::new(),
        _ => val.to_string().to_uppercase(),
    };
    TagValue::String(s)
}
