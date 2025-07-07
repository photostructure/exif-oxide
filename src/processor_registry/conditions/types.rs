//! Type definitions for the condition evaluation system
//!
//! This module contains the core types used throughout the condition
//! evaluation system, including the Condition enum that represents
//! parsed condition expressions and the TagEvaluator trait for
//! type-specific evaluation logic.

use crate::types::TagValue;

/// Structured representation of a condition
///
/// This enum represents parsed condition expressions in a form that can be
/// efficiently evaluated against different contexts.
#[derive(Debug, Clone)]
pub enum Condition {
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

    /// Logical AND of multiple conditions
    And(Vec<Condition>),

    /// Logical OR of multiple conditions
    Or(Vec<Condition>),

    /// Logical NOT of a condition
    Not(Box<Condition>),
}

/// Trait for evaluating specific tag types
///
/// This trait allows custom evaluation logic for different tag types,
/// enabling sophisticated condition evaluation based on tag semantics.
pub trait TagEvaluator: Send + Sync {
    /// Evaluate a tag value against a condition
    fn evaluate(&self, value: &TagValue, condition: &Condition) -> bool;
}

/// String tag evaluator
pub struct StringEvaluator;
impl TagEvaluator for StringEvaluator {
    fn evaluate(&self, value: &TagValue, condition: &Condition) -> bool {
        if let Some(s) = value.as_string() {
            match condition {
                Condition::Equals(_, expected) => {
                    expected.as_string().map(|e| s == e).unwrap_or(false)
                }
                Condition::RegexMatch(_, pattern) => {
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
    fn evaluate(&self, value: &TagValue, condition: &Condition) -> bool {
        if let Some(i) = value
            .as_u32()
            .map(|v| v as i64)
            .or_else(|| value.as_u16().map(|v| v as i64))
        {
            match condition {
                Condition::Equals(_, expected) => expected
                    .as_u32()
                    .map(|e| i == e as i64)
                    .or_else(|| expected.as_u16().map(|e| i == e as i64))
                    .unwrap_or(false),
                Condition::GreaterThan(_, expected) => expected
                    .as_u32()
                    .map(|e| i > e as i64)
                    .or_else(|| expected.as_u16().map(|e| i > e as i64))
                    .unwrap_or(false),
                Condition::LessThan(_, expected) => expected
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
    fn evaluate(&self, value: &TagValue, condition: &Condition) -> bool {
        if let Some(f) = value.as_f64() {
            match condition {
                Condition::Equals(_, expected) => expected
                    .as_f64()
                    .map(|e| (f - e).abs() < f64::EPSILON)
                    .unwrap_or(false),
                Condition::GreaterThan(_, expected) => {
                    expected.as_f64().map(|e| f > e).unwrap_or(false)
                }
                Condition::LessThan(_, expected) => {
                    expected.as_f64().map(|e| f < e).unwrap_or(false)
                }
                _ => false,
            }
        } else {
            false
        }
    }
}