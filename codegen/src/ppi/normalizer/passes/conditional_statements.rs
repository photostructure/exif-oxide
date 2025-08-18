//! Conditional Statement Normalizer - Converts Perl postfix conditionals
//!
//! Transforms Perl's postfix conditional syntax into proper Rust if statements.
//! Handles both `if` and `unless` variants in return statements and general statements.
//!
//! Examples:
//! - `return "Off" unless $val` → `if (!$val) { return "Off"; }`
//! - `return $expr if $condition` → `if ($condition) { return $expr; }`
//! - `$x = 5 if $condition` → `if ($condition) { $x = 5; }`

use crate::ppi::normalizer::{multi_pass::RewritePass, utils};
use crate::ppi::types::PpiNode;
use tracing::{debug, trace};

/// Normalizes postfix conditionals (if/unless) to prefix if statements
///
/// ExifTool commonly uses both `return VALUE if $condition` and `return VALUE unless $condition`
/// patterns for conditional returns. This pass converts them to proper Rust conditional statements.
///
/// Patterns handled:
/// 1. `return EXPR if CONDITION` → `if (CONDITION) { return EXPR; }`
/// 2. `return EXPR unless CONDITION` → `if (!(CONDITION)) { return EXPR; }`
/// 3. `STATEMENT if CONDITION` → `if (CONDITION) { STATEMENT; }`
/// 4. `STATEMENT unless CONDITION` → `if (!(CONDITION)) { STATEMENT; }`
///
/// This pass must run early to avoid conflicts with other conditional processing.
pub struct ConditionalStatementsNormalizer;

impl RewritePass for ConditionalStatementsNormalizer {
    fn name(&self) -> &str {
        "ConditionalStatementsNormalizer"
    }

    fn transform(&self, node: PpiNode) -> PpiNode {
        // Handle return statements with postfix conditionals
        if node.class == "PPI::Statement::Break" {
            // Try unless pattern first (more specific)
            if let Some(transformed) = self.try_transform_unless_return(&node) {
                trace!("Transformed unless return statement");
                return transformed;
            }

            // Try if pattern
            if let Some(transformed) = self.try_transform_if_return(&node) {
                trace!("Transformed if return statement");
                return transformed;
            }
        }

        // Handle general statements with postfix conditionals
        if node.class == "PPI::Statement" {
            // Try unless pattern first (more specific)
            if let Some(transformed) = self.try_transform_unless_statement(&node) {
                trace!("Transformed unless statement");
                return transformed;
            }

            // Try if pattern
            if let Some(transformed) = self.try_transform_if_statement(&node) {
                trace!("Transformed if statement");
                return transformed;
            }
        }

        // No patterns matched, return unchanged
        node
    }
}

impl ConditionalStatementsNormalizer {
    // === UNLESS PATTERNS ===

    /// Try to transform a return statement with unless: `return EXPR unless CONDITION`
    fn try_transform_unless_return(&self, node: &PpiNode) -> Option<PpiNode> {
        let children = &node.children;

        // Need at least 4 tokens: return, value, unless, condition
        if children.len() < 4 {
            return None;
        }

        // Find keyword positions
        let return_pos = self.find_keyword(children, "return")?;
        let unless_pos = self.find_keyword_after(children, "unless", return_pos + 1)?;

        // Extract components
        let return_expr = self.extract_expression_between(children, return_pos + 1, unless_pos)?;
        let condition_expr = self.extract_expression_after(children, unless_pos)?;

        // Create negated conditional: if (!(condition)) return expr
        let negated_condition = self.create_negated_condition(condition_expr);
        let conditional = self.create_conditional_return(negated_condition, return_expr);

        debug!("Transformed unless return statement");
        Some(conditional)
    }

    /// Try to transform a general statement with unless: `STATEMENT unless CONDITION`
    fn try_transform_unless_statement(&self, node: &PpiNode) -> Option<PpiNode> {
        let children = &node.children;

        // Find unless keyword
        let unless_pos = self.find_keyword(children, "unless")?;

        // Extract components
        let stmt_nodes = self.extract_expression_before(children, unless_pos)?;
        let condition_expr = self.extract_expression_after(children, unless_pos)?;

        // Create negated conditional: if (!(condition)) statement
        let negated_condition = self.create_negated_condition(condition_expr);
        let conditional = self.create_conditional_statement(negated_condition, stmt_nodes);

        debug!("Transformed unless statement");
        Some(conditional)
    }

    // === IF PATTERNS ===

    /// Try to transform a return statement with if: `return EXPR if CONDITION`
    fn try_transform_if_return(&self, node: &PpiNode) -> Option<PpiNode> {
        let children = &node.children;

        // Find keyword positions
        let return_pos = self.find_keyword(children, "return")?;
        let if_pos = self.find_keyword_after(children, "if", return_pos + 1)?;

        // Skip semicolon at end if present
        let end_pos = if let Some(semicolon_pos) = children.iter().rposition(|child| {
            child.class == "PPI::Token::Structure"
                && child.content.as_ref() == Some(&";".to_string())
        }) {
            semicolon_pos
        } else {
            children.len()
        };

        // Verify structure: return comes before if, if comes before end
        if return_pos >= if_pos || if_pos >= end_pos {
            return None;
        }

        // Extract components
        let return_expr = self.extract_expression_between(children, return_pos + 1, if_pos)?;
        let condition_expr = self.extract_expression_between(children, if_pos + 1, end_pos)?;

        // Create conditional: if (condition) return expr
        let condition = self.create_condition(condition_expr);
        let conditional = self.create_conditional_return(condition, return_expr);

        debug!("Transformed if return statement");
        Some(conditional)
    }

    /// Try to transform a general statement with if: `STATEMENT if CONDITION`
    fn try_transform_if_statement(&self, node: &PpiNode) -> Option<PpiNode> {
        let children = &node.children;

        // Find if keyword
        let if_pos = self.find_keyword(children, "if")?;

        // Skip semicolon at end if present
        let end_pos = if let Some(semicolon_pos) = children.iter().rposition(|child| {
            child.class == "PPI::Token::Structure"
                && child.content.as_ref() == Some(&";".to_string())
        }) {
            semicolon_pos
        } else {
            children.len()
        };

        // Verify if comes before end
        if if_pos >= end_pos {
            return None;
        }

        // Extract components
        let stmt_nodes = self.extract_expression_before(children, if_pos)?;
        let condition_expr = self.extract_expression_between(children, if_pos + 1, end_pos)?;

        // Create conditional: if (condition) statement
        let condition = self.create_condition(condition_expr);
        let conditional = self.create_conditional_statement(condition, stmt_nodes);

        debug!("Transformed if statement");
        Some(conditional)
    }

    // === HELPER METHODS ===

    /// Find position of keyword in children
    fn find_keyword(&self, children: &[PpiNode], keyword: &str) -> Option<usize> {
        children.iter().position(|child| {
            child.class == "PPI::Token::Word"
                && child.content.as_ref().map_or(false, |c| c == keyword)
        })
    }

    /// Find position of keyword after given start position
    fn find_keyword_after(
        &self,
        children: &[PpiNode],
        keyword: &str,
        start_pos: usize,
    ) -> Option<usize> {
        children
            .iter()
            .skip(start_pos)
            .position(|child| {
                child.class == "PPI::Token::Word"
                    && child.content.as_ref().map_or(false, |c| c == keyword)
            })
            .map(|pos| pos + start_pos)
    }

    /// Extract expression nodes between two positions
    fn extract_expression_between(
        &self,
        children: &[PpiNode],
        start: usize,
        end: usize,
    ) -> Option<Vec<PpiNode>> {
        if end <= start {
            return None;
        }

        let expr_nodes: Vec<PpiNode> = children[start..end].to_vec();
        if expr_nodes.is_empty() {
            None
        } else {
            Some(expr_nodes)
        }
    }

    /// Extract expression nodes before a position
    fn extract_expression_before(&self, children: &[PpiNode], pos: usize) -> Option<Vec<PpiNode>> {
        if pos == 0 {
            return None;
        }

        let expr_nodes: Vec<PpiNode> = children[..pos].to_vec();
        if expr_nodes.is_empty() {
            None
        } else {
            Some(expr_nodes)
        }
    }

    /// Extract expression nodes after a position
    fn extract_expression_after(&self, children: &[PpiNode], pos: usize) -> Option<Vec<PpiNode>> {
        if pos + 1 >= children.len() {
            return None;
        }

        let expr_nodes: Vec<PpiNode> = children[(pos + 1)..].to_vec();
        if expr_nodes.is_empty() {
            None
        } else {
            Some(expr_nodes)
        }
    }

    /// Create a positive condition from expression nodes
    fn create_condition(&self, condition_nodes: Vec<PpiNode>) -> PpiNode {
        PpiNode {
            class: "PPI::Statement::Expression".to_string(),
            content: None,
            children: condition_nodes,
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        }
    }

    /// Create a negated condition: !(EXPR)
    fn create_negated_condition(&self, condition_nodes: Vec<PpiNode>) -> PpiNode {
        // Create NOT operator
        let not_operator = PpiNode {
            class: "PPI::Token::Operator".to_string(),
            content: Some("!".to_string()),
            children: vec![],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        // Create parenthesized condition: (condition_nodes)
        let condition_expr = PpiNode {
            class: "PPI::Statement::Expression".to_string(),
            content: None,
            children: condition_nodes,
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        let parenthesized_condition = PpiNode {
            class: "PPI::Structure::List".to_string(),
            content: None,
            children: vec![condition_expr],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: Some("( ... )".to_string()),
        };

        // Create negated expression: !(condition)
        PpiNode {
            class: "PPI::Statement::Expression".to_string(),
            content: None,
            children: vec![not_operator, parenthesized_condition],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        }
    }

    /// Create conditional return statement: if (condition) { return expr; }
    fn create_conditional_return(&self, condition: PpiNode, return_expr: Vec<PpiNode>) -> PpiNode {
        // Create return keyword
        let return_keyword = PpiNode {
            class: "PPI::Token::Word".to_string(),
            content: Some("return".to_string()),
            children: vec![],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        // Create return statement with expression
        let mut return_statement_children = vec![return_keyword];
        return_statement_children.extend(return_expr);

        let return_statement = PpiNode {
            class: "PPI::Statement::Break".to_string(),
            content: None,
            children: return_statement_children,
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        // Create conditional: if (condition) { return expr; }
        utils::create_function_call("if", vec![condition, return_statement])
    }

    /// Create conditional statement: if (condition) { statement; }
    fn create_conditional_statement(
        &self,
        condition: PpiNode,
        stmt_nodes: Vec<PpiNode>,
    ) -> PpiNode {
        // Create statement block
        let stmt = PpiNode {
            class: "PPI::Statement".to_string(),
            content: None,
            children: stmt_nodes,
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        // Create conditional: if (condition) { statement; }
        utils::create_function_call("if", vec![condition, stmt])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create test PpiNode from basic components
    fn create_test_node(class: &str, content: Option<&str>) -> PpiNode {
        PpiNode {
            class: class.to_string(),
            content: content.map(|s| s.to_string()),
            children: vec![],
            symbol_type: if class == "PPI::Token::Symbol" {
                Some("scalar".to_string())
            } else {
                None
            },
            numeric_value: None,
            string_value: content.map(|s| s.to_string()),
            structure_bounds: None,
        }
    }

    #[test]
    fn test_unless_return_pattern() {
        let normalizer = ConditionalStatementsNormalizer;

        // Create AST for: return "Off" unless $val
        let statement = PpiNode {
            class: "PPI::Statement::Break".to_string(),
            content: None,
            children: vec![
                create_test_node("PPI::Token::Word", Some("return")),
                create_test_node("PPI::Token::Quote::Double", Some("\"Off\"")),
                create_test_node("PPI::Token::Word", Some("unless")),
                create_test_node("PPI::Token::Symbol", Some("$val")),
            ],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        let result = RewritePass::transform(&normalizer, statement);

        // Should transform to conditional structure
        assert_eq!(result.class, "FunctionCall");
        assert_eq!(result.content, Some("if".to_string()));
        assert_eq!(result.children.len(), 2);

        // First argument should be negated condition
        let condition = &result.children[0];
        assert_eq!(condition.class, "PPI::Statement::Expression");
        assert_eq!(condition.children[0].content, Some("!".to_string()));

        // Second argument should be return statement
        let return_stmt = &result.children[1];
        assert_eq!(return_stmt.class, "PPI::Statement::Break");
        assert_eq!(return_stmt.children[0].content, Some("return".to_string()));
        assert_eq!(return_stmt.children[1].content, Some("\"Off\"".to_string()));
    }

    #[test]
    fn test_if_return_pattern() {
        let normalizer = ConditionalStatementsNormalizer;

        // Create AST for: return "On" if $val
        let statement = PpiNode {
            class: "PPI::Statement::Break".to_string(),
            content: None,
            children: vec![
                create_test_node("PPI::Token::Word", Some("return")),
                create_test_node("PPI::Token::Quote::Double", Some("\"On\"")),
                create_test_node("PPI::Token::Word", Some("if")),
                create_test_node("PPI::Token::Symbol", Some("$val")),
                create_test_node("PPI::Token::Structure", Some(";")),
            ],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        let result = RewritePass::transform(&normalizer, statement);

        // Should transform to conditional structure
        assert_eq!(result.class, "FunctionCall");
        assert_eq!(result.content, Some("if".to_string()));
        assert_eq!(result.children.len(), 2);

        // First argument should be positive condition (no negation)
        let condition = &result.children[0];
        assert_eq!(condition.class, "PPI::Statement::Expression");
        assert_eq!(condition.children[0].content, Some("$val".to_string()));

        // Second argument should be return statement
        let return_stmt = &result.children[1];
        assert_eq!(return_stmt.class, "PPI::Statement::Break");
        assert_eq!(return_stmt.children[0].content, Some("return".to_string()));
        assert_eq!(return_stmt.children[1].content, Some("\"On\"".to_string()));
    }

    #[test]
    fn test_unless_statement_pattern() {
        let normalizer = ConditionalStatementsNormalizer;

        // Create AST for: $x = 5 unless $condition
        let statement = PpiNode {
            class: "PPI::Statement".to_string(),
            content: None,
            children: vec![
                create_test_node("PPI::Token::Symbol", Some("$x")),
                create_test_node("PPI::Token::Operator", Some("=")),
                create_test_node("PPI::Token::Number", Some("5")),
                create_test_node("PPI::Token::Word", Some("unless")),
                create_test_node("PPI::Token::Symbol", Some("$condition")),
            ],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        let result = RewritePass::transform(&normalizer, statement);

        // Should transform to conditional structure
        assert_eq!(result.class, "FunctionCall");
        assert_eq!(result.content, Some("if".to_string()));
        assert_eq!(result.children.len(), 2);

        // First argument should be negated condition
        let condition = &result.children[0];
        assert_eq!(condition.class, "PPI::Statement::Expression");
        assert_eq!(condition.children[0].content, Some("!".to_string()));

        // Second argument should be assignment statement
        let stmt = &result.children[1];
        assert_eq!(stmt.class, "PPI::Statement");
        assert_eq!(stmt.children[0].content, Some("$x".to_string()));
        assert_eq!(stmt.children[1].content, Some("=".to_string()));
        assert_eq!(stmt.children[2].content, Some("5".to_string()));
    }

    #[test]
    fn test_no_conditional_pattern() {
        let normalizer = ConditionalStatementsNormalizer;

        // Create AST for regular return: return $val
        let statement = PpiNode {
            class: "PPI::Statement::Break".to_string(),
            content: None,
            children: vec![
                create_test_node("PPI::Token::Word", Some("return")),
                create_test_node("PPI::Token::Symbol", Some("$val")),
            ],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        let result = RewritePass::transform(&normalizer, statement);

        // Should return unchanged - no conditional pattern
        assert_eq!(result.class, "PPI::Statement::Break");
        assert_eq!(result.children.len(), 2);
        assert_eq!(result.children[0].content, Some("return".to_string()));
        assert_eq!(result.children[1].content, Some("$val".to_string()));
    }
}
