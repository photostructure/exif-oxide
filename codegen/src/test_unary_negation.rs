#[cfg(test)]
mod tests {
    use crate::ppi::rust_generator::visitor::PpiVisitor;
    use crate::ppi::rust_generator::RustGenerator;
    use crate::ppi::types::{ExpressionType, PpiNode};

    #[test]
    fn test_unary_negation_transformation() {
        // Test the visitor directly with a UnaryNegation node wrapping ArrayAccess
        let normalized = PpiNode {
            class: "UnaryNegation".to_string(),
            content: Some("-".to_string()),
            children: vec![PpiNode {
                class: "ArrayAccess".to_string(),
                content: Some("$val".to_string()),
                children: vec![PpiNode {
                    class: "PPI::Structure::Subscript".to_string(),
                    content: None,
                    children: vec![PpiNode {
                        class: "PPI::Statement::Expression".to_string(),
                        content: None,
                        children: vec![PpiNode {
                            class: "PPI::Token::Number".to_string(),
                            content: Some("0".to_string()),
                            children: vec![],
                            symbol_type: None,
                            numeric_value: Some(0.0),
                            string_value: None,
                            structure_bounds: None,
                        }],
                        symbol_type: None,
                        numeric_value: None,
                        string_value: None,
                        structure_bounds: None,
                    }],
                    symbol_type: None,
                    numeric_value: None,
                    string_value: None,
                    structure_bounds: None,
                }],
                symbol_type: Some("scalar".to_string()),
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            }],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        println!("Normalized AST: {:#?}", normalized);

        // Check that we get a UnaryNegation node instead of BinaryOperation
        assert!(
            contains_unary_negation(&normalized),
            "Expected to find UnaryNegation node in the normalized AST"
        );

        // Test the visitor
        let generator = RustGenerator::new(
            ExpressionType::ValueConv,
            "test_function".to_string(),
            "-$val[0]".to_string(),
        );
        let generated_code = generator.visit_node(&normalized).unwrap();

        println!("Generated code: {}", generated_code);

        // Verify that the generated code uses negate() instead of (0i32 - ...)
        assert!(
            generated_code.contains("codegen_runtime::negate("),
            "Expected generated code to contain negate() call, got: {}",
            generated_code
        );
        assert!(
            !generated_code.contains("0i32 -"),
            "Generated code should not contain '0i32 -' pattern, got: {}",
            generated_code
        );
    }

    // Helper function to recursively check for UnaryNegation nodes
    fn contains_unary_negation(node: &PpiNode) -> bool {
        if node.class == "UnaryNegation" {
            return true;
        }
        for child in &node.children {
            if contains_unary_negation(child) {
                return true;
            }
        }
        false
    }
}
