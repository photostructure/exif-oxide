//! Recursive descent parser for converting expressions to AST
//!
//! This module implements recursive descent parsing to convert
//! tokenized expressions into Abstract Syntax Trees (AST).

use super::types::*;

/// Parse a sequence of tokens into an AST using recursive descent
pub fn parse_expression(tokens: Vec<ParseToken>) -> Result<AstNode, String> {
    if tokens.is_empty() {
        return Err("Empty expression".to_string());
    }
    
    // Use recursive descent parser for all expressions
    let mut parser = Parser::new(tokens);
    parser.parse_ternary_expression()
}

/// Recursive descent parser for complex expressions
struct Parser {
    tokens: Vec<ParseToken>,
    position: usize,
}

impl Parser {
    fn new(tokens: Vec<ParseToken>) -> Self {
        Self { tokens, position: 0 }
    }
    
    fn current_token(&self) -> Option<&ParseToken> {
        self.tokens.get(self.position)
    }
    
    fn advance(&mut self) -> Option<&ParseToken> {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
        self.tokens.get(self.position - 1)
    }
    
    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }
    
    /// Parse ternary expression: comparison ? expression : expression
    fn parse_ternary_expression(&mut self) -> Result<AstNode, String> {
        let condition = self.parse_comparison_expression()?;
        
        if matches!(self.current_token(), Some(ParseToken::Question)) {
            self.advance(); // consume '?'
            let true_expr = self.parse_comparison_expression()?;
            
            if !matches!(self.current_token(), Some(ParseToken::Colon)) {
                return Err("Expected ':' in ternary expression".to_string());
            }
            self.advance(); // consume ':'
            
            let false_expr = self.parse_comparison_expression()?;
            
            Ok(AstNode::TernaryOp {
                condition: Box::new(condition),
                true_expr: Box::new(true_expr),
                false_expr: Box::new(false_expr)
            })
        } else {
            Ok(condition)
        }
    }
    
    /// Parse comparison expression: arithmetic >= arithmetic
    fn parse_comparison_expression(&mut self) -> Result<AstNode, String> {
        let left = self.parse_arithmetic_expression()?;
        
        if let Some(ParseToken::Comparison(comp_op)) = self.current_token() {
            let comp_type = comp_op.comp_type;
            self.advance(); // consume comparison operator
            let right = self.parse_arithmetic_expression()?;
            
            Ok(AstNode::ComparisonOp {
                op: comp_type,
                left: Box::new(left),
                right: Box::new(right)
            })
        } else {
            Ok(left)
        }
    }
    
    /// Parse arithmetic expression using precedence climbing
    fn parse_arithmetic_expression(&mut self) -> Result<AstNode, String> {
        self.parse_arithmetic_precedence(0)
    }
    
    /// Parse arithmetic with precedence climbing
    fn parse_arithmetic_precedence(&mut self, min_precedence: u8) -> Result<AstNode, String> {
        let mut left = self.parse_primary_expression()?;
        
        while let Some(ParseToken::Operator(op)) = self.current_token() {
            if op.precedence < min_precedence {
                break;
            }
            
            let op_type = op.op_type;
            let precedence = op.precedence;
            let is_left_associative = op.is_left_associative;
            self.advance(); // consume operator
            
            let next_min_precedence = if is_left_associative {
                precedence + 1
            } else {
                precedence
            };
            
            let right = self.parse_arithmetic_precedence(next_min_precedence)?;
            
            left = AstNode::BinaryOp {
                op: op_type,
                left: Box::new(left),
                right: Box::new(right)
            };
        }
        
        Ok(left)
    }
    
    /// Parse primary expressions: variables, numbers, strings, functions, parentheses
    fn parse_primary_expression(&mut self) -> Result<AstNode, String> {
        match self.advance() {
            Some(ParseToken::UnaryMinus) => {
                // Parse unary minus operation
                let operand = self.parse_primary_expression()?;
                Ok(AstNode::UnaryMinus { operand: Box::new(operand) })
            }
            Some(ParseToken::Variable) => Ok(AstNode::Variable),
            Some(ParseToken::Number(n)) => Ok(AstNode::Number(*n)),
            Some(ParseToken::String(s)) => {
                let has_interpolation = s.contains('$');
                Ok(AstNode::String { 
                    value: s.clone(), 
                    has_interpolation 
                })
            }
            Some(ParseToken::Undefined) => Ok(AstNode::Undefined),
            Some(ParseToken::Function(func_type)) => {
                let func_type = *func_type;
                if !matches!(self.current_token(), Some(ParseToken::LeftParen)) {
                    return Err("Expected '(' after function name".to_string());
                }
                self.advance(); // consume '('
                
                let arg = self.parse_ternary_expression()?;
                
                if !matches!(self.current_token(), Some(ParseToken::RightParen)) {
                    return Err("Expected ')' after function argument".to_string());
                }
                self.advance(); // consume ')'
                
                Ok(AstNode::FunctionCall {
                    func: func_type,
                    arg: Box::new(arg)
                })
            }
            Some(ParseToken::ExifToolFunction(func_name)) => {
                let func_name = func_name.clone();
                if !matches!(self.current_token(), Some(ParseToken::LeftParen)) {
                    return Err("Expected '(' after ExifTool function name".to_string());
                }
                self.advance(); // consume '('
                
                let arg = self.parse_ternary_expression()?;
                
                if !matches!(self.current_token(), Some(ParseToken::RightParen)) {
                    return Err("Expected ')' after ExifTool function argument".to_string());
                }
                self.advance(); // consume ')'
                
                Ok(AstNode::ExifToolFunction {
                    name: func_name,
                    arg: Box::new(arg)
                })
            }
            Some(ParseToken::Sprintf) => {
                if !matches!(self.current_token(), Some(ParseToken::LeftParen)) {
                    return Err("Expected '(' after sprintf".to_string());
                }
                self.advance(); // consume '('
                
                // Parse format string (must be first argument)
                let format_arg = self.parse_ternary_expression()?;
                let format_string = match format_arg {
                    AstNode::String { value, .. } => value,
                    _ => return Err("sprintf first argument must be a string literal".to_string()),
                };
                
                // Parse additional arguments separated by commas
                let mut args = Vec::new();
                while matches!(self.current_token(), Some(ParseToken::Comma)) {
                    self.advance(); // consume ','
                    let arg = self.parse_ternary_expression()?;
                    args.push(Box::new(arg));
                }
                
                if !matches!(self.current_token(), Some(ParseToken::RightParen)) {
                    return Err("Expected ')' after sprintf arguments".to_string());
                }
                self.advance(); // consume ')'
                
                Ok(AstNode::Sprintf {
                    format_string,
                    args
                })
            }
            Some(ParseToken::RegexSubstitution { pattern, replacement, flags }) => {
                let pattern = pattern.clone();
                let replacement = replacement.clone();
                let flags = flags.clone();
                
                // For now, regex operates on the variable - in full implementation
                // this would need to handle complex expressions as targets
                Ok(AstNode::RegexSubstitution {
                    target: Box::new(AstNode::Variable),
                    pattern,
                    replacement,
                    flags
                })
            }
            Some(ParseToken::Transliteration { search_list, replace_list, flags }) => {
                let search_list = search_list.clone();
                let replace_list = replace_list.clone();
                let flags = flags.clone();
                
                // For now, transliteration operates on the variable
                Ok(AstNode::Transliteration {
                    target: Box::new(AstNode::Variable),
                    search_list,
                    replace_list,
                    flags
                })
            }
            Some(ParseToken::LeftParen) => {
                let expr = self.parse_ternary_expression()?;
                if !matches!(self.current_token(), Some(ParseToken::RightParen)) {
                    return Err("Expected ')' after expression".to_string());
                }
                self.advance(); // consume ')'
                Ok(expr)
            }
            Some(token) => Err(format!("Unexpected token: {:?}", token)),
            None => Err("Unexpected end of input".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expression_compiler::tokenizer::tokenize;

    #[test]
    fn test_simple_ternary() {
        let tokens = tokenize("$val >= 0 ? $val : undef").unwrap();
        let ast = parse_expression(tokens).unwrap();
        
        match ast {
            AstNode::TernaryOp { condition, true_expr, false_expr } => {
                // Check condition: $val >= 0
                match *condition {
                    AstNode::ComparisonOp { op, .. } => {
                        assert_eq!(op, CompType::GreaterEq);
                    }
                    _ => panic!("Expected comparison in condition")
                }
                
                // Check true expression: $val
                assert!(matches!(*true_expr, AstNode::Variable));
                
                // Check false expression: undef
                assert!(matches!(*false_expr, AstNode::Undefined));
            }
            _ => panic!("Expected ternary operation")
        }
    }

    #[test]
    fn test_string_ternary() {
        let tokens = tokenize("$val > 655.345 ? \"inf\" : \"$val m\"").unwrap();
        let ast = parse_expression(tokens).unwrap();
        
        match ast {
            AstNode::TernaryOp { condition, true_expr, false_expr } => {
                // Check condition: $val > 655.345
                match *condition {
                    AstNode::ComparisonOp { op, left, right } => {
                        assert_eq!(op, CompType::Greater);
                        assert!(matches!(*left, AstNode::Variable));
                        assert!(matches!(*right, AstNode::Number(n) if (n - 655.345).abs() < f64::EPSILON));
                    }
                    _ => panic!("Expected comparison in condition")
                }
                
                // Check true expression: "inf"
                match *true_expr {
                    AstNode::String { value, has_interpolation } => {
                        assert_eq!(value, "inf");
                        assert!(!has_interpolation);
                    }
                    _ => panic!("Expected string literal")
                }
                
                // Check false expression: "$val m"
                match *false_expr {
                    AstNode::String { value, has_interpolation } => {
                        assert_eq!(value, "$val m");
                        assert!(has_interpolation);
                    }
                    _ => panic!("Expected string literal with interpolation")
                }
            }
            _ => panic!("Expected ternary operation")
        }
    }

    #[test]
    fn test_comparison_only() {
        let tokens = tokenize("$val >= 0").unwrap();
        let ast = parse_expression(tokens).unwrap();
        
        match ast {
            AstNode::ComparisonOp { op, left, right } => {
                assert_eq!(op, CompType::GreaterEq);
                assert!(matches!(*left, AstNode::Variable));
                assert!(matches!(*right, AstNode::Number(0.0)));
            }
            _ => panic!("Expected comparison operation")
        }
    }

    #[test]
    fn test_simple_arithmetic() {
        // Simple arithmetic should now use AST parsing directly
        let tokens = tokenize("$val / 8").unwrap();
        let ast = parse_expression(tokens).unwrap();
        
        match ast {
            AstNode::BinaryOp { op, left, right } => {
                assert_eq!(op, OpType::Divide);
                assert!(matches!(*left, AstNode::Variable));
                assert!(matches!(*right, AstNode::Number(8.0)));
            }
            _ => panic!("Expected binary operation")
        }
    }

    #[test]
    fn test_function_with_ternary() {
        let tokens = tokenize("int($val >= 0 ? $val : 0)").unwrap();
        let ast = parse_expression(tokens).unwrap();
        
        match ast {
            AstNode::FunctionCall { func, arg } => {
                assert_eq!(func, FuncType::Int);
                // The argument should be a ternary expression
                assert!(matches!(arg.as_ref(), AstNode::TernaryOp { .. }));
            }
            _ => panic!("Expected function call")
        }
    }
    
    #[test]
    fn test_precedence_in_ternary() {
        // Test that arithmetic precedence works within ternary expressions
        let tokens = tokenize("$val + 1 > 0 ? $val * 2 : 0").unwrap();
        let ast = parse_expression(tokens).unwrap();
        
        match ast {
            AstNode::TernaryOp { condition, true_expr, .. } => {
                // Condition should be: ($val + 1) > 0
                match condition.as_ref() {
                    AstNode::ComparisonOp { left, .. } => {
                        assert!(matches!(left.as_ref(), AstNode::BinaryOp { op: OpType::Add, .. }));
                    }
                    _ => panic!("Expected comparison with binary operation")
                }
                
                // True expression should be: $val * 2
                assert!(matches!(true_expr.as_ref(), AstNode::BinaryOp { op: OpType::Multiply, .. }));
            }
            _ => panic!("Expected ternary operation")
        }
    }

}