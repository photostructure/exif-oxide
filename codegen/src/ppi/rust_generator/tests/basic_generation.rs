//! Tests for basic PPI Rust code generation
//!
//! These tests cover fundamental generation capabilities:
//! - Simple arithmetic operations
//! - String interpolation
//! - Function signature generation

use crate::ppi::rust_generator::{signature, RustGenerator};
use crate::ppi::types::ExpressionContext;
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
    // Fix assertion - check what the generated comment actually contains
    assert!(result.contains("$val / 100"));
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
fn test_standalone_interpolated_string_valueconv_composite() {
    // Regression: composite ValueConv expressions that are a single
    // double-quoted interpolated string — e.g. IPTC's DateTimeCreated
    // ValueConv '"$val[0] $val[1]"' (IPTC.pm Composite table) — were routed
    // through the standalone-literal fast path, which emitted the raw text
    // (`TagValue::String("$val[0] $val[1]".to_string())`) instead of
    // interpolating. A double-quoted string containing `$` is not a literal.
    let ast_json = json!({
        "children": [{
            "children": [{
                "class": "PPI::Token::Quote::Double",
                "content": "\"$val[0] $val[1]\"",
                "string_value": "$val[0] $val[1]"
            }],
            "class": "PPI::Statement"
        }],
        "class": "PPI::Document"
    });

    let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

    let generator = RustGenerator::with_context(
        ExpressionType::ValueConv,
        ExpressionContext::Composite,
        "test_datetimecreated".to_string(),
        "\"$val[0] $val[1]\"".to_string(),
    );

    let result = generator.generate_function(&ast).unwrap();

    // Must interpolate both dependencies, not echo the Perl source
    assert!(
        result.contains("format!") && result.contains("vals"),
        "expected interpolation via format!/vals, got:\n{result}"
    );
    assert!(
        !result.contains("\"$val[0] $val[1]\".to_string()"),
        "raw Perl text leaked into generated code:\n{result}"
    );
}

#[test]
fn test_signature_generation() {
    let generator = RustGenerator::new(
        ExpressionType::Condition,
        "test_condition".to_string(),
        "$$self{Make} =~ /Canon/".to_string(),
    );

    let signature =
        signature::generate_signature(&generator.expression_type, &generator.function_name);

    assert!(signature.contains("pub fn test_condition"));
    assert!(signature.contains("val: &TagValue"));
    assert!(signature.contains("ctx: Option<&ExifContext>"));
    assert!(signature.contains("-> bool"));
}
