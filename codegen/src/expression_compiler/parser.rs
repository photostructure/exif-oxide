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
    
    // Check if this expression contains ternary, comparison, sprintf, ExifTool functions, or string concatenation operators
    let has_ternary = tokens.iter().any(|t| matches!(t, ParseToken::Question | ParseToken::Colon));
    let has_comparison = tokens.iter().any(|t| matches!(t, ParseToken::Comparison(_)));
    let has_sprintf = tokens.iter().any(|t| matches!(t, ParseToken::Sprintf));
    let has_exiftool_function = tokens.iter().any(|t| matches!(t, ParseToken::ExifToolFunction(_)));
    let has_concatenation = tokens.iter().any(|t| matches!(t, ParseToken::Operator(op) if op.op_type == OpType::Concatenate));
    
    if has_ternary || has_comparison || has_sprintf || has_exiftool_function || has_concatenation {
        // Use recursive descent parser for complex expressions
        let mut parser = Parser::new(tokens);
        parser.parse_ternary_expression()
    } else {
        // Fall back to RPN for simple arithmetic expressions (compatibility)
        let rpn_tokens = shunting_yard(tokens)?;
        convert_rpn_to_ast(rpn_tokens)
    }
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

/// Temporary function to convert RPN tokens to AST (maintains compatibility)
fn convert_rpn_to_ast(rpn_tokens: Vec<RpnToken>) -> Result<AstNode, String> {
    if rpn_tokens.is_empty() {
        return Err("Empty RPN sequence".to_string());
    }
    
    let mut stack: Vec<AstNode> = Vec::new();
    
    for token in rpn_tokens {
        match token {
            RpnToken::Variable => {
                stack.push(AstNode::Variable);
            }
            RpnToken::Number(n) => {
                stack.push(AstNode::Number(n));
            }
            RpnToken::Operator(op) => {
                if stack.len() < 2 {
                    return Err("Insufficient operands for operator".to_string());
                }
                let right = Box::new(stack.pop().unwrap());
                let left = Box::new(stack.pop().unwrap());
                stack.push(AstNode::BinaryOp { op, left, right });
            }
            RpnToken::Function(func) => {
                if stack.is_empty() {
                    return Err("Insufficient operands for function".to_string());
                }
                let arg = Box::new(stack.pop().unwrap());
                stack.push(AstNode::FunctionCall { func, arg });
            }
        }
    }
    
    if stack.len() != 1 {
        return Err("Invalid expression: multiple root nodes".to_string());
    }
    
    Ok(stack.pop().unwrap())
}

/// Convert infix tokens to RPN using Shunting Yard algorithm
pub fn shunting_yard(tokens: Vec<ParseToken>) -> Result<Vec<RpnToken>, String> {
    if tokens.is_empty() {
        return Err("Empty expression".to_string());
    }
    
    let mut output = Vec::new();
    let mut operators = Vec::new();
    
    for token in tokens {
        match token {
            ParseToken::Variable => output.push(RpnToken::Variable),
            ParseToken::Number(n) => output.push(RpnToken::Number(n)),
            
            // Temporary: ignore new tokens for now to maintain compatibility
            ParseToken::String(_) => return Err("String literals not yet supported in RPN compatibility mode".to_string()),
            ParseToken::Undefined => return Err("Undefined values not yet supported in RPN compatibility mode".to_string()),
            ParseToken::Comparison(_) => return Err("Comparison operators not yet supported in RPN compatibility mode".to_string()),
            ParseToken::Question => return Err("Ternary operators not yet supported in RPN compatibility mode".to_string()),
            ParseToken::Colon => return Err("Ternary operators not yet supported in RPN compatibility mode".to_string()),
            ParseToken::Sprintf => return Err("Sprintf function not yet supported in RPN compatibility mode".to_string()),
            ParseToken::ExifToolFunction(_) => return Err("ExifTool functions not yet supported in RPN compatibility mode".to_string()),
            ParseToken::Comma => return Err("Comma-separated arguments not yet supported in RPN compatibility mode".to_string()),
            
            ParseToken::Function(_func_type) => {
                // Functions are handled like operators but with highest precedence
                // They will be applied after their argument is evaluated
                operators.push(token);
            }
            
            ParseToken::LeftParen => operators.push(token),
            
            ParseToken::RightParen => {
                if !pop_until(&mut operators, &mut output, ParseToken::LeftParen) {
                    return Err("Mismatched ')'".to_string());
                }
                
                // After processing a closing paren, check if there's a function to apply
                if let Some(ParseToken::Function(func_type)) = operators.last() {
                    let func_type = *func_type;
                    operators.pop();
                    output.push(RpnToken::Function(func_type));
                }
            }
            
            ParseToken::Operator(op) => {
                while let Some(top) = operators.top() {
                    match top {
                        ParseToken::LeftParen | ParseToken::Function(_) => break,
                        ParseToken::Operator(top_op) => {
                            let should_pop = top_op.precedence > op.precedence ||
                                (top_op.precedence == op.precedence && op.is_left_associative);
                            
                            if should_pop {
                                if let Some(ParseToken::Operator(popped_op)) = operators.pop() {
                                    output.push(RpnToken::Operator(popped_op.op_type));
                                }
                            } else {
                                break;
                            }
                        }
                        _ => return Err("Invalid token on operator stack".to_string()),
                    }
                }
                operators.push(token);
            }
        }
    }
    
    // Pop remaining operators
    if pop_until(&mut operators, &mut output, ParseToken::LeftParen) {
        return Err("Mismatched '('".to_string());
    }
    
    validate_expression(&output)?;
    
    Ok(output)
}

/// Validate that we have a complete expression
fn validate_expression(output: &[RpnToken]) -> Result<(), String> {
    // Check operand/operator balance: operators consume 2 operands, functions consume 1
    let operands = output.iter().filter(|t| matches!(t, RpnToken::Variable | RpnToken::Number(_))).count();
    let operators = output.iter().filter(|t| matches!(t, RpnToken::Operator(_))).count();
    let functions = output.iter().filter(|t| matches!(t, RpnToken::Function(_))).count();
    
    if operands == 0 {
        return Err("Expression must contain at least one operand".to_string());
    }
    
    // Each operator consumes 2 operands and produces 1, each function consumes 1 and produces 1
    // Final result should be exactly 1 value
    let consumed_operands = operators * 2 + functions;
    let produced_values = operators + functions;
    
    if operands + produced_values != consumed_operands + 1 {
        return Err("Invalid expression: operator/operand mismatch".to_string());
    }
    
    Ok(())
}

/// Helper function to pop operators until a stop token is found
fn pop_until(operators: &mut Vec<ParseToken>, output: &mut Vec<RpnToken>, stop: ParseToken) -> bool {
    while let Some(token) = operators.pop() {
        if token == stop {
            return true;
        }
        match token {
            ParseToken::Operator(op) => output.push(RpnToken::Operator(op.op_type)),
            ParseToken::Function(func_type) => output.push(RpnToken::Function(func_type)),
            _ => {} // Skip other tokens
        }
    }
    false
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
    fn test_arithmetic_fallback() {
        // Simple arithmetic should still use RPN conversion for compatibility
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

    // RPN compatibility tests (simplified from original)
    #[test]
    fn test_rpn_simple_expression() {
        let tokens = tokenize("$val / 8").unwrap();
        let rpn = shunting_yard(tokens).unwrap();
        assert_eq!(rpn, vec![
            RpnToken::Variable,
            RpnToken::Number(8.0),
            RpnToken::Operator(OpType::Divide)
        ]);
    }

    #[test]
    fn test_rpn_function_expression() {
        let tokens = tokenize("int($val)").unwrap();
        let rpn = shunting_yard(tokens).unwrap();
        assert_eq!(rpn, vec![
            RpnToken::Variable,
            RpnToken::Function(FuncType::Int)
        ]);
    }
}