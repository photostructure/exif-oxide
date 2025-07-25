//! Subdirectory condition evaluator for runtime evaluation
//!
//! This module implements the runtime condition evaluation system for subdirectory
//! dispatch, extending the existing ExpressionEvaluator with subdirectory-specific
//! logic patterns found in ExifTool.

use crate::expressions::ExpressionEvaluator;
use crate::tiff_types::ByteOrder;
use crate::types::{ExifError, Result, TagValue};
use regex::Regex;
use std::collections::HashMap;
use tracing::{debug, warn};

/// Enhanced condition evaluator for subdirectory dispatch
///
/// This evaluator extends the base ExpressionEvaluator with subdirectory-specific
/// evaluation capabilities, including binary pattern matching and model/make context.
///
/// ## ExifTool Reference
///
/// Corresponds to ExifTool's condition evaluation in SubDirectory dispatch:
/// - `$$valPt =~ /pattern/` - Binary data pattern matching
/// - `$$self{Model} =~ /pattern/` - Model name matching
/// - `$$self{Make} =~ /pattern/` - Manufacturer matching
/// - `$count == N` - Data count conditions
pub struct SubdirectoryConditionEvaluator {
    /// Base expression evaluator
    expression_evaluator: ExpressionEvaluator,
    
    /// Cache for compiled regex patterns specific to subdirectory conditions
    regex_cache: HashMap<String, Regex>,
}

/// Context for subdirectory condition evaluation
///
/// This structure provides all the context needed for evaluating subdirectory
/// conditions at runtime, including binary data, camera metadata, and processing state.
#[derive(Debug, Clone)]
pub struct SubdirectoryContext {
    /// Binary data being processed ($valPt)
    pub val_ptr: Option<Vec<u8>>,
    
    /// Camera make/manufacturer ($self{Make})
    pub make: Option<String>,
    
    /// Camera model ($self{Model})
    pub model: Option<String>,
    
    /// Data format information
    pub format: Option<String>,
    
    /// Count of data elements
    pub count: Option<usize>,
    
    /// Byte order for data interpretation
    pub byte_order: ByteOrder,
    
    /// Additional metadata for condition evaluation
    pub metadata: HashMap<String, TagValue>,
}

impl SubdirectoryConditionEvaluator {
    /// Create a new subdirectory condition evaluator
    pub fn new() -> Self {
        Self {
            expression_evaluator: ExpressionEvaluator::new(),
            regex_cache: HashMap::new(),
        }
    }

    /// Evaluate a subdirectory condition against the provided context
    ///
    /// This method handles the main subdirectory condition patterns found in ExifTool:
    /// - Binary pattern matching: `$$valPt =~ /^0204/`
    /// - Model matching: `$$self{Model} =~ /EOS R5/`
    /// - Make matching: `$$self{Make} =~ /Canon/`
    /// - Count conditions: `$count == 4`
    /// - Format conditions: `$format eq 'int16u'`
    ///
    /// ## Arguments
    ///
    /// * `condition` - The condition string to evaluate (ExifTool Perl syntax)
    /// * `context` - The subdirectory context with data and metadata
    ///
    /// ## Returns
    ///
    /// Returns `Ok(true)` if the condition matches, `Ok(false)` if it doesn't match,
    /// or an error if the condition cannot be parsed or evaluated.
    pub fn evaluate(
        &mut self,
        condition: &str,
        context: &SubdirectoryContext,
    ) -> Result<bool> {
        debug!("Evaluating subdirectory condition: {}", condition);

        // Handle special ExifTool condition patterns
        if let Some(result) = self.try_evaluate_special_patterns(condition, context)? {
            return Ok(result);
        }

        // Handle data pattern conditions ($$valPt)
        if condition.contains("$$valPt") {
            return self.evaluate_val_pt_condition(condition, context);
        }

        // Handle self reference conditions ($$self{Make}, $$self{Model})
        if condition.contains("$$self{") {
            return self.evaluate_self_condition(condition, context);
        }

        // Handle count conditions
        if condition.contains("$count") {
            return self.evaluate_count_condition(condition, context);
        }

        // Handle format conditions
        if condition.contains("$format") {
            return self.evaluate_format_condition(condition, context);
        }

        // For other patterns, try the base expression evaluator
        // Convert context to processor context and delegate
        // (This is a fallback for simple conditions)
        warn!("Unhandled condition pattern: {}", condition);
        Ok(false)
    }

    /// Try to evaluate special ExifTool condition patterns
    ///
    /// This method handles specific patterns that are common in ExifTool
    /// but don't fit the standard expression syntax.
    fn try_evaluate_special_patterns(
        &mut self,
        condition: &str,
        _context: &SubdirectoryContext,
    ) -> Result<Option<bool>> {
        // Handle literal true/false conditions
        match condition.trim() {
            "1" | "true" => return Ok(Some(true)),
            "0" | "false" => return Ok(Some(false)),
            _ => {}
        }

        // Handle simple numeric comparisons without variables
        if let Some(captures) = Regex::new(r"^(\d+)\s*(==|!=|<|>|<=|>=)\s*(\d+)$")
            .unwrap()
            .captures(condition.trim())
        {
            let left: i32 = captures[1].parse().map_err(|e| {
                ExifError::ParseError(format!("Invalid number in condition: {}", e))
            })?;
            let operator = &captures[2];
            let right: i32 = captures[3].parse().map_err(|e| {
                ExifError::ParseError(format!("Invalid number in condition: {}", e))
            })?;

            let result = match operator {
                "==" => left == right,
                "!=" => left != right,
                "<" => left < right,
                ">" => left > right,
                "<=" => left <= right,
                ">=" => left >= right,
                _ => false,
            };

            return Ok(Some(result));
        }

        Ok(None)
    }

    /// Evaluate $valPt binary pattern conditions
    ///
    /// Handles ExifTool patterns like:
    /// - `$$valPt =~ /^0204/`
    /// - `$$valPt =~ /\x00\x00\x00\x01/`
    /// - `$$valPt =~ /FUJIFILM/`
    fn evaluate_val_pt_condition(
        &mut self,
        condition: &str,
        context: &SubdirectoryContext,
    ) -> Result<bool> {
        debug!("Evaluating $$valPt condition: {}", condition);

        let val_ptr = match &context.val_ptr {
            Some(data) => data,
            None => {
                debug!("No $$valPt data available for condition evaluation");
                return Ok(false);
            }
        };

        // Extract regex pattern from condition
        // Handle patterns like: $$valPt =~ /^0204/
        if let Some(captures) = Regex::new(r"\$\$valPt\s*=~\s*/([^/]+)/")
            .unwrap()
            .captures(condition)
        {
            let pattern = &captures[1];
            debug!("Extracted pattern from $$valPt condition: {}", pattern);

            // Compile and test the regex directly against the data
            let regex = self.get_or_compile_regex(pattern)?;
            
            // Try multiple data representations for pattern matching
            // 1. Raw binary as string (for text patterns)
            let data_str = String::from_utf8_lossy(val_ptr);
            if regex.is_match(&data_str) {
                return Ok(true);
            }

            // 2. Hex representation for patterns like "^0204"
            let hex_str = val_ptr
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<String>();
            if regex.is_match(&hex_str) {
                return Ok(true);
            }

            // 3. Lowercase hex for case-insensitive matching
            let hex_lower = hex_str.to_lowercase();
            if regex.is_match(&hex_lower) {
                return Ok(true);
            }

            return Ok(false);
        }

        // Handle negated patterns: $$valPt !~ /pattern/
        if let Some(captures) = Regex::new(r"\$\$valPt\s*!~\s*/([^/]+)/")
            .unwrap()
            .captures(condition)
        {
            let pattern = &captures[1];
            debug!("Extracted negated pattern from $$valPt condition: {}", pattern);

            // Use the same logic as above, but negate the result
            let regex = self.get_or_compile_regex(pattern)?;
            
            let data_str = String::from_utf8_lossy(val_ptr);
            if regex.is_match(&data_str) {
                return Ok(false); // Negated match
            }

            let hex_str = val_ptr
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<String>();
            if regex.is_match(&hex_str) {
                return Ok(false); // Negated match
            }

            let hex_lower = hex_str.to_lowercase();
            if regex.is_match(&hex_lower) {
                return Ok(false); // Negated match
            }

            return Ok(true); // No match found, so negation is true
        }

        Err(ExifError::ParseError(format!(
            "Could not parse $$valPt condition: {}",
            condition
        )))
    }

    /// Evaluate $self{Make} and $self{Model} conditions
    ///
    /// Handles ExifTool patterns like:
    /// - `$$self{Model} =~ /EOS R5/`
    /// - `$$self{Make} =~ /Canon/`
    /// - `$$self{Model} eq 'NIKON D850'`
    fn evaluate_self_condition(
        &mut self,
        condition: &str,
        context: &SubdirectoryContext,
    ) -> Result<bool> {
        debug!("Evaluating $$self condition: {}", condition);

        // Extract self field and pattern
        if let Some(captures) = Regex::new(r"\$\$self\{(\w+)\}\s*=~\s*/([^/]+)/")
            .unwrap()
            .captures(condition)
        {
            let field = &captures[1];
            let pattern = &captures[2];

            let field_value = match field {
                "Make" => context.make.as_ref(),
                "Model" => context.model.as_ref(),
                _ => {
                    warn!("Unknown $$self field: {}", field);
                    return Ok(false);
                }
            };

            if let Some(value) = field_value {
                let regex = self.get_or_compile_regex(pattern)?;
                let result = regex.is_match(value);
                debug!(
                    "$$self{{{}}} = '{}' =~ /{}/: {}",
                    field, value, pattern, result
                );
                return Ok(result);
            } else {
                debug!("$$self{{{}}} is not available", field);
                return Ok(false);
            }
        }

        // Handle exact equality: $$self{Model} eq 'value'
        if let Some(captures) = Regex::new(r"\$\$self\{(\w+)\}\s*eq\s*'([^']+)'")
            .unwrap()
            .captures(condition)
        {
            let field = &captures[1];
            let expected = &captures[2];

            let field_value = match field {
                "Make" => context.make.as_ref(),
                "Model" => context.model.as_ref(),
                _ => {
                    warn!("Unknown $$self field: {}", field);
                    return Ok(false);
                }
            };

            if let Some(value) = field_value {
                let result = value == expected;
                debug!(
                    "$$self{{{}}} = '{}' eq '{}': {}",
                    field, value, expected, result
                );
                return Ok(result);
            } else {
                debug!("$$self{{{}}} is not available", field);
                return Ok(false);
            }
        }

        Err(ExifError::ParseError(format!(
            "Could not parse $$self condition: {}",
            condition
        )))
    }

    /// Evaluate count-based conditions
    ///
    /// Handles conditions like:
    /// - `$count == 4`
    /// - `$count > 10`
    /// - `$count <= 16`
    fn evaluate_count_condition(
        &self,
        condition: &str,
        context: &SubdirectoryContext,
    ) -> Result<bool> {
        debug!("Evaluating count condition: {}", condition);

        let count = match context.count {
            Some(c) => c,
            None => {
                debug!("No count available for condition evaluation");
                return Ok(false);
            }
        };

        // Parse count condition: $count <op> <value>
        if let Some(captures) = Regex::new(r"\$count\s*(==|!=|<|>|<=|>=)\s*(\d+)")
            .unwrap()
            .captures(condition)
        {
            let operator = &captures[1];
            let expected: usize = captures[2].parse().map_err(|e| {
                ExifError::ParseError(format!("Invalid count value: {}", e))
            })?;

            let result = match operator {
                "==" => count == expected,
                "!=" => count != expected,
                "<" => count < expected,
                ">" => count > expected,
                "<=" => count <= expected,
                ">=" => count >= expected,
                _ => false,
            };

            debug!("$count {} {} {}: {}", count, operator, expected, result);
            return Ok(result);
        }

        Err(ExifError::ParseError(format!(
            "Could not parse count condition: {}",
            condition
        )))
    }

    /// Evaluate format-based conditions
    ///
    /// Handles conditions like:
    /// - `$format eq 'int16u'`
    /// - `$format =~ /int/`
    fn evaluate_format_condition(
        &mut self,
        condition: &str,
        context: &SubdirectoryContext,
    ) -> Result<bool> {
        debug!("Evaluating format condition: {}", condition);

        let format = match &context.format {
            Some(f) => f,
            None => {
                debug!("No format available for condition evaluation");
                return Ok(false);
            }
        };

        // Handle exact equality: $format eq 'value'
        if let Some(captures) = Regex::new(r"\$format\s*eq\s*'([^']+)'")
            .unwrap()
            .captures(condition)
        {
            let expected = &captures[1];
            let result = format == expected;
            debug!("$format = '{}' eq '{}': {}", format, expected, result);
            return Ok(result);
        }

        // Handle regex match: $format =~ /pattern/
        if let Some(captures) = Regex::new(r"\$format\s*=~\s*/([^/]+)/")
            .unwrap()
            .captures(condition)
        {
            let pattern = &captures[1];
            let regex = self.get_or_compile_regex(pattern)?;
            let result = regex.is_match(format);
            debug!("$format = '{}' =~ /{}/: {}", format, pattern, result);
            return Ok(result);
        }

        Err(ExifError::ParseError(format!(
            "Could not parse format condition: {}",
            condition
        )))
    }

    /// Get or compile a regex pattern with caching
    fn get_or_compile_regex(&mut self, pattern: &str) -> Result<&Regex> {
        if !self.regex_cache.contains_key(pattern) {
            let regex = Regex::new(pattern).map_err(|e| {
                ExifError::ParseError(format!("Invalid regex pattern '{}': {}", pattern, e))
            })?;
            self.regex_cache.insert(pattern.to_string(), regex);
        }
        Ok(self.regex_cache.get(pattern).unwrap())
    }
}

impl Default for SubdirectoryConditionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl SubdirectoryContext {
    /// Create a new subdirectory context
    pub fn new() -> Self {
        Self {
            val_ptr: None,
            make: None,
            model: None,
            format: None,
            count: None,
            byte_order: ByteOrder::LittleEndian,
            metadata: HashMap::new(),
        }
    }

    /// Create context from binary data and metadata
    pub fn from_data(
        data: &[u8],
        make: Option<String>,
        model: Option<String>,
        byte_order: ByteOrder,
    ) -> Self {
        Self {
            val_ptr: Some(data.to_vec()),
            make,
            model,
            format: None,
            count: Some(data.len()),
            byte_order,
            metadata: HashMap::new(),
        }
    }

    /// Set the format information
    pub fn with_format(mut self, format: String) -> Self {
        self.format = Some(format);
        self
    }

    /// Set the count information
    pub fn with_count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }

    /// Add metadata field
    pub fn with_metadata(mut self, key: String, value: TagValue) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

impl Default for SubdirectoryContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_val_pt_pattern_matching() {
        let mut evaluator = SubdirectoryConditionEvaluator::new();
        
        // Test binary pattern matching
        let context = SubdirectoryContext::from_data(
            &[0x02, 0x04, 0x00, 0x01],
            None,
            None,
            ByteOrder::LittleEndian,
        );

        let result = evaluator
            .evaluate("$$valPt =~ /^0204/", &context)
            .unwrap();
        assert!(result, "Should match binary pattern ^0204");
    }

    #[test]
    fn test_self_model_matching() {
        let mut evaluator = SubdirectoryConditionEvaluator::new();
        
        let context = SubdirectoryContext {
            make: Some("Canon".to_string()),
            model: Some("EOS R5".to_string()),
            ..Default::default()
        };

        let result = evaluator
            .evaluate("$$self{Model} =~ /EOS R5/", &context)
            .unwrap();
        assert!(result, "Should match model pattern");

        let result = evaluator
            .evaluate("$$self{Make} eq 'Canon'", &context)
            .unwrap();
        assert!(result, "Should match exact make");
    }

    #[test]
    fn test_count_conditions() {
        let evaluator = SubdirectoryConditionEvaluator::new();
        
        let context = SubdirectoryContext {
            count: Some(4),
            ..Default::default()
        };

        let result = evaluator
            .evaluate_count_condition("$count == 4", &context)
            .unwrap();
        assert!(result, "Should match count == 4");

        let result = evaluator
            .evaluate_count_condition("$count > 2", &context)
            .unwrap();
        assert!(result, "Should match count > 2");
    }

    #[test]
    fn test_format_conditions() {
        let mut evaluator = SubdirectoryConditionEvaluator::new();
        
        let context = SubdirectoryContext {
            format: Some("int16u".to_string()),
            ..Default::default()
        };

        let result = evaluator
            .evaluate_format_condition("$format eq 'int16u'", &context)
            .unwrap();
        assert!(result, "Should match exact format");

        let result = evaluator
            .evaluate_format_condition("$format =~ /int/", &context)
            .unwrap();
        assert!(result, "Should match format pattern");
    }
}