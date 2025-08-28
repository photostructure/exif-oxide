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
                    // For sprintf with parentheses, only normalize if it's the ONLY thing in the statement
                    // If there are more children after the parentheses, this statement contains other operations
                    if func_name == "sprintf"
                        && node.children.len() >= 2
                        && node.children[1].class == "PPI::Structure::List"
                    {
                        // Check if this is JUST sprintf(...) or sprintf(...) followed by other operations
                        if node.children.len() > 2 {
                            // This statement has sprintf + other operations (like concatenation)
                            // Don't transform the whole statement - let the visitor handle it piece by piece
                            trace!(
                                "Skipping sprintf normalization - statement contains additional operations after sprintf"
                            );
                            return node;
                        }

                        // This is JUST sprintf(...) - safe to normalize
                        trace!("Normalizing standalone sprintf call");
                        let args = self.extract_args_from_parentheses(&node.children[1]);
                        return utils::create_function_call(&func_name, args);
                    }

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

    /// Extract function arguments from a PPI::Structure::List node (parentheses)
    fn extract_args_from_parentheses(&self, list_node: &PpiNode) -> Vec<PpiNode> {
        if list_node.class != "PPI::Structure::List" {
            return Vec::new();
        }

        // PPI::Structure::List contains expressions and comma separators
        // We need to collect the expression nodes and skip the commas
        let mut args = Vec::new();

        for child in &list_node.children {
            // Skip commas and collect expression nodes
            if child.class == "PPI::Statement::Expression" {
                // For expressions, we want the actual content nodes
                for expr_child in &child.children {
                    if expr_child.class != "PPI::Token::Operator"
                        || expr_child.content.as_deref() != Some(",")
                    {
                        args.push(expr_child.clone());
                    }
                }
            } else if child.class != "PPI::Token::Operator" {
                // Direct nodes that aren't comma operators
                args.push(child.clone());
            }
        }

        args
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

    #[test]
    fn test_sprintf_with_parentheses_normalization() {
        let normalizer = FunctionCallNormalizer;

        // Test sprintf("%.2f s", $val) normalization
        let sprintf_node = PpiNode {
            class: "PPI::Statement".to_string(),
            content: None,
            children: vec![
                PpiNode {
                    class: "PPI::Token::Word".to_string(),
                    content: Some("sprintf".to_string()),
                    children: vec![],
                    symbol_type: None,
                    numeric_value: None,
                    string_value: None,
                    structure_bounds: None,
                },
                PpiNode {
                    class: "PPI::Structure::List".to_string(),
                    content: Some("(".to_string()),
                    children: vec![PpiNode {
                        class: "PPI::Statement::Expression".to_string(),
                        content: None,
                        children: vec![
                            PpiNode {
                                class: "PPI::Token::Quote::Double".to_string(),
                                content: Some("\"%.2f s\"".to_string()),
                                string_value: Some("%.2f s".to_string()),
                                children: vec![],
                                symbol_type: None,
                                numeric_value: None,
                                structure_bounds: None,
                            },
                            PpiNode {
                                class: "PPI::Token::Operator".to_string(),
                                content: Some(",".to_string()),
                                children: vec![],
                                symbol_type: None,
                                numeric_value: None,
                                string_value: None,
                                structure_bounds: None,
                            },
                            PpiNode {
                                class: "PPI::Token::Symbol".to_string(),
                                content: Some("$val".to_string()),
                                symbol_type: Some("scalar".to_string()),
                                children: vec![],
                                numeric_value: None,
                                string_value: None,
                                structure_bounds: None,
                            },
                        ],
                        symbol_type: None,
                        numeric_value: None,
                        string_value: None,
                        structure_bounds: None,
                    }],
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

        let result = RewritePass::transform(&normalizer, sprintf_node);

        // Should transform to FunctionCall despite having parentheses
        assert_eq!(result.class, "FunctionCall");
        assert_eq!(result.content, Some("sprintf".to_string()));

        // Should have extracted arguments: format string and variable
        assert_eq!(result.children.len(), 2);
        assert_eq!(result.children[0].content, Some("\"%.2f s\"".to_string()));
        assert_eq!(result.children[1].content, Some("$val".to_string()));
    }
}
