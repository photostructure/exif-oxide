//! Sneaky conditional assignment normalization for AST transformation
//!
//! Transforms Perl idioms like `$val > 1800 and $val -= 3600; -$val / 10` into normalized conditional blocks.
//! This pattern is common in ExifTool where conditional logic is combined with side-effect assignments.
//!
//! Trust ExifTool: Canon.pm line 9359 - `$val > 1800 and $val -= 3600; -$val / 10`

use crate::ppi::normalizer::multi_pass::RewritePass;
use crate::ppi::types::PpiNode;
use tracing::trace;

/// Normalizes sneaky conditional assignment patterns
/// Pattern: `condition and assignment; return_expr`
/// Example: `$val > 1800 and $val -= 3600; -$val / 10`
pub struct SneakyConditionalAssignmentNormalizer;

impl RewritePass for SneakyConditionalAssignmentNormalizer {
    fn name(&self) -> &str {
        "SneakyConditionalAssignmentNormalizer"
    }

    fn transform(&self, node: PpiNode) -> PpiNode {
        // Look for PPI::Document with multiple statements
        if node.class == "PPI::Document" && node.children.len() == 2 {
            if let Some(normalized) = self.try_normalize_sneaky_conditional(&node) {
                trace!("Found sneaky conditional assignment pattern");
                return normalized;
            }
        }

        // No transformation - return unchanged
        node
    }
}

impl SneakyConditionalAssignmentNormalizer {
    /// Try to normalize a sneaky conditional assignment pattern
    fn try_normalize_sneaky_conditional(&self, document: &PpiNode) -> Option<PpiNode> {
        if document.children.len() != 2 {
            return None;
        }

        let first_stmt = &document.children[0];
        let second_stmt = &document.children[1];

        // First statement should be: condition and assignment;
        if !self.is_conditional_assignment_statement(first_stmt) {
            return None;
        }

        // Extract the condition and assignment from first statement
        let (condition, assignment) = self.extract_condition_and_assignment(first_stmt)?;

        // Second statement is the return expression
        let return_expr = second_stmt.clone();

        // Create normalized conditional block
        Some(self.create_conditional_block(condition, assignment, return_expr))
    }

    /// Check if this statement matches the pattern: condition and assignment;
    fn is_conditional_assignment_statement(&self, stmt: &PpiNode) -> bool {
        if stmt.class != "PPI::Statement" {
            return false;
        }

        // Look for pattern: expr "and" expr [";"]
        let mut has_and = false;
        let mut has_assignment = false;
        let mut _has_semicolon = false;

        for child in &stmt.children {
            match child.class.as_str() {
                "PPI::Token::Operator" => {
                    if let Some(ref content) = child.content {
                        if content == "and" {
                            has_and = true;
                        } else if content == "-="
                            || content == "+="
                            || content == "*="
                            || content == "/="
                        {
                            has_assignment = true;
                        }
                    }
                }
                "PPI::Token::Structure" => {
                    if let Some(ref content) = child.content {
                        if content == ";" {
                            _has_semicolon = true;
                        }
                    }
                }
                _ => {}
            }
        }

        has_and && has_assignment
    }

    /// Extract condition and assignment from statement
    fn extract_condition_and_assignment(
        &self,
        stmt: &PpiNode,
    ) -> Option<(Vec<PpiNode>, Vec<PpiNode>)> {
        let mut condition = Vec::new();
        let mut assignment = Vec::new();
        let mut in_assignment = false;

        for child in &stmt.children {
            // Skip whitespace
            if child.class == "PPI::Token::Whitespace" {
                continue;
            }

            // Stop at semicolon
            if child.class == "PPI::Token::Structure"
                && child.content.as_ref() == Some(&";".to_string())
            {
                break;
            }

            // Switch to assignment section after "and"
            if child.class == "PPI::Token::Operator"
                && child.content.as_ref() == Some(&"and".to_string())
            {
                in_assignment = true;
                continue;
            }

            if in_assignment {
                assignment.push(child.clone());
            } else {
                condition.push(child.clone());
            }
        }

        if condition.is_empty() || assignment.is_empty() {
            None
        } else {
            Some((condition, assignment))
        }
    }

    /// Create a normalized conditional block AST node
    fn create_conditional_block(
        &self,
        condition: Vec<PpiNode>,
        assignment: Vec<PpiNode>,
        return_expr: PpiNode,
    ) -> PpiNode {
        PpiNode {
            class: "ConditionalBlock".to_string(),
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
                // Child 1: assignment
                if assignment.len() == 1 {
                    assignment[0].clone()
                } else {
                    PpiNode {
                        class: "Assignment".to_string(),
                        content: None,
                        children: assignment,
                        symbol_type: None,
                        numeric_value: None,
                        string_value: None,
                        structure_bounds: None,
                    }
                },
                // Child 2: return expression
                return_expr,
            ],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sneaky_conditional_assignment_recognition() {
        // Create AST for: $val > 1800 and $val -= 3600; -$val / 10
        let document = PpiNode {
            class: "PPI::Document".to_string(),
            content: None,
            children: vec![
                // First statement: $val > 1800 and $val -= 3600;
                PpiNode {
                    class: "PPI::Statement".to_string(),
                    content: None,
                    children: vec![
                        PpiNode {
                            class: "PPI::Token::Symbol".to_string(),
                            content: Some("$val".to_string()),
                            children: vec![],
                            symbol_type: Some("scalar".to_string()),
                            numeric_value: None,
                            string_value: None,
                            structure_bounds: None,
                        },
                        PpiNode {
                            class: "PPI::Token::Operator".to_string(),
                            content: Some(">".to_string()),
                            children: vec![],
                            symbol_type: None,
                            numeric_value: None,
                            string_value: None,
                            structure_bounds: None,
                        },
                        PpiNode {
                            class: "PPI::Token::Number".to_string(),
                            content: Some("1800".to_string()),
                            children: vec![],
                            symbol_type: None,
                            numeric_value: Some(1800.0),
                            string_value: None,
                            structure_bounds: None,
                        },
                        PpiNode {
                            class: "PPI::Token::Operator".to_string(),
                            content: Some("and".to_string()),
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
                        PpiNode {
                            class: "PPI::Token::Operator".to_string(),
                            content: Some("-=".to_string()),
                            children: vec![],
                            symbol_type: None,
                            numeric_value: None,
                            string_value: None,
                            structure_bounds: None,
                        },
                        PpiNode {
                            class: "PPI::Token::Number".to_string(),
                            content: Some("3600".to_string()),
                            children: vec![],
                            symbol_type: None,
                            numeric_value: Some(3600.0),
                            string_value: None,
                            structure_bounds: None,
                        },
                        PpiNode {
                            class: "PPI::Token::Structure".to_string(),
                            content: Some(";".to_string()),
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
                },
                // Second statement: -$val / 10
                PpiNode {
                    class: "PPI::Statement".to_string(),
                    content: None,
                    children: vec![
                        PpiNode {
                            class: "PPI::Token::Operator".to_string(),
                            content: Some("-".to_string()),
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
                        PpiNode {
                            class: "PPI::Token::Operator".to_string(),
                            content: Some("/".to_string()),
                            children: vec![],
                            symbol_type: None,
                            numeric_value: None,
                            string_value: None,
                            structure_bounds: None,
                        },
                        PpiNode {
                            class: "PPI::Token::Number".to_string(),
                            content: Some("10".to_string()),
                            children: vec![],
                            symbol_type: None,
                            numeric_value: Some(10.0),
                            string_value: None,
                            structure_bounds: None,
                        },
                    ],
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

        let normalizer = SneakyConditionalAssignmentNormalizer;
        let result = RewritePass::transform(&normalizer, document);

        // Should be normalized to ConditionalBlock
        assert_eq!(result.class, "ConditionalBlock");
        assert_eq!(result.children.len(), 3);

        // Verify structure
        assert!(
            result.children[0].class == "PPI::Token::Symbol"
                || result.children[0].class == "Condition"
        );
        assert!(
            result.children[1].class == "PPI::Token::Symbol"
                || result.children[1].class == "Assignment"
        );
        assert_eq!(result.children[2].class, "PPI::Statement");
    }

    #[test]
    fn test_non_matching_pattern() {
        // Create AST that shouldn't match (no "and")
        let simple_stmt = PpiNode {
            class: "PPI::Statement".to_string(),
            content: None,
            children: vec![PpiNode {
                class: "PPI::Token::Symbol".to_string(),
                content: Some("$val".to_string()),
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

        let normalizer = SneakyConditionalAssignmentNormalizer;
        let result = RewritePass::transform(&normalizer, simple_stmt.clone());

        // Should be unchanged
        assert_eq!(result.class, simple_stmt.class);
    }
}
