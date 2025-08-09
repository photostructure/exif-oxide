//! Comprehensive integration tests for the expression compiler
//!
//! These tests cover the full compilation pipeline from expression strings
//! to generated Rust code, including edge cases and error conditions.

#[cfg(test)]
mod integration_tests {
    use super::super::*;
    use crate::expression_compiler::types::{AstNode, CompType, FuncType, OpType};

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
            _ => panic!("Expected BinaryOp for division"),
        }

        let code = expr.generate_rust_code();
        assert!(code.contains("val / 8"));
    }

    #[test]
    fn test_complex_expression() {
        let expr = CompiledExpression::compile("($val - 104) / 8").unwrap();
        // Verify AST structure for complex expression: ($val - 104) / 8
        match expr.ast.as_ref() {
            AstNode::BinaryOp {
                op: OpType::Divide,
                left,
                right,
            } => {
                // Left side should be ($val - 104)
                match left.as_ref() {
                    AstNode::BinaryOp {
                        op: OpType::Subtract,
                        left: inner_left,
                        right: inner_right,
                    } => {
                        assert!(matches!(inner_left.as_ref(), AstNode::Variable));
                        assert!(matches!(inner_right.as_ref(), AstNode::Number(104.0)));
                    }
                    _ => panic!("Expected subtraction on left side"),
                }
                // Right side should be 8
                assert!(matches!(right.as_ref(), AstNode::Number(8.0)));
            }
            _ => panic!("Expected division at root"),
        }
    }

    #[test]
    fn test_val_index_patterns() {
        // Test basic $val[0] pattern
        let expr = CompiledExpression::compile("$val[0]").unwrap();
        match expr.ast.as_ref() {
            AstNode::ValIndex(index) => {
                assert_eq!(*index, 0);
            }
            _ => panic!("Expected ValIndex for $val[0]"),
        }

        let code = expr.generate_rust_code();
        assert!(code.contains("resolved_dependencies.get(0)"));
    }

    #[test]
    fn test_val_index_expression() {
        // Test GPS ValueConv pattern: $val[1] =~ /^S/i ? -$val[0] : $val[0]
        let expr = CompiledExpression::compile("$val[1] >= 0 ? -$val[0] : $val[0]").unwrap();
        match expr.ast.as_ref() {
            AstNode::TernaryOp {
                condition,
                true_expr,
                false_expr,
            } => {
                // Condition should be comparison with $val[1]
                match condition.as_ref() {
                    AstNode::ComparisonOp { left, .. } => {
                        assert!(matches!(left.as_ref(), AstNode::ValIndex(1)));
                    }
                    _ => panic!("Expected comparison in condition"),
                }
                // True expr should be -$val[0]
                match true_expr.as_ref() {
                    AstNode::UnaryMinus { operand } => {
                        assert!(matches!(operand.as_ref(), AstNode::ValIndex(0)));
                    }
                    _ => panic!("Expected unary minus of $val[0]"),
                }
                // False expr should be $val[0]
                assert!(matches!(false_expr.as_ref(), AstNode::ValIndex(0)));
            }
            _ => panic!("Expected ternary expression"),
        }
    }

    #[test]
    fn test_val_index_arithmetic() {
        // Test arithmetic with indexed values
        let expr = CompiledExpression::compile("$val[0] + $val[1]").unwrap();
        match expr.ast.as_ref() {
            AstNode::BinaryOp { op, left, right } => {
                assert_eq!(*op, OpType::Add);
                assert!(matches!(left.as_ref(), AstNode::ValIndex(0)));
                assert!(matches!(right.as_ref(), AstNode::ValIndex(1)));
            }
            _ => panic!("Expected BinaryOp for addition"),
        }

        let code = expr.generate_rust_code();
        assert!(code.contains("resolved_dependencies.get(0)"));
        assert!(code.contains("resolved_dependencies.get(1)"));
    }

    #[test]
    fn test_val_index_compilation_check() {
        // Test that expressions with $val[n] are now compilable
        assert!(CompiledExpression::is_compilable("$val[0]"));
        assert!(CompiledExpression::is_compilable(
            "$val[1] >= 0 ? -$val[0] : $val[0]"
        ));
        assert!(CompiledExpression::is_compilable(
            "$val[0] + $val[1] * $val[2]"
        ));
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
            _ => panic!("Expected BinaryOp for multiplication"),
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
            _ => panic!("Expected FunctionCall for int()"),
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
            _ => panic!("Expected FunctionCall for exp()"),
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
            _ => panic!("Expected FunctionCall for log()"),
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
        assert!(CompiledExpression::is_compilable(
            "$val >= 0 ? $val : undef"
        ));
        assert!(CompiledExpression::is_compilable(
            "$val > 655.345 ? \"inf\" : \"$val m\""
        ));
        assert!(CompiledExpression::is_compilable(
            "$val == 0 ? \"Auto\" : \"Manual\""
        ));

        // sprintf patterns are now supported
        assert!(CompiledExpression::is_compilable(
            "sprintf(\"%.1f mm\", $val)"
        ));
        assert!(CompiledExpression::is_compilable("sprintf(\"%.2f\", $val)"));
        assert!(CompiledExpression::is_compilable("sprintf(\"%d\", $val)"));

        // String concatenation is now supported
        assert!(CompiledExpression::is_compilable("$val . \" m\""));
        assert!(CompiledExpression::is_compilable("\"Error: \" . $val"));
        assert!(CompiledExpression::is_compilable("\"$val\" . \" mm\""));

        // Complex patterns should still not be compilable (except power with unary minus, which we now support)
        assert!(!CompiledExpression::is_compilable("abs($val)"));
        assert!(!CompiledExpression::is_compilable(
            "IsFloat($val) && $val < 100"
        ));
        assert!(!CompiledExpression::is_compilable("$val =~ s/ +$//"));
        // Note: Bitwise operations are now supported - moved to test_bitwise_is_compilable

        // Simple ExifTool function calls are now supported
        assert!(CompiledExpression::is_compilable(
            "Image::ExifTool::Exif::PrintExposureTime($val)"
        ));
        assert!(CompiledExpression::is_compilable(
            "Image::ExifTool::Exif::PrintFNumber($val)"
        ));
        assert!(CompiledExpression::is_compilable(
            "Image::ExifTool::GPS::ToDegrees($val)"
        ));

        // Complex ExifTool function calls with multiple arguments are not supported
        assert!(!CompiledExpression::is_compilable(
            "Image::ExifTool::GPS::ToDMS($self, $val, 1, \"N\")"
        ));
    }

    #[test]
    fn test_invalid_expressions() {
        assert!(CompiledExpression::compile("$val +").is_err());
        assert!(CompiledExpression::compile("($val").is_err());
        assert!(CompiledExpression::compile("$val )").is_err());
        assert!(CompiledExpression::compile("$foo / 8").is_err());
    }

    #[test]
    fn test_malformed_syntax_with_bare_closing_paren() {
        // Test for parsing bug: expressions with bare closing parenthesis should fail
        // In ExifTool, closing parenthesis must either:
        // 1. Be part of a quoted string literal like "$val )"
        // 2. Be part of string concatenation like $val . ")"
        // 3. Close an opening parenthesis like ($val + 5)
        // 4. Be part of a function call like sprintf("format", $val)
        //
        // A bare ) after a variable with no opening ( should be a tokenization error
        assert!(
            CompiledExpression::compile("$val )").is_err(),
            "Bare closing parenthesis after variable should fail to parse"
        );

        // Related malformed cases that should also fail
        assert!(
            CompiledExpression::compile("$val ] } >").is_err(),
            "Other unexpected closing delimiters should fail"
        );

        // Valid alternatives for comparison (these should work)
        assert!(
            CompiledExpression::compile("\"$val )\"").is_ok(),
            "String literal with interpolation should work"
        );
        assert!(
            CompiledExpression::compile("($val)").is_ok(),
            "Properly parenthesized expression should work"
        );
    }

    #[test]
    fn test_arithmetic_expressions_compile() {
        // Test that various arithmetic expressions compile successfully
        let test_cases = vec![
            "$val + 2 * 3",         // precedence
            "$val - 8 / 2",         // precedence
            "$val - 5 + 3",         // left associativity
            "$val / 4 / 2",         // associativity
            "($val + 2) * 3",       // parentheses
            "$val / (8 - (2 + 1))", // nested parentheses
            "$val * 25.4 / 1000",   // real-world
            "$val / 32 + 5",        // offset
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

        // Invalid characters (note: & and | are now supported, but ^ % @ are not)
        assert!(CompiledExpression::compile("$val ^ 8").is_err());
        assert!(CompiledExpression::compile("$val % 8").is_err());
        assert!(CompiledExpression::compile("$val @ 8").is_err());
    }

    #[test]
    fn test_comprehensive_compilation() {
        // Should be compilable (simple arithmetic)
        let compilable = vec![
            "$val / 8",
            "$val + 3",
            "$val - 5",
            "$val * 100",
            "($val - 104) / 8",
            "$val * 25.4 / 1000",
            "$val / 32 + 5",
            "($val + 2) * 3",
            // Functions
            "int($val)",
            "exp($val)",
            "log($val)",
            "int($val * 1000 / 25.4 + 0.5)",
            // Ternary expressions (now supported!)
            "$val >= 0 ? $val : undef",
            "$val > 655.345 ? \"inf\" : \"$val m\"",
            "$val == 0 ? \"Auto\" : \"Manual\"",
            // Bitwise and shift operations (now supported!)
            "$val & 0xffc0",
            "$val >> 6",
            "$val << 8",
        ];

        for expr in compilable {
            assert!(
                CompiledExpression::is_compilable(expr),
                "Should compile: {}",
                expr
            );
        }

        // Should NOT be compilable (complex/unsupported expressions)
        let non_compilable = vec![
            "abs($val)",
            "IsFloat($val) && $val < 100",
            "$val =~ s/ +$//",
            // Note: Bitwise (&, |) and shift (<<, >>) operations are now supported
            "$val +",
            "($val",
            "", // Invalid syntax
        ];

        for expr in non_compilable {
            assert!(
                !CompiledExpression::is_compilable(expr),
                "Should not compile: {}",
                expr
            );
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
            AstNode::TernaryOp {
                condition,
                true_expr,
                false_expr,
            } => {
                // Condition: $val > 655.345
                match condition.as_ref() {
                    AstNode::ComparisonOp {
                        op: CompType::Greater,
                        left,
                        right,
                    } => {
                        assert!(matches!(left.as_ref(), AstNode::Variable));
                        assert!(matches!(right.as_ref(), AstNode::Number(655.345)));
                    }
                    _ => panic!("Expected comparison in condition"),
                }

                // True branch: "inf"
                match true_expr.as_ref() {
                    AstNode::String {
                        value,
                        has_interpolation,
                    } => {
                        assert_eq!(value, "inf");
                        assert!(!has_interpolation);
                    }
                    _ => panic!("Expected string literal in true branch"),
                }

                // False branch: "$val m" (with interpolation)
                match false_expr.as_ref() {
                    AstNode::String {
                        value,
                        has_interpolation,
                    } => {
                        assert_eq!(value, "$val m");
                        assert!(*has_interpolation);
                    }
                    _ => panic!("Expected string with interpolation in false branch"),
                }
            }
            _ => panic!("Expected ternary operation"),
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
            AstNode::TernaryOp {
                condition,
                true_expr,
                false_expr,
            } => {
                // Condition: $val >= 0
                match condition.as_ref() {
                    AstNode::ComparisonOp {
                        op: CompType::GreaterEq,
                        ..
                    } => {}
                    _ => panic!("Expected >= comparison"),
                }

                // True branch: $val
                assert!(matches!(true_expr.as_ref(), AstNode::Variable));

                // False branch: undef
                assert!(matches!(false_expr.as_ref(), AstNode::Undefined));
            }
            _ => panic!("Expected ternary operation"),
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
            _ => panic!("Expected function call with ternary argument"),
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
            _ => panic!("Expected comparison operation"),
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
                _ => panic!("Expected comparison for {}", expr_str),
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
            AstNode::TernaryOp {
                condition,
                true_expr,
                ..
            } => {
                // Condition should be: ($val + 1) > 0
                match condition.as_ref() {
                    AstNode::ComparisonOp { left, .. } => {
                        assert!(matches!(
                            left.as_ref(),
                            AstNode::BinaryOp {
                                op: OpType::Add,
                                ..
                            }
                        ));
                    }
                    _ => panic!("Expected comparison with addition"),
                }

                // True expression should be: $val * 2
                assert!(matches!(
                    true_expr.as_ref(),
                    AstNode::BinaryOp {
                        op: OpType::Multiply,
                        ..
                    }
                ));
            }
            _ => panic!("Expected ternary operation"),
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
                AstNode::TernaryOp {
                    true_expr,
                    false_expr,
                    ..
                } => {
                    match true_expr.as_ref() {
                        AstNode::String {
                            has_interpolation, ..
                        } => {
                            assert_eq!(*has_interpolation, left_interp);
                        }
                        _ => panic!("Expected string in true branch"),
                    }

                    match false_expr.as_ref() {
                        AstNode::String {
                            has_interpolation, ..
                        } => {
                            assert_eq!(*has_interpolation, right_interp);
                        }
                        _ => panic!("Expected string in false branch"),
                    }
                }
                _ => panic!("Expected ternary for {}", expr_str),
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
            "exp($val / 32)",
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
            AstNode::Sprintf {
                format_string,
                args,
            } => {
                assert_eq!(format_string, "%.1f mm");
                assert_eq!(args.len(), 1);
                assert!(matches!(args[0].as_ref(), AstNode::Variable));
            }
            _ => panic!("Expected Sprintf AST node"),
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
            assert!(
                code.contains(expected_format),
                "Format conversion failed for {}: expected {} in {}",
                input,
                expected_format,
                code
            );
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
            assert!(
                code.contains("TagValue::String"),
                "String concatenation should generate TagValue::String for {}: {}",
                input,
                code
            );
            assert!(
                code.contains("format!"),
                "String concatenation should use format! macro for {}: {}",
                input,
                code
            );
        }

        // Test is_compilable detection
        assert!(CompiledExpression::is_compilable("$val . \" m\""));
        assert!(CompiledExpression::is_compilable("\"Error: \" . $val"));
        assert!(CompiledExpression::is_compilable("\"$val\" . \" mm\""));
    }

    #[test]
    fn test_exiftool_function_compilation() {
        // Test ExifTool function call compilation
        let expr =
            CompiledExpression::compile("Image::ExifTool::Exif::PrintExposureTime($val)").unwrap();

        // Verify AST structure
        match expr.ast.as_ref() {
            AstNode::ExifToolFunction { name, arg } => {
                assert_eq!(name, "Image::ExifTool::Exif::PrintExposureTime");
                assert!(matches!(arg.as_ref(), AstNode::Variable));
            }
            _ => panic!("Expected ExifToolFunction AST node"),
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
            (
                "Image::ExifTool::Exif::PrintExposureTime($val)",
                "exposuretime_print_conv",
            ),
            (
                "Image::ExifTool::Exif::PrintFNumber($val)",
                "fnumber_print_conv",
            ),
            (
                "Image::ExifTool::Exif::PrintFraction($val)",
                "print_fraction",
            ),
        ];

        for (input, expected_func) in test_cases {
            let expr = CompiledExpression::compile(input).unwrap();
            let code = expr.generate_rust_code();
            assert!(
                code.contains(expected_func),
                "Expected {} in generated code for {}: {}",
                expected_func,
                input,
                code
            );
        }
    }

    #[test]
    fn test_exiftool_function_unknown_fallback() {
        // Test unknown ExifTool function generates fallback
        let expr =
            CompiledExpression::compile("Image::ExifTool::Unknown::SomeFunction($val)").unwrap();
        let code = expr.generate_rust_code();

        // Should generate fallback to missing_print_conv
        assert!(code.contains("missing_print_conv"));
        assert!(code.contains("Unknown::SomeFunction"));
    }

    #[test]
    fn test_exiftool_function_is_compilable() {
        // Simple single-argument ExifTool functions should be compilable
        assert!(CompiledExpression::is_compilable(
            "Image::ExifTool::Exif::PrintExposureTime($val)"
        ));
        assert!(CompiledExpression::is_compilable(
            "Image::ExifTool::Exif::PrintFNumber($val)"
        ));
        assert!(CompiledExpression::is_compilable(
            "Image::ExifTool::GPS::ToDegrees($val)"
        ));
        assert!(CompiledExpression::is_compilable(
            "Image::ExifTool::Custom::Unknown($val)"
        ));

        // Complex multi-argument functions should not be compilable
        assert!(!CompiledExpression::is_compilable(
            "Image::ExifTool::GPS::ToDMS($self, $val, 1, \"N\")"
        ));
        assert!(!CompiledExpression::is_compilable(
            "Image::ExifTool::Exif::SomeFunc($val, $extra)"
        ));
        assert!(!CompiledExpression::is_compilable(
            "Image::ExifTool::Test::WithSelf($self)"
        ));
    }

    #[test]
    fn test_failing_registry_patterns() {
        // Test patterns from the failing conv_registry test
        let failing_patterns = vec![
            "$val ? exp(($val/8-6)*log(2))*100 : $val", // Complex ternary with exp/log - should be compilable!
            "2 ** (($val/8 - 1) / 2)",                  // Power operations not supported
            "$val ? 2 ** (6 - $val/8) : 0",             // Power operations not supported
            "$val=~s/ +$//; $val",                      // Regex operations not supported
            "2 ** (-$val/3)",                           // Power operations not supported
        ];

        for pattern in &failing_patterns {
            println!("Testing registry pattern: {}", pattern);
            let is_compilable = CompiledExpression::is_compilable(pattern);
            println!("  is_compilable: {}", is_compilable);

            if is_compilable {
                match CompiledExpression::compile(pattern) {
                    Ok(expr) => {
                        println!("  ✅ Compiled successfully");
                        let code = expr.generate_rust_code();
                        println!("  Generated: {}", code);
                    }
                    Err(e) => {
                        println!("  ❌ Compilation failed: {}", e);
                    }
                }
            } else {
                println!("  ❌ Not compilable (expected for regex/power operations)");
            }
        }
    }

    #[test]
    fn test_string_interpolation_debug() {
        // Debug string interpolation patterns
        let patterns = vec!["\"$val mm\"", "\"$val\"", "\"hello $val world\""];

        for pattern in patterns {
            println!("Testing pattern: {}", pattern);
            match CompiledExpression::compile(pattern) {
                Ok(expr) => {
                    println!("  ✅ Compiled successfully");
                    println!("  AST: {:?}", expr.ast);
                    let code = expr.generate_rust_code();
                    println!("  Generated: {}", code);
                }
                Err(e) => {
                    println!("  ❌ Failed: {}", e);
                }
            }
            println!(
                "  is_compilable: {}",
                CompiledExpression::is_compilable(pattern)
            );
        }
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
            // String interpolation expressions - now supported!
            ("\"$val mm\"", true), // String interpolation (equivalent to format!("{} mm", val))
            ("$val . \" mm\"", true), // String concatenation (also works)
            // Complex regex expressions - these might not be compilable yet
            ("$val =~ /^(inf|undef)$/ ? $val : \"$val m\"", false), // regex not supported
        ];

        let results = CompiledExpression::test_multiple_is_compilable(
            &registry_patterns
                .iter()
                .map(|(expr, _)| *expr)
                .collect::<Vec<_>>(),
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

    // ================================
    // POWER OPERATION TESTS
    // ================================

    #[test]
    fn test_simple_power_operation() {
        // Test basic power operation: 2**3
        let expr = CompiledExpression::compile("2**3").unwrap();

        // Verify AST structure
        match expr.ast.as_ref() {
            AstNode::BinaryOp { op, left, right } => {
                assert_eq!(*op, OpType::Power);
                assert!(matches!(left.as_ref(), AstNode::Number(2.0)));
                assert!(matches!(right.as_ref(), AstNode::Number(3.0)));
            }
            _ => panic!("Expected power operation"),
        }

        // Verify code generation - debug the actual output
        let code = expr.generate_rust_code();
        println!("Generated code for 2**3: {}", code);
        assert!(code.contains("2.0_f64.powf(3.0_f64)"));
    }

    #[test]
    fn test_power_right_associativity() {
        // Test right associativity: 2**3**2 should be 2**(3**2) = 512, not (2**3)**2 = 64
        let expr = CompiledExpression::compile("2**3**2").unwrap();

        // Verify AST structure: should be Power(2, Power(3, 2))
        match expr.ast.as_ref() {
            AstNode::BinaryOp { op, left, right } => {
                assert_eq!(*op, OpType::Power);
                assert!(matches!(left.as_ref(), AstNode::Number(2.0)));
                // Right side should be another power operation
                match right.as_ref() {
                    AstNode::BinaryOp {
                        op: OpType::Power,
                        left: inner_left,
                        right: inner_right,
                    } => {
                        assert!(matches!(inner_left.as_ref(), AstNode::Number(3.0)));
                        assert!(matches!(inner_right.as_ref(), AstNode::Number(2.0)));
                    }
                    _ => panic!("Expected nested power operation"),
                }
            }
            _ => panic!("Expected power operation"),
        }

        // Debug the actual output
        let code = expr.generate_rust_code();
        println!("Generated code for 2**3**2: {}", code);
        assert!(code.contains("2.0_f64.powf(3.0_f64.powf(2.0_f64))"));
    }

    #[test]
    fn test_power_with_variable() {
        // Test power with variable: 2**$val
        let expr = CompiledExpression::compile("2**$val").unwrap();

        match expr.ast.as_ref() {
            AstNode::BinaryOp { op, left, right } => {
                assert_eq!(*op, OpType::Power);
                assert!(matches!(left.as_ref(), AstNode::Number(2.0)));
                assert!(matches!(right.as_ref(), AstNode::Variable));
            }
            _ => panic!("Expected power operation"),
        }

        let code = expr.generate_rust_code();
        assert!(code.contains("2.0_f64.powf(val)"));
    }

    #[test]
    fn test_negative_power() {
        // Test negative power: 2**(-$val) - common APEX pattern
        let expr = CompiledExpression::compile("2**(-$val)").unwrap();

        match expr.ast.as_ref() {
            AstNode::BinaryOp { op, left, right } => {
                assert_eq!(*op, OpType::Power);
                assert!(matches!(left.as_ref(), AstNode::Number(2.0)));
                // Right side should be unary minus of variable
                match right.as_ref() {
                    AstNode::UnaryMinus { operand } => {
                        assert!(matches!(operand.as_ref(), AstNode::Variable));
                    }
                    _ => panic!("Expected unary minus of variable"),
                }
            }
            _ => panic!("Expected power operation"),
        }

        let code = expr.generate_rust_code();
        assert!(code.contains("2.0_f64.powf(-val)"));
    }

    #[test]
    fn test_complex_power_expression() {
        // Test complex expression: 2**(($val/8 - 1)/2) - Sony F-number pattern
        let expr = CompiledExpression::compile("2**(($val/8 - 1)/2)").unwrap();

        // Should parse correctly with proper precedence
        match expr.ast.as_ref() {
            AstNode::BinaryOp {
                op: OpType::Power,
                left,
                right,
            } => {
                assert!(matches!(left.as_ref(), AstNode::Number(2.0)));
                // Right side should be complex expression
                match right.as_ref() {
                    AstNode::BinaryOp {
                        op: OpType::Divide, ..
                    } => {
                        // Detailed structure verification would be very verbose
                        // The key is that it parses without error
                    }
                    _ => panic!("Expected division in power exponent"),
                }
            }
            _ => panic!("Expected power operation"),
        }

        let code = expr.generate_rust_code();
        assert!(code.contains(".powf("));
        assert!(code.contains("val / 8"));
    }

    #[test]
    fn test_power_precedence() {
        // Test precedence: 2*3**2 should be 2*(3**2) = 18, not (2*3)**2 = 36
        let expr = CompiledExpression::compile("2*3**2").unwrap();

        // Should parse as Multiply(2, Power(3, 2))
        match expr.ast.as_ref() {
            AstNode::BinaryOp {
                op: OpType::Multiply,
                left,
                right,
            } => {
                assert!(matches!(left.as_ref(), AstNode::Number(2.0)));
                match right.as_ref() {
                    AstNode::BinaryOp {
                        op: OpType::Power,
                        left: base,
                        right: exp,
                    } => {
                        assert!(matches!(base.as_ref(), AstNode::Number(3.0)));
                        assert!(matches!(exp.as_ref(), AstNode::Number(2.0)));
                    }
                    _ => panic!("Expected power operation on right side"),
                }
            }
            _ => panic!("Expected multiplication"),
        }
    }

    #[test]
    fn test_power_in_ternary() {
        // Test power in ternary: $val ? 2**(-$val/3) : 0
        let expr = CompiledExpression::compile("$val ? 2**(-$val/3) : 0").unwrap();

        match expr.ast.as_ref() {
            AstNode::TernaryOp {
                condition,
                true_expr,
                false_expr,
            } => {
                assert!(matches!(condition.as_ref(), AstNode::Variable));
                assert!(matches!(false_expr.as_ref(), AstNode::Number(0.0)));

                // True expression should be power operation
                match true_expr.as_ref() {
                    AstNode::BinaryOp {
                        op: OpType::Power, ..
                    } => {
                        // Power operation is present
                    }
                    _ => panic!("Expected power operation in true branch"),
                }
            }
            _ => panic!("Expected ternary operation"),
        }

        let code = expr.generate_rust_code();
        assert!(code.contains("if val != 0.0"));
        assert!(code.contains(".powf("));
    }

    #[test]
    fn test_apex_shutter_speed_pattern() {
        // Test real ExifTool pattern: 2**(-$val) for APEX shutter speed
        let expr = CompiledExpression::compile("2**(-$val)").unwrap();
        let code = expr.generate_rust_code();

        // Should generate valid Rust code
        assert!(code.contains("match value.as_f64()"));
        assert!(code.contains("2.0_f64.powf(-val)"));
        assert!(code.contains("Some(val) => Ok(TagValue::F64("));
    }

    #[test]
    fn test_sony_exposure_pattern() {
        // Test Sony pattern: $val ? 2**(6 - $val/8) : 0
        let expr = CompiledExpression::compile("$val ? 2**(6 - $val/8) : 0").unwrap();
        let code = expr.generate_rust_code();

        // Should compile to ternary with power operation
        assert!(code.contains("if val != 0.0"));
        assert!(code.contains("2.0_f64.powf(6.0_f64 - val / 8.0_f64)"));
    }

    #[test]
    fn test_is_compilable_with_power() {
        // Power operations should now be considered compilable
        assert!(CompiledExpression::is_compilable("2**3"));
        assert!(CompiledExpression::is_compilable("$val**2"));
        assert!(CompiledExpression::is_compilable("2**$val"));
        assert!(CompiledExpression::is_compilable("2**(-$val)"));
        assert!(CompiledExpression::is_compilable("2**(6 - $val/8)"));
        assert!(CompiledExpression::is_compilable("$val ? 2**(-$val/3) : 0"));

        // Complex power expressions should be compilable
        assert!(CompiledExpression::is_compilable("2**(($val/8 - 1)/2)"));
        assert!(CompiledExpression::is_compilable(
            "100 * 2**(16 - $val/256)"
        ));
    }

    // ================================
    // UNARY MINUS OPERATION TESTS
    // ================================

    #[test]
    fn test_simple_unary_minus() {
        // Test basic unary minus: -$val
        let expr = CompiledExpression::compile("-$val").unwrap();

        // Verify AST structure
        match expr.ast.as_ref() {
            AstNode::UnaryMinus { operand } => {
                assert!(matches!(operand.as_ref(), AstNode::Variable));
            }
            _ => panic!("Expected unary minus operation"),
        }

        // Verify code generation
        let code = expr.generate_rust_code();
        assert!(code.contains("TagValue::F64(-val)"));
    }

    #[test]
    fn test_unary_minus_with_numbers() {
        // Test unary minus with number: -42
        let expr = CompiledExpression::compile("-42").unwrap();

        match expr.ast.as_ref() {
            AstNode::UnaryMinus { operand } => {
                assert!(matches!(operand.as_ref(), AstNode::Number(42.0)));
            }
            _ => panic!("Expected unary minus operation"),
        }

        let code = expr.generate_rust_code();
        assert!(code.contains("TagValue::F64(-42.0_f64)"));
    }

    #[test]
    fn test_unary_minus_precedence() {
        // Test that unary minus binds tighter than multiplication: -$val * 2
        let expr = CompiledExpression::compile("-$val * 2").unwrap();

        match expr.ast.as_ref() {
            AstNode::BinaryOp {
                op: OpType::Multiply,
                left,
                right,
            } => {
                // Left should be unary minus of variable
                assert!(matches!(left.as_ref(), AstNode::UnaryMinus { .. }));
                // Right should be number 2
                assert!(matches!(right.as_ref(), AstNode::Number(2.0)));
            }
            _ => panic!("Expected multiplication with unary minus on left"),
        }
    }

    #[test]
    fn test_unary_minus_in_expressions() {
        // Test various contexts where unary minus should work
        let test_cases = vec![
            "-$val",                    // Simple unary minus
            "(-$val)",                  // Parentheses
            "-$val + 5",                // In arithmetic
            "$val * (-2)",              // In function argument
            "$val >= 0 ? $val : -$val", // In ternary
        ];

        for expr_str in test_cases {
            let expr = CompiledExpression::compile(expr_str).unwrap();
            let code = expr.generate_rust_code();

            // Should generate working code
            assert!(code.contains("match value.as_f64()"));
            println!("✅ Compiled: {} -> valid code", expr_str);
        }
    }

    #[test]
    fn test_is_compilable_with_unary_minus() {
        // Unary minus should be considered compilable
        assert!(CompiledExpression::is_compilable("-$val"));
        assert!(CompiledExpression::is_compilable("-42"));
        assert!(CompiledExpression::is_compilable("2**(-$val)"));
        assert!(CompiledExpression::is_compilable("-$val * 2"));
        assert!(CompiledExpression::is_compilable("$val ? -$val : $val"));
    }

    // ================================
    // REGEX OPERATION TESTS
    // ================================

    #[test]
    fn test_regex_substitution_compilation() {
        let test_cases = vec![
            "s/Alpha/a/i",      // Case-insensitive substitution
            "s/\\0+$//",        // Remove null terminators
            "s/(.{3})$/\\.$1/", // Insert decimal point
        ];

        for expr in test_cases {
            let result = CompiledExpression::compile(expr);
            assert!(result.is_ok(), "Failed to compile regex: {}", expr);

            let compiled = result.unwrap();
            let code = compiled.generate_rust_code();
            assert!(
                code.contains("regex::Regex"),
                "Generated code should use regex crate"
            );
            assert!(
                code.contains("TagValue::String"),
                "Regex should produce string results"
            );
        }
    }

    #[test]
    fn test_transliteration_compilation() {
        let test_cases = vec![
            "tr/a-fA-F0-9//dc", // Keep only hex characters
            "tr/ABC/abc/",      // Simple character replacement
        ];

        for expr in test_cases {
            let result = CompiledExpression::compile(expr);
            assert!(
                result.is_ok(),
                "Failed to compile transliteration: {}",
                expr
            );

            let compiled = result.unwrap();
            let code = compiled.generate_rust_code();
            assert!(
                code.contains("TagValue::String"),
                "Transliteration should produce string results"
            );
        }
    }

    #[test]
    fn test_regex_substitution_ast() {
        let expr = CompiledExpression::compile("s/Alpha/a/i").unwrap();

        match expr.ast.as_ref() {
            AstNode::RegexSubstitution {
                target,
                pattern,
                replacement,
                flags,
            } => {
                assert!(matches!(target.as_ref(), AstNode::Variable));
                assert_eq!(pattern, "Alpha");
                assert_eq!(replacement, "a");
                assert_eq!(flags, "i");
            }
            _ => panic!("Expected RegexSubstitution AST node"),
        }
    }

    #[test]
    fn test_transliteration_delete_complement() {
        // Test tr/a-fA-F0-9//dc pattern (keep only hex characters)
        let expr = CompiledExpression::compile("tr/a-fA-F0-9//dc").unwrap();

        match expr.ast.as_ref() {
            AstNode::Transliteration {
                target,
                search_list,
                replace_list,
                flags,
            } => {
                assert!(matches!(target.as_ref(), AstNode::Variable));
                assert_eq!(search_list, "a-fA-F0-9");
                assert_eq!(replace_list, "");
                assert_eq!(flags, "dc");
            }
            _ => panic!("Expected Transliteration AST node"),
        }

        let code = expr.generate_rust_code();
        assert!(code.contains("keep_chars.contains(c)"));
        assert!(code.contains("filter"));
    }

    #[test]
    fn test_olympus_firmware_pattern() {
        // Test real Olympus pattern: s/(.{3})$/\.$1/
        let expr = CompiledExpression::compile("s/(.{3})$/\\.$1/").unwrap();
        let code = expr.generate_rust_code();

        assert!(code.contains("regex::Regex"));
        assert!(code.contains("replace("));
        assert!(code.contains("TagValue::String"));
    }

    #[test]
    fn test_regex_is_compilable() {
        // Regex operations should be considered compilable
        assert!(CompiledExpression::is_compilable("s/Alpha/a/i"));
        assert!(CompiledExpression::is_compilable("s/\\0+$//"));
        assert!(CompiledExpression::is_compilable("s/(.{3})$/\\.$1/"));
        assert!(CompiledExpression::is_compilable("tr/a-fA-F0-9//dc"));
        assert!(CompiledExpression::is_compilable("tr/ABC/abc/"));
    }

    // ================================
    // BITWISE OPERATION TESTS
    // ================================

    #[test]
    fn test_bitwise_and_compilation() {
        let expr = CompiledExpression::compile("$val & 0xffff").unwrap();

        // Verify AST structure
        match expr.ast.as_ref() {
            AstNode::BinaryOp { op, left, right: _ } => {
                assert_eq!(*op, OpType::BitwiseAnd);
                assert!(matches!(left.as_ref(), AstNode::Variable));
                // Right side should be a hex number (parsed as decimal for now)
            }
            _ => panic!("Expected bitwise AND operation"),
        }

        let code = expr.generate_rust_code();
        assert!(code.contains("as i64")); // Should convert to integers
        assert!(code.contains("&")); // Should use bitwise AND
        assert!(code.contains("as f64")); // Should convert back to f64
    }

    #[test]
    fn test_right_shift_compilation() {
        let expr = CompiledExpression::compile("$val >> 16").unwrap();

        match expr.ast.as_ref() {
            AstNode::BinaryOp { op, left, right } => {
                assert_eq!(*op, OpType::RightShift);
                assert!(matches!(left.as_ref(), AstNode::Variable));
                assert!(matches!(right.as_ref(), AstNode::Number(16.0)));
            }
            _ => panic!("Expected right shift operation"),
        }

        let code = expr.generate_rust_code();
        assert!(code.contains(">>"));
        assert!(code.contains("as i64"));
    }

    #[test]
    fn test_version_extraction_pattern() {
        // Test real ExifTool pattern: sprintf("%d.%.4d",$val >> 16, $val & 0xffff)
        // For now, just test the bitwise operations part
        let expr1 = CompiledExpression::compile("$val >> 16").unwrap();
        let expr2 = CompiledExpression::compile("$val & 65535").unwrap(); // 0xffff as decimal

        let code1 = expr1.generate_rust_code();
        let code2 = expr2.generate_rust_code();

        assert!(code1.contains(">>"));
        assert!(code2.contains("&"));
        assert!(code1.contains("16.0_f64"));
        assert!(code2.contains("65535.0_f64"));
    }

    #[test]
    fn test_multi_flag_extraction() {
        // Test pattern: (($val >> 13) & 0x7) - extract 3-bit field
        let expr = CompiledExpression::compile("($val >> 13) & 7").unwrap();

        // Should parse as AND of (shift) and number
        match expr.ast.as_ref() {
            AstNode::BinaryOp {
                op: OpType::BitwiseAnd,
                left,
                right,
            } => {
                // Left should be shift operation
                assert!(matches!(
                    left.as_ref(),
                    AstNode::BinaryOp {
                        op: OpType::RightShift,
                        ..
                    }
                ));
                // Right should be number 7 (0x7)
                assert!(matches!(right.as_ref(), AstNode::Number(7.0)));
            }
            _ => panic!("Expected AND of shift and number"),
        }

        let code = expr.generate_rust_code();
        assert!(code.contains(">>"));
        assert!(code.contains("&"));
        assert!(code.contains("13.0_f64"));
        assert!(code.contains("7.0_f64"));
    }

    #[test]
    fn test_bitwise_precedence() {
        // Test precedence: shifts should bind tighter than & and |
        // Expression: $val & 0xff << 8 should be $val & (0xff << 8)
        let expr = CompiledExpression::compile("$val & 255 << 8").unwrap();

        // Should parse as AND of variable and (shift of number)
        match expr.ast.as_ref() {
            AstNode::BinaryOp {
                op: OpType::BitwiseAnd,
                left,
                right,
            } => {
                assert!(matches!(left.as_ref(), AstNode::Variable));
                // Right should be left shift operation
                assert!(matches!(
                    right.as_ref(),
                    AstNode::BinaryOp {
                        op: OpType::LeftShift,
                        ..
                    }
                ));
            }
            _ => panic!("Expected AND with shift having higher precedence"),
        }
    }

    #[test]
    fn test_all_bitwise_operations() {
        let test_cases = vec![
            ("$val & 255", OpType::BitwiseAnd),
            ("$val | 128", OpType::BitwiseOr),
            ("$val << 4", OpType::LeftShift),
            ("$val >> 2", OpType::RightShift),
        ];

        for (expr_str, expected_op) in test_cases {
            let expr = CompiledExpression::compile(expr_str).unwrap();

            match expr.ast.as_ref() {
                AstNode::BinaryOp { op, .. } => {
                    assert_eq!(*op, expected_op, "Failed for expression: {}", expr_str);
                }
                _ => panic!("Expected binary operation for: {}", expr_str),
            }

            let code = expr.generate_rust_code();
            assert!(
                code.contains("as i64"),
                "Should convert to i64 for: {}",
                expr_str
            );
            assert!(
                code.contains("as f64"),
                "Should convert back to f64 for: {}",
                expr_str
            );
        }
    }

    #[test]
    fn test_bitwise_is_compilable() {
        // Bitwise operations should be considered compilable
        assert!(CompiledExpression::is_compilable("$val & 0xffff"));
        assert!(CompiledExpression::is_compilable("$val | 128"));
        assert!(CompiledExpression::is_compilable("$val << 4"));
        assert!(CompiledExpression::is_compilable("$val >> 16"));
        assert!(CompiledExpression::is_compilable("($val >> 13) & 7"));
        assert!(CompiledExpression::is_compilable("$val & 255 << 8"));
    }

    #[test]
    fn test_array_indexing_compilable() {
        // Array indexing expressions should be compilable for the hybrid architecture
        // These are used in composite tags and handled by the composite evaluation system
        assert!(CompiledExpression::is_compilable("\"$val[0] $val[1]\""));
        assert!(CompiledExpression::is_compilable(
            "\"$val[0] $val[1] $val[2]\""
        ));
        assert!(CompiledExpression::is_compilable("$val[0]"));
        assert!(CompiledExpression::is_compilable("$val[1] + $val[2]"));
        assert!(CompiledExpression::is_compilable(
            "sprintf(\"%s %s\", $val[0], $val[1])"
        ));

        // Regular $val expressions should still be compilable
        assert!(CompiledExpression::is_compilable("$val + 1"));
        assert!(CompiledExpression::is_compilable("\"$val m\""));
        assert!(CompiledExpression::is_compilable("sprintf(\"%.1f\", $val)"));
    }
}
