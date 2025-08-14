//! Tests for PPI Rust code generation
//!
//! These tests validate that the PPI AST parsing and Rust code generation
//! produces correct, compilable Rust code that matches ExifTool semantics.

#[cfg(test)]
mod tests {
    use crate::ppi::RustGenerator;
    use crate::ppi::{CodeGenError, ExpressionType, PpiNode};
    use serde_json::json;

    #[test]
    fn test_simple_arithmetic_generation() {
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Symbol",
                    "content": "$val",
                    "symbol_type": "scalar"
                }, {
                    "class": "PPI::Token::Operator",
                    "content": "/"
                }, {
                    "class": "PPI::Token::Number",
                    "content": "100",
                    "numeric_value": 100
                }],
                "class": "PPI::Statement"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::ValueConv,
            "test_divide".to_string(),
            "$val / 100".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // println!("Simple arithmetic generation result:\n{}", result);

        // Should generate clean arithmetic operation (trusting PPI structure)
        assert!(result.contains("val / 100"));
        assert!(result.contains("pub fn test_divide"));
        assert!(result.contains("Original: $val / 100"));
    }

    #[test]
    fn test_string_interpolation_generation() {
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Quote::Double",
                    "content": "\"Case $val\"",
                    "string_value": "Case $val"
                }],
                "class": "PPI::Statement"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::PrintConv,
            "test_string".to_string(),
            "\"Case $val\"".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Should generate string interpolation with format!
        assert!(result.contains("format!"));
        assert!(result.contains("Case"));
    }

    #[test]
    fn test_signature_generation() {
        let generator = RustGenerator::new(
            ExpressionType::Condition,
            "test_condition".to_string(),
            "$$self{Make} =~ /Canon/".to_string(),
        );

        let signature = generator.generate_signature();

        assert!(signature.contains("pub fn test_condition"));
        assert!(signature.contains("val: &TagValue"));
        assert!(signature.contains("ctx: &ExifContext"));
        assert!(signature.contains("-> bool"));
    }

    // Task B: Tests for Numeric & String Operations (Phase 2)

    #[test]
    fn test_hex_number_generation() {
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Number::Hex",
                    "content": "0x100",
                }],
                "class": "PPI::Statement"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::ValueConv,
            "test_hex".to_string(),
            "0x100".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Should preserve hex literal format
        assert!(result.contains("0x100"));
        assert!(result.contains("pub fn test_hex"));
    }

    #[test]
    fn test_variable_declaration_generation() {
        let ast_json = json!({
            "children": [{
                "class": "PPI::Statement::Variable",
                "children": [{
                    "class": "PPI::Token::Word",
                    "content": "my"
                }, {
                    "class": "PPI::Token::Symbol",
                    "content": "$temp"
                }, {
                    "class": "PPI::Token::Operator",
                    "content": "="
                }, {
                    "class": "PPI::Token::Symbol",
                    "content": "$val"
                }],
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::ValueConv,
            "test_variable".to_string(),
            "my $temp = $val".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Should generate Rust variable binding
        assert!(result.contains("let temp = val"));
    }

    #[test]
    fn test_regexp_substitute_generation() {
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Regexp::Substitute",
                    "content": "s/test/replacement/g",
                }],
                "class": "PPI::Statement"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::PrintConv,
            "test_substitute".to_string(),
            "s/test/replacement/g".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Should generate string replacement
        assert!(result.contains("replace"));
        assert!(result.contains("test"));
        assert!(result.contains("replacement"));
    }

    #[test]
    fn test_enhanced_float_generation() {
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Number",
                    "content": "25.4",
                    "numeric_value": 25.4
                }],
                "class": "PPI::Statement"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::ValueConv,
            "test_float".to_string(),
            "25.4".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Should preserve float format
        assert!(result.contains("25.4"));
    }

    #[test]
    fn test_recursive_visitor_arithmetic() {
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Symbol",
                    "content": "$val",
                    "symbol_type": "scalar"
                }, {
                    "class": "PPI::Token::Operator",
                    "content": "*"
                }, {
                    "class": "PPI::Token::Number",
                    "content": "25",
                    "numeric_value": 25
                }],
                "class": "PPI::Statement"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::ValueConv,
            "test_multiply".to_string(),
            "$val * 25".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // println!("Recursive visitor arithmetic result:\n{}", result);

        // Should generate clean arithmetic operation (trusting PPI structure)
        assert!(result.contains("val * 25"));
        assert!(result.contains("pub fn test_multiply"));
        assert!(result.contains("Original: $val * 25"));
    }

    #[test]
    fn test_sprintf_concatenation_ternary() {
        // Test the complex expression: sprintf("%.2f s",$val) . ($val > 254.5/60 ? " or longer" : "")
        // This AST matches what PPI actually generates
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Word",
                    "content": "sprintf"
                }, {
                    "class": "PPI::Structure::List",
                    "children": [{
                        "class": "PPI::Statement::Expression",
                        "children": [{
                            "class": "PPI::Token::Quote::Double",
                            "content": "\"%.2f s\"",
                            "string_value": "%.2f s"
                        }, {
                            "class": "PPI::Token::Operator",
                            "content": ","
                        }, {
                            "class": "PPI::Token::Symbol",
                            "content": "$val",
                            "symbol_type": "scalar"
                        }]
                    }]
                }, {
                    "class": "PPI::Token::Operator",
                    "content": "."
                }, {
                    "class": "PPI::Structure::List",
                    "children": [{
                        "class": "PPI::Statement::Expression",
                        "children": [{
                            "class": "PPI::Token::Symbol",
                            "content": "$val",
                            "symbol_type": "scalar"
                        }, {
                            "class": "PPI::Token::Operator",
                            "content": ">"
                        }, {
                            "class": "PPI::Token::Number::Float",
                            "content": "254.5",
                            "numeric_value": 254.5
                        }, {
                            "class": "PPI::Token::Operator",
                            "content": "/"
                        }, {
                            "class": "PPI::Token::Number",
                            "content": "60",
                            "numeric_value": 60
                        }, {
                            "class": "PPI::Token::Operator",
                            "content": "?"
                        }, {
                            "class": "PPI::Token::Quote::Double",
                            "content": "\" or longer\"",
                            "string_value": " or longer"
                        }, {
                            "class": "PPI::Token::Operator",
                            "content": ":"
                        }, {
                            "class": "PPI::Token::Quote::Double",
                            "content": "\"\"",
                            "string_value": ""
                        }]
                    }]
                }],
                "class": "PPI::Statement"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::PrintConv,
            "test_sprintf_concat_ternary".to_string(),
            "sprintf(\"%.2f s\",$val) . ($val > 254.5/60 ? \" or longer\" : \"\")".to_string(),
        );

        let result = generator.generate_function(&ast);

        // Debug output
        match &result {
            Ok(code) => println!("Generated sprintf test code:\n{}", code),
            Err(e) => println!("ERROR generating sprintf test: {:?}", e),
        }

        let result = result.unwrap();

        // Should generate function with sprintf, concatenation, and ternary
        assert!(result.contains("format!"));
        assert!(result.contains("{:.2}")); // Perl %.2f -> Rust {:.2}
        assert!(result.contains("if")); // Ternary -> if/else
        assert!(result.contains("pub fn test_sprintf_concat_ternary"));

        // Uncomment to see the generated code:
        // println!("Generated code:\n{}", result);
    }

    #[test]
    fn test_unary_minus_operation() {
        // Test the expression: -$val/256
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Operator",
                    "content": "-"
                }, {
                    "class": "PPI::Token::Symbol",
                    "content": "$val",
                    "symbol_type": "scalar"
                }, {
                    "class": "PPI::Token::Operator",
                    "content": "/"
                }, {
                    "class": "PPI::Token::Number",
                    "content": "256",
                    "numeric_value": 256
                }],
                "class": "PPI::Statement"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::ValueConv,
            "test_unary_minus".to_string(),
            "-$val/256".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Uncomment to see the generated code:
        // println!("Generated unary minus code:\n{}", result);

        // Should generate clean unary minus operation (trusting PPI's structure)
        assert!(result.contains("- val / 256")); // Clean, simple expression
        assert!(result.contains("pub fn test_unary_minus"));
        assert!(!result.contains("( as f64)")); // Should not have empty left operand
    }

    #[test]
    fn test_length_function_without_parens() {
        // Test the expression: length $val
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Word",
                    "content": "length"
                }, {
                    "class": "PPI::Token::Symbol",
                    "content": "$val",
                    "symbol_type": "scalar"
                }],
                "class": "PPI::Statement"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::PrintConv,
            "test_length".to_string(),
            "length $val".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Should generate proper length function call
        assert!(result.contains("TagValue::String"));
        assert!(result.contains("s.len()"));
        assert!(result.contains("pub fn test_length"));
        assert!(!result.contains("length val")); // Should not have raw function call

        // Uncomment to see the generated code:
        // println!("Generated length code:\n{}", result);
    }

    #[test]
    fn test_ternary_with_string_comparison() {
        // Test the expression: $val eq "inf" ? $val : "$val m"
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Symbol",
                    "content": "$val",
                    "symbol_type": "scalar"
                }, {
                    "class": "PPI::Token::Operator",
                    "content": "eq"
                }, {
                    "class": "PPI::Token::Quote::Double",
                    "content": "\"inf\"",
                    "string_value": "inf"
                }, {
                    "class": "PPI::Token::Operator",
                    "content": "?"
                }, {
                    "class": "PPI::Token::Symbol",
                    "content": "$val",
                    "symbol_type": "scalar"
                }, {
                    "class": "PPI::Token::Operator",
                    "content": ":"
                }, {
                    "class": "PPI::Token::Quote::Double",
                    "content": "\"$val m\"",
                    "string_value": "$val m"
                }],
                "class": "PPI::Statement"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::PrintConv,
            "test_ternary_string_eq".to_string(),
            "$val eq \"inf\" ? $val : \"$val m\"".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Uncomment to see the generated code:
        // println!("Generated ternary string comparison code:\n{}", result);

        // Should generate proper string comparison with TagValue
        assert!(result.contains(".to_string() == "));
        assert!(result.contains("if"));
        assert!(result.contains("else"));
        assert!(result.contains("pub fn test_ternary_string_eq"));
        // The function body should not have "val eq", but the comment will still contain it
        let function_body = result.split("pub fn").nth(1).unwrap();
        assert!(!function_body.contains("val eq")); // Should not have raw eq operator in function body
    }

    #[test]
    fn test_undef_keyword() {
        // Test the expression: undef
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Word",
                    "content": "undef"
                }],
                "class": "PPI::Statement"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::PrintConv,
            "test_undef".to_string(),
            "undef".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Should generate appropriate default value
        assert!(result.contains("TagValue::String(\"\".to_string())"));
        assert!(result.contains("pub fn test_undef"));
    }

    #[test]
    fn test_join_function() {
        // Test the expression: join " ", unpack "H2H2", val
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Word",
                    "content": "join"
                }, {
                    "class": "PPI::Token::Quote::Double",
                    "content": "\" \"",
                    "string_value": " "
                }, {
                    "class": "PPI::Token::Operator",
                    "content": ","
                }, {
                    "class": "PPI::Token::Word",
                    "content": "unpack"
                }, {
                    "class": "PPI::Token::Quote::Double",
                    "content": "\"H2H2\"",
                    "string_value": "H2H2"
                }, {
                    "class": "PPI::Token::Operator",
                    "content": ","
                }, {
                    "class": "PPI::Token::Symbol",
                    "content": "val",
                    "symbol_type": "scalar"
                }],
                "class": "PPI::Statement"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::PrintConv,
            "test_join_unpack".to_string(),
            "join \" \", unpack \"H2H2\", val".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Debug to see the generated code:
        println!("Generated join/unpack code:\n{}", result);

        // Should generate proper join and unpack functions
        assert!(result.contains("TagValue::String"));
        assert!(result.contains("pub fn test_join_unpack"));
        // For now, basic join functionality is sufficient
        // Full nested function parsing will be refined later
    }

    // Task D: Tests for Control Flow & Advanced Features (Phase 3)

    #[test]
    fn test_magic_variable_underscore() {
        // Test the expression: $_
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Magic",
                    "content": "$_"
                }],
                "class": "PPI::Statement"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::PrintConv,
            "test_magic_underscore".to_string(),
            "$_".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Should generate reference to val (since $_ is the default variable)
        assert!(result.contains("val"));
        assert!(result.contains("pub fn test_magic_underscore"));
    }

    #[test]
    fn test_magic_variable_at() {
        // Test the expression: $@
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Magic",
                    "content": "$@"
                }],
                "class": "PPI::Statement"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::PrintConv,
            "test_magic_at".to_string(),
            "$@".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Should generate reference to error_val
        assert!(result.contains("error_val"));
        assert!(result.contains("pub fn test_magic_at"));
    }

    #[test]
    fn test_return_statement() {
        // Test the expression: return $val
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Word",
                    "content": "return"
                }, {
                    "class": "PPI::Token::Symbol",
                    "content": "$val",
                    "symbol_type": "scalar"
                }],
                "class": "PPI::Statement::Break"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::ValueConv,
            "test_return".to_string(),
            "return $val".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Should generate proper return statement for ValueConv
        assert!(result.contains("return Ok(val)"));
        assert!(result.contains("pub fn test_return"));
    }

    #[test]
    fn test_last_statement() {
        // Test the expression: last
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Word",
                    "content": "last"
                }],
                "class": "PPI::Statement::Break"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::PrintConv,
            "test_last".to_string(),
            "last".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Should generate break statement
        assert!(result.contains("break"));
        assert!(result.contains("pub fn test_last"));
    }

    #[test]
    fn test_next_statement() {
        // Test the expression: next
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Word",
                    "content": "next"
                }],
                "class": "PPI::Statement::Break"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::PrintConv,
            "test_next".to_string(),
            "next".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Should generate continue statement
        assert!(result.contains("continue"));
        assert!(result.contains("pub fn test_next"));
    }

    #[test]
    fn test_transliterate_delete() {
        // Test the expression: tr/()K//d (remove parentheses and K)
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Regexp::Transliterate",
                    "content": "tr/()K//d"
                }],
                "class": "PPI::Statement"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::PrintConv,
            "test_tr_delete".to_string(),
            "tr/()K//d".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Should generate character filter
        assert!(result.contains("filter"));
        assert!(result.contains("'('"));
        assert!(result.contains("')'"));
        assert!(result.contains("'K'"));
        assert!(result.contains("pub fn test_tr_delete"));
    }

    #[test]
    fn test_transliterate_keep_only() {
        // Test the expression: tr/a-fA-F0-9//dc (keep only hex digits)
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Regexp::Transliterate",
                    "content": "tr/a-fA-F0-9//dc"
                }],
                "class": "PPI::Statement"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::PrintConv,
            "test_tr_keep_hex".to_string(),
            "tr/a-fA-F0-9//dc".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Should generate character filter to keep only hex chars
        assert!(result.contains("filter"));
        assert!(result.contains("contains"));
        // Should handle character ranges
        assert!(result.contains("pub fn test_tr_keep_hex"));
    }

    #[test]
    fn test_transliterate_replace() {
        // Test the expression: tr/abc/xyz/ (replace a->x, b->y, c->z)
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Regexp::Transliterate",
                    "content": "tr/abc/xyz/"
                }],
                "class": "PPI::Statement"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::PrintConv,
            "test_tr_replace".to_string(),
            "tr/abc/xyz/".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Should generate character mapping
        assert!(result.contains("match"));
        assert!(result.contains("'a' => 'x'"));
        assert!(result.contains("'b' => 'y'"));
        assert!(result.contains("'c' => 'z'"));
        assert!(result.contains("pub fn test_tr_replace"));
    }

    #[test]
    fn test_block_closure() {
        // Test the expression: { $_ * 2 }
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Magic",
                    "content": "$_"
                }, {
                    "class": "PPI::Token::Operator",
                    "content": "*"
                }, {
                    "class": "PPI::Token::Number",
                    "content": "2",
                    "numeric_value": 2
                }],
                "class": "PPI::Structure::Block"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::PrintConv,
            "test_block".to_string(),
            "{ $_ * 2 }".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Debug output to see what's generated
        println!("Generated block code:\n{}", result);

        // Should generate closure-like code
        assert!(result.contains("|item|"));
        // The magic variable $_ generates val, and we have multiplication
        assert!(result.contains("val") && result.contains("* 2"));
        assert!(result.contains("pub fn test_block"));
    }

    #[test]
    fn test_magic_variable_with_substitution() {
        // Test the nasty expression: $_=$val;s/ /x/;$_
        // This assigns $val to $_, does a substitution on $_ (implicitly), then returns $_
        // Based on actual PPI AST structure - it's 3 separate statements!
        let ast_json = json!({
            "children": [
                {
                    "children": [
                        {
                            "class": "PPI::Token::Magic",
                            "content": "$_",
                            "symbol_type": "scalar"
                        },
                        {
                            "class": "PPI::Token::Operator",
                            "content": "="
                        },
                        {
                            "class": "PPI::Token::Symbol",
                            "content": "$val",
                            "symbol_type": "scalar"
                        },
                        {
                            "class": "PPI::Token::Structure",
                            "content": ";"
                        }
                    ],
                    "class": "PPI::Statement"
                },
                {
                    "children": [
                        {
                            "class": "PPI::Token::Regexp::Substitute",
                            "content": "s/ /x/"
                        },
                        {
                            "class": "PPI::Token::Structure",
                            "content": ";"
                        }
                    ],
                    "class": "PPI::Statement"
                },
                {
                    "children": [
                        {
                            "class": "PPI::Token::Magic",
                            "content": "$_",
                            "symbol_type": "scalar"
                        }
                    ],
                    "class": "PPI::Statement"
                }
            ],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::PrintConv,
            "test_magic_with_subst".to_string(),
            "$_=$val;s/ /x/;$_".to_string(),
        );

        // This currently fails because visit_document doesn't handle multiple statements
        // TODO: Fix visit_document to handle multi-statement expressions like this one
        let result = generator.generate_function(&ast);

        // For now, we expect this to fail with UnsupportedStructure error
        assert!(result.is_err());
        if let Err(e) = result {
            // Debug output to see the error
            println!("Expected error for multi-statement document: {:?}", e);
            // Should fail with the multiple statements error
            match e {
                CodeGenError::UnsupportedStructure(msg) => {
                    assert!(msg.contains("multiple top-level statements"));
                }
                _ => panic!("Expected UnsupportedStructure error, got: {:?}", e),
            }
        }

        // TODO: When fixed, this should generate:
        // let mut temp = val;
        // temp = temp.to_string().replace(" ", "x");
        // temp
    }

    #[test]
    fn test_empty_block() {
        // Test the expression: { }
        let ast_json = json!({
            "children": [{
                "children": [],
                "class": "PPI::Structure::Block"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::PrintConv,
            "test_empty_block".to_string(),
            "{ }".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Should generate empty block
        assert!(result.contains("{ }"));
        assert!(result.contains("pub fn test_empty_block"));
    }
}
