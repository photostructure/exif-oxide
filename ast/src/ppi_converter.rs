//! PPI AST to Rust Code Converter for P08
//!
//! This module converts PPI AST data into Rust code following ExifTool semantics.
//! Handles the core AST-to-code generation pipeline.
//!
//! Trust ExifTool: All generated code must preserve exact Perl evaluation semantics

use super::exif_context::utils;
use super::ppi_types::*;
use std::collections::HashMap;

/// Main converter for PPI AST to Rust code
pub struct PpiConverter {
    /// Current generation context
    context: GenerationContext,
    /// Registry of known function conversions
    function_registry: HashMap<String, String>,
}

impl PpiConverter {
    pub fn new(expression_type: ExpressionType) -> Self {
        let mut converter = Self {
            context: GenerationContext::new(expression_type),
            function_registry: HashMap::new(),
        };

        // Initialize standard function mappings
        converter.init_function_registry();
        converter
    }

    /// Initialize registry of Perl to Rust function mappings
    fn init_function_registry(&mut self) {
        // Perl sprintf -> Rust format!
        self.function_registry
            .insert("sprintf".to_string(), "format!".to_string());

        // Perl int() -> Rust as i32
        self.function_registry
            .insert("int".to_string(), "as i32".to_string());

        // Perl abs() -> Rust abs()
        self.function_registry
            .insert("abs".to_string(), "abs".to_string());

        // Perl sqrt() -> Rust sqrt()
        self.function_registry
            .insert("sqrt".to_string(), "sqrt".to_string());

        // Perl exp() -> Rust exp()
        self.function_registry
            .insert("exp".to_string(), "exp".to_string());

        // Perl log() -> Rust ln()
        self.function_registry
            .insert("log".to_string(), "ln".to_string());
    }

    /// Convert AST info to Rust function code using flag-based routing
    pub fn convert_ast_to_rust(&mut self, ast_info: &AstInfo) -> Result<String, ConversionError> {
        // Analyze expression for context requirements
        self.analyze_context_requirements(ast_info);

        // Use flag-based generation routing instead of complexity levels
        if ast_info.has_variables
            && ast_info.has_operators
            && !ast_info.has_functions
            && !ast_info.has_self_refs
        {
            // Simple variable + operator expressions: $val / 100, $val >= 0, etc.
            self.generate_variable_operator_expression(ast_info)
        } else if ast_info.has_operators && !ast_info.has_functions && !ast_info.has_self_refs {
            // Pure arithmetic/comparison expressions
            self.generate_pure_arithmetic_expression(ast_info)
        } else if ast_info.has_variables
            && !ast_info.has_operators
            && !ast_info.has_functions
            && !ast_info.has_self_refs
        {
            // Variable-only expressions: just $val
            self.generate_variable_only_expression(ast_info)
        } else {
            // Everything else should be handled by impl_registry
            Err(ConversionError::RequiresManualImplementation {
                reason: format!(
                    "Expression with flags - functions: {}, self_refs: {}, complex patterns",
                    ast_info.has_functions, ast_info.has_self_refs
                ),
                fallback_suggestion: "Use impl_registry for functions and $$self patterns"
                    .to_string(),
            })
        }
    }

    /// Analyze expression for $$self context requirements
    fn analyze_context_requirements(&mut self, ast_info: &AstInfo) {
        if ast_info.has_self_refs {
            self.context.has_self_context = true;

            // Extract specific field references
            let field_refs = utils::extract_self_references(&ast_info.original);
            for field in field_refs {
                self.context.add_context_field(&field);
            }
        }

        if ast_info.has_functions {
            // Extract function names from node_types
            for node_type in &ast_info.node_types {
                if node_type == "PPI::Token::Word" {
                    // This is a function call - would need more detailed AST to extract name
                    // For now, mark as requiring function support
                    self.context.add_function("unknown_function");
                }
            }
        }
    }

    /// Generate code for variable + operator expressions: $val / 100, $val >= 0
    fn generate_variable_operator_expression(
        &self,
        ast_info: &AstInfo,
    ) -> Result<String, ConversionError> {
        let expr = &ast_info.original;

        // Try common arithmetic patterns
        if let Some(rust_code) = self.try_simple_arithmetic(expr) {
            return Ok(rust_code);
        }

        // Try comparison patterns
        if let Some(rust_code) = self.try_simple_conditional(expr) {
            return Ok(rust_code);
        }

        // Try ternary conditional patterns
        if expr.contains('?') && expr.contains(':') {
            return self.convert_ternary_expression(expr);
        }

        // Fallback to literal translation
        self.literal_translation_with_substitution(expr)
    }

    /// Generate code for pure arithmetic without variables
    fn generate_pure_arithmetic_expression(
        &self,
        ast_info: &AstInfo,
    ) -> Result<String, ConversionError> {
        let expr = &ast_info.original;
        // For pure arithmetic, just clean up and pass through
        self.literal_translation_with_substitution(expr)
    }

    /// Generate code for variable-only expressions: $val
    fn generate_variable_only_expression(
        &self,
        ast_info: &AstInfo,
    ) -> Result<String, ConversionError> {
        let expr = &ast_info.original;
        // Simple variable substitution
        Ok(expr.replace('$', ""))
    }

    /// Try to convert simple arithmetic expressions
    fn try_simple_arithmetic(&self, expr: &str) -> Option<String> {
        // $val / 100 -> val / 100.0
        if let Some(captures) = regex::Regex::new(r"\$(\w+)\s*/\s*(\d+)")
            .ok()?
            .captures(expr)
        {
            let var = &captures[1];
            let divisor = &captures[2];
            return Some(format!("{} / {}.0", var, divisor));
        }

        // $val * 100 -> val * 100.0
        if let Some(captures) = regex::Regex::new(r"\$(\w+)\s*\*\s*(\d+)")
            .ok()?
            .captures(expr)
        {
            let var = &captures[1];
            let multiplier = &captures[2];
            return Some(format!("{} * {}.0", var, multiplier));
        }

        None
    }

    /// Try to convert simple conditional expressions
    fn try_simple_conditional(&self, expr: &str) -> Option<String> {
        // $val >= 0 -> val >= 0.0
        if let Some(captures) = regex::Regex::new(r"\$(\w+)\s*(>=|<=|>|<|==|!=)\s*(\d+)")
            .ok()?
            .captures(expr)
        {
            let var = &captures[1];
            let op = &captures[2];
            let value = &captures[3];
            return Some(format!("{} {} {}.0", var, op, value));
        }

        None
    }

    /// Convert sprintf expressions to Rust format!
    fn convert_sprintf_expression(&self, expr: &str) -> Result<String, ConversionError> {
        // sprintf("%.1f mm", $val) -> format!("{:.1} mm", val)
        if let Some(captures) = regex::Regex::new(r#"sprintf\s*\(\s*"([^"]+)"\s*,\s*\$(\w+)\s*\)"#)
            .ok()
            .and_then(|re| re.captures(expr))
        {
            let format_str = &captures[1];
            let var = &captures[2];

            // Convert Perl format to Rust format
            let rust_format = format_str
                .replace("%.1f", "{:.1}")
                .replace("%d", "{}")
                .replace("%s", "{}");

            return Ok(format!(r#"format!("{}", {})"#, rust_format, var));
        }

        Err(ConversionError::UnsupportedSprintfPattern {
            pattern: expr.to_string(),
        })
    }

    /// Convert function call expressions
    fn convert_function_expression(&self, expr: &str) -> Result<String, ConversionError> {
        // Look for known functions and convert them
        for (perl_func, rust_func) in &self.function_registry {
            if expr.contains(perl_func) {
                let rust_expr = expr.replace(perl_func, rust_func);
                return Ok(rust_expr);
            }
        }

        Err(ConversionError::UnknownFunction {
            function: "unknown".to_string(),
            expression: expr.to_string(),
        })
    }

    /// Convert ternary conditional expressions
    fn convert_ternary_expression(&self, expr: &str) -> Result<String, ConversionError> {
        // $val >= 0 ? $val : 0 -> if val >= 0.0 { val } else { 0.0 }
        if let Some(captures) = regex::Regex::new(r"(.+?)\s*\?\s*(.+?)\s*:\s*(.+)")
            .ok()
            .and_then(|re| re.captures(expr))
        {
            let condition = captures[1].trim();
            let then_part = captures[2].trim();
            let else_part = captures[3].trim();

            // Convert condition
            let rust_condition = condition.replace('$', "");
            let rust_then = then_part.replace('$', "");
            let rust_else = else_part.replace('$', "");

            return Ok(format!(
                "if {} {{ {} }} else {{ {} }}",
                rust_condition, rust_then, rust_else
            ));
        }

        Err(ConversionError::UnsupportedTernaryPattern {
            pattern: expr.to_string(),
        })
    }

    /// Literal translation with variable substitution
    fn literal_translation_with_substitution(&self, expr: &str) -> Result<String, ConversionError> {
        let mut result = expr.to_string();

        // Replace Perl variables with Rust equivalents
        result = result.replace('$', ""); // Remove $ sigils

        // Apply context mappings if needed
        for (perl_pattern, rust_replacement) in &self.context.variable_mappings {
            result = result.replace(perl_pattern, rust_replacement);
        }

        Ok(result)
    }

    /// Generate complete function signature with context if needed
    pub fn generate_function_signature(&self, function_name: &str) -> String {
        let return_type = self.context.expression_type.default_return_type();

        if self.context.has_self_context {
            format!(
                "pub fn {}(val: &TagValue, ctx: &ExifContext) -> {}",
                function_name, return_type
            )
        } else {
            format!(
                "pub fn {}(val: &TagValue) -> {}",
                function_name, return_type
            )
        }
    }

    /// Generate complete function body
    pub fn generate_function_body(
        &mut self,
        ast_info: &AstInfo,
        function_name: &str,
    ) -> Result<String, ConversionError> {
        let body_code = self.convert_ast_to_rust(ast_info)?;
        let signature = self.generate_function_signature(function_name);

        Ok(format!("{} {{\n    {}\n}}", signature, body_code))
    }
}

/// Errors that can occur during AST conversion
#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error(
        "Expression requires manual implementation: {reason}. Suggestion: {fallback_suggestion}"
    )]
    RequiresManualImplementation {
        reason: String,
        fallback_suggestion: String,
    },

    #[error("Expression too complex for AST generation: {complexity}. Suggestion: {suggestion}")]
    TooComplex {
        complexity: String,
        suggestion: String,
    },

    #[error("Unknown expression pattern: {expression}")]
    UnknownPattern { expression: String },

    #[error("Unsupported medium complexity pattern: {pattern}")]
    UnsupportedMediumPattern { pattern: String },

    #[error("Unsupported sprintf pattern: {pattern}")]
    UnsupportedSprintfPattern { pattern: String },

    #[error("Unknown function: {function} in expression: {expression}")]
    UnknownFunction {
        function: String,
        expression: String,
    },

    #[error("Unsupported ternary pattern: {pattern}")]
    UnsupportedTernaryPattern { pattern: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_arithmetic_conversion() {
        let mut converter = PpiConverter::new(ExpressionType::ValueConv);

        assert_eq!(
            converter.try_simple_arithmetic("$val / 100"),
            Some("val / 100.0".to_string())
        );

        assert_eq!(
            converter.try_simple_arithmetic("$val * 25"),
            Some("val * 25.0".to_string())
        );
    }

    #[test]
    fn test_simple_conditional_conversion() {
        let mut converter = PpiConverter::new(ExpressionType::Condition);

        assert_eq!(
            converter.try_simple_conditional("$val >= 0"),
            Some("val >= 0.0".to_string())
        );
    }

    #[test]
    fn test_sprintf_conversion() {
        let converter = PpiConverter::new(ExpressionType::PrintConv);

        let result = converter.convert_sprintf_expression(r#"sprintf("%.1f mm", $val)"#);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("format!"));
    }

    #[test]
    fn test_ternary_conversion() {
        let converter = PpiConverter::new(ExpressionType::ValueConv);

        let result = converter.convert_ternary_expression("$val >= 0 ? $val : 0");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("if"));
    }

    #[test]
    fn test_function_signature_generation() {
        let converter = PpiConverter::new(ExpressionType::PrintConv);

        let signature = converter.generate_function_signature("test_func");
        assert!(signature.contains("pub fn test_func"));
        assert!(signature.contains("-> String"));
    }

    #[test]
    fn test_function_signature_with_context() {
        let mut converter = PpiConverter::new(ExpressionType::ValueConv);
        converter.context.has_self_context = true;

        let signature = converter.generate_function_signature("context_func");
        assert!(signature.contains("ctx: &ExifContext"));
    }
}
