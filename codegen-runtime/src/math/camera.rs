//! Camera-specific mathematical functions for ExifTool compatibility
//!
//! These functions implement common photographic calculations found in ExifTool,
//! leveraging the existing TagValue trait system for clean, idiomatic code.

use crate::TagValue;

/// Calculate 2^x for the given expression
///
/// Common pattern: `2**($val/384-1)` becomes `pow2(val / 384i32 - 1i32)`
///
/// # Examples
/// ```
/// use codegen_runtime::{TagValue, math::pow2};
///
/// let val = TagValue::I32(768);
/// let result = pow2(val / 384i32 - 1i32);  // 2^((768/384) - 1) = 2^1 = 2.0
/// ```
pub fn pow2(exponent: TagValue) -> TagValue {
    let exp_f64: f64 = exponent.into();
    TagValue::F64(2.0_f64.powf(exp_f64))
}

/// Calculate aperture values from camera data
///
/// ExifTool pattern: `2**($val/384-1)` for Nikon lens data
pub fn aperture_from_nikon_lens(val: &TagValue) -> TagValue {
    pow2(val / 384i32 - 1i32)
}

/// Calculate ISO values with power-of-2 scaling
///
/// ExifTool pattern: `2**($val/8-6)*100` for various camera ISO calculations
pub fn iso_from_exposure_value(val: &TagValue, base_divisor: i32, offset: i32) -> TagValue {
    pow2(val / base_divisor - offset) * 100i32
}

/// Round value to specified decimal places
///
/// ExifTool pattern: `int($val * 100 + 0.5) / 100` becomes `round_to_places(val, 2)`
pub fn round_to_places(val: &TagValue, places: i32) -> TagValue {
    let scale = TagValue::I32(10_i32.pow(places as u32));
    let scaled = val * &scale + 0.5f64;
    let rounded: f64 = scaled.into();
    TagValue::F64(rounded.trunc()) / scale
}

/// Convert exposure compensation values
///
/// Common pattern for exposure values: `($val - 80) / 12` with power-of-2
pub fn exposure_compensation(val: &TagValue, offset: i32, divisor: i32) -> TagValue {
    pow2((val - offset) / divisor)
}

/// Calculate shutter speed from exposure data
///
/// Pattern: `2**(-$val)` for reciprocal exposure times
pub fn shutter_speed_from_exposure(val: &TagValue) -> TagValue {
    let neg_val = crate::math::negate(val.clone());
    pow2(neg_val)
}

/// Generic power calculation with clean syntax
///
/// Replaces verbose `(Into::<f64>::into(base)).powf(Into::<f64>::into(exp))`
pub fn power(base: TagValue, exponent: TagValue) -> TagValue {
    let base_f64: f64 = base.into();
    let exp_f64: f64 = exponent.into();
    TagValue::F64(base_f64.powf(exp_f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pow2() {
        assert_eq!(pow2(TagValue::I32(0)), TagValue::F64(1.0));
        assert_eq!(pow2(TagValue::I32(3)), TagValue::F64(8.0));
    }

    #[test]
    fn test_aperture_from_nikon_lens() {
        let val = TagValue::I32(768); // 768/384 - 1 = 1, so 2^1 = 2.0
        assert_eq!(aperture_from_nikon_lens(&val), TagValue::F64(2.0));
    }

    #[test]
    fn test_round_to_places() {
        let val = TagValue::F64(3.14159);
        let result = round_to_places(&val, 2);
        if let TagValue::F64(rounded) = result {
            assert!((rounded - 3.14).abs() < 0.01);
        } else {
            panic!("Expected F64 result");
        }
    }

    #[test]
    fn test_power() {
        assert_eq!(
            power(TagValue::I32(2), TagValue::I32(3)),
            TagValue::F64(8.0)
        );
        assert_eq!(
            power(TagValue::F64(4.0), TagValue::F64(0.5)),
            TagValue::F64(2.0)
        );
    }
}
