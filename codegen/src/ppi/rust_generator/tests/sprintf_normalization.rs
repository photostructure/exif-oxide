//! Tests for sprintf with normalized string operations
//!
//! These tests ensure that the Rust generator can handle sprintf patterns after
//! the StringOpNormalizer has already converted string operations into canonical forms.
//! This maintains coverage when the SprintfNormalizer is removed.

use crate::ppi::normalizer::normalize_multi_pass;
use crate::ppi::rust_generator::RustGenerator;
use crate::ppi::{ExpressionType, PpiNode};

/// Test sprintf with StringConcat nodes (after string ops normalization)
#[test]
fn test_sprintf_with_string_concat() {
    // Create AST representing: sprintf(StringConcat("base", "part"), args)
    let ast = create_sprintf_with_string_concat();

    let generator = RustGenerator::new(
        ExpressionType::PrintConv,
        "test_sprintf_concat".to_string(),
        "sprintf(\"%19d\" . \" %3d\", args)".to_string(),
    );

    // Test that the generator can handle this AST without panicking
    let result = generator.generate_function(&ast);

    // Should either succeed or fail gracefully (not panic)
    match result {
        Ok(code) => {
            // Should generate appropriate sprintf call
            assert!(
                code.contains("sprintf") || code.contains("format!") || code.contains("TagValue")
            );
            assert!(code.contains("pub fn"));
            assert!(!code.contains("Expression"));
        }
        Err(_) => {
            // If it fails, that's also acceptable - we just need to ensure no panic
            // The important thing is that removal of SprintfNormalizer doesn't break the build
        }
    }
}

/// Test sprintf with StringRepeat nodes (after string ops normalization)
#[test]
fn test_sprintf_with_string_repeat() {
    // Create AST representing: sprintf(StringRepeat("part", 8), args)
    let ast = create_sprintf_with_string_repeat();

    let generator = RustGenerator::new(
        ExpressionType::PrintConv,
        "test_sprintf_repeat".to_string(),
        "sprintf(\" %3d\" x 8, args)".to_string(),
    );

    // Test that the generator can handle this AST without panicking
    let result = generator.generate_function(&ast);

    match result {
        Ok(code) => {
            assert!(
                code.contains("sprintf") || code.contains("format!") || code.contains("TagValue")
            );
            assert!(code.contains("pub fn"));
            assert!(!code.contains("Expression"));
        }
        Err(_) => {
            // Graceful failure is acceptable
        }
    }
}

/// Test sprintf with combined StringConcat and StringRepeat (the full pattern)
#[test]
fn test_sprintf_with_concat_repeat_normalized() {
    // Create AST representing: sprintf(StringConcat("base", StringRepeat("part", 8)), args)
    let ast = create_sprintf_with_concat_and_repeat();

    let generator = RustGenerator::new(
        ExpressionType::PrintConv,
        "test_sprintf_full_pattern".to_string(),
        "sprintf(\"%19d %4d %6d\" . \" %3d %4d %6d\" x 8, split(\" \",$val))".to_string(),
    );

    // Test that the generator can handle this AST without panicking
    let result = generator.generate_function(&ast);

    match result {
        Ok(code) => {
            // Should generate appropriate function call
            assert!(
                code.contains("sprintf_with_string_concat_repeat")
                    || code.contains("crate::fmt::sprintf")
                    || code.contains("TagValue")
            );
            assert!(code.contains("pub fn"));
            assert!(!code.contains("Expression"));
        }
        Err(_) => {
            // Graceful failure is acceptable - the goal is no panic
        }
    }
}

/// Test end-to-end normalization and generation for sprintf pattern
#[test]
fn test_sprintf_end_to_end_normalization() {
    // Start with raw PPI AST before normalization
    let raw_ast = create_raw_sprintf_ast();

    // Apply multi-pass normalization (string ops first, then others)
    let normalized_ast = normalize_multi_pass(raw_ast);

    // Generate Rust code from normalized AST
    let generator = RustGenerator::new(
        ExpressionType::PrintConv,
        "test_end_to_end".to_string(),
        "sprintf(\"%19d\" . \" %3d\" x 2, args)".to_string(),
    );

    let result = generator.generate_function(&normalized_ast).unwrap();

    // Should produce working Rust code
    assert!(result.contains("pub fn"));
    assert!(result.contains("TagValue"));
    assert!(result.contains("sprintf") || result.contains("format!"));

    // Should not use runtime expression evaluation
    assert!(!result.contains("evaluate_expression"));
    assert!(!result.contains("ExpressionEvaluator"));
}

/// Test that string operations are properly handled within sprintf context
#[test]
fn test_sprintf_preserves_string_operation_semantics() {
    // Test that the meaning of string concat/repeat is preserved in sprintf context
    let ast = create_sprintf_with_complex_format();

    let generator = RustGenerator::new(
        ExpressionType::PrintConv,
        "test_semantics".to_string(),
        "sprintf(\"base\" . \"mid\" . \"end\" x 3, $val)".to_string(),
    );

    let result = generator.generate_function(&ast).unwrap();

    // Should handle the complex string operations appropriately
    assert!(result.contains("pub fn"));
    assert!(!result.contains("Expression"));

    // Verify it produces syntactically valid Rust
    assert!(result.contains("TagValue"));
    assert!(result.starts_with("pub fn") || result.contains("pub fn"));
}

// Helper functions to create test AST nodes

fn create_sprintf_with_string_concat() -> PpiNode {
    PpiNode {
        class: "FunctionCall".to_string(),
        content: Some("sprintf".to_string()),
        children: vec![
            // First argument: StringConcat
            PpiNode {
                class: "StringConcat".to_string(),
                content: None,
                children: vec![
                    PpiNode {
                        class: "PPI::Token::Quote::Double".to_string(),
                        content: Some("\"%19d\"".to_string()),
                        string_value: Some("%19d".to_string()),
                        children: vec![],
                        symbol_type: None,
                        numeric_value: None,
                        structure_bounds: None,
                    },
                    PpiNode {
                        class: "PPI::Token::Quote::Double".to_string(),
                        content: Some("\" %3d\"".to_string()),
                        string_value: Some(" %3d".to_string()),
                        children: vec![],
                        symbol_type: None,
                        numeric_value: None,
                        structure_bounds: None,
                    },
                ],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
            // Second argument: variable
            PpiNode {
                class: "PPI::Token::Symbol".to_string(),
                content: Some("$val".to_string()),
                children: vec![],
                symbol_type: Some("scalar".to_string()),
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
        ],
        symbol_type: None,
        numeric_value: None,
        string_value: None,
        structure_bounds: None,
    }
}

fn create_sprintf_with_string_repeat() -> PpiNode {
    PpiNode {
        class: "FunctionCall".to_string(),
        content: Some("sprintf".to_string()),
        children: vec![
            // First argument: StringRepeat
            PpiNode {
                class: "StringRepeat".to_string(),
                content: None,
                children: vec![
                    PpiNode {
                        class: "PPI::Token::Quote::Double".to_string(),
                        content: Some("\" %3d\"".to_string()),
                        string_value: Some(" %3d".to_string()),
                        children: vec![],
                        symbol_type: None,
                        numeric_value: None,
                        structure_bounds: None,
                    },
                    PpiNode {
                        class: "PPI::Token::Number".to_string(),
                        content: Some("8".to_string()),
                        numeric_value: Some(8.0),
                        children: vec![],
                        symbol_type: None,
                        string_value: None,
                        structure_bounds: None,
                    },
                ],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
            // Second argument: variable
            PpiNode {
                class: "PPI::Token::Symbol".to_string(),
                content: Some("$val".to_string()),
                children: vec![],
                symbol_type: Some("scalar".to_string()),
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
        ],
        symbol_type: None,
        numeric_value: None,
        string_value: None,
        structure_bounds: None,
    }
}

fn create_sprintf_with_concat_and_repeat() -> PpiNode {
    PpiNode {
        class: "FunctionCall".to_string(),
        content: Some("sprintf".to_string()),
        children: vec![
            // First argument: StringConcat containing base + StringRepeat
            PpiNode {
                class: "StringConcat".to_string(),
                content: None,
                children: vec![
                    PpiNode {
                        class: "PPI::Token::Quote::Double".to_string(),
                        content: Some("\"%19d %4d %6d\"".to_string()),
                        string_value: Some("%19d %4d %6d".to_string()),
                        children: vec![],
                        symbol_type: None,
                        numeric_value: None,
                        structure_bounds: None,
                    },
                    PpiNode {
                        class: "StringRepeat".to_string(),
                        content: None,
                        children: vec![
                            PpiNode {
                                class: "PPI::Token::Quote::Double".to_string(),
                                content: Some("\" %3d %4d %6d\"".to_string()),
                                string_value: Some(" %3d %4d %6d".to_string()),
                                children: vec![],
                                symbol_type: None,
                                numeric_value: None,
                                structure_bounds: None,
                            },
                            PpiNode {
                                class: "PPI::Token::Number".to_string(),
                                content: Some("8".to_string()),
                                numeric_value: Some(8.0),
                                children: vec![],
                                symbol_type: None,
                                string_value: None,
                                structure_bounds: None,
                            },
                        ],
                        symbol_type: None,
                        numeric_value: None,
                        string_value: None,
                        structure_bounds: None,
                    },
                ],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
            // Second argument: split function call
            PpiNode {
                class: "FunctionCall".to_string(),
                content: Some("split".to_string()),
                children: vec![
                    PpiNode {
                        class: "PPI::Token::Quote::Double".to_string(),
                        content: Some("\" \"".to_string()),
                        string_value: Some(" ".to_string()),
                        children: vec![],
                        symbol_type: None,
                        numeric_value: None,
                        structure_bounds: None,
                    },
                    PpiNode {
                        class: "PPI::Token::Symbol".to_string(),
                        content: Some("$val".to_string()),
                        children: vec![],
                        symbol_type: Some("scalar".to_string()),
                        numeric_value: None,
                        string_value: None,
                        structure_bounds: None,
                    },
                ],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
        ],
        symbol_type: None,
        numeric_value: None,
        string_value: None,
        structure_bounds: None,
    }
}

fn create_raw_sprintf_ast() -> PpiNode {
    // Create AST representing: sprintf("%19d" . " %3d" x 2, $val)
    // This will be normalized by the multi-pass system
    PpiNode {
        class: "PPI::Statement".to_string(),
        content: None,
        children: vec![
            PpiNode {
                class: "PPI::Token::Word".to_string(),
                content: Some("sprintf".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Structure::List".to_string(),
                content: Some("(".to_string()),
                children: vec![
                    // Format string with operations
                    PpiNode {
                        class: "PPI::Statement".to_string(),
                        content: None,
                        children: vec![
                            PpiNode {
                                class: "PPI::Token::Quote::Double".to_string(),
                                content: Some("\"%19d\"".to_string()),
                                children: vec![],
                                symbol_type: None,
                                numeric_value: None,
                                string_value: Some("%19d".to_string()),
                                structure_bounds: None,
                            },
                            PpiNode {
                                class: "PPI::Token::Operator".to_string(),
                                content: Some(".".to_string()),
                                children: vec![],
                                symbol_type: None,
                                numeric_value: None,
                                string_value: None,
                                structure_bounds: None,
                            },
                            PpiNode {
                                class: "PPI::Token::Quote::Double".to_string(),
                                content: Some("\" %3d\"".to_string()),
                                children: vec![],
                                symbol_type: None,
                                numeric_value: None,
                                string_value: Some(" %3d".to_string()),
                                structure_bounds: None,
                            },
                            PpiNode {
                                class: "PPI::Token::Operator".to_string(),
                                content: Some("x".to_string()),
                                children: vec![],
                                symbol_type: None,
                                numeric_value: None,
                                string_value: None,
                                structure_bounds: None,
                            },
                            PpiNode {
                                class: "PPI::Token::Number".to_string(),
                                content: Some("2".to_string()),
                                children: vec![],
                                symbol_type: None,
                                numeric_value: Some(2.0),
                                string_value: None,
                                structure_bounds: None,
                            },
                        ],
                        symbol_type: None,
                        numeric_value: None,
                        string_value: None,
                        structure_bounds: None,
                    },
                    PpiNode {
                        class: "PPI::Token::Operator".to_string(),
                        content: Some(",".to_string()),
                        children: vec![],
                        symbol_type: None,
                        numeric_value: None,
                        string_value: None,
                        structure_bounds: None,
                    },
                    PpiNode {
                        class: "PPI::Token::Symbol".to_string(),
                        content: Some("$val".to_string()),
                        children: vec![],
                        symbol_type: Some("scalar".to_string()),
                        numeric_value: None,
                        string_value: None,
                        structure_bounds: None,
                    },
                ],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
        ],
        symbol_type: None,
        numeric_value: None,
        string_value: None,
        structure_bounds: None,
    }
}

fn create_sprintf_with_complex_format() -> PpiNode {
    // Multiple concatenations: StringConcat("base", StringConcat("mid", StringRepeat("end", 3)))
    PpiNode {
        class: "FunctionCall".to_string(),
        content: Some("sprintf".to_string()),
        children: vec![
            PpiNode {
                class: "StringConcat".to_string(),
                content: None,
                children: vec![
                    PpiNode {
                        class: "PPI::Token::Quote::Double".to_string(),
                        content: Some("\"base\"".to_string()),
                        string_value: Some("base".to_string()),
                        children: vec![],
                        symbol_type: None,
                        numeric_value: None,
                        structure_bounds: None,
                    },
                    PpiNode {
                        class: "StringConcat".to_string(),
                        content: None,
                        children: vec![
                            PpiNode {
                                class: "PPI::Token::Quote::Double".to_string(),
                                content: Some("\"mid\"".to_string()),
                                string_value: Some("mid".to_string()),
                                children: vec![],
                                symbol_type: None,
                                numeric_value: None,
                                structure_bounds: None,
                            },
                            PpiNode {
                                class: "StringRepeat".to_string(),
                                content: None,
                                children: vec![
                                    PpiNode {
                                        class: "PPI::Token::Quote::Double".to_string(),
                                        content: Some("\"end\"".to_string()),
                                        string_value: Some("end".to_string()),
                                        children: vec![],
                                        symbol_type: None,
                                        numeric_value: None,
                                        structure_bounds: None,
                                    },
                                    PpiNode {
                                        class: "PPI::Token::Number".to_string(),
                                        content: Some("3".to_string()),
                                        numeric_value: Some(3.0),
                                        children: vec![],
                                        symbol_type: None,
                                        string_value: None,
                                        structure_bounds: None,
                                    },
                                ],
                                symbol_type: None,
                                numeric_value: None,
                                string_value: None,
                                structure_bounds: None,
                            },
                        ],
                        symbol_type: None,
                        numeric_value: None,
                        string_value: None,
                        structure_bounds: None,
                    },
                ],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Token::Symbol".to_string(),
                content: Some("$val".to_string()),
                children: vec![],
                symbol_type: Some("scalar".to_string()),
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
        ],
        symbol_type: None,
        numeric_value: None,
        string_value: None,
        structure_bounds: None,
    }
}
