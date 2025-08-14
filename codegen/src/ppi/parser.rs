//! PPI JSON Parser
//!
//! Parses PPI JSON structures from field_extractor.pl output.
//! Handles the inline AST fields (PrintConv_ast, ValueConv_ast, Condition_ast).

use super::types::*;
use serde_json::Value as JsonValue;

/// Parse PPI JSON structure into PpiNode
pub fn parse_ppi_json(json: &JsonValue) -> Result<PpiNode, PpiParseError> {
    serde_json::from_value(json.clone()).map_err(PpiParseError::Json)
}

/// Extract inline AST fields from tag data
///
/// Looks for fields like "PrintConv_ast", "ValueConv_ast", "Condition_ast"
/// in the tag JSON structure.
pub fn extract_inline_ast_fields(tag_data: &JsonValue) -> Vec<(String, PpiNode, ExpressionType)> {
    let mut ast_fields = Vec::new();

    if let JsonValue::Object(map) = tag_data {
        // Look for all *_ast fields
        for (key, value) in map {
            if key.ends_with("_ast") {
                if let Ok(ppi_node) = parse_ppi_json(value) {
                    let expr_type = ExpressionType::from_field_name(key);
                    ast_fields.push((key.clone(), ppi_node, expr_type));
                }
            }
        }
    }

    ast_fields
}

/// Check if a tag has any inline AST fields  
pub fn has_inline_ast(tag_data: &JsonValue) -> bool {
    if let JsonValue::Object(map) = tag_data {
        map.keys().any(|key| key.ends_with("_ast"))
    } else {
        false
    }
}

/// Extract the original Perl expression corresponding to an AST field
///
/// For "PrintConv_ast", looks for "PrintConv" field with the original expression.
pub fn get_original_expression(tag_data: &JsonValue, ast_field: &str) -> Option<String> {
    if let JsonValue::Object(map) = tag_data {
        // Convert "PrintConv_ast" -> "PrintConv"
        let original_field = ast_field.strip_suffix("_ast")?;

        map.get(original_field)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_ppi_json_simple_symbol() {
        let json = json!({
            "class": "PPI::Token::Symbol",
            "content": "$val",
            "symbol_type": "scalar"
        });

        let node = parse_ppi_json(&json).unwrap();

        assert_eq!(node.class, "PPI::Token::Symbol");
        assert_eq!(node.content, Some("$val".to_string()));
        assert_eq!(node.symbol_type, Some("scalar".to_string()));
    }

    #[test]
    fn test_extract_inline_ast_fields() {
        let tag_json = json!({
            "Name": "AFConfigTool",
            "PrintConv": "\"Case $val\"",
            "PrintConv_ast": {
                "children": [{
                    "children": [{
                        "class": "PPI::Token::Quote::Double",
                        "content": "\"Case $val\"",
                        "string_value": "Case $val"
                    }],
                    "class": "PPI::Statement"
                }],
                "class": "PPI::Document"
            },
            "ValueConv": "$val + 1",
            "ValueConv_ast": {
                "children": [{
                    "children": [{
                        "class": "PPI::Token::Symbol",
                        "content": "$val",
                        "symbol_type": "scalar"
                    }, {
                        "class": "PPI::Token::Operator",
                        "content": "+"
                    }, {
                        "class": "PPI::Token::Number",
                        "content": "1",
                        "numeric_value": 1
                    }],
                    "class": "PPI::Statement"
                }],
                "class": "PPI::Document"
            }
        });

        let ast_fields = extract_inline_ast_fields(&tag_json);

        assert_eq!(ast_fields.len(), 2);

        // Check PrintConv_ast field
        let (print_field, print_node, print_type) = &ast_fields[0];
        assert_eq!(print_field, "PrintConv_ast");
        assert_eq!(*print_type, ExpressionType::PrintConv);
        assert_eq!(print_node.class, "PPI::Document");

        // Check ValueConv_ast field
        let (value_field, value_node, value_type) = &ast_fields[1];
        assert_eq!(value_field, "ValueConv_ast");
        assert_eq!(*value_type, ExpressionType::ValueConv);
        assert_eq!(value_node.class, "PPI::Document");
    }

    #[test]
    fn test_has_inline_ast() {
        let with_ast = json!({
            "PrintConv": "$val / 100",
            "PrintConv_ast": {"class": "PPI::Document", "children": []}
        });

        let without_ast = json!({
            "PrintConv": "$val / 100"
        });

        assert!(has_inline_ast(&with_ast));
        assert!(!has_inline_ast(&without_ast));
    }

    #[test]
    fn test_get_original_expression() {
        let tag_json = json!({
            "PrintConv": "$val / 100",
            "PrintConv_ast": {"class": "PPI::Document"},
            "ValueConv": "$val + 1",
            "ValueConv_ast": {"class": "PPI::Document"}
        });

        assert_eq!(
            get_original_expression(&tag_json, "PrintConv_ast"),
            Some("$val / 100".to_string())
        );

        assert_eq!(
            get_original_expression(&tag_json, "ValueConv_ast"),
            Some("$val + 1".to_string())
        );

        assert_eq!(get_original_expression(&tag_json, "Unknown_ast"), None);
    }

    #[test]
    fn test_canon_real_output() {
        // Test with real Canon.pm output structure
        let canon_json = json!({
            "1": {
                "Name": "AFConfigTool",
                "PrintConv": "\"Case $val\"",
                "PrintConvInv": "$val=~/(\\d+)/ ? $1 : undef",
                "PrintConv_ast": {
                    "children": [{
                        "children": [{
                            "class": "PPI::Token::Quote::Double",
                            "content": "\"Case $val\"",
                            "string_value": "Case $val"
                        }],
                        "class": "PPI::Statement"
                    }],
                    "class": "PPI::Document"
                },
                "ValueConv": "$val + 1",
                "ValueConvInv": "$val - 1",
                "ValueConv_ast": {
                    "children": [{
                        "children": [{
                            "class": "PPI::Token::Symbol",
                            "content": "$val",
                            "symbol_type": "scalar"
                        }, {
                            "class": "PPI::Token::Operator",
                            "content": "+"
                        }, {
                            "class": "PPI::Token::Number",
                            "content": "1",
                            "numeric_value": 1
                        }],
                        "class": "PPI::Statement"
                    }],
                    "class": "PPI::Document"
                }
            }
        });

        if let JsonValue::Object(data_map) = &canon_json {
            if let Some(tag_1) = data_map.get("1") {
                let ast_fields = extract_inline_ast_fields(tag_1);

                // Should find PrintConv_ast and ValueConv_ast
                assert_eq!(ast_fields.len(), 2);

                // Verify we can parse both AST structures
                for (field_name, node, expr_type) in ast_fields {
                    match expr_type {
                        ExpressionType::PrintConv => {
                            assert_eq!(field_name, "PrintConv_ast");
                            assert_eq!(node.class, "PPI::Document");
                        }
                        ExpressionType::ValueConv => {
                            assert_eq!(field_name, "ValueConv_ast");
                            assert_eq!(node.class, "PPI::Document");
                        }
                        _ => panic!("Unexpected expression type"),
                    }
                }
            }
        }
    }
}
