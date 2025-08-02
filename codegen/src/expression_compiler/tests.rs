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
        // Verify AST structure for simple division
        match expr.ast.as_ref() {
            AstNode::BinaryOp { op, left, right } => {
                assert_eq!(*op, OpType::Divide);
                assert!(matches!(left.as_ref(), AstNode::Variable));
                assert!(matches!(right.as_ref(), AstNode::Number(8.0)));
            }
            _ => panic!("Expected BinaryOp for division")
        }
        
        let code = expr.generate_rust_code();
        assert!(code.contains("val / 8"));
    }
    
    #[test]
    fn test_complex_expression() {
        let expr = CompiledExpression::compile("($val - 104) / 8").unwrap();
        // Verify AST structure for complex expression: ($val - 104) / 8
        match expr.ast.as_ref() {
            AstNode::BinaryOp { op: OpType::Divide, left, right } => {
                // Left side should be ($val - 104)
                match left.as_ref() {
                    AstNode::BinaryOp { op: OpType::Subtract, left: inner_left, right: inner_right } => {
                        assert!(matches!(inner_left.as_ref(), AstNode::Variable));
                        assert!(matches!(inner_right.as_ref(), AstNode::Number(104.0)));
                    }
                    _ => panic!("Expected subtraction on left side")
                }
                // Right side should be 8
                assert!(matches!(right.as_ref(), AstNode::Number(8.0)));
            }
            _ => panic!("Expected division at root")
        }
    }
    
    #[test]
    fn test_decimal_numbers() {
        let expr = CompiledExpression::compile("$val * 25.4").unwrap();
        // Verify AST handles decimal numbers correctly
        match expr.ast.as_ref() {
            AstNode::BinaryOp { op, left, right } => {
                assert_eq!(*op, OpType::Multiply);
                assert!(matches!(left.as_ref(), AstNode::Variable));
                assert!(matches!(right.as_ref(), AstNode::Number(25.4)));
            }
            _ => panic!("Expected BinaryOp for multiplication")
        }
    }
    
    #[test]
    fn test_int_function() {
        // Test int() function with simple expression
        let expr = CompiledExpression::compile("int($val)").unwrap();
        match expr.ast.as_ref() {
            AstNode::FunctionCall { func, arg } => {
                assert_eq!(*func, FuncType::Int);
                assert!(matches!(arg.as_ref(), AstNode::Variable));
            }
            _ => panic!("Expected FunctionCall for int()")
        }
        
        // Test code generation for int()
        let code = expr.generate_rust_code();
        assert!(code.contains("val.trunc()"));
        
        // Test int() with complex arithmetic should use backward compatibility
        let expr = CompiledExpression::compile("int($val * 1000 / 25.4 + 0.5)").unwrap();
        let code = expr.generate_rust_code();
        assert!(code.contains("trunc()"));
    }
    
    #[test]
    fn test_exp_function() {
        // Test simple exp() function
        let expr = CompiledExpression::compile("exp($val)").unwrap();
        match expr.ast.as_ref() {
            AstNode::FunctionCall { func, arg } => {
                assert_eq!(*func, FuncType::Exp);
                assert!(matches!(arg.as_ref(), AstNode::Variable));
            }
            _ => panic!("Expected FunctionCall for exp()")
        }
        
        // Test code generation for exp()
        let code = expr.generate_rust_code();
        assert!(code.contains("val.exp()"));
        
        // Test complex expression compiles
        let expr = CompiledExpression::compile("exp($val/32*log(2))*100").unwrap();
        let code = expr.generate_rust_code();
        assert!(code.contains("exp()"));
        assert!(code.contains("ln()"));
    }
    
    #[test]
    fn test_log_function() {
        // Test simple log() function
        let expr = CompiledExpression::compile("log($val)").unwrap();
        match expr.ast.as_ref() {
            AstNode::FunctionCall { func, arg } => {
                assert_eq!(*func, FuncType::Log);
                assert!(matches!(arg.as_ref(), AstNode::Variable));
            }
            _ => panic!("Expected FunctionCall for log()")
        }
        
        // Test code generation for log()
        let code = expr.generate_rust_code();
        assert!(code.contains("val.ln()"));
        
        // Test complex expression compiles
        let expr = CompiledExpression::compile("32*log($val/100)/log(2)").unwrap();
        let code = expr.generate_rust_code();
        assert!(code.contains("ln()"));
    }
    
    #[test]
    fn test_function_combinations() {
        // Test real-world example: int($val * 1000 / 25.4 + 0.5)
        let expr = CompiledExpression::compile("int($val * 1000 / 25.4 + 0.5)").unwrap();
        let code = expr.generate_rust_code();
        // Should generate working code with int function
        assert!(code.contains("trunc()"));
        assert!(code.contains("match value.as_f64()"));
    }

    #[test]
    fn test_is_compilable() {
        // Basic arithmetic should be compilable
        assert!(CompiledExpression::is_compilable("$val / 8"));
        assert!(CompiledExpression::is_compilable("($val - 104) / 8"));
        assert!(CompiledExpression::is_compilable("$val * 25.4 / 1000"));
        
        // Functions are now supported
        assert!(CompiledExpression::is_compilable("exp($val / 32 * log(2))"));
        assert!(CompiledExpression::is_compilable("int($val)"));
        assert!(CompiledExpression::is_compilable("log($val)"));
        
        // Ternary expressions are now supported!
        assert!(CompiledExpression::is_compilable("$val >= 0 ? $val : undef"));
        assert!(CompiledExpression::is_compilable("$val > 655.345 ? \"inf\" : \"$val m\""));
        assert!(CompiledExpression::is_compilable("$val == 0 ? \"Auto\" : \"Manual\""));
        
        // sprintf patterns are now supported
        assert!(CompiledExpression::is_compilable("sprintf(\"%.1f mm\", $val)"));
        assert!(CompiledExpression::is_compilable("sprintf(\"%.2f\", $val)"));
        assert!(CompiledExpression::is_compilable("sprintf(\"%d\", $val)"));
        
        // String concatenation is now supported
        assert!(CompiledExpression::is_compilable("$val . \" m\""));
        assert!(CompiledExpression::is_compilable("\"Error: \" . $val"));
        assert!(CompiledExpression::is_compilable("\"$val\" . \" mm\""));
        
        // Complex patterns should still not be compilable
        assert!(!CompiledExpression::is_compilable("2 ** (-$val/3)"));
        assert!(!CompiledExpression::is_compilable("abs($val)"));
        assert!(!CompiledExpression::is_compilable("IsFloat($val) && $val < 100"));
        assert!(!CompiledExpression::is_compilable("$val =~ s/ +$//"));
        assert!(!CompiledExpression::is_compilable("$val & 0xffc0"));
        
        // Simple ExifTool function calls are now supported
        assert!(CompiledExpression::is_compilable("Image::ExifTool::Exif::PrintExposureTime($val)"));
        assert!(CompiledExpression::is_compilable("Image::ExifTool::Exif::PrintFNumber($val)"));
        assert!(CompiledExpression::is_compilable("Image::ExifTool::GPS::ToDegrees($val)"));
        
        // Complex ExifTool function calls with multiple arguments are not supported
        assert!(!CompiledExpression::is_compilable("Image::ExifTool::GPS::ToDMS($self, $val, 1, \"N\")"));
    }
    
    #[test]
    fn test_invalid_expressions() {
        assert!(CompiledExpression::compile("$val +").is_err());
        assert!(CompiledExpression::compile("($val").is_err());
        assert!(CompiledExpression::compile("$val )").is_err());
        assert!(CompiledExpression::compile("$foo / 8").is_err());
    }
    
    #[test]
    fn test_arithmetic_expressions_compile() {
        // Test that various arithmetic expressions compile successfully
        let test_cases = vec![
            "$val + 2 * 3",        // precedence
            "$val - 8 / 2",        // precedence
            "$val - 5 + 3",        // left associativity
            "$val / 4 / 2",        // associativity  
            "($val + 2) * 3",      // parentheses
            "$val / (8 - (2 + 1))", // nested parentheses
            "$val * 25.4 / 1000",  // real-world
            "$val / 32 + 5",       // offset
        ];
        
        for expr_str in test_cases {
            let expr = CompiledExpression::compile(expr_str).unwrap();
            let code = expr.generate_rust_code();
            
            // Should generate working code
            assert!(code.contains("match value.as_f64()"));
            assert!(code.contains("Some(val) => Ok("));
        }
    }
    
    #[test]
    fn test_whitespace_and_numbers() {
        // Test whitespace handling
        let expr1 = CompiledExpression::compile("$val/8").unwrap();
        let expr2 = CompiledExpression::compile("$val / 8").unwrap();
        let expr3 = CompiledExpression::compile("  $val  /  8  ").unwrap();
        
        // Should all generate valid code
        let code1 = expr1.generate_rust_code();
        let code2 = expr2.generate_rust_code();
        let code3 = expr3.generate_rust_code();
        
        assert!(code1.contains("val / 8"));
        assert!(code2.contains("val / 8"));
        assert!(code3.contains("val / 8"));
        
        // Test number parsing
        let expr = CompiledExpression::compile("$val * 0.5").unwrap();
        let code = expr.generate_rust_code();
        assert!(code.contains("0.5"));
        
        // Decimal without leading zero should fail
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
    fn test_comprehensive_compilation() {
        // Should be compilable (simple arithmetic)
        let compilable = vec![
            "$val / 8", "$val + 3", "$val - 5", "$val * 100",
            "($val - 104) / 8", "$val * 25.4 / 1000",
            "$val / 32 + 5", "($val + 2) * 3",
            // Functions
            "int($val)", "exp($val)", "log($val)",
            "int($val * 1000 / 25.4 + 0.5)",
            // Ternary expressions (now supported!)
            "$val >= 0 ? $val : undef",
            "$val > 655.345 ? \"inf\" : \"$val m\"",
            "$val == 0 ? \"Auto\" : \"Manual\"",
        ];
        
        for expr in compilable {
            assert!(CompiledExpression::is_compilable(expr), "Should compile: {}", expr);
        }
        
        // Should NOT be compilable (complex/unsupported expressions)
        let non_compilable = vec![
            "2 ** (-$val/3)", "abs($val)", "IsFloat($val) && $val < 100",
            "$val =~ s/ +$//", "$val & 0xffc0", "$val >> 6", "$val << 8",
            "$val +", "($val", "",  // Invalid syntax
        ];
        
        for expr in non_compilable {
            assert!(!CompiledExpression::is_compilable(expr), "Should not compile: {}", expr);
        }
    }
    
    // ================================
    // TERNARY EXPRESSION INTEGRATION TESTS
    // ================================
    
    #[test]
    fn test_ternary_boundary_check() {
        // Most common ternary pattern: boundary check with units
        let expr = CompiledExpression::compile("$val > 655.345 ? \"inf\" : \"$val m\"").unwrap();
        
        // Verify AST structure
        match expr.ast.as_ref() {
            AstNode::TernaryOp { condition, true_expr, false_expr } => {
                // Condition: $val > 655.345
                match condition.as_ref() {
                    AstNode::ComparisonOp { op: CompType::Greater, left, right } => {
                        assert!(matches!(left.as_ref(), AstNode::Variable));
                        assert!(matches!(right.as_ref(), AstNode::Number(655.345)));
                    }
                    _ => panic!("Expected comparison in condition")
                }
                
                // True branch: "inf"
                match true_expr.as_ref() {
                    AstNode::String { value, has_interpolation } => {
                        assert_eq!(value, "inf");
                        assert!(!has_interpolation);
                    }
                    _ => panic!("Expected string literal in true branch")
                }
                
                // False branch: "$val m" (with interpolation)
                match false_expr.as_ref() {
                    AstNode::String { value, has_interpolation } => {
                        assert_eq!(value, "$val m");
                        assert!(*has_interpolation);
                    }
                    _ => panic!("Expected string with interpolation in false branch")
                }
            }
            _ => panic!("Expected ternary operation")
        }
        
        // Verify code generation
        let code = expr.generate_rust_code();
        assert!(code.contains("if val > 655.345"));
        assert!(code.contains("TagValue::String(\"inf\".to_string())"));
        assert!(code.contains("format!(\"{} m\", val)"));
    }
    
    #[test]
    fn test_ternary_undef_handling() {
        // Common pattern: return value if valid, undef otherwise
        let expr = CompiledExpression::compile("$val >= 0 ? $val : undef").unwrap();
        
        match expr.ast.as_ref() {
            AstNode::TernaryOp { condition, true_expr, false_expr } => {
                // Condition: $val >= 0
                match condition.as_ref() {
                    AstNode::ComparisonOp { op: CompType::GreaterEq, .. } => {}
                    _ => panic!("Expected >= comparison")
                }
                
                // True branch: $val
                assert!(matches!(true_expr.as_ref(), AstNode::Variable));
                
                // False branch: undef
                assert!(matches!(false_expr.as_ref(), AstNode::Undefined));
            }
            _ => panic!("Expected ternary operation")
        }
        
        let code = expr.generate_rust_code();
        assert!(code.contains("if val >= 0.0"));
        assert!(code.contains("TagValue::F64(val)"));
        assert!(code.contains("value.clone()"));
    }
    
    #[test]
    fn test_ternary_numeric_branches() {
        // Test ternary with numeric branches: $val == 0 ? 1 : 2
        let expr = CompiledExpression::compile("$val == 0 ? 1 : 2").unwrap();
        
        let code = expr.generate_rust_code();
        assert!(code.contains("if val == 0.0_f64"));
        assert!(code.contains("TagValue::F64(1.0_f64)"));
        assert!(code.contains("TagValue::F64(2.0_f64)"));
    }
    
    #[test]
    fn test_function_with_ternary_arg() {
        // Test function with ternary argument: int($val >= 0 ? $val : 0)
        let expr = CompiledExpression::compile("int($val >= 0 ? $val : 0)").unwrap();
        
        match expr.ast.as_ref() {
            AstNode::FunctionCall { func, arg } => {
                assert_eq!(*func, FuncType::Int);
                assert!(matches!(arg.as_ref(), AstNode::TernaryOp { .. }));
            }
            _ => panic!("Expected function call with ternary argument")
        }
        
        let code = expr.generate_rust_code();
        assert!(code.contains("if val >= 0.0_f64 { val } else { 0.0_f64 }"));
        assert!(code.contains(".trunc()"));
    }
    
    #[test]
    fn test_comparison_only() {
        // Test standalone comparison: $val >= 0
        let expr = CompiledExpression::compile("$val >= 0").unwrap();
        
        match expr.ast.as_ref() {
            AstNode::ComparisonOp { op, left, right } => {
                assert_eq!(*op, CompType::GreaterEq);
                assert!(matches!(left.as_ref(), AstNode::Variable));
                assert!(matches!(right.as_ref(), AstNode::Number(0.0)));
            }
            _ => panic!("Expected comparison operation")
        }
        
        let code = expr.generate_rust_code();
        assert!(code.contains("TagValue::U8(if val >= 0.0_f64 { 1 } else { 0 })"));
    }
    
    #[test]
    fn test_all_comparison_operators() {
        let test_cases = vec![
            ("$val >= 0", CompType::GreaterEq, ">="),
            ("$val > 0", CompType::Greater, ">"),
            ("$val <= 0", CompType::LessEq, "<="),
            ("$val < 0", CompType::Less, "<"),
            ("$val == 0", CompType::Equal, "=="),
            ("$val != 0", CompType::NotEqual, "!="),
        ];
        
        for (expr_str, expected_op, rust_op) in test_cases {
            let expr = CompiledExpression::compile(expr_str).unwrap();
            
            match expr.ast.as_ref() {
                AstNode::ComparisonOp { op, .. } => {
                    assert_eq!(*op, expected_op);
                }
                _ => panic!("Expected comparison for {}", expr_str)
            }
            
            let code = expr.generate_rust_code();
            assert!(code.contains(&format!("val {} 0.0", rust_op)));
        }
    }
    
    #[test]
    fn test_ternary_precedence() {
        // Test that arithmetic precedence works within ternary: $val + 1 > 0 ? $val * 2 : 0
        let expr = CompiledExpression::compile("$val + 1 > 0 ? $val * 2 : 0").unwrap();
        
        match expr.ast.as_ref() {
            AstNode::TernaryOp { condition, true_expr, .. } => {
                // Condition should be: ($val + 1) > 0
                match condition.as_ref() {
                    AstNode::ComparisonOp { left, .. } => {
                        assert!(matches!(left.as_ref(), AstNode::BinaryOp { op: OpType::Add, .. }));
                    }
                    _ => panic!("Expected comparison with addition")
                }
                
                // True expression should be: $val * 2
                assert!(matches!(true_expr.as_ref(), AstNode::BinaryOp { op: OpType::Multiply, .. }));
            }
            _ => panic!("Expected ternary operation")
        }
    }
    
    #[test]
    fn test_ternary_string_edge_cases() {
        // Test various string interpolation patterns
        let test_cases = vec![
            ("$val == 0 ? \"Zero\" : \"NonZero\"", false, false), // No interpolation
            ("$val > 100 ? \"$val mm\" : \"Small\"", true, false), // Left interpolation only
            ("$val < 5 ? \"Tiny\" : \"$val units\"", false, true), // Right interpolation only
            ("$val != 0 ? \"$val°\" : \"$val (zero)\"", true, true), // Both interpolation
        ];
        
        for (expr_str, left_interp, right_interp) in test_cases {
            let expr = CompiledExpression::compile(expr_str).unwrap();
            
            match expr.ast.as_ref() {
                AstNode::TernaryOp { true_expr, false_expr, .. } => {
                    match true_expr.as_ref() {
                        AstNode::String { has_interpolation, .. } => {
                            assert_eq!(*has_interpolation, left_interp);
                        }
                        _ => panic!("Expected string in true branch")
                    }
                    
                    match false_expr.as_ref() {
                        AstNode::String { has_interpolation, .. } => {
                            assert_eq!(*has_interpolation, right_interp);
                        }
                        _ => panic!("Expected string in false branch")
                    }
                }
                _ => panic!("Expected ternary for {}", expr_str)
            }
        }
    }
    
    #[test]
    fn test_backward_compatibility_with_ast() {
        // Ensure simple arithmetic still works efficiently (should use RPN fallback)
        let simple_cases = vec![
            "$val / 8",
            "$val + 3", 
            "$val * 100",
            "($val - 5) * 2",
            "int($val)",
            "exp($val / 32)"
        ];
        
        for expr_str in simple_cases {
            let expr = CompiledExpression::compile(expr_str).unwrap();
            let code = expr.generate_rust_code();
            
            // Should generate working code
            assert!(code.contains("match value.as_f64()"));
            assert!(code.contains("Some(val) => Ok("));
            assert!(code.contains("None => Ok(value.clone())"));
        }
    }
    
    #[test] 
    fn test_real_world_ternary_patterns() {
        // Test actual ExifTool ternary patterns found in the wild
        
        // Canon distance units (boundary check)
        let expr = CompiledExpression::compile("$val > 655.345 ? \"inf\" : \"$val m\"").unwrap();
        let code = expr.generate_rust_code();
        assert!(code.contains("if val > 655.345"));
        assert!(code.contains("format!(\"{} m\", val)"));
        
        // Nikon flash compensation (sign handling)
        let expr = CompiledExpression::compile("$val >= 0 ? \"+$val\" : \"$val\"").unwrap();
        let code = expr.generate_rust_code();
        assert!(code.contains("if val >= 0.0"));
        assert!(code.contains("format!(\"+{}\", val)"));
        assert!(code.contains("format!(\"{}\", val)"));
        
        // Sony lens info (special value detection)
        let expr = CompiledExpression::compile("$val == 0 ? \"n/a\" : \"$val mm\"").unwrap();
        let code = expr.generate_rust_code();
        assert!(code.contains("if val == 0.0"));
        assert!(code.contains("TagValue::String(\"n/a\".to_string())"));
        
        // Olympus mode (numeric branches)
        let expr = CompiledExpression::compile("$val != 0 ? $val : undef").unwrap();
        let code = expr.generate_rust_code();
        assert!(code.contains("if val != 0.0"));
        assert!(code.contains("TagValue::F64(val)"));
        assert!(code.contains("value.clone()"));
    }
    
    #[test]
    fn test_sprintf_compilation_and_codegen() {
        // Test sprintf("%.1f mm", $val) - the most common pattern for FocalLength
        let expr = CompiledExpression::compile("sprintf(\"%.1f mm\", $val)").unwrap();
        
        // Verify AST structure
        match expr.ast.as_ref() {
            AstNode::Sprintf { format_string, args } => {
                assert_eq!(format_string, "%.1f mm");
                assert_eq!(args.len(), 1);
                assert!(matches!(args[0].as_ref(), AstNode::Variable));
            }
            _ => panic!("Expected Sprintf AST node")
        }
        
        // Verify code generation
        let code = expr.generate_rust_code();
        assert!(code.contains("TagValue::String(format!("));
        assert!(code.contains("{:.1} mm")); // Perl %.1f should convert to Rust {:.1}
        assert!(code.contains("val")); // Variable should be included
    }
    
    #[test]
    fn test_sprintf_format_conversion() {
        // Test various format specifiers
        let test_cases = vec![
            ("sprintf(\"%.1f\", $val)", "{:.1}"),
            ("sprintf(\"%.2f\", $val)", "{:.2}"),
            ("sprintf(\"%d\", $val)", "{}"),
            ("sprintf(\"%x\", $val)", "{:x}"),
            ("sprintf(\"%.1f mm\", $val)", "{:.1} mm"),
        ];
        
        for (input, expected_format) in test_cases {
            let expr = CompiledExpression::compile(input).unwrap();
            let code = expr.generate_rust_code();
            assert!(code.contains(expected_format), 
                   "Format conversion failed for {}: expected {} in {}", input, expected_format, code);
        }
    }
    
    #[test]
    fn test_string_concatenation() {
        // Test Perl string concatenation operator
        let test_cases = vec![
            // Basic concatenation
            ("$val . \" m\"", "format!(\"{}{}\", val, \" m\")"),
            ("\"Error: \" . $val", "format!(\"{}{}\", \"Error: \", val)"),
            // More complex patterns  
            ("$val . \" mm\"", "format!(\"{}{}\", val, \" mm\")"),
        ];
        
        for (input, _expected_code) in test_cases {
            // Test that it compiles
            let expr = CompiledExpression::compile(input).unwrap();
            let code = expr.generate_rust_code();
            
            // Verify it generates TagValue::String with format! call
            assert!(code.contains("TagValue::String"), 
                   "String concatenation should generate TagValue::String for {}: {}", input, code);
            assert!(code.contains("format!"), 
                   "String concatenation should use format! macro for {}: {}", input, code);
        }
        
        // Test is_compilable detection
        assert!(CompiledExpression::is_compilable("$val . \" m\""));
        assert!(CompiledExpression::is_compilable("\"Error: \" . $val"));
        assert!(CompiledExpression::is_compilable("\"$val\" . \" mm\""));
    }
    
    #[test]
    fn test_exiftool_function_compilation() {
        // Test ExifTool function call compilation
        let expr = CompiledExpression::compile("Image::ExifTool::Exif::PrintExposureTime($val)").unwrap();
        
        // Verify AST structure
        match expr.ast.as_ref() {
            AstNode::ExifToolFunction { name, arg } => {
                assert_eq!(name, "Image::ExifTool::Exif::PrintExposureTime");
                assert!(matches!(arg.as_ref(), AstNode::Variable));
            }
            _ => panic!("Expected ExifToolFunction AST node")
        }
        
        // Verify code generation
        let code = expr.generate_rust_code();
        assert!(code.contains("exposuretime_print_conv"));
        assert!(code.contains("match value.as_f64()"));
    }
    
    #[test]
    fn test_exiftool_function_codegen() {
        // Test known ExifTool functions generate direct calls
        let test_cases = vec![
            ("Image::ExifTool::Exif::PrintExposureTime($val)", "exposuretime_print_conv"),
            ("Image::ExifTool::Exif::PrintFNumber($val)", "fnumber_print_conv"),
            ("Image::ExifTool::Exif::PrintFraction($val)", "print_fraction"),
        ];
        
        for (input, expected_func) in test_cases {
            let expr = CompiledExpression::compile(input).unwrap();
            let code = expr.generate_rust_code();
            assert!(code.contains(expected_func), 
                   "Expected {} in generated code for {}: {}", expected_func, input, code);
        }
    }
    
    #[test]
    fn test_exiftool_function_unknown_fallback() {
        // Test unknown ExifTool function generates fallback
        let expr = CompiledExpression::compile("Image::ExifTool::Unknown::SomeFunction($val)").unwrap();
        let code = expr.generate_rust_code();
        
        // Should generate fallback to missing_print_conv
        assert!(code.contains("missing_print_conv"));
        assert!(code.contains("Unknown::SomeFunction"));
    }
    
    #[test]
    fn test_exiftool_function_is_compilable() {
        // Simple single-argument ExifTool functions should be compilable
        assert!(CompiledExpression::is_compilable("Image::ExifTool::Exif::PrintExposureTime($val)"));
        assert!(CompiledExpression::is_compilable("Image::ExifTool::Exif::PrintFNumber($val)"));
        assert!(CompiledExpression::is_compilable("Image::ExifTool::GPS::ToDegrees($val)"));
        assert!(CompiledExpression::is_compilable("Image::ExifTool::Custom::Unknown($val)"));
        
        // Complex multi-argument functions should not be compilable
        assert!(!CompiledExpression::is_compilable("Image::ExifTool::GPS::ToDMS($self, $val, 1, \"N\")"));
        assert!(!CompiledExpression::is_compilable("Image::ExifTool::Exif::SomeFunc($val, $extra)"));
        assert!(!CompiledExpression::is_compilable("Image::ExifTool::Test::WithSelf($self)"));
    }
    
    #[test]
    fn test_registry_patterns_obsolescence() {
        // Test patterns that should be compilable and thus obsolete in conv_registry
        let registry_patterns = vec![
            // sprintf patterns that should be compilable
            ("sprintf(\"%.1f mm\", $val)", true),
            ("sprintf(\"%.1f\", $val)", true),
            ("sprintf(\"%.2f\", $val)", true),
            ("sprintf(\"%+d\", $val)", true),
            ("sprintf(\"%.3f mm\", $val)", true),
            
            // Simple string expressions - raw strings are NOT compilable  
            ("\"$val mm\"", false),  // This is a raw string literal, not an expression
            ("$val . \" mm\"", true),  // This would be the compilable concatenation form
            
            // Complex regex expressions - these might not be compilable yet
            ("$val =~ /^(inf|undef)$/ ? $val : \"$val m\"", false), // regex not supported
        ];
        
        let results = CompiledExpression::test_multiple_is_compilable(
            &registry_patterns.iter().map(|(expr, _)| *expr).collect::<Vec<_>>()
        );
        
        for ((pattern, expected), (_, actual)) in registry_patterns.iter().zip(results.iter()) {
            if *expected {
                assert!(actual, "Pattern should be compilable: {}", pattern);
                println!("✅ OBSOLETE in registry: {}", pattern);
            } else {
                if *actual {
                    println!("⚠️  Unexpectedly compilable: {}", pattern);
                } else {
                    println!("❌ Still needs registry: {}", pattern);
                }
            }
        }
    }
}