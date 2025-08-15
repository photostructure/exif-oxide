//! AST Normalizer for PPI Nodes
//!
//! Transforms PPI AST patterns into canonical forms before code generation.
//! This reduces the complexity of the expression generator from 730+ lines to <250.
//!
//! See docs/todo/P07-normalize-ast.md for the full technical plan.

use crate::ppi::types::PpiNode;
use tracing::{debug, trace};

pub mod passes;

/// Main AST normalizer that applies transformation passes
pub struct AstNormalizer {
    passes: Vec<Box<dyn NormalizationPass>>,
}

/// Trait for individual normalization passes
pub trait NormalizationPass: Send + Sync {
    /// Name of this normalization pass for debugging
    fn name(&self) -> &str;

    /// Transform a PpiNode, potentially replacing patterns with canonical forms
    fn transform(&self, node: PpiNode) -> PpiNode;
}

impl Default for AstNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

impl AstNormalizer {
    /// Create a new normalizer with all passes configured
    pub fn new() -> Self {
        let passes: Vec<Box<dyn NormalizationPass>> = vec![
            // Phase 1: Syntax normalization (no dependencies)
            Box::new(passes::FunctionCallNormalizer), // Must run first
            Box::new(passes::StringOpNormalizer),     // Must run second
            // Phase 2: Pattern detection (depends on Phase 1)
            Box::new(passes::SafeDivisionNormalizer), // Depends on standard form
            // Phase 3: Complex patterns (depends on Phase 1 & 2)
            Box::new(passes::SprintfNormalizer), // Handles sprintf with string concat/repeat
        ];

        debug!("Initialized AST normalizer with {} passes", passes.len());
        for pass in &passes {
            debug!("  - {}", pass.name());
        }

        Self { passes }
    }

    /// Apply all normalization passes to transform the AST
    pub fn normalize(&self, ast: PpiNode) -> PpiNode {
        self.passes.iter().fold(ast, |node, pass| {
            debug!("Running normalization pass: {}", pass.name());
            let transformed = pass.transform(node);
            trace!("Pass {} complete", pass.name());
            transformed
        })
    }
}

/// Public entry point for AST normalization
pub fn normalize(ast: PpiNode) -> PpiNode {
    debug!("Normalizing AST");
    let normalizer = AstNormalizer::new();
    let result = normalizer.normalize(ast);
    debug!("AST normalization complete");
    result
}

/// Helper utilities for working with PpiNodes during normalization
pub(crate) mod utils {
    use super::*;

    /// Check if a node represents a ternary operator (? :)
    pub fn is_ternary(node: &PpiNode) -> bool {
        if node.class != "PPI::Statement" {
            return false;
        }

        let mut has_question = false;
        let mut has_colon = false;

        for child in &node.children {
            if child.class == "PPI::Token::Operator" {
                if let Some(ref content) = child.content {
                    if content == "?" {
                        has_question = true;
                    } else if content == ":" {
                        has_colon = true;
                    }
                }
            }
        }

        has_question && has_colon
    }

    /// Extract parts of a ternary expression
    pub fn extract_ternary(node: &PpiNode) -> Option<(Vec<PpiNode>, Vec<PpiNode>, Vec<PpiNode>)> {
        if !is_ternary(node) {
            return None;
        }

        let mut condition = Vec::new();
        let mut true_branch = Vec::new();
        let mut false_branch = Vec::new();
        let mut current_part = &mut condition;

        for child in &node.children {
            if child.class == "PPI::Token::Operator" {
                if let Some(ref content) = child.content {
                    if content == "?" {
                        current_part = &mut true_branch;
                        continue;
                    } else if content == ":" {
                        current_part = &mut false_branch;
                        continue;
                    }
                }
            }
            current_part.push(child.clone());
        }

        Some((condition, true_branch, false_branch))
    }

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

    /// Deep clone and transform children of a node
    pub fn transform_children<F>(node: PpiNode, transform: F) -> PpiNode
    where
        F: Fn(PpiNode) -> PpiNode,
    {
        PpiNode {
            children: node.children.into_iter().map(transform).collect(),
            ..node
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_normalization() {
        // Test that normalizer preserves simple AST when no patterns match
        let simple_ast = PpiNode {
            class: "PPI::Token::Symbol".to_string(),
            content: Some("$val".to_string()),
            children: vec![],
            symbol_type: Some("scalar".to_string()),
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        let normalized = normalize(simple_ast.clone());

        // Should be identical for simple cases
        assert_eq!(
            format!("{:?}", simple_ast),
            format!("{:?}", normalized),
            "Identity normalization should preserve simple AST"
        );
    }
}
