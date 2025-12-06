//! Binary operators and comparisons
//!
//! This module handles binary operations including arithmetic, comparison,
//! logical, and bitwise operators.

use crate::ppi::rust_generator::errors::CodeGenError;
use crate::ppi::types::*;

/// Get the precedence of an operator (lower number = lower precedence, splits first)
/// Based on Perl operator precedence table
fn get_operator_precedence(op: &str) -> i32 {
    match op {
        // Lowest precedence (split first)
        "or" => 10,
        "xor" => 15,
        "and" => 20,
        "||" => 30,
        "&&" => 40,
        "|" => 50,
        "^" => 60,
        "&" => 70,
        "==" | "!=" | "eq" | "ne" => 80,
        "<" | ">" | "<=" | ">=" | "lt" | "gt" | "le" | "ge" => 90,
        "<<" | ">>" => 100,
        "+" | "-" | "." => 110, // . is string concatenation
        "*" | "/" | "%" => 120,
        "=~" | "!~" => 130,
        "**" => 140, // Highest precedence (split last)
        _ => 100,    // Default to middle precedence
    }
}

/// Wrap bare integer/float literals with .into() for TagValue conversion.
/// Used when calling functions that expect TagValue arguments.
pub fn wrap_literal_for_tagvalue(s: &str) -> String {
    if s.ends_with("i32") || s.ends_with("f64") || s.ends_with("u32") {
        format!("{}.into()", s)
    } else {
        s.to_string()
    }
}

/// Check if a condition string is already a boolean expression (comparison, etc.)
fn is_boolean_expression(s: &str) -> bool {
    s.contains("==")
        || s.contains("!=")
        || s.contains("<=")
        || s.contains(">=")
        || s.contains(".is_truthy()")
        || s.contains(".is_empty()")
        // Simple < and > need special handling to avoid matching << and >>
        || (s.contains('<') && !s.contains("<<") && !s.contains("<="))
        || (s.contains('>') && !s.contains(">>") && !s.contains(">="))
}

/// Wrap a ternary condition with .is_truthy() if needed.
/// In Perl, `$val ? ... : ...` checks truthiness (non-zero, non-empty).
/// Also handles expressions like `($val & 0x01)` that return TagValue.
pub fn wrap_condition_for_bool(condition: &str) -> String {
    // Already a boolean expression - no wrapping needed
    if is_boolean_expression(condition) {
        return condition.to_string();
    }

    // Bare variable reference
    if condition == "val" || condition == "val_pt" {
        return format!("{}.is_truthy()", condition);
    }

    // Expressions involving val that produce TagValue need is_truthy()
    if condition.contains("val") || condition.contains("val_pt") {
        return format!("({}).is_truthy()", condition);
    }

    condition.to_string()
}

/// Wrap a ternary branch with appropriate conversion for ownership.
/// - Bare variable references need .clone()
/// - Bare integer/float literals need .into() for TagValue conversion
/// - String literals need .into() for TagValue conversion
pub fn wrap_branch_for_owned(branch: &str) -> String {
    if branch == "val" || branch == "val_pt" {
        format!("{}.clone()", branch)
    } else if branch.ends_with("i32") || branch.ends_with("u32") || branch.ends_with("f64") {
        // Bare integer/float literal - wrap with .into() for TagValue conversion
        format!("{}.into()", branch)
    } else if branch.starts_with('"') && branch.ends_with('"') {
        // String literal - wrap with .into() for TagValue conversion
        format!("{}.into()", branch)
    } else {
        branch.to_string()
    }
}

/// Trait for handling binary operations
pub trait BinaryOperationsHandler {
    fn expression_type(&self) -> &ExpressionType;

    /// Try to handle binary operation pattern
    /// Uses precedence-aware splitting to correctly handle expressions like `a * b ** c`
    fn try_binary_operation_pattern(
        &self,
        parts: &[String],
    ) -> Result<Option<String>, CodeGenError> {
        // Find the LOWEST precedence operator to split on
        // This ensures proper precedence handling (e.g., * before ** means we split on * first,
        // leaving ** to be processed in the right-hand side)
        let mut best_idx = None;
        let mut best_precedence = i32::MAX;

        for (i, part) in parts.iter().enumerate() {
            if self.is_binary_operator(part) && i > 0 && i < parts.len() - 1 {
                let prec = get_operator_precedence(part);
                // Use <= to prefer the leftmost operator at the same precedence (left-to-right associativity)
                if prec <= best_precedence {
                    best_precedence = prec;
                    best_idx = Some(i);
                }
            }
        }

        if let Some(i) = best_idx {
            let op = &parts[i];
            let left = parts[..i].join(" ");
            let right = parts[i + 1..].join(" ");
            #[cfg(test)]
            eprintln!(
                "DEBUG: binary op pattern - left: '{}', op: '{}', right: '{}'",
                left, op, right
            );
            let result = self.generate_binary_operation_from_parts(&left, op, &right)?;
            return Ok(Some(result));
        }
        Ok(None)
    }

    /// Recursively process an expression that may contain binary operators
    fn process_expression_recursively(&self, expr: &str) -> Result<String, CodeGenError> {
        // Split into parts and try to find binary operators
        let parts: Vec<String> = expr.split_whitespace().map(|s| s.to_string()).collect();
        if let Some(result) = self.try_binary_operation_pattern(&parts)? {
            Ok(result)
        } else {
            Ok(expr.to_string())
        }
    }

    /// Generate binary operation
    fn generate_binary_operation_from_parts(
        &self,
        left: &str,
        op: &str,
        right: &str,
    ) -> Result<String, CodeGenError> {
        // Recursively process operands that may contain binary operators
        let left_processed = self.process_expression_recursively(left)?;
        let right_processed = self.process_expression_recursively(right)?;

        // Handle power operator specially
        if op == "**" {
            // Power function takes TagValue args - wrap bare integer/float literals
            let left_wrapped = wrap_literal_for_tagvalue(&left_processed);
            let right_wrapped = wrap_literal_for_tagvalue(&right_processed);
            return Ok(format!("power({}, {})", left_wrapped, right_wrapped));
        }

        // Handle string concatenation operator
        if op == "." {
            // Use cleaner concat function
            return Ok(format!(
                "codegen_runtime::string::concat(&{}, &{})",
                left_processed, right_processed
            ));
        }

        // Handle regex match operators
        if op == "=~" || op == "!~" {
            return self.handle_regex_operation(&left_processed, op, &right_processed);
        }

        // Handle Perl string comparison operators (eq, ne, lt, gt, le, ge)
        // These require string conversion of operands, but be smart about string literals
        // Also handle already-converted operators when they involve TagValue comparisons
        let is_perl_string_op = matches!(op, "eq" | "ne" | "lt" | "gt" | "le" | "ge");
        let is_converted_string_op = matches!(op, "==" | "!=" | "<" | ">" | "<=" | ">=")
            && self.is_string_comparison(&left_processed, &right_processed);

        if is_perl_string_op || is_converted_string_op {
            let rust_op = if is_perl_string_op {
                self.perl_to_rust_operator(op)?
            } else {
                op.to_string() // Already converted
            };

            // Smart conversion: only add .to_string() to non-string-literal operands
            let left_converted = if self.is_string_literal_or_wrapped(&left_processed) {
                self.extract_string_literal(&left_processed)
            } else {
                format!("{}.to_string()", left_processed)
            };

            let right_converted = if self.is_string_literal_or_wrapped(&right_processed) {
                self.extract_string_literal(&right_processed)
            } else {
                format!("{}.to_string()", right_processed)
            };

            return Ok(format!(
                "{} {} {}",
                left_converted, rust_op, right_converted
            ));
        }

        let rust_op = self.perl_to_rust_operator(op)?;

        // Handle converted Perl string comparisons (== != < > <= >=) when involving TagValues
        if matches!(op, "==" | "!=" | "<" | ">" | "<=" | ">=")
            && self.is_string_comparison(&left_processed, &right_processed)
        {
            // Smart conversion: only add .to_string() to non-string-literal operands
            let left_converted = if self.is_string_literal_or_wrapped(&left_processed) {
                self.extract_string_literal(&left_processed)
            } else {
                format!("{}.to_string()", left_processed)
            };

            let right_converted = if self.is_string_literal_or_wrapped(&right_processed) {
                self.extract_string_literal(&right_processed)
            } else {
                format!("{}.to_string()", right_processed)
            };

            return Ok(format!(
                "{} {} {}",
                left_converted, rust_op, right_converted
            ));
        }

        Ok(format!(
            "{} {} {}",
            left_processed, rust_op, right_processed
        ))
    }

    /// Check if a string is a binary operator
    fn is_binary_operator(&self, s: &str) -> bool {
        matches!(
            s,
            "+" | "-"
                | "*"
                | "**"  // Power operator
                | "/"
                | "%"
                | "=="
                | "!="
                | "<"
                | ">"
                | "<="
                | ">="
                | "&&"
                | "||"
                | "&"
                | "|"
                | "^"
                | "<<"
                | ">>"
                | "eq"
                | "ne"
                | "lt"
                | "gt"
                | "le"
                | "ge"
                | "and"
                | "or"
                | "xor"
                | "=~"  // Regex match
                | "!~" // Regex no-match
                | "." // String concatenation
        )
    }

    /// Convert Perl operators to Rust
    fn perl_to_rust_operator(&self, op: &str) -> Result<String, CodeGenError> {
        Ok(match op {
            "eq" => "==",
            "ne" => "!=",
            "lt" => "<",
            "gt" => ">",
            "le" => "<=",
            "ge" => ">=",
            "and" => "&&",
            "or" => "||",
            "xor" => "^",
            _ => op,
        }
        .to_string())
    }

    /// Check if this is a string comparison (TagValue to string literal or similar)
    fn is_string_comparison(&self, left: &str, right: &str) -> bool {
        // If either operand is a string literal, treat as string comparison
        let left_is_string = left.starts_with('"') && left.ends_with('"');
        let right_is_string = right.starts_with('"') && right.ends_with('"');

        // Only treat as string comparison if at least one side is actually a string
        // Don't assume val comparisons are always strings - they could be numeric
        left_is_string || right_is_string
    }

    /// Handle regex matching operations - delegate to string operations
    fn handle_regex_operation(
        &self,
        left: &str,
        op: &str,
        right: &str,
    ) -> Result<String, CodeGenError>;

    /// Check if a string is a string literal or TagValue-wrapped string literal
    fn is_string_literal_or_wrapped(&self, s: &str) -> bool {
        // Direct string literal
        if s.starts_with('"') && s.ends_with('"') {
            return true;
        }

        // TagValue-wrapped string literal: Into::<TagValue>::into("literal")
        if s.starts_with("Into::<TagValue>::into(") && s.ends_with(")") {
            let inner = &s[23..s.len() - 1]; // Extract content between ( and )
            return inner.starts_with('"') && inner.ends_with('"');
        }

        false
    }

    /// Extract string literal from either direct form or TagValue wrapper
    fn extract_string_literal(&self, s: &str) -> String {
        // Direct string literal
        if s.starts_with('"') && s.ends_with('"') {
            return s.to_string();
        }

        // TagValue-wrapped string literal: Into::<TagValue>::into("literal")
        if s.starts_with("Into::<TagValue>::into(") && s.ends_with(")") {
            let inner = &s[23..s.len() - 1]; // Extract "literal" from ( "literal" )
            if inner.starts_with('"') && inner.ends_with('"') {
                return inner.to_string();
            }
        }

        // Fallback - shouldn't happen if is_string_literal_or_wrapped returned true
        s.to_string()
    }
}
