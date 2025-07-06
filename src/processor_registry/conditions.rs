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

        let condition = Self::parse_condition(condition_expr)?;
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

    /// Parse a condition expression into a structured condition
    /// MILESTONE-14.5 Phase 2 - Full ExifTool condition parsing implementation
    /// Supports comprehensive ExifTool condition syntax including data patterns,
    /// logical operators, numeric comparisons, and complex expressions
    fn parse_condition(expr: &str) -> Result<Condition> {
        let expr = expr.trim();

        // Handle parentheses for grouping
        if expr.starts_with('(') && expr.ends_with(')') {
            return Self::parse_condition(&expr[1..expr.len() - 1]);
        }

        // Handle logical NOT operator
        if let Some(stripped) = expr.strip_prefix("not ") {
            let inner_condition = Self::parse_condition(stripped)?;
            return Ok(Condition::Not(Box::new(inner_condition)));
        }
        if let Some(stripped) = expr.strip_prefix("!") {
            let inner_condition = Self::parse_condition(stripped)?;
            return Ok(Condition::Not(Box::new(inner_condition)));
        }

        // Handle logical operators (and, or) with proper precedence
        // Parse OR first (lower precedence), then AND (higher precedence)
        if let Some(or_index) = Self::find_operator_outside_parens(expr, " or ") {
            let left_expr = &expr[..or_index];
            let right_expr = &expr[or_index + 4..]; // " or " is 4 chars
            let left_condition = Self::parse_condition(left_expr)?;
            let right_condition = Self::parse_condition(right_expr)?;
            return Ok(Condition::Or(vec![left_condition, right_condition]));
        }

        if let Some(and_index) = Self::find_operator_outside_parens(expr, " and ") {
            let left_expr = &expr[..and_index];
            let right_expr = &expr[and_index + 5..]; // " and " is 5 chars
            let left_condition = Self::parse_condition(left_expr)?;
            let right_condition = Self::parse_condition(right_expr)?;
            return Ok(Condition::And(vec![left_condition, right_condition]));
        }

        // Handle exists() function
        if expr.starts_with("exists(") && expr.ends_with(")") {
            let tag_name = &expr[7..expr.len() - 1]; // Remove "exists(" and ")"
            let tag_name = tag_name
                .trim_matches('$')
                .trim_matches('"')
                .trim_matches('\'');
            return Ok(Condition::Exists(tag_name.to_string()));
        }

        // Handle data pattern matching ($$valPt =~ /pattern/)
        if expr.contains("$$valPt") && expr.contains("=~") {
            return Self::parse_data_pattern_condition(expr);
        }

        // Handle regex patterns (=~ and !~)
        if expr.contains("=~") || expr.contains("!~") {
            return Self::parse_regex_condition(expr);
        }

        // Handle numeric comparisons (>, <, >=, <=)
        if let Some(comparison_op) = Self::find_comparison_operator(expr) {
            return Self::parse_numeric_comparison(expr, &comparison_op);
        }

        // Handle equality and inequality comparisons (==, eq, !=, ne)
        if expr.contains("==")
            || expr.contains(" eq ")
            || expr.contains("!=")
            || expr.contains(" ne ")
        {
            return Self::parse_equality_condition(expr);
        }

        // Handle hexadecimal number patterns (0x1234, 0X1234)
        if Self::is_hex_number_condition(expr) {
            return Self::parse_hex_condition(expr);
        }

        Err(ExifError::ParseError(format!(
            "Unsupported condition expression: {expr}"
        )))
    }

    /// Find operator position outside of parentheses
    fn find_operator_outside_parens(expr: &str, operator: &str) -> Option<usize> {
        let mut paren_count = 0;
        let mut quote_char: Option<char> = None;
        let operator_bytes = operator.as_bytes();
        let expr_bytes = expr.as_bytes();

        for i in 0..expr_bytes.len() {
            let ch = expr_bytes[i] as char;

            // Handle quote tracking
            if quote_char.is_none() && (ch == '"' || ch == '\'') {
                quote_char = Some(ch);
                continue;
            } else if let Some(qc) = quote_char {
                if ch == qc {
                    quote_char = None;
                }
                continue;
            }

            // Skip if inside quotes
            if quote_char.is_some() {
                continue;
            }

            // Handle parentheses
            if ch == '(' {
                paren_count += 1;
            } else if ch == ')' {
                paren_count -= 1;
            } else if paren_count == 0 {
                // Check for operator match
                if i + operator_bytes.len() <= expr_bytes.len()
                    && &expr_bytes[i..i + operator_bytes.len()] == operator_bytes
                {
                    return Some(i);
                }
            }
        }

        None
    }

    /// Parse data pattern condition ($$valPt =~ /pattern/)
    fn parse_data_pattern_condition(expr: &str) -> Result<Condition> {
        if let Some(pattern_start) = expr.find('/') {
            if let Some(pattern_end) = expr.rfind('/') {
                if pattern_start < pattern_end {
                    let pattern = &expr[pattern_start + 1..pattern_end];
                    return Ok(Condition::DataPattern(pattern.to_string()));
                }
            }
        }
        Err(ExifError::ParseError(format!(
            "Invalid data pattern condition: {expr}"
        )))
    }

    /// Parse regex condition (field =~ /pattern/ or field !~ /pattern/)
    fn parse_regex_condition(expr: &str) -> Result<Condition> {
        let is_negative = expr.contains("!~");
        let operator = if is_negative { "!~" } else { "=~" };

        if let Some(op_pos) = expr.find(operator) {
            let var_part = expr[..op_pos].trim();
            let pattern_part = expr[op_pos + operator.len()..].trim();

            let var_name = var_part.trim_start_matches('$');
            let pattern_str = pattern_part.trim_matches('/');

            let condition = Condition::RegexMatch(var_name.to_string(), pattern_str.to_string());

            if is_negative {
                Ok(Condition::Not(Box::new(condition)))
            } else {
                Ok(condition)
            }
        } else {
            Err(ExifError::ParseError(format!(
                "Invalid regex condition: {expr}"
            )))
        }
    }

    /// Find comparison operator in expression
    fn find_comparison_operator(expr: &str) -> Option<String> {
        // Check in order of specificity (longer operators first)
        for op in &[">=", "<=", ">", "<"] {
            if expr.contains(op) {
                return Some(op.to_string());
            }
        }
        None
    }

    /// Parse numeric comparison condition
    fn parse_numeric_comparison(expr: &str, operator: &str) -> Result<Condition> {
        if let Some(op_pos) = expr.find(operator) {
            let var_part = expr[..op_pos].trim();
            let value_part = expr[op_pos + operator.len()..].trim();

            let var_name = var_part.trim_start_matches('$');
            let value = Self::parse_value(value_part)?;

            match operator {
                ">" => Ok(Condition::GreaterThan(var_name.to_string(), value)),
                ">=" => Ok(Condition::GreaterThanOrEqual(var_name.to_string(), value)),
                "<" => Ok(Condition::LessThan(var_name.to_string(), value)),
                "<=" => Ok(Condition::LessThanOrEqual(var_name.to_string(), value)),
                _ => Err(ExifError::ParseError(format!(
                    "Unknown comparison operator: {operator}"
                ))),
            }
        } else {
            Err(ExifError::ParseError(format!(
                "Invalid comparison condition: {expr}"
            )))
        }
    }

    /// Parse equality/inequality condition
    fn parse_equality_condition(expr: &str) -> Result<Condition> {
        let (operator, is_negative) = if expr.contains("!=") {
            ("!=", true)
        } else if expr.contains(" ne ") {
            (" ne ", true)
        } else if expr.contains("==") {
            ("==", false)
        } else if expr.contains(" eq ") {
            (" eq ", false)
        } else {
            return Err(ExifError::ParseError(format!(
                "No equality operator found in: {expr}"
            )));
        };

        if let Some(op_pos) = expr.find(operator) {
            let var_part = expr[..op_pos].trim();
            let value_part = expr[op_pos + operator.len()..].trim();

            let var_name = var_part.trim_start_matches('$');
            let value = Self::parse_value(value_part)?;

            let condition = Condition::Equals(var_name.to_string(), value);

            if is_negative {
                Ok(Condition::Not(Box::new(condition)))
            } else {
                Ok(condition)
            }
        } else {
            Err(ExifError::ParseError(format!(
                "Invalid equality condition: {expr}"
            )))
        }
    }

    /// Check if expression is a hex number condition
    fn is_hex_number_condition(expr: &str) -> bool {
        expr.contains("0x") || expr.contains("0X")
    }

    /// Parse hex number condition
    fn parse_hex_condition(expr: &str) -> Result<Condition> {
        // This handles cases like "$tagID == 0x001d"
        if expr.contains("==") {
            return Self::parse_equality_condition(expr);
        }

        Err(ExifError::ParseError(format!(
            "Unsupported hex condition format: {expr}"
        )))
    }

    /// Parse a value from string representation
    fn parse_value(value_str: &str) -> Result<TagValue> {
        let value_str = value_str.trim().trim_matches('"').trim_matches('\'');

        // Try hex number first
        if value_str.starts_with("0x") || value_str.starts_with("0X") {
            if let Ok(hex_val) = u32::from_str_radix(&value_str[2..], 16) {
                return Ok(TagValue::U32(hex_val));
            }
        }

        // Try decimal integers
        if let Ok(int_val) = value_str.parse::<i32>() {
            return Ok(TagValue::I32(int_val));
        }

        // Try unsigned integers
        if let Ok(uint_val) = value_str.parse::<u32>() {
            return Ok(TagValue::U32(uint_val));
        }

        // Try floating point
        if let Ok(float_val) = value_str.parse::<f64>() {
            return Ok(TagValue::F64(float_val));
        }

        // Default to string
        Ok(value_str.into())
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

impl ConditionEvaluator {
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

    #[test]
    fn test_complex_logical_operators() {
        let mut evaluator = ConditionEvaluator::new();

        // Test AND operator
        let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::Main".to_string())
            .with_manufacturer("Canon".to_string())
            .with_model("EOS R5".to_string());

        let result = evaluator
            .evaluate_context_condition(&context, "$manufacturer eq 'Canon' and $model =~ /EOS R5/")
            .unwrap();
        assert!(result);

        let result = evaluator
            .evaluate_context_condition(&context, "$manufacturer eq 'Canon' and $model =~ /R6/")
            .unwrap();
        assert!(!result);

        // Test OR operator
        let result = evaluator
            .evaluate_context_condition(&context, "$model =~ /R5/ or $model =~ /R6/")
            .unwrap();
        assert!(result);

        let result = evaluator
            .evaluate_context_condition(&context, "$model =~ /R6/ or $model =~ /R3/")
            .unwrap();
        assert!(!result);

        // Test NOT operator
        let result = evaluator
            .evaluate_context_condition(&context, "not $manufacturer eq 'Nikon'")
            .unwrap();
        assert!(result);

        let result = evaluator
            .evaluate_context_condition(&context, "!($model =~ /R6/)")
            .unwrap();
        assert!(result);
    }

    #[test]
    fn test_data_pattern_conditions() {
        let mut evaluator = ConditionEvaluator::new();

        // Test Nikon encryption pattern
        let nikon_encrypted_data = vec![0x02, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04];

        let result = evaluator
            .evaluate_data_condition(&nikon_encrypted_data, "$$valPt =~ /^0200/")
            .unwrap();
        assert!(result);

        // Test pattern that doesn't match
        let result = evaluator
            .evaluate_data_condition(&nikon_encrypted_data, "$$valPt =~ /^0400/")
            .unwrap();
        assert!(!result);

        // Test different encryption patterns
        let nikon_204_data = vec![0x02, 0x04, 0x00, 0x01];
        let result = evaluator
            .evaluate_data_condition(&nikon_204_data, "$$valPt =~ /^0204/")
            .unwrap();
        assert!(result);

        let nikon_402_data = vec![0x04, 0x02, 0x00, 0x01];
        let result = evaluator
            .evaluate_data_condition(&nikon_402_data, "$$valPt =~ /^0402/")
            .unwrap();
        assert!(result);
    }

    #[test]
    fn test_tag_id_conditions() {
        let mut evaluator = ConditionEvaluator::new();

        // Test hex tag ID
        let context = ProcessorContext::new(FileFormat::Jpeg, "Nikon::Main".to_string())
            .with_manufacturer("NIKON CORPORATION".to_string())
            .with_tag_id(0x001d);

        let result = evaluator
            .evaluate_context_condition(&context, "$tagID == 0x001d")
            .unwrap();
        assert!(result);

        let result = evaluator
            .evaluate_context_condition(&context, "$tagID == 0x00a7")
            .unwrap();
        assert!(!result);

        // Test decimal tag ID
        let context =
            ProcessorContext::new(FileFormat::Jpeg, "Canon::Main".to_string()).with_tag_id(29); // 0x001d in decimal

        let result = evaluator
            .evaluate_context_condition(&context, "$tag_id == 29")
            .unwrap();
        assert!(result);
    }

    #[test]
    fn test_numeric_comparisons() {
        let mut evaluator = ConditionEvaluator::new();

        let mut context = ProcessorContext::new(FileFormat::Jpeg, "Canon::Main".to_string());
        context.add_parent_tag("AFInfoVersion".to_string(), TagValue::U16(0x0002));

        // Test greater than
        let result = evaluator
            .evaluate_context_condition(&context, "$AFInfoVersion > 0x0001")
            .unwrap();
        assert!(result);

        // Test greater than or equal
        let result = evaluator
            .evaluate_context_condition(&context, "$AFInfoVersion >= 0x0002")
            .unwrap();
        assert!(result);

        // Test less than
        let result = evaluator
            .evaluate_context_condition(&context, "$AFInfoVersion < 0x0003")
            .unwrap();
        assert!(result);

        // Test less than or equal
        let result = evaluator
            .evaluate_context_condition(&context, "$AFInfoVersion <= 0x0002")
            .unwrap();
        assert!(result);
    }

    #[test]
    fn test_inequality_conditions() {
        let mut evaluator = ConditionEvaluator::new();

        let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::Main".to_string())
            .with_manufacturer("Canon".to_string());

        // Test not equal (!=)
        let result = evaluator
            .evaluate_context_condition(&context, "$manufacturer != 'Nikon'")
            .unwrap();
        assert!(result);

        // Test not equal (ne)
        let result = evaluator
            .evaluate_context_condition(&context, "$manufacturer ne 'Nikon'")
            .unwrap();
        assert!(result);

        let result = evaluator
            .evaluate_context_condition(&context, "$manufacturer != 'Canon'")
            .unwrap();
        assert!(!result);
    }

    #[test]
    fn test_parentheses_grouping() {
        let mut evaluator = ConditionEvaluator::new();

        let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::Main".to_string())
            .with_manufacturer("Canon".to_string())
            .with_model("EOS R5".to_string());

        // Test simple parentheses
        let result = evaluator
            .evaluate_context_condition(&context, "($manufacturer eq 'Canon')")
            .unwrap();
        assert!(result);

        // Test AND with parentheses
        let result = evaluator
            .evaluate_context_condition(
                &context,
                "($manufacturer eq 'Canon' and $model eq 'EOS R5')",
            )
            .unwrap();
        assert!(result);

        // Test OR with simple conditions
        let result = evaluator
            .evaluate_context_condition(
                &context,
                "$manufacturer eq 'Canon' or $manufacturer eq 'Nikon'",
            )
            .unwrap();
        assert!(result);

        let result = evaluator
            .evaluate_context_condition(
                &context,
                "$manufacturer eq 'Nikon' or $manufacturer eq 'Sony'",
            )
            .unwrap();
        assert!(!result);
    }

    #[test]
    fn test_regex_negation() {
        let mut evaluator = ConditionEvaluator::new();

        let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::Main".to_string())
            .with_model("Canon EOS R5".to_string());

        // Test regex negation (!~)
        let result = evaluator
            .evaluate_context_condition(&context, "$model !~ /R6/")
            .unwrap();
        assert!(result);

        let result = evaluator
            .evaluate_context_condition(&context, "$model !~ /R5/")
            .unwrap();
        assert!(!result);
    }

    #[test]
    fn test_binary_data_complex_patterns() {
        let mut evaluator = ConditionEvaluator::new();

        // Test complex Nikon data patterns
        let complex_data = vec![0x02, 0x04, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06];

        // Test multiple pattern matching attempts
        let result = evaluator
            .evaluate_data_condition(&complex_data, "$$valPt =~ /^0204/ and $$valPt =~ /0102/")
            .unwrap();
        assert!(result);

        let result = evaluator
            .evaluate_data_condition(&complex_data, "$$valPt =~ /^0300/ or $$valPt =~ /^0204/")
            .unwrap();
        assert!(result);
    }

    #[test]
    fn test_error_handling() {
        let mut evaluator = ConditionEvaluator::new();

        // Test invalid syntax
        assert!(evaluator
            .evaluate_context_condition(&ProcessorContext::default(), "invalid syntax")
            .is_err());

        // Test invalid regex pattern
        assert!(evaluator
            .evaluate_data_condition(&[0u8; 4], "$$valPt =~ /[/")
            .is_err());

        // Test truly unsupported syntax - malformed expression
        let result = evaluator.evaluate_context_condition(
            &ProcessorContext::default(),
            "malformed & invalid #% syntax",
        );
        assert!(
            result.is_err(),
            "Expected error for malformed syntax but got: {result:?}"
        );
    }
}
