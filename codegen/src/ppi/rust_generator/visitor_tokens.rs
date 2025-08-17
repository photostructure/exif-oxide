//! Helper functions for basic token processing
//!
//! This module contains standalone helper functions for processing simple PPI tokens
//! that don't require recursion or complex traversal logic.

use crate::ppi::rust_generator::errors::CodeGenError;
use crate::ppi::types::*;

/// Process symbol nodes (variables like $val, $$self{Field})
pub fn process_symbol(node: &PpiNode) -> Result<String, CodeGenError> {
    let content = node
        .content
        .as_ref()
        .ok_or(CodeGenError::MissingContent("symbol".to_string()))?;

    if node.is_self_reference() {
        if let Some(field) = node.extract_self_field() {
            Ok(format!("ctx.get(\"{field}\").unwrap_or_default()"))
        } else {
            Err(CodeGenError::InvalidSelfReference(content.clone()))
        }
    } else if content == "$val" {
        Ok("val".to_string())
    } else if content == "$valPt" {
        Ok("val_pt".to_string())
    } else {
        // Generic variable
        Ok(content.trim_start_matches('$').to_string())
    }
}

/// Process operator nodes
pub fn process_operator(node: &PpiNode) -> Result<String, CodeGenError> {
    let op = node
        .content
        .as_ref()
        .ok_or(CodeGenError::MissingContent("operator".to_string()))?;

    // Convert Perl operators to Rust equivalents
    // Trust ExifTool: preserve original logic but translate to valid Rust syntax
    let rust_op = match op.as_str() {
        "eq" => "==",
        "ne" => "!=",
        "lt" => "<",
        "gt" => ">",
        "le" => "<=",
        "ge" => ">=",
        "and" => "&&",
        "or" => "||",
        "xor" => "^",
        // Keep other operators as-is
        _ => op,
    };

    Ok(rust_op.to_string())
}

/// Process number nodes - enhanced for better float and scientific notation handling
pub fn process_number(node: &PpiNode) -> Result<String, CodeGenError> {
    if let Some(num) = node.numeric_value {
        // For code generation, use appropriate literal format
        if num.fract() == 0.0 && num.abs() < 1e10 {
            // Integer value within reasonable range
            Ok(format!("{}", num as i64))
        } else {
            // Float value or large number - ensure Rust float literal format
            let num_str = num.to_string();
            // Add explicit float suffix if not present for clarity
            if !num_str.contains('e') && !num_str.contains('.') {
                Ok(format!("{}.0", num_str))
            } else {
                Ok(num_str)
            }
        }
    } else if let Some(content) = &node.content {
        // Handle special numeric formats
        if content.contains('e') || content.contains('E') {
            // Scientific notation - ensure proper format
            Ok(content.to_lowercase())
        } else if content.contains('.') {
            // Decimal number - preserve as-is
            Ok(content.clone())
        } else {
            // Integer - validate and return
            if content
                .chars()
                .all(|c| c.is_ascii_digit() || c == '-' || c == '+')
            {
                Ok(content.clone())
            } else {
                Err(CodeGenError::InvalidNumber(content.clone()))
            }
        }
    } else {
        Err(CodeGenError::MissingContent("number".to_string()))
    }
}

/// Process hex number nodes
pub fn process_number_hex(node: &PpiNode) -> Result<String, CodeGenError> {
    let hex_string = node
        .content
        .as_ref()
        .ok_or(CodeGenError::MissingContent("hex number".to_string()))?;

    // Preserve hex format
    if hex_string.starts_with("0x") || hex_string.starts_with("0X") {
        Ok(hex_string.to_lowercase())
    } else {
        // Add 0x prefix if missing
        Ok(format!("0x{}", hex_string.trim_start_matches('#')))
    }
}

/// Process string nodes (quoted strings)
pub fn process_string(node: &PpiNode) -> Result<String, CodeGenError> {
    let string_value = node
        .string_value
        .as_ref()
        .or(node.content.as_ref())
        .ok_or(CodeGenError::MissingContent("string".to_string()))?;

    // Handle simple variable interpolation
    if string_value.contains("$val") && string_value.matches('$').count() == 1 {
        let template = string_value.replace("$val", "{}");
        Ok(format!("format!(\"{}\", val)", template))
    } else {
        // Simple string literal
        Ok(format!("\"{}\"", string_value.replace('\"', "\\\"")))
    }
}

/// Process word nodes (function names, keywords) - requires expression type context
pub fn process_word(
    node: &PpiNode,
    expression_type: &ExpressionType,
) -> Result<String, CodeGenError> {
    let word = node
        .content
        .as_ref()
        .ok_or(CodeGenError::MissingContent("word".to_string()))?;

    // Handle special Perl keywords
    match word.as_str() {
        "undef" => {
            // Perl's undef translates to appropriate default value
            match expression_type {
                ExpressionType::PrintConv => Ok("TagValue::String(\"\".to_string())".to_string()),
                ExpressionType::ValueConv => Ok("TagValue::String(\"\".to_string())".to_string()),
                ExpressionType::Condition => Ok("false".to_string()),
            }
        }
        _ => Ok(word.clone()),
    }
}

/// Process structure tokens - handles structural elements like parentheses, brackets
pub fn process_structure(node: &PpiNode) -> Result<String, CodeGenError> {
    let content = node
        .content
        .as_ref()
        .ok_or(CodeGenError::MissingContent("structure".to_string()))?;

    // For basic structure tokens, just return the content
    // More complex handling would go in specific structure types
    Ok(content.clone())
}
