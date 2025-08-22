//! Arithmetic operations for TagValue with ExifTool-compatible type coercion
//!
//! Task A1: TagValue Arithmetic Operations - Foundation Prerequisite for PPI code generation
//! ExifTool: Implements automatic type coercion for arithmetic expressions like $val / 256
//! P07: PPI Enhancement - see docs/todo/P07-ppi-enhancement.md

use super::TagValue;
use std::ops::{Add, Div, Mul, Sub};

/// Implement division operations for TagValue with ExifTool-compatible type coercion
impl Div<i32> for &TagValue {
    type Output = TagValue;
    
    fn div(self, rhs: i32) -> TagValue {
        match self {
            TagValue::I32(v) => TagValue::I32(v / rhs),
            TagValue::I16(v) => TagValue::I32(*v as i32 / rhs),
            TagValue::U8(v) => TagValue::I32(*v as i32 / rhs),
            TagValue::U16(v) => TagValue::I32(*v as i32 / rhs),
            TagValue::U32(v) => {
                if *v <= i32::MAX as u32 {
                    TagValue::I32(*v as i32 / rhs)
                } else {
                    TagValue::F64(*v as f64 / rhs as f64)
                }
            }
            TagValue::U64(v) => TagValue::F64(*v as f64 / rhs as f64),
            TagValue::F64(v) => TagValue::F64(v / rhs as f64),
            TagValue::String(s) => {
                // ExifTool: Attempt string→number conversion, preserve behavior for non-numeric
                if let Ok(num) = s.parse::<f64>() {
                    TagValue::F64(num / rhs as f64)
                } else {
                    // ExifTool behavior: return string representation of operation
                    TagValue::String(format!("({} / {})", s, rhs))
                }
            }
            TagValue::Rational(num, denom) => {
                if *denom != 0 {
                    TagValue::F64((*num as f64 / *denom as f64) / rhs as f64)
                } else {
                    TagValue::String("inf".to_string())
                }
            }
            TagValue::SRational(num, denom) => {
                if *denom != 0 {
                    TagValue::F64((*num as f64 / *denom as f64) / rhs as f64)
                } else {
                    TagValue::String("inf".to_string())
                }
            }
            // For complex types, return string representation
            _ => TagValue::String(format!("({} / {})", self, rhs)),
        }
    }
}

impl Div<f64> for &TagValue {
    type Output = TagValue;
    
    fn div(self, rhs: f64) -> TagValue {
        match self {
            TagValue::I32(v) => TagValue::F64(*v as f64 / rhs),
            TagValue::I16(v) => TagValue::F64(*v as f64 / rhs),
            TagValue::U8(v) => TagValue::F64(*v as f64 / rhs),
            TagValue::U16(v) => TagValue::F64(*v as f64 / rhs),
            TagValue::U32(v) => TagValue::F64(*v as f64 / rhs),
            TagValue::U64(v) => TagValue::F64(*v as f64 / rhs),
            TagValue::F64(v) => TagValue::F64(v / rhs),
            TagValue::String(s) => {
                if let Ok(num) = s.parse::<f64>() {
                    TagValue::F64(num / rhs)
                } else {
                    TagValue::String(format!("({} / {})", s, rhs))
                }
            }
            TagValue::Rational(num, denom) => {
                if *denom != 0 {
                    TagValue::F64((*num as f64 / *denom as f64) / rhs)
                } else {
                    TagValue::String("inf".to_string())
                }
            }
            TagValue::SRational(num, denom) => {
                if *denom != 0 {
                    TagValue::F64((*num as f64 / *denom as f64) / rhs)
                } else {
                    TagValue::String("inf".to_string())
                }
            }
            _ => TagValue::String(format!("({} / {})", self, rhs)),
        }
    }
}

/// Implement multiplication operations
impl Mul<i32> for &TagValue {
    type Output = TagValue;
    
    fn mul(self, rhs: i32) -> TagValue {
        match self {
            TagValue::I32(v) => TagValue::I32(v * rhs),
            TagValue::I16(v) => TagValue::I32(*v as i32 * rhs),
            TagValue::U8(v) => TagValue::I32(*v as i32 * rhs),
            TagValue::U16(v) => TagValue::I32(*v as i32 * rhs),
            TagValue::U32(v) => {
                if *v <= i32::MAX as u32 {
                    TagValue::I32(*v as i32 * rhs)
                } else {
                    TagValue::F64(*v as f64 * rhs as f64)
                }
            }
            TagValue::U64(v) => TagValue::F64(*v as f64 * rhs as f64),
            TagValue::F64(v) => TagValue::F64(v * rhs as f64),
            TagValue::String(s) => {
                if let Ok(num) = s.parse::<f64>() {
                    TagValue::F64(num * rhs as f64)
                } else {
                    TagValue::String(format!("({} * {})", s, rhs))
                }
            }
            TagValue::Rational(num, denom) => {
                if *denom != 0 {
                    TagValue::F64((*num as f64 / *denom as f64) * rhs as f64)
                } else {
                    TagValue::String("inf".to_string())
                }
            }
            TagValue::SRational(num, denom) => {
                if *denom != 0 {
                    TagValue::F64((*num as f64 / *denom as f64) * rhs as f64)
                } else {
                    TagValue::String("inf".to_string())
                }
            }
            _ => TagValue::String(format!("({} * {})", self, rhs)),
        }
    }
}

impl Mul<f64> for &TagValue {
    type Output = TagValue;
    
    fn mul(self, rhs: f64) -> TagValue {
        match self {
            TagValue::I32(v) => TagValue::F64(*v as f64 * rhs),
            TagValue::I16(v) => TagValue::F64(*v as f64 * rhs),
            TagValue::U8(v) => TagValue::F64(*v as f64 * rhs),
            TagValue::U16(v) => TagValue::F64(*v as f64 * rhs),
            TagValue::U32(v) => TagValue::F64(*v as f64 * rhs),
            TagValue::U64(v) => TagValue::F64(*v as f64 * rhs),
            TagValue::F64(v) => TagValue::F64(v * rhs),
            TagValue::String(s) => {
                if let Ok(num) = s.parse::<f64>() {
                    TagValue::F64(num * rhs)
                } else {
                    TagValue::String(format!("({} * {})", s, rhs))
                }
            }
            TagValue::Rational(num, denom) => {
                if *denom != 0 {
                    TagValue::F64((*num as f64 / *denom as f64) * rhs)
                } else {
                    TagValue::String("inf".to_string())
                }
            }
            TagValue::SRational(num, denom) => {
                if *denom != 0 {
                    TagValue::F64((*num as f64 / *denom as f64) * rhs)
                } else {
                    TagValue::String("inf".to_string())
                }
            }
            _ => TagValue::String(format!("({} * {})", self, rhs)),
        }
    }
}

/// Implement addition operations
impl Add<i32> for &TagValue {
    type Output = TagValue;
    
    fn add(self, rhs: i32) -> TagValue {
        match self {
            TagValue::I32(v) => TagValue::I32(v + rhs),
            TagValue::I16(v) => TagValue::I32(*v as i32 + rhs),
            TagValue::U8(v) => TagValue::I32(*v as i32 + rhs),
            TagValue::U16(v) => TagValue::I32(*v as i32 + rhs),
            TagValue::U32(v) => {
                if *v <= i32::MAX as u32 {
                    TagValue::I32(*v as i32 + rhs)
                } else {
                    TagValue::F64(*v as f64 + rhs as f64)
                }
            }
            TagValue::U64(v) => TagValue::F64(*v as f64 + rhs as f64),
            TagValue::F64(v) => TagValue::F64(v + rhs as f64),
            TagValue::String(s) => {
                if let Ok(num) = s.parse::<f64>() {
                    TagValue::F64(num + rhs as f64)
                } else {
                    TagValue::String(format!("({} + {})", s, rhs))
                }
            }
            TagValue::Rational(num, denom) => {
                if *denom != 0 {
                    TagValue::F64((*num as f64 / *denom as f64) + rhs as f64)
                } else {
                    TagValue::String("inf".to_string())
                }
            }
            TagValue::SRational(num, denom) => {
                if *denom != 0 {
                    TagValue::F64((*num as f64 / *denom as f64) + rhs as f64)
                } else {
                    TagValue::String("inf".to_string())
                }
            }
            _ => TagValue::String(format!("({} + {})", self, rhs)),
        }
    }
}

/// Implement subtraction operations
impl Sub<i32> for &TagValue {
    type Output = TagValue;
    
    fn sub(self, rhs: i32) -> TagValue {
        match self {
            TagValue::I32(v) => TagValue::I32(v - rhs),
            TagValue::I16(v) => TagValue::I32(*v as i32 - rhs),
            TagValue::U8(v) => TagValue::I32(*v as i32 - rhs),
            TagValue::U16(v) => TagValue::I32(*v as i32 - rhs),
            TagValue::U32(v) => {
                if *v <= i32::MAX as u32 {
                    TagValue::I32(*v as i32 - rhs)
                } else {
                    TagValue::F64(*v as f64 - rhs as f64)
                }
            }
            TagValue::U64(v) => TagValue::F64(*v as f64 - rhs as f64),
            TagValue::F64(v) => TagValue::F64(v - rhs as f64),
            TagValue::String(s) => {
                if let Ok(num) = s.parse::<f64>() {
                    TagValue::F64(num - rhs as f64)
                } else {
                    TagValue::String(format!("({} - {})", s, rhs))
                }
            }
            TagValue::Rational(num, denom) => {
                if *denom != 0 {
                    TagValue::F64((*num as f64 / *denom as f64) - rhs as f64)
                } else {
                    TagValue::String("inf".to_string())
                }
            }
            TagValue::SRational(num, denom) => {
                if *denom != 0 {
                    TagValue::F64((*num as f64 / *denom as f64) - rhs as f64)
                } else {
                    TagValue::String("inf".to_string())
                }
            }
            _ => TagValue::String(format!("({} - {})", self, rhs)),
        }
    }
}