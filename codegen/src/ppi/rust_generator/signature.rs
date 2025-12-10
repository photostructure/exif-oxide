//! Function signature generation for different expression types
//!
//! Handles type-specific signature generation for Condition, ValueConv, and PrintConv expressions.
//! Supports both Regular context (single TagValue) and Composite context (vals/prts/raws slices).
//!
//! Trust ExifTool: Preserves exact parameter and return type requirements.

use crate::ppi::types::{ExpressionContext, ExpressionType};

/// Generate function signature for different expression types (Regular context)
pub fn generate_signature(expression_type: &ExpressionType, function_name: &str) -> String {
    generate_signature_with_context(expression_type, &ExpressionContext::Regular, function_name)
}

/// Generate function signature with explicit context
///
/// For Regular context (single tag ValueConv/PrintConv):
///   `fn(val: &TagValue, ctx: Option<&ExifContext>) -> ...`
///
/// For Composite context (composite tag expressions with $val[n], $prt[n], $raw[n]):
///   `fn(vals: &[TagValue], prts: &[TagValue], raws: &[TagValue], ctx: Option<&ExifContext>) -> ...`
///
/// See docs/todo/P03c-composite-tags.md for composite function signature design.
pub fn generate_signature_with_context(
    expression_type: &ExpressionType,
    expression_context: &ExpressionContext,
    function_name: &str,
) -> String {
    match expression_context {
        ExpressionContext::Regular => generate_regular_signature(expression_type, function_name),
        ExpressionContext::Composite => {
            generate_composite_signature(expression_type, function_name)
        }
    }
}

/// Generate regular (single-value) function signature
fn generate_regular_signature(expression_type: &ExpressionType, function_name: &str) -> String {
    let return_type = expression_type.return_type();

    match expression_type {
        ExpressionType::Condition => {
            format!(
                "pub fn {function_name}(val: &TagValue, ctx: Option<&ExifContext>) -> {return_type}"
            )
        }
        ExpressionType::ValueConv => {
            format!(
                "pub fn {function_name}(val: &TagValue, ctx: Option<&ExifContext>) -> Result<TagValue, crate::core::types::ExifError>"
            )
        }
        ExpressionType::PrintConv => {
            format!(
                "pub fn {function_name}(val: &TagValue, ctx: Option<&ExifContext>) -> {return_type}"
            )
        }
    }
}

/// Generate composite function signature
///
/// Composite tags receive three parallel slices:
/// - `vals`: TagValues from resolved dependencies (after ValueConv)
/// - `prts`: TagValues from resolved dependencies (after PrintConv)
/// - `raws`: TagValues from resolved dependencies (raw unconverted)
///
/// These correspond to Perl's `@val`, `@prt`, `@raw` arrays in composite expressions.
/// See: lib/Image/ExifTool.pm:3553-3560 for ExifTool's array population.
///
/// Note: The `#[allow(unused_variables)]` attribute is added to each generated function
/// in composite_tag.rs to suppress warnings for unused parameters.
fn generate_composite_signature(expression_type: &ExpressionType, function_name: &str) -> String {
    // Composite functions always return Result<TagValue> to handle potential errors
    // from dependency resolution or calculation failures
    //
    // Allow attributes suppress various lints for generated code:
    // - unused_variables: most functions don't use all params
    // - clippy::get_first: some patterns generate .get(0)
    // - clippy::collapsible_else_if: nested conditionals from ternary translation
    // - clippy::blocks_in_conditions: regex blocks in if conditions
    // - clippy::unnecessary_cast: numeric literal casts
    // - clippy::redundant_clone: defensive cloning
    let allow_attrs = "#[allow(unused_variables, clippy::get_first, clippy::collapsible_else_if, clippy::blocks_in_conditions, clippy::unnecessary_cast, clippy::redundant_clone)]";

    match expression_type {
        ExpressionType::ValueConv => {
            format!(
                "{allow_attrs}\npub fn {function_name}(vals: &[TagValue], prts: &[TagValue], raws: &[TagValue], ctx: Option<&ExifContext>) -> Result<TagValue, crate::core::types::ExifError>"
            )
        }
        ExpressionType::PrintConv => {
            // PrintConv in composite context also takes the slices but returns Result
            // to maintain consistency with CompositeValueConvFn/CompositePrintConvFn types
            format!(
                "{allow_attrs}\npub fn {function_name}(vals: &[TagValue], prts: &[TagValue], raws: &[TagValue], ctx: Option<&ExifContext>) -> Result<TagValue, crate::core::types::ExifError>"
            )
        }
        ExpressionType::Condition => {
            // Conditions in composite context are rare but should use the same signature pattern
            format!(
                "{allow_attrs}\npub fn {function_name}(vals: &[TagValue], prts: &[TagValue], raws: &[TagValue], ctx: Option<&ExifContext>) -> bool"
            )
        }
    }
}
