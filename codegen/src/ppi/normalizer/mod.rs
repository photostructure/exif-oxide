//! AST Normalizer for PPI Nodes
//!
//! Transforms PPI AST patterns into canonical forms before code generation.
//! This reduces the complexity of the expression generator from 730+ lines to <250.
//!
//! See docs/todo/P07-normalize-ast.md for the full technical plan.

use crate::ppi::types::PpiNode;
use tracing::{debug, trace};

pub mod passes;

/// Precedence levels based on Perl operator precedence (perlop)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrecedenceLevel {
    /// Level 1-18: High precedence - terms, arithmetic, comparison operators
    High,
    /// Level 19: Medium precedence - ternary conditional (?:)
    Medium,
    /// Level 22+: Low precedence - list operators without parentheses
    Low,
}

/// Main AST normalizer that applies transformation passes in precedence order
pub struct AstNormalizer {
    high_precedence_passes: Vec<Box<dyn NormalizationPass>>,
    medium_precedence_passes: Vec<Box<dyn NormalizationPass>>,
    low_precedence_passes: Vec<Box<dyn NormalizationPass>>,
}

/// Trait for individual normalization passes
pub trait NormalizationPass: Send + Sync {
    /// Name of this normalization pass for debugging
    fn name(&self) -> &str;

    /// Precedence level this pass operates at
    fn precedence_level(&self) -> PrecedenceLevel;

    /// Transform a PpiNode, potentially replacing patterns with canonical forms
    fn transform(&self, node: PpiNode) -> PpiNode;
}

impl Default for AstNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

impl AstNormalizer {
    /// Create a new normalizer with all passes configured in precedence order
    pub fn new() -> Self {
        // HIGH PRECEDENCE (Level 1-18): Terms, arithmetic, comparison operators
        // These operate on already-structured elements and don't interfere with function boundaries
        let high_precedence_passes: Vec<Box<dyn NormalizationPass>> = vec![
            Box::new(passes::SneakyConditionalAssignmentNormalizer), // Document-level patterns first
            Box::new(passes::PostfixConditionalNormalizer),          // Structural transformations
            Box::new(passes::StringOpNormalizer), // String operations - no precedence conflicts
            Box::new(passes::SafeDivisionNormalizer), // Specific ternary patterns for safe division
        ];

        // MEDIUM PRECEDENCE (Level 19): Ternary conditional (?:)
        // Must run after high-precedence operations but before function calls
        let medium_precedence_passes: Vec<Box<dyn NormalizationPass>> = vec![
            Box::new(passes::TernaryNormalizer), // General ternary patterns
        ];

        // LOW PRECEDENCE (Level 22+): List operators without parentheses
        // These transform function calls and must run LAST to avoid breaking higher-precedence operators
        let low_precedence_passes: Vec<Box<dyn NormalizationPass>> = vec![
            Box::new(passes::FunctionCallNormalizer), // Simple single-function calls without parentheses
            Box::new(passes::NestedFunctionNormalizer), // Nested functions without parentheses
            Box::new(passes::SprintfNormalizer),      // Complex sprintf patterns
        ];

        let total_passes = high_precedence_passes.len()
            + medium_precedence_passes.len()
            + low_precedence_passes.len();
        debug!(
            "Initialized precedence-based AST normalizer with {} passes",
            total_passes
        );

        debug!("HIGH precedence passes:");
        for pass in &high_precedence_passes {
            debug!("  - {}", pass.name());
        }
        debug!("MEDIUM precedence passes:");
        for pass in &medium_precedence_passes {
            debug!("  - {}", pass.name());
        }
        debug!("LOW precedence passes:");
        for pass in &low_precedence_passes {
            debug!("  - {}", pass.name());
        }

        Self {
            high_precedence_passes,
            medium_precedence_passes,
            low_precedence_passes,
        }
    }

    /// Apply all normalization passes in precedence order to transform the AST
    pub fn normalize(&self, ast: PpiNode) -> PpiNode {
        debug!("Starting precedence-based AST normalization");

        // Apply high precedence passes first (terms, arithmetic, comparison)
        let ast = self.apply_passes(&self.high_precedence_passes, ast, "HIGH");

        // Apply medium precedence passes (ternary conditional)
        let ast = self.apply_passes(&self.medium_precedence_passes, ast, "MEDIUM");

        // Apply low precedence passes last (function calls without parentheses)
        let ast = self.apply_passes(&self.low_precedence_passes, ast, "LOW");

        debug!("Precedence-based AST normalization complete");
        ast
    }

    /// Apply a set of passes with debug logging
    fn apply_passes(
        &self,
        passes: &[Box<dyn NormalizationPass>],
        ast: PpiNode,
        level: &str,
    ) -> PpiNode {
        debug!("Applying {} precedence passes", level);
        passes.iter().fold(ast, |node, pass| {
            debug!("Running {} precedence pass: {}", level, pass.name());
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
