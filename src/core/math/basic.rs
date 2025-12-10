//! Basic mathematical functions for Perl-to-Rust code generation
//!
//! This module provides Rust implementations of basic Perl mathematical functions
//! like int(), exp(), log() that are commonly used in ExifTool expressions.

use crate::core::TagValue;

/// Perl exp() function - returns e raised to the power of the argument
///
/// In Perl, exp() converts its argument to a number and returns e^x.
///
/// # Arguments
/// * `val` - Value that can be converted to TagValue
///
/// # Returns
/// TagValue::F64 containing e^val
///
/// # Example
/// ```rust
/// # use exif_oxide::core::{TagValue, exp};
/// let result = exp(TagValue::F64(1.0));
/// let result2 = exp(2i32);  // Also works
/// // Should be approximately e ≈ 2.718281828
/// ```
pub fn exp<T: Into<TagValue>>(val: T) -> TagValue {
    let val = val.into();
    let num = match val {
        TagValue::F64(f) => f,
        TagValue::I32(i) => i as f64,
        TagValue::I16(i) => i as f64,
        TagValue::U8(u) => u as f64,
        TagValue::U16(u) => u as f64,
        TagValue::U32(u) => u as f64,
        TagValue::U64(u) => u as f64,
        TagValue::String(s) => s.parse::<f64>().unwrap_or(0.0),
        TagValue::Rational(num, denom) => {
            if denom != 0 {
                num as f64 / denom as f64
            } else {
                0.0
            }
        }
        TagValue::SRational(num, denom) => {
            if denom != 0 {
                num as f64 / denom as f64
            } else {
                0.0
            }
        }
        TagValue::Empty => 0.0,
        _ => val.to_string().parse::<f64>().unwrap_or(0.0),
    };

    TagValue::F64(num.exp())
}

/// Perl log() function - returns the natural logarithm of the argument
///
/// In Perl, log() converts its argument to a number and returns ln(x).
/// For values <= 0, Perl's behavior varies, but we'll return NaN for safety.
///
/// # Arguments
/// * `val` - Value that can be converted to TagValue
///
/// # Returns
/// TagValue::F64 containing ln(val)
///
/// # Example
/// ```rust
/// # use exif_oxide::core::{TagValue, log};
/// let result = log(TagValue::F64(2.718281828));
/// let result2 = log(2i32);  // Also works
/// // Should be approximately 1.0
/// ```
pub fn log<T: Into<TagValue>>(val: T) -> TagValue {
    let val = val.into();
    let num = match val {
        TagValue::F64(f) => f,
        TagValue::I32(i) => i as f64,
        TagValue::I16(i) => i as f64,
        TagValue::U8(u) => u as f64,
        TagValue::U16(u) => u as f64,
        TagValue::U32(u) => u as f64,
        TagValue::U64(u) => u as f64,
        TagValue::String(s) => s.parse::<f64>().unwrap_or(0.0),
        TagValue::Rational(num, denom) => {
            if denom != 0 {
                num as f64 / denom as f64
            } else {
                0.0
            }
        }
        TagValue::SRational(num, denom) => {
            if denom != 0 {
                num as f64 / denom as f64
            } else {
                0.0
            }
        }
        TagValue::Empty => 0.0,
        _ => val.to_string().parse::<f64>().unwrap_or(0.0),
    };

    if num > 0.0 {
        TagValue::F64(num.ln())
    } else {
        TagValue::F64(f64::NAN)
    }
}

/// Perl int() function - converts a number to integer by truncating towards zero
///
/// In Perl, int() truncates towards zero (not floor), so:
/// - int(3.7) = 3
/// - int(-3.7) = -3
/// - int(0.5) = 0
///
/// This matches Perl's behavior exactly, following the "Trust ExifTool" principle.
///
/// # Arguments
/// * `val` - Value that can be converted to TagValue
///
/// # Returns
/// TagValue containing the truncated integer value
///
/// # Example
/// ```rust
/// # use exif_oxide::core::{TagValue, int};
/// assert_eq!(int(TagValue::F64(3.7)), TagValue::F64(3.0));
/// assert_eq!(int(TagValue::F64(-3.7)), TagValue::F64(-3.0));
/// assert_eq!(int(TagValue::I32(42)), TagValue::F64(42.0));
/// assert_eq!(int(3.7f64), TagValue::F64(3.0));  // Also works with literals
/// ```
pub fn int<T: Into<TagValue>>(val: T) -> TagValue {
    let val = val.into();
    match val {
        TagValue::F64(f) => TagValue::F64(f.trunc()),
        TagValue::U8(n) => TagValue::F64(n as f64),
        TagValue::U16(n) => TagValue::F64(n as f64),
        TagValue::U32(n) => TagValue::F64(n as f64),
        TagValue::U64(n) => TagValue::F64(n as f64),
        TagValue::I16(n) => TagValue::F64(n as f64),
        TagValue::I32(n) => TagValue::F64(n as f64),
        TagValue::String(s) => {
            if let Ok(f) = s.parse::<f64>() {
                TagValue::F64(f.trunc())
            } else {
                // Non-numeric string - Perl int() returns 0
                TagValue::F64(0.0)
            }
        }
        TagValue::Rational(num, denom) => {
            if denom != 0 {
                let f = num as f64 / denom as f64;
                TagValue::F64(f.trunc())
            } else {
                TagValue::F64(0.0)
            }
        }
        TagValue::SRational(num, denom) => {
            if denom != 0 {
                let f = num as f64 / denom as f64;
                TagValue::F64(f.trunc())
            } else {
                TagValue::F64(0.0)
            }
        }
        TagValue::Empty => TagValue::F64(0.0),
        _ => {
            // For complex types, try converting to string then parsing
            if let Ok(f) = val.to_string().parse::<f64>() {
                TagValue::F64(f.trunc())
            } else {
                TagValue::F64(0.0)
            }
        }
    }
}

/// Perl abs() function - absolute value
///
/// Returns the absolute value of a number, following Perl's behavior.
pub fn abs<T: Into<TagValue>>(val: T) -> TagValue {
    let val = val.into();
    match val {
        TagValue::F64(f) => TagValue::F64(f.abs()),
        TagValue::I32(i) => TagValue::F64((i as f64).abs()),
        TagValue::I16(i) => TagValue::F64((i as f64).abs()),
        TagValue::U8(i) => TagValue::F64(i as f64),
        TagValue::U16(i) => TagValue::F64(i as f64),
        TagValue::U32(i) => TagValue::F64(i as f64),
        TagValue::U64(i) => TagValue::F64(i as f64),
        TagValue::String(s) => {
            if let Ok(f) = s.parse::<f64>() {
                TagValue::F64(f.abs())
            } else {
                // Non-numeric string - Perl abs() returns 0
                TagValue::F64(0.0)
            }
        }
        TagValue::Rational(num, denom) => {
            if denom != 0 {
                let f = num as f64 / denom as f64;
                TagValue::F64(f.abs())
            } else {
                TagValue::F64(0.0)
            }
        }
        TagValue::SRational(num, denom) => {
            if denom != 0 {
                let f = num as f64 / denom as f64;
                TagValue::F64(f.abs())
            } else {
                TagValue::F64(0.0)
            }
        }
        TagValue::Empty => TagValue::F64(0.0),
        _ => TagValue::F64(0.0),
    }
}

/// Unary negation operator - returns the negative of a value
///
/// This function handles unary negation (-$val) which is converted by the PPI normalizer
/// from unary minus to a binary operation (0 - $val). This function provides a cleaner
/// implementation that works directly with TagValue types.
///
/// # Arguments
/// * `val` - Value that can be converted to TagValue
///
/// # Returns
/// TagValue containing the negated value
///
/// # Example
/// ```rust
/// # use exif_oxide::core::{TagValue, negate};
/// assert_eq!(negate(TagValue::I32(42)), TagValue::I32(-42));
/// assert_eq!(negate(TagValue::F64(3.14)), TagValue::F64(-3.14));
/// assert_eq!(negate(-5i32), TagValue::I32(5));  // Also works with literals
/// ```
pub fn negate<T: Into<TagValue>>(val: T) -> TagValue {
    let val = val.into();
    match val {
        TagValue::F64(f) => TagValue::F64(-f),
        TagValue::I32(i) => TagValue::I32(-i),
        TagValue::I16(i) => TagValue::I16(-i),
        TagValue::U8(u) => TagValue::I32(-(u as i32)),
        TagValue::U16(u) => TagValue::I32(-(u as i32)),
        TagValue::U32(u) => TagValue::F64(-(u as f64)), // Convert large values to F64
        TagValue::U64(u) => TagValue::F64(-(u as f64)), // Convert large values to F64
        TagValue::String(s) => {
            if let Ok(f) = s.parse::<f64>() {
                TagValue::F64(-f)
            } else {
                // Non-numeric string - Perl negation of non-number gives 0
                TagValue::F64(0.0)
            }
        }
        TagValue::Rational(num, denom) => {
            if denom != 0 {
                TagValue::SRational(-(num as i32), denom as i32)
            } else {
                TagValue::F64(0.0)
            }
        }
        TagValue::SRational(num, denom) => {
            if denom != 0 {
                TagValue::SRational(-num, denom)
            } else {
                TagValue::F64(0.0)
            }
        }
        TagValue::Empty => TagValue::F64(0.0),
        _ => {
            // For complex types, try converting to string then parsing
            if let Ok(f) = val.to_string().parse::<f64>() {
                TagValue::F64(-f)
            } else {
                TagValue::F64(0.0)
            }
        }
    }
}

/// Perl sqrt() function - square root
pub fn sqrt<T: Into<TagValue>>(val: T) -> TagValue {
    let val = val.into();
    let f = match val {
        TagValue::F64(f) => f,
        TagValue::I32(i) => i as f64,
        TagValue::I16(i) => i as f64,
        TagValue::U8(i) => i as f64,
        TagValue::U16(i) => i as f64,
        TagValue::U32(i) => i as f64,
        TagValue::U64(i) => i as f64,
        TagValue::String(s) => s.parse::<f64>().unwrap_or(0.0),
        TagValue::Rational(num, denom) => {
            if denom != 0 {
                num as f64 / denom as f64
            } else {
                0.0
            }
        }
        TagValue::SRational(num, denom) => {
            if denom != 0 {
                num as f64 / denom as f64
            } else {
                0.0
            }
        }
        TagValue::Empty => 0.0,
        _ => 0.0,
    };

    if f < 0.0 {
        // Perl sqrt of negative number throws error, we'll return NaN like Rust
        TagValue::F64(f64::NAN)
    } else {
        TagValue::F64(f.sqrt())
    }
}

/// Perl sin() function - sine
pub fn sin<T: Into<TagValue>>(val: T) -> TagValue {
    let val = val.into();
    let f = match val {
        TagValue::F64(f) => f,
        TagValue::I32(i) => i as f64,
        TagValue::I16(i) => i as f64,
        TagValue::U8(i) => i as f64,
        TagValue::U16(i) => i as f64,
        TagValue::U32(i) => i as f64,
        TagValue::U64(i) => i as f64,
        TagValue::String(s) => s.parse::<f64>().unwrap_or(0.0),
        TagValue::Rational(num, denom) => {
            if denom != 0 {
                num as f64 / denom as f64
            } else {
                0.0
            }
        }
        TagValue::SRational(num, denom) => {
            if denom != 0 {
                num as f64 / denom as f64
            } else {
                0.0
            }
        }
        TagValue::Empty => 0.0,
        _ => 0.0,
    };
    TagValue::F64(f.sin())
}

/// Perl cos() function - cosine
pub fn cos<T: Into<TagValue>>(val: T) -> TagValue {
    let val = val.into();
    let f = match val {
        TagValue::F64(f) => f,
        TagValue::I32(i) => i as f64,
        TagValue::I16(i) => i as f64,
        TagValue::U8(i) => i as f64,
        TagValue::U16(i) => i as f64,
        TagValue::U32(i) => i as f64,
        TagValue::U64(i) => i as f64,
        TagValue::String(s) => s.parse::<f64>().unwrap_or(0.0),
        TagValue::Rational(num, denom) => {
            if denom != 0 {
                num as f64 / denom as f64
            } else {
                0.0
            }
        }
        TagValue::SRational(num, denom) => {
            if denom != 0 {
                num as f64 / denom as f64
            } else {
                0.0
            }
        }
        TagValue::Empty => 0.0,
        _ => 0.0,
    };
    TagValue::F64(f.cos())
}

/// Perl atan2() function - arctangent of y/x
pub fn atan2<T: Into<TagValue>>(y: T, x: T) -> TagValue {
    let y_val = y.into();
    let x_val = x.into();

    let y_f = match y_val {
        TagValue::F64(f) => f,
        TagValue::I32(i) => i as f64,
        TagValue::I16(i) => i as f64,
        TagValue::U8(i) => i as f64,
        TagValue::U16(i) => i as f64,
        TagValue::U32(i) => i as f64,
        TagValue::U64(i) => i as f64,
        TagValue::String(s) => s.parse::<f64>().unwrap_or(0.0),
        TagValue::Rational(num, denom) => {
            if denom != 0 {
                num as f64 / denom as f64
            } else {
                0.0
            }
        }
        TagValue::SRational(num, denom) => {
            if denom != 0 {
                num as f64 / denom as f64
            } else {
                0.0
            }
        }
        TagValue::Empty => 0.0,
        _ => 0.0,
    };

    let x_f = match x_val {
        TagValue::F64(f) => f,
        TagValue::I32(i) => i as f64,
        TagValue::I16(i) => i as f64,
        TagValue::U8(i) => i as f64,
        TagValue::U16(i) => i as f64,
        TagValue::U32(i) => i as f64,
        TagValue::U64(i) => i as f64,
        TagValue::String(s) => s.parse::<f64>().unwrap_or(0.0),
        TagValue::Rational(num, denom) => {
            if denom != 0 {
                num as f64 / denom as f64
            } else {
                0.0
            }
        }
        TagValue::SRational(num, denom) => {
            if denom != 0 {
                num as f64 / denom as f64
            } else {
                0.0
            }
        }
        TagValue::Empty => 0.0,
        _ => 0.0,
    };

    TagValue::F64(y_f.atan2(x_f))
}

/// Check if a value is a floating point number (Perl IsFloat function)
///
/// This checks if the value is stored as or represents a floating point number.
/// In Perl context, this would return true for values that are floats.
///
/// # Arguments
/// * `val` - Value that can be converted to TagValue
///
/// # Returns
/// true if the value is a floating point number, false otherwise
#[allow(non_snake_case)] // Matches ExifTool's Image::ExifTool::IsFloat function
pub fn IsFloat<T: Into<TagValue>>(val: T) -> bool {
    let val = val.into();
    match val {
        TagValue::F64(_) => true,
        TagValue::Rational(_, _) => true,
        TagValue::SRational(_, _) => true,
        TagValue::F64Array(_) => true,
        TagValue::RationalArray(_) => true,
        TagValue::SRationalArray(_) => true,
        TagValue::String(s) => {
            // Check if the string represents a float
            s.parse::<f64>().is_ok() && s.contains('.')
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_function() {
        // Positive floats - truncate towards zero
        assert_eq!(int(TagValue::F64(3.7)), TagValue::F64(3.0));
        assert_eq!(int(TagValue::F64(3.2)), TagValue::F64(3.0));
        assert_eq!(int(TagValue::F64(0.9)), TagValue::F64(0.0));

        // Negative floats - truncate towards zero (not floor)
        assert_eq!(int(TagValue::F64(-3.7)), TagValue::F64(-3.0));
        assert_eq!(int(TagValue::F64(-3.2)), TagValue::F64(-3.0));
        assert_eq!(int(TagValue::F64(-0.9)), TagValue::F64(0.0));

        // Integers should convert to F64
        assert_eq!(int(TagValue::I32(42)), TagValue::F64(42.0));
        assert_eq!(int(TagValue::I32(-42)), TagValue::F64(-42.0));
        assert_eq!(int(TagValue::U16(100)), TagValue::F64(100.0));

        // String parsing
        assert_eq!(int(TagValue::String("3.7".to_string())), TagValue::F64(3.0));
        assert_eq!(
            int(TagValue::String("-2.9".to_string())),
            TagValue::F64(-2.0)
        );
        assert_eq!(int(TagValue::String("42".to_string())), TagValue::F64(42.0));

        // Non-numeric strings return 0 (Perl behavior)
        assert_eq!(
            int(TagValue::String("hello".to_string())),
            TagValue::F64(0.0)
        );
        assert_eq!(int(TagValue::String("".to_string())), TagValue::F64(0.0));

        // Rational numbers
        assert_eq!(int(TagValue::Rational(7, 2)), TagValue::F64(3.0)); // 7/2 = 3.5 -> 3
        assert_eq!(int(TagValue::SRational(-7, 2)), TagValue::F64(-3.0)); // -7/2 = -3.5 -> -3
        assert_eq!(int(TagValue::SRational(5, 2)), TagValue::F64(2.0)); // 5/2 = 2.5 -> 2

        // Edge cases
        assert_eq!(int(TagValue::Empty), TagValue::F64(0.0));
        assert_eq!(int(TagValue::Rational(0, 1)), TagValue::F64(0.0));
        assert_eq!(int(TagValue::Rational(5, 0)), TagValue::F64(0.0)); // Division by zero
    }

    #[test]
    fn test_int_vs_floor() {
        // Verify that int() truncates towards zero, not floor
        // This is the key difference between Perl's int() and floor()

        // Positive numbers: int() and floor() are the same
        assert_eq!(int(TagValue::F64(3.7)), TagValue::F64(3.0));
        // floor(3.7) would also be 3.0

        // Negative numbers: int() and floor() are different
        assert_eq!(int(TagValue::F64(-3.7)), TagValue::F64(-3.0)); // int() truncates towards zero
                                                                   // floor(-3.7) would be -4.0 (rounds down)

        assert_eq!(int(TagValue::F64(-0.5)), TagValue::F64(0.0)); // int() -> 0
                                                                  // floor(-0.5) would be -1.0 (rounds down)
    }

    #[test]
    fn test_exp_function() {
        // exp(0) = 1
        assert_eq!(exp(TagValue::F64(0.0)), TagValue::F64(1.0));

        // exp(1) ≈ e
        let result = exp(TagValue::F64(1.0));
        if let TagValue::F64(val) = result {
            assert!((val - std::f64::consts::E).abs() < 1e-10);
        } else {
            panic!("Expected F64 result");
        }

        // Integer input
        let result = exp(TagValue::I32(2));
        if let TagValue::F64(val) = result {
            assert!((val - (2.0_f64.exp())).abs() < 1e-10);
        } else {
            panic!("Expected F64 result");
        }
    }

    #[test]
    fn test_log_function() {
        // log(1) = 0
        assert_eq!(log(TagValue::F64(1.0)), TagValue::F64(0.0));

        // log(e) ≈ 1
        let result = log(TagValue::F64(std::f64::consts::E));
        if let TagValue::F64(val) = result {
            assert!((val - 1.0).abs() < 1e-10);
        } else {
            panic!("Expected F64 result");
        }

        // log(2) ≈ ln(2)
        let result = log(TagValue::F64(2.0));
        if let TagValue::F64(val) = result {
            assert!((val - 2.0_f64.ln()).abs() < 1e-10);
        } else {
            panic!("Expected F64 result");
        }

        // log(0) or negative values return NaN
        let result = log(TagValue::F64(0.0));
        if let TagValue::F64(val) = result {
            assert!(val.is_nan());
        } else {
            panic!("Expected F64 result");
        }
    }

    #[test]
    fn test_log_with_generic_input() {
        // Test log() with integer literals (what was failing before)
        let result = log(2i32);
        if let TagValue::F64(val) = result {
            assert!((val - 2.0_f64.ln()).abs() < 1e-10);
        } else {
            panic!("Expected F64 result");
        }

        // Test log() with f64 literals
        let result = log(2.0f64);
        if let TagValue::F64(val) = result {
            assert!((val - 2.0_f64.ln()).abs() < 1e-10);
        } else {
            panic!("Expected F64 result");
        }
    }

    #[test]
    fn test_exp_with_generic_input() {
        // Test exp() with integer literals (what was failing before)
        let result = exp(2i32);
        if let TagValue::F64(val) = result {
            assert!((val - (2.0_f64.exp())).abs() < 1e-10);
        } else {
            panic!("Expected F64 result");
        }

        // Test exp() with f64 literals
        let result = exp(1.0f64);
        if let TagValue::F64(val) = result {
            assert!((val - std::f64::consts::E).abs() < 1e-10);
        } else {
            panic!("Expected F64 result");
        }
    }

    #[test]
    fn test_int_with_generic_input() {
        // Test int() with f64 literals
        assert_eq!(int(3.7f64), TagValue::F64(3.0));
        assert_eq!(int(-3.7f64), TagValue::F64(-3.0));

        // Test int() with i32 literals
        assert_eq!(int(42i32), TagValue::F64(42.0));
        assert_eq!(int(-42i32), TagValue::F64(-42.0));
    }

    #[test]
    fn test_negate_function() {
        // Test with different numeric types
        assert_eq!(negate(TagValue::I32(42)), TagValue::I32(-42));
        assert_eq!(negate(TagValue::I32(-42)), TagValue::I32(42));
        assert_eq!(negate(TagValue::F64(1.23)), TagValue::F64(-1.23));
        assert_eq!(negate(TagValue::F64(-2.5)), TagValue::F64(2.5));

        // Test with unsigned types (should convert to signed)
        assert_eq!(negate(TagValue::U8(100)), TagValue::I32(-100));
        assert_eq!(negate(TagValue::U16(1000)), TagValue::I32(-1000));
        assert_eq!(negate(TagValue::U32(50000)), TagValue::F64(-50000.0));

        // Test with string parsing
        assert_eq!(
            negate(TagValue::String("42".to_string())),
            TagValue::F64(-42.0)
        );
        assert_eq!(
            negate(TagValue::String("-1.23".to_string())),
            TagValue::F64(1.23)
        );

        // Non-numeric strings return 0
        assert_eq!(
            negate(TagValue::String("hello".to_string())),
            TagValue::F64(0.0)
        );
        assert_eq!(negate(TagValue::String("".to_string())), TagValue::F64(0.0));

        // Test with rational numbers
        assert_eq!(negate(TagValue::Rational(3, 4)), TagValue::SRational(-3, 4));
        assert_eq!(
            negate(TagValue::SRational(-5, 2)),
            TagValue::SRational(5, 2)
        );
        assert_eq!(
            negate(TagValue::SRational(7, 3)),
            TagValue::SRational(-7, 3)
        );

        // Edge cases
        assert_eq!(negate(TagValue::Empty), TagValue::F64(0.0));
        assert_eq!(negate(TagValue::Rational(5, 0)), TagValue::F64(0.0)); // Division by zero
    }

    #[test]
    fn test_negate_with_generic_input() {
        // Test negate() with literals (common use case in generated code)
        assert_eq!(negate(42i32), TagValue::I32(-42));
        assert_eq!(negate(-5i32), TagValue::I32(5));
        assert_eq!(negate(1.23f64), TagValue::F64(-1.23));
        assert_eq!(negate(-2.5f64), TagValue::F64(2.5));
    }
}
