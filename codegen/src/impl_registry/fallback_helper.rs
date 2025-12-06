//! Shared fallback helper for impl_registry integration
//!
//! This module provides a unified interface for attempting registry lookups
//! when PPI generation fails. It generates complete function implementations
//! that call registry-provided implementations.
//!
//! Pattern: PPI generation → [FAIL] → Registry lookup → [FAIL] → Placeholder

use crate::impl_registry::{
    classify_valueconv_expression, lookup_function, lookup_printconv, ValueConvType,
};
use crate::ppi::ExpressionType;
use indoc::formatdoc;

/// Try registry lookup for a failed PPI expression
///
/// This function implements the unified fallback pattern used across the codebase:
/// 1. TagKit: process_print_conv() and process_value_conv() - lines 371-375, 426-444
/// 2. Visitor: visit_function_call() - lines 560+ for ExifTool functions
/// 3. fn_registry: This integration (filling the missing link)
///
/// Returns Some(complete_function_code) if registry has implementation, None otherwise
pub fn try_registry_lookup(
    expression_type: ExpressionType,
    original_expression: &str,
    module: &str,
    function_name: &str,
) -> Option<String> {
    match expression_type {
        ExpressionType::PrintConv => {
            if let Some((module_path, func_name)) = lookup_printconv(original_expression, module) {
                Some(generate_printconv_function(
                    function_name,
                    original_expression,
                    module_path,
                    func_name,
                ))
            } else {
                None
            }
        }
        ExpressionType::ValueConv => {
            match classify_valueconv_expression(original_expression, module) {
                ValueConvType::CustomFunction(module_path, func_name) => {
                    Some(generate_valueconv_function(
                        function_name,
                        original_expression,
                        module_path,
                        func_name,
                    ))
                }
                _ => None,
            }
        }
        ExpressionType::Condition => {
            if let Some(func_impl) = lookup_function(original_expression) {
                Some(generate_condition_function(
                    function_name,
                    original_expression,
                    func_impl,
                ))
            } else {
                None
            }
        }
    }
}

/// Generate complete PrintConv function implementation
/// Following TagKit pattern from tag_kit.rs:366-367
fn generate_printconv_function(
    function_name: &str,
    original_expression: &str,
    module_path: &str,
    func_name: &str,
) -> String {
    let escaped_expr = escape_expression(original_expression);
    // module_path already includes "crate::implementations::" prefix
    let function_path = format!("{}::{}", module_path, func_name);

    formatdoc! {r#"
        /// Registry fallback: PrintConv implementation found
        /// Original perl expression:
        /// ``` perl
        /// {}
        /// ```
        pub fn {}(val: &TagValue, ctx: Option<&ExifContext>) -> TagValue {{
            {}(val, ctx)
        }}
    "#, escaped_expr, function_name, function_path}
}

/// Generate complete ValueConv function implementation
/// Following TagKit pattern from tag_kit.rs:426-444
fn generate_valueconv_function(
    function_name: &str,
    original_expression: &str,
    module_path: &str,
    func_name: &str,
) -> String {
    let escaped_expr = escape_expression(original_expression);
    // module_path already includes "crate::implementations::" prefix
    let function_path = format!("{}::{}", module_path, func_name);

    formatdoc! {r#"
        /// Registry fallback: ValueConv implementation found
        /// Original perl expression:
        /// ``` perl
        /// {}
        /// ```
        pub fn {}(val: &TagValue, ctx: Option<&ExifContext>) -> Result<TagValue, codegen_runtime::types::ExifError> {{
            {}(val, ctx)
        }}
    "#, escaped_expr, function_name, function_path}
}

/// Generate complete Condition function implementation
/// Following visitor pattern from visitor.rs:560+
fn generate_condition_function(
    function_name: &str,
    original_expression: &str,
    func_impl: &crate::impl_registry::FunctionImplementation,
) -> String {
    use crate::impl_registry::FunctionImplementation;

    let escaped_expr = escape_expression(original_expression);
    let function_path = match func_impl {
        FunctionImplementation::Builtin(builtin) => {
            // module_path already includes "crate::implementations::" prefix
            format!("{}::{}", builtin.module_path, builtin.function_name)
        }
        FunctionImplementation::ExifToolModule(module_func) => {
            // module_path already includes "crate::implementations::" prefix
            format!("{}::{}", module_func.module_path, module_func.function_name)
        }
        FunctionImplementation::CustomScript(script) => {
            // module_path already includes "crate::implementations::" prefix
            format!("{}::{}", script.module_path, script.function_name)
        }
    };

    formatdoc! {r#"
        /// Registry fallback: Condition implementation found
        /// Original perl expression:
        /// ``` perl
        /// {}
        /// ```
        pub fn {}(val: &TagValue, ctx: Option<&ExifContext>) -> bool {{
            {}(val, ctx)
        }}
    "#, escaped_expr, function_name, function_path}
}

/// Escape Perl expression for use in Rust documentation
fn escape_expression(expr: &str) -> String {
    expr.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
