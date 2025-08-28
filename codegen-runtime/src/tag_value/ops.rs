//! Arithmetic operators for TagValue
//!
//! Implements std::ops traits to enable arithmetic operations on TagValue.
//! Follows ExifTool's type coercion rules for numeric operations.

use super::TagValue;
use std::ops::{Add, Div, Mul, Sub};

// Helper macro to implement arithmetic ops for TagValue
macro_rules! impl_arithmetic_op {
    ($trait:ident, $method:ident, $op:tt) => {
        impl $trait for &TagValue {
            type Output = TagValue;

            fn $method(self, rhs: Self) -> Self::Output {
                match (self, rhs) {
                    // Both are integers - preserve integer type
                    (TagValue::U8(a), TagValue::U8(b)) => TagValue::U8(a $op b),
                    (TagValue::U16(a), TagValue::U16(b)) => TagValue::U16(a $op b),
                    (TagValue::U32(a), TagValue::U32(b)) => TagValue::U32(a $op b),
                    (TagValue::I32(a), TagValue::I32(b)) => TagValue::I32(a $op b),

                    // At least one is float - result is float
                    (TagValue::F64(a), TagValue::F64(b)) => TagValue::F64(a $op b),
                    (TagValue::F64(a), TagValue::U32(b)) => TagValue::F64(a $op (*b as f64)),
                    (TagValue::U32(a), TagValue::F64(b)) => TagValue::F64((*a as f64) $op b),
                    (TagValue::F64(a), TagValue::U16(b)) => TagValue::F64(a $op (*b as f64)),
                    (TagValue::U16(a), TagValue::F64(b)) => TagValue::F64((*a as f64) $op b),
                    (TagValue::F64(a), TagValue::U8(b)) => TagValue::F64(a $op (*b as f64)),
                    (TagValue::U8(a), TagValue::F64(b)) => TagValue::F64((*a as f64) $op b),
                    (TagValue::F64(a), TagValue::I32(b)) => TagValue::F64(a $op (*b as f64)),
                    (TagValue::I32(a), TagValue::F64(b)) => TagValue::F64((*a as f64) $op b),

                    // Mixed integer types - promote to larger type
                    (TagValue::U32(a), TagValue::U16(b)) => TagValue::U32(a $op (*b as u32)),
                    (TagValue::U16(a), TagValue::U32(b)) => TagValue::U32((*a as u32) $op b),
                    (TagValue::U32(a), TagValue::U8(b)) => TagValue::U32(a $op (*b as u32)),
                    (TagValue::U8(a), TagValue::U32(b)) => TagValue::U32((*a as u32) $op b),
                    (TagValue::U16(a), TagValue::U8(b)) => TagValue::U16(a $op (*b as u16)),
                    (TagValue::U8(a), TagValue::U16(b)) => TagValue::U16((*a as u16) $op b),

                    // I32 mixed with unsigned - convert to I32
                    (TagValue::I32(a), TagValue::U32(b)) => TagValue::I32(a $op (*b as i32)),
                    (TagValue::U32(a), TagValue::I32(b)) => TagValue::I32((*a as i32) $op b),
                    (TagValue::I32(a), TagValue::U16(b)) => TagValue::I32(a $op (*b as i32)),
                    (TagValue::U16(a), TagValue::I32(b)) => TagValue::I32((*a as i32) $op b),
                    (TagValue::I32(a), TagValue::U8(b)) => TagValue::I32(a $op (*b as i32)),
                    (TagValue::U8(a), TagValue::I32(b)) => TagValue::I32((*a as i32) $op b),

                    // Default: convert both to f64
                    _ => {
                        let a_val = self.to_numeric().unwrap_or(0.0);
                        let b_val = rhs.to_numeric().unwrap_or(0.0);
                        TagValue::F64(a_val $op b_val)
                    }
                }
            }
        }

        // Implement for integer literals (e.g., val * 2)
        impl $trait<i32> for &TagValue {
            type Output = TagValue;

            fn $method(self, rhs: i32) -> Self::Output {
                match self {
                    TagValue::U8(a) if rhs >= 0 && rhs <= 255 => TagValue::U8(a $op (rhs as u8)),
                    TagValue::U16(a) if rhs >= 0 && rhs <= 65535 => TagValue::U16(a $op (rhs as u16)),
                    TagValue::U32(a) if rhs >= 0 => TagValue::U32(a $op (rhs as u32)),
                    TagValue::I32(a) => TagValue::I32(a $op rhs),
                    TagValue::F64(a) => TagValue::F64(a $op (rhs as f64)),
                    // Type promotion needed
                    TagValue::U8(a) => TagValue::I32((*a as i32) $op rhs),
                    TagValue::U16(a) => TagValue::I32((*a as i32) $op rhs),
                    TagValue::U32(a) => {
                        if rhs < 0 {
                            TagValue::I32((*a as i32) $op rhs)
                        } else {
                            TagValue::U32(a $op (rhs as u32))
                        }
                    }
                    _ => {
                        let val = self.to_numeric().unwrap_or(0.0);
                        TagValue::F64(val $op (rhs as f64))
                    }
                }
            }
        }

        // Implement for float literals (e.g., val * 2.5)
        impl $trait<f64> for &TagValue {
            type Output = TagValue;

            fn $method(self, rhs: f64) -> Self::Output {
                let val = self.to_numeric().unwrap_or(0.0);
                TagValue::F64(val $op rhs)
            }
        }
    };
}

// Implement the four basic arithmetic operations
impl_arithmetic_op!(Add, add, +);
impl_arithmetic_op!(Sub, sub, -);
impl_arithmetic_op!(Mul, mul, *);
impl_arithmetic_op!(Div, div, /);

// Also need to handle literals on the left side (e.g., 2 * val)
impl Mul<&TagValue> for i32 {
    type Output = TagValue;

    fn mul(self, rhs: &TagValue) -> Self::Output {
        rhs * self // Delegate to the implementation above
    }
}

impl Mul<&TagValue> for f64 {
    type Output = TagValue;

    fn mul(self, rhs: &TagValue) -> Self::Output {
        rhs * self
    }
}

impl Add<&TagValue> for i32 {
    type Output = TagValue;

    fn add(self, rhs: &TagValue) -> Self::Output {
        rhs + self
    }
}

impl Add<&TagValue> for f64 {
    type Output = TagValue;

    fn add(self, rhs: &TagValue) -> Self::Output {
        rhs + self
    }
}

impl Sub<&TagValue> for i32 {
    type Output = TagValue;

    fn sub(self, rhs: &TagValue) -> Self::Output {
        // For 0 - val, we want to negate val
        // For other values, we do literal subtraction
        match rhs {
            TagValue::U8(v) => TagValue::I32(self - (*v as i32)),
            TagValue::U16(v) => TagValue::I32(self - (*v as i32)),
            TagValue::U32(v) => TagValue::I32(self - (*v as i32)),
            TagValue::I32(v) => TagValue::I32(self - v),
            TagValue::F64(v) => TagValue::F64((self as f64) - v),
            _ => {
                let val = rhs.to_numeric().unwrap_or(0.0);
                TagValue::F64((self as f64) - val)
            }
        }
    }
}

impl Sub<&TagValue> for f64 {
    type Output = TagValue;

    fn sub(self, rhs: &TagValue) -> Self::Output {
        let val = rhs.to_numeric().unwrap_or(0.0);
        TagValue::F64(self - val)
    }
}

// Implement owned TagValue operations (TagValue op i32) by delegating to borrowed operations
impl Add<i32> for TagValue {
    type Output = TagValue;

    fn add(self, rhs: i32) -> Self::Output {
        (&self) + rhs
    }
}

impl Sub<i32> for TagValue {
    type Output = TagValue;

    fn sub(self, rhs: i32) -> Self::Output {
        (&self) - rhs
    }
}

impl Mul<i32> for TagValue {
    type Output = TagValue;

    fn mul(self, rhs: i32) -> Self::Output {
        (&self) * rhs
    }
}

impl Div<i32> for TagValue {
    type Output = TagValue;

    fn div(self, rhs: i32) -> Self::Output {
        (&self) / rhs
    }
}

impl Add<f64> for TagValue {
    type Output = TagValue;

    fn add(self, rhs: f64) -> Self::Output {
        (&self) + rhs
    }
}

impl Sub<f64> for TagValue {
    type Output = TagValue;

    fn sub(self, rhs: f64) -> Self::Output {
        (&self) - rhs
    }
}

impl Mul<f64> for TagValue {
    type Output = TagValue;

    fn mul(self, rhs: f64) -> Self::Output {
        (&self) * rhs
    }
}

impl Div<f64> for TagValue {
    type Output = TagValue;

    fn div(self, rhs: f64) -> Self::Output {
        (&self) / rhs
    }
}

// Helper methods for TagValue
impl TagValue {
    /// Convert to numeric value (f64) if possible
    pub fn to_numeric(&self) -> Option<f64> {
        match self {
            TagValue::U8(v) => Some(*v as f64),
            TagValue::U16(v) => Some(*v as f64),
            TagValue::U32(v) => Some(*v as f64),
            TagValue::U64(v) => Some(*v as f64),
            TagValue::I16(v) => Some(*v as f64),
            TagValue::I32(v) => Some(*v as f64),
            TagValue::F64(v) => Some(*v),
            TagValue::Rational(n, d) if *d != 0 => Some(*n as f64 / *d as f64),
            TagValue::SRational(n, d) if *d != 0 => Some(*n as f64 / *d as f64),
            TagValue::String(s) => s.parse::<f64>().ok(),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiply_tagvalue_by_integer() {
        let val = TagValue::U32(10);
        let result = &val * 2;
        assert_eq!(result, TagValue::U32(20));
    }

    #[test]
    fn test_multiply_tagvalue_by_float() {
        let val = TagValue::U32(10);
        let result = &val * 2.5;
        assert_eq!(result, TagValue::F64(25.0));
    }

    #[test]
    fn test_add_tagvalues() {
        let a = TagValue::U32(10);
        let b = TagValue::U32(5);
        let result = &a + &b;
        assert_eq!(result, TagValue::U32(15));
    }

    #[test]
    fn test_divide_tagvalues() {
        let val = TagValue::U32(100);
        let result = &val / 4;
        assert_eq!(result, TagValue::U32(25));
    }

    #[test]
    fn test_mixed_types() {
        let a = TagValue::F64(10.5);
        let b = TagValue::U32(2);
        let result = &a * &b;
        assert_eq!(result, TagValue::F64(21.0));
    }
}
