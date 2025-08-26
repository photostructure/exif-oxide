//! Tests for control flow and advanced features in PPI Rust code generation
//!
//! These tests cover:
//! - Magic variables ($_, $@)
//! - Control flow statements (return, last, next)
//! - Transliterate operations (tr///)
//! - Block structures and closures
//! - Multi-statement expressions

use crate::ppi::rust_generator::{CodeGenError, RustGenerator};
use crate::ppi::{ExpressionType, PpiNode};
use serde_json::json;

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

    // Check what happened - either error or invalid code generation
    match &result {
        Ok(code) => {
            // If code generation succeeded, check if the result is syntactically valid
            if code.contains("val = val ;;") || code.contains(";;") {
                // Invalid code generation - treat as failure
                println!("Generated invalid Rust code: {}", code);
                panic!("Code generation produced syntactically invalid Rust code");
            } else {
                println!("Unexpectedly generated valid code: {}", code);
                panic!("Expected this complex multi-statement expression to fail");
            }
        }
        Err(e) => {
            println!("Got expected error: {:?}", e);
            // Should fail with appropriate error
            match e {
                CodeGenError::UnsupportedStructure(_) => {
                    // This is expected
                }
                _ => {
                    // Other errors are also acceptable for this complex case
                    println!("Got different error type (also acceptable): {:?}", e);
                }
            }
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
