//! Multi-Pass AST Normalizer for PPI Nodes
//!
//! Transforms PPI AST patterns into canonical forms before code generation.
//! Uses a clean multi-pass architecture with explicit ordering.
//!
//! See docs/todo/P06-multi-pass-ast-rewriter.md for the technical plan.

use crate::ppi::types::PpiNode;

pub mod multi_pass;
pub mod passes;

/// Public entry point for AST normalization using multi-pass approach
/// This handles multi-token patterns like join+unpack that require pattern recognition
pub fn normalize_multi_pass(ast: PpiNode) -> PpiNode {
    multi_pass::normalize_multi_pass(ast)
}

/// Helper utilities for working with PpiNodes during normalization
pub(crate) mod utils {
    use super::*;

    /// Create a normalized function call node
    pub fn create_function_call(name: &str, args: Vec<PpiNode>) -> PpiNode {
        PpiNode {
            class: "FunctionCall".to_string(),
            content: Some(name.to_string()),
            children: args,
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_pass_normalization() {
        // Test that multi-pass normalizer preserves simple AST when no patterns match
        let simple_ast = PpiNode {
            class: "PPI::Token::Symbol".to_string(),
            content: Some("$val".to_string()),
            children: vec![],
            symbol_type: Some("scalar".to_string()),
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        let normalized = normalize_multi_pass(simple_ast.clone());

        // Should be identical for simple cases
        assert_eq!(
            format!("{:?}", simple_ast),
            format!("{:?}", normalized),
            "Multi-pass normalization should preserve simple AST"
        );
    }
}
