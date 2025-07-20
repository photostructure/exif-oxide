//! Type definitions for the expression evaluation system
//!
//! This module contains the core types used throughout the expression
//! evaluation system, including the Expression enum that represents
//! parsed ExifTool expressions and the TagEvaluator trait for
//! type-specific evaluation logic.

use crate::types::TagValue;

/// Structured representation of an expression
///
/// This enum represents parsed ExifTool expressions in a form that can be
/// efficiently evaluated against different contexts.
#[derive(Debug, Clone)]
pub enum Expression {
    /// Check if a field exists
    Exists(String),

    /// Check if a field equals a specific value
    Equals(String, TagValue),

    /// Check if a field is greater than a value
    GreaterThan(String, TagValue),

    /// Check if a field is greater than or equal to a value
    GreaterThanOrEqual(String, TagValue),

    /// Check if a field is less than a value
    LessThan(String, TagValue),

    /// Check if a field is less than or equal to a value
    LessThanOrEqual(String, TagValue),

    /// Check if a field matches a regex pattern
    RegexMatch(String, String),

    /// Check if binary data matches a pattern ($$valPt =~ /pattern/)
    DataPattern(String),

    /// Logical AND of multiple expressions
    And(Vec<Expression>),

    /// Logical OR of multiple expressions
    Or(Vec<Expression>),

    /// Logical NOT of an expression
    Not(Box<Expression>),

    /// Value reference ($val{N})
    /// Used in binary data format expressions
    ValueReference(u32),

    /// Mathematical expression for ceiling division
    /// int(($val{N} + addend) / divisor)
    CeilingDivision {
        val_index: u32,
        addend: usize,
        divisor: usize,
    },
}

/// Trait for evaluating specific tag types
///
/// This trait allows custom evaluation logic for different tag types,
/// enabling sophisticated expression evaluation based on tag semantics.
pub trait TagEvaluator: Send + Sync {
    /// Evaluate a tag value against an expression
    fn evaluate(&self, value: &TagValue, expression: &Expression) -> bool;
}

/// String tag evaluator
pub struct StringEvaluator;
impl TagEvaluator for StringEvaluator {
    fn evaluate(&self, value: &TagValue, expression: &Expression) -> bool {
        if let Some(s) = value.as_string() {
            match expression {
                Expression::Equals(_, expected) => {
                    expected.as_string().map(|e| s == e).unwrap_or(false)
                }
                Expression::RegexMatch(_, pattern) => {
                    // Would need regex compilation here
                    s.contains(pattern) // Simplified for now
                }
                _ => false,
            }
        } else {
            false
        }
    }
}

/// Integer tag evaluator
pub struct IntegerEvaluator;
impl TagEvaluator for IntegerEvaluator {
    fn evaluate(&self, value: &TagValue, expression: &Expression) -> bool {
        if let Some(i) = value
            .as_u32()
            .map(|v| v as i64)
            .or_else(|| value.as_u16().map(|v| v as i64))
        {
            match expression {
                Expression::Equals(_, expected) => expected
                    .as_u32()
                    .map(|e| i == e as i64)
                    .or_else(|| expected.as_u16().map(|e| i == e as i64))
                    .unwrap_or(false),
                Expression::GreaterThan(_, expected) => expected
                    .as_u32()
                    .map(|e| i > e as i64)
                    .or_else(|| expected.as_u16().map(|e| i > e as i64))
                    .unwrap_or(false),
                Expression::LessThan(_, expected) => expected
                    .as_u32()
                    .map(|e| i < e as i64)
                    .or_else(|| expected.as_u16().map(|e| i < e as i64))
                    .unwrap_or(false),
                _ => false,
            }
        } else {
            false
        }
    }
}

/// Float tag evaluator
pub struct FloatEvaluator;
impl TagEvaluator for FloatEvaluator {
    fn evaluate(&self, value: &TagValue, expression: &Expression) -> bool {
        if let Some(f) = value.as_f64() {
            match expression {
                Expression::Equals(_, expected) => expected
                    .as_f64()
                    .map(|e| (f - e).abs() < f64::EPSILON)
                    .unwrap_or(false),
                Expression::GreaterThan(_, expected) => {
                    expected.as_f64().map(|e| f > e).unwrap_or(false)
                }
                Expression::LessThan(_, expected) => {
                    expected.as_f64().map(|e| f < e).unwrap_or(false)
                }
                _ => false,
            }
        } else {
            false
        }
    }
}
