//! Tests for static function generation compliance
//!
//! These tests verify that pattern recognition generates proper static functions
//! rather than runtime expression evaluation, ensuring P07 compliance.

use crate::ppi::rust_generator::{expressions::ExpressionCombiner, RustGenerator};
use crate::ppi::ExpressionType;

#[test]
fn test_static_function_generation() {
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
                result.contains("sprintf_with_string") || result.contains("crate::fmt::sprintf")
            );
            assert!(!result.contains("Expression"));
            assert!(!result.contains("evaluate_expression"));
        }
        Err(e) => panic!("Sprintf static function generation failed: {:?}", e),
    }

    // Test 4: Verify P07 compliance - static functions not runtime evaluation
    // All generated functions should produce compile-time Rust code, not runtime expression strings
    for generator in [pack_map_generator, sprintf_generator] {
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
        // Note: We use codegen_runtime for TagValue types, so checking for "runtime"
        // in imports is overly broad. The real validation is that we don't generate
        // runtime expression evaluation strings.
        assert!(!function_result.contains("evaluate_expression"));
        assert!(!function_result.contains("dynamic"));
    }
}
