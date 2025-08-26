//! Binary operators and comparisons
//!
//! This module handles binary operations including arithmetic, comparison,
//! logical, and bitwise operators.

use crate::ppi::rust_generator::errors::CodeGenError;
use crate::ppi::types::*;

/// Trait for handling binary operations
pub trait BinaryOperationsHandler {
    fn expression_type(&self) -> &ExpressionType;

    /// Try to handle binary operation pattern
    fn try_binary_operation_pattern(
        &self,
        parts: &[String],
    ) -> Result<Option<String>, CodeGenError> {
        // Binary operations
        for (i, part) in parts.iter().enumerate() {
            if self.is_binary_operator(part) {
                if i > 0 && i < parts.len() - 1 {
                    let left = parts[..i].join(" ");
                    let right = parts[i + 1..].join(" ");
                    #[cfg(test)]
                    eprintln!(
                        "DEBUG: binary op pattern - left: '{}', op: '{}', right: '{}'",
                        left, part, right
                    );
                    let result = self.generate_binary_operation_from_parts(&left, part, &right)?;
                    return Ok(Some(result));
                }
            }
        }
        Ok(None)
    }

    /// Generate binary operation  
    fn generate_binary_operation_from_parts(
        &self,
        left: &str,
        op: &str,
        right: &str,
    ) -> Result<String, CodeGenError> {
        // Handle power operator specially
        if op == "**" {
            // Perl's ** operator is exponentiation, use powf in Rust
            return Ok(format!("({} as f64).powf({} as f64)", left, right));
        }

        // Handle string concatenation operator
        if op == "." {
            // Perl's . operator is string concatenation, use format! in Rust
            match self.expression_type() {
                ExpressionType::PrintConv | ExpressionType::ValueConv => {
                    return Ok(format!(
                        "TagValue::String(format!(\"{{}}{{}}\", {}, {}))",
                        left, right
                    ));
                }
                _ => {
                    return Ok(format!("format!(\"{{}}{{}}\", {}, {})", left, right));
                }
            }
        }

        // Handle regex match operators
        if op == "=~" || op == "!~" {
            return self.handle_regex_operation(left, op, right);
        }

        // Handle Perl string comparison operators (eq, ne, lt, gt, le, ge)
        // These require string conversion of operands, but be smart about string literals
        // Also handle already-converted operators when they involve TagValue comparisons
        let is_perl_string_op = matches!(op, "eq" | "ne" | "lt" | "gt" | "le" | "ge");
        let is_converted_string_op = matches!(op, "==" | "!=" | "<" | ">" | "<=" | ">=")
            && self.is_string_comparison(left, right);

        if is_perl_string_op || is_converted_string_op {
            let rust_op = if is_perl_string_op {
                self.perl_to_rust_operator(op)?
            } else {
                op.to_string() // Already converted
            };

            // Smart conversion: only add .to_string() to non-string-literal operands
            let left_converted = if left.starts_with('"') && left.ends_with('"') {
                left.to_string()
            } else {
                format!("{}.to_string()", left)
            };

            let right_converted = if right.starts_with('"') && right.ends_with('"') {
                right.to_string()
            } else {
                format!("{}.to_string()", right)
            };

            return Ok(format!(
                "{} {} {}",
                left_converted, rust_op, right_converted
            ));
        }

        let rust_op = self.perl_to_rust_operator(op)?;

        // Handle converted Perl string comparisons (== != < > <= >=) when involving TagValues
        if matches!(op, "==" | "!=" | "<" | ">" | "<=" | ">=")
            && self.is_string_comparison(left, right)
        {
            // Smart conversion: only add .to_string() to non-string-literal operands
            let left_converted = if left.starts_with('"') && left.ends_with('"') {
                left.to_string()
            } else {
                format!("{}.to_string()", left)
            };

            let right_converted = if right.starts_with('"') && right.ends_with('"') {
                right.to_string()
            } else {
                format!("{}.to_string()", right)
            };

            return Ok(format!(
                "{} {} {}",
                left_converted, rust_op, right_converted
            ));
        }

        Ok(format!("{} {} {}", left, rust_op, right))
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
}
