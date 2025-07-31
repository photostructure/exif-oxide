//! Rust code generation for compiled expressions
//!
//! This module generates Rust code from RPN token sequences,
//! optimizing simple cases and using stack-based evaluation for complex ones.

use super::types::*;

impl CompiledExpression {
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
        // Simple function call like int($val)
        if self.rpn_tokens.len() == 2 {
            if let [RpnToken::Variable, RpnToken::Function(func)] = &self.rpn_tokens[..] {
                let rust_op = match func {
                    FuncType::Int => "val.trunc()",
                    FuncType::Exp => "val.exp()",
                    FuncType::Log => "val.ln()",
                };
                
                return Some(format!(
                    "match value.as_f64() {{\n        Some(val) => Ok(TagValue::F64({})),\n        None => Ok(value.clone()),\n    }}",
                    rust_op
                ));
            }
        }
        
        // Simple arithmetic like $val / 8
        if self.rpn_tokens.len() == 3 {
            if let [RpnToken::Variable, RpnToken::Number(n), RpnToken::Operator(op)] = &self.rpn_tokens[..] {
                // Format number as floating-point literal to ensure proper f64 arithmetic
                let n_str = format_number(*n);
                
                let rust_op = match op {
                    OpType::Add => format!("val + {}", n_str),
                    OpType::Subtract => format!("val - {}", n_str),
                    OpType::Multiply => format!("val * {}", n_str),
                    OpType::Divide => format!("val / {}", n_str),
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
                    let n_str = format_number(*n);
                    code.push_str(&format!("            stack.push({});\n", n_str));
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
                RpnToken::Function(func) => {
                    code.push_str("            let b = stack.pop().unwrap();\n");
                    let operation = match func {
                        FuncType::Int => "b.trunc()",
                        FuncType::Exp => "b.exp()",
                        FuncType::Log => "b.ln()",
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
}

/// Format number as floating-point literal to ensure proper f64 arithmetic
fn format_number(n: Number) -> String {
    if n.fract() == 0.0 {
        format!("{:.1}", n) // Add .0 to integers like 8 -> 8.0
    } else {
        n.to_string() // Keep decimals as-is like 25.4 -> 25.4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_function_codegen() {
        let expr = CompiledExpression {
            original_expr: "int($val)".to_string(),
            rpn_tokens: vec![
                RpnToken::Variable,
                RpnToken::Function(FuncType::Int)
            ]
        };
        
        let code = expr.generate_rust_code();
        assert!(code.contains("val.trunc()"));
        assert!(code.contains("match value.as_f64()"));
        assert!(!code.contains("stack")); // Should use simple generation
    }

    #[test]
    fn test_simple_arithmetic_codegen() {
        let expr = CompiledExpression {
            original_expr: "$val / 8".to_string(),
            rpn_tokens: vec![
                RpnToken::Variable,
                RpnToken::Number(8.0),
                RpnToken::Operator(OpType::Divide)
            ]
        };
        
        let code = expr.generate_rust_code();
        assert!(code.contains("val / 8.0"));
        assert!(code.contains("match value.as_f64()"));
        assert!(!code.contains("stack")); // Should use simple generation
    }

    #[test]
    fn test_complex_expression_codegen() {
        let expr = CompiledExpression {
            original_expr: "($val - 104) / 8".to_string(),
            rpn_tokens: vec![
                RpnToken::Variable,
                RpnToken::Number(104.0),
                RpnToken::Operator(OpType::Subtract),
                RpnToken::Number(8.0),
                RpnToken::Operator(OpType::Divide)
            ]
        };
        
        let code = expr.generate_rust_code();
        assert!(code.contains("let mut stack = Vec::new()"));
        assert!(code.contains("stack.push(val)"));
        assert!(code.contains("stack.push(104.0)"));
        assert!(code.contains("a - b"));
        assert!(code.contains("a / b"));
    }

    #[test]
    fn test_function_in_complex_expression() {
        let expr = CompiledExpression {
            original_expr: "int($val * 2)".to_string(),
            rpn_tokens: vec![
                RpnToken::Variable,
                RpnToken::Number(2.0),
                RpnToken::Operator(OpType::Multiply),
                RpnToken::Function(FuncType::Int)
            ]
        };
        
        let code = expr.generate_rust_code();
        assert!(code.contains("stack"));
        assert!(code.contains("b.trunc()"));
        assert!(code.contains("a * b"));
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(8.0), "8.0");
        assert_eq!(format_number(25.4), "25.4");
        assert_eq!(format_number(0.0), "0.0");
        assert_eq!(format_number(1000.0), "1000.0");
    }
}