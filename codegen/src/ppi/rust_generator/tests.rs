//! Tests for PPI Rust code generation
//!
//! These tests validate that the PPI AST parsing and Rust code generation
//! produces correct, compilable Rust code that matches ExifTool semantics.

#[cfg(test)]
mod tests {
    use crate::ppi::rust_generator::expressions::ExpressionCombiner;
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
        println!("Generated length function result:\n{}", result);

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

    // Phase 1.1: Pack Pattern Recognition Tests

    #[test]
    fn test_pack_c_star_bit_extraction_pattern() {
        // Test the specific pattern from ExifTool: pack "C*", map { (($val>>$_)&0x1f)+0x60 } 10, 5, 0
        let generator = RustGenerator::new(
            ExpressionType::ValueConv,
            "test_pack_bit_extract".to_string(),
            "pack \"C*\", map { (($val>>$_)&0x1f)+0x60 } 10, 5, 0".to_string(),
        );

        // Simulate the parts that would be extracted from this expression by combine_statement_parts
        let parts = vec![
            "pack".to_string(),
            "\"C*\"".to_string(),
            ",".to_string(),
            "map".to_string(),
            "{".to_string(),
            "(".to_string(),
            "(".to_string(),
            "$val".to_string(),
            ">>".to_string(),
            "$_".to_string(),
            ")".to_string(),
            "&".to_string(),
            "0x1f".to_string(), // This should be detected as mask
            ")".to_string(),
            "+".to_string(),
            "0x60".to_string(), // This should be detected as offset
            "}".to_string(),
            ",".to_string(),
            "10".to_string(), // These should be detected as shifts
            ",".to_string(),
            "5".to_string(),
            ",".to_string(),
            "0".to_string(),
        ];

        // Create dummy PpiNode children for the method signature
        let children: Vec<PpiNode> = vec![];

        match generator.combine_statement_parts(&parts, &children) {
            Ok(result) => {
                println!("Generated pack pattern code: {}", result);

                // Verify the result contains our helper function call
                assert!(result.contains("crate::fmt::pack_c_star_bit_extract"));
                assert!(result.contains("val"));
                assert!(result.contains("[10, 5, 0]")); // The shift values
                assert!(result.contains("31")); // 0x1f = 31 (mask)
                assert!(result.contains("96")); // 0x60 = 96 (offset)
            }
            Err(e) => panic!("Pack pattern recognition failed: {:?}", e),
        }
    }

    #[test]
    fn test_pack_c_star_fallback_pattern() {
        // Test fallback when mask/offset aren't clearly detected
        let generator = RustGenerator::new(
            ExpressionType::ValueConv,
            "test_pack_fallback".to_string(),
            "pack \"C*\", map { ... } 8, 4, 0".to_string(),
        );

        let parts = vec![
            "pack".to_string(),
            "\"C*\"".to_string(),
            ",".to_string(),
            "map".to_string(),
            "{".to_string(),
            "complex_expression".to_string(),
            "}".to_string(),
            ",".to_string(),
            "8".to_string(),
            ",".to_string(),
            "4".to_string(),
            ",".to_string(),
            "0".to_string(),
        ];

        let children: Vec<PpiNode> = vec![];

        match generator.combine_statement_parts(&parts, &children) {
            Ok(result) => {
                println!("Fallback pack pattern code: {}", result);

                // When pattern extraction fails, should still use the hardcoded fallback values
                assert!(result.contains("crate::fmt::pack_c_star_bit_extract"));
                assert!(result.contains("[8, 4, 0]")); // The shift values
                assert!(result.contains("31")); // Default mask 0x1f
                assert!(result.contains("96")); // Default offset 0x60
            }
            Err(e) => panic!("Fallback pack pattern recognition failed: {:?}", e),
        }
    }

    #[test]
    fn test_pack_c_star_different_mask_offset() {
        // Test pattern with different mask and offset values
        let generator = RustGenerator::new(
            ExpressionType::ValueConv,
            "test_pack_different_values".to_string(),
            "pack \"C*\", map { (($val>>$_)&0x0f)+0x30 } 12, 8, 4, 0".to_string(),
        );

        let parts = vec![
            "pack".to_string(),
            "\"C*\"".to_string(),
            ",".to_string(),
            "map".to_string(),
            "{".to_string(),
            "(".to_string(),
            "(".to_string(),
            "$val".to_string(),
            ">>".to_string(),
            "$_".to_string(),
            ")".to_string(),
            "&".to_string(),
            "0x0f".to_string(), // Different mask (15)
            ")".to_string(),
            "+".to_string(),
            "0x30".to_string(), // Different offset (48)
            "}".to_string(),
            ",".to_string(),
            "12".to_string(), // Different shift values
            ",".to_string(),
            "8".to_string(),
            ",".to_string(),
            "4".to_string(),
            ",".to_string(),
            "0".to_string(),
        ];

        let children: Vec<PpiNode> = vec![];

        match generator.combine_statement_parts(&parts, &children) {
            Ok(result) => {
                println!("Different values pack pattern code: {}", result);

                assert!(result.contains("crate::fmt::pack_c_star_bit_extract"));
                assert!(result.contains("[12, 8, 4, 0]")); // The shift values
                assert!(result.contains("15")); // 0x0f = 15 (mask)
                assert!(result.contains("48")); // 0x30 = 48 (offset)
            }
            Err(e) => panic!("Different values pack pattern recognition failed: {:?}", e),
        }
    }

    #[test]
    fn test_extract_pack_map_pattern_method() {
        // Test the helper method directly
        let generator = RustGenerator::new(
            ExpressionType::ValueConv,
            "test_extraction".to_string(),
            "test".to_string(),
        );

        // Test successful extraction
        let parts_with_pattern = vec![
            "pack".to_string(),
            "\"C*\"".to_string(),
            "map".to_string(),
            "0x1f".to_string(), // mask
            "0x60".to_string(), // offset
            "10".to_string(),   // shifts
            "5".to_string(),
            "0".to_string(),
        ];

        let children: Vec<PpiNode> = vec![];

        match generator.extract_pack_map_pattern(&parts_with_pattern, &children) {
            Ok(Some((mask, offset, shifts))) => {
                assert_eq!(mask, 31); // 0x1f
                assert_eq!(offset, 96); // 0x60
                assert_eq!(shifts, vec![10, 5, 0]);
            }
            Ok(None) => panic!("Should have extracted pattern"),
            Err(e) => panic!("Extraction failed: {:?}", e),
        }

        // Test no pattern found
        let parts_no_pattern = vec!["join".to_string(), "\" \"".to_string(), "split".to_string()];

        match generator.extract_pack_map_pattern(&parts_no_pattern, &children) {
            Ok(None) => {
                // Expected: no pattern found
            }
            Ok(Some(_)) => panic!("Should not have found pattern in non-pack expression"),
            Err(e) => panic!("Extraction should not error on non-pattern: {:?}", e),
        }
    }

    // Phase 1.2: Join+Unpack Pattern Recognition Tests

    #[test]
    fn test_join_unpack_pattern() {
        // Test the pattern: join " ", unpack "H2H2", val
        let generator = RustGenerator::new(
            ExpressionType::PrintConv,
            "test_join_unpack".to_string(),
            "join \" \", unpack \"H2H2\", val".to_string(),
        );

        // Simulate the parts that would be extracted from this expression
        let parts = vec![
            "join".to_string(),
            "\" \"".to_string(), // separator
            ",".to_string(),
            "unpack".to_string(),
            "\"H2H2\"".to_string(), // format
            ",".to_string(),
            "val".to_string(), // data
        ];

        let children: Vec<PpiNode> = vec![];

        match generator.combine_statement_parts(&parts, &children) {
            Ok(result) => {
                println!("Generated join+unpack code: {}", result);

                // Verify it uses the join_unpack_binary helper
                assert!(result.contains("crate::fmt::join_unpack_binary"));
                assert!(result.contains("\" \"")); // separator
                assert!(result.contains("\"H2H2\"")); // format
                assert!(result.contains("val")); // data variable
            }
            Err(e) => panic!("Join+unpack pattern recognition failed: {:?}", e),
        }
    }

    #[test]
    fn test_join_unpack_different_separator() {
        // Test with different separator: join "-", unpack "C*", val
        let generator = RustGenerator::new(
            ExpressionType::PrintConv,
            "test_join_unpack_dash".to_string(),
            "join \"-\", unpack \"C*\", val".to_string(),
        );

        let parts = vec![
            "join".to_string(),
            "\"-\"".to_string(), // dash separator
            ",".to_string(),
            "unpack".to_string(),
            "\"C*\"".to_string(), // different format
            ",".to_string(),
            "val".to_string(),
        ];

        let children: Vec<PpiNode> = vec![];

        match generator.combine_statement_parts(&parts, &children) {
            Ok(result) => {
                println!("Generated join+unpack with dash separator: {}", result);

                assert!(result.contains("crate::fmt::join_unpack_binary"));
                assert!(result.contains("\"-\"")); // dash separator
                assert!(result.contains("\"C*\"")); // C* format
                assert!(result.contains("val"));
            }
            Err(e) => panic!("Join+unpack with dash separator failed: {:?}", e),
        }
    }

    #[test]
    fn test_join_unpack_complex_data() {
        // Test with more complex data expression: join " ", unpack "H2H2", $val[0]
        let generator = RustGenerator::new(
            ExpressionType::PrintConv,
            "test_join_unpack_complex".to_string(),
            "join \" \", unpack \"H2H2\", $val[0]".to_string(),
        );

        let parts = vec![
            "join".to_string(),
            "\" \"".to_string(),
            ",".to_string(),
            "unpack".to_string(),
            "\"H2H2\"".to_string(),
            ",".to_string(),
            "$val[0]".to_string(), // more complex data reference
        ];

        let children: Vec<PpiNode> = vec![];

        match generator.combine_statement_parts(&parts, &children) {
            Ok(result) => {
                println!("Generated join+unpack with complex data: {}", result);

                assert!(result.contains("crate::fmt::join_unpack_binary"));
                assert!(result.contains("\" \""));
                assert!(result.contains("\"H2H2\""));
                assert!(result.contains("$val[0]"));
            }
            Err(e) => panic!("Join+unpack with complex data failed: {:?}", e),
        }
    }

    #[test]
    fn test_standalone_unpack_not_affected() {
        // Ensure standalone unpack calls still work and aren't affected by join+unpack pattern
        let generator = RustGenerator::new(
            ExpressionType::PrintConv,
            "test_standalone_unpack".to_string(),
            "unpack \"H2H2\", val".to_string(),
        );

        let parts = vec![
            "unpack".to_string(),
            "\"H2H2\"".to_string(),
            ",".to_string(),
            "val".to_string(),
        ];

        let children: Vec<PpiNode> = vec![];

        match generator.combine_statement_parts(&parts, &children) {
            Ok(result) => {
                println!("Generated standalone unpack code: {}", result);

                // Should use standalone unpack helper, not join+unpack
                assert!(result.contains("crate::fmt::unpack_binary"));
                assert!(!result.contains("join_unpack_binary"));
                assert!(result.contains("\"H2H2\""));
                assert!(result.contains("val"));
            }
            Err(e) => panic!("Standalone unpack pattern failed: {:?}", e),
        }
    }

    #[test]
    fn test_pack_map_pattern_extraction() {
        // Test the restored pack "C*", map { bit extraction } pattern
        // From ExifTool Canon.pm line 1847: pack "C*", map { (($_>>$_)&0x1f)+0x60 } 10, 5, 0
        let generator = RustGenerator::new(
            ExpressionType::ValueConv,
            "test_pack_map".to_string(),
            "pack \"C*\", map { ... } 10, 5, 0".to_string(),
        );

        let parts = vec![
            "pack".to_string(),
            "\"C*\"".to_string(),
            ",".to_string(),
            "map".to_string(),
            "{".to_string(),
            "...".to_string(),
            "}".to_string(),
            "0x1f".to_string(),
            "0x60".to_string(),
            "10".to_string(),
            "5".to_string(),
            "0".to_string(),
        ];

        let children = vec![]; // Empty for this test

        match generator.combine_statement_parts(&parts, &children) {
            Ok(result) => {
                // Should generate pack_c_star_bit_extract call
                assert!(result.contains("crate::fmt::pack_c_star_bit_extract"));
                assert!(result.contains("[10, 5, 0]")); // Shift values as array
                assert!(result.contains("31")); // Mask value (0x1f = 31)
                assert!(result.contains("96")); // Offset value (0x60 = 96)
            }
            Err(e) => panic!("Pack map pattern failed: {:?}", e),
        }
    }

    #[test]
    fn test_safe_division_pattern() {
        // Test the restored safe division pattern recognition
        // From ExifTool Canon.pm: $val ? 1/$val : 0
        let generator = RustGenerator::new(
            ExpressionType::ValueConv,
            "test_safe_reciprocal".to_string(),
            "$val ? 1/$val : 0".to_string(),
        );

        let parts = vec![
            "$val".to_string(),
            "?".to_string(),
            "1".to_string(),
            "/".to_string(),
            "$val".to_string(),
            ":".to_string(),
            "0".to_string(),
        ];

        let children = vec![]; // Empty for this test

        match generator.combine_statement_parts(&parts, &children) {
            Ok(result) => {
                // Should generate safe_reciprocal call
                assert!(result.contains("crate::fmt::safe_reciprocal"));
                assert!(result.contains("$val"));
            }
            Err(e) => panic!("Safe reciprocal pattern failed: {:?}", e),
        }
    }

    #[test]
    fn test_safe_division_with_numerator() {
        // Test safe division with custom numerator: $val ? 10/$val : 0
        let generator = RustGenerator::new(
            ExpressionType::ValueConv,
            "test_safe_division".to_string(),
            "$val ? 10/$val : 0".to_string(),
        );

        let parts = vec![
            "$val".to_string(),
            "?".to_string(),
            "10".to_string(),
            "/".to_string(),
            "$val".to_string(),
            ":".to_string(),
            "0".to_string(),
        ];

        let children = vec![]; // Empty for this test

        match generator.combine_statement_parts(&parts, &children) {
            Ok(result) => {
                // Should generate safe_division call with numerator
                assert!(result.contains("crate::fmt::safe_division"));
                assert!(result.contains("10.0"));
                assert!(result.contains("$val"));
            }
            Err(e) => panic!("Safe division pattern failed: {:?}", e),
        }
    }

    #[test]
    fn test_sprintf_with_string_operations() {
        // Test the restored sprintf with string concatenation pattern
        // From ExifTool Canon.pm: sprintf("%19d %4d %6d" . " %3d %4d %6d" x 8, args)
        let generator = RustGenerator::new(
            ExpressionType::PrintConv,
            "test_sprintf_concat".to_string(),
            "sprintf(\"%19d %4d %6d\" . \" %3d %4d %6d\" x 8, split(\" \",$val))".to_string(),
        );

        let parts = vec![
            "sprintf".to_string(),
            "(".to_string(),
            "\"%19d %4d %6d\"".to_string(),
            ".".to_string(),
            "\" %3d %4d %6d\"".to_string(),
            "x".to_string(),
            "8".to_string(),
            ",".to_string(),
            "split".to_string(),
            "(".to_string(),
            "\" \"".to_string(),
            ",".to_string(),
            "$val".to_string(),
            ")".to_string(),
            ")".to_string(),
        ];

        let children = vec![]; // Empty for this test

        match generator.combine_statement_parts(&parts, &children) {
            Ok(result) => {
                // Should generate sprintf_with_string function call
                assert!(
                    result.contains("sprintf_with_string")
                        || result.contains("crate::fmt::sprintf")
                );
            }
            Err(e) => panic!("Sprintf string operations pattern failed: {:?}", e),
        }
    }

    #[test]
    fn test_static_function_generation() {
        // Test that restored patterns generate proper static Function variants for P07 compliance
        // This is the critical test for Task E: P07 Static Function Generation Compliance

        // Test 1: Pack/map bit extraction pattern should generate static function
        let pack_map_generator = RustGenerator::new(
            ExpressionType::ValueConv,
            "test_pack_map_static".to_string(),
            "pack \"C*\", map { (($_>>$_)&0x1f)+0x60 } 10, 5, 0".to_string(),
        );

        let pack_parts = vec![
            "pack".to_string(),
            "\"C*\"".to_string(),
            ",".to_string(),
            "map".to_string(),
            "{".to_string(),
            "(($_>>$_)&0x1f)+0x60".to_string(),
            "}".to_string(),
            "10".to_string(),
            "5".to_string(),
            "0".to_string(),
        ];

        match pack_map_generator.combine_statement_parts(&pack_parts, &[]) {
            Ok(result) => {
                // Should generate static function call, not dynamic evaluation
                assert!(result.contains("pack_c_star_bit_extract"));
                assert!(!result.contains("Expression"));
                assert!(!result.contains("evaluate_expression"));

                // Should use compile-time constants for bit operations
                assert!(result.contains("10") && result.contains("5") && result.contains("0"));
            }
            Err(e) => panic!("Pack/map static function generation failed: {:?}", e),
        }

        // Test 2: Safe division should generate static function
        let safe_div_generator = RustGenerator::new(
            ExpressionType::ValueConv,
            "test_safe_division_static".to_string(),
            "$val ? 1/$val : 0".to_string(),
        );

        let safe_div_parts = vec![
            "$val".to_string(),
            "?".to_string(),
            "1".to_string(),
            "/".to_string(),
            "$val".to_string(),
            ":".to_string(),
            "0".to_string(),
        ];

        match safe_div_generator.combine_statement_parts(&safe_div_parts, &[]) {
            Ok(result) => {
                // Should generate static safe_reciprocal call
                assert!(result.contains("safe_reciprocal"));
                assert!(!result.contains("Expression"));
                assert!(!result.contains("runtime"));

                // Should use compile-time pattern recognition
                assert!(result.contains("crate::fmt::"));
            }
            Err(e) => panic!("Safe division static function generation failed: {:?}", e),
        }

        // Test 3: Complex sprintf should generate static function
        let sprintf_generator = RustGenerator::new(
            ExpressionType::PrintConv,
            "test_sprintf_static".to_string(),
            "sprintf(\"%19d %4d %6d\" . \" %3d %4d %6d\" x 8, split(\" \",$val))".to_string(),
        );

        let sprintf_parts = vec![
            "sprintf".to_string(),
            "(".to_string(),
            "\"%19d %4d %6d\"".to_string(),
            ".".to_string(),
            "\" %3d %4d %6d\"".to_string(),
            "x".to_string(),
            "8".to_string(),
            ",".to_string(),
            "split".to_string(),
            "(".to_string(),
            "\" \"".to_string(),
            ",".to_string(),
            "$val".to_string(),
            ")".to_string(),
            ")".to_string(),
        ];

        match sprintf_generator.combine_statement_parts(&sprintf_parts, &[]) {
            Ok(result) => {
                // Should generate static sprintf function with string operations
                assert!(
                    result.contains("sprintf_with_string")
                        || result.contains("crate::fmt::sprintf")
                );
                assert!(!result.contains("Expression"));
                assert!(!result.contains("evaluate_expression"));
            }
            Err(e) => panic!("Sprintf static function generation failed: {:?}", e),
        }

        // Test 4: Verify P07 compliance - static functions not runtime evaluation
        // All generated functions should produce compile-time Rust code, not runtime expression strings
        for generator in [pack_map_generator, safe_div_generator, sprintf_generator] {
            let ast_json = serde_json::json!({
                "children": [{
                    "children": [{
                        "class": "PPI::Token::Symbol",
                        "content": "$val",
                        "symbol_type": "scalar"
                    }],
                    "class": "PPI::Statement"
                }],
                "class": "PPI::Document"
            });

            let ast: crate::ppi::PpiNode = serde_json::from_value(ast_json).unwrap();
            let function_result = generator.generate_function(&ast).unwrap();

            // P07 compliance: Should generate static Rust function, not expression evaluator calls
            assert!(function_result.contains("pub fn"));
            assert!(function_result.contains("TagValue"));
            assert!(!function_result.contains("ExpressionEvaluator"));
            assert!(!function_result.contains("evaluate_expression"));

            // Should be pure static code generation
            assert!(!function_result.contains("runtime"));
            assert!(!function_result.contains("dynamic"));
        }
    }
}
