//! Condition expression parser for ExifTool-style conditions
//!
//! This module provides parsing functionality to convert string expressions
//! into structured Condition enums. The parser supports ExifTool's condition
//! syntax including regex matching, logical operators, comparisons, and more.

use crate::types::{ExifError, Result, TagValue};

use super::types::Condition;

/// Parse a condition expression into a structured condition
/// MILESTONE-14.5 Phase 2 - Full ExifTool condition parsing implementation
/// Supports comprehensive ExifTool condition syntax including data patterns,
/// logical operators, numeric comparisons, and complex expressions
pub fn parse_condition(expr: &str) -> Result<Condition> {
    let expr = expr.trim();

    // Handle parentheses for grouping
    if expr.starts_with('(') && expr.ends_with(')') {
        return parse_condition(&expr[1..expr.len() - 1]);
    }

    // Handle logical NOT operator
    if let Some(stripped) = expr.strip_prefix("not ") {
        let inner_condition = parse_condition(stripped)?;
        return Ok(Condition::Not(Box::new(inner_condition)));
    }
    if let Some(stripped) = expr.strip_prefix("!") {
        let inner_condition = parse_condition(stripped)?;
        return Ok(Condition::Not(Box::new(inner_condition)));
    }

    // Handle logical operators (and, or) with proper precedence
    // Parse OR first (lower precedence), then AND (higher precedence)
    if let Some(or_index) = find_operator_outside_parens(expr, " or ") {
        let left_expr = &expr[..or_index];
        let right_expr = &expr[or_index + 4..]; // " or " is 4 chars
        let left_condition = parse_condition(left_expr)?;
        let right_condition = parse_condition(right_expr)?;
        return Ok(Condition::Or(vec![left_condition, right_condition]));
    }

    if let Some(and_index) = find_operator_outside_parens(expr, " and ") {
        let left_expr = &expr[..and_index];
        let right_expr = &expr[and_index + 5..]; // " and " is 5 chars
        let left_condition = parse_condition(left_expr)?;
        let right_condition = parse_condition(right_expr)?;
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
        return parse_data_pattern_condition(expr);
    }

    // Handle regex patterns (=~ and !~)
    if expr.contains("=~") || expr.contains("!~") {
        return parse_regex_condition(expr);
    }

    // Handle numeric comparisons (>, <, >=, <=)
    if let Some(comparison_op) = find_comparison_operator(expr) {
        return parse_numeric_comparison(expr, &comparison_op);
    }

    // Handle equality and inequality comparisons (==, eq, !=, ne)
    if expr.contains("==") || expr.contains(" eq ") || expr.contains("!=") || expr.contains(" ne ")
    {
        return parse_equality_condition(expr);
    }

    // Handle hexadecimal number patterns (0x1234, 0X1234)
    if is_hex_number_condition(expr) {
        return parse_hex_condition(expr);
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
        let value = parse_value(value_part)?;

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
        let value = parse_value(value_part)?;

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
        return parse_equality_condition(expr);
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
