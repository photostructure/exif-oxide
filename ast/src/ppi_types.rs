//! PPI AST Type Definitions for P08
//!
//! Rust representations of Perl PPI AST nodes for code generation.
//! These types mirror the JSON structure output by field_extractor_with_ast.pl
//!
//! ExifTool references:
//! - Trust ExifTool: Preserve exact Perl semantics in Rust generation
//! - Expression types: Condition, ValueConv, PrintConv have different requirements

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main AST analysis structure from field_extractor_with_ast.pl
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstAnalysis {
    pub expressions_found: usize,
    pub ast_parseable: usize,
    pub ppi_ast_data: HashMap<String, AstInfo>,
    pub expression_types: ExpressionTypeCounts,
}

/// Count of different expression types found
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpressionTypeCounts {
    pub condition: usize,
    pub value_conv: usize,
    pub print_conv: usize,
}

/// Detailed AST information for a single expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstInfo {
    pub original: String,
    pub parseable: bool,
    pub node_types: Vec<String>,
    pub has_variables: bool,
    pub has_self_refs: bool,
    pub has_functions: bool,
    pub has_operators: bool,
}

// Removed ComplexityLevel and FeasibilityLevel enums
// Using granular boolean flags for smarter routing decisions

/// PPI AST Node representation
/// Maps to PPI node types from Perl AST parsing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PpiNode {
    pub node_type: String, // e.g., "PPI::Token::Symbol", "PPI::Statement::Compound"
    pub content: String,
    pub children: Vec<PpiNode>,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

impl PpiNode {
    /// Check if this node represents a $$self context access
    pub fn is_self_reference(&self) -> bool {
        self.node_type == "PPI::Token::Symbol" && self.content.starts_with("$$self")
    }

    /// Check if this node represents a variable access
    pub fn is_variable(&self) -> bool {
        self.node_type == "PPI::Token::Symbol" && self.content.starts_with('$')
    }

    /// Check if this node represents a function call
    pub fn is_function_call(&self) -> bool {
        self.node_type == "PPI::Token::Word"
            || self.node_type == "PPI::Statement::Expression"
                && self
                    .children
                    .iter()
                    .any(|c| c.node_type == "PPI::Token::Word")
    }

    /// Extract $$self field name (e.g., "$$self{Make}" -> "Make")
    pub fn extract_self_field(&self) -> Option<String> {
        if !self.is_self_reference() {
            return None;
        }

        // Parse $$self{Field} pattern
        if let Some(start) = self.content.find('{') {
            if let Some(end) = self.content.find('}') {
                if end > start {
                    return Some(self.content[start + 1..end].to_string());
                }
            }
        }

        None
    }
}

/// Expression type classification for code generation strategy
#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionType {
    /// Boolean conditions for tag variants
    Condition,
    /// Mathematical value transformations  
    ValueConv,
    /// String formatting for human-readable output
    PrintConv,
}

impl ExpressionType {
    /// Get expression type from context string
    pub fn from_context(context: &str) -> Option<Self> {
        if context.contains("Condition") {
            Some(ExpressionType::Condition)
        } else if context.contains("ValueConv") {
            Some(ExpressionType::ValueConv)
        } else if context.contains("PrintConv") {
            Some(ExpressionType::PrintConv)
        } else {
            None
        }
    }

    /// Get default return type for this expression type
    pub fn default_return_type(&self) -> &'static str {
        match self {
            ExpressionType::Condition => "bool",
            ExpressionType::ValueConv => "TagValue",
            ExpressionType::PrintConv => "String",
        }
    }
}

/// Code generation context for expressions
#[derive(Debug, Clone)]
pub struct GenerationContext {
    pub expression_type: ExpressionType,
    pub has_self_context: bool,
    pub required_functions: Vec<String>,
    pub variable_mappings: HashMap<String, String>,
}

impl GenerationContext {
    pub fn new(expression_type: ExpressionType) -> Self {
        Self {
            expression_type,
            has_self_context: false,
            required_functions: Vec::new(),
            variable_mappings: HashMap::new(),
        }
    }

    /// Add a context field requirement (e.g., "Make", "Model")
    pub fn add_context_field(&mut self, field: &str) {
        self.has_self_context = true;
        self.variable_mappings.insert(
            format!("$$self{{{}}}", field),
            format!("ctx.{}.as_deref().unwrap_or(\"\")", field.to_lowercase()),
        );
    }

    /// Add a function dependency (e.g., "sprintf", "int")
    pub fn add_function(&mut self, func_name: &str) {
        if !self.required_functions.contains(&func_name.to_string()) {
            self.required_functions.push(func_name.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ppi_node_self_reference() {
        let node = PpiNode {
            node_type: "PPI::Token::Symbol".to_string(),
            content: "$$self{Make}".to_string(),
            children: vec![],
            line: Some(1),
            column: Some(0),
        };

        assert!(node.is_self_reference());
        assert_eq!(node.extract_self_field(), Some("Make".to_string()));
    }

    #[test]
    fn test_ppi_node_variable() {
        let node = PpiNode {
            node_type: "PPI::Token::Symbol".to_string(),
            content: "$val".to_string(),
            children: vec![],
            line: None,
            column: None,
        };

        assert!(node.is_variable());
        assert!(!node.is_self_reference());
    }

    #[test]
    fn test_expression_type_from_context() {
        assert_eq!(
            ExpressionType::from_context("TagName.Condition"),
            Some(ExpressionType::Condition)
        );
        assert_eq!(
            ExpressionType::from_context("TagName.ValueConv"),
            Some(ExpressionType::ValueConv)
        );
        assert_eq!(
            ExpressionType::from_context("TagName.PrintConv"),
            Some(ExpressionType::PrintConv)
        );
    }

    #[test]
    fn test_generation_context() {
        let mut ctx = GenerationContext::new(ExpressionType::ValueConv);
        ctx.add_context_field("Make");
        ctx.add_function("sprintf");

        assert!(ctx.has_self_context);
        assert!(ctx.required_functions.contains(&"sprintf".to_string()));
        assert!(ctx.variable_mappings.contains_key("$$self{Make}"));
    }
}
