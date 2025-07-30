//! Expression compiler for ValueConv arithmetic expressions
//!
//! This module provides compile-time parsing and code generation for simple
//! arithmetic expressions found in ExifTool ValueConv patterns. Uses the
//! Shunting Yard algorithm to convert infix expressions to Rust code.

use std::fmt;

type Number = f64;

/// A compiled arithmetic expression that can generate Rust code
#[derive(Debug, Clone)]
pub struct CompiledExpression {
    pub original_expr: String,
    pub rpn_tokens: Vec<RpnToken>,
}

/// Token in Reverse Polish Notation
#[derive(Debug, Clone, PartialEq)]
pub enum RpnToken {
    Variable,           // Represents $val
    Number(Number),     // Numeric constant
    Operator(OpType),   // Arithmetic operator
}

/// Arithmetic operator types
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OpType {
    Add,
    Subtract, 
    Multiply,
    Divide,
}

/// Internal token used during parsing
#[derive(Debug, Clone, PartialEq)]
enum ParseToken {
    Variable,
    Number(Number),
    Operator(Operator),
    LeftParen,
    RightParen,
}

/// Operator with precedence and associativity
#[derive(Debug, Copy, Clone, PartialEq)]
struct Operator {
    op_type: OpType,
    precedence: u8,
    is_left_associative: bool,
}

impl Operator {
    fn new(op_type: OpType, precedence: u8, is_left_associative: bool) -> Self {
        Self { op_type, precedence, is_left_associative }
    }
}

/// Helper trait for stack operations
trait Stack<T> {
    fn top(&self) -> Option<&T>;
}

impl<T> Stack<T> for Vec<T> {
    fn top(&self) -> Option<&T> {
        self.last()
    }
}

impl fmt::Display for OpType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpType::Add => write!(f, "+"),
            OpType::Subtract => write!(f, "-"),
            OpType::Multiply => write!(f, "*"),
            OpType::Divide => write!(f, "/"),
        }
    }
}

impl CompiledExpression {
    /// Parse an ExifTool arithmetic expression into a compiled form
    /// 
    /// Supports: $val, numbers, +, -, *, /, parentheses
    /// Examples: "$val / 8", "($val - 104) / 8", "$val * 25.4 / 1000"
    pub fn compile(expr: &str) -> Result<Self, String> {
        let tokens = tokenize(expr)?;
        let rpn_tokens = shunting_yard(tokens)?;
        
        Ok(CompiledExpression {
            original_expr: expr.to_string(),
            rpn_tokens,
        })
    }
    
    /// Generate Rust code that evaluates this expression
    /// 
    /// Returns code like: `match value.as_f64() { Some(val) => Ok(TagValue::F64(val / 8.0)), None => Ok(value.clone()) }`
    pub fn generate_rust_code(&self) -> String {
        if self.rpn_tokens.is_empty() {
            return "Ok(value.clone())".to_string();
        }
        
        // For simple expressions, generate direct arithmetic
        if let Some(simple_code) = self.try_generate_simple_arithmetic() {
            return simple_code;
        }
        
        // For complex expressions, generate stack-based evaluation
        self.generate_stack_evaluation()
    }
    
    /// Try to generate direct arithmetic for simple cases like "$val / 8"
    fn try_generate_simple_arithmetic(&self) -> Option<String> {
        if self.rpn_tokens.len() == 3 {
            if let [RpnToken::Variable, RpnToken::Number(n), RpnToken::Operator(op)] = &self.rpn_tokens[..] {
                let rust_op = match op {
                    OpType::Add => format!("val + {}", n),
                    OpType::Subtract => format!("val - {}", n),
                    OpType::Multiply => format!("val * {}", n),
                    OpType::Divide => format!("val / {}", n),
                };
                
                return Some(format!(
                    "match value.as_f64() {{\n        Some(val) => Ok(TagValue::F64({})),\n        None => Ok(value.clone()),\n    }}",
                    rust_op
                ));
            }
        }
        None
    }
    
    /// Generate stack-based evaluation for complex expressions
    fn generate_stack_evaluation(&self) -> String {
        let mut code = String::new();
        code.push_str("match value.as_f64() {\n");
        code.push_str("        Some(val) => {\n");
        code.push_str("            let mut stack = Vec::new();\n");
        
        for token in &self.rpn_tokens {
            match token {
                RpnToken::Variable => {
                    code.push_str("            stack.push(val);\n");
                }
                RpnToken::Number(n) => {
                    code.push_str(&format!("            stack.push({});\n", n));
                }
                RpnToken::Operator(op) => {
                    code.push_str("            let b = stack.pop().unwrap();\n");
                    code.push_str("            let a = stack.pop().unwrap();\n");
                    let operation = match op {
                        OpType::Add => "a + b",
                        OpType::Subtract => "a - b", 
                        OpType::Multiply => "a * b",
                        OpType::Divide => "a / b",
                    };
                    code.push_str(&format!("            stack.push({});\n", operation));
                }
            }
        }
        
        code.push_str("            Ok(TagValue::F64(stack[0]))\n");
        code.push_str("        },\n");
        code.push_str("        None => Ok(value.clone()),\n");
        code.push_str("    }");
        
        code
    }
    
    /// Check if this expression can be compiled (simple arithmetic only)
    pub fn is_compilable(expr: &str) -> bool {
        // Quick checks for obviously non-compilable expressions
        if expr.contains('?') || expr.contains("exp") || expr.contains("log") || 
           expr.contains("**") || expr.contains("abs") || expr.contains("IsFloat") ||
           expr.contains("=~") || expr.contains("&") || expr.contains("|") ||
           expr.contains(">>") || expr.contains("<<") {
            return false;
        }
        
        // Try to compile - if it works, it's compilable
        Self::compile(expr).is_ok()
    }
}

/// Tokenize an expression string into parse tokens
fn tokenize(expr: &str) -> Result<Vec<ParseToken>, String> {
    let mut tokens = Vec::new();
    let mut chars = expr.chars().peekable();
    
    while let Some(ch) = chars.next() {
        match ch {
            ' ' | '\t' => continue, // Skip whitespace
            
            '$' => {
                // Expect "val" after $
                let val_chars: String = chars.by_ref().take(3).collect();
                if val_chars == "val" {
                    tokens.push(ParseToken::Variable);
                } else {
                    return Err(format!("Expected 'val' after '$', found '{}'", val_chars));
                }
            }
            
            '0'..='9' => {
                // Parse number (including decimals like 25.4)
                let mut number_str = String::new();
                number_str.push(ch);
                
                while let Some(&next_ch) = chars.peek() {
                    if next_ch.is_ascii_digit() || next_ch == '.' {
                        number_str.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                
                let number: f64 = number_str.parse()
                    .map_err(|_| format!("Invalid number: {}", number_str))?;
                tokens.push(ParseToken::Number(number));
            }
            
            '+' => tokens.push(ParseToken::Operator(Operator::new(OpType::Add, 1, true))),
            '-' => tokens.push(ParseToken::Operator(Operator::new(OpType::Subtract, 1, true))),
            '*' => tokens.push(ParseToken::Operator(Operator::new(OpType::Multiply, 2, true))),
            '/' => tokens.push(ParseToken::Operator(Operator::new(OpType::Divide, 2, true))),
            '(' => tokens.push(ParseToken::LeftParen),
            ')' => tokens.push(ParseToken::RightParen),
            
            _ => return Err(format!("Unexpected character: '{}'", ch)),
        }
    }
    
    Ok(tokens)
}

/// Convert infix tokens to RPN using Shunting Yard algorithm
fn shunting_yard(tokens: Vec<ParseToken>) -> Result<Vec<RpnToken>, String> {
    if tokens.is_empty() {
        return Err("Empty expression".to_string());
    }
    
    let mut output = Vec::new();
    let mut operators = Vec::new();
    
    for token in tokens {
        match token {
            ParseToken::Variable => output.push(RpnToken::Variable),
            ParseToken::Number(n) => output.push(RpnToken::Number(n)),
            
            ParseToken::LeftParen => operators.push(token),
            
            ParseToken::RightParen => {
                if !tilt_until(&mut operators, &mut output, ParseToken::LeftParen) {
                    return Err("Mismatched ')'".to_string());
                }
            }
            
            ParseToken::Operator(op) => {
                while let Some(top) = operators.top() {
                    match top {
                        ParseToken::LeftParen => break,
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
    if tilt_until(&mut operators, &mut output, ParseToken::LeftParen) {
        return Err("Mismatched '('".to_string());
    }
    
    // Validate that we have a complete expression
    // Simple check: must have at least one operand and proper operator count
    let operands = output.iter().filter(|t| matches!(t, RpnToken::Variable | RpnToken::Number(_))).count();
    let operators = output.iter().filter(|t| matches!(t, RpnToken::Operator(_))).count();
    
    if operands == 0 {
        return Err("Expression must contain at least one operand".to_string());
    }
    
    if operands != operators + 1 {
        return Err("Invalid expression: operator/operand mismatch".to_string());
    }
    
    Ok(output)
}

/// Helper function to pop operators until a stop token is found
fn tilt_until(operators: &mut Vec<ParseToken>, output: &mut Vec<RpnToken>, stop: ParseToken) -> bool {
    while let Some(token) = operators.pop() {
        if token == stop {
            return true;
        }
        if let ParseToken::Operator(op) = token {
            output.push(RpnToken::Operator(op.op_type));
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_division() {
        let expr = CompiledExpression::compile("$val / 8").unwrap();
        assert_eq!(expr.rpn_tokens, vec![
            RpnToken::Variable,
            RpnToken::Number(8.0),
            RpnToken::Operator(OpType::Divide)
        ]);
        
        let code = expr.generate_rust_code();
        assert!(code.contains("val / 8"));
    }
    
    #[test]
    fn test_complex_expression() {
        let expr = CompiledExpression::compile("($val - 104) / 8").unwrap();
        assert_eq!(expr.rpn_tokens, vec![
            RpnToken::Variable,
            RpnToken::Number(104.0),
            RpnToken::Operator(OpType::Subtract),
            RpnToken::Number(8.0),
            RpnToken::Operator(OpType::Divide)
        ]);
    }
    
    #[test]
    fn test_decimal_numbers() {
        let expr = CompiledExpression::compile("$val * 25.4").unwrap();
        assert_eq!(expr.rpn_tokens, vec![
            RpnToken::Variable,
            RpnToken::Number(25.4),
            RpnToken::Operator(OpType::Multiply)
        ]);
    }
    
    #[test]
    fn test_is_compilable() {
        assert!(CompiledExpression::is_compilable("$val / 8"));
        assert!(CompiledExpression::is_compilable("($val - 104) / 8"));
        assert!(CompiledExpression::is_compilable("$val * 25.4 / 1000"));
        
        assert!(!CompiledExpression::is_compilable("$val ? 10 / $val : 0"));
        assert!(!CompiledExpression::is_compilable("exp($val / 32 * log(2))"));
        assert!(!CompiledExpression::is_compilable("2 ** (-$val/3)"));
    }
    
    #[test]
    fn test_invalid_expressions() {
        assert!(CompiledExpression::compile("$val +").is_err());
        assert!(CompiledExpression::compile("($val").is_err());
        assert!(CompiledExpression::compile("$val )").is_err());
        assert!(CompiledExpression::compile("$foo / 8").is_err());
    }
    
    #[test]
    fn test_operator_precedence() {
        // Test that * has higher precedence than +
        let expr = CompiledExpression::compile("$val + 2 * 3").unwrap();
        // Should be: $val 2 3 * +  (not $val 2 + 3 *)
        assert_eq!(expr.rpn_tokens, vec![
            RpnToken::Variable,
            RpnToken::Number(2.0),
            RpnToken::Number(3.0),
            RpnToken::Operator(OpType::Multiply),
            RpnToken::Operator(OpType::Add)
        ]);
        
        // Test that / has higher precedence than -
        let expr = CompiledExpression::compile("$val - 8 / 2").unwrap();
        // Should be: $val 8 2 / -
        assert_eq!(expr.rpn_tokens, vec![
            RpnToken::Variable,
            RpnToken::Number(8.0),
            RpnToken::Number(2.0),
            RpnToken::Operator(OpType::Divide),
            RpnToken::Operator(OpType::Subtract)
        ]);
    }
    
    #[test]
    fn test_left_associativity() {
        // Test that operators are left associative: $val - 5 + 3 should be (($val - 5) + 3)
        let expr = CompiledExpression::compile("$val - 5 + 3").unwrap();
        // Should be: $val 5 - 3 +
        assert_eq!(expr.rpn_tokens, vec![
            RpnToken::Variable,
            RpnToken::Number(5.0),
            RpnToken::Operator(OpType::Subtract),
            RpnToken::Number(3.0),
            RpnToken::Operator(OpType::Add)
        ]);
        
        // Test division associativity: $val / 4 / 2 should be (($val / 4) / 2)
        let expr = CompiledExpression::compile("$val / 4 / 2").unwrap();
        // Should be: $val 4 / 2 /
        assert_eq!(expr.rpn_tokens, vec![
            RpnToken::Variable,
            RpnToken::Number(4.0),
            RpnToken::Operator(OpType::Divide),
            RpnToken::Number(2.0),
            RpnToken::Operator(OpType::Divide)
        ]);
    }
    
    #[test]
    fn test_parentheses_override_precedence() {
        // Test that parentheses override precedence: ($val + 2) * 3
        let expr = CompiledExpression::compile("($val + 2) * 3").unwrap();
        // Should be: $val 2 + 3 *
        assert_eq!(expr.rpn_tokens, vec![
            RpnToken::Variable,
            RpnToken::Number(2.0),
            RpnToken::Operator(OpType::Add),
            RpnToken::Number(3.0),
            RpnToken::Operator(OpType::Multiply)
        ]);
        
        // Test nested parentheses: $val / (8 - (2 + 1))
        let expr = CompiledExpression::compile("$val / (8 - (2 + 1))").unwrap();
        // Should be: $val 8 2 1 + - /
        assert_eq!(expr.rpn_tokens, vec![
            RpnToken::Variable,
            RpnToken::Number(8.0),
            RpnToken::Number(2.0),
            RpnToken::Number(1.0),
            RpnToken::Operator(OpType::Add),
            RpnToken::Operator(OpType::Subtract),
            RpnToken::Operator(OpType::Divide)
        ]);
    }
    
    #[test]
    fn test_real_world_expressions() {
        // Test actual expressions from the registry
        
        // Canon millimeter conversion: $val * 25.4 / 1000
        let expr = CompiledExpression::compile("$val * 25.4 / 1000").unwrap();
        assert_eq!(expr.rpn_tokens[0], RpnToken::Variable);
        assert_eq!(expr.rpn_tokens[1], RpnToken::Number(25.4));
        assert_eq!(expr.rpn_tokens[2], RpnToken::Operator(OpType::Multiply));
        assert_eq!(expr.rpn_tokens[3], RpnToken::Number(1000.0));
        assert_eq!(expr.rpn_tokens[4], RpnToken::Operator(OpType::Divide));
        
        // Canon offset plus division: $val / 32 + 5
        let expr = CompiledExpression::compile("$val / 32 + 5").unwrap();
        assert_eq!(expr.rpn_tokens, vec![
            RpnToken::Variable,
            RpnToken::Number(32.0),
            RpnToken::Operator(OpType::Divide),
            RpnToken::Number(5.0),
            RpnToken::Operator(OpType::Add)
        ]);
    }
    
    #[test]
    fn test_code_generation_simple_cases() {
        // Test simple division generates direct arithmetic
        let expr = CompiledExpression::compile("$val / 8").unwrap();
        let code = expr.generate_rust_code();
        
        assert!(code.contains("match value.as_f64()"));
        assert!(code.contains("Some(val) => Ok(TagValue::F64(val / 8"));
        assert!(code.contains("None => Ok(value.clone())"));
        assert!(!code.contains("stack")); // Should not use stack for simple case
        
        // Test simple addition
        let expr = CompiledExpression::compile("$val + 3").unwrap();
        let code = expr.generate_rust_code();
        assert!(code.contains("val + 3"));
        
        // Test simple multiplication
        let expr = CompiledExpression::compile("$val * 100").unwrap();
        let code = expr.generate_rust_code();
        assert!(code.contains("val * 100"));
        
        // Test simple subtraction
        let expr = CompiledExpression::compile("$val - 5").unwrap();
        let code = expr.generate_rust_code();
        assert!(code.contains("val - 5"));
    }
    
    #[test]
    fn test_code_generation_complex_cases() {
        // Test complex expressions use stack-based evaluation
        let expr = CompiledExpression::compile("($val - 104) / 8").unwrap();
        let code = expr.generate_rust_code();
        
        assert!(code.contains("let mut stack = Vec::new()"));
        assert!(code.contains("stack.push(val)"));
        assert!(code.contains("stack.push(104"));
        assert!(code.contains("stack.push(8"));
        assert!(code.contains("let b = stack.pop().unwrap()"));
        assert!(code.contains("let a = stack.pop().unwrap()"));
        assert!(code.contains("a - b"));
        assert!(code.contains("a / b"));
        
        // Test multi-operation expression
        let expr = CompiledExpression::compile("$val * 25.4 / 1000").unwrap();
        let code = expr.generate_rust_code();
        assert!(code.contains("stack"));
        assert!(code.contains("a * b"));
        assert!(code.contains("a / b"));
    }
    
    #[test]
    fn test_whitespace_handling() {
        // Test that whitespace doesn't affect parsing
        let expr1 = CompiledExpression::compile("$val/8").unwrap();
        let expr2 = CompiledExpression::compile("$val / 8").unwrap();
        let expr3 = CompiledExpression::compile("  $val  /  8  ").unwrap();
        
        assert_eq!(expr1.rpn_tokens, expr2.rpn_tokens);
        assert_eq!(expr2.rpn_tokens, expr3.rpn_tokens);
        
        // Test whitespace in complex expressions
        let expr1 = CompiledExpression::compile("($val-104)/8").unwrap();
        let expr2 = CompiledExpression::compile("( $val - 104 ) / 8").unwrap();
        assert_eq!(expr1.rpn_tokens, expr2.rpn_tokens);
    }
    
    #[test]
    fn test_edge_case_numbers() {
        // Test zero
        let expr = CompiledExpression::compile("$val + 0").unwrap();
        assert_eq!(expr.rpn_tokens[1], RpnToken::Number(0.0));
        
        // Test large integer
        let expr = CompiledExpression::compile("$val / 1000").unwrap();
        assert_eq!(expr.rpn_tokens[1], RpnToken::Number(1000.0));
        
        // Test decimal with leading zero
        let expr = CompiledExpression::compile("$val * 0.5").unwrap();
        assert_eq!(expr.rpn_tokens[1], RpnToken::Number(0.5));
        
        // Test decimal without leading zero (not supported by our parser, but test error)
        assert!(CompiledExpression::compile("$val * .5").is_err());
    }
    
    #[test]
    fn test_error_conditions() {
        // Empty expression
        assert!(CompiledExpression::compile("").is_err());
        
        // Only whitespace
        assert!(CompiledExpression::compile("   ").is_err());
        
        // Missing operand
        assert!(CompiledExpression::compile("+ 8").is_err());
        assert!(CompiledExpression::compile("$val +").is_err());
        assert!(CompiledExpression::compile("$val * + 8").is_err());
        
        // Missing operator
        assert!(CompiledExpression::compile("$val 8").is_err());
        assert!(CompiledExpression::compile("$val () 8").is_err());
        
        // Invalid variable name
        assert!(CompiledExpression::compile("$value / 8").is_err());
        assert!(CompiledExpression::compile("$x / 8").is_err());
        
        // Mismatched parentheses
        assert!(CompiledExpression::compile("($val / 8").is_err());
        assert!(CompiledExpression::compile("$val / 8)").is_err());
        assert!(CompiledExpression::compile("(($val / 8)").is_err());
        assert!(CompiledExpression::compile("($val / 8))").is_err());
        
        // Invalid characters
        assert!(CompiledExpression::compile("$val & 8").is_err());
        assert!(CompiledExpression::compile("$val ^ 8").is_err());
        assert!(CompiledExpression::compile("$val % 8").is_err());
        assert!(CompiledExpression::compile("$val @ 8").is_err());
    }
    
    #[test]
    fn test_is_compilable_comprehensive() {
        // Should be compilable (simple arithmetic)
        assert!(CompiledExpression::is_compilable("$val / 8"));
        assert!(CompiledExpression::is_compilable("$val + 3"));
        assert!(CompiledExpression::is_compilable("$val - 5"));
        assert!(CompiledExpression::is_compilable("$val * 100"));
        assert!(CompiledExpression::is_compilable("($val - 104) / 8"));
        assert!(CompiledExpression::is_compilable("$val * 25.4 / 1000"));
        assert!(CompiledExpression::is_compilable("$val / 32 + 5"));
        assert!(CompiledExpression::is_compilable("($val + 2) * 3"));
        
        // Should NOT be compilable (complex expressions)
        assert!(!CompiledExpression::is_compilable("$val ? 10 / $val : 0"));
        assert!(!CompiledExpression::is_compilable("exp($val / 32 * log(2))"));
        assert!(!CompiledExpression::is_compilable("2 ** (-$val/3)"));
        assert!(!CompiledExpression::is_compilable("IsFloat($val) && abs($val) < 100"));
        assert!(!CompiledExpression::is_compilable("$val =~ s/ +$//"));
        assert!(!CompiledExpression::is_compilable("$val & 0xffc0"));
        assert!(!CompiledExpression::is_compilable("$val >> 6"));
        assert!(!CompiledExpression::is_compilable("$val << 8"));
        
        // Invalid expressions should not be compilable
        assert!(!CompiledExpression::is_compilable("$val +"));
        assert!(!CompiledExpression::is_compilable("($val"));
        assert!(!CompiledExpression::is_compilable(""));
    }
}