//! Function call normalization for AST transformation
//!
//! Transforms function calls without parentheses into consistent structure
//!
//! Single-node implementation: Focuses only on pattern recognition at the current node level.
//! Tree traversal is handled by the leaves-first orchestrator.

use crate::ppi::normalizer::{multi_pass::RewritePass, utils};
use crate::ppi::types::PpiNode;
use tracing::trace;

/// Normalizes function calls to consistent structure
pub struct FunctionCallNormalizer;

impl RewritePass for FunctionCallNormalizer {
    fn name(&self) -> &str {
        "FunctionCallNormalizer"
    }

    fn transform(&self, node: PpiNode) -> PpiNode {
        // Pattern: word followed by arguments (e.g., "length $val")
        // BUT NOT functions that already have proper parentheses (e.g., "abs($val)")
        if node.class == "PPI::Statement" && node.children.len() >= 2 {
            if let Some(func_name) = self.extract_function_name(&node.children[0]) {
                if self.is_known_function(&func_name) {
                    // Skip if function is followed by PPI::Structure::List - already properly structured
                    if node.children.len() >= 2 && node.children[1].class == "PPI::Structure::List"
                    {
                        trace!(
                            "Skipping function call normalization for {} - already has parentheses",
                            func_name
                        );
                        return node;
                    }

                    trace!("Found function call pattern: {}", func_name);

                    // Collect arguments (everything after the function name)
                    let args: Vec<PpiNode> = node.children.iter().skip(1).cloned().collect();

                    return utils::create_function_call(&func_name, args);
                }
            }
        }

        // No pattern matched, return unchanged
        node
    }
}

impl FunctionCallNormalizer {
    fn extract_function_name(&self, node: &PpiNode) -> Option<String> {
        if node.class == "PPI::Token::Word" {
            node.content.clone()
        } else {
            None
        }
    }

    fn is_known_function(&self, name: &str) -> bool {
        // Common Perl functions we want to normalize
        matches!(
            name,
            "length"
                | "int"
                | "sprintf"
                | "substr"
                | "index"
                | "join"
                | "split"
                | "unpack"
                | "pack"
                | "ord"
                | "chr"
                | "uc"
                | "lc"
                | "abs"
                | "sqrt"
                | "hex"
                | "oct"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_call_single_node() {
        let normalizer = FunctionCallNormalizer;

        // Test single-node transformation: length $val
        let function_call_node = PpiNode {
            class: "PPI::Statement".to_string(),
            content: None,
            children: vec![
                PpiNode {
                    class: "PPI::Token::Word".to_string(),
                    content: Some("length".to_string()),
                    children: vec![],
                    symbol_type: None,
                    numeric_value: None,
                    string_value: None,
                    structure_bounds: None,
                },
                PpiNode {
                    class: "PPI::Token::Symbol".to_string(),
                    content: Some("$val".to_string()),
                    children: vec![],
                    symbol_type: Some("scalar".to_string()),
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

        let result = RewritePass::transform(&normalizer, function_call_node);

        // Should transform to FunctionCall
        assert_eq!(result.class, "FunctionCall");
        assert_eq!(result.content, Some("length".to_string()));
        assert_eq!(result.children.len(), 1);
        assert_eq!(result.children[0].content, Some("$val".to_string()));
    }

    #[test]
    fn test_function_call_single_node_no_recursion() {
        let normalizer = FunctionCallNormalizer;

        // Test that single-node doesn't recurse into children
        let non_function_node = PpiNode {
            class: "PPI::Statement".to_string(),
            content: None,
            children: vec![PpiNode {
                class: "PPI::Token::Symbol".to_string(),
                content: Some("$other".to_string()),
                children: vec![],
                symbol_type: Some("scalar".to_string()),
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            }],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        let result = RewritePass::transform(&normalizer, non_function_node.clone());

        // Should return unchanged (no recursion into children)
        assert_eq!(result.class, "PPI::Statement");
        assert_eq!(format!("{:?}", result), format!("{:?}", non_function_node));
    }
}
