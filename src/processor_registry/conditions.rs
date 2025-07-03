//! Enhanced condition evaluation system for processor dispatch
//!
//! This module provides sophisticated condition evaluation capabilities that
//! enable complex processor selection logic based on context, data patterns,
//! and metadata. It extends the existing conditions system with processor-specific
//! evaluation logic.

use crate::types::{ExifError, Result, TagValue};
use regex::Regex;
use std::collections::HashMap;
use tracing::trace;

use super::ProcessorContext;

/// Enhanced condition evaluator for processor dispatch
///
/// This evaluator extends the basic condition system with processor-specific
/// evaluation capabilities, including data pattern matching, context evaluation,
/// and complex condition expressions.
///
/// ## ExifTool Reference
///
/// ExifTool uses various condition patterns in SubDirectory definitions:
/// ```perl
/// {
///     Condition => '$$valPt =~ /^0204/',
///     SubDirectory => { ProcessProc => \&ProcessNikonEncrypted }
/// },
/// {
///     Condition => '$$self{Model} =~ /EOS R5/',
///     SubDirectory => { ProcessProc => \&ProcessCanonSerialDataMkII }
/// }
/// ```
pub struct ConditionEvaluator {
    /// Cache for compiled regex patterns
    regex_cache: HashMap<String, Regex>,

    /// Registered tag evaluators for custom evaluation logic
    tag_evaluators: HashMap<String, Box<dyn TagEvaluator>>,
}

impl ConditionEvaluator {
    /// Create a new condition evaluator
    pub fn new() -> Self {
        let mut evaluator = Self {
            regex_cache: HashMap::new(),
            tag_evaluators: HashMap::new(),
        };

        // Register default tag evaluators
        evaluator.register_default_evaluators();
        evaluator
    }

    /// Evaluate a condition expression against a processor context
    ///
    /// This method parses and evaluates ExifTool-style condition expressions
    /// against the provided context, returning true if the condition matches.
    ///
    /// ## Examples
    ///
    /// - `"$model =~ /EOS R5/"` - Model contains "EOS R5"
    /// - `"$manufacturer eq 'Canon'"` - Exact manufacturer match
    /// - `"exists($serialNumber)"` - Serial number tag is available
    /// - `"$tagID == 0x001d"` - Specific tag ID match
    pub fn evaluate_context_condition(
        &mut self,
        context: &ProcessorContext,
        condition_expr: &str,
    ) -> Result<bool> {
        trace!(
            "Evaluating condition: {} for context: {}",
            condition_expr,
            context.table_name
        );

        let condition = Self::parse_condition(condition_expr)?;
        self.evaluate_condition(&condition, context)
    }

    /// Evaluate a condition against data pattern
    ///
    /// This method evaluates conditions that match against binary data patterns,
    /// commonly used for format detection and processor selection.
    pub fn evaluate_data_condition(&mut self, data: &[u8], condition_expr: &str) -> Result<bool> {
        trace!(
            "Evaluating data condition: {} against {} bytes",
            condition_expr,
            data.len()
        );

        // Handle data pattern matching ($$valPt =~ /pattern/)
        if let Some(pattern) = self.extract_data_pattern(condition_expr) {
            let regex = self.get_or_compile_regex(&pattern)?;

            // Convert data to string for pattern matching
            let data_str = String::from_utf8_lossy(data);
            Ok(regex.is_match(&data_str))
        } else {
            // For non-data patterns, we can't evaluate without context
            Err(ExifError::ParseError(format!(
                "Cannot evaluate context-dependent condition without context: {condition_expr}"
            )))
        }
    }

    /// Parse a condition expression into a structured condition
    /// TODO: MILESTONE-14.5 Phase 2 - Implement full ExifTool condition parsing
    /// This is a minimal stub for Phase 1, will be fully implemented in Phase 2
    fn parse_condition(expr: &str) -> Result<Condition> {
        let expr = expr.trim();

        // Handle exists() function
        if expr.starts_with("exists(") && expr.ends_with(")") {
            let tag_name = &expr[7..expr.len() - 1]; // Remove "exists(" and ")"
            let tag_name = tag_name
                .trim_matches('$')
                .trim_matches('"')
                .trim_matches('\'');
            return Ok(Condition::Exists(tag_name.to_string()));
        }

        // Handle regex patterns (=~)
        if expr.contains("=~") {
            let parts: Vec<&str> = expr.split("=~").collect();
            if parts.len() == 2 {
                let var_name = parts[0].trim().trim_start_matches('$');
                let pattern_str = parts[1].trim().trim_matches('/');
                return Ok(Condition::RegexMatch(
                    var_name.to_string(),
                    pattern_str.to_string(),
                ));
            }
        }

        // Handle equality comparisons (== or eq)
        if expr.contains("==") || expr.contains(" eq ") {
            let (var_name, value) = if expr.contains("==") {
                let parts: Vec<&str> = expr.split("==").collect();
                if parts.len() != 2 {
                    return Err(ExifError::ParseError(format!(
                        "Invalid == expression: {expr}"
                    )));
                }
                (parts[0].trim(), parts[1].trim())
            } else {
                let parts: Vec<&str> = expr.split(" eq ").collect();
                if parts.len() != 2 {
                    return Err(ExifError::ParseError(format!(
                        "Invalid eq expression: {expr}"
                    )));
                }
                (parts[0].trim(), parts[1].trim())
            };

            let var_name = var_name.trim_start_matches('$');
            let value_str = value.trim_matches('"').trim_matches('\'');

            // Try to parse as integer
            if let Ok(int_val) = value_str.parse::<i32>() {
                return Ok(Condition::Equals(
                    var_name.to_string(),
                    TagValue::I32(int_val),
                ));
            }

            // Otherwise treat as string
            return Ok(Condition::Equals(
                var_name.to_string(),
                TagValue::String(value_str.to_string()),
            ));
        }

        // Handle numeric comparisons
        if expr.contains(">") || expr.contains("<") {
            // TODO: Implement numeric comparison parsing
            return Err(ExifError::ParseError(format!(
                "Numeric comparisons not yet implemented: {expr}"
            )));
        }

        // Handle logical operators (and, or)
        if expr.contains(" and ") {
            let parts: Vec<&str> = expr.split(" and ").collect();
            let conditions: Result<Vec<_>> = parts
                .iter()
                .map(|part| Self::parse_condition(part))
                .collect();
            return Ok(Condition::And(conditions?));
        }

        if expr.contains(" or ") {
            let parts: Vec<&str> = expr.split(" or ").collect();
            let conditions: Result<Vec<_>> = parts
                .iter()
                .map(|part| Self::parse_condition(part))
                .collect();
            return Ok(Condition::Or(conditions?));
        }

        Err(ExifError::ParseError(format!(
            "Unsupported condition expression: {expr}"
        )))
    }

    /// Evaluate a structured condition against context
    fn evaluate_condition(
        &mut self,
        condition: &Condition,
        context: &ProcessorContext,
    ) -> Result<bool> {
        match condition {
            Condition::Exists(field_name) => Ok(self.field_exists(context, field_name)),

            Condition::Equals(field_name, expected_value) => {
                if let Some(actual_value) = self.get_field_value(context, field_name) {
                    Ok(self.values_equal(&actual_value, expected_value))
                } else {
                    Ok(false) // Field doesn't exist
                }
            }

            Condition::RegexMatch(field_name, pattern) => {
                if let Some(field_value) = self.get_field_value(context, field_name) {
                    let value_str = field_value.as_string().unwrap_or_default();
                    let regex = self.get_or_compile_regex(pattern)?;
                    Ok(regex.is_match(value_str))
                } else {
                    Ok(false) // Field doesn't exist
                }
            }

            Condition::And(conditions) => {
                for cond in conditions {
                    if !self.evaluate_condition(cond, context)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }

            Condition::Or(conditions) => {
                for cond in conditions {
                    if self.evaluate_condition(cond, context)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }

            Condition::Not(inner_condition) => {
                Ok(!self.evaluate_condition(inner_condition, context)?)
            }

            Condition::GreaterThan(field_name, expected_value) => {
                if let Some(actual_value) = self.get_field_value(context, field_name) {
                    Ok(self.compare_values(&actual_value, expected_value) > 0)
                } else {
                    Ok(false)
                }
            }

            Condition::LessThan(field_name, expected_value) => {
                if let Some(actual_value) = self.get_field_value(context, field_name) {
                    Ok(self.compare_values(&actual_value, expected_value) < 0)
                } else {
                    Ok(false)
                }
            }
        }
    }

    /// Check if a field exists in the context
    fn field_exists(&self, context: &ProcessorContext, field_name: &str) -> bool {
        match field_name {
            "manufacturer" | "make" => context.manufacturer.is_some(),
            "model" => context.model.is_some(),
            "firmware" | "firmwareVersion" => context.firmware.is_some(),
            "tagID" | "tag_id" => context.tag_id.is_some(),
            "formatVersion" | "format_version" => context.format_version.is_some(),
            _ => {
                // Check parameters and parent tags
                context.parameters.contains_key(field_name)
                    || context.parent_tags.contains_key(field_name)
            }
        }
    }

    /// Get field value from context
    fn get_field_value(&self, context: &ProcessorContext, field_name: &str) -> Option<TagValue> {
        match field_name {
            "manufacturer" | "make" => context
                .manufacturer
                .as_ref()
                .map(|v| TagValue::String(v.clone())),
            "model" => context.model.as_ref().map(|v| TagValue::String(v.clone())),
            "firmware" | "firmwareVersion" => context
                .firmware
                .as_ref()
                .map(|v| TagValue::String(v.clone())),
            "tagID" | "tag_id" => context.tag_id.map(TagValue::U16),
            "formatVersion" | "format_version" => context
                .format_version
                .as_ref()
                .map(|v| TagValue::String(v.clone())),
            _ => {
                // Check parameters first
                if let Some(param_value) = context.parameters.get(field_name) {
                    Some(TagValue::String(param_value.clone()))
                } else {
                    // Check parent tags
                    context.parent_tags.get(field_name).cloned()
                }
            }
        }
    }

    /// Compare two tag values for equality
    fn values_equal(&self, actual: &TagValue, expected: &TagValue) -> bool {
        match (actual, expected) {
            (TagValue::String(a), TagValue::String(b)) => a == b,
            (TagValue::I32(a), TagValue::I32(b)) => a == b,
            (TagValue::U16(a), TagValue::U16(b)) => a == b,
            (TagValue::U32(a), TagValue::U32(b)) => a == b,
            (TagValue::F64(a), TagValue::F64(b)) => (a - b).abs() < f64::EPSILON,
            // Cross-type comparisons
            (TagValue::String(s), TagValue::I32(i)) => {
                s.parse::<i32>().map(|parsed| parsed == *i).unwrap_or(false)
            }
            (TagValue::I32(i), TagValue::String(s)) => {
                s.parse::<i32>().map(|parsed| parsed == *i).unwrap_or(false)
            }
            (TagValue::String(s), TagValue::U16(i)) => {
                s.parse::<u16>().map(|parsed| parsed == *i).unwrap_or(false)
            }
            (TagValue::U16(i), TagValue::String(s)) => {
                s.parse::<u16>().map(|parsed| parsed == *i).unwrap_or(false)
            }
            _ => false,
        }
    }

    /// Compare two tag values (returns -1, 0, or 1)
    fn compare_values(&self, actual: &TagValue, expected: &TagValue) -> i8 {
        match (actual, expected) {
            (TagValue::I32(a), TagValue::I32(b)) => {
                if a < b {
                    -1
                } else if a > b {
                    1
                } else {
                    0
                }
            }
            (TagValue::U16(a), TagValue::U16(b)) => {
                if a < b {
                    -1
                } else if a > b {
                    1
                } else {
                    0
                }
            }
            (TagValue::U32(a), TagValue::U32(b)) => {
                if a < b {
                    -1
                } else if a > b {
                    1
                } else {
                    0
                }
            }
            (TagValue::F64(a), TagValue::F64(b)) => {
                if a < b {
                    -1
                } else if a > b {
                    1
                } else {
                    0
                }
            }
            (TagValue::String(a), TagValue::String(b)) => a.cmp(b) as i8,
            _ => 0, // Can't compare different types
        }
    }

    /// Extract data pattern from condition expression
    fn extract_data_pattern(&self, expr: &str) -> Option<String> {
        // Look for $$valPt =~ /pattern/ expressions
        if expr.contains("$$valPt") && expr.contains("=~") {
            if let Some(start) = expr.find('/') {
                if let Some(end) = expr.rfind('/') {
                    if start < end {
                        return Some(expr[start + 1..end].to_string());
                    }
                }
            }
        }
        None
    }

    /// Get or compile a regex pattern
    fn get_or_compile_regex(&mut self, pattern: &str) -> Result<&Regex> {
        if !self.regex_cache.contains_key(pattern) {
            let regex = Regex::new(pattern).map_err(|e| {
                ExifError::ParseError(format!("Invalid regex pattern '{pattern}': {e}"))
            })?;
            self.regex_cache.insert(pattern.to_string(), regex);
        }
        Ok(self.regex_cache.get(pattern).unwrap())
    }

    /// Register default tag evaluators
    fn register_default_evaluators(&mut self) {
        // Add default evaluators for common tag types
        self.tag_evaluators
            .insert("string".to_string(), Box::new(StringEvaluator));
        self.tag_evaluators
            .insert("integer".to_string(), Box::new(IntegerEvaluator));
        self.tag_evaluators
            .insert("float".to_string(), Box::new(FloatEvaluator));
    }

    /// Register a custom tag evaluator
    pub fn register_tag_evaluator<E: TagEvaluator + 'static>(
        &mut self,
        name: String,
        evaluator: E,
    ) {
        self.tag_evaluators.insert(name, Box::new(evaluator));
    }
}

impl Default for ConditionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

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

    /// Check if a field is less than a value
    LessThan(String, TagValue),

    /// Check if a field matches a regex pattern
    RegexMatch(String, String),

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
struct StringEvaluator;
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
struct IntegerEvaluator;
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
struct FloatEvaluator;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::FileFormat;

    #[test]
    fn test_simple_equality_condition() {
        let mut evaluator = ConditionEvaluator::new();
        let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::Main".to_string())
            .with_manufacturer("Canon".to_string());

        let result = evaluator
            .evaluate_context_condition(&context, "$manufacturer eq 'Canon'")
            .unwrap();
        assert!(result);

        let result = evaluator
            .evaluate_context_condition(&context, "$manufacturer eq 'Nikon'")
            .unwrap();
        assert!(!result);
    }

    #[test]
    fn test_regex_condition() {
        let mut evaluator = ConditionEvaluator::new();
        let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::Main".to_string())
            .with_model("Canon EOS R5".to_string());

        let result = evaluator
            .evaluate_context_condition(&context, "$model =~ /EOS R5/")
            .unwrap();
        assert!(result);

        let result = evaluator
            .evaluate_context_condition(&context, "$model =~ /R6/")
            .unwrap();
        assert!(!result);
    }

    #[test]
    fn test_exists_condition() {
        let mut evaluator = ConditionEvaluator::new();
        let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::Main".to_string())
            .with_manufacturer("Canon".to_string());

        let result = evaluator
            .evaluate_context_condition(&context, "exists($manufacturer)")
            .unwrap();
        assert!(result);

        let result = evaluator
            .evaluate_context_condition(&context, "exists($model)")
            .unwrap();
        assert!(!result);
    }

    // TODO: MILESTONE-14.5 Phase 2 - Add comprehensive logical operator tests
    // Test cases for: and, or, not operators with complex conditions

    // TODO: MILESTONE-14.5 Phase 2 - Add data pattern condition tests
    // Test cases for: $$valPt pattern matching with binary data

    // TODO: MILESTONE-14.5 Phase 2 - Add tag ID condition tests
    // Test cases for: $tagID numeric comparisons with hex and decimal values
}
