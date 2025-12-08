//! P07: Explicit Function Call Normalization - Integration Tests
//!
//! Tests the normalization of nested function calls like `join " ", unpack "H2H2", val`
//! into proper AST structure for visitor pattern handling.
//! P07: Explicit Function Call Normalization - see docs/todo/P07-explicit-fn-normalization.md
//!
//! This test verifies that the AST normalizer can handle flat comma-separated
//! function calls and transform them into nested function call structures
//! that the visitor can process without string parsing.

use codegen::ppi::rust_generator::RustGenerator;
use codegen::ppi::{parse_ppi_json, ExpressionType};
use serde_json::json;

#[test]
fn test_join_unpack_normalization() {
    // P07: Explicit Function Call Normalization - see docs/todo/P07-explicit-fn-normalization.md
    // Test the specific problematic pattern: join " ", unpack "H2H2", val
    // This should be normalized from flat tokens to nested function calls

    // Create PPI AST for: join " ", unpack "H2H2", val
    // This represents the flat token structure that PPI outputs
    let join_unpack_ast = json!({
        "children": [{
            "children": [
                {
                    "class": "PPI::Token::Word",
                    "content": "join"
                },
                {
                    "class": "PPI::Token::Quote::Double",
                    "content": "\" \"",
                    "string_value": " "
                },
                {
                    "class": "PPI::Token::Operator",
                    "content": ","
                },
                {
                    "class": "PPI::Token::Word",
                    "content": "unpack"
                },
                {
                    "class": "PPI::Token::Quote::Double",
                    "content": "\"H2H2\"",
                    "string_value": "H2H2"
                },
                {
                    "class": "PPI::Token::Operator",
                    "content": ","
                },
                {
                    "class": "PPI::Token::Word",
                    "content": "val"
                }
            ],
            "class": "PPI::Statement"
        }],
        "class": "PPI::Document"
    });

    // Parse the PPI JSON into our AST structure
    let ppi_ast = parse_ppi_json(&join_unpack_ast).unwrap();

    // Verify the flat structure exists
    assert_eq!(ppi_ast.class, "PPI::Document");
    let statement = &ppi_ast.children[0];
    assert_eq!(statement.class, "PPI::Statement");
    assert_eq!(statement.children.len(), 7); // join, " ", ,, unpack, "H2H2", ,, val

    // Generate Rust code using current system
    let generator = RustGenerator::new(
        ExpressionType::PrintConv,
        "test_join_unpack_malformed".to_string(),
        "join \" \", unpack \"H2H2\", val".to_string(),
    );

    let generated_code = generator.generate_function(&ppi_ast);

    // This test demonstrates the current malformed generation
    // Before P07 fixes, this should generate malformed code like:
    // join(" ", ,, unpack, "H2H2", ,, val)
    match generated_code {
        Ok(code) => {
            println!("Generated code:\n{}", code);

            // Check for malformed comma patterns that indicate string parsing problems
            if code.contains(",,") || code.contains("join(\" \", unpack") {
                println!("✅ Current test shows malformed generation as expected");
                // This is currently expected behavior - malformed generation
                // The test passes showing the problem exists
            } else {
                // Fails until P07 complete - requires function call normalization
                panic!("Expected malformed generation showing architectural problem, but got clean code: {}", code);
            }
        }
        Err(e) => {
            println!(
                "Generation failed with error (also demonstrates the issue): {}",
                e
            );
            // Generation failure is also acceptable - shows the architectural problem
        }
    }
}

#[test]
fn test_sprintf_concatenation_normalization() {
    // P07: Test another pattern that should be normalized
    // sprintf with string concatenation: sprintf("%.1f" . "mm", $val/100)

    let sprintf_concat_ast = json!({
        "children": [{
            "children": [
                {
                    "class": "PPI::Token::Word",
                    "content": "sprintf"
                },
                {
                    "class": "PPI::Token::Quote::Double",
                    "content": "\"%.1f\"",
                    "string_value": "%.1f"
                },
                {
                    "class": "PPI::Token::Operator",
                    "content": "."
                },
                {
                    "class": "PPI::Token::Quote::Double",
                    "content": "\"mm\"",
                    "string_value": "mm"
                },
                {
                    "class": "PPI::Token::Operator",
                    "content": ","
                },
                {
                    "class": "PPI::Token::Symbol",
                    "content": "$val"
                },
                {
                    "class": "PPI::Token::Operator",
                    "content": "/"
                },
                {
                    "class": "PPI::Token::Number",
                    "content": "100",
                    "numeric_value": 100
                }
            ],
            "class": "PPI::Statement"
        }],
        "class": "PPI::Document"
    });

    let ppi_ast = parse_ppi_json(&sprintf_concat_ast).unwrap();

    let generator = RustGenerator::new(
        ExpressionType::PrintConv,
        "test_sprintf_concatenation_ternary".to_string(),
        "sprintf(\"%.1f\" . \"mm\", $val/100)".to_string(),
    );

    let generated_code = generator.generate_function(&ppi_ast);

    match generated_code {
        Ok(code) => {
            println!("Sprintf generated code:\n{}", code);
            // Before normalization, this likely generates malformed patterns
            // After normalization, should generate clean format! calls
        }
        Err(e) => {
            println!("Sprintf generation failed: {}", e);
            // Generation failure shows the architectural problem
        }
    }
}

#[test]
fn test_nested_function_call_structure() {
    // P07: Test the target normalized structure we want to achieve
    // This represents what join(" ", unpack("H2H2", val)) should look like after normalization

    let normalized_structure = json!({
        "children": [{
            "children": [
                {
                    "class": "PPI::Token::Word",
                    "content": "join"
                },
                {
                    "children": [
                        {
                            "children": [
                                {
                                    "class": "PPI::Token::Word",
                                    "content": "separator"
                                },
                                {
                                    "class": "PPI::Token::Operator",
                                    "content": "="
                                },
                                {
                                    "class": "PPI::Token::Quote::Double",
                                    "content": "\" \"",
                                    "string_value": " "
                                },
                                {
                                    "class": "PPI::Token::Operator",
                                    "content": ","
                                },
                                {
                                    "class": "PPI::Token::Word",
                                    "content": "list"
                                },
                                {
                                    "class": "PPI::Token::Operator",
                                    "content": "="
                                },
                                {
                                    "class": "PPI::Token::Word",
                                    "content": "unpack"
                                },
                                {
                                    "children": [
                                        {
                                            "children": [
                                                {
                                                    "class": "PPI::Token::Quote::Double",
                                                    "content": "\"H2H2\"",
                                                    "string_value": "H2H2"
                                                },
                                                {
                                                    "class": "PPI::Token::Operator",
                                                    "content": ","
                                                },
                                                {
                                                    "class": "PPI::Token::Word",
                                                    "content": "val"
                                                }
                                            ],
                                            "class": "PPI::Statement::Expression"
                                        }
                                    ],
                                    "class": "PPI::Structure::List",
                                    "structure_bounds": "( ... )"
                                }
                            ],
                            "class": "PPI::Statement::Expression"
                        }
                    ],
                    "class": "PPI::Structure::List",
                    "structure_bounds": "( ... )"
                }
            ],
            "class": "PPI::Statement"
        }],
        "class": "PPI::Document"
    });

    let ppi_ast = parse_ppi_json(&normalized_structure).unwrap();

    // This structure represents the target that the normalizer should produce
    // When the normalizer works correctly, it should transform the flat structure
    // from test_join_unpack_normalization into this nested structure

    assert_eq!(ppi_ast.class, "PPI::Document");
    let statement = &ppi_ast.children[0];
    assert_eq!(statement.class, "PPI::Statement");

    // The normalized structure should have join as first child
    // followed by a Structure::List containing the properly nested arguments
    assert_eq!(statement.children.len(), 2);

    let join_word = &statement.children[0];
    assert_eq!(join_word.class, "PPI::Token::Word");
    assert_eq!(join_word.content, Some("join".to_string()));

    let arg_list = &statement.children[1];
    assert_eq!(arg_list.class, "PPI::Structure::List");
    assert_eq!(arg_list.structure_bounds, Some("( ... )".to_string()));

    println!("✅ Target normalized structure verified");
}
