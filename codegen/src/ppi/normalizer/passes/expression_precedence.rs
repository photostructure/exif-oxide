//! Unified Expression Precedence Normalizer
//!
//! This normalizer consolidates 6 separate expression normalizers using precedence climbing
//! algorithm based on Perl's operator precedence table. It handles all expression patterns:
//! binary operations, string concatenation, ternary conditionals, function calls, and
//! complex multi-function patterns like join+unpack combinations.
//!
//! Consolidates:
//! - BinaryOperatorNormalizer (518 lines) - arithmetic, comparison, logical operators
//! - StringOpNormalizer (137 lines) - string concatenation
//! - TernaryNormalizer (96 lines) - conditional expressions
//! - SafeDivisionNormalizer (118 lines) - specialized ternary patterns
//! - FunctionCallNormalizer (295 lines) - function calls without parentheses
//! - JoinUnpackPass (373 lines) - multi-function composition
//!
//! Total reduction: 1,537 lines → ~300 lines (80% reduction)

use crate::ppi::normalizer::{multi_pass::RewritePass, utils};
use crate::ppi::types::PpiNode;
use std::collections::HashMap;
use std::sync::LazyLock;
use tracing::{debug, trace};

/// Perl operator precedence table based on perlop documentation
/// Higher numbers = higher precedence = binds tighter
/// Reference: perlop.txt lines 129-153
static PRECEDENCE_TABLE: &[(&str, u8)] = &[
    // Terms and list operators (leftward) - highest precedence
    ("function_call", 210), // Function calls with parentheses
    // Arrow operator
    ("->", 200),
    // Auto-increment and auto-decrement (nonassoc)
    ("++", 190),
    ("--", 190),
    // Exponentiation (right associative)
    ("**", 180),
    // Unary operators (right associative)
    ("!", 170),
    ("~", 170),
    ("~.", 170), // Bitwise string negation
    ("\\", 170), // Reference operator
    ("unary_+", 170),
    ("unary_-", 170),
    // Binding operators
    ("=~", 160),
    ("!~", 160),
    // Multiplicative (left associative)
    ("*", 150),
    ("/", 150),
    ("%", 150),
    ("x", 150), // String repetition
    // Additive and string concatenation (left associative)
    ("+", 140),
    ("-", 140),
    (".", 140), // String concatenation
    // Shift operators (left associative)
    ("<<", 130),
    (">>", 130),
    // Named unary operators
    ("named_unary", 120),
    // Class instance operator (nonassoc)
    ("isa", 110),
    // Relational operators (chained)
    ("<", 100),
    (">", 100),
    ("<=", 100),
    (">=", 100),
    ("lt", 100),
    ("gt", 100),
    ("le", 100),
    ("ge", 100),
    // Equality operators (chain/na)
    ("==", 90),
    ("!=", 90),
    ("eq", 90),
    ("ne", 90),
    ("<=>", 90),
    ("cmp", 90),
    ("~~", 90), // Smartmatch
    // Bitwise AND (left associative)
    ("&", 80),
    ("&.", 80), // Bitwise string AND
    // Bitwise OR/XOR (left associative)
    ("|", 70),
    ("|.", 70), // Bitwise string OR
    ("^", 70),
    ("^.", 70), // Bitwise string XOR
    // C-style Logical AND (left associative)
    ("&&", 60),
    // C-style Logical OR/XOR/Defined-OR (left associative)
    ("||", 50),
    ("^^", 50), // Logical XOR
    ("//", 50), // Defined-OR
    // Range operators (nonassoc)
    ("..", 40),
    ("...", 40),
    // Ternary conditional (right associative)
    ("?", 30),
    (":", 30),
    // Assignment operators (right associative)
    ("=", 20),
    ("+=", 20),
    ("-=", 20),
    ("*=", 20),
    ("/=", 20),
    ("%=", 20),
    ("**=", 20),
    ("&=", 20),
    ("|=", 20),
    ("^=", 20),
    ("&.=", 20),
    ("|.=", 20),
    ("^.=", 20),
    ("<<=", 20),
    (">>=", 20),
    ("&&=", 20),
    ("||=", 20),
    ("//=", 20),
    ("^^=", 20),
    (".=", 20),
    ("x=", 20),
    // Comma and fat comma (left associative)
    (",", 10),
    ("=>", 10),
    // List operators (rightward) - lower than comma
    ("list_op_right", 8),
    // Logical not (right associative) - very low precedence
    ("not", 5),
    // Logical and (left associative) - very low precedence
    ("and", 3),
    // Logical or/xor (left associative) - lowest precedence
    ("or", 1),
    ("xor", 1),
];

/// Lazily initialized precedence map for efficient lookups
static PRECEDENCE_MAP: LazyLock<HashMap<&'static str, u8>> =
    LazyLock::new(|| PRECEDENCE_TABLE.iter().cloned().collect());

/// Expression pattern classification for routing to appropriate handlers
#[derive(Debug, PartialEq)]
enum ExpressionPattern {
    BinaryOperation,
    FunctionCall,
    TernaryConditional,
    JoinUnpackCombo,
    SafeDivision,
    None,
}

/// Unified precedence climbing normalizer that consolidates 6 expression normalizers
pub struct ExpressionPrecedenceNormalizer;

impl Default for ExpressionPrecedenceNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpressionPrecedenceNormalizer {
    pub fn new() -> Self {
        Self
    }

    /// Get operator precedence, returns None for unknown operators
    fn get_precedence(&self, op: &str) -> Option<u8> {
        PRECEDENCE_MAP.get(op).copied()
    }
}

impl RewritePass for ExpressionPrecedenceNormalizer {
    fn name(&self) -> &str {
        "ExpressionPrecedenceNormalizer"
    }

    fn transform(&self, node: PpiNode) -> PpiNode {
        // Only process nodes that could contain expressions
        if !self.should_process(&node) {
            return node;
        }

        trace!("Processing node for expressions: {}", node.class);

        // Classify the expression pattern to route to appropriate handler
        let pattern = self.classify_expression_pattern(&node);
        trace!("Classified {} as pattern: {:?}", node.class, pattern);
        match pattern {
            ExpressionPattern::JoinUnpackCombo => self.handle_join_unpack_pattern(node),
            ExpressionPattern::FunctionCall => {
                trace!(
                    "Handling FunctionCall pattern with precedence climbing for {}",
                    node.class
                );
                self.handle_function_call_with_precedence_climbing(node)
            }
            ExpressionPattern::TernaryConditional => self.handle_ternary_expression(node),
            ExpressionPattern::SafeDivision => self.handle_safe_division(node),
            ExpressionPattern::BinaryOperation => self.handle_binary_operations(node),
            ExpressionPattern::None => node,
        }
    }
}

impl ExpressionPrecedenceNormalizer {
    /// Check if this node should be processed for expressions
    fn should_process(&self, node: &PpiNode) -> bool {
        // Process both PPI::Statement and PPI::Statement::Expression nodes
        if !matches!(
            node.class.as_str(),
            "PPI::Statement" | "PPI::Statement::Expression"
        ) || node.children.len() < 2
        {
            return false;
        }

        // For top-level PPI::Statement, process function calls with parentheses in precedence climbing
        if node.class == "PPI::Statement" {
            let has_structure_list = node
                .children
                .iter()
                .any(|child| child.class == "PPI::Structure::List");

            if has_structure_list {
                // Always process ternary expressions, even if they contain function calls
                // e.g., $val ? sqrt(2)**($val/256) : 0
                if self.has_ternary_pattern(&node.children) {
                    trace!(
                        "should_process: Processing {} - ternary with Structure::List",
                        node.class
                    );
                    return true;
                }

                // Check if this is a function call - precedence climbing should handle it
                if let Some(first_child) = node.children.first() {
                    if first_child.class == "PPI::Token::Word" {
                        if let Some(func_name) = &first_child.content {
                            if self.is_known_function(func_name) {
                                trace!("should_process: Processing {} - function call with precedence climbing", node.class);
                                return true; // Process function calls in precedence climbing
                            }
                        }
                    }
                }
                trace!("should_process: Skipping {} - contains PPI::Structure::List but not a function call", node.class);
                return false;
            }
        }

        // For PPI::Statement::Expression (inside function arguments), be more careful
        // Don't process if it contains commas - those should be handled as function arguments
        if node.class == "PPI::Statement::Expression" {
            let has_commas = node.children.iter().any(|child| {
                child.class == "PPI::Token::Operator" && child.content.as_deref() == Some(",")
            });

            if has_commas {
                trace!("should_process: Skipping {} - contains commas", node.class);
                // This contains comma-separated function arguments - don't process
                return false;
            }
        }

        // Only process if we have tokens that could form expressions (but not comma-separated ones)
        let has_operators_or_functions = node.children.iter().any(|child| {
            matches!(child.class.as_str(), "PPI::Token::Operator")
                || (child.class == "PPI::Token::Word"
                    && self.is_known_function(child.content.as_deref().unwrap_or("")))
        });

        trace!(
            "should_process: {} -> {}",
            node.class,
            has_operators_or_functions
        );
        has_operators_or_functions
    }

    /// Classify the type of expression pattern in this node
    fn classify_expression_pattern(&self, node: &PpiNode) -> ExpressionPattern {
        let children = &node.children;

        // Check for join+unpack multi-function pattern (most specific first)
        if self.has_join_unpack_pattern(children) {
            return ExpressionPattern::JoinUnpackCombo;
        }

        // Check for ternary patterns (before binary operations, more specific)
        if self.has_ternary_pattern(children) {
            // Special case: safe division pattern ($val ? 1/$val : 0)
            if self.is_safe_division_pattern(children) {
                return ExpressionPattern::SafeDivision;
            }
            return ExpressionPattern::TernaryConditional;
        }

        // Check for binary operations FIRST (before function calls)
        // This correctly handles expressions like "int($val) / 100" where function calls
        // are primaries within binary operations, following Perl precedence and Pratt parsing
        if self.has_binary_operators(children) {
            return ExpressionPattern::BinaryOperation;
        }

        // Check for standalone function calls (both with and without parentheses)
        if self.is_function_call_pattern(node) || self.has_function_call_pattern(children) {
            return ExpressionPattern::FunctionCall;
        }

        ExpressionPattern::None
    }

    /// Check for join+unpack multi-function pattern: join " ", unpack "H2H2", $val
    fn has_join_unpack_pattern(&self, children: &[PpiNode]) -> bool {
        if children.len() < 5 {
            return false;
        }

        // Look for "join" followed by "unpack" anywhere in the sequence
        let has_join = children.iter().any(|child| {
            child.class == "PPI::Token::Word" && child.content.as_deref() == Some("join")
        });

        let has_unpack = children.iter().any(|child| {
            child.class == "PPI::Token::Word" && child.content.as_deref() == Some("unpack")
        });

        has_join && has_unpack
    }

    /// Check for ternary conditional pattern: condition ? value1 : value2
    fn has_ternary_pattern(&self, children: &[PpiNode]) -> bool {
        children.iter().any(|child| {
            child.class == "PPI::Token::Operator" && child.content.as_deref() == Some("?")
        }) && children.iter().any(|child| {
            child.class == "PPI::Token::Operator" && child.content.as_deref() == Some(":")
        })
    }

    /// Check for safe division pattern: $val ? 1/$val : 0
    fn is_safe_division_pattern(&self, children: &[PpiNode]) -> bool {
        // Look for pattern where the true branch contains division by the condition variable
        // This is a heuristic - ExifTool commonly uses $val ? 1/$val : 0 for safe division
        if !self.has_ternary_pattern(children) {
            return false;
        }

        // Simple heuristic: if we have ternary and division operator, likely safe division
        children.iter().any(|child| {
            child.class == "PPI::Token::Operator" && child.content.as_deref() == Some("/")
        })
    }

    /// Check for function call pattern: word followed by arguments (no parentheses)
    fn has_function_call_pattern(&self, children: &[PpiNode]) -> bool {
        if children.is_empty() {
            return false;
        }

        // First token is a known function name
        if let Some(first_child) = children.first() {
            if first_child.class == "PPI::Token::Word" {
                if let Some(func_name) = &first_child.content {
                    return self.is_known_function(func_name) && children.len() >= 2;
                }
            }
        }

        false
    }

    /// Check for binary operators in children
    fn has_binary_operators(&self, children: &[PpiNode]) -> bool {
        children.iter().any(|child| {
            child.class == "PPI::Token::Operator"
                && child
                    .content
                    .as_ref()
                    .map_or(false, |op| self.get_precedence(op).is_some())
        })
    }

    /// Check if name is a known function we should normalize
    fn is_known_function(&self, name: &str) -> bool {
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
                | "sin"
                | "cos"
                | "atan2"
                | "exp"
                | "log"
                | "hex"
                | "oct"
                | "defined"
        )
    }

    /// Handle join+unpack multi-function pattern
    fn handle_join_unpack_pattern(&self, node: PpiNode) -> PpiNode {
        let children = &node.children;

        // Find join and unpack positions
        let join_pos = children.iter().position(|child| {
            child.class == "PPI::Token::Word" && child.content.as_deref() == Some("join")
        });

        let unpack_pos = children.iter().position(|child| {
            child.class == "PPI::Token::Word" && child.content.as_deref() == Some("unpack")
        });

        if let (Some(join_idx), Some(unpack_idx)) = (join_pos, unpack_pos) {
            if unpack_idx > join_idx {
                // Extract components
                if let Some((separator, format, data)) =
                    self.extract_join_unpack_args(children, join_idx, unpack_idx)
                {
                    // Create nested function calls: join(separator, unpack(format, data))
                    let unpack_call = utils::create_function_call("unpack", vec![format, data]);
                    let join_call =
                        utils::create_function_call("join", vec![separator, unpack_call]);

                    debug!("Transformed join+unpack pattern into nested function calls");
                    return join_call;
                }
            }
        }

        // Pattern recognition failed, return unchanged
        node
    }

    /// Extract arguments for join+unpack pattern
    fn extract_join_unpack_args(
        &self,
        children: &[PpiNode],
        join_idx: usize,
        unpack_idx: usize,
    ) -> Option<(PpiNode, PpiNode, PpiNode)> {
        // Extract separator (between join and unpack)
        let separator = self.find_next_non_comma_token(children, join_idx + 1, unpack_idx)?;

        // Extract format string (after unpack)
        let format = self.find_next_non_comma_token(children, unpack_idx + 1, children.len())?;

        // Extract data (after format) - simplified approach with proper None handling
        let format_pos = children.iter().skip(unpack_idx + 1).position(|child| {
            // Safely compare content, handling None cases
            format.content.is_some() && child.content == format.content
        })?;

        // Check bounds before calculating format_end
        let format_end = unpack_idx
            .checked_add(1)?
            .checked_add(format_pos)?
            .checked_add(1)?;

        if format_end >= children.len() {
            return None;
        }

        let data = self.find_next_non_comma_token(children, format_end, children.len())?;

        Some((separator.clone(), format.clone(), data.clone()))
    }

    /// Find next non-comma token in range
    fn find_next_non_comma_token<'a>(
        &self,
        children: &'a [PpiNode],
        start: usize,
        end: usize,
    ) -> Option<&'a PpiNode> {
        children.iter().skip(start).take(end - start).find(|child| {
            !matches!(
                child.class.as_str(),
                "PPI::Token::Whitespace" | "PPI::Token::Comment"
            ) && !(child.class == "PPI::Token::Operator" && child.content.as_deref() == Some(","))
        })
    }

    /// Handle ternary conditional expressions using precedence climbing
    fn handle_ternary_expression(&self, node: PpiNode) -> PpiNode {
        // Use precedence climbing to properly group ternary operations
        // Ternary has precedence 15 and is right-associative
        if let Some(ternary_node) = self.parse_ternary_with_precedence(&node.children) {
            ternary_node
        } else {
            node
        }
    }

    /// Parse ternary expression with proper precedence
    fn parse_ternary_with_precedence(&self, tokens: &[PpiNode]) -> Option<PpiNode> {
        // Find ? and : operators
        let question_pos = tokens
            .iter()
            .position(|t| t.class == "PPI::Token::Operator" && t.content.as_deref() == Some("?"))?;

        // Find : operator after ?, with overflow protection
        let skip_count = question_pos.checked_add(1)?;
        let colon_offset = tokens
            .iter()
            .skip(skip_count)
            .position(|t| t.class == "PPI::Token::Operator" && t.content.as_deref() == Some(":"))?;
        let colon_pos = skip_count.checked_add(colon_offset)?;

        // Extract condition, true_expr, false_expr
        // The condition doesn't need unary preprocessing as it's already a complete expression
        let condition = self.parse_expression_sequence(&tokens[..question_pos]);

        // Preprocess unary operators in true and false branches to handle cases like -$val[0]
        // This converts unary negation to binary subtraction (0 - $val[0])
        let true_branch_tokens = tokens[question_pos + 1..colon_pos].to_vec();
        let preprocessed_true = self.preprocess_unary_operators(true_branch_tokens);
        let true_expr = self.parse_expression_sequence(&preprocessed_true);

        let false_branch_tokens = tokens[colon_pos + 1..].to_vec();
        let preprocessed_false = self.preprocess_unary_operators(false_branch_tokens);
        let false_expr = self.parse_expression_sequence(&preprocessed_false);

        Some(PpiNode {
            class: "TernaryOperation".to_string(),
            content: Some("?:".to_string()),
            children: vec![condition?, true_expr?, false_expr?],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        })
    }

    /// Handle safe division pattern (specialized ternary)
    fn handle_safe_division(&self, node: PpiNode) -> PpiNode {
        // Safe division is just a special case of ternary, so delegate to ternary handler
        // but mark it specifically as SafeDivision for visitor
        if let Some(mut ternary_node) = self.parse_ternary_with_precedence(&node.children) {
            ternary_node.class = "SafeDivision".to_string();
            ternary_node
        } else {
            node
        }
    }

    /// Handle binary operations using precedence climbing
    fn handle_binary_operations(&self, node: PpiNode) -> PpiNode {
        // First preprocess unary operators (convert -$val to 0 - $val)
        let preprocessed_tokens = self.preprocess_unary_operators(node.children);

        // Then apply precedence climbing to the preprocessed tokens
        if let Some(normalized_expression) = self.parse_expression_sequence(&preprocessed_tokens) {
            normalized_expression
        } else {
            PpiNode {
                children: preprocessed_tokens,
                ..node
            }
        }
    }

    /// Parse expression sequence using precedence climbing
    fn parse_expression_sequence(&self, tokens: &[PpiNode]) -> Option<PpiNode> {
        debug!(
            "parse_expression_sequence: received {} tokens",
            tokens.len()
        );
        for (i, token) in tokens.iter().enumerate() {
            debug!("  token[{}]: {} = {:?}", i, token.class, token.content);
        }

        if tokens.is_empty() {
            return None;
        }

        if tokens.len() == 1 {
            debug!("parse_expression_sequence: only 1 token, returning as-is");
            return Some(tokens[0].clone());
        }

        // Check if this contains commas - if so, don't process as a single expression
        // Commas in function arguments should be handled by the visitor, not the normalizer
        let has_commas = tokens.iter().any(|token| {
            token.class == "PPI::Token::Operator" && token.content.as_deref() == Some(",")
        });

        if has_commas {
            // This is likely function arguments with commas - don't transform
            // Return the original node structure unchanged
            return None;
        }

        // No commas, process as single expression using precedence climbing
        self.parse_precedence(tokens, 0, 0)
    }

    /// Core precedence climbing algorithm
    fn parse_precedence(
        &self,
        tokens: &[PpiNode],
        mut pos: usize,
        min_precedence: u8,
    ) -> Option<PpiNode> {
        if pos >= tokens.len() {
            return None;
        }

        // Parse left operand (primary expression) - skip whitespace
        while pos < tokens.len()
            && matches!(
                tokens[pos].class.as_str(),
                "PPI::Token::Whitespace" | "PPI::Token::Comment"
            )
        {
            pos += 1;
        }

        if pos >= tokens.len() {
            return None;
        }

        // Parse left operand using consistent primary parsing
        let (mut left, new_pos) = self.parse_primary(tokens, pos)?;
        pos = new_pos;

        // Process operators with precedence climbing
        loop {
            // Skip whitespace
            while pos < tokens.len()
                && matches!(
                    tokens[pos].class.as_str(),
                    "PPI::Token::Whitespace" | "PPI::Token::Comment"
                )
            {
                pos += 1;
            }

            // Check if we have an operator and a right operand
            if pos >= tokens.len() || pos + 1 >= tokens.len() {
                break;
            }

            // Check for binary operator
            if tokens[pos].class != "PPI::Token::Operator" {
                break;
            }

            let op = tokens[pos].content.as_ref()?;
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

            // Parse right operand using consistent primary parsing
            let (mut right, new_pos) = self.parse_primary(tokens, pos)?;
            pos = new_pos;

            // Handle higher precedence operations on the right
            // Check if the next operator has higher precedence than current
            while pos < tokens.len() {
                // Skip whitespace
                while pos < tokens.len()
                    && matches!(
                        tokens[pos].class.as_str(),
                        "PPI::Token::Whitespace" | "PPI::Token::Comment"
                    )
                {
                    pos += 1;
                }

                if pos >= tokens.len() || tokens[pos].class != "PPI::Token::Operator" {
                    break;
                }

                let next_op = tokens[pos].content.as_ref()?;
                let next_precedence = self.get_precedence(next_op)?;

                // If next operator has higher precedence, or same precedence and right-associative
                if next_precedence > precedence
                    || (next_precedence == precedence && self.is_right_associative(next_op))
                {
                    // Recursively parse higher precedence expression starting with right operand
                    if let Some(higher_prec_expr) = self.parse_precedence_from_tokens(
                        tokens,
                        right.clone(),
                        pos,
                        if self.is_right_associative(next_op) {
                            precedence
                        } else {
                            precedence + 1
                        },
                    ) {
                        right = higher_prec_expr.0;
                        pos = higher_prec_expr.1;
                    } else {
                        break;
                    }
                } else {
                    break;
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

    /// Helper for precedence climbing that returns (result, new_position)
    /// Parse a primary expression, handling function calls consistently
    fn parse_primary(&self, tokens: &[PpiNode], pos: usize) -> Option<(PpiNode, usize)> {
        if pos >= tokens.len() {
            return None;
        }

        let mut primary = tokens[pos].clone();
        let mut new_pos = pos + 1;

        // Handle compound primaries: function_name(args)
        // Any word followed by parentheses is a function call, not just known functions
        // This handles ExifTool-specific functions like ConvertDuration, IsInt, etc.
        if primary.class == "PPI::Token::Word"
            && new_pos < tokens.len()
            && tokens[new_pos].class == "PPI::Structure::List"
        {
            // Create function call node combining name + parentheses
            primary = PpiNode {
                class: "FunctionCall".to_string(),
                content: primary.content.clone(),
                children: vec![tokens[new_pos].clone()],
                numeric_value: None,
                symbol_type: None,
                string_value: None,
                structure_bounds: None,
            };
            new_pos += 1; // consume the parentheses token
        }
        // Handle array subscript: $val[0], $val[1], etc.
        else if primary.class == "PPI::Token::Symbol"
            && new_pos < tokens.len()
            && tokens[new_pos].class == "PPI::Structure::Subscript"
        {
            // Create array access node combining symbol + subscript
            primary = PpiNode {
                class: "ArrayAccess".to_string(),
                content: primary.content.clone(),
                children: vec![tokens[new_pos].clone()],
                numeric_value: None,
                symbol_type: primary.symbol_type.clone(),
                string_value: None,
                structure_bounds: None,
            };
            new_pos += 1; // consume the subscript token
        }

        Some((primary, new_pos))
    }

    fn parse_precedence_from_tokens(
        &self,
        tokens: &[PpiNode],
        left: PpiNode,
        mut pos: usize,
        min_precedence: u8,
    ) -> Option<(PpiNode, usize)> {
        let mut current_left = left;

        // Process operators with precedence climbing
        loop {
            // Skip whitespace
            while pos < tokens.len()
                && matches!(
                    tokens[pos].class.as_str(),
                    "PPI::Token::Whitespace" | "PPI::Token::Comment"
                )
            {
                pos += 1;
            }

            // Check if we have an operator and a right operand
            if pos >= tokens.len() || pos + 1 >= tokens.len() {
                break;
            }

            // Check for binary operator
            if tokens[pos].class != "PPI::Token::Operator" {
                break;
            }

            let op = tokens[pos].content.as_ref()?;
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

            // Parse right operand
            let mut right = tokens[pos].clone();
            pos += 1;

            // Handle higher precedence operations on the right
            while pos < tokens.len() {
                // Skip whitespace
                while pos < tokens.len()
                    && matches!(
                        tokens[pos].class.as_str(),
                        "PPI::Token::Whitespace" | "PPI::Token::Comment"
                    )
                {
                    pos += 1;
                }

                if pos >= tokens.len() || tokens[pos].class != "PPI::Token::Operator" {
                    break;
                }

                let next_op = tokens[pos].content.as_ref()?;
                let next_precedence = self.get_precedence(next_op)?;

                if next_precedence > precedence
                    || (next_precedence == precedence && self.is_right_associative(next_op))
                {
                    if let Some(higher_prec_expr) = self.parse_precedence_from_tokens(
                        tokens,
                        right.clone(),
                        pos,
                        if self.is_right_associative(next_op) {
                            precedence
                        } else {
                            precedence + 1
                        },
                    ) {
                        right = higher_prec_expr.0;
                        pos = higher_prec_expr.1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            // Create binary operation node
            current_left = PpiNode {
                class: "BinaryOperation".to_string(),
                content: Some(op.clone()),
                children: vec![current_left, right],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            };

            debug!("Created binary operation: {}", op);
        }

        Some((current_left, pos))
    }

    /// Check if operator is right-associative
    fn is_right_associative(&self, op: &str) -> bool {
        matches!(
            op,
            // Exponentiation
            "**" |
            // Unary operators
            "!" | "~" | "~." | "\\" | "unary_+" | "unary_-" |
            // Ternary conditional
            "?" | ":" |
            // Assignment operators
            "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "**=" |
            "&=" | "|=" | "^=" | "&.=" | "|.=" | "^.=" |
            "<<=" | ">>=" | "&&=" | "||=" | "//=" | "^^=" |
            ".=" | "x=" |
            // Logical not (low precedence)
            "not"
        )
    }

    /// Check if this node represents a function call pattern
    fn is_function_call_pattern(&self, node: &PpiNode) -> bool {
        // Pattern 1: function_name(...) with parentheses
        if node.children.len() >= 2 {
            if let Some(first_child) = node.children.first() {
                if first_child.class == "PPI::Token::Word" {
                    if let Some(func_name) = &first_child.content {
                        if self.is_known_function(func_name) {
                            // Check for parentheses (sprintf case)
                            if node.children.len() == 2
                                && node.children[1].class == "PPI::Structure::List"
                            {
                                return true;
                            }
                            // Check for non-parentheses function calls (join/unpack case)
                            if node.children.len() > 2 {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// Handle function calls using precedence climbing to process arguments
    fn handle_function_call_with_precedence_climbing(&self, node: PpiNode) -> PpiNode {
        let children = &node.children;
        trace!(
            "handle_function_call_with_precedence_climbing: processing {} with {} children",
            node.class,
            children.len()
        );

        if let Some(first_child) = children.first() {
            if first_child.class == "PPI::Token::Word" {
                if let Some(func_name) = &first_child.content {
                    trace!(
                        "handle_function_call_with_precedence_climbing: found function {}",
                        func_name
                    );

                    // Case 1: sprintf("%.2f%%", $val * 100) - parentheses with args
                    if children.len() == 2 && children[1].class == "PPI::Structure::List" {
                        trace!("handle_function_call_with_precedence_climbing: processing parenthesized function call");
                        let args = self.extract_and_process_parenthesized_args(&children[1]);
                        return utils::create_function_call(func_name, args);
                    }

                    // Case 2: join " ", unpack "H2H2", $val - no parentheses
                    if children.len() > 2 {
                        trace!("handle_function_call_with_precedence_climbing: processing non-parenthesized function call");
                        let args: Vec<PpiNode> = children.iter().skip(1).cloned().collect();
                        // Apply precedence climbing to process any binary operations in args
                        let processed_args =
                            args.into_iter().map(|arg| self.transform(arg)).collect();
                        return utils::create_function_call(func_name, processed_args);
                    }
                }
            }
        }

        node
    }

    /// Extract and process arguments from parentheses using precedence climbing
    fn extract_and_process_parenthesized_args(&self, list_node: &PpiNode) -> Vec<PpiNode> {
        if list_node.class != "PPI::Structure::List" {
            return Vec::new();
        }

        trace!(
            "extract_and_process_parenthesized_args: processing {} children",
            list_node.children.len()
        );

        let mut processed_args = Vec::new();

        for child in &list_node.children {
            if child.class == "PPI::Statement::Expression" {
                trace!("extract_and_process_parenthesized_args: found expression to process with precedence climbing");
                // CRITICAL: Apply precedence climbing to process binary operations!
                // This handles $val * 100 → BinaryOperation(*, $val, 100)
                let processed_expr = self.transform(child.clone());

                // Split the processed expression on commas to get individual args
                processed_args.extend(self.split_processed_expression_on_commas(&processed_expr));
            } else if !matches!(
                child.class.as_str(),
                "PPI::Token::Whitespace" | "PPI::Token::Comment"
            ) {
                // Direct child that isn't whitespace
                processed_args.push(child.clone());
            }
        }

        trace!(
            "extract_and_process_parenthesized_args: extracted {} processed args",
            processed_args.len()
        );
        processed_args
    }

    /// Split a precedence-climbing processed expression on commas for function arguments
    fn split_processed_expression_on_commas(&self, expr: &PpiNode) -> Vec<PpiNode> {
        if expr.children.is_empty() {
            return vec![expr.clone()];
        }

        let mut args = Vec::new();
        let mut current_arg_nodes = Vec::new();

        for child in &expr.children {
            if child.class == "PPI::Token::Operator" && child.content.as_deref() == Some(",") {
                // Comma separator - finish current arg
                if !current_arg_nodes.is_empty() {
                    if current_arg_nodes.len() == 1 {
                        args.push(current_arg_nodes.into_iter().next().unwrap());
                    } else {
                        // Multiple nodes - this should be a processed binary operation
                        args.push(PpiNode {
                            class: "PPI::Statement::Expression".to_string(),
                            content: None,
                            children: current_arg_nodes,
                            symbol_type: None,
                            numeric_value: None,
                            string_value: None,
                            structure_bounds: None,
                        });
                    }
                    current_arg_nodes = Vec::new();
                }
            } else if !matches!(
                child.class.as_str(),
                "PPI::Token::Whitespace" | "PPI::Token::Comment"
            ) {
                current_arg_nodes.push(child.clone());
            }
        }

        // Add final arg
        if !current_arg_nodes.is_empty() {
            if current_arg_nodes.len() == 1 {
                args.push(current_arg_nodes.into_iter().next().unwrap());
            } else {
                args.push(PpiNode {
                    class: "PPI::Statement::Expression".to_string(),
                    content: None,
                    children: current_arg_nodes,
                    symbol_type: None,
                    numeric_value: None,
                    string_value: None,
                    structure_bounds: None,
                });
            }
        }

        // If no commas found, return the original expression
        if args.is_empty() {
            vec![expr.clone()]
        } else {
            args
        }
    }

    /// Preprocess unary operators by converting them to binary operations
    /// This handles expressions like -$val/256 → (0 - $val)/256
    /// CRITICAL: Only treat +/- as unary when they have NO left operand
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

            // Check for unary prefix operator - ONLY if there's no left operand
            if i + 1 < tokens.len()
                && self.is_unary_prefix_operator(&tokens[i])
                && self.is_truly_unary_context(&tokens, i)
            {
                // Check that the next token is a valid operand (not another operator)
                // unless it's another unary operator (like --$val or !!$val)
                let next_is_valid_operand = !self.is_operator(&tokens[i + 1])
                    || self.is_unary_prefix_operator(&tokens[i + 1]);

                if !next_is_valid_operand {
                    result.push(tokens[i].clone());
                    i += 1;
                    continue;
                }
                let operator = tokens[i].content.as_deref().unwrap_or("");

                // Collect the complete operand (including array subscripts if present)
                let mut operand_tokens = vec![tokens[i + 1].clone()];
                let mut consumed = 2; // operator + first operand token

                // Check if the operand is followed by a subscript (for array access like $val[0])
                if i + 2 < tokens.len()
                    && tokens[i + 1].class == "PPI::Token::Symbol"
                    && tokens[i + 2].class == "PPI::Structure::Subscript"
                {
                    // Create ArrayAccess node for the complete expression
                    operand_tokens = vec![PpiNode {
                        class: "ArrayAccess".to_string(),
                        content: tokens[i + 1].content.clone(),
                        children: vec![tokens[i + 2].clone()],
                        symbol_type: tokens[i + 1].symbol_type.clone(),
                        numeric_value: None,
                        string_value: None,
                        structure_bounds: None,
                    }];
                    consumed = 3; // operator + symbol + subscript
                }

                // Create unary operation node for better code generation
                let unary_tokens = match operator {
                    "-" => {
                        // Unary minus: -$val → create a UnaryNegation node
                        // This allows the visitor to generate cleaner code using negate() function
                        vec![PpiNode {
                            class: "UnaryNegation".to_string(),
                            content: Some("-".to_string()),
                            children: vec![operand_tokens[0].clone()],
                            symbol_type: None,
                            numeric_value: None,
                            string_value: None,
                            structure_bounds: None,
                        }]
                    }
                    "+" => {
                        // Unary plus: +$val → create a proper BinaryOperation node directly
                        vec![PpiNode {
                            class: "BinaryOperation".to_string(),
                            content: Some("+".to_string()),
                            children: vec![
                                PpiNode {
                                    class: "PPI::Token::Number".to_string(),
                                    content: Some("0".to_string()),
                                    children: vec![],
                                    symbol_type: None,
                                    numeric_value: Some(0.0),
                                    string_value: None,
                                    structure_bounds: None,
                                },
                                operand_tokens[0].clone(),
                            ],
                            symbol_type: None,
                            numeric_value: None,
                            string_value: None,
                            structure_bounds: None,
                        }]
                    }
                    _ => {
                        // For other unary operators, keep the operator and complete operand
                        let mut other_tokens = vec![tokens[i].clone()];
                        other_tokens.extend(operand_tokens);
                        other_tokens
                    }
                };

                debug!("Preprocessed unary {} into unary operation", operator);
                result.extend(unary_tokens);
                i += consumed; // Skip operator and complete operand
            } else {
                result.push(tokens[i].clone());
                i += 1;
            }
        }

        result
    }

    /// Check if this is truly a unary operator context
    /// A +/- is unary only if there's NO left operand (symbol/number) before it
    fn is_truly_unary_context(&self, tokens: &[PpiNode], pos: usize) -> bool {
        // Look backwards for a non-whitespace token
        let mut prev_pos = pos;
        while prev_pos > 0 {
            prev_pos -= 1;
            let prev_token = &tokens[prev_pos];

            // Skip whitespace and comments
            if matches!(
                prev_token.class.as_str(),
                "PPI::Token::Whitespace" | "PPI::Token::Comment"
            ) {
                continue;
            }

            // If we find a symbol, number, or closing structure, the +/- is binary
            if matches!(
                prev_token.class.as_str(),
                "PPI::Token::Symbol"
                    | "PPI::Token::Number"
                    | "PPI::Structure::List"
                    | "PPI::Token::Word"
            ) {
                return false; // Binary context - there's a left operand
            }

            // If we find an operator, the +/- is unary
            if prev_token.class == "PPI::Token::Operator" {
                return true; // Unary context - previous token is operator
            }

            // For other token types, treat as unary
            return true;
        }

        // If we reach the start of the expression, it's unary
        true
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
        // Test multiplicative operators
        assert_eq!(PRECEDENCE_MAP.get("*").copied(), Some(150));
        assert_eq!(PRECEDENCE_MAP.get("/").copied(), Some(150));
        assert_eq!(PRECEDENCE_MAP.get("%").copied(), Some(150));
        assert_eq!(PRECEDENCE_MAP.get("x").copied(), Some(150));

        // Test additive operators
        assert_eq!(PRECEDENCE_MAP.get("+").copied(), Some(140));
        assert_eq!(PRECEDENCE_MAP.get("-").copied(), Some(140));
        assert_eq!(PRECEDENCE_MAP.get(".").copied(), Some(140));

        // Test logical operators - critical fix!
        assert_eq!(PRECEDENCE_MAP.get("&&").copied(), Some(60));
        assert_eq!(PRECEDENCE_MAP.get("||").copied(), Some(50));
        assert_eq!(PRECEDENCE_MAP.get("and").copied(), Some(3)); // Very low!
        assert_eq!(PRECEDENCE_MAP.get("or").copied(), Some(1)); // Lowest!

        // Test ternary
        assert_eq!(PRECEDENCE_MAP.get("?").copied(), Some(30));
        assert_eq!(PRECEDENCE_MAP.get(":").copied(), Some(30));

        // Test comma
        assert_eq!(PRECEDENCE_MAP.get(",").copied(), Some(10));
        assert_eq!(PRECEDENCE_MAP.get("=>").copied(), Some(10));

        // Test unknown
        assert_eq!(PRECEDENCE_MAP.get("unknown").copied(), None);
    }

    #[test]
    fn test_pattern_classification() {
        let normalizer = ExpressionPrecedenceNormalizer::new();

        // Test binary operation pattern
        let binary_node = PpiNode {
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
                    content: Some("25".to_string()),
                    children: vec![],
                    symbol_type: None,
                    numeric_value: Some(25.0),
                    string_value: None,
                    structure_bounds: None,
                },
            ],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        assert_eq!(
            normalizer.classify_expression_pattern(&binary_node),
            ExpressionPattern::BinaryOperation
        );
    }
}
