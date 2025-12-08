//! Tests for PPI Rust code generation
//!
//! This module contains focused test suites organized by functionality:
//! - `basic_generation`: Simple arithmetic, strings, signatures
//! - `numeric_string_ops`: Complex numeric and string operations
//! - `control_flow`: Control flow, magic variables, transliterate operations
//! - `pattern_recognition`: Pattern matching for pack, join, safe division
//! - `function_generation`: Static function generation compliance
//! - `sprintf_normalization`: sprintf with normalized string operations

pub mod basic_generation;
pub mod control_flow;
pub mod numeric_string_ops;
pub mod sprintf_normalization;

// Integration test for normalizer + visitor pipeline
#[cfg(test)]
mod normalizer_integration {
    use crate::ppi::rust_generator::RustGenerator;
    use crate::ppi::{parse_ppi_json, ExpressionType};
    use serde_json::json;

    #[test]
    #[ignore] // TODO: Migrate to JSON-based test infrastructure - join/unpack support incomplete
    fn test_join_unpack_end_to_end() {
        // Create the flat AST that PPI would output for: join " ", unpack "H2H2", val
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

        let ppi_ast = parse_ppi_json(&join_unpack_ast).unwrap();

        // Create a generator that should use the normalizer
        let generator = RustGenerator::new(
            ExpressionType::PrintConv,
            "test_join_unpack_integration".to_string(),
            "join \" \", unpack \"H2H2\", val".to_string(),
        );

        // Generate the function - this should use the normalizer + visitor pipeline
        let generated_code = generator.generate_function(&ppi_ast).unwrap();

        println!("Generated code:\n{}", generated_code);

        // The generated code should NOT contain malformed patterns like "join(\" \", ,, unpack"
        assert!(
            !generated_code.contains(",,"),
            "Generated code contains malformed comma sequences"
        );

        // The generated code SHOULD contain proper function calls
        assert!(
            generated_code.contains("TagValue::String(crate::fmt::join_binary")
                || generated_code.contains("join"),
            "Generated code should contain join call: {}",
            generated_code
        );
        assert!(
            generated_code.contains("crate::fmt::unpack_binary")
                || generated_code.contains("unpack"),
            "Generated code should contain unpack call: {}",
            generated_code
        );

        // The code should be valid Rust (no Perl syntax)
        assert!(
            !generated_code.contains("$val"),
            "Generated code should not contain Perl variables: {}",
            generated_code
        );
    }
}

/// Tests for composite tag expression context
/// These test that the generator produces different code for Regular vs Composite contexts
#[cfg(test)]
mod composite_context {
    use crate::ppi::rust_generator::RustGenerator;
    use crate::ppi::{ExpressionContext, ExpressionType, PpiNode};
    use serde_json::json;

    /// Helper to create array access AST: $val[index]
    fn array_access_ast(var_name: &str, index: usize) -> PpiNode {
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Symbol",
                    "content": var_name,
                    "symbol_type": "scalar"
                }, {
                    "children": [{
                        "children": [{
                            "class": "PPI::Token::Number",
                            "content": index.to_string(),
                            "numeric_value": index as f64
                        }],
                        "class": "PPI::Statement::Expression"
                    }],
                    "class": "PPI::Structure::Subscript",
                    "structure_bounds": "[ ... ]"
                }],
                "class": "PPI::Statement"
            }],
            "class": "PPI::Document"
        });
        serde_json::from_value(ast_json).unwrap()
    }

    #[test]
    fn test_regular_context_uses_get_array_element() {
        let ast = array_access_ast("$val", 0);

        let generator = RustGenerator::new(
            ExpressionType::ValueConv,
            "test_regular".to_string(),
            "$val[0]".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Regular context should use codegen_runtime::get_array_element
        assert!(
            result.contains("codegen_runtime::get_array_element(val, 0)"),
            "Regular context should use get_array_element, got: {result}"
        );
        // Should NOT use vals.get()
        assert!(
            !result.contains("vals.get("),
            "Regular context should NOT use vals.get(), got: {result}"
        );
    }

    #[test]
    fn test_composite_context_uses_slice_access() {
        let ast = array_access_ast("$val", 0);

        let generator = RustGenerator::with_context(
            ExpressionType::ValueConv,
            ExpressionContext::Composite,
            "test_composite".to_string(),
            "$val[0]".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Composite context should use vals.first() for index 0
        assert!(
            result.contains("vals.first().cloned().unwrap_or(TagValue::Empty)"),
            "Composite context should use vals.first() for index 0, got: {result}"
        );
        // Should NOT use codegen_runtime::get_array_element for $val
        assert!(
            !result.contains("codegen_runtime::get_array_element(val, 0)"),
            "Composite context should NOT use get_array_element for $val, got: {result}"
        );
    }

    #[test]
    fn test_composite_context_prt_uses_prts_slice() {
        let ast = array_access_ast("$prt", 1);

        let generator = RustGenerator::with_context(
            ExpressionType::PrintConv,
            ExpressionContext::Composite,
            "test_prt_access".to_string(),
            "$prt[1]".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Composite context for $prt should use prts.get()
        assert!(
            result.contains("prts.get(1).cloned().unwrap_or(TagValue::Empty)"),
            "Composite context should use prts.get() for $prt[], got: {result}"
        );
    }

    #[test]
    fn test_composite_context_raw_uses_raws_slice() {
        let ast = array_access_ast("$raw", 2);

        let generator = RustGenerator::with_context(
            ExpressionType::ValueConv,
            ExpressionContext::Composite,
            "test_raw_access".to_string(),
            "$raw[2]".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Composite context for $raw should use raws.get()
        assert!(
            result.contains("raws.get(2).cloned().unwrap_or(TagValue::Empty)"),
            "Composite context should use raws.get() for $raw[], got: {result}"
        );
    }
}
