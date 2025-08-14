//! PPI AST visitor pattern implementation
//!
//! This module contains the visitor pattern implementation for traversing
//! PPI AST nodes and generating Rust code from them.

use super::errors::CodeGenError;
use crate::ppi::types::*;

/// Trait for visiting PPI AST nodes and generating Rust code
pub trait PpiVisitor {
    fn expression_type(&self) -> &ExpressionType;

    /// Recursive visitor for PPI nodes - dispatches based on node class
    fn visit_node(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        match node.class.as_str() {
            "PPI::Document" => self.visit_document(node),
            "PPI::Statement" => self.visit_statement(node),
            // Task A: Critical Foundation Tokens (Phase 1)
            "PPI::Statement::Expression" => self.visit_expression(node),
            "PPI::Token::Cast" => self.visit_cast(node),
            "PPI::Structure::Subscript" => self.visit_subscript(node),
            "PPI::Token::Regexp::Match" => self.visit_regexp_match(node),
            // Task B: Numeric & String Operations (Phase 2)
            "PPI::Token::Number::Hex" => self.visit_number_hex(node),
            "PPI::Token::Number::Float" => self.visit_number(node), // Handle float the same as number
            "PPI::Statement::Variable" => self.visit_variable(node),
            "PPI::Token::Regexp::Substitute" => self.visit_regexp_substitute(node),
            // Existing supported tokens
            "PPI::Token::Symbol" => self.visit_symbol(node),
            "PPI::Token::Operator" => self.visit_operator(node),
            "PPI::Token::Number" => self.visit_number(node),
            "PPI::Token::Quote::Double" | "PPI::Token::Quote::Single" => self.visit_string(node),
            "PPI::Token::Word" => self.visit_word(node),
            "PPI::Structure::List" => self.visit_list(node),
            _ => Err(CodeGenError::UnsupportedToken(node.class.clone())),
        }
    }

    /// Visit document node (top level)
    fn visit_document(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        if node.children.len() == 1 {
            self.visit_node(&node.children[0])
        } else {
            Err(CodeGenError::UnsupportedStructure(
                "Document with multiple top-level statements".to_string(),
            ))
        }
    }

    /// Visit statement node - processes children and combines them intelligently
    fn visit_statement(&self, node: &PpiNode) -> Result<String, CodeGenError>;

    /// Visit symbol node (variables like $val, $$self{Field})
    fn visit_symbol(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        let content = node
            .content
            .as_ref()
            .ok_or(CodeGenError::MissingContent("symbol".to_string()))?;

        if node.is_self_reference() {
            if let Some(field) = node.extract_self_field() {
                Ok(format!("ctx.get(\"{field}\").unwrap_or_default()"))
            } else {
                Err(CodeGenError::InvalidSelfReference(content.clone()))
            }
        } else if content == "$val" {
            Ok("val".to_string())
        } else if content == "$valPt" {
            Ok("val_pt".to_string())
        } else {
            // Generic variable
            Ok(content.trim_start_matches('$').to_string())
        }
    }

    /// Visit operator node
    fn visit_operator(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        let op = node
            .content
            .as_ref()
            .ok_or(CodeGenError::MissingContent("operator".to_string()))?;

        // Return the operator - parent will decide how to use it
        Ok(op.clone())
    }

    /// Visit number node - enhanced for better float and scientific notation handling
    fn visit_number(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        if let Some(num) = node.numeric_value {
            // For code generation, use appropriate literal format
            if num.fract() == 0.0 && num.abs() < 1e10 {
                // Integer value within reasonable range
                Ok(format!("{}", num as i64))
            } else {
                // Float value or large number - ensure Rust float literal format
                let num_str = num.to_string();
                // Add explicit float suffix if not present for clarity
                if !num_str.contains('e') && !num_str.contains('.') {
                    Ok(format!("{}.0", num_str))
                } else {
                    Ok(num_str)
                }
            }
        } else if let Some(content) = &node.content {
            // Handle special numeric formats
            if content.contains('e') || content.contains('E') {
                // Scientific notation - ensure proper format
                Ok(content.to_lowercase())
            } else if content.contains('.') {
                // Decimal number - preserve as-is
                Ok(content.clone())
            } else {
                // Integer - validate and return
                if content
                    .chars()
                    .all(|c| c.is_ascii_digit() || c == '-' || c == '+')
                {
                    Ok(content.clone())
                } else {
                    Err(CodeGenError::InvalidNumber(content.clone()))
                }
            }
        } else {
            Err(CodeGenError::MissingContent("number".to_string()))
        }
    }

    /// Visit string node (quoted strings)
    fn visit_string(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        let string_value = node
            .string_value
            .as_ref()
            .or(node.content.as_ref())
            .ok_or(CodeGenError::MissingContent("string".to_string()))?;

        // Handle simple variable interpolation
        if string_value.contains("$val") && string_value.matches('$').count() == 1 {
            let template = string_value.replace("$val", "{}");
            Ok(format!("format!(\"{}\", val)", template))
        } else {
            // Simple string literal
            Ok(format!("\"{}\"", string_value.replace('\"', "\\\"")))
        }
    }

    /// Visit word node (function names, keywords)
    fn visit_word(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        let word = node
            .content
            .as_ref()
            .ok_or(CodeGenError::MissingContent("word".to_string()))?;

        // Handle special Perl keywords
        match word.as_str() {
            "undef" => {
                // Perl's undef translates to appropriate default value
                match self.expression_type() {
                    ExpressionType::PrintConv => {
                        Ok("TagValue::String(\"\".to_string())".to_string())
                    }
                    ExpressionType::ValueConv => {
                        Ok("TagValue::String(\"\".to_string())".to_string())
                    }
                    ExpressionType::Condition => Ok("false".to_string()),
                }
            }
            _ => Ok(word.clone()),
        }
    }

    /// Visit list node (function arguments, parentheses)
    fn visit_list(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        // Simple delegation: visit each child and let the dispatcher handle it
        let mut args = Vec::new();
        for child in &node.children {
            // Skip comma operators - they're just separators
            if child.class == "PPI::Token::Operator"
                && child.content.as_ref().map_or(false, |c| c == ",")
            {
                continue;
            }
            args.push(self.visit_node(child)?);
        }
        Ok(format!("({})", args.join(", ")))
    }

    // Task A: Critical Foundation Tokens (Phase 1) - P07: PPI Enhancement

    /// Visit expression node - handles complex expressions with function composition
    /// PPI::Statement::Expression (4,172 occurrences) - Essential for complex expressions
    fn visit_expression(&self, node: &PpiNode) -> Result<String, CodeGenError>;

    /// Visit cast node - handles dereference operators $$self{Field}
    /// PPI::Token::Cast (2,420 occurrences) - Required for $$self{Field} pattern
    fn visit_cast(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        let content = node
            .content
            .as_ref()
            .ok_or(CodeGenError::MissingContent("cast".to_string()))?;

        // Handle $$self{Field} pattern - most common cast usage in ExifTool
        if content.starts_with("$$self{") && content.ends_with('}') {
            let field_name = &content[7..content.len() - 1]; // Remove $$self{ and }
            Ok(format!("ctx.get(\"{}\").unwrap_or_default()", field_name))
        } else if content.starts_with("$$self") {
            // Handle $$self direct reference
            Ok("ctx.get_self().unwrap_or_default()".to_string())
        } else if content.starts_with("$$valPt") {
            // Handle $$valPt pattern for binary data
            Ok("val_pt".to_string())
        } else if content.starts_with("$$") {
            // Generic dereference - handle as string for now
            Ok(format!("deref({})", &content[2..]))
        } else {
            // Single $ dereference
            Ok(content[1..].to_string())
        }
    }

    /// Visit subscript node - handles array/hash element access
    /// PPI::Structure::Subscript (1,730 occurrences) - Critical for array/hash access
    fn visit_subscript(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        let content = node
            .content
            .as_ref()
            .ok_or(CodeGenError::MissingContent("subscript".to_string()))?;

        // Parse subscript patterns: $val[0], $$self{Model}, etc.
        if let Some(bracket_pos) = content.find('[') {
            // Array subscript: $val[0]
            let array_name = &content[..bracket_pos];
            let index_part = &content[bracket_pos + 1..];
            let index = index_part.trim_end_matches(']');

            // Convert variable name
            let rust_array = if array_name == "$val" {
                "val"
            } else {
                array_name.trim_start_matches('$')
            };

            // Generate bounds-checked indexing
            Ok(format!(
                "{}.as_array().and_then(|arr| arr.get({})).unwrap_or(&TagValue::Empty)",
                rust_array, index
            ))
        } else if let Some(brace_pos) = content.find('{') {
            // Hash subscript: $$self{Model} (but this should be handled by cast)
            let hash_name = &content[..brace_pos];
            let key_part = &content[brace_pos + 1..];
            let key = key_part.trim_end_matches('}');

            if hash_name.starts_with("$$self") {
                Ok(format!("ctx.get(\"{}\").unwrap_or_default()", key))
            } else {
                let rust_hash = hash_name.trim_start_matches('$');
                Ok(format!(
                    "{}.as_object().and_then(|obj| obj.get(\"{}\")).unwrap_or(&TagValue::Empty)",
                    rust_hash, key
                ))
            }
        } else {
            // Fallback for complex subscript patterns
            Ok(format!("subscript_access({})", content))
        }
    }

    /// Visit regexp match node - handles pattern matching =~, !~
    /// PPI::Token::Regexp::Match (731 occurrences) - Critical for model detection
    fn visit_regexp_match(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        let content = node
            .content
            .as_ref()
            .ok_or(CodeGenError::MissingContent("regexp_match".to_string()))?;

        // Parse regex patterns: /Canon/, /EOS D30\b/, etc.
        if content.starts_with('/') && content.ends_with('/') {
            let pattern = &content[1..content.len() - 1]; // Remove / delimiters

            // Escape Rust regex special characters and convert Perl patterns
            let rust_pattern = pattern
                .replace("\\b", "\\b") // Word boundaries work the same
                .replace("\\0", "\\x00") // Null bytes
                .replace("\\xff", "\\xFF"); // Hex escapes

            // Generate regex matching code
            match self.expression_type() {
                ExpressionType::Condition => {
                    Ok(format!(
                        "regex::Regex::new(r\"{}\").unwrap().is_match(&val.to_string())",
                        rust_pattern
                    ))
                }
                _ => {
                    Ok(format!(
                        "TagValue::from(regex::Regex::new(r\"{}\").unwrap().is_match(&val.to_string()))",
                        rust_pattern
                    ))
                }
            }
        } else {
            Err(CodeGenError::UnsupportedStructure(format!(
                "Invalid regex pattern: {}",
                content
            )))
        }
    }

    // Task B: Numeric & String Operations (Phase 2) - P07: PPI Enhancement

    /// Visit hex number node - handles hexadecimal literals
    /// PPI::Token::Number::Hex (188 occurrences) - Used in binary data and flags
    fn visit_number_hex(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        let content = node
            .content
            .as_ref()
            .ok_or(CodeGenError::MissingContent("hex number".to_string()))?;

        // ExifTool hex literals: 0x123, 0xABCD
        // Convert directly to Rust hex literal
        if content.starts_with("0x") || content.starts_with("0X") {
            // Validate hex format
            let hex_part = &content[2..];
            if hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
                // Preserve the hex literal format for clarity in generated code
                Ok(content.to_lowercase())
            } else {
                Err(CodeGenError::InvalidNumber(content.clone()))
            }
        } else {
            // Shouldn't happen if PPI classified it as hex
            Err(CodeGenError::InvalidNumber(content.clone()))
        }
    }

    /// Visit variable declaration node - handles my $var = expr patterns
    /// PPI::Statement::Variable (1,524 occurrences) - Critical for multi-step processing
    fn visit_variable(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        // Parse variable declarations: my $var = expr, my @array = split()
        // Children typically: [Word(my), Symbol($var), Operator(=), Expression]

        if node.children.len() < 4 {
            return Err(CodeGenError::UnsupportedStructure(
                "Invalid variable declaration structure".to_string(),
            ));
        }

        // Find the variable name and value
        let mut var_name = None;
        let mut var_value = None;
        let mut is_array = false;

        for (i, child) in node.children.iter().enumerate() {
            if child.class == "PPI::Token::Symbol" {
                if let Some(content) = &child.content {
                    // Remove the sigil ($, @, %)
                    var_name = Some(content[1..].to_string());
                    is_array = content.starts_with('@');
                }
            } else if child.class == "PPI::Token::Operator"
                && child.content.as_deref() == Some("=")
                && i + 1 < node.children.len()
            {
                // Everything after = is the value
                let value_nodes = &node.children[i + 1..];
                var_value = Some(self.process_node_sequence(value_nodes)?);
                break;
            }
        }

        match (var_name, var_value) {
            (Some(name), Some(value)) => {
                // Generate Rust variable binding
                if is_array {
                    Ok(format!("let {} = {};", name, value))
                } else {
                    Ok(format!("let {} = {};", name, value))
                }
            }
            _ => Err(CodeGenError::UnsupportedStructure(
                "Could not parse variable declaration".to_string(),
            )),
        }
    }

    /// Visit regexp substitute node - handles s/pattern/replacement/ operations
    /// PPI::Token::Regexp::Substitute (176 occurrences) - String manipulation
    fn visit_regexp_substitute(&self, node: &PpiNode) -> Result<String, CodeGenError> {
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

        // Generate Rust string replacement code
        // For now, use simple string replacement - regex can be added later
        if pattern
            .chars()
            .all(|c| c.is_alphanumeric() || c.is_whitespace())
        {
            // Simple string replacement
            if is_global {
                Ok(format!(
                    "val.to_string().replace(\"{}\", \"{}\")",
                    pattern, replacement
                ))
            } else {
                Ok(format!(
                    "val.to_string().replacen(\"{}\", \"{}\", 1)",
                    pattern, replacement
                ))
            }
        } else {
            // Regex replacement
            if is_global {
                Ok(format!(
                    "regex::Regex::new(r\"{}\").unwrap().replace_all(&val.to_string(), \"{}\")",
                    pattern, replacement
                ))
            } else {
                Ok(format!(
                    "regex::Regex::new(r\"{}\").unwrap().replace(&val.to_string(), \"{}\")",
                    pattern, replacement
                ))
            }
        }
    }

    // Helper method for processing node sequences (needed by visit_variable)
    fn process_node_sequence(&self, children: &[PpiNode]) -> Result<String, CodeGenError> {
        if children.is_empty() {
            return Ok("".to_string());
        }

        if children.len() == 1 {
            return self.visit_node(&children[0]);
        }

        let mut parts = Vec::new();
        for child in children {
            if child.class != "PPI::Token::Whitespace" {
                parts.push(self.visit_node(child)?);
            }
        }

        Ok(parts.join(" "))
    }
}
