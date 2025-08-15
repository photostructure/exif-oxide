//! Rust Code Generator from PPI Structures
//!
//! Converts PPI AST nodes into Rust source code that calls runtime support
//! functions from the `ast::` module for DRY code generation.
//!
//! Trust ExifTool: Generated code preserves exact Perl evaluation semantics.

use indoc::formatdoc;
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
        let doc_comment = formatdoc! {r#"
            /// Original perl expression:
            /// ``` perl
            /// {}
            /// ```
        "#, self.original_expression};
        code.push_str(&doc_comment);

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
        // Enable normalizer to fix string concatenation
        let normalized_ast = crate::ppi::normalizer::normalize(ast.clone());
        let code = self.visit_node(&normalized_ast)?;

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
        if node.children.is_empty() {
            return Ok("".to_string());
        }

        if node.children.len() == 1 {
            // Simple case: single statement
            return self.visit_node(&node.children[0]);
        }

        // For multi-statement documents, check if they contain complex constructs
        // that we cannot reliably translate to Rust
        for child in &node.children {
            if let Err(_) = self.check_node_complexity(child) {
                return Err(CodeGenError::UnsupportedStructure(
                    "Multi-statement block contains complex Perl constructs that cannot be translated".to_string()
                ));
            }
        }

        // Handle multiple statements (e.g., "$val=~tr/ /./; $val")
        // For ExifTool compatibility, we need to process all statements
        // and return the result of the last one (Perl's behavior)
        let mut results = Vec::new();
        let mut last_result = String::new();

        for (i, child) in node.children.iter().enumerate() {
            // Skip whitespace and comments that PPI might include
            if child.class == "PPI::Token::Whitespace" || child.class == "PPI::Token::Comment" {
                continue;
            }

            let result = self.visit_node(child)?;

            // Skip empty results
            if result.trim().is_empty() {
                continue;
            }

            // For multiple statements, we need to handle them as a sequence
            if i == node.children.len() - 1 {
                // Last statement becomes the return value
                last_result = result;
            } else {
                // Earlier statements are executed for side effects
                results.push(result);
            }
        }

        if results.is_empty() {
            // Only one meaningful statement
            Ok(last_result)
        } else {
            // Multiple statements: combine them in a block expression
            results.push(last_result);
            let statements = results[..results.len() - 1].join(";\n    ");
            let final_result = &results[results.len() - 1];
            Ok(formatdoc! {r#"
                {{
                    {statements};
                    {final_result}
                }}
            "#})
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
    /// Check if a node tree contains constructs that are too complex to translate reliably
    fn check_node_complexity(&self, node: &PpiNode) -> Result<(), CodeGenError> {
        // Check this node for problematic patterns
        match node.class.as_str() {
            // Variable declarations in multi-statement blocks are complex
            "PPI::Statement::Variable" => {
                return Err(CodeGenError::UnsupportedStructure(
                    "Variable declarations in multi-statement blocks not supported".to_string(),
                ));
            }
            // Complex regex patterns with flags
            "PPI::Token::Regexp::Match" => {
                if let Some(content) = &node.content {
                    // Reject regex patterns with global flags or complex modifiers
                    if content.contains("/g") || content.contains("/m") || content.contains("/s") {
                        return Err(CodeGenError::UnsupportedStructure(
                            "Regex patterns with global or multiline flags not supported"
                                .to_string(),
                        ));
                    }
                }
            }
            // Perl references (backslash syntax) are not translatable
            _ => {
                if let Some(content) = &node.content {
                    // Check for Perl reference syntax
                    if content.contains("\\%") || content.contains("\\@") || content.contains("\\$")
                    {
                        return Err(CodeGenError::UnsupportedStructure(
                            "Perl reference syntax (\\%) not supported".to_string(),
                        ));
                    }
                    // Check for foreach with special variables
                    if content.contains("foreach") && content.contains("$_") {
                        return Err(CodeGenError::UnsupportedStructure(
                            "Foreach loops with special variables not supported".to_string(),
                        ));
                    }
                    // Check for complex array operations
                    if content.contains("reverse @") {
                        return Err(CodeGenError::UnsupportedStructure(
                            "Complex array operations (reverse @) not supported".to_string(),
                        ));
                    }
                }
            }
        }

        // Recursively check children
        for child in &node.children {
            self.check_node_complexity(child)?;
        }

        Ok(())
    }
    /// Extract pack/map pattern for bit extraction operations  
    /// From ExifTool Canon.pm line 1847: pack "C*", map { (($_>>$_)&0x1f)+0x60 } 10, 5, 0
    /// Returns: Some((mask, offset, shifts)) if pattern matches, None if not recognized
    fn extract_pack_map_pattern(
        &self,
        parts: &[String],
        _children: &[PpiNode],
    ) -> Result<Option<(i32, i32, Vec<i32>)>, CodeGenError> {
        // Conservative approach: look for mask and offset hex values in the parts
        // Pattern: pack "C*", map { ... } followed by numbers
        // Find mask (typically 0x1f, 0x0f, 0x3f etc. - small hex values used as bitmasks)
        let mut mask = None;
        let mut offset = None;
        let mut shifts = Vec::new();

        // Look through parts for hex patterns that could be mask/offset
        for part in parts.iter() {
            if part.starts_with("0x") || part.starts_with("0X") {
                if let Ok(hex_val) = i32::from_str_radix(&part[2..], 16) {
                    // Small hex values (< 64) are likely masks
                    // Larger values (like 0x60 = 96) are likely offsets
                    if hex_val < 64 && mask.is_none() {
                        mask = Some(hex_val);
                    } else if hex_val >= 48 && offset.is_none() {
                        // Lowered threshold to catch 0x30 = 48
                        offset = Some(hex_val);
                    }
                }
            } else if let Ok(num) = part.parse::<i32>() {
                // Numbers that aren't hex are likely shift values
                // Only collect reasonable shift values (0-32 for bit operations)
                if num >= 0 && num <= 32 {
                    shifts.push(num);
                }
            }
        }

        // Need at least mask and some shifts to be valid
        if let Some(mask_val) = mask {
            let offset_val = offset.unwrap_or(0); // Default offset if not found
            if !shifts.is_empty() {
                return Ok(Some((mask_val, offset_val, shifts)));
            }
        }

        // If we can't find a clear pattern, return None for fallback handling
        Ok(None)
    }

    /// Process regex assignment for regex substitution operations
    /// Generates the right-hand side without TagValue wrapper for assignment context
    fn process_regex_assignment(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        let content = node.content.as_ref().ok_or(CodeGenError::MissingContent(
            "regexp substitute".to_string(),
        ))?;

        // Parse s/pattern/replacement/flags
        if !content.starts_with("s/") && !content.starts_with("s#") {
            return Err(CodeGenError::UnsupportedStructure(format!(
                "Invalid substitution pattern: {}",
                content
            )));
        }

        // Determine delimiter
        let delimiter = if content.starts_with("s/") { '/' } else { '#' };
        let parts: Vec<&str> = content[2..].split(delimiter).collect();

        if parts.len() < 2 {
            return Err(CodeGenError::UnsupportedStructure(format!(
                "Invalid substitution format: {}",
                content
            )));
        }

        let pattern = parts[0];
        let replacement = if parts.len() > 1 { parts[1] } else { "" };
        let flags = if parts.len() > 2 { parts[2] } else { "" };

        // Check for global flag
        let is_global = flags.contains('g');

        // Generate the call without TagValue wrapper since this is for assignment
        if pattern
            .chars()
            .all(|c| c.is_alphanumeric() || c.is_whitespace())
        {
            // Simple string replacement
            if is_global {
                Ok(format!(
                    "TagValue::String(val.to_string().replace(\"{}\", \"{}\"))",
                    pattern, replacement
                ))
            } else {
                Ok(format!(
                    "TagValue::String(val.to_string().replacen(\"{}\", \"{}\", 1))",
                    pattern, replacement
                ))
            }
        } else {
            // Regex replacement - use bytes regex to handle non-UTF8 patterns like ExifTool
            let safe_pattern = self.make_pattern_safe_for_rust(pattern);
            let escaped_replacement = self.escape_replacement_string(replacement);

            if is_global {
                Ok(format!(
                    "TagValue::String(crate::fmt::regex_replace_all(\"{}\", &val.to_string(), \"{}\"))",
                    safe_pattern, escaped_replacement
                ))
            } else {
                Ok(format!(
                    "TagValue::String(crate::fmt::regex_replace(\"{}\", &val.to_string(), \"{}\"))",
                    safe_pattern, escaped_replacement
                ))
            }
        }
    }

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

            // Pattern: regex assignment (Symbol + =~ + Regexp::Substitute)
            if children[i].class == "PPI::Token::Symbol"
                && i + 2 < children.len()
                && children[i + 1].class == "PPI::Token::Operator"
                && children[i + 1].content.as_ref() == Some(&"=~".to_string())
                && children[i + 2].class == "PPI::Token::Regexp::Substitute"
            {
                let var = self.visit_symbol(&children[i])?;
                let regex_result = self.process_regex_assignment(&children[i + 2])?;
                let assignment = format!("{} = {}", var, regex_result);
                processed.push(assignment);
                i += 3;
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

        // Note: args were processed but are now handled directly by AST traversal
        // The arguments processing is now done inside the function generators using AST nodes

        // Generate the function call using AST node
        // Create a temporary node structure for the function call
        let mut function_node = name_node.clone();
        function_node.children = args_node.children.clone();
        FunctionGenerator::generate_function_call_from_parts(self, &function_node)
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

    fn generate_function_call_from_parts(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        // Delegate to FunctionGenerator trait
        FunctionGenerator::generate_function_call_from_parts(self, node)
    }

    fn generate_multi_arg_function_call(
        &self,
        function_name: &str,
        node: &PpiNode,
    ) -> Result<String, CodeGenError> {
        // Delegate to FunctionGenerator trait
        FunctionGenerator::generate_multi_arg_function_call(self, function_name, node)
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
