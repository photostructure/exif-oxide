//! Rust Code Generator from PPI Structures
//!
//! Converts PPI AST nodes into Rust source code that calls runtime support
//! functions from the `ast::` module for DRY code generation.
//!
//! Trust ExifTool: Generated code preserves exact Perl evaluation semantics.

use std::fmt::Write;

// Module exports
pub mod errors;
pub mod expressions;
pub mod functions;
pub mod visitor;

#[cfg(test)]
pub mod tests;

// Re-export everything for backward compatibility
pub use errors::CodeGenError;
pub use expressions::ExpressionCombiner;
pub use functions::FunctionGenerator;
pub use visitor::PpiVisitor;

// Import types
use crate::ppi::types::*;

/// Generate Rust function from PPI AST node
pub struct RustGenerator {
    expression_type: ExpressionType,
    function_name: String,
    original_expression: String,
}

impl RustGenerator {
    pub fn new(
        expression_type: ExpressionType,
        function_name: String,
        original_expression: String,
    ) -> Self {
        Self {
            expression_type,
            function_name,
            original_expression,
        }
    }

    /// Generate complete function from PPI AST
    pub fn generate_function(&self, ast: &PpiNode) -> Result<String, CodeGenError> {
        let mut code = String::new();

        // Function header with documentation
        writeln!(code, "/// Generated from PPI AST")?;
        writeln!(code, "/// Original: {}", self.original_expression)?;
        writeln!(code, "/// ExifTool AST parsing")?;

        // Function signature
        let signature = self.generate_signature();
        writeln!(code, "{signature} {{")?;

        // Function body
        let body = self.generate_body(ast)?;
        for line in body.lines() {
            writeln!(code, "    {line}")?;
        }

        writeln!(code, "}}")?;

        Ok(code)
    }

    /// Generate function signature
    fn generate_signature(&self) -> String {
        let return_type = self.expression_type.return_type();

        match self.expression_type {
            ExpressionType::Condition => {
                format!(
                    "pub fn {}(val: &TagValue, ctx: &ExifContext) -> {}",
                    self.function_name, return_type
                )
            }
            ExpressionType::ValueConv => {
                format!(
                    "pub fn {}(val: &TagValue) -> Result<TagValue, crate::types::ExifError>",
                    self.function_name
                )
            }
            ExpressionType::PrintConv => {
                format!(
                    "pub fn {}(val: &TagValue) -> {}",
                    self.function_name, return_type
                )
            }
        }
    }

    /// Generate function body from AST using recursive visitor pattern
    fn generate_body(&self, ast: &PpiNode) -> Result<String, CodeGenError> {
        let code = self.visit_node(ast)?;

        // Wrap the generated expression based on expression type
        match self.expression_type {
            ExpressionType::ValueConv => Ok(format!("Ok({})", code)),
            ExpressionType::PrintConv => Ok(code),
            ExpressionType::Condition => Ok(code),
        }
    }
}

// Implement all the traits on RustGenerator
impl PpiVisitor for RustGenerator {
    fn expression_type(&self) -> &ExpressionType {
        &self.expression_type
    }

    /// Visit statement node - processes children and combines them intelligently
    fn visit_statement(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        self.process_node_sequence(&node.children)
    }

    /// Visit expression node - handles complex expressions with function composition
    /// PPI::Statement::Expression (4,172 occurrences) - Essential for complex expressions
    fn visit_expression(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        self.process_node_sequence(&node.children)
    }
}

// Helper methods for RustGenerator
impl RustGenerator {
    /// Process a sequence of nodes recursively, recognizing patterns
    fn process_node_sequence(&self, children: &[PpiNode]) -> Result<String, CodeGenError> {
        if children.is_empty() {
            return Ok("".to_string());
        }

        // Single child - just visit it
        if children.len() == 1 {
            return self.visit_node(&children[0]);
        }

        #[cfg(test)]
        eprintln!("DEBUG process_node_sequence: {} children", children.len());
        
        // Look for patterns in the sequence
        let mut processed = Vec::new();
        let mut i = 0;

        while i < children.len() {
            // Skip whitespace
            if children[i].class == "PPI::Token::Whitespace" {
                i += 1;
                continue;
            }

            // Pattern: function call with parentheses (Word + Structure::List)
            if children[i].is_word()
                && i + 1 < children.len()
                && children[i + 1].class == "PPI::Structure::List"
            {
                let func_result = self.process_function_call(&children[i], &children[i + 1])?;
                processed.push(func_result);
                i += 2;
                continue;
            }

            // Default: visit the node normally
            processed.push(self.visit_node(&children[i])?);
            i += 1;
        }

        // Combine the processed parts - this handles multi-arg functions without parens
        self.combine_processed_parts(&processed, children)
    }

    /// Process a function call (word + argument list)
    fn process_function_call(
        &self,
        name_node: &PpiNode,
        args_node: &PpiNode,
    ) -> Result<String, CodeGenError> {
        let func_name = self.visit_word(name_node)?;

        // Recursively process the arguments
        let args = if !args_node.children.is_empty() {
            // Check if there's a single Expression child (common PPI pattern)
            if args_node.children.len() == 1
                && args_node.children[0].class == "PPI::Statement::Expression"
            {
                // Arguments are wrapped in an expression - process that expression
                self.process_node_sequence(&args_node.children[0].children)?
            } else {
                // Direct children in the list - process them as arguments
                // Preserve exact formatting for function argument parsing
                let mut result = String::new();
                let mut need_space = false;

                for child in &args_node.children {
                    if child.class == "PPI::Token::Operator"
                        && child.content.as_ref().map_or(false, |c| c == ",")
                    {
                        result.push_str(", ");
                        need_space = false;
                    } else {
                        if need_space {
                            result.push(' ');
                        }
                        result.push_str(&self.visit_node(child)?);
                        need_space = true;
                    }
                }
                result
            }
        } else {
            "()".to_string()
        };

        // Generate the function call using the FunctionGenerator trait method
        FunctionGenerator::generate_function_call_from_parts(
            self,
            &func_name,
            &format!("({})", args),
        )
    }

    /// Combine processed parts into final code
    fn combine_processed_parts(
        &self,
        parts: &[String],
        original_children: &[PpiNode],
    ) -> Result<String, CodeGenError> {
        // Filter out empty parts
        let parts: Vec<_> = parts.iter().filter(|p| !p.is_empty()).cloned().collect();

        if parts.is_empty() {
            return Ok("".to_string());
        }

        if parts.len() == 1 {
            return Ok(parts[0].clone());
        }

        // Use combine_statement_parts which handles multi-arg functions without parens
        self.combine_statement_parts(&parts, original_children)
    }
}

impl ExpressionCombiner for RustGenerator {
    fn expression_type(&self) -> &ExpressionType {
        &self.expression_type
    }

    fn generate_function_call_from_parts(
        &self,
        function_name: &str,
        args_part: &str,
    ) -> Result<String, CodeGenError> {
        // Delegate to FunctionGenerator trait
        FunctionGenerator::generate_function_call_from_parts(self, function_name, args_part)
    }

    fn generate_multi_arg_function_call(
        &self,
        function_name: &str,
        args: &[String],
    ) -> Result<String, CodeGenError> {
        // Delegate to FunctionGenerator trait
        FunctionGenerator::generate_multi_arg_function_call(self, function_name, args)
    }

    fn generate_function_call_without_parens(
        &self,
        function_name: &str,
        arg: &str,
    ) -> Result<String, CodeGenError> {
        // Delegate to FunctionGenerator trait
        FunctionGenerator::generate_function_call_without_parens(self, function_name, arg)
    }
}

impl FunctionGenerator for RustGenerator {
    fn expression_type(&self) -> &ExpressionType {
        &self.expression_type
    }
}
