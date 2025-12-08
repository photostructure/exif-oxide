//! Expression combination logic split by operation type
//!
//! This module handles combining parsed PPI tokens into Rust expressions.
//! Most complex patterns are handled by the AST normalizer, so this only
//! needs to handle basic cases and normalized forms.

mod binary_ops;
mod normalized;
mod patterns;
mod string_ops;

pub use binary_ops::*;
pub use normalized::*;
pub use patterns::*;
pub use string_ops::*;

use super::errors::CodeGenError;
use crate::ppi::types::*;

/// Trait for combining expression parts into coherent Rust code
pub trait ExpressionCombiner:
    BinaryOperationsHandler + StringOperationsHandler + NormalizedAstHandler + ComplexPatternHandler
{
    /// Combine statement parts, handling normalized AST nodes and basic patterns
    fn combine_statement_parts(
        &self,
        parts: &[String],
        children: &[PpiNode],
    ) -> Result<String, CodeGenError> {
        if parts.is_empty() {
            return Ok("".to_string());
        }

        // Handle normalized AST nodes first
        if !children.is_empty() {
            match children[0].class.as_str() {
                "ConditionalBlock" => {
                    return self.handle_normalized_conditional_block(&children[0]);
                }
                "FunctionCall" => {
                    return self.handle_normalized_function_call(&children[0]);
                }
                "StringConcat" => {
                    return self.handle_normalized_string_concat(&children[0]);
                }
                "StringRepeat" => {
                    return self.handle_normalized_string_repeat(&children[0]);
                }
                _ => {}
            }
        }

        if parts.len() == 1 {
            return Ok(parts[0].clone());
        }

        // Delegate to specialized pattern handlers
        // NOTE: Pattern recognition for function calls without parentheses
        // Required for test compatibility and ExifTool expression handling

        // join/unpack patterns - handle both normalized and direct cases
        if let Some(result) = self.try_join_unpack_pattern(parts)? {
            return Ok(result);
        }

        // standalone unpack patterns
        if let Some(result) = self.try_unpack_pattern(parts)? {
            return Ok(result);
        }

        // log without parentheses
        if let Some(result) = self.try_log_pattern(parts)? {
            return Ok(result);
        }

        // length without parentheses
        if let Some(result) = self.try_length_pattern(parts)? {
            return Ok(result);
        }

        // Pattern: pack "C*", map { bit extraction } numbers...
        if let Some(result) = self.try_pack_map_pattern(parts, children)? {
            return Ok(result);
        }

        // Pattern: sprintf with string concatenation and repetition operations
        if let Some(result) = self.try_sprintf_string_ops_pattern(parts, children)? {
            return Ok(result);
        }

        // Pattern: basic sprintf arguments - format_string, comma, variable
        if let Some(result) = self.try_basic_sprintf_pattern(parts)? {
            return Ok(result);
        }

        // String concatenation pattern: expr . expr
        if let Some(result) = self.try_string_concat_pattern(parts)? {
            return Ok(result);
        }

        // Unary operations (!, -, etc.)
        if parts.len() == 2 {
            let op = &parts[0];
            let operand = &parts[1];

            if matches!(op.as_str(), "!" | "-" | "+" | "~") {
                let result = match op.as_str() {
                    "!" => format!("!({operand})"),
                    "-" => format!("-({operand})"),
                    "+" => format!("+({operand})"),
                    "~" => format!("!({operand})"), // Convert to boolean NOT
                    _ => unreachable!(),
                };
                return Ok(result);
            }
        }

        // Binary operations
        if let Some(result) = self.try_binary_operation_pattern(parts)? {
            return Ok(result);
        }

        // No pattern recognized - return error to trigger function registry fallback
        // This ensures unsupported expressions become placeholder functions that preserve
        // the original ExifTool expression instead of generating invalid Rust syntax
        Err(CodeGenError::UnsupportedStructure(format!(
            "No supported pattern found for expression parts: {parts:?}"
        )))
    }
}
