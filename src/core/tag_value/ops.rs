//! Arithmetic operators for TagValue
//!
//! Implements std::ops traits to enable arithmetic operations on TagValue.
//! Follows ExifTool's type coercion rules for numeric operations.

use crate::core::TagValue;
use std::cmp::Ordering;
use std::ops::{Add, BitAnd, Div, Mul, Neg, Shr, Sub};

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
                    TagValue::U8(a) if (0..=255).contains(&rhs) => TagValue::U8(a $op (rhs as u8)),
                    TagValue::U16(a) if (0..=65535).contains(&rhs) => TagValue::U16(a $op (rhs as u16)),
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

        // Implement for u32 literals (e.g., val * 2u32)
        impl $trait<u32> for &TagValue {
            type Output = TagValue;

            fn $method(self, rhs: u32) -> Self::Output {
                match self {
                    TagValue::U8(a) if rhs <= 255 => TagValue::U8(a $op (rhs as u8)),
                    TagValue::U16(a) if rhs <= 65535 => TagValue::U16(a $op (rhs as u16)),
                    TagValue::U32(a) => TagValue::U32(a $op rhs),
                    TagValue::U64(a) => TagValue::U64(a $op (rhs as u64)),
                    TagValue::I32(a) if rhs <= i32::MAX as u32 => TagValue::I32(a $op (rhs as i32)),
                    TagValue::F64(a) => TagValue::F64(a $op (rhs as f64)),
                    // Type promotion needed
                    TagValue::U8(a) => TagValue::U32((*a as u32) $op rhs),
                    TagValue::U16(a) => TagValue::U32((*a as u32) $op rhs),
                    TagValue::I32(a) => TagValue::F64((*a as f64) $op (rhs as f64)),
                    _ => {
                        let val = self.to_numeric().unwrap_or(0.0);
                        TagValue::F64(val $op (rhs as f64))
                    }
                }
            }
        }

        // Implement for i64 literals (e.g., val / 4294967296i64 for large values)
        impl $trait<i64> for &TagValue {
            type Output = TagValue;

            fn $method(self, rhs: i64) -> Self::Output {
                // For large i64 values, use f64 arithmetic to avoid overflow
                let val = self.to_numeric().unwrap_or(0.0);
                TagValue::F64(val $op (rhs as f64))
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

// Implement the three arithmetic operations that keep integer operands integral.
// Division is deliberately excluded - see `impl_perl_div!` below.
impl_arithmetic_op!(Add, add, +);
impl_arithmetic_op!(Sub, sub, -);
impl_arithmetic_op!(Mul, mul, *);

/// Numify a TagValue the way Perl numifies a scalar in numeric context:
/// numbers keep their value, anything non-numeric becomes 0.
fn numify(value: &TagValue) -> f64 {
    value.to_numeric().unwrap_or(0.0)
}

// Perl's `/` is ALWAYS floating-point division. Integer division exists only
// inside the lexical scope of `use integer`, which ExifTool never enables in a
// conversion expression, so every generated division must keep its fractional
// part: `perl -e 'print 15/10'` prints 1.5, not 1.
// perlop, "Multiplicative Operators".
//
// Perl dies on division by zero; these impls yield infinity/NaN instead of
// panicking, which is what the integer arms this replaced already tried to do
// via explicit zero checks (and what the U32/U32 arm got wrong: it panicked).
macro_rules! impl_perl_div {
    ($rhs:ty, $to_f64:expr) => {
        impl Div<$rhs> for &TagValue {
            type Output = TagValue;

            fn div(self, rhs: $rhs) -> Self::Output {
                #[allow(clippy::redundant_closure_call)]
                TagValue::F64(numify(self) / ($to_f64)(rhs))
            }
        }
    };
}

impl_perl_div!(&TagValue, numify);
impl_perl_div!(i32, |v: i32| v as f64);
impl_perl_div!(u32, |v: u32| v as f64);
impl_perl_div!(i64, |v: i64| v as f64);
impl_perl_div!(f64, |v: f64| v);

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

impl Mul<&TagValue> for u32 {
    type Output = TagValue;

    fn mul(self, rhs: &TagValue) -> Self::Output {
        rhs * self // Delegate to the implementation above
    }
}

impl Add<&TagValue> for u32 {
    type Output = TagValue;

    fn add(self, rhs: &TagValue) -> Self::Output {
        rhs + self
    }
}

impl Sub<&TagValue> for u32 {
    type Output = TagValue;

    fn sub(self, rhs: &TagValue) -> Self::Output {
        match rhs {
            TagValue::U8(v) => TagValue::U32(self - (*v as u32)),
            TagValue::U16(v) => TagValue::U32(self - (*v as u32)),
            TagValue::U32(v) => TagValue::U32(self - v),
            TagValue::U64(v) => TagValue::U64((self as u64) - v),
            TagValue::I32(v) if *v >= 0 => TagValue::U32(self - (*v as u32)),
            TagValue::I32(v) => TagValue::I32((self as i32) - v),
            TagValue::F64(v) => TagValue::F64((self as f64) - v),
            _ => {
                let val = rhs.to_numeric().unwrap_or(0.0);
                TagValue::F64((self as f64) - val)
            }
        }
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

// Operations for i32 with owned TagValue (not just borrowed)
impl Mul<TagValue> for i32 {
    type Output = TagValue;

    fn mul(self, rhs: TagValue) -> Self::Output {
        self * (&rhs)
    }
}

impl Add<TagValue> for i32 {
    type Output = TagValue;

    fn add(self, rhs: TagValue) -> Self::Output {
        self + (&rhs)
    }
}

impl Sub<TagValue> for i32 {
    type Output = TagValue;

    fn sub(self, rhs: TagValue) -> Self::Output {
        self - (&rhs)
    }
}

impl Div<TagValue> for i32 {
    type Output = TagValue;

    fn div(self, rhs: TagValue) -> Self::Output {
        self / (&rhs)
    }
}

// Operations for f64 with owned TagValue (not just borrowed)
impl Mul<TagValue> for f64 {
    type Output = TagValue;

    fn mul(self, rhs: TagValue) -> Self::Output {
        self * (&rhs)
    }
}

impl Add<TagValue> for f64 {
    type Output = TagValue;

    fn add(self, rhs: TagValue) -> Self::Output {
        self + (&rhs)
    }
}

impl Sub<TagValue> for f64 {
    type Output = TagValue;

    fn sub(self, rhs: TagValue) -> Self::Output {
        self - (&rhs)
    }
}

impl Div<TagValue> for f64 {
    type Output = TagValue;

    fn div(self, rhs: TagValue) -> Self::Output {
        self / (&rhs)
    }
}

// Literal-on-the-left division. Same Perl rule: always floating point.
macro_rules! impl_perl_div_lhs {
    ($lhs:ty) => {
        impl Div<&TagValue> for $lhs {
            type Output = TagValue;

            fn div(self, rhs: &TagValue) -> Self::Output {
                TagValue::F64((self as f64) / numify(rhs))
            }
        }
    };
}

impl_perl_div_lhs!(i32);
impl_perl_div_lhs!(u32);
impl_perl_div_lhs!(f64);

// Operations for u32 with owned TagValue (not just borrowed)
impl Mul<TagValue> for u32 {
    type Output = TagValue;

    fn mul(self, rhs: TagValue) -> Self::Output {
        self * (&rhs)
    }
}

impl Add<TagValue> for u32 {
    type Output = TagValue;

    fn add(self, rhs: TagValue) -> Self::Output {
        self + (&rhs)
    }
}

impl Sub<TagValue> for u32 {
    type Output = TagValue;

    fn sub(self, rhs: TagValue) -> Self::Output {
        self - (&rhs)
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

// Implement owned TagValue operations for u32
impl Add<u32> for TagValue {
    type Output = TagValue;

    fn add(self, rhs: u32) -> Self::Output {
        (&self) + rhs
    }
}

impl Sub<u32> for TagValue {
    type Output = TagValue;

    fn sub(self, rhs: u32) -> Self::Output {
        (&self) - rhs
    }
}

impl Mul<u32> for TagValue {
    type Output = TagValue;

    fn mul(self, rhs: u32) -> Self::Output {
        (&self) * rhs
    }
}

impl Div<u32> for TagValue {
    type Output = TagValue;

    fn div(self, rhs: u32) -> Self::Output {
        (&self) / rhs
    }
}

// Implement owned TagValue operations for i64 (for large literals like 4294967296)
impl Add<i64> for TagValue {
    type Output = TagValue;

    fn add(self, rhs: i64) -> Self::Output {
        (&self) + rhs
    }
}

impl Sub<i64> for TagValue {
    type Output = TagValue;

    fn sub(self, rhs: i64) -> Self::Output {
        (&self) - rhs
    }
}

impl Mul<i64> for TagValue {
    type Output = TagValue;

    fn mul(self, rhs: i64) -> Self::Output {
        (&self) * rhs
    }
}

impl Div<i64> for TagValue {
    type Output = TagValue;

    fn div(self, rhs: i64) -> Self::Output {
        (&self) / rhs
    }
}

// Implement TagValue op TagValue (owned values) by delegating to borrowed operations
impl Add for TagValue {
    type Output = TagValue;

    fn add(self, rhs: TagValue) -> Self::Output {
        (&self) + (&rhs)
    }
}

impl Sub for TagValue {
    type Output = TagValue;

    fn sub(self, rhs: TagValue) -> Self::Output {
        (&self) - (&rhs)
    }
}

impl Mul for TagValue {
    type Output = TagValue;

    fn mul(self, rhs: TagValue) -> Self::Output {
        (&self) * (&rhs)
    }
}

impl Div for TagValue {
    type Output = TagValue;

    fn div(self, rhs: TagValue) -> Self::Output {
        (&self) / (&rhs)
    }
}

// Mixed reference combinations for all operations
impl Add<TagValue> for &TagValue {
    type Output = TagValue;

    fn add(self, rhs: TagValue) -> Self::Output {
        self + (&rhs)
    }
}

impl Add<&TagValue> for TagValue {
    type Output = TagValue;

    fn add(self, rhs: &TagValue) -> Self::Output {
        (&self) + rhs
    }
}

impl Sub<TagValue> for &TagValue {
    type Output = TagValue;

    fn sub(self, rhs: TagValue) -> Self::Output {
        self - (&rhs)
    }
}

impl Sub<&TagValue> for TagValue {
    type Output = TagValue;

    fn sub(self, rhs: &TagValue) -> Self::Output {
        (&self) - rhs
    }
}

impl Mul<TagValue> for &TagValue {
    type Output = TagValue;

    fn mul(self, rhs: TagValue) -> Self::Output {
        self * (&rhs)
    }
}

impl Mul<&TagValue> for TagValue {
    type Output = TagValue;

    fn mul(self, rhs: &TagValue) -> Self::Output {
        (&self) * rhs
    }
}

impl Div<TagValue> for &TagValue {
    type Output = TagValue;

    fn div(self, rhs: TagValue) -> Self::Output {
        self / (&rhs)
    }
}

impl Div<&TagValue> for TagValue {
    type Output = TagValue;

    fn div(self, rhs: &TagValue) -> Self::Output {
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

// Comparison operators for TagValue with numeric types
impl PartialEq<i32> for TagValue {
    fn eq(&self, other: &i32) -> bool {
        match self {
            TagValue::I32(v) => *v == *other,
            TagValue::U32(v) if *other >= 0 => *v == (*other as u32),
            _ => {
                if let Some(val) = self.to_numeric() {
                    val == (*other as f64)
                } else {
                    false
                }
            }
        }
    }
}

impl PartialEq<u32> for TagValue {
    fn eq(&self, other: &u32) -> bool {
        match self {
            TagValue::U32(v) => *v == *other,
            TagValue::I32(v) if *v >= 0 => (*v as u32) == *other,
            _ => {
                if let Some(val) = self.to_numeric() {
                    val == (*other as f64)
                } else {
                    false
                }
            }
        }
    }
}

impl PartialEq<f64> for TagValue {
    fn eq(&self, other: &f64) -> bool {
        self.to_numeric().map(|val| val == *other).unwrap_or(false)
    }
}

impl PartialEq<i64> for TagValue {
    fn eq(&self, other: &i64) -> bool {
        // For large i64 values (outside i32 range), compare as f64
        if let Some(val) = self.to_numeric() {
            val == (*other as f64)
        } else {
            false
        }
    }
}

impl PartialOrd<i32> for TagValue {
    fn partial_cmp(&self, other: &i32) -> Option<Ordering> {
        match self {
            TagValue::I32(v) => v.partial_cmp(other),
            TagValue::U32(v) if *other >= 0 => (*v as f64).partial_cmp(&(*other as f64)),
            _ => {
                if let Some(val) = self.to_numeric() {
                    val.partial_cmp(&(*other as f64))
                } else {
                    None
                }
            }
        }
    }
}

impl PartialOrd<u32> for TagValue {
    fn partial_cmp(&self, other: &u32) -> Option<Ordering> {
        match self {
            TagValue::U32(v) => v.partial_cmp(other),
            TagValue::I32(v) if *v >= 0 => (*v as u32).partial_cmp(other),
            _ => {
                if let Some(val) = self.to_numeric() {
                    val.partial_cmp(&(*other as f64))
                } else {
                    None
                }
            }
        }
    }
}

// Also implement for borrowed TagValue
impl PartialEq<i32> for &TagValue {
    fn eq(&self, other: &i32) -> bool {
        (*self).eq(other)
    }
}

impl PartialEq<u32> for &TagValue {
    fn eq(&self, other: &u32) -> bool {
        (*self).eq(other)
    }
}

impl PartialEq<f64> for &TagValue {
    fn eq(&self, other: &f64) -> bool {
        (*self).eq(other)
    }
}

impl PartialEq<i64> for &TagValue {
    fn eq(&self, other: &i64) -> bool {
        (*self).eq(other)
    }
}

impl PartialOrd<i32> for &TagValue {
    fn partial_cmp(&self, other: &i32) -> Option<Ordering> {
        (*self).partial_cmp(other)
    }
}

impl PartialOrd<u32> for &TagValue {
    fn partial_cmp(&self, other: &u32) -> Option<Ordering> {
        (*self).partial_cmp(other)
    }
}

impl PartialOrd<f64> for TagValue {
    fn partial_cmp(&self, other: &f64) -> Option<Ordering> {
        self.to_numeric().and_then(|val| val.partial_cmp(other))
    }
}

impl PartialOrd<f64> for &TagValue {
    fn partial_cmp(&self, other: &f64) -> Option<Ordering> {
        (*self).partial_cmp(other)
    }
}

// Bitwise operations for TagValue
impl BitAnd<u32> for &TagValue {
    type Output = TagValue;

    fn bitand(self, rhs: u32) -> Self::Output {
        match self {
            TagValue::U8(v) => TagValue::U8(v & (rhs as u8)),
            TagValue::U16(v) => TagValue::U16(v & (rhs as u16)),
            TagValue::U32(v) => TagValue::U32(v & rhs),
            TagValue::U64(v) => TagValue::U64(v & (rhs as u64)),
            TagValue::I32(v) if *v >= 0 => TagValue::U32((*v as u32) & rhs),
            _ => {
                // For other types, convert to integer if possible
                if let Some(val) = self.to_numeric() {
                    let int_val = val as u32;
                    TagValue::U32(int_val & rhs)
                } else {
                    TagValue::U32(0)
                }
            }
        }
    }
}

impl BitAnd<i32> for &TagValue {
    type Output = TagValue;

    fn bitand(self, rhs: i32) -> Self::Output {
        if rhs >= 0 {
            self & (rhs as u32)
        } else {
            // Handle negative values by converting to i32
            match self {
                TagValue::I32(v) => TagValue::I32(v & rhs),
                _ => {
                    if let Some(val) = self.to_numeric() {
                        TagValue::I32((val as i32) & rhs)
                    } else {
                        TagValue::I32(0)
                    }
                }
            }
        }
    }
}

impl Shr<i32> for &TagValue {
    type Output = TagValue;

    fn shr(self, rhs: i32) -> Self::Output {
        if rhs < 0 {
            // Right shift by negative is left shift
            return self.shl(-rhs);
        }

        match self {
            TagValue::U8(v) => TagValue::U8(v >> rhs),
            TagValue::U16(v) => TagValue::U16(v >> rhs),
            TagValue::U32(v) => TagValue::U32(v >> rhs),
            TagValue::U64(v) => TagValue::U64(v >> rhs),
            TagValue::I32(v) => TagValue::I32(v >> rhs),
            _ => {
                if let Some(val) = self.to_numeric() {
                    let int_val = val as u32;
                    TagValue::U32(int_val >> rhs)
                } else {
                    TagValue::U32(0)
                }
            }
        }
    }
}

// Helper for left shift (used by right shift with negative values)
impl TagValue {
    fn shl(&self, rhs: i32) -> TagValue {
        if rhs < 0 {
            return self.shr(-rhs);
        }

        match self {
            TagValue::U8(v) => TagValue::U8(v << rhs),
            TagValue::U16(v) => TagValue::U16(v << rhs),
            TagValue::U32(v) => TagValue::U32(v << rhs),
            TagValue::U64(v) => TagValue::U64(v << rhs),
            TagValue::I32(v) => TagValue::I32(v << rhs),
            _ => {
                if let Some(val) = self.to_numeric() {
                    let int_val = val as u32;
                    TagValue::U32(int_val << rhs)
                } else {
                    TagValue::U32(0)
                }
            }
        }
    }
}

// Implement owned versions by delegating to borrowed ones
impl BitAnd<u32> for TagValue {
    type Output = TagValue;

    fn bitand(self, rhs: u32) -> Self::Output {
        (&self) & rhs
    }
}

impl BitAnd<i32> for TagValue {
    type Output = TagValue;

    fn bitand(self, rhs: i32) -> Self::Output {
        (&self) & rhs
    }
}

impl Shr<i32> for TagValue {
    type Output = TagValue;

    fn shr(self, rhs: i32) -> Self::Output {
        (&self) >> rhs
    }
}

// Unary negation operator
impl Neg for TagValue {
    type Output = TagValue;

    fn neg(self) -> Self::Output {
        match self {
            TagValue::U8(v) => TagValue::I32(-(v as i32)),
            TagValue::U16(v) => TagValue::I32(-(v as i32)),
            TagValue::U32(v) if v <= (i32::MAX as u32) => TagValue::I32(-(v as i32)),
            TagValue::U32(v) => TagValue::F64(-(v as f64)),
            TagValue::U64(v) => TagValue::F64(-(v as f64)),
            TagValue::I16(v) => TagValue::I16(-v),
            TagValue::I32(v) => TagValue::I32(-v),
            TagValue::F64(v) => TagValue::F64(-v),
            TagValue::Rational(n, d) => TagValue::SRational(-(n as i32), d as i32),
            TagValue::SRational(n, d) => TagValue::SRational(-n, d),
            _ => {
                if let Some(val) = self.to_numeric() {
                    TagValue::F64(-val)
                } else {
                    self // Return unchanged for non-numeric types
                }
            }
        }
    }
}

impl Neg for &TagValue {
    type Output = TagValue;

    fn neg(self) -> Self::Output {
        self.clone().neg()
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

    /// Perl's `/` is always floating-point division. Integer division only happens
    /// under a lexical `use integer`, which ExifTool never enables in conversions.
    ///
    /// Probe (perl 5, vendored ExifTool's interpreter):
    ///   $ perl -e 'print 15/10'   -> 1.5
    ///   $ perl -e 'print 100/4'   -> 25
    ///   $ perl -e 'print 4/1'     -> 4
    ///
    /// perlop, "Multiplicative Operators": binary `/` divides two numbers.
    #[test]
    fn test_divide_is_always_floating_point() {
        // The exact case from Canon SelfTimer (Canon.pm:2231): ($val & 0xfff) / 10
        assert_eq!(&TagValue::I32(15) / &TagValue::I32(10), TagValue::F64(1.5));
        assert_eq!(&TagValue::U16(15) / &TagValue::U16(10), TagValue::F64(1.5));
        assert_eq!(&TagValue::U32(15) / &TagValue::U32(10), TagValue::F64(1.5));
        assert_eq!(&TagValue::U8(15) / &TagValue::U8(10), TagValue::F64(1.5));

        // Integer literal divisors take the same path.
        assert_eq!(&TagValue::U16(15) / 10i32, TagValue::F64(1.5));
        assert_eq!(&TagValue::U32(15) / 10u32, TagValue::F64(1.5));
        assert_eq!(&TagValue::I32(15) / 10i64, TagValue::F64(1.5));

        // Literal on the left divided by a TagValue.
        assert_eq!(1i32 / &TagValue::I32(2), TagValue::F64(0.5));
        assert_eq!(1u32 / &TagValue::U32(2), TagValue::F64(0.5));
        assert_eq!(1i32 / TagValue::I32(2), TagValue::F64(0.5));

        // Exact quotients still come back as F64 (Perl has one numeric type here);
        // it is the *serialization* that renders 25.0 as "25", not the arithmetic.
        assert_eq!(&TagValue::U32(100) / 4, TagValue::F64(25.0));
        assert_eq!(&TagValue::U32(100) / &TagValue::U32(4), TagValue::F64(25.0));
    }

    /// Perl's `+`, `-` and `*` on integer-valued operands stay integer-valued, so
    /// only `/` changes representation. Guards against over-correcting the fix.
    #[test]
    fn test_non_division_ops_keep_integer_variants() {
        assert_eq!(&TagValue::U32(10) + &TagValue::U32(5), TagValue::U32(15));
        assert_eq!(&TagValue::U32(10) - &TagValue::U32(5), TagValue::U32(5));
        assert_eq!(&TagValue::U32(10) * &TagValue::U32(5), TagValue::U32(50));
    }

    /// Rational division routes through `to_numeric()` and was already float;
    /// confirm the fix leaves it alone.
    #[test]
    fn test_rational_division_unchanged() {
        // 1/2 divided by 1/4 == 2
        let a = TagValue::Rational(1, 2);
        let b = TagValue::Rational(1, 4);
        assert_eq!(&a / &b, TagValue::F64(2.0));
        assert_eq!(&TagValue::Rational(1, 2) / 2, TagValue::F64(0.25));
    }

    /// Integer division by zero used to panic in the integer arms - reachable from
    /// PanasonicRaw DistortionScale `1 / (1 + $val/32768)` (PanasonicRaw.pm:460)
    /// at raw -32768, which is generated as
    /// `1i32 / (1i32 + val / 32768i32)` in functions/hash_39.rs.
    ///
    /// Perl dies there ("Illegal division by zero"); ExifTool catches it and drops
    /// the tag. Producing infinity is not that behaviour, but it is a strict
    /// improvement on crashing, and matching ExifTool needs fallible generated
    /// conversions - tracked separately.
    #[test]
    fn test_divide_by_zero_yields_infinity_not_panic() {
        let r = &TagValue::U32(5) / &TagValue::U32(0);
        assert_eq!(r, TagValue::F64(f64::INFINITY));
        let r = &TagValue::I32(5) / 0i32;
        assert_eq!(r, TagValue::F64(f64::INFINITY));
    }

    #[test]
    fn test_mixed_types() {
        let a = TagValue::F64(10.5);
        let b = TagValue::U32(2);
        let result = &a * &b;
        assert_eq!(result, TagValue::F64(21.0));
    }

    #[test]
    fn test_tagvalue_multiply_tagvalue_owned() {
        let a = TagValue::U32(10);
        let b = TagValue::U32(5);
        let result = a * b; // Using owned values, not borrowed
        assert_eq!(result, TagValue::U32(50));
    }

    #[test]
    fn test_tagvalue_multiply_float_owned() {
        let a = TagValue::F64(3.5);
        let b = TagValue::F64(2.0);
        let result = a * b; // Using owned values, not borrowed
        assert_eq!(result, TagValue::F64(7.0));
    }
}
