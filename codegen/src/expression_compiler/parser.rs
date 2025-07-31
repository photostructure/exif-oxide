//! Shunting yard algorithm for converting infix expressions to RPN
//!
//! This module implements the shunting yard algorithm to convert
//! tokenized infix expressions into Reverse Polish Notation (RPN).

use super::types::*;

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
    fn test_simple_expression() {
        let tokens = tokenize("$val / 8").unwrap();
        let rpn = shunting_yard(tokens).unwrap();
        assert_eq!(rpn, vec![
            RpnToken::Variable,
            RpnToken::Number(8.0),
            RpnToken::Operator(OpType::Divide)
        ]);
    }

    #[test]
    fn test_function_expression() {
        let tokens = tokenize("int($val)").unwrap();
        let rpn = shunting_yard(tokens).unwrap();
        assert_eq!(rpn, vec![
            RpnToken::Variable,
            RpnToken::Function(FuncType::Int)
        ]);
    }

    #[test]
    fn test_complex_function() {
        let tokens = tokenize("int($val * 2 + 1)").unwrap();
        let rpn = shunting_yard(tokens).unwrap();
        assert_eq!(rpn, vec![
            RpnToken::Variable,
            RpnToken::Number(2.0),
            RpnToken::Operator(OpType::Multiply),
            RpnToken::Number(1.0),
            RpnToken::Operator(OpType::Add),
            RpnToken::Function(FuncType::Int)
        ]);
    }

    #[test]
    fn test_operator_precedence() {
        let tokens = tokenize("$val + 2 * 3").unwrap();
        let rpn = shunting_yard(tokens).unwrap();
        // Should be: $val 2 3 * +  (not $val 2 + 3 *)
        assert_eq!(rpn, vec![
            RpnToken::Variable,
            RpnToken::Number(2.0),
            RpnToken::Number(3.0),
            RpnToken::Operator(OpType::Multiply),
            RpnToken::Operator(OpType::Add)
        ]);
    }

    #[test]
    fn test_parentheses() {
        let tokens = tokenize("($val + 2) * 3").unwrap();
        let rpn = shunting_yard(tokens).unwrap();
        // Should be: $val 2 + 3 *
        assert_eq!(rpn, vec![
            RpnToken::Variable,
            RpnToken::Number(2.0),
            RpnToken::Operator(OpType::Add),
            RpnToken::Number(3.0),
            RpnToken::Operator(OpType::Multiply)
        ]);
    }

    #[test]
    fn test_mismatched_parens() {
        let tokens = tokenize("($val").unwrap();
        let result = shunting_yard(tokens);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Mismatched"));
    }

    #[test]
    fn test_invalid_expression() {
        let tokens = tokenize("$val +").unwrap();
        let result = shunting_yard(tokens);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("operator/operand mismatch"));
    }
}