//! Types for the implementation registry system
//!
//! This module defines the core types used throughout the implementation registry,
//! primarily for PrintConv/ValueConv classification. Function registry types are
//! defined in the function_registry module.

use crate::expression_compiler::CompiledExpression;

/// Classification of ValueConv expressions for code generation
#[derive(Debug, Clone)]
pub enum ValueConvType {
    /// Simple arithmetic expression that can be compiled to inline code
    CompiledExpression(CompiledExpression),
    /// Complex expression requiring a custom function
    CustomFunction(&'static str, &'static str), // (module_path, function_name)
}
