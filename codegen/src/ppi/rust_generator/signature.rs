//! Function signature generation for different expression types
//!
//! Handles type-specific signature generation for Condition, ValueConv, and PrintConv expressions.
//! Trust ExifTool: Preserves exact parameter and return type requirements.

use crate::ppi::types::ExpressionType;

/// Generate function signature for different expression types
pub fn generate_signature(expression_type: &ExpressionType, function_name: &str) -> String {
    let return_type = expression_type.return_type();

    match expression_type {
        ExpressionType::Condition => {
            format!(
                "pub fn {}(val: &TagValue, ctx: &ExifContext) -> {}",
                function_name, return_type
            )
        }
        ExpressionType::ValueConv => {
            format!(
                "pub fn {}(val: &TagValue) -> Result<TagValue, crate::types::ExifError>",
                function_name
            )
        }
        ExpressionType::PrintConv => {
            format!(
                "pub fn {}(val: &TagValue) -> {}",
                function_name, return_type
            )
        }
    }
}
