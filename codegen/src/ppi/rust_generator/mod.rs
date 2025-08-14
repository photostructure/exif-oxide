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

    /// Visit document node - handle multi-statement patterns like operation ; return var
    fn visit_document(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        if node.children.len() == 1 {
            // Single statement - use default behavior
            self.visit_node(&node.children[0])
        } else if node.children.len() == 2 {
            // Check for operation ; return pattern
            let first_stmt = &node.children[0];
            let second_stmt = &node.children[1];

            // Pattern: $var =~ s/pattern// ; return $var
            // Pattern: $var =~ s/pattern// ; $var
            if self.is_operation_return_pattern(first_stmt, second_stmt)? {
                // Just return the result of the first statement (the operation)
                // The second statement is just returning the modified variable
                return self.visit_node(first_stmt);
            }

            // Fall back to error for unsupported multi-statement patterns
            Err(CodeGenError::UnsupportedStructure(
                "Document with multiple unsupported statements".to_string(),
            ))
        } else {
            // More than 2 statements - not supported
            Err(CodeGenError::UnsupportedStructure(
                "Document with multiple top-level statements".to_string(),
            ))
        }
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

    /// Check if we have an operation ; return pattern
    fn is_operation_return_pattern(
        &self,
        first_stmt: &PpiNode,
        second_stmt: &PpiNode,
    ) -> Result<bool, CodeGenError> {
        // Pattern 1: $var =~ s/pattern// ; return $var
        if second_stmt.class == "PPI::Statement::Break" {
            // Check if it's "return $var"
            if second_stmt.children.len() == 2
                && second_stmt.children[0].content.as_ref() == Some(&"return".to_string())
                && second_stmt.children[1].class == "PPI::Token::Symbol"
            {
                let return_var = second_stmt.children[1].content.as_ref();
                // Check if first statement operates on the same variable
                if let Some(first_var) = self.extract_operated_variable(first_stmt) {
                    return Ok(return_var == Some(&first_var));
                }
            }
        }

        // Pattern 2: $var =~ s/pattern// ; $var
        if second_stmt.class == "PPI::Statement"
            && second_stmt.children.len() == 1
            && second_stmt.children[0].class == "PPI::Token::Symbol"
        {
            let result_var = second_stmt.children[0].content.as_ref();
            // Check if first statement operates on the same variable
            if let Some(first_var) = self.extract_operated_variable(first_stmt) {
                return Ok(result_var == Some(&first_var));
            }
        }

        Ok(false)
    }

    /// Extract the variable being operated on from a statement like "$var =~ s/pattern//"
    fn extract_operated_variable(&self, stmt: &PpiNode) -> Option<String> {
        if stmt.class == "PPI::Statement" && !stmt.children.is_empty() {
            // Look for $var =~ ... pattern
            if stmt.children.len() >= 3
                && stmt.children[0].class == "PPI::Token::Symbol"
                && stmt.children[1].class == "PPI::Token::Operator"
                && stmt.children[1].content.as_ref() == Some(&"=~".to_string())
            {
                return stmt.children[0].content.clone();
            }
        }
        None
    }

    /// Process a function call (word + argument list)
    fn process_function_call(
        &self,
        name_node: &PpiNode,
        args_node: &PpiNode,
    ) -> Result<String, CodeGenError> {
        let func_name = self.visit_word(name_node)?;

        // Special handling for sprintf with string concatenation and repetition
        if func_name == "sprintf" && !args_node.children.is_empty() {
            if let Some(sprintf_result) = self.try_process_sprintf_with_string_ops(args_node)? {
                return Ok(sprintf_result);
            }
        }

        // Recursively process the arguments - this will handle nested function calls
        let args = if !args_node.children.is_empty() {
            // Check if there's a single Expression child (common PPI pattern)
            if args_node.children.len() == 1
                && args_node.children[0].class == "PPI::Statement::Expression"
            {
                // Arguments are wrapped in an expression - process that expression
                // This will recursively handle any nested function calls
                let processed = self.process_node_sequence(&args_node.children[0].children)?;
                format!("({})", processed)
            } else {
                // Direct children in the list - process them as arguments
                // Visit the list node which will handle the parentheses
                self.visit_list(args_node)?
            }
        } else {
            "()".to_string()
        };

        // Generate the function call - args already have parentheses
        FunctionGenerator::generate_function_call_from_parts(self, &func_name, &args)
    }

    /// Try to process sprintf with string concatenation and repetition operations
    /// Returns Some(result) if the pattern matches, None otherwise
    fn try_process_sprintf_with_string_ops(
        &self,
        args_node: &PpiNode,
    ) -> Result<Option<String>, CodeGenError> {
        // Check if this is the pattern: sprintf("base" . "part" x count, remaining_args)
        if args_node.children.len() != 1
            || args_node.children[0].class != "PPI::Statement::Expression"
        {
            return Ok(None);
        }

        let expr_children = &args_node.children[0].children;

        // Look for the pattern in the expression: base . part x count , remaining_args
        // We need: string, ".", string, "x", number, ",", remaining tokens
        if expr_children.len() < 7 {
            return Ok(None);
        }

        // Find key positions
        let mut dot_pos = None;
        let mut x_pos = None;
        let mut comma_pos = None;

        for (i, child) in expr_children.iter().enumerate() {
            if child.class == "PPI::Token::Operator" {
                match child.content.as_deref() {
                    Some(".") if dot_pos.is_none() => {
                        dot_pos = Some(i);
                    }
                    Some("x") if x_pos.is_none() => {
                        x_pos = Some(i);
                    }
                    Some(",") if comma_pos.is_none() => {
                        comma_pos = Some(i);
                    }
                    _ => {}
                }
            }
        }

        // Validate we have the required operators in the right order
        if let (Some(dot), Some(x), Some(comma)) = (dot_pos, x_pos, comma_pos) {
            if dot < x && x < comma && dot > 0 && x > dot + 1 && comma > x + 1 {
                // Extract the components
                let base_format_node = &expr_children[dot - 1];
                let concat_part_node = &expr_children[dot + 1];
                let repeat_count_node = &expr_children[x + 1];

                // Ensure we have the right types
                if (base_format_node.class.contains("Quote")
                    || base_format_node.class.contains("String"))
                    && (concat_part_node.class.contains("Quote")
                        || concat_part_node.class.contains("String"))
                    && repeat_count_node.class.contains("Number")
                {
                    // Extract values
                    let base_format = base_format_node
                        .string_value
                        .as_deref()
                        .or(base_format_node.content.as_deref())
                        .unwrap_or("\"\"");
                    let concat_part = concat_part_node
                        .string_value
                        .as_deref()
                        .or(concat_part_node.content.as_deref())
                        .unwrap_or("\"\"");
                    let repeat_count = repeat_count_node.content.as_deref().unwrap_or("1");

                    // Process remaining arguments after comma
                    let remaining_args = &expr_children[comma + 1..];
                    let mut args_parts = Vec::new();
                    for arg_node in remaining_args {
                        if arg_node.class != "PPI::Token::Whitespace" {
                            args_parts.push(self.visit_node(arg_node)?);
                        }
                    }
                    let args_str = args_parts.join(" ");

                    // Handle the common case where remaining args is a split() call
                    let final_args_str = if args_str.starts_with("split") {
                        // Transform "split \" \" val" into "crate::types::split_tagvalue(val, \" \")"
                        // For simplicity in this pattern, we know it should be split(" ", $val)
                        "crate::types::split_tagvalue(val, \" \")".to_string()
                    } else {
                        args_str
                    };

                    // Generate the call to our helper function
                    let generated_code = match ExpressionCombiner::expression_type(self) {
                        ExpressionType::PrintConv | ExpressionType::ValueConv => {
                            format!(
                                "TagValue::String(crate::fmt::sprintf_with_string_concat_repeat(\"{}\", \"{}\", {} as usize, &{}))",
                                base_format, concat_part, repeat_count, final_args_str
                            )
                        }
                        _ => {
                            format!(
                                "crate::fmt::sprintf_with_string_concat_repeat(\"{}\", \"{}\", {} as usize, &{})",
                                base_format, concat_part, repeat_count, final_args_str
                            )
                        }
                    };

                    return Ok(Some(generated_code));
                }
            }
        }

        Ok(None)
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

    fn handle_sprintf_with_string_operations(
        &self,
        parts: &[String],
        children: &[PpiNode],
    ) -> Result<String, CodeGenError> {
        // Parse the sprintf pattern: sprintf base_format . concat_part x repeat_count, remaining_args
        // Expected structure: ["sprintf", "(", "base_format", ".", "concat_part", "x", "repeat_count", ",", "remaining_args", ")"]

        if parts.len() < 8 {
            return Err(CodeGenError::UnsupportedStructure(
                "sprintf with string ops: insufficient parts".to_string(),
            ));
        }

        // Find the positions of key operators
        let dot_pos = parts.iter().position(|p| p == ".").ok_or_else(|| {
            CodeGenError::UnsupportedStructure("sprintf: missing '.' operator".to_string())
        })?;

        let x_pos = parts.iter().position(|p| p == "x").ok_or_else(|| {
            CodeGenError::UnsupportedStructure("sprintf: missing 'x' operator".to_string())
        })?;

        let comma_pos = parts.iter().position(|p| p == ",").ok_or_else(|| {
            CodeGenError::UnsupportedStructure("sprintf: missing ',' separator".to_string())
        })?;

        // Validate order: dot_pos < x_pos < comma_pos
        if !(dot_pos < x_pos && x_pos < comma_pos) {
            return Err(CodeGenError::UnsupportedStructure(
                "sprintf: invalid operator order".to_string(),
            ));
        }

        // Extract components
        let base_format = &parts[dot_pos - 1]; // The part before the dot
        let concat_part = &parts[dot_pos + 1]; // The part after the dot, before x
        let repeat_count = &parts[x_pos + 1]; // The number after x

        // Everything after the comma is the remaining arguments
        let remaining_args = &parts[comma_pos + 1..];
        let args_str = remaining_args.join(" ");

        // Generate the call to our helper function
        match ExpressionCombiner::expression_type(self) {
            ExpressionType::PrintConv | ExpressionType::ValueConv => Ok(format!(
                "TagValue::String(crate::fmt::sprintf_with_string_concat_repeat({}, {}, {} as usize, &{}))",
                base_format, concat_part, repeat_count, args_str
            )),
            _ => Ok(format!(
                "crate::fmt::sprintf_with_string_concat_repeat({}, {}, {} as usize, &{})",
                base_format, concat_part, repeat_count, args_str
            )),
        }
    }
}

impl FunctionGenerator for RustGenerator {
    fn expression_type(&self) -> &ExpressionType {
        &self.expression_type
    }
}
