//! Expression compiler for ValueConv arithmetic expressions
//!
//! This module provides compile-time parsing and code generation for simple
//! arithmetic expressions found in ExifTool ValueConv patterns. Uses the
//! Shunting Yard algorithm to convert infix expressions to Rust code.
//!
//! # Architecture
//!
//! The compilation pipeline consists of several stages:
//!
//! 1. **Tokenization** (`tokenizer.rs`) - Parse expression strings into tokens
//! 2. **Parsing** (`parser.rs`) - Convert infix tokens to RPN using Shunting Yard algorithm
//! 3. **Code Generation** (`codegen.rs`) - Generate optimized Rust code from RPN tokens
//! 4. **Types** (`types.rs`) - Core data types shared across modules
//!
//! # Supported Features
//!
//! - Basic arithmetic: `+`, `-`, `*`, `/`
//! - Parentheses for grouping: `(`, `)`
//! - Math functions: `int()`, `exp()`, `log()`
//! - Variable substitution: `$val`
//! - Numeric literals: `123`, `25.4`
//!
//! # Examples
//!
//! ```rust
//! use expression_compiler::CompiledExpression;
//!
//! // Simple arithmetic
//! let expr = CompiledExpression::compile("$val / 8").unwrap();
//! let code = expr.generate_rust_code();
//!
//! // Function calls
//! let expr = CompiledExpression::compile("int($val * 1000 / 25.4 + 0.5)").unwrap();
//! let code = expr.generate_rust_code();
//!
//! // Complex expressions
//! let expr = CompiledExpression::compile("exp($val/32*log(2))*100").unwrap();
//! let code = expr.generate_rust_code();
//! ```

pub mod types;
pub mod tokenizer;
pub mod parser;
pub mod codegen;

#[cfg(test)]
pub mod tests;

// Re-export the main API
pub use types::{CompiledExpression, AstNode, OpType, CompType, FuncType};
use tokenizer::tokenize;
// Remove the conflicting use statement - we'll call parser functions directly

impl CompiledExpression {
    /// Parse an ExifTool expression into AST form
    /// 
    /// Supports: $val, numbers, +, -, *, /, comparisons, ternary (?:), int(), exp(), log()
    /// Examples: "$val / 8", "$val >= 0 ? $val : undef", "int($val * 1000 / 25.4 + 0.5)"
    pub fn compile(expr: &str) -> Result<Self, String> {
        let tokens = tokenize(expr)?;
        let ast = crate::expression_compiler::parser::parse_expression(tokens)?;
        
        Ok(CompiledExpression {
            original_expr: expr.to_string(),
            ast: Box::new(ast),
        })
    }
    
    /// Check if this expression can be compiled (supports ternary, arithmetic, sprintf, and string concatenation)
    pub fn is_compilable(expr: &str) -> bool {
        // Quick checks for obviously non-compilable expressions  
        if expr.contains("**") || expr.contains("abs") || 
           expr.contains("IsFloat") || expr.contains("=~") || expr.contains("&") || 
           expr.contains("|") || expr.contains(">>") || expr.contains("<<") {
            return false;
        }
        
        // ExifTool function calls should be handled by conv_registry, not expression compiler
        if expr.contains("Image::ExifTool::") {
            return false;
        }
        
        // Check for supported sprintf patterns
        if expr.contains("sprintf(") {
            // sprintf patterns are compilable
            return Self::compile(expr).is_ok();
        }
        
        // Check for string concatenation patterns
        if expr.contains(" . ") {
            // String concatenation is compilable
            return Self::compile(expr).is_ok();
        }
        
        // Try to compile - if it works, it's compilable
        Self::compile(expr).is_ok()
    }
}