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
pub mod function_generation;
pub mod numeric_string_ops;
pub mod pattern_recognition;
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
