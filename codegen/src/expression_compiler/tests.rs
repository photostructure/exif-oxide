//! Comprehensive integration tests for the expression compiler
//!
//! These tests cover the full compilation pipeline from expression strings
//! to generated Rust code, including edge cases and error conditions.

#[cfg(test)]
mod integration_tests {
    use super::super::*;
    
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
    fn test_int_function() {
        // Test int() function with simple expression
        let expr = CompiledExpression::compile("int($val)").unwrap();
        assert_eq!(expr.rpn_tokens, vec![
            RpnToken::Variable,
            RpnToken::Function(FuncType::Int)
        ]);
        
        // Test int() with arithmetic: int($val * 1000 / 25.4 + 0.5)
        let expr = CompiledExpression::compile("int($val * 1000 / 25.4 + 0.5)").unwrap();
        assert_eq!(expr.rpn_tokens, vec![
            RpnToken::Variable,
            RpnToken::Number(1000.0),
            RpnToken::Operator(OpType::Multiply),
            RpnToken::Number(25.4),
            RpnToken::Operator(OpType::Divide),
            RpnToken::Number(0.5),
            RpnToken::Operator(OpType::Add),
            RpnToken::Function(FuncType::Int)
        ]);
        
        // Test code generation for int()
        let expr = CompiledExpression::compile("int($val)").unwrap();
        let code = expr.generate_rust_code();
        assert!(code.contains("val.trunc()"));
    }
    
    #[test]
    fn test_exp_function() {
        // Test exp() function: exp($val/32*log(2))*100
        let expr = CompiledExpression::compile("exp($val/32*log(2))*100").unwrap();
        assert_eq!(expr.rpn_tokens, vec![
            RpnToken::Variable,
            RpnToken::Number(32.0),
            RpnToken::Operator(OpType::Divide),
            RpnToken::Number(2.0),
            RpnToken::Function(FuncType::Log),
            RpnToken::Operator(OpType::Multiply),
            RpnToken::Function(FuncType::Exp),
            RpnToken::Number(100.0),
            RpnToken::Operator(OpType::Multiply)
        ]);
        
        // Test code generation for exp()
        let expr = CompiledExpression::compile("exp($val)").unwrap();
        let code = expr.generate_rust_code();
        assert!(code.contains("val.exp()"));
    }
    
    #[test]
    fn test_log_function() {
        // Test log() function: 32*log($val/100)/log(2)
        let expr = CompiledExpression::compile("32*log($val/100)/log(2)").unwrap();
        assert_eq!(expr.rpn_tokens, vec![
            RpnToken::Number(32.0),
            RpnToken::Variable,
            RpnToken::Number(100.0),
            RpnToken::Operator(OpType::Divide),
            RpnToken::Function(FuncType::Log),
            RpnToken::Operator(OpType::Multiply),
            RpnToken::Number(2.0),
            RpnToken::Function(FuncType::Log),
            RpnToken::Operator(OpType::Divide)
        ]);
        
        // Test code generation for log()
        let expr = CompiledExpression::compile("log($val)").unwrap();
        let code = expr.generate_rust_code();
        assert!(code.contains("val.ln()"));
    }
    
    #[test]
    fn test_function_combinations() {
        // Test real-world example: int($val * 1000 / 25.4 + 0.5)
        let expr = CompiledExpression::compile("int($val * 1000 / 25.4 + 0.5)").unwrap();
        let code = expr.generate_rust_code();
        assert!(code.contains("stack.push(val)"));
        assert!(code.contains("stack.push(1000"));
        assert!(code.contains("stack.push(25.4)"));
        assert!(code.contains("stack.push(0.5)"));
        assert!(code.contains("a * b"));
        assert!(code.contains("a / b"));
        assert!(code.contains("a + b"));
        assert!(code.contains("b.trunc()"));
    }

    #[test]
    fn test_is_compilable() {
        assert!(CompiledExpression::is_compilable("$val / 8"));
        assert!(CompiledExpression::is_compilable("($val - 104) / 8"));
        assert!(CompiledExpression::is_compilable("$val * 25.4 / 1000"));
        
        // Functions are now supported
        assert!(CompiledExpression::is_compilable("exp($val / 32 * log(2))"));
        assert!(CompiledExpression::is_compilable("int($val)"));
        assert!(CompiledExpression::is_compilable("log($val)"));
        
        // These should still not be compilable
        assert!(!CompiledExpression::is_compilable("$val ? 10 / $val : 0"));
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
        
        // Should be compilable (functions)
        assert!(CompiledExpression::is_compilable("int($val)"));
        assert!(CompiledExpression::is_compilable("exp($val)"));
        assert!(CompiledExpression::is_compilable("log($val)"));
        assert!(CompiledExpression::is_compilable("int($val * 1000 / 25.4 + 0.5)"));
        assert!(CompiledExpression::is_compilable("32*log($val/100)/log(2)"));
        assert!(CompiledExpression::is_compilable("exp($val/32*log(2))*100"));
        
        // Should NOT be compilable (complex expressions)
        assert!(!CompiledExpression::is_compilable("$val ? 10 / $val : 0"));
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