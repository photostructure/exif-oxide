//! PPI AST Node Types for Codegen
//!
//! Rust representations of PPI JSON structures from field_extractor.pl
//! These types match the actual JSON output format we observed from the Perl script.
//!
//! Trust ExifTool: These structures preserve the exact PPI token hierarchy.

use serde::{Deserialize, Serialize};

/// PPI Document/Node from JSON
///
/// This matches the JSON structure we see from Perl:
/// ```json
/// {
///   "class": "PPI::Token::Symbol",
///   "content": "$val",
///   "symbol_type": "scalar",
///   "children": [...]
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PpiNode {
    /// PPI class name (e.g., "PPI::Token::Symbol", "PPI::Statement")
    pub class: String,

    /// Raw text content for atomic tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Child nodes for container elements
    #[serde(default)]
    pub children: Vec<PpiNode>,

    /// Symbol type for PPI::Token::Symbol nodes ("scalar", "array", "hash")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol_type: Option<String>,

    /// Numeric value for PPI::Token::Number nodes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numeric_value: Option<f64>,

    /// String value for PPI::Token::Quote::* nodes (without quotes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub string_value: Option<String>,

    /// Structure bounds for PPI::Structure::* nodes ("( ... )", "{ ... }")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structure_bounds: Option<String>,
}

/// Expression type classification for code generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpressionType {
    /// Boolean conditions for tag variants ($$self{Make} =~ /Canon/)
    Condition,
    /// Mathematical value transformations ($val / 100)
    ValueConv,
    /// String formatting for human-readable output (sprintf("%.1f mm", $val))
    PrintConv,
}

impl ExpressionType {
    /// Determine expression type from field name context
    pub fn from_field_name(field_name: &str) -> Self {
        if field_name.contains("Condition") {
            ExpressionType::Condition
        } else if field_name.contains("ValueConv") {
            ExpressionType::ValueConv
        } else if field_name.contains("PrintConv") {
            ExpressionType::PrintConv
        } else {
            // Default to ValueConv for unknown contexts
            ExpressionType::ValueConv
        }
    }

    /// Get the expected return type for generated functions
    pub fn return_type(&self) -> &'static str {
        match self {
            ExpressionType::Condition => "bool",
            ExpressionType::ValueConv => "TagValue",
            ExpressionType::PrintConv => "TagValue", // PrintConv returns TagValue (usually String variant)
        }
    }

    /// Get imports needed for this expression type
    pub fn required_imports(&self) -> Vec<&'static str> {
        match self {
            ExpressionType::Condition => vec!["crate::TagValue"],
            ExpressionType::ValueConv => vec!["crate::TagValue"],
            ExpressionType::PrintConv => vec!["crate::TagValue"],
        }
    }
}

/// Helper methods for PPI node analysis
impl PpiNode {
    /// Check if this is a variable reference ($val, $$self{Field})
    pub fn is_variable(&self) -> bool {
        self.class == "PPI::Token::Symbol"
            && self
                .content
                .as_ref()
                .map(|c| c.starts_with('$'))
                .unwrap_or(false)
    }

    /// Check if this is a self-reference ($$self{Field})
    pub fn is_self_reference(&self) -> bool {
        self.class == "PPI::Token::Symbol"
            && self
                .content
                .as_ref()
                .map(|c| c.starts_with("$$self"))
                .unwrap_or(false)
    }

    /// Extract field name from $$self{FieldName} pattern
    pub fn extract_self_field(&self) -> Option<String> {
        if !self.is_self_reference() {
            return None;
        }

        let content = self.content.as_ref()?;

        // Look for $$self{Field} pattern
        if let Some(start) = content.find('{') {
            if let Some(end) = content.find('}') {
                if end > start + 1 {
                    return Some(content[start + 1..end].to_string());
                }
            }
        }

        None
    }

    /// Check if this is an operator token
    pub fn is_operator(&self) -> bool {
        self.class == "PPI::Token::Operator"
    }

    /// Check if this is a number literal
    pub fn is_number(&self) -> bool {
        self.class == "PPI::Token::Number"
    }

    /// Check if this is a string literal
    pub fn is_string(&self) -> bool {
        self.class.starts_with("PPI::Token::Quote::")
    }

    /// Check if this is a function call word
    pub fn is_word(&self) -> bool {
        self.class == "PPI::Token::Word"
    }

    /// Get the operator text if this is an operator
    pub fn operator_text(&self) -> Option<&str> {
        if self.is_operator() {
            self.content.as_deref()
        } else {
            None
        }
    }
}

/// Error types for PPI parsing
#[derive(Debug, thiserror::Error)]
pub enum PpiParseError {
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid PPI structure: {0}")]
    InvalidStructure(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Unsupported PPI token type: {0}")]
    UnsupportedToken(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_ppi_node_deserialization() {
        let json = json!({
            "class": "PPI::Token::Symbol",
            "content": "$val",
            "symbol_type": "scalar"
        });

        let node: PpiNode = serde_json::from_value(json).unwrap();

        assert_eq!(node.class, "PPI::Token::Symbol");
        assert_eq!(node.content, Some("$val".to_string()));
        assert_eq!(node.symbol_type, Some("scalar".to_string()));
        assert!(node.is_variable());
        assert!(!node.is_self_reference());
    }

    #[test]
    fn test_self_reference_detection() {
        let json = json!({
            "class": "PPI::Token::Symbol",
            "content": "$$self{Make}",
            "symbol_type": "scalar"
        });

        let node: PpiNode = serde_json::from_value(json).unwrap();

        assert!(node.is_self_reference());
        assert_eq!(node.extract_self_field(), Some("Make".to_string()));
    }

    #[test]
    fn test_expression_type_from_field_name() {
        assert_eq!(
            ExpressionType::from_field_name("PrintConv_ast"),
            ExpressionType::PrintConv
        );
        assert_eq!(
            ExpressionType::from_field_name("ValueConv_ast"),
            ExpressionType::ValueConv
        );
        assert_eq!(
            ExpressionType::from_field_name("Condition_ast"),
            ExpressionType::Condition
        );
        assert_eq!(
            ExpressionType::from_field_name("unknown_field"),
            ExpressionType::ValueConv
        );
    }

    #[test]
    fn test_complex_ast_structure() {
        // Test the real structure from Canon.pm output
        let json = json!({
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

        let node: PpiNode = serde_json::from_value(json).unwrap();

        assert_eq!(node.class, "PPI::Document");
        assert_eq!(node.children.len(), 1);

        let statement = &node.children[0];
        assert_eq!(statement.class, "PPI::Statement");
        assert_eq!(statement.children.len(), 3);

        let symbol = &statement.children[0];
        assert!(symbol.is_variable());
        assert_eq!(symbol.content, Some("$val".to_string()));

        let operator = &statement.children[1];
        assert!(operator.is_operator());
        assert_eq!(operator.operator_text(), Some("/"));

        let number = &statement.children[2];
        assert!(number.is_number());
        assert_eq!(number.numeric_value, Some(100.0));
    }
}
