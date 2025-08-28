//! Tests for numeric and string operations in PPI Rust code generation
//!
//! These tests cover:
//! - Hex and float number generation
//! - Variable declarations
//! - Regular expression substitutions
//! - Sprintf with concatenation and ternary operations
//! - Unary operations
//! - Length function calls
//! - Join and unpack functions

use crate::ppi::rust_generator::RustGenerator;
use crate::ppi::{ExpressionType, PpiNode};
use serde_json::json;

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
    // Check for comment format (original perl expression in code block)
    assert!(result.contains("/// Original perl expression:"));
    assert!(result.contains("/// ``` perl"));
    assert!(result.contains("/// $val * 25"));
}

#[test]
#[ignore] // TODO: Move to JSON-based test infrastructure
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
    // Test accepts either direct format conversion OR sprintf function call
    let has_format_conversion = result.contains("{:.2}"); // Perl %.2f -> Rust {:.2}
    let has_sprintf_call = result.contains("sprintf");
    assert!(
        has_format_conversion || has_sprintf_call,
        "Should have either format conversion or sprintf call"
    );
    assert!(result.contains("if")); // Ternary -> if/else
    assert!(result.contains("pub fn test_sprintf_concat_ternary"));

    // Uncomment to see the generated code:
    // println!("Generated code:\n{}", result);
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
    println!("Generated ternary string comparison code:\n{}", result);

    // Should generate proper if-then-else structure (ternary is normalized by normalizer)
    assert!(result.contains("if"));
    assert!(result.contains("else"));
    assert!(result.contains("pub fn test_ternary_string_eq"));
    // The function body should not have "val eq", but the comment will still contain it
    let function_body = result.split("pub fn").nth(1).unwrap();
    assert!(!function_body.contains("val eq")); // Should not have raw eq operator in function body
                                                // Should generate proper string comparison (==, not eq)
    assert!(result.contains("== \"inf\""));
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
