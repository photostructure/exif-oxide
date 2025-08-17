//! Safe division pattern normalization for AST transformation
//!
//! Transforms patterns like `$val ? 1/$val : 0` into safe function calls

use crate::ppi::normalizer::{utils, NormalizationPass, PrecedenceLevel};
use crate::ppi::types::PpiNode;
use tracing::trace;

/// Normalizes safe division patterns like `$val ? 1/$val : 0`
pub struct SafeDivisionNormalizer;

impl NormalizationPass for SafeDivisionNormalizer {
    fn name(&self) -> &str {
        "SafeDivisionNormalizer"
    }

    fn precedence_level(&self) -> PrecedenceLevel {
        PrecedenceLevel::High // Level 1-18 - specific ternary patterns for mathematical operations
    }

    fn transform(&self, node: PpiNode) -> PpiNode {
        // Pattern: $val ? N / $val : 0
        if utils::is_ternary(&node) {
            if let Some((condition, true_branch, false_branch)) = utils::extract_ternary(&node) {
                // Check if this matches safe division pattern
                if self.matches_safe_division(&condition, &true_branch, &false_branch) {
                    trace!("Found safe division pattern");

                    // Extract numerator from true branch
                    let numerator = self.extract_numerator(&true_branch);
                    let denominator = condition[0].clone(); // The condition variable

                    // Special case: if numerator is 1, use safe_reciprocal
                    if self.is_one(&numerator) {
                        return utils::create_function_call("safe_reciprocal", vec![denominator]);
                    }

                    // General case: safe_division
                    return utils::create_function_call(
                        "safe_division",
                        vec![numerator, denominator],
                    );
                }
            }
        }

        // Recurse into children
        utils::transform_children(node, |child| self.transform(child))
    }
}

impl SafeDivisionNormalizer {
    fn matches_safe_division(
        &self,
        condition: &[PpiNode],
        true_branch: &[PpiNode],
        false_branch: &[PpiNode],
    ) -> bool {
        // Condition should be a single variable (e.g., $val)
        if condition.len() != 1 || condition[0].class != "PPI::Token::Symbol" {
            return false;
        }

        // False branch should be 0
        if false_branch.len() != 1 {
            return false;
        }
        if let Some(num_val) = false_branch[0].numeric_value {
            if num_val != 0.0 {
                return false;
            }
        } else {
            return false;
        }

        // True branch should be division with same variable
        if true_branch.len() < 3 {
            return false;
        }

        // Look for division operator
        let has_division = true_branch.iter().any(|node| {
            node.class == "PPI::Token::Operator" && node.content.as_deref() == Some("/")
        });

        if !has_division {
            return false;
        }

        // Check that denominator matches condition variable
        let cond_var = &condition[0].content;
        true_branch
            .iter()
            .any(|node| node.class == "PPI::Token::Symbol" && &node.content == cond_var)
    }

    fn extract_numerator(&self, true_branch: &[PpiNode]) -> PpiNode {
        // Find the number before the division operator
        for (i, node) in true_branch.iter().enumerate() {
            if node.class == "PPI::Token::Operator" && node.content.as_deref() == Some("/") {
                if i > 0 {
                    return true_branch[i - 1].clone();
                }
            }
        }

        // Default to 1 if not found
        PpiNode {
            class: "PPI::Token::Number".to_string(),
            content: Some("1".to_string()),
            children: vec![],
            symbol_type: None,
            numeric_value: Some(1.0),
            string_value: None,
            structure_bounds: None,
        }
    }

    fn is_one(&self, node: &PpiNode) -> bool {
        node.numeric_value == Some(1.0) || node.content.as_deref() == Some("1")
    }
}
