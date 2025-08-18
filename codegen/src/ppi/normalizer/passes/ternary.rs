//! General ternary operator normalization for AST transformation
//!
//! Transforms patterns like `condition ? true_expr : false_expr` into normalized AST nodes
//! BECAUSE RUST DOES NOT HAVE TERNARY SUPPORT whaaaaaat

use crate::ppi::normalizer::{multi_pass::RewritePass, utils};
use crate::ppi::types::PpiNode;
use tracing::trace;

/// Normalizes general ternary expressions like `condition ? true_expr : false_expr`
pub struct TernaryNormalizer;

impl RewritePass for TernaryNormalizer {
    fn name(&self) -> &str {
        "TernaryNormalizer"
    }

    fn transform(&self, node: PpiNode) -> PpiNode {
        // Only process ternary expressions that weren't already handled by SafeDivisionNormalizer
        if utils::is_ternary(&node) {
            if let Some((condition, true_branch, false_branch)) = utils::extract_ternary(&node) {
                trace!("Found general ternary pattern");

                // Create a normalized ternary AST node
                return self.create_ternary_node(condition, true_branch, false_branch);
            }
        }

        // No transformation - return unchanged
        node
    }
}

impl TernaryNormalizer {
    /// Create a normalized ternary AST node
    fn create_ternary_node(
        &self,
        condition: Vec<PpiNode>,
        true_branch: Vec<PpiNode>,
        false_branch: Vec<PpiNode>,
    ) -> PpiNode {
        // Create a custom normalized node for ternary expressions
        PpiNode {
            class: "TernaryOp".to_string(),
            content: None,
            children: vec![
                // Child 0: condition
                if condition.len() == 1 {
                    condition[0].clone()
                } else {
                    PpiNode {
                        class: "Condition".to_string(),
                        content: None,
                        children: condition,
                        symbol_type: None,
                        numeric_value: None,
                        string_value: None,
                        structure_bounds: None,
                    }
                },
                // Child 1: true branch
                if true_branch.len() == 1 {
                    true_branch[0].clone()
                } else {
                    PpiNode {
                        class: "TrueBranch".to_string(),
                        content: None,
                        children: true_branch,
                        symbol_type: None,
                        numeric_value: None,
                        string_value: None,
                        structure_bounds: None,
                    }
                },
                // Child 2: false branch
                if false_branch.len() == 1 {
                    false_branch[0].clone()
                } else {
                    PpiNode {
                        class: "FalseBranch".to_string(),
                        content: None,
                        children: false_branch,
                        symbol_type: None,
                        numeric_value: None,
                        string_value: None,
                        structure_bounds: None,
                    }
                },
            ],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        }
    }
}
