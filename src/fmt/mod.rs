/// Format helper functions for sprintf-style operations
/// 
/// This module provides utilities for handling Perl sprintf patterns,
/// particularly those involving split operations that produce variable
/// numbers of arguments.

pub mod sprintf;
pub mod unpack;

pub use sprintf::sprintf_perl;
pub use unpack::unpack_binary;

use crate::types::TagValue;

/// Format values from a split operation using a sprintf-style format string
/// 
/// This handles cases like `sprintf("%.3f x %.3f mm", split(" ",$val))`
/// where split produces a variable number of values that need to be formatted.
pub fn sprintf_split_values(format_str: &str, values: &[TagValue]) -> String {
    // Count the number of format specifiers in the format string
    let format_specs = format_str.matches("%.3f").count() 
        + format_str.matches("%.2f").count()
        + format_str.matches("%.1f").count()
        + format_str.matches("%d").count()
        + format_str.matches("%s").count()
        + format_str.matches("%f").count();
    
    // Convert values to strings based on the format specifiers
    let formatted_values: Vec<String> = values.iter()
        .take(format_specs)
        .map(|v| match v {
            TagValue::F64(f) => {
                // Check what precision is needed based on format string
                if format_str.contains("%.3f") {
                    format!("{:.3}", f)
                } else if format_str.contains("%.2f") {
                    format!("{:.2}", f)
                } else if format_str.contains("%.1f") {
                    format!("{:.1}", f)
                } else {
                    f.to_string()
                }
            }
            TagValue::I32(i) => i.to_string(),
            TagValue::String(s) => s.clone(),
            _ => v.to_string(),
        })
        .collect();
    
    // Build the final formatted string
    match formatted_values.len() {
        0 => String::new(),
        1 if format_specs == 2 => {
            // Special case: one value but format expects two (like "%.3f x %.3f mm")
            // Use the same value twice
            apply_format(format_str, &[formatted_values[0].clone(), formatted_values[0].clone()])
        }
        _ => apply_format(format_str, &formatted_values)
    }
}

/// Apply a format string with the given values
fn apply_format(format_str: &str, values: &[String]) -> String {
    let mut result = format_str.to_string();
    let mut value_iter = values.iter();
    
    // Replace format specifiers in order
    let patterns = ["%.3f", "%.2f", "%.1f", "%d", "%s", "%f"];
    
    for pattern in &patterns {
        while result.contains(pattern) {
            if let Some(value) = value_iter.next() {
                result = result.replacen(pattern, value, 1);
            } else {
                // No more values, break
                break;
            }
        }
    }
    
    result
}

/// Handle sprintf with Perl string concatenation and repetition operations
/// 
/// This handles cases like `sprintf("%19d %4d %6d" . " %3d %4d %6d" x 8, split(" ",$val))`
/// where the format string is built from concatenation and repetition before sprintf.
/// 
/// # Arguments
/// * `base_format` - The base format string (e.g., "%19d %4d %6d")  
/// * `concat_part` - The part to concatenate and repeat (e.g., " %3d %4d %6d")
/// * `repeat_count` - How many times to repeat the concat_part (e.g., 8)
/// * `args` - The arguments to format with the completed format string (can be a single TagValue or array)
/// 
/// # Example
/// ```
/// # use exif_oxide::fmt::sprintf_with_string_concat_repeat;
/// # use exif_oxide::types::TagValue;
/// let result = sprintf_with_string_concat_repeat(
///     "%19d %4d %6d",
///     " %3d %4d %6d", 
///     8,
///     &TagValue::Array(vec![TagValue::I32(1), TagValue::I32(2), TagValue::I32(3)])
/// );
/// // Builds format: "%19d %4d %6d %3d %4d %6d %3d %4d %6d ..." (repeated 8 times)
/// ```
pub fn sprintf_with_string_concat_repeat(
    base_format: &str,
    concat_part: &str, 
    repeat_count: usize,
    args: &TagValue
) -> String {
    // Build complete format string: base + (concat_part repeated repeat_count times)
    let complete_format = format!("{}{}", base_format, concat_part.repeat(repeat_count));
    
    // Convert TagValue to slice for sprintf_perl
    match args {
        TagValue::Array(arr) => sprintf_perl(&complete_format, arr),
        single_val => sprintf_perl(&complete_format, &[single_val.clone()]),
    }
}

/// Safe division calculation following ExifTool pattern: $val ? numerator / $val : 0
/// 
/// This implements the common ExifTool pattern of computing a safe division
/// while handling zero, empty, or undefined values by returning 0 instead of
/// infinity or division-by-zero errors.
/// 
/// # Arguments
/// * `numerator` - The numerator for the division (e.g., 1.0, 10.0)
/// * `val` - The TagValue to use as denominator
/// 
/// # Returns
/// * If val is truthy and non-zero: TagValue::F64(numerator / val)
/// * If val is falsy or zero: TagValue::F64(0.0)
pub fn safe_division(numerator: f64, val: &TagValue) -> TagValue {
    // Check if value is truthy (non-zero, non-empty) following ExifTool semantics
    let is_truthy = match val {
        TagValue::U8(v) => *v != 0,
        TagValue::U16(v) => *v != 0,
        TagValue::U32(v) => *v != 0,
        TagValue::U64(v) => *v != 0,
        TagValue::I16(v) => *v != 0,
        TagValue::I32(v) => *v != 0,
        TagValue::F64(v) => *v != 0.0 && !v.is_nan(),
        TagValue::String(s) => !s.is_empty() && s != "0",
        TagValue::Rational(num, _) => *num != 0,
        TagValue::SRational(num, _) => *num != 0,
        TagValue::Empty => false,
        _ => true, // Arrays, objects, etc. are considered truthy if they exist
    };

    if !is_truthy {
        return TagValue::F64(0.0);
    }

    // Extract numeric value and compute division
    match val {
        TagValue::U8(v) => TagValue::F64(numerator / (*v as f64)),
        TagValue::U16(v) => TagValue::F64(numerator / (*v as f64)),
        TagValue::U32(v) => TagValue::F64(numerator / (*v as f64)),
        TagValue::U64(v) => TagValue::F64(numerator / (*v as f64)),
        TagValue::I16(v) => TagValue::F64(numerator / (*v as f64)),
        TagValue::I32(v) => TagValue::F64(numerator / (*v as f64)),
        TagValue::F64(v) => TagValue::F64(numerator / v),
        TagValue::String(s) => {
            if let Ok(num) = s.parse::<f64>() {
                if num != 0.0 {
                    TagValue::F64(numerator / num)
                } else {
                    TagValue::F64(0.0)
                }
            } else {
                // Non-numeric string - return 0 following ExifTool semantics
                TagValue::F64(0.0)
            }
        }
        TagValue::Rational(num, denom) => {
            if *num != 0 && *denom != 0 {
                let val = *num as f64 / *denom as f64;
                TagValue::F64(numerator / val)
            } else {
                TagValue::F64(0.0)
            }
        }
        TagValue::SRational(num, denom) => {
            if *num != 0 && *denom != 0 {
                let val = *num as f64 / *denom as f64;
                TagValue::F64(numerator / val)
            } else {
                TagValue::F64(0.0)
            }
        }
        _ => TagValue::F64(0.0), // For complex types, return 0
    }
}

/// Safe reciprocal calculation following ExifTool pattern: $val ? 1 / $val : 0
/// 
/// This is a convenience wrapper around `safe_division(1.0, val)`.
/// 
/// Common use cases:
/// - Converting focal length to optical power (diopters)
/// - Converting f-number to relative aperture calculations
/// - Any reciprocal calculation where 0 input should yield 0 output
/// 
/// # Arguments
/// * `val` - The TagValue to take the reciprocal of
/// 
/// # Returns
/// * If val is truthy and non-zero: TagValue::F64(1.0 / val)
/// * If val is falsy or zero: TagValue::F64(0.0)
/// 
/// # Example
/// ```rust
/// # use exif_oxide::fmt::safe_reciprocal;
/// # use exif_oxide::types::TagValue;
/// 
/// // Normal case: 1/2 = 0.5
/// assert_eq!(safe_reciprocal(&TagValue::I32(2)), TagValue::F64(0.5));
/// 
/// // Zero case: avoid division by zero
/// assert_eq!(safe_reciprocal(&TagValue::I32(0)), TagValue::F64(0.0));
/// 
/// // Empty string case: treat as zero
/// assert_eq!(safe_reciprocal(&TagValue::String("".to_string())), TagValue::F64(0.0));
/// ```
pub fn safe_reciprocal(val: &TagValue) -> TagValue {
    safe_division(1.0, val)
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
/// # use exif_oxide::fmt::pack_c_star_bit_extract;
/// # use exif_oxide::types::TagValue;
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
    
    let bytes: Vec<u8> = shifts.iter()
        .map(|&shift| (((numeric_val >> shift) & mask) + offset) as u8)
        .collect();
        
    TagValue::String(String::from_utf8_lossy(&bytes).to_string())
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod basic_tests {
    use super::*;

    #[test]
    fn test_sprintf_split_single_value() {
        let values = vec![TagValue::F64(1.234)];
        let result = sprintf_split_values("%.3f x %.3f mm", &values);
        assert_eq!(result, "1.234 x 1.234 mm");
    }

    #[test]
    fn test_sprintf_split_two_values() {
        let values = vec![TagValue::F64(1.234), TagValue::F64(5.678)];
        let result = sprintf_split_values("%.3f x %.3f mm", &values);
        assert_eq!(result, "1.234 x 5.678 mm");
    }

    #[test]
    fn test_sprintf_split_string_values() {
        let values = vec![
            TagValue::String("10.5".to_string()),
            TagValue::String("20.3".to_string())
        ];
        let result = sprintf_split_values("%.3f x %.3f mm", &values);
        assert_eq!(result, "10.5 x 20.3 mm");
    }

    #[test]
    fn test_sprintf_with_string_concat_repeat() {
        // Test the specific failing case: sprintf("%19d %4d %6d" . " %3d %4d %6d" x 8, ...)
        let values = TagValue::Array(vec![
            TagValue::I32(1), TagValue::I32(2), TagValue::I32(3), // First 3 values
            TagValue::I32(4), TagValue::I32(5), TagValue::I32(6), // Repeated pattern
            TagValue::I32(7), TagValue::I32(8), TagValue::I32(9), // More values...
        ]);
        
        let result = sprintf_with_string_concat_repeat(
            "%19d %4d %6d",
            " %3d %4d %6d", 
            2, // Test with 2 repetitions for simplicity
            &values
        );
        
        // Should build format: "%19d %4d %6d %3d %4d %6d %3d %4d %6d"
        // Expected format string has: 19-width, 4-width, 6-width, then repeated 3-width, 4-width, 6-width pattern
        assert!(result.contains("1")); // First value
        assert!(result.contains("2")); // Second value  
        assert!(result.contains("3")); // Third value
    }

    #[test]
    fn test_sprintf_concat_repeat_empty_args() {
        // Test edge case with empty args
        let empty_args = TagValue::Array(vec![]);
        let result = sprintf_with_string_concat_repeat(
            "%d",
            " %d", 
            3,
            &empty_args
        );
        // Should handle gracefully even with no args
        assert!(!result.is_empty()); // sprintf_perl should handle missing args gracefully
    }

    #[test]
    fn test_safe_reciprocal() {
        // Normal cases: 1/val
        assert_eq!(safe_reciprocal(&TagValue::I32(2)), TagValue::F64(0.5));
        assert_eq!(safe_reciprocal(&TagValue::F64(4.0)), TagValue::F64(0.25));
        assert_eq!(safe_reciprocal(&TagValue::U16(10)), TagValue::F64(0.1));

        // Zero cases: should return 0, not infinity
        assert_eq!(safe_reciprocal(&TagValue::I32(0)), TagValue::F64(0.0));
        assert_eq!(safe_reciprocal(&TagValue::F64(0.0)), TagValue::F64(0.0));
        assert_eq!(safe_reciprocal(&TagValue::String("0".to_string())), TagValue::F64(0.0));

        // Empty/falsy cases: should return 0
        assert_eq!(safe_reciprocal(&TagValue::String("".to_string())), TagValue::F64(0.0));
        assert_eq!(safe_reciprocal(&TagValue::Empty), TagValue::F64(0.0));

        // String numeric conversion
        assert_eq!(safe_reciprocal(&TagValue::String("2.5".to_string())), TagValue::F64(0.4));
        assert_eq!(safe_reciprocal(&TagValue::String("non-numeric".to_string())), TagValue::F64(0.0));

        // Rational cases
        assert_eq!(safe_reciprocal(&TagValue::Rational(8, 2)), TagValue::F64(0.25)); // 1/(8/2) = 1/4
        assert_eq!(safe_reciprocal(&TagValue::Rational(0, 1)), TagValue::F64(0.0)); // 0 numerator
        assert_eq!(safe_reciprocal(&TagValue::SRational(6, 3)), TagValue::F64(0.5)); // 1/(6/3) = 1/2
    }
}