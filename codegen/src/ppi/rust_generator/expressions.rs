//! Expression combination and pattern recognition logic
//!
//! This module handles the complex logic for combining parsed PPI tokens
//! into coherent Rust expressions with proper operator precedence and
//! pattern recognition.

use super::errors::CodeGenError;
use crate::ppi::types::*;

/// Trait for combining expression parts into coherent Rust code
pub trait ExpressionCombiner {
    fn expression_type(&self) -> &ExpressionType;

    /// Combine statement parts, handling only essential Rust-specific conversions
    fn combine_statement_parts(
        &self,
        parts: &[String],
        children: &[PpiNode],
    ) -> Result<String, CodeGenError> {
        if parts.is_empty() {
            return Ok("".to_string());
        }

        if parts.len() == 1 {
            return Ok(parts[0].clone());
        }

        // Look for common patterns and handle them intelligently

        // Pattern: function_name ( args )
        if parts.len() >= 2 && children.len() >= 2 {
            if children[0].is_word() && children[1].class.contains("Structure") {
                return self.generate_function_call_from_parts(&parts[0], &parts[1]);
            }
        }

        // Pattern: multi-argument function calls (join " ", unpack "H2H2", val)
        if parts.len() >= 3 && children.len() >= 3 {
            if children[0].is_word() {
                let function_name = &parts[0];
                if matches!(
                    function_name.as_str(),
                    "join" | "unpack" | "pack" | "substr" | "split"
                ) {
                    // Parse arguments with parentheses awareness for nested function calls
                    let mut args = Vec::new();
                    let mut current_arg = Vec::new();
                    let mut paren_depth = 0;

                    for part in parts[1..].iter() {
                        if part.contains('(') {
                            paren_depth += part.matches('(').count();
                        }
                        if part.contains(')') {
                            paren_depth -= part.matches(')').count();
                        }

                        if part == "," && paren_depth == 0 {
                            // Only split on commas outside of parentheses
                            if !current_arg.is_empty() {
                                args.push(current_arg.join(" "));
                                current_arg.clear();
                            }
                        } else {
                            current_arg.push(part.clone());
                        }
                    }
                    // Add the last argument
                    if !current_arg.is_empty() {
                        args.push(current_arg.join(" "));
                    }

                    return self.generate_multi_arg_function_call(function_name, &args);
                }
            }
        }

        // Pattern: function_name arg (without parentheses, like "length $val")
        if parts.len() == 2 && children.len() == 2 {
            if children[0].is_word() {
                let function_name = &parts[0];
                if matches!(
                    function_name.as_str(),
                    "length" | "int" | "abs" | "sqrt" | "sin" | "cos" | "defined"
                ) {
                    return self.generate_function_call_without_parens(function_name, &parts[1]);
                }
            }
        }

        // Pattern: expr . expr (concatenation)
        if let Some(concat_pos) = parts.iter().position(|p| p == ".") {
            return self.generate_concatenation_from_parts(parts, concat_pos);
        }

        // Pattern: condition ? true_expr : false_expr (ternary)
        if let (Some(question_pos), Some(colon_pos)) = (
            parts.iter().position(|p| p == "?"),
            parts.iter().position(|p| p == ":"),
        ) {
            if question_pos < colon_pos {
                return self.generate_ternary_from_parts(parts, question_pos, colon_pos);
            }
        }

        // Default: trust PPI's AST structure and join parts appropriately
        Ok(parts.join(" "))
    }

    /// Combine expression parts with advanced pattern recognition
    fn combine_expression_parts(
        &self,
        parts: &[String],
        children: &[PpiNode],
    ) -> Result<String, CodeGenError> {
        if parts.is_empty() {
            return Ok("".to_string());
        }

        if parts.len() == 1 {
            return Ok(parts[0].clone());
        }

        // Pattern: function_name(args) - handle function calls with parentheses
        if parts.len() == 2 && children.len() == 2 {
            if children[0].is_word() && children[1].class == "PPI::Structure::List" {
                let function_name = &parts[0];
                let args_part = &parts[1];
                return self.generate_function_call_from_parts(function_name, args_part);
            }
        }

        // Pattern: left =~ regex (regex matching)
        if parts.len() == 3 && parts[1] == "=~" {
            return Ok(format!(
                "regex::Regex::new(r\"{}\").unwrap().is_match(&{}.to_string())",
                parts[2].trim_matches('/'),
                parts[0]
            ));
        }

        // Pattern: left !~ regex (negative regex matching)
        if parts.len() == 3 && parts[1] == "!~" {
            return Ok(format!(
                "!regex::Regex::new(r\"{}\").unwrap().is_match(&{}.to_string())",
                parts[2].trim_matches('/'),
                parts[0]
            ));
        }

        // Pattern: arithmetic operations with TagValue
        if parts.len() == 3 {
            match parts[1].as_str() {
                "/" => return Ok(format!("&{} / {}", parts[0], parts[2])),
                "*" => return Ok(format!("&{} * {}", parts[0], parts[2])),
                "+" => return Ok(format!("&{} + {}", parts[0], parts[2])),
                "-" => return Ok(format!("&{} - {}", parts[0], parts[2])),
                "eq" => {
                    return Ok(format!(
                        "{}.to_string() == {}.to_string()",
                        parts[0], parts[2]
                    ))
                }
                "ne" => {
                    return Ok(format!(
                        "{}.to_string() != {}.to_string()",
                        parts[0], parts[2]
                    ))
                }
                _ => {}
            }
        }

        // Pattern: condition ? true_expr : false_expr (ternary)
        if let (Some(question_pos), Some(colon_pos)) = (
            parts.iter().position(|p| p == "?"),
            parts.iter().position(|p| p == ":"),
        ) {
            if question_pos < colon_pos {
                return self.generate_ternary_from_parts(parts, question_pos, colon_pos);
            }
        }

        // Fall back to the existing statement combination logic
        self.combine_statement_parts(parts, children)
    }

    /// Generate concatenation from parts array
    fn generate_concatenation_from_parts(
        &self,
        parts: &[String],
        concat_pos: usize,
    ) -> Result<String, CodeGenError> {
        let left = parts[..concat_pos].join(" ");
        let right = parts[concat_pos + 1..].join(" ");

        match self.expression_type() {
            ExpressionType::PrintConv => Ok(format!(
                "/* expression printconv */ TagValue::String(format!(\"{{}}{{}}\", {}, {}))",
                left, right
            )),
            ExpressionType::ValueConv => Ok(format!(
                "/* expression valueconv */ TagValue::String(format!(\"{{}}{{}}\", {}, {}))",
                left, right
            )),
            _ => Ok(format!(
                "/* expression fallthrough */ format!(\"{{}}{{}}\", {}, {})",
                left, right
            )),
        }
    }

    /// Generate ternary conditional from parts
    fn generate_ternary_from_parts(
        &self,
        parts: &[String],
        question_pos: usize,
        colon_pos: usize,
    ) -> Result<String, CodeGenError> {
        let condition_parts = &parts[..question_pos];
        let true_branch_parts = &parts[question_pos + 1..colon_pos];
        let false_branch_parts = &parts[colon_pos + 1..];

        // Process condition - look for binary operations within it
        let condition = if condition_parts.len() == 3 {
            // Try to handle as binary operation (e.g., "val eq inf")
            self.generate_binary_operation_from_parts(
                &condition_parts[0],
                &condition_parts[1],
                &condition_parts[2],
            )?
        } else {
            condition_parts.join(" ")
        };

        let true_branch = true_branch_parts.join(" ");
        let false_branch = false_branch_parts.join(" ");

        Ok(format!(
            "if {} {{ {} }} else {{ {} }}",
            condition, true_branch, false_branch
        ))
    }

    /// Generate binary operation from three parts
    fn generate_binary_operation_from_parts(
        &self,
        left: &str,
        op: &str,
        right: &str,
    ) -> Result<String, CodeGenError> {
        let rust_op = match op {
            "eq" => "==",
            "ne" => "!=",
            "=~" => ".contains",  // Simplified regex matching
            "!~" => "!.contains", // Simplified regex matching
            "/" => "/",
            "*" => "*",
            "+" => "+",
            "-" => "-",
            ">" => ">",
            "<" => "<",
            ">=" => ">=",
            "<=" => "<=",
            _ => op,
        };

        match self.expression_type() {
            ExpressionType::Condition => {
                // Generate boolean expression for conditions
                if op == "=~" || op == "!~" {
                    let negate = op == "!~";
                    Ok(format!(
                        "{}{}.to_string().contains(&{}.to_string())",
                        if negate { "!" } else { "" },
                        left,
                        right
                    ))
                } else {
                    Ok(format!("{} {} {}", left, rust_op, right))
                }
            }
            ExpressionType::PrintConv | ExpressionType::ValueConv => {
                // For PrintConv/ValueConv, handle TagValue comparisons and arithmetic
                match rust_op {
                    "==" | "!=" => {
                        // Handle TagValue string comparisons
                        Ok(format!(
                            "{}.to_string() {} {}.to_string()",
                            left, rust_op, right
                        ))
                    }
                    "/" | "*" | "+" | "-" => {
                        // Ensure floating point arithmetic
                        Ok(format!("({} as f64) {} ({} as f64)", left, rust_op, right))
                    }
                    _ => Ok(format!("{} {} {}", left, rust_op, right)),
                }
            }
        }
    }

    // Function generation methods (delegated to FunctionGenerator trait)
    fn generate_function_call_from_parts(
        &self,
        function_name: &str,
        args_part: &str,
    ) -> Result<String, CodeGenError>;

    fn generate_multi_arg_function_call(
        &self,
        function_name: &str,
        args: &[String],
    ) -> Result<String, CodeGenError>;

    fn generate_function_call_without_parens(
        &self,
        function_name: &str,
        arg: &str,
    ) -> Result<String, CodeGenError>;
}
