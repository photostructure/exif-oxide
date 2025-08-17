//! Function call normalization for AST transformation
//!
//! Transforms function calls without parentheses into consistent structure

use crate::ppi::normalizer::{utils, NormalizationPass, PrecedenceLevel};
use crate::ppi::types::PpiNode;
use tracing::trace;

/// Normalizes function calls to consistent structure
pub struct FunctionCallNormalizer;

impl NormalizationPass for FunctionCallNormalizer {
    fn name(&self) -> &str {
        "FunctionCallNormalizer"
    }

    fn precedence_level(&self) -> PrecedenceLevel {
        PrecedenceLevel::Low // Level 22+ - list operators without parentheses
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
                        // Recurse into children without transforming this level
                        return utils::transform_children(node, |child| self.transform(child));
                    }

                    trace!("Found function call pattern: {}", func_name);

                    // Collect arguments (everything after the function name)
                    let args: Vec<PpiNode> = node.children.iter().skip(1).cloned().collect();

                    return utils::create_function_call(&func_name, args);
                }
            }
        }

        // Recurse into children
        utils::transform_children(node, |child| self.transform(child))
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
