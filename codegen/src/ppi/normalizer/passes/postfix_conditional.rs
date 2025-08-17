//! Postfix conditional normalization for AST transformation
//!
//! Transforms Perl's postfix conditional syntax into proper Rust if statements.
//! Example: "return EXPR if PREDICATE" -> "if PREDICATE { return EXPR }"

use crate::ppi::normalizer::{utils, NormalizationPass, PrecedenceLevel};
use crate::ppi::types::PpiNode;
use tracing::trace;

/// Normalizes postfix conditionals to prefix if statements
pub struct PostfixConditionalNormalizer;

impl NormalizationPass for PostfixConditionalNormalizer {
    fn name(&self) -> &str {
        "PostfixConditionalNormalizer"
    }

    fn precedence_level(&self) -> PrecedenceLevel {
        PrecedenceLevel::High // Level 1-18 - structural transformations
    }

    fn transform(&self, node: PpiNode) -> PpiNode {
        // Pattern: PPI::Statement::Break with "return EXPR if PREDICATE;"
        if node.class == "PPI::Statement::Break" {
            if let Some(transformed) = self.try_transform_postfix_return(&node) {
                trace!("Transformed postfix conditional return statement");
                return transformed;
            }
        }

        // Also handle general statements that might have postfix conditionals
        if node.class == "PPI::Statement" {
            if let Some(transformed) = self.try_transform_postfix_statement(&node) {
                trace!("Transformed postfix conditional statement");
                return transformed;
            }
        }

        // Recurse into children
        utils::transform_children(node, |child| self.transform(child))
    }
}

impl PostfixConditionalNormalizer {
    /// Try to transform a break statement with postfix conditional
    fn try_transform_postfix_return(&self, node: &PpiNode) -> Option<PpiNode> {
        // Look for pattern: return EXPR if PREDICATE;
        let children = &node.children;

        // Find positions of key tokens
        let return_pos = children.iter().position(|child| {
            child.class == "PPI::Token::Word"
                && child.content.as_ref() == Some(&"return".to_string())
        })?;

        let if_pos = children.iter().position(|child| {
            child.class == "PPI::Token::Word" && child.content.as_ref() == Some(&"if".to_string())
        })?;

        // Semicolon should be at the end
        let semicolon_pos = children.iter().rposition(|child| {
            child.class == "PPI::Token::Structure"
                && child.content.as_ref() == Some(&";".to_string())
        })?;

        // Verify structure: return comes before if, if comes before semicolon
        if return_pos >= if_pos || if_pos >= semicolon_pos {
            return None;
        }

        // Extract expression (between return and if)
        let expr_nodes: Vec<PpiNode> = children[(return_pos + 1)..if_pos].to_vec();
        if expr_nodes.is_empty() {
            return None;
        }

        // Extract predicate (between if and semicolon)
        let predicate_nodes: Vec<PpiNode> = children[(if_pos + 1)..semicolon_pos].to_vec();
        if predicate_nodes.is_empty() {
            return None;
        }

        // Build the transformed if statement
        // For return statements, we need to prepend "return" to the body
        let mut body_nodes = vec![PpiNode {
            class: "PPI::Token::Word".to_string(),
            content: Some("return".to_string()),
            children: vec![],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        }];
        body_nodes.extend(expr_nodes);

        Some(self.create_if_statement(predicate_nodes, body_nodes))
    }

    /// Try to transform a general statement with postfix conditional
    fn try_transform_postfix_statement(&self, node: &PpiNode) -> Option<PpiNode> {
        // Look for pattern: STATEMENT if PREDICATE;
        let children = &node.children;

        let if_pos = children.iter().position(|child| {
            child.class == "PPI::Token::Word" && child.content.as_ref() == Some(&"if".to_string())
        })?;

        let semicolon_pos = children.iter().rposition(|child| {
            child.class == "PPI::Token::Structure"
                && child.content.as_ref() == Some(&";".to_string())
        })?;

        // Verify if comes before semicolon
        if if_pos >= semicolon_pos {
            return None;
        }

        // Extract statement (before if)
        let stmt_nodes: Vec<PpiNode> = children[..if_pos].to_vec();
        if stmt_nodes.is_empty() {
            return None;
        }

        // Extract predicate (between if and semicolon)
        let predicate_nodes: Vec<PpiNode> = children[(if_pos + 1)..semicolon_pos].to_vec();
        if predicate_nodes.is_empty() {
            return None;
        }

        // Build the transformed if statement
        Some(self.create_if_statement(predicate_nodes, stmt_nodes))
    }

    /// Create a normalized if statement from predicate and body parts
    fn create_if_statement(
        &self,
        predicate_nodes: Vec<PpiNode>,
        body_nodes: Vec<PpiNode>,
    ) -> PpiNode {
        // Create the predicate expression
        let predicate = PpiNode {
            class: "PPI::Statement::Expression".to_string(),
            content: None,
            children: predicate_nodes,
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        // Create the body statement
        let mut body_children = body_nodes;

        // Add semicolon to body if not present and it's a return statement
        if let Some(first) = body_children.first() {
            if first.class == "PPI::Token::Word"
                && first.content.as_ref() == Some(&"return".to_string())
            {
                // Check if semicolon is missing
                if !body_children.iter().any(|child| {
                    child.class == "PPI::Token::Structure"
                        && child.content.as_ref() == Some(&";".to_string())
                }) {
                    body_children.push(PpiNode {
                        class: "PPI::Token::Structure".to_string(),
                        content: Some(";".to_string()),
                        children: vec![],
                        symbol_type: None,
                        numeric_value: None,
                        string_value: None,
                        structure_bounds: None,
                    });
                }
            }
        }

        let body_stmt = PpiNode {
            class: "PPI::Statement".to_string(),
            content: None,
            children: body_children,
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        // Create the if block body
        let block_body = PpiNode {
            class: "PPI::Structure::Block".to_string(),
            content: None,
            children: vec![body_stmt],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: Some("{ ... }".to_string()),
        };

        // Create the complete if statement
        PpiNode {
            class: "IfStatement".to_string(),
            content: None,
            children: vec![predicate, block_body],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        }
    }
}
