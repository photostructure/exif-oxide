//! Binary operator normalization with precedence climbing
//!
//! This pass implements precedence climbing to properly group binary operations
//! in flat token sequences. It respects Perl operator precedence rules to ensure
//! expressions like $val * 100 are grouped correctly before comma separation.
//!
//! Based on the precedence climbing algorithm from LLVM tutorial and Rust Pratt parsers.

use crate::ppi::normalizer::multi_pass::RewritePass;
use crate::ppi::types::PpiNode;
use tracing::{debug, trace};

/// Perl operator precedence table
/// Based on perlop documentation: https://perldoc.perl.org/perlop
static PRECEDENCE: &[(&str, u8)] = &[
    // Multiplicative (highest precedence for our use case)
    ("*", 40),
    ("/", 40),
    ("%", 40),
    // Additive
    ("+", 20),
    ("-", 20),
    // String concatenation
    (".", 18),
    // Bitwise operations
    ("&", 12),
    ("|", 10),
    ("^", 11),
    // Comparison
    ("==", 8),
    ("!=", 8),
    ("<", 8),
    (">", 8),
    ("<=", 8),
    (">=", 8),
    ("eq", 8),
    ("ne", 8),
    ("lt", 8),
    ("gt", 8),
    ("le", 8),
    ("ge", 8),
    // Regex
    ("=~", 7),
    ("!~", 7),
    // Logical AND
    ("&&", 6),
    ("and", 6),
    // Logical OR
    ("||", 5),
    ("or", 5),
    // Power operator
    ("**", 50), // Higher than multiplicative
];

/// Normalizes binary operations using precedence climbing algorithm
pub struct BinaryOperatorNormalizer;

impl RewritePass for BinaryOperatorNormalizer {
    fn name(&self) -> &str {
        "BinaryOperatorNormalizer"
    }

    fn transform(&self, node: PpiNode) -> PpiNode {
        // Only process nodes that might contain binary operations
        if self.should_process(&node) {
            trace!("Processing node for binary operations: {}", node.class);
            self.normalize_binary_operations(node)
        } else {
            node
        }
    }
}

impl BinaryOperatorNormalizer {
    /// Check if this node should be processed for binary operations
    fn should_process(&self, node: &PpiNode) -> bool {
        // Process nodes that contain sequences of tokens that might include operators
        matches!(
            node.class.as_str(),
            "PPI::Statement" | "PPI::Statement::Expression" | "FunctionCall"
        ) && node.children.len() >= 3 // Need at least: operand, operator, operand
            && self.has_binary_operators(node)
    }

    /// Check if node contains binary operators
    fn has_binary_operators(&self, node: &PpiNode) -> bool {
        node.children.iter().any(|child| {
            child.class == "PPI::Token::Operator"
                && child
                    .content
                    .as_ref()
                    .is_some_and(|op| self.get_precedence(op).is_some())
        })
    }

    /// Get operator precedence, returns None for non-binary operators
    fn get_precedence(&self, op: &str) -> Option<u8> {
        PRECEDENCE
            .iter()
            .find(|(name, _)| *name == op)
            .map(|(_, prec)| *prec)
    }

    /// Normalize binary operations in a node using precedence climbing
    fn normalize_binary_operations(&self, node: PpiNode) -> PpiNode {
        if node.children.is_empty() {
            return node;
        }

        // STEP 1: Preprocess unary operators first
        let tokens_with_unary = self.preprocess_unary_operators(node.children);

        // STEP 2: Apply precedence climbing to the children (including preprocessed unary operations)
        let normalized_children = self.parse_expression_sequence(tokens_with_unary);

        PpiNode {
            children: normalized_children,
            ..node
        }
    }

    /// Parse a sequence of tokens into properly grouped binary operations
    fn parse_expression_sequence(&self, tokens: Vec<PpiNode>) -> Vec<PpiNode> {
        if tokens.len() < 3 {
            return tokens; // Not enough tokens for binary operation
        }

        let mut result = Vec::new();
        let mut i = 0;

        while i < tokens.len() {
            // Skip whitespace and comments
            if matches!(
                tokens[i].class.as_str(),
                "PPI::Token::Whitespace" | "PPI::Token::Comment"
            ) {
                result.push(tokens[i].clone());
                i += 1;
                continue;
            }

            // Look for comma-separated expressions
            // Split on commas and process each segment separately
            let segment_start = i;
            let mut segment_end = i;

            // Find the end of this comma-separated segment
            while segment_end < tokens.len() {
                if tokens[segment_end].class == "PPI::Token::Operator"
                    && tokens[segment_end].content.as_deref() == Some(",")
                {
                    break;
                }
                segment_end += 1;
            }

            // Process this segment for binary operations
            let segment = tokens[segment_start..segment_end].to_vec();
            if let Some(parsed_expr) = self.parse_binary_expression(segment) {
                result.push(parsed_expr);
            } else {
                // If parsing failed, keep original tokens
                result.extend_from_slice(&tokens[segment_start..segment_end]);
            }

            i = segment_end;

            // Add comma if present
            if i < tokens.len()
                && tokens[i].class == "PPI::Token::Operator"
                && tokens[i].content.as_deref() == Some(",")
            {
                result.push(tokens[i].clone());
                i += 1;
            }
        }

        result
    }

    /// Parse binary expression using precedence climbing
    /// Returns None if no binary operations found
    fn parse_binary_expression(&self, tokens: Vec<PpiNode>) -> Option<PpiNode> {
        if tokens.len() < 3 {
            return if tokens.len() == 1 {
                Some(tokens.into_iter().next().unwrap())
            } else {
                None
            };
        }

        debug!("Parsing binary expression with {} tokens", tokens.len());

        // Use precedence climbing algorithm
        self.parse_precedence(&tokens, 0, 0)
    }

    /// Precedence climbing recursive parser
    /// Returns parsed expression or None if no binary operations
    fn parse_precedence(
        &self,
        tokens: &[PpiNode],
        mut pos: usize,
        min_precedence: u8,
    ) -> Option<PpiNode> {
        if pos >= tokens.len() {
            return None;
        }

        // Parse left-hand side (primary expression)
        let mut left = tokens[pos].clone();
        pos += 1;

        // Process binary operators
        while pos + 1 < tokens.len() {
            // Skip whitespace
            if matches!(
                tokens[pos].class.as_str(),
                "PPI::Token::Whitespace" | "PPI::Token::Comment"
            ) {
                pos += 1;
                continue;
            }

            // Check if this is a binary operator
            let op_token = &tokens[pos];
            if op_token.class != "PPI::Token::Operator" {
                break;
            }

            let op = op_token.content.as_ref()?;
            let precedence = self.get_precedence(op)?;

            if precedence < min_precedence {
                break;
            }

            pos += 1; // consume operator

            // Skip whitespace after operator
            while pos < tokens.len()
                && matches!(
                    tokens[pos].class.as_str(),
                    "PPI::Token::Whitespace" | "PPI::Token::Comment"
                )
            {
                pos += 1;
            }

            if pos >= tokens.len() {
                break;
            }

            // Parse right-hand side
            let mut right = tokens[pos].clone();
            pos += 1;

            // Handle right-associative operators by looking ahead
            if pos + 1 < tokens.len() {
                if let Some(next_op) = tokens[pos].content.as_ref() {
                    if tokens[pos].class == "PPI::Token::Operator" {
                        if let Some(next_prec) = self.get_precedence(next_op) {
                            if next_prec > precedence {
                                // Right-associative: parse the right side with higher precedence
                                if let Some(parsed_right) =
                                    self.parse_precedence(tokens, pos - 1, next_prec)
                                {
                                    right = parsed_right;
                                    // Update position - this is approximate, but we'll break anyway
                                    pos = tokens.len();
                                }
                            }
                        }
                    }
                }
            }

            // Create binary operation node
            left = PpiNode {
                class: "BinaryOperation".to_string(),
                content: Some(op.clone()),
                children: vec![left, right],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            };

            debug!("Created binary operation: {}", op);
        }

        Some(left)
    }

    /// Preprocess unary operators by converting them into BinaryOperation nodes
    /// This runs before precedence climbing to handle expressions like -$val/256
    /// Unary minus becomes subtraction from zero: -$val → (0 - $val)
    fn preprocess_unary_operators(&self, tokens: Vec<PpiNode>) -> Vec<PpiNode> {
        let mut result = Vec::new();
        let mut i = 0;

        while i < tokens.len() {
            // Skip whitespace and comments
            if matches!(
                tokens[i].class.as_str(),
                "PPI::Token::Whitespace" | "PPI::Token::Comment"
            ) {
                result.push(tokens[i].clone());
                i += 1;
                continue;
            }

            // Check for unary prefix operator followed by operand
            if i + 1 < tokens.len()
                && self.is_unary_prefix_operator(&tokens[i])
                && !self.is_operator(&tokens[i + 1])
            {
                let operator = tokens[i].content.as_deref().unwrap_or("");

                // Create BinaryOperation node for unary operations
                let binary_node = match operator {
                    "-" => {
                        // Unary minus: -$val → (0 - $val)
                        // The reverse Sub<&TagValue> for i32 impl handles the type conversion
                        let zero_node = PpiNode {
                            class: "PPI::Token::Number".to_string(),
                            content: Some("0".to_string()),
                            children: vec![],
                            symbol_type: None,
                            numeric_value: Some(0.0),
                            string_value: None,
                            structure_bounds: None,
                        };

                        PpiNode {
                            class: "BinaryOperation".to_string(),
                            content: Some("-".to_string()),
                            children: vec![zero_node, tokens[i + 1].clone()],
                            symbol_type: None,
                            numeric_value: None,
                            string_value: None,
                            structure_bounds: None,
                        }
                    }
                    "+" => {
                        // Unary plus: +$val → (0 + $val)
                        let zero_node = PpiNode {
                            class: "PPI::Token::Number".to_string(),
                            content: Some("0".to_string()),
                            children: vec![],
                            symbol_type: None,
                            numeric_value: Some(0.0),
                            string_value: None,
                            structure_bounds: None,
                        };

                        PpiNode {
                            class: "BinaryOperation".to_string(),
                            content: Some("+".to_string()),
                            children: vec![zero_node, tokens[i + 1].clone()],
                            symbol_type: None,
                            numeric_value: None,
                            string_value: None,
                            structure_bounds: None,
                        }
                    }
                    _ => {
                        // For other unary operators like ! and ~, keep as binary but note the limitation
                        debug!("Unary operator {} converted to binary form - may need special handling", operator);
                        let zero_node = PpiNode {
                            class: "PPI::Token::Number".to_string(),
                            content: Some("0".to_string()),
                            children: vec![],
                            symbol_type: None,
                            numeric_value: Some(0.0),
                            string_value: None,
                            structure_bounds: None,
                        };

                        PpiNode {
                            class: "BinaryOperation".to_string(),
                            content: tokens[i].content.clone(),
                            children: vec![zero_node, tokens[i + 1].clone()],
                            symbol_type: None,
                            numeric_value: None,
                            string_value: None,
                            structure_bounds: None,
                        }
                    }
                };

                debug!(
                    "Created binary operation for unary: {} {} → BinaryOperation({}, 0, {})",
                    operator,
                    tokens[i + 1].content.as_deref().unwrap_or(""),
                    operator,
                    tokens[i + 1].content.as_deref().unwrap_or("")
                );

                result.push(binary_node);
                i += 2; // Skip both operator and operand
            } else {
                result.push(tokens[i].clone());
                i += 1;
            }
        }

        result
    }

    /// Check if token is a unary prefix operator
    fn is_unary_prefix_operator(&self, token: &PpiNode) -> bool {
        token.class == "PPI::Token::Operator"
            && matches!(token.content.as_deref(), Some("-" | "+" | "!" | "~"))
    }

    /// Check if token is any operator
    fn is_operator(&self, token: &PpiNode) -> bool {
        token.class == "PPI::Token::Operator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precedence_table() {
        let normalizer = BinaryOperatorNormalizer;
        assert_eq!(normalizer.get_precedence("*"), Some(40));
        assert_eq!(normalizer.get_precedence("+"), Some(20));
        assert_eq!(normalizer.get_precedence("unknown"), None);
    }

    #[test]
    fn test_has_binary_operators() {
        let normalizer = BinaryOperatorNormalizer;

        let node_with_mult = PpiNode {
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
            ],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        assert!(normalizer.has_binary_operators(&node_with_mult));
    }

    #[test]
    fn test_simple_binary_operation() {
        let normalizer = BinaryOperatorNormalizer;

        // Test $val * 100 pattern
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

        let result = normalizer.parse_binary_expression(tokens);
        assert!(result.is_some());

        let binary_op = result.unwrap();
        assert_eq!(binary_op.class, "BinaryOperation");
        assert_eq!(binary_op.content, Some("*".to_string()));
        assert_eq!(binary_op.children.len(), 2);
    }
}
