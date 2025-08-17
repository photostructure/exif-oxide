/// Test module for nested function call handling

#[cfg(test)]
mod tests {
    use crate::ppi::rust_generator::*;
    use crate::ppi::types::*;
    use serde_json;
    use std::fs;

    #[test]
    fn test_sprintf_with_nested_split() {
        // Load the AST from our Perl test script
        let ast_json = fs::read_to_string("/tmp/test_nested_ast.json")
            .expect("Failed to read test AST");
        
        let ast: PpiNode = serde_json::from_str(&ast_json)
            .expect("Failed to parse AST JSON");

        // Create a generator for PrintConv expression
        let generator = RustGenerator::new(ExpressionType::PrintConv);
        
        // Generate Rust code
        let result = generator.visit_node(&ast);
        
        match result {
            Ok(code) => {
                println!("Generated code:\n{}", code);
                
                // Check that the generated code handles the nested split properly
                assert!(code.contains("crate::types::split_tagvalue"), 
                    "Code should contain split_tagvalue call");
                assert!(code.contains(":.3"), 
                    "Code should preserve %.3f format as {:.3}");
                assert!(code.contains("TagValue::String"), 
                    "Code should wrap result in TagValue::String");
            }
            Err(e) => {
                panic!("Failed to generate code: {:?}", e);
            }
        }
    }

    #[test] 
    fn test_simple_sprintf() {
        // Test simple sprintf without nested calls
        let expr = r#"{"class":"PPI::Document","children":[{"class":"PPI::Statement","children":[{"class":"PPI::Token::Word","content":"sprintf"},{"class":"PPI::Structure::List","structure_bounds":"( ... )","children":[{"class":"PPI::Statement::Expression","children":[{"class":"PPI::Token::Quote::Double","content":"\"Value: %d\"","string_value":"Value: %d"},{"class":"PPI::Token::Operator","content":","},{"class":"PPI::Token::Symbol","content":"$val","symbol_type":"scalar"}]}]}]}]}"#;
        
        let ast: PpiNode = serde_json::from_str(expr)
            .expect("Failed to parse simple sprintf AST");
            
        let generator = RustGenerator::new(ExpressionType::PrintConv);
        let result = generator.visit_node(&ast).expect("Failed to generate code");
        
        println!("Simple sprintf generated: {}", result);
        assert!(result.contains("format!(\"Value: {}\""), 
            "Should convert %d to {}");
    }
}