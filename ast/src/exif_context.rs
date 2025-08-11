//! ExifContext modeling for P08 AST generation
//!
//! This module defines the context structure needed for expressions that access
//! $$self references in ExifTool expressions.
//!
//! ExifTool references:
//! - lib/Image/ExifTool.pm: $$self hash contains file state and metadata
//! - Expression patterns: $$self{Make}, $$self{Model}, $$self{FileType}, etc.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Context structure for ExifTool $$self access patterns
/// This models the runtime state that ExifTool expressions can access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExifContext {
    /// Camera manufacturer ($$self{Make})
    pub make: Option<String>,

    /// Camera model ($$self{Model})
    pub model: Option<String>,

    /// File type ($$self{FileType})
    pub file_type: Option<String>,

    /// MIME type ($$self{MIMEType})
    pub mime_type: Option<String>,

    /// File size in bytes ($$self{FileSize})
    pub file_size: Option<u64>,

    /// Image width ($$self{ImageWidth})
    pub image_width: Option<u32>,

    /// Image height ($$self{ImageHeight})
    pub image_height: Option<u32>,

    /// Color space ($$self{ColorSpace})
    pub color_space: Option<String>,

    /// Camera settings and other dynamic fields
    /// This handles less common $$self{Key} patterns
    pub additional_fields: HashMap<String, String>,
}

impl ExifContext {
    /// Create new empty context
    pub fn new() -> Self {
        Self {
            make: None,
            model: None,
            file_type: None,
            mime_type: None,
            file_size: None,
            image_width: None,
            image_height: None,
            color_space: None,
            additional_fields: HashMap::new(),
        }
    }

    /// Get field value by name (for dynamic $$self{Field} access)
    pub fn get_field(&self, field_name: &str) -> Option<&str> {
        match field_name {
            "Make" => self.make.as_deref(),
            "Model" => self.model.as_deref(),
            "FileType" => self.file_type.as_deref(),
            "MIMEType" => self.mime_type.as_deref(),
            "ColorSpace" => self.color_space.as_deref(),
            _ => self.additional_fields.get(field_name).map(|s| s.as_str()),
        }
    }

    /// Set field value by name
    pub fn set_field(&mut self, field_name: &str, value: String) {
        match field_name {
            "Make" => self.make = Some(value),
            "Model" => self.model = Some(value),
            "FileType" => self.file_type = Some(value),
            "MIMEType" => self.mime_type = Some(value),
            "ColorSpace" => self.color_space = Some(value),
            _ => {
                self.additional_fields.insert(field_name.to_string(), value);
            }
        }
    }

    /// Get numeric field value (for width, height, size fields)
    pub fn get_numeric_field(&self, field_name: &str) -> Option<f64> {
        match field_name {
            "FileSize" => self.file_size.map(|s| s as f64),
            "ImageWidth" => self.image_width.map(|w| w as f64),
            "ImageHeight" => self.image_height.map(|h| h as f64),
            _ => {
                // Try to parse additional fields as numbers
                self.additional_fields
                    .get(field_name)
                    .and_then(|s| s.parse::<f64>().ok())
            }
        }
    }

    /// Check if context has all required fields for an expression
    pub fn has_required_fields(&self, required_fields: &[&str]) -> bool {
        required_fields
            .iter()
            .all(|field| self.get_field(field).is_some() || self.get_numeric_field(field).is_some())
    }
}

impl Default for ExifContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Context access pattern for code generation
#[derive(Debug, Clone)]
pub struct ContextAccess {
    pub field_name: String,
    pub access_pattern: String,  // The original $$self{Field} pattern
    pub rust_expression: String, // Generated Rust code
    pub return_type: ContextFieldType,
}

/// Type classification for context fields
#[derive(Debug, Clone, PartialEq)]
pub enum ContextFieldType {
    String,
    Numeric,
    Boolean,
}

impl ContextAccess {
    /// Create context access for a field name
    pub fn new(field_name: &str) -> Self {
        let access_pattern = format!("$$self{{{}}}", field_name);
        let (rust_expression, return_type) = Self::generate_rust_access(field_name);

        Self {
            field_name: field_name.to_string(),
            access_pattern,
            rust_expression,
            return_type,
        }
    }

    /// Generate Rust code for accessing a context field
    fn generate_rust_access(field_name: &str) -> (String, ContextFieldType) {
        match field_name {
            // Numeric fields
            "FileSize" => (
                "ctx.file_size.unwrap_or(0) as f64".to_string(),
                ContextFieldType::Numeric,
            ),
            "ImageWidth" => (
                "ctx.image_width.unwrap_or(0) as f64".to_string(),
                ContextFieldType::Numeric,
            ),
            "ImageHeight" => (
                "ctx.image_height.unwrap_or(0) as f64".to_string(),
                ContextFieldType::Numeric,
            ),

            // String fields
            _ => (
                format!("ctx.get_field(\"{}\").unwrap_or(\"\")", field_name),
                ContextFieldType::String,
            ),
        }
    }
}

/// Utility functions for context field analysis
pub mod utils {
    use regex;

    /// Extract all $$self field references from an expression
    pub fn extract_self_references(expression: &str) -> Vec<String> {
        let mut references = Vec::new();

        // Use regex to find $$self{field} patterns
        if let Ok(re) = regex::Regex::new(r"\$\$self\{([^}]+)\}") {
            for captures in re.captures_iter(expression) {
                if let Some(field_match) = captures.get(1) {
                    let field_name = field_match.as_str().to_string();
                    if !references.contains(&field_name) {
                        references.push(field_name);
                    }
                }
            }
        }

        references
    }

    /// Determine if expression requires runtime context
    pub fn requires_context(expression: &str) -> bool {
        expression.contains("$$self{")
    }

    /// Generate context parameter for function signature
    pub fn generate_context_parameter(has_context: bool) -> String {
        if has_context {
            "ctx: &ExifContext".to_string()
        } else {
            String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::utils::*;
    use super::*;

    #[test]
    fn test_exif_context_basic_fields() {
        let mut ctx = ExifContext::new();
        ctx.set_field("Make", "Canon".to_string());
        ctx.set_field("Model", "EOS R5".to_string());

        assert_eq!(ctx.get_field("Make"), Some("Canon"));
        assert_eq!(ctx.get_field("Model"), Some("EOS R5"));
        assert_eq!(ctx.get_field("Unknown"), None);
    }

    #[test]
    fn test_numeric_fields() {
        let mut ctx = ExifContext::new();
        ctx.file_size = Some(1024);
        ctx.image_width = Some(6000);

        assert_eq!(ctx.get_numeric_field("FileSize"), Some(1024.0));
        assert_eq!(ctx.get_numeric_field("ImageWidth"), Some(6000.0));
        assert_eq!(ctx.get_numeric_field("Unknown"), None);
    }

    #[test]
    fn test_extract_self_references() {
        let expr = r#"$$self{Make} eq "Canon" && $$self{Model} =~ /R5/"#;
        let refs = extract_self_references(expr);

        assert_eq!(refs.len(), 2);
        assert!(refs.contains(&"Make".to_string()));
        assert!(refs.contains(&"Model".to_string()));
    }

    #[test]
    fn test_requires_context() {
        assert!(requires_context("$$self{Make} eq 'Canon'"));
        assert!(!requires_context("$val * 100"));
        assert!(requires_context("$val > 0 && $$self{FileType} eq 'JPEG'"));
    }

    #[test]
    fn test_context_access() {
        let access = ContextAccess::new("Make");

        assert_eq!(access.field_name, "Make");
        assert_eq!(access.access_pattern, "$$self{Make}");
        assert_eq!(access.return_type, ContextFieldType::String);
        assert!(access.rust_expression.contains("Make"));
    }

    #[test]
    fn test_numeric_context_access() {
        let access = ContextAccess::new("FileSize");

        assert_eq!(access.return_type, ContextFieldType::Numeric);
        assert!(access.rust_expression.contains("file_size"));
    }
}
