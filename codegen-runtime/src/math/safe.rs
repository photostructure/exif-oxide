//! Safe mathematical operations for Perl-to-Rust code generation
//!
//! This module provides safe mathematical operations that handle zero, empty,
//! or undefined values by returning safe fallback values instead of errors.

use crate::TagValue;

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
/// # use codegen_runtime::{TagValue, safe_reciprocal};
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_reciprocal() {
        // Normal cases: 1/val
        assert_eq!(safe_reciprocal(&TagValue::I32(2)), TagValue::F64(0.5));
        assert_eq!(safe_reciprocal(&TagValue::F64(4.0)), TagValue::F64(0.25));
        assert_eq!(safe_reciprocal(&TagValue::U16(10)), TagValue::F64(0.1));

        // Zero cases: should return 0, not infinity
        assert_eq!(safe_reciprocal(&TagValue::I32(0)), TagValue::F64(0.0));
        assert_eq!(safe_reciprocal(&TagValue::F64(0.0)), TagValue::F64(0.0));
        assert_eq!(
            safe_reciprocal(&TagValue::String("0".to_string())),
            TagValue::F64(0.0)
        );

        // Empty/falsy cases: should return 0
        assert_eq!(
            safe_reciprocal(&TagValue::String("".to_string())),
            TagValue::F64(0.0)
        );
        assert_eq!(safe_reciprocal(&TagValue::Empty), TagValue::F64(0.0));

        // String numeric conversion
        assert_eq!(
            safe_reciprocal(&TagValue::String("2.5".to_string())),
            TagValue::F64(0.4)
        );
        assert_eq!(
            safe_reciprocal(&TagValue::String("non-numeric".to_string())),
            TagValue::F64(0.0)
        );

        // Rational cases
        assert_eq!(
            safe_reciprocal(&TagValue::Rational(8, 2)),
            TagValue::F64(0.25)
        ); // 1/(8/2) = 1/4
        assert_eq!(
            safe_reciprocal(&TagValue::Rational(0, 1)),
            TagValue::F64(0.0)
        ); // 0 numerator
        assert_eq!(
            safe_reciprocal(&TagValue::SRational(6, 3)),
            TagValue::F64(0.5)
        ); // 1/(6/3) = 1/2
    }

    #[test]
    fn test_safe_division() {
        // Normal cases: numerator/val
        assert_eq!(safe_division(10.0, &TagValue::I32(2)), TagValue::F64(5.0)); // 10/2 = 5
        assert_eq!(safe_division(3.0, &TagValue::F64(1.5)), TagValue::F64(2.0)); // 3/1.5 = 2
        assert_eq!(
            safe_division(100.0, &TagValue::U16(10)),
            TagValue::F64(10.0)
        ); // 100/10 = 10

        // Zero cases: should return 0, not infinity
        assert_eq!(safe_division(10.0, &TagValue::I32(0)), TagValue::F64(0.0));
        assert_eq!(safe_division(5.0, &TagValue::F64(0.0)), TagValue::F64(0.0));
        assert_eq!(
            safe_division(1.0, &TagValue::String("0".to_string())),
            TagValue::F64(0.0)
        );

        // Empty/falsy cases: should return 0
        assert_eq!(
            safe_division(10.0, &TagValue::String("".to_string())),
            TagValue::F64(0.0)
        );
        assert_eq!(safe_division(100.0, &TagValue::Empty), TagValue::F64(0.0));

        // String numeric conversion
        assert_eq!(
            safe_division(10.0, &TagValue::String("2.5".to_string())),
            TagValue::F64(4.0)
        ); // 10/2.5 = 4
        assert_eq!(
            safe_division(1.0, &TagValue::String("non-numeric".to_string())),
            TagValue::F64(0.0)
        );

        // Rational cases
        assert_eq!(
            safe_division(2.0, &TagValue::Rational(8, 2)),
            TagValue::F64(0.5)
        ); // 2/(8/2) = 2/4 = 0.5
        assert_eq!(
            safe_division(10.0, &TagValue::Rational(0, 1)),
            TagValue::F64(0.0)
        ); // 0 numerator
        assert_eq!(
            safe_division(6.0, &TagValue::SRational(6, 3)),
            TagValue::F64(3.0)
        ); // 6/(6/3) = 6/2 = 3

        // Verify safe_reciprocal is equivalent to safe_division(1.0, ...)
        assert_eq!(
            safe_reciprocal(&TagValue::I32(5)),
            safe_division(1.0, &TagValue::I32(5))
        );
        assert_eq!(
            safe_reciprocal(&TagValue::F64(2.5)),
            safe_division(1.0, &TagValue::F64(2.5))
        );
    }
}