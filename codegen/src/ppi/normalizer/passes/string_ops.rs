//! String operation normalization for AST transformation
//!
//! Transforms string concatenation (.) and repetition (x) into canonical forms

use crate::ppi::normalizer::{utils, NormalizationPass, PrecedenceLevel};
use crate::ppi::types::PpiNode;
use tracing::trace;

/// Normalizes string operations (concatenation and repetition)
pub struct StringOpNormalizer;

impl NormalizationPass for StringOpNormalizer {
    fn name(&self) -> &str {
        "StringOpNormalizer"
    }

    fn precedence_level(&self) -> PrecedenceLevel {
        PrecedenceLevel::High // Level 1-18 - string operations, no precedence conflicts
    }

    fn transform(&self, node: PpiNode) -> PpiNode {
        if node.class == "PPI::Statement" {
            // Look for string concatenation (.)
            if let Some(concat_result) = self.try_normalize_concat(&node) {
                return concat_result;
            }

            // Look for string repetition (x)
            if let Some(repeat_result) = self.try_normalize_repeat(&node) {
                return repeat_result;
            }
        }

        // Recurse into children
        utils::transform_children(node, |child| self.transform(child))
    }
}

impl StringOpNormalizer {
    fn try_normalize_concat(&self, node: &PpiNode) -> Option<PpiNode> {
        // Find concatenation operator positions
        let concat_positions: Vec<usize> = node
            .children
            .iter()
            .enumerate()
            .filter_map(|(i, child)| {
                if child.class == "PPI::Token::Operator" && child.content.as_deref() == Some(".") {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();

        if concat_positions.is_empty() {
            return None;
        }

        trace!("Found string concatenation pattern");

        // Collect all operands between dots
        let mut operands = Vec::new();
        let mut start = 0;

        for &dot_pos in &concat_positions {
            // Collect all nodes from start to dot_pos as one operand
            let operand_children: Vec<PpiNode> = node.children[start..dot_pos].to_vec();

            if operand_children.len() == 1 {
                operands.push(operand_children[0].clone());
            } else if !operand_children.is_empty() {
                // Wrap multiple nodes in a Statement
                operands.push(PpiNode {
                    class: "PPI::Statement".to_string(),
                    content: None,
                    children: operand_children,
                    symbol_type: None,
                    numeric_value: None,
                    string_value: None,
                    structure_bounds: None,
                });
            }
            start = dot_pos + 1;
        }

        // Add final operand after last dot
        let final_children: Vec<PpiNode> = node.children[start..].to_vec();
        if final_children.len() == 1 {
            operands.push(final_children[0].clone());
        } else if !final_children.is_empty() {
            operands.push(PpiNode {
                class: "PPI::Statement".to_string(),
                content: None,
                children: final_children,
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            });
        }

        Some(PpiNode {
            class: "StringConcat".to_string(),
            content: None,
            children: operands,
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        })
    }

    fn try_normalize_repeat(&self, node: &PpiNode) -> Option<PpiNode> {
        // Find repetition operator (x)
        let repeat_pos = node.children.iter().position(|child| {
            child.class == "PPI::Token::Operator" && child.content.as_deref() == Some("x")
        })?;

        if repeat_pos > 0 && repeat_pos + 1 < node.children.len() {
            trace!("Found string repetition pattern");

            let string_operand = node.children[repeat_pos - 1].clone();
            let count_operand = node.children[repeat_pos + 1].clone();

            return Some(PpiNode {
                class: "StringRepeat".to_string(),
                content: None,
                children: vec![string_operand, count_operand],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            });
        }

        None
    }
}
