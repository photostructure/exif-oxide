//! Tests for PPI Rust code generation
//!
//! These tests validate that the PPI AST parsing and Rust code generation
//! produces correct, compilable Rust code that matches ExifTool semantics.

#[cfg(test)]
mod tests {
    use crate::ppi::RustGenerator;
    use crate::ppi::{ExpressionType, PpiNode};
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
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Word",
                    "content": "sprintf"
                }, {
                    "class": "PPI::Structure::List",
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
                }, {
                    "class": "PPI::Token::Operator",
                    "content": "."
                }, {
                    "class": "PPI::Structure::List",
                    "children": [{
                        "class": "PPI::Token::Symbol",
                        "content": "$val",
                        "symbol_type": "scalar"
                    }, {
                        "class": "PPI::Token::Operator",
                        "content": ">"
                    }, {
                        "class": "PPI::Token::Number",
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
}
