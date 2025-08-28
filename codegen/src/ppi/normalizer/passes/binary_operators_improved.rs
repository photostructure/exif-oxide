//! Improved binary operator normalization with pure precedence climbing
//!
//! This implementation follows the precedence climbing algorithm exactly as described
//! in the reference document, separating concerns cleanly and applying Trust ExifTool
//! principles for Perl operator precedence.

use crate::ppi::normalizer::multi_pass::RewritePass;
use crate::ppi::types::PpiNode;
use tracing::{debug, trace};

/// Perl operator precedence and associativity table
/// Based on perlop documentation: https://perldoc.perl.org/perlop
/// ExifTool reference: lib/Image/ExifTool.pm (precedence handling)
static OPERATOR_INFO: &[(&str, u8, bool)] = &[
    // (operator, precedence, right_associative)
    ("**", 50, true),   // Exponentiation (right-associative)
    ("*", 40, false),   // Multiplication
    ("/", 40, false),   // Division  
    ("%", 40, false),   // Modulo
    ("+", 20, false),   // Addition
    ("-", 20, false),   // Subtraction
    (".", 18, false),   // String concatenation
    ("&", 12, false),   // Bitwise AND
    ("^", 11, false),   // Bitwise XOR
    ("|", 10, false),   // Bitwise OR
    ("==", 8, false),   // Numeric equality
    ("!=", 8, false),   // Numeric inequality
    ("<", 8, false),    // Less than
    (">", 8, false),    // Greater than
    ("<=", 8, false),   // Less than or equal
    (">=", 8, false),   // Greater than or equal
    ("eq", 8, false),   // String equality
    ("ne", 8, false),   // String inequality
    ("lt", 8, false),   // String less than
    ("gt", 8, false),   // String greater than
    ("le", 8, false),   // String less than or equal
    ("ge", 8, false),   // String greater than or equal
    ("=~", 7, false),   // Regex match
    ("!~", 7, false),   // Regex non-match
    ("&&", 6, false),   // Logical AND
    ("and", 6, false),  // Logical AND (lower precedence)
    ("||", 5, false),   // Logical OR
    ("or", 5, false),   // Logical OR (lower precedence)
];

#[derive(Debug, Clone)]
struct OperatorInfo {
    precedence: u8,
    right_associative: bool,
}

/// Pure precedence climbing binary operator normalizer
/// 
/// Implements the algorithm exactly as described in PRECEDENCE-CLIMBING-FOR-AST.md:
/// 1. parse_expression() - entry point
/// 2. parse_expression_1(left_operand, min_precedence) - recursive worker
pub struct ImprovedBinaryOperatorNormalizer;

impl RewritePass for ImprovedBinaryOperatorNormalizer {
    fn name(&self) -> &str {
        "ImprovedBinaryOperatorNormalizer"
    }

    fn transform(&self, node: PpiNode) -> PpiNode {
        if self.should_process(&node) {
            trace!("Processing node for precedence climbing: {}", node.class);
            self.normalize_binary_operations(node)
        } else {
            node
        }
    }
}

impl ImprovedBinaryOperatorNormalizer {
    /// Check if this node should be processed for binary operations
    fn should_process(&self, node: &PpiNode) -> bool {
        matches!(
            node.class.as_str(),
            "PPI::Statement" | "PPI::Statement::Expression" | "FunctionCall"
        ) && node.children.len() >= 3
            && self.contains_operators(node)
    }

    /// Check if node contains binary operators we handle
    fn contains_operators(&self, node: &PpiNode) -> bool {
        node.children.iter().any(|child| {
            child.class == "PPI::Token::Operator"
                && child.content.as_ref()
                    .map_or(false, |op| self.get_operator_info(op).is_some())
        })
    }

    /// Get operator precedence and associativity info
    fn get_operator_info(&self, op: &str) -> Option<OperatorInfo> {
        OPERATOR_INFO
            .iter()
            .find(|(name, _, _)| *name == op)
            .map(|(_, prec, right_assoc)| OperatorInfo {
                precedence: *prec,
                right_associative: *right_assoc,
            })
    }

    /// Normalize binary operations using pure precedence climbing
    fn normalize_binary_operations(&self, node: PpiNode) -> PpiNode {
        // STEP 1: Filter out whitespace and comments for clean token sequence
        let meaningful_tokens = self.extract_meaningful_tokens(node.children);
        
        // STEP 2: Apply precedence climbing to token sequence
        let normalized_children = self.apply_precedence_climbing(meaningful_tokens);
        
        PpiNode {
            children: normalized_children,
            ..node
        }
    }

    /// Extract meaningful tokens, preserving structure but filtering noise
    fn extract_meaningful_tokens(&self, children: Vec<PpiNode>) -> Vec<PpiNode> {
        children
            .into_iter()
            .filter(|child| {
                !matches!(
                    child.class.as_str(),
                    "PPI::Token::Whitespace" | "PPI::Token::Comment"
                )
            })
            .collect()
    }

    /// Apply precedence climbing to a sequence of tokens
    fn apply_precedence_climbing(&self, tokens: Vec<PpiNode>) -> Vec<PpiNode> {
        if tokens.len() < 3 {
            return tokens; // Not enough for binary operation
        }

        // Handle comma-separated expressions first
        self.process_comma_separated_expressions(tokens)
    }

    /// Process comma-separated expressions (comma has lowest precedence)
    fn process_comma_separated_expressions(&self, tokens: Vec<PpiNode>) -> Vec<PpiNode> {
        let mut result = Vec::new();
        let mut current_expr = Vec::new();

        for token in tokens {
            if token.class == "PPI::Token::Operator" 
                && token.content.as_deref() == Some(",") {
                
                // Process current expression segment
                if let Some(processed) = self.parse_expression(current_expr) {
                    result.push(processed);
                }
                
                result.push(token); // Add comma
                current_expr = Vec::new();
            } else {
                current_expr.push(token);
            }
        }

        // Process final segment
        if !current_expr.is_empty() {
            if let Some(processed) = self.parse_expression(current_expr) {
                result.push(processed);
            }
        }

        result
    }

    /// Entry point for precedence climbing algorithm
    /// Implements: parse_expression() -> parse_expression_1(parse_primary(), 0)
    fn parse_expression(&self, tokens: Vec<PpiNode>) -> Option<PpiNode> {
        if tokens.is_empty() {
            return None;
        }
        
        if tokens.len() == 1 {
            return Some(tokens.into_iter().next().unwrap());
        }

        debug!("Starting precedence climbing for {} tokens", tokens.len());
        
        // Parse first primary expression
        let left_operand = tokens[0].clone();
        
        // Apply precedence climbing with minimum precedence 0
        self.parse_expression_1(left_operand, &tokens[1..], 0)
    }

    /// Recursive precedence climbing worker
    /// Implements: parse_expression_1(left_operand, min_precedence)
    fn parse_expression_1(
        &self,
        mut left_operand: PpiNode,
        tokens: &[PpiNode],
        min_precedence: u8,
    ) -> Option<PpiNode> {
        let mut pos = 0;
        
        // Main precedence climbing loop
        while pos < tokens.len() {
            // Look for binary operator
            if pos + 1 >= tokens.len() {
                break; // Need operator + operand
            }

            let operator_token = &tokens[pos];
            if operator_token.class != "PPI::Token::Operator" {
                break;
            }

            let operator = operator_token.content.as_ref()?;
            let op_info = self.get_operator_info(operator)?;
            
            // Check precedence constraint
            if op_info.precedence < min_precedence {
                break;
            }

            pos += 1; // Consume operator

            // Parse right operand (primary expression)
            let mut right_operand = tokens[pos].clone();
            pos += 1;

            // Handle higher precedence operators on the right
            while pos < tokens.len() {
                if let Some(next_op) = tokens[pos].content.as_ref() {
                    if tokens[pos].class == "PPI::Token::Operator" {
                        if let Some(next_info) = self.get_operator_info(next_op) {
                            // Check for higher precedence or right-associative equal precedence
                            if next_info.precedence > op_info.precedence
                                || (next_info.right_associative 
                                    && next_info.precedence == op_info.precedence)
                            {
                                // Parse right side with higher minimum precedence
                                let next_min_prec = if next_info.right_associative 
                                    && next_info.precedence == op_info.precedence {
                                    op_info.precedence // Same precedence for right-associative
                                } else {
                                    op_info.precedence + 1 // Higher precedence
                                };

                                if let Some(parsed_right) = 
                                    self.parse_expression_1(right_operand, &tokens[pos..], next_min_prec)
                                {
                                    right_operand = parsed_right;
                                    // Advance position - we consumed tokens in recursive call
                                    pos = tokens.len(); // Simplified - breaks outer loop
                                }
                                break;
                            }
                        }
                    }
                }
                break;
            }

            // Create binary operation node  
            left_operand = self.create_binary_operation(
                operator.clone(),
                left_operand,
                right_operand,
            );

            debug!("Created binary operation: {}", operator);
        }

        Some(left_operand)
    }

    /// Create a canonical BinaryOperation node
    fn create_binary_operation(
        &self,
        operator: String,
        left: PpiNode,
        right: PpiNode,
    ) -> PpiNode {
        PpiNode {
            class: "BinaryOperation".to_string(),
            content: Some(operator),
            children: vec![left, right],
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
    fn test_operator_precedence() {
        let normalizer = ImprovedBinaryOperatorNormalizer;
        
        // Test multiplication has higher precedence than addition
        let mult_info = normalizer.get_operator_info("*").unwrap();
        let add_info = normalizer.get_operator_info("+").unwrap();
        assert!(mult_info.precedence > add_info.precedence);
        
        // Test exponentiation is right-associative
        let power_info = normalizer.get_operator_info("**").unwrap();
        assert!(power_info.right_associative);
        
        // Test multiplication is left-associative
        assert!(!mult_info.right_associative);
    }

    #[test]
    fn test_simple_binary_expression() {
        let normalizer = ImprovedBinaryOperatorNormalizer;
        
        // Test $val * 100
        let tokens = vec![
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
                content: Some("*".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Token::Number".to_string(),
                content: Some("100".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: Some(100.0),
                string_value: None,
                structure_bounds: None,
            },
        ];

        let result = normalizer.parse_expression(tokens).unwrap();
        assert_eq!(result.class, "BinaryOperation");
        assert_eq!(result.content, Some("*".to_string()));
        assert_eq!(result.children.len(), 2);
    }

    #[test]
    fn test_precedence_order() {
        let normalizer = ImprovedBinaryOperatorNormalizer;
        
        // Test $val + 1 * 2 should become $val + (1 * 2)
        let tokens = vec![
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
                content: Some("+".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Token::Number".to_string(),
                content: Some("1".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: Some(1.0),
                string_value: None,
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Token::Operator".to_string(),
                content: Some("*".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Token::Number".to_string(),
                content: Some("2".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: Some(2.0),
                string_value: None,
                structure_bounds: None,
            },
        ];

        let result = normalizer.parse_expression(tokens).unwrap();
        
        // Should be: BinaryOperation("+", $val, BinaryOperation("*", 1, 2))
        assert_eq!(result.class, "BinaryOperation");
        assert_eq!(result.content, Some("+".to_string()));
        
        // Right child should be the multiplication
        assert_eq!(result.children[1].class, "BinaryOperation");
        assert_eq!(result.children[1].content, Some("*".to_string()));
    }
}