//! Core Rust code generator from PPI structures
//!
//! Contains the main RustGenerator struct and orchestrates the generation
//! of complete Rust functions from PPI AST nodes.
//!
//! Trust ExifTool: Generated code preserves exact Perl evaluation semantics.

use indoc::formatdoc;
use std::fmt::Write;

use crate::ppi::rust_generator::{
    errors::CodeGenError, expressions::ExpressionCombiner, functions::FunctionGenerator,
    pattern_matching, signature, visitor::PpiVisitor,
};
use crate::ppi::types::{ExpressionType, PpiNode};

/// Generate Rust function from PPI AST node
pub struct RustGenerator {
    pub expression_type: ExpressionType,
    pub function_name: String,
    pub original_expression: String,
}

impl RustGenerator {
    /// Create a new RustGenerator instance
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

    /// Format a Perl expression as properly escaped Rust documentation
    pub fn format_perl_expression_doc(expression: &str) -> String {
        // Prefix each line with /// to make it valid Rust doc comment
        // Trim leading whitespace from each line for cleaner formatting
        let doc_expression = expression
            .lines()
            .map(|line| format!("/// {}", line.trim_start()))
            .collect::<Vec<_>>()
            .join("\n");

        formatdoc! {r#"
            /// Original perl expression:
            /// ``` perl
            {}
            /// ```
        "#, doc_expression}
    }

    /// Generate complete function from PPI AST
    pub fn generate_function(&self, ast: &PpiNode) -> Result<String, CodeGenError> {
        let mut code = String::new();

        // Function header with documentation
        let doc_comment = Self::format_perl_expression_doc(&self.original_expression);
        code.push_str(&doc_comment);

        // Function signature
        let signature = signature::generate_signature(&self.expression_type, &self.function_name);
        writeln!(code, "{signature} {{")?;

        // Function body
        let body = self.generate_body(ast)?;
        for line in body.lines() {
            writeln!(code, "    {line}")?;
        }

        writeln!(code, "}}")?;

        Ok(code)
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

    /// Visit document node - handle multi-statement patterns like operation ; return var
    pub fn visit_document(&self, node: &PpiNode) -> Result<String, CodeGenError> {
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
            if let Err(_) = pattern_matching::check_node_complexity(child) {
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
    pub fn visit_statement(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        self.process_node_sequence(&node.children)
    }

    /// Visit expression node - handles complex expressions with function composition
    /// PPI::Statement::Expression (4,172 occurrences) - Essential for complex expressions
    pub fn visit_expression(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        self.process_node_sequence(&node.children)
    }

    /// Process a sequence of child nodes with intelligent pattern recognition
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

        // First check for ternary pattern: condition ? true_expr : false_expr
        if let Some(ternary_result) = self.try_process_ternary_pattern(children)? {
            return Ok(ternary_result);
        }

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

            // Pattern: hash dereference (Cast + Symbol + Subscript for $$self{key})
            if children[i].class == "PPI::Token::Cast"
                && children[i].content.as_ref() == Some(&"$".to_string())
                && i + 2 < children.len()
                && children[i + 1].class == "PPI::Token::Symbol"
                && children[i + 1].content.as_ref() == Some(&"$self".to_string())
                && children[i + 2].class == "PPI::Structure::Subscript"
            {
                // Extract the key from the subscript structure
                if let Some(key) = self.extract_subscript_key(&children[i + 2])? {
                    let hash_access = format!("ctx.get(\"{}\").unwrap_or_default()", key);
                    processed.push(hash_access);
                    i += 3;
                    continue;
                }
            }

            // Default: visit the node normally
            processed.push(self.visit_node(&children[i])?);
            i += 1;
        }

        // Combine the processed parts - this handles multi-arg functions without parens
        self.combine_processed_parts(&processed, children)
    }

    /// Try to process ternary pattern: condition ? true_expr : false_expr
    /// Returns Some(result) if ternary pattern found, None otherwise
    fn try_process_ternary_pattern(
        &self,
        children: &[PpiNode],
    ) -> Result<Option<String>, CodeGenError> {
        // Find the ? and : operators to identify ternary structure
        let mut question_idx = None;
        let mut colon_idx = None;

        for (i, child) in children.iter().enumerate() {
            if child.class == "PPI::Token::Operator" {
                if let Some(content) = &child.content {
                    if content == "?" && question_idx.is_none() {
                        question_idx = Some(i);
                    } else if content == ":" && question_idx.is_some() && colon_idx.is_none() {
                        colon_idx = Some(i);
                        break; // Found both operators
                    }
                }
            }
        }

        // If we found both ? and : operators, this is a ternary expression
        if let (Some(q_idx), Some(c_idx)) = (question_idx, colon_idx) {
            if q_idx < c_idx {
                // Extract the three parts: condition, true_expr, false_expr
                let condition_nodes = &children[..q_idx];
                let true_expr_nodes = &children[q_idx + 1..c_idx];
                let false_expr_nodes = &children[c_idx + 1..];

                // Process each part
                let condition = self.process_node_sequence(condition_nodes)?;
                let true_expr = self.process_node_sequence(true_expr_nodes)?;
                let false_expr = self.process_node_sequence(false_expr_nodes)?;

                // Generate Rust if-else expression
                let result = format!(
                    "if {} {{ {} }} else {{ {} }}",
                    condition, true_expr, false_expr
                );
                return Ok(Some(result));
            }
        }

        Ok(None)
    }

    /// Process function call with arguments
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
        _args_node: &PpiNode,
    ) -> Result<Option<String>, CodeGenError> {
        // This method handles complex sprintf patterns - implementation delegated to mod.rs for now
        // to minimize disruption during refactoring
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
                    "crate::text::regex_replace_all(val, \"{}\", \"{}\")",
                    pattern, replacement
                ))
            } else {
                Ok(format!(
                    "crate::text::regex_replace(val, \"{}\", \"{}\")",
                    pattern, replacement
                ))
            }
        } else {
            // Complex pattern - use regex
            if is_global {
                Ok(format!(
                    "crate::text::regex_replace_all(val, r\"{}\", \"{}\")",
                    pattern, replacement
                ))
            } else {
                Ok(format!(
                    "crate::text::regex_replace(val, r\"{}\", \"{}\")",
                    pattern, replacement
                ))
            }
        }
    }

    /// Combine processed parts using expression combiner logic
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
        ExpressionCombiner::combine_statement_parts(self, &parts, original_children)
    }

    /// Extract the key from a subscript structure (e.g., {FocalUnits} -> "FocalUnits")
    fn extract_subscript_key(
        &self,
        subscript_node: &PpiNode,
    ) -> Result<Option<String>, CodeGenError> {
        // The subscript should have children that represent the key
        if subscript_node.children.is_empty() {
            return Ok(None);
        }

        // For {FocalUnits}, we expect a single expression child containing the word
        if subscript_node.children.len() == 1
            && subscript_node.children[0].class == "PPI::Statement::Expression"
        {
            let expr_node = &subscript_node.children[0];
            if !expr_node.children.is_empty() && expr_node.children[0].class == "PPI::Token::Word" {
                if let Some(key) = &expr_node.children[0].content {
                    return Ok(Some(key.clone()));
                }
            }
        }

        Ok(None)
    }
}
