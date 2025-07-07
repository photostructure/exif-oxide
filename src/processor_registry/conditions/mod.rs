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

// Module organization
pub mod parser;
pub mod types;

// Re-export key types
pub use parser::parse_condition;
pub use types::{Condition, FloatEvaluator, IntegerEvaluator, StringEvaluator, TagEvaluator};

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

        let condition = parse_condition(condition_expr)?;
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

        let condition = parse_condition(condition_expr)?;
        self.evaluate_data_condition_parsed(data, &condition)
    }

    /// Evaluate a parsed condition against binary data
    pub fn evaluate_data_condition_parsed(
        &mut self,
        data: &[u8],
        condition: &Condition,
    ) -> Result<bool> {
        match condition {
            Condition::DataPattern(pattern) => {
                let regex = self.get_or_compile_regex(pattern)?;

                // Try multiple data representations for pattern matching
                // 1. Raw binary as string
                let data_str = String::from_utf8_lossy(data);
                if regex.is_match(&data_str) {
                    return Ok(true);
                }

                // 2. Hex representation for patterns like "^0204"
                let hex_str = hex_string_from_bytes(data);
                if regex.is_match(&hex_str) {
                    return Ok(true);
                }

                // 3. Decimal representation for specific byte patterns
                if data.len() >= 4 {
                    let first_u32 = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                    let u32_str = first_u32.to_string();
                    if regex.is_match(&u32_str) {
                        return Ok(true);
                    }
                }

                Ok(false)
            }

            Condition::RegexMatch(field_name, pattern) if field_name == "valPt" => {
                // Handle $$valPt conditions that aren't explicitly DataPattern
                let regex = self.get_or_compile_regex(pattern)?;
                let data_str = String::from_utf8_lossy(data);
                Ok(regex.is_match(&data_str))
            }

            Condition::And(conditions) => {
                for cond in conditions {
                    if !self.evaluate_data_condition_parsed(data, cond)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }

            Condition::Or(conditions) => {
                for cond in conditions {
                    if self.evaluate_data_condition_parsed(data, cond)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }

            Condition::Not(inner_condition) => {
                Ok(!self.evaluate_data_condition_parsed(data, inner_condition)?)
            }

            _ => {
                // For non-data patterns, we can't evaluate without context
                Err(ExifError::ParseError(format!(
                    "Cannot evaluate context-dependent condition without binary data: {condition:?}"
                )))
            }
        }
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

            Condition::GreaterThanOrEqual(field_name, expected_value) => {
                if let Some(actual_value) = self.get_field_value(context, field_name) {
                    Ok(self.compare_values(&actual_value, expected_value) >= 0)
                } else {
                    Ok(false)
                }
            }

            Condition::LessThanOrEqual(field_name, expected_value) => {
                if let Some(actual_value) = self.get_field_value(context, field_name) {
                    Ok(self.compare_values(&actual_value, expected_value) <= 0)
                } else {
                    Ok(false)
                }
            }

            Condition::DataPattern(_pattern) => {
                // Data pattern conditions require binary data, which isn't available in context
                // This should be evaluated separately using evaluate_data_condition
                Err(ExifError::ParseError(
                    "Data pattern conditions cannot be evaluated without binary data".to_string(),
                ))
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
            // Cross-type numeric comparisons
            (TagValue::U16(a), TagValue::U32(b)) => *a as u32 == *b,
            (TagValue::U32(a), TagValue::U16(b)) => *a == *b as u32,
            (TagValue::U16(a), TagValue::I32(b)) => *a as i32 == *b,
            (TagValue::I32(a), TagValue::U16(b)) => *a == *b as i32,
            (TagValue::U32(a), TagValue::I32(b)) => *a as i32 == *b,
            (TagValue::I32(a), TagValue::U32(b)) => *a == *b as i32,
            // String to numeric comparisons
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
            (TagValue::String(s), TagValue::U32(i)) => {
                s.parse::<u32>().map(|parsed| parsed == *i).unwrap_or(false)
            }
            (TagValue::U32(i), TagValue::String(s)) => {
                s.parse::<u32>().map(|parsed| parsed == *i).unwrap_or(false)
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
            // Cross-type numeric comparisons
            (TagValue::U16(a), TagValue::U32(b)) => {
                let a_val = *a as u32;
                if a_val < *b {
                    -1
                } else if a_val > *b {
                    1
                } else {
                    0
                }
            }
            (TagValue::U32(a), TagValue::U16(b)) => {
                let b_val = *b as u32;
                if *a < b_val {
                    -1
                } else if *a > b_val {
                    1
                } else {
                    0
                }
            }
            (TagValue::U16(a), TagValue::I32(b)) => {
                let a_val = *a as i32;
                if a_val < *b {
                    -1
                } else if a_val > *b {
                    1
                } else {
                    0
                }
            }
            (TagValue::I32(a), TagValue::U16(b)) => {
                let b_val = *b as i32;
                if *a < b_val {
                    -1
                } else if *a > b_val {
                    1
                } else {
                    0
                }
            }
            (TagValue::String(a), TagValue::String(b)) => a.cmp(b) as i8,
            _ => 0, // Can't compare different types
        }
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

/// Convert binary data to hex string for pattern matching
///
/// This helper function converts binary data to a hex string representation
/// used for ExifTool-style data pattern matching.
fn hex_string_from_bytes(data: &[u8]) -> String {
    // Take only the first few bytes for pattern matching to avoid huge strings
    let max_bytes = std::cmp::min(data.len(), 16);
    data[..max_bytes]
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<String>()
}

#[cfg(test)]
mod tests;
