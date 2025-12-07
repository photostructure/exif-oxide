/// Format helper functions for sprintf-style operations
///
/// This module provides utilities for handling Perl sprintf patterns,
/// particularly those involving split operations that produce variable
/// numbers of arguments.
pub mod sprintf;

pub use sprintf::sprintf_perl;

use crate::TagValue;

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
    let formatted_values: Vec<String> = values
        .iter()
        .take(format_specs)
        .map(|v| match v {
            TagValue::F64(f) => {
                // Check what precision is needed based on format string
                if format_str.contains("%.3f") {
                    format!("{f:.3}")
                } else if format_str.contains("%.2f") {
                    format!("{f:.2}")
                } else if format_str.contains("%.1f") {
                    format!("{f:.1}")
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
            apply_format(
                format_str,
                &[formatted_values[0].clone(), formatted_values[0].clone()],
            )
        }
        _ => apply_format(format_str, &formatted_values),
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
/// use codegen_runtime::{TagValue, fmt::sprintf_with_string_concat_repeat};
///
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
    args: &TagValue,
) -> String {
    // Build complete format string: base + (concat_part repeated repeat_count times)
    let complete_format = format!("{}{}", base_format, concat_part.repeat(repeat_count));

    // Convert TagValue to slice for sprintf_perl
    match args {
        TagValue::Array(arr) => sprintf_perl(&complete_format, arr),
        single_val => sprintf_perl(&complete_format, &[single_val.clone()]),
    }
}

/// Conservative fallback for complex Perl expressions that can't be parsed
///
/// When pattern recognition fails, this provides a safe fallback that
/// preserves the original value while generating valid Rust code.
///
/// # Arguments
/// * `original_expression` - The original Perl expression for documentation
/// * `val` - The TagValue to pass through unchanged
///
/// # Returns
/// The original value with a warning comment in debug builds
pub fn conservative_fallback(original_expression: &str, val: &TagValue) -> TagValue {
    #[cfg(debug_assertions)]
    eprintln!(
        "FALLBACK: Complex expression not fully parsed: {}",
        original_expression
    );

    // Conservative approach: return the original value unchanged
    val.clone()
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
            TagValue::String("20.3".to_string()),
        ];
        let result = sprintf_split_values("%.3f x %.3f mm", &values);
        assert_eq!(result, "10.5 x 20.3 mm");
    }

    #[test]
    fn test_sprintf_with_string_concat_repeat() {
        // Test the specific failing case: sprintf("%19d %4d %6d" . " %3d %4d %6d" x 8, ...)
        let values = TagValue::Array(vec![
            TagValue::I32(1),
            TagValue::I32(2),
            TagValue::I32(3), // First 3 values
            TagValue::I32(4),
            TagValue::I32(5),
            TagValue::I32(6), // Repeated pattern
            TagValue::I32(7),
            TagValue::I32(8),
            TagValue::I32(9), // More values...
        ]);

        let result = sprintf_with_string_concat_repeat(
            "%19d %4d %6d",
            " %3d %4d %6d",
            2, // Test with 2 repetitions for simplicity
            &values,
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
        let result = sprintf_with_string_concat_repeat("%d", " %d", 3, &empty_args);
        // Should handle gracefully even with no args
        assert!(!result.is_empty()); // sprintf_perl should handle missing args gracefully
    }

    #[test]
    fn test_conservative_fallback() {
        // Test that fallback preserves original values
        let original = TagValue::String("test".to_string());
        let result = conservative_fallback("complex perl expression", &original);
        assert_eq!(result, original);

        let numeric = TagValue::I32(42);
        let result = conservative_fallback("$complex =~ s/foo/bar/g", &numeric);
        assert_eq!(result, numeric);
    }
}
