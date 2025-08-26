//! Multi-Pass AST Rewriter Architecture
//!
//! This module implements a clean multi-pass approach to AST normalization that eliminates
//! the complexity of precedence levels in favor of explicit ordering. Each pass focuses on
//! a single transformation responsibility and can handle multi-token patterns.
//!
//! Key benefits over single-pass system:
//! - Multi-token pattern recognition (e.g., join + unpack sequences)
//! - Explicit pass ordering follows LLVM patterns (simpler than precedence levels)
//! - Each pass has single responsibility and clear name
//! - Easy to add new passes without complex precedence decisions
//! - Debug output shows exactly which passes run in what order
//!
//! Architecture: Central orchestrator applies simple transformation passes in declared order
//! using post-order traversal to ensure children are processed before parents.

use crate::ppi::types::PpiNode;
use tracing::{debug, trace};

/// Trait for individual rewrite passes that transform AST nodes
///
/// This trait replaces the complex `NormalizationPass` trait to eliminate artificial
/// precedence level constraints. Each pass has a single responsibility and operates
/// in the declared order within the multi-pass system.
///
/// # Usage Example
/// ```rust,no_run
/// use codegen::ppi::normalizer::multi_pass::RewritePass;
/// use codegen::ppi::PpiNode;
///
/// struct ExamplePass;
///
/// impl RewritePass for ExamplePass {
///     fn name(&self) -> &str {
///         "ExamplePass"
///     }
///
///     fn transform(&self, node: PpiNode) -> PpiNode {
///         // Transform this node and return result
///         // No recursion needed - orchestrator handles tree traversal
///         if node.class == "PPI::Token::Word" && node.content == Some("example".to_string()) {
///             PpiNode {
///                 class: "ExampleTransformed".to_string(),
///                 ..node
///             }
///         } else {
///             node // Return unchanged if no pattern matches
///         }
///     }
/// }
/// ```
pub trait RewritePass: Send + Sync {
    /// Name of this rewrite pass for debugging and logging
    fn name(&self) -> &str;

    /// Transform a node, potentially replacing patterns with canonical forms
    ///
    /// # Key Principles
    /// - **Single Responsibility**: Focus on one type of transformation
    /// - **Multi-token Awareness**: Can examine sibling nodes for complex patterns  
    /// - **No Recursion**: Orchestrator handles tree traversal
    /// - **Pattern Recognition**: Detect and transform specific AST structures
    ///
    /// # Parameters
    /// - `node`: The node to potentially transform (children already processed)
    ///
    /// # Returns
    /// - Same node (unchanged) if no pattern matches
    /// - New node with transformed structure if pattern matches
    fn transform(&self, node: PpiNode) -> PpiNode;
}

/// Multi-pass AST rewriter that applies transformation passes in explicit order
///
/// This orchestrator eliminates the complexity of precedence levels by using simple
/// Vec ordering. Each pass runs on every node in post-order traversal, ensuring
/// children are normalized before parents.
pub struct MultiPassRewriter {
    /// Passes to apply in declared order (no precedence levels needed)
    passes: Vec<Box<dyn RewritePass>>,
}

impl Default for MultiPassRewriter {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiPassRewriter {
    /// Create a new multi-pass rewriter with no passes (empty system)
    ///
    /// Passes must be added via `add_pass()` or use `with_standard_passes()`
    /// for the complete transformation pipeline.
    pub fn new() -> Self {
        Self { passes: Vec::new() }
    }

    /// Add a rewrite pass to the transformation pipeline
    ///
    /// Passes are executed in the order they are added - no precedence levels needed.
    /// This follows LLVM's "passes executed in order they were added" principle.
    pub fn add_pass(&mut self, pass: Box<dyn RewritePass>) {
        debug!("Adding rewrite pass: {}", pass.name());
        self.passes.push(pass);
    }

    /// Create rewriter with standard transformation passes in optimal order
    ///
    /// Pass ordering based on Perl operator precedence and pattern complexity:
    /// 1. Multi-token patterns (join + unpack combinations - MUST run before function normalization)
    /// 2. Function calls (establish proper boundaries after multi-token recognition)
    /// 3. Conditional statements (if/unless - converts control flow)
    /// 4. String operations (concatenation, repetition)
    /// 5. Mathematical operations (safe division)  
    /// 6. Ternary expressions (general ternary patterns)
    /// 7. Assignment patterns (sneaky conditional assignments)
    ///
    /// This ordering ensures function boundaries are established first, eliminating
    /// ambiguity about what constitutes function arguments vs separate expressions.
    pub fn with_standard_passes() -> Self {
        let mut rewriter = Self::new();

        // Add passes in explicit order based on Perl operator precedence
        // Multi-token patterns MUST run before single-function normalization
        rewriter.add_pass(Box::new(crate::ppi::normalizer::passes::JoinUnpackPass));
        rewriter.add_pass(Box::new(
            crate::ppi::normalizer::passes::FunctionCallNormalizer,
        ));
        rewriter.add_pass(Box::new(
            crate::ppi::normalizer::passes::ConditionalStatementsNormalizer,
        ));
        rewriter.add_pass(Box::new(crate::ppi::normalizer::passes::StringOpNormalizer));
        rewriter.add_pass(Box::new(
            crate::ppi::normalizer::passes::SafeDivisionNormalizer,
        ));
        rewriter.add_pass(Box::new(crate::ppi::normalizer::passes::TernaryNormalizer));
        rewriter.add_pass(Box::new(
            crate::ppi::normalizer::passes::SneakyConditionalAssignmentNormalizer,
        ));

        debug!(
            "Initialized multi-pass rewriter with {} passes",
            rewriter.passes.len()
        );

        for pass in &rewriter.passes {
            debug!("  - {}", pass.name());
        }

        rewriter
    }

    /// Apply multi-pass normalization using post-order traversal
    ///
    /// Each pass runs on every node after children have been processed.
    /// This ensures proper transformation order and allows multi-token
    /// pattern recognition across sibling nodes.
    pub fn normalize(&self, ast: PpiNode) -> PpiNode {
        debug!(
            "Starting multi-pass AST normalization with {} passes",
            self.passes.len()
        );
        let result = self.normalize_recursive(ast);
        debug!("Multi-pass AST normalization complete");
        result
    }

    /// Recursive implementation of post-order traversal with multi-pass application
    fn normalize_recursive(&self, node: PpiNode) -> PpiNode {
        trace!(
            "Processing node: {} with {} children",
            node.class,
            node.children.len()
        );

        // STEP 1: Recursively normalize all children first (post-order traversal)
        let normalized_children: Vec<PpiNode> = node
            .children
            .into_iter()
            .map(|child| self.normalize_recursive(child))
            .collect();

        // STEP 2: Create node with normalized children
        let node_with_normalized_children = PpiNode {
            children: normalized_children,
            ..node
        };

        // STEP 3: Apply all rewrite passes in declared order to this node
        // No sorting needed - explicit Vec ordering is cleaner than precedence levels
        let final_node =
            self.passes
                .iter()
                .fold(node_with_normalized_children, |current_node, pass| {
                    trace!("Applying rewrite pass: {}", pass.name());
                    let transformed = pass.transform(current_node);
                    trace!("Pass {} complete", pass.name());
                    transformed
                });

        trace!("Node processing complete: {}", final_node.class);
        final_node
    }
}

/// Public entry point for multi-pass AST normalization
///
/// This function provides the same API as the existing normalizers but uses
/// the multi-pass architecture internally. It can be used as a drop-in
/// replacement for `normalize_leaves_first()`.
pub fn normalize_multi_pass(ast: PpiNode) -> PpiNode {
    debug!("Normalizing AST using multi-pass approach");
    let rewriter = MultiPassRewriter::with_standard_passes();
    let result = rewriter.normalize(ast);
    debug!("Multi-pass normalization complete");
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test implementation for verifying multi-pass behavior
    struct TestPass {
        name: String,
        transform_count: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    }

    impl TestPass {
        fn new(name: &str) -> (Self, std::sync::Arc<std::sync::atomic::AtomicUsize>) {
            let counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let pass = TestPass {
                name: name.to_string(),
                transform_count: counter.clone(),
            };
            (pass, counter)
        }
    }

    impl RewritePass for TestPass {
        fn name(&self) -> &str {
            &self.name
        }

        fn transform(&self, node: PpiNode) -> PpiNode {
            self.transform_count
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

            // Simple test transformation: add pass name to content
            if node.class == "PPI::Token::Word" {
                PpiNode {
                    content: node.content.map(|c| format!("{}_{}", c, self.name)),
                    ..node
                }
            } else {
                node
            }
        }
    }

    #[test]
    fn test_multi_pass_framework() {
        let (pass1, counter1) = TestPass::new("Pass1");
        let (pass2, counter2) = TestPass::new("Pass2");

        let mut rewriter = MultiPassRewriter::new();
        rewriter.add_pass(Box::new(pass1));
        rewriter.add_pass(Box::new(pass2));

        let test_ast = PpiNode {
            class: "PPI::Token::Word".to_string(),
            content: Some("test".to_string()),
            children: vec![],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        let result = rewriter.normalize(test_ast);

        // Both passes should have run (once each)
        assert_eq!(counter1.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(counter2.load(std::sync::atomic::Ordering::SeqCst), 1);

        // Passes should run in order: Pass1 first, then Pass2
        assert_eq!(result.content, Some("test_Pass1_Pass2".to_string()));
    }

    #[test]
    fn test_post_order_traversal() {
        let (pass, counter) = TestPass::new("TestPass");

        let mut rewriter = MultiPassRewriter::new();
        rewriter.add_pass(Box::new(pass));

        // Create nested AST: parent with two children
        let nested_ast = PpiNode {
            class: "PPI::Statement".to_string(),
            content: None,
            children: vec![
                PpiNode {
                    class: "PPI::Token::Word".to_string(),
                    content: Some("child1".to_string()),
                    children: vec![],
                    symbol_type: None,
                    numeric_value: None,
                    string_value: None,
                    structure_bounds: None,
                },
                PpiNode {
                    class: "PPI::Token::Word".to_string(),
                    content: Some("child2".to_string()),
                    children: vec![],
                    symbol_type: None,
                    numeric_value: None,
                    string_value: None,
                    structure_bounds: None,
                },
            ],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        let result = rewriter.normalize(nested_ast);

        // Pass should run on parent + 2 children = 3 times total
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 3);

        // Children should be processed (content modified)
        assert_eq!(
            result.children[0].content,
            Some("child1_TestPass".to_string())
        );
        assert_eq!(
            result.children[1].content,
            Some("child2_TestPass".to_string())
        );
    }

    #[test]
    fn test_normalize_multi_pass_api() {
        // Test that public API works (even with empty pass list for now)
        let simple_ast = PpiNode {
            class: "PPI::Token::Symbol".to_string(),
            content: Some("$val".to_string()),
            children: vec![],
            symbol_type: Some("scalar".to_string()),
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        let result = normalize_multi_pass(simple_ast.clone());

        // Should preserve simple AST when no passes are configured
        assert_eq!(result.class, simple_ast.class);
        assert_eq!(result.content, simple_ast.content);
        assert_eq!(result.symbol_type, simple_ast.symbol_type);
    }

    #[test]
    fn test_empty_rewriter() {
        let rewriter = MultiPassRewriter::new();

        let test_ast = PpiNode {
            class: "PPI::Statement".to_string(),
            content: None,
            children: vec![],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        let result = rewriter.normalize(test_ast.clone());

        // Empty rewriter should return identical AST
        assert_eq!(format!("{:?}", result), format!("{:?}", test_ast));
    }
}
