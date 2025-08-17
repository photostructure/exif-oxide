//! String concatenation and regex operations
//!
//! This module handles string operations including concatenation, repetition,
//! and regex pattern matching.

use crate::ppi::rust_generator::errors::CodeGenError;
use crate::ppi::types::*;

/// Trait for handling string operations
pub trait StringOperationsHandler {
    fn expression_type(&self) -> &ExpressionType;

    /// Handle normalized StringConcat nodes
    fn handle_normalized_string_concat(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        // Process each child node properly - they may be complex expressions
        let mut parts = Vec::new();

        for child in &node.children {
            let part = if let Some(ref content) = child.content {
                // Simple content
                content.clone()
            } else if let Some(ref string_value) = child.string_value {
                // String literal
                format!("\"{}\"", string_value)
            } else {
                // Complex expression - recursively process it
                self.combine_statement_parts(&[], &[child.clone()])?
            };
            parts.push(part);
        }

        // Generate format! call with all parts
        match self.expression_type() {
            ExpressionType::PrintConv | ExpressionType::ValueConv => Ok(format!(
                "TagValue::String(format!(\"{}\", {}))",
                "{}".repeat(parts.len()),
                parts.join(", ")
            )),
            _ => Ok(format!(
                "format!(\"{}\", {})",
                "{}".repeat(parts.len()),
                parts.join(", ")
            )),
        }
    }

    /// Handle normalized StringRepeat nodes  
    fn handle_normalized_string_repeat(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        if node.children.len() != 2 {
            return Err(CodeGenError::UnsupportedStructure(
                "StringRepeat needs exactly 2 children".to_string(),
            ));
        }

        let string_part = self.process_function_args(&[node.children[0].clone()])?[0].clone();
        let count = self.process_function_args(&[node.children[1].clone()])?[0].clone();

        match self.expression_type() {
            ExpressionType::PrintConv | ExpressionType::ValueConv => Ok(format!(
                "TagValue::String({}.repeat({} as usize))",
                string_part, count
            )),
            _ => Ok(format!("{}.repeat({} as usize)", string_part, count)),
        }
    }

    /// Try to handle string concatenation pattern: expr . expr
    fn try_string_concat_pattern(&self, parts: &[String]) -> Result<Option<String>, CodeGenError> {
        if let Some(dot_pos) = parts.iter().position(|p| p == ".") {
            if dot_pos > 0 && dot_pos < parts.len() - 1 {
                let left_parts = &parts[..dot_pos];
                let right_parts = &parts[dot_pos + 1..];

                // Join the parts back into expressions
                let left_expr = left_parts.join(" ");
                let right_expr = right_parts.join(" ");

                // Generate string concatenation using format!
                let result = match self.expression_type() {
                    ExpressionType::PrintConv | ExpressionType::ValueConv => format!(
                        "TagValue::String(format!(\"{{}}{{}}\", {}, {}))",
                        left_expr, right_expr
                    ),
                    _ => format!("format!(\"{{}}{{}}\", {}, {})", left_expr, right_expr),
                };
                return Ok(Some(result));
            }
        }
        Ok(None)
    }

    /// Handle regex matching in binary operations
    fn handle_regex_operation(
        &self,
        left: &str,
        op: &str,
        right: &str,
    ) -> Result<String, CodeGenError> {
        // Extract the regex pattern from right side (e.g., "/\\d/" -> "\\d")
        let pattern = if right.starts_with('/') && right.ends_with('/') {
            &right[1..right.len() - 1]
        } else {
            right
        };

        // For simple pattern matching, use contains or regex
        // \d means contains a digit
        if pattern == "\\d" {
            if op == "=~" {
                return Ok(format!(
                    "{}.to_string().chars().any(|c| c.is_ascii_digit())",
                    left
                ));
            } else {
                return Ok(format!(
                    "!{}.to_string().chars().any(|c| c.is_ascii_digit())",
                    left
                ));
            }
        }

        // For other patterns, use a simple contains check
        // This is a simplification - full regex support would need the regex crate
        if op == "=~" {
            return Ok(format!("{}.to_string().contains(r\"{}\")", left, pattern));
        } else {
            return Ok(format!("!{}.to_string().contains(r\"{}\")", left, pattern));
        }
    }

    /// Check if operator is a string operation
    fn is_string_operator(&self, op: &str) -> bool {
        matches!(op, "." | "=~" | "!~")
    }

    /// Legacy method compatibility - delegate to main combiner
    fn combine_statement_parts(
        &self,
        parts: &[String],
        children: &[PpiNode],
    ) -> Result<String, CodeGenError>;

    /// Process function arguments from child nodes
    fn process_function_args(&self, children: &[PpiNode]) -> Result<Vec<String>, CodeGenError>;
}
