//! String concatenation and regex operations
//!
//! This module handles string operations including concatenation, repetition,
//! and regex pattern matching.

use crate::ppi::rust_generator::errors::CodeGenError;
use crate::ppi::types::*;

/// Trait for handling string operations
pub trait StringOperationsHandler {
    fn expression_type(&self) -> &ExpressionType;

    /// Handle normalized StringConcat nodes
    fn handle_normalized_string_concat(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        // Process each child node properly - they may be complex expressions
        let mut parts = Vec::new();

        for child in &node.children {
            let part = if let Some(ref content) = child.content {
                // Simple content - check if it's a function like sprintf
                if content == "sprintf" && child.children.len() > 0 {
                    // This is a sprintf function call with arguments
                    self.handle_sprintf_function_call(child)?
                } else {
                    content.clone()
                }
            } else if let Some(ref string_value) = child.string_value {
                // String literal
                format!("\"{}\"", string_value)
            } else {
                // Check if this is a function call with structure
                if child.class == "PPI::Statement" && child.children.len() >= 2 {
                    let first_child = &child.children[0];
                    if first_child.content.as_deref() == Some("sprintf")
                        && child.children.len() >= 2
                    {
                        // This is a sprintf call - handle it directly using visitor pattern
                        self.handle_sprintf_statement_directly(child)?
                    } else {
                        // Other PPI::Statement - process recursively
                        self.combine_statement_parts(&[], &[child.clone()])?
                    }
                } else if child.class == "PPI::Structure::List" && child.children.len() == 1 {
                    // This is likely a parenthesized expression - check if it's a ternary
                    let expr = &child.children[0];
                    if expr.class == "PPI::Statement::Expression"
                        && self.contains_ternary_operators(&expr.children)
                    {
                        // This is a ternary expression in parentheses
                        self.handle_parenthesized_ternary(expr)?
                    } else {
                        // Complex expression - recursively process it
                        self.combine_statement_parts(&[], &[child.clone()])?
                    }
                } else {
                    // Complex expression - recursively process it
                    self.combine_statement_parts(&[], &[child.clone()])?
                }
            };
            parts.push(part);
        }

        // Generate format! call with all parts
        match self.expression_type() {
            ExpressionType::PrintConv | ExpressionType::ValueConv => Ok(format!(
                "TagValue::String(format!(\"{}\", {}))",
                "{}".repeat(parts.len()),
                parts.join(", ")
            )),
            _ => Ok(format!(
                "format!(\"{}\", {})",
                "{}".repeat(parts.len()),
                parts.join(", ")
            )),
        }
    }

    /// Handle normalized StringRepeat nodes  
    fn handle_normalized_string_repeat(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        if node.children.len() != 2 {
            return Err(CodeGenError::UnsupportedStructure(
                "StringRepeat needs exactly 2 children".to_string(),
            ));
        }

        let string_part = self.process_function_args(&[node.children[0].clone()])?[0].clone();
        let count = self.process_function_args(&[node.children[1].clone()])?[0].clone();

        match self.expression_type() {
            ExpressionType::PrintConv | ExpressionType::ValueConv => Ok(format!(
                "TagValue::String({}.repeat({} as usize))",
                string_part, count
            )),
            _ => Ok(format!("{}.repeat({} as usize)", string_part, count)),
        }
    }

    /// Try to handle string concatenation pattern: expr . expr
    fn try_string_concat_pattern(&self, parts: &[String]) -> Result<Option<String>, CodeGenError> {
        if let Some(dot_pos) = parts.iter().position(|p| p == ".") {
            if dot_pos > 0 && dot_pos < parts.len() - 1 {
                let left_parts = &parts[..dot_pos];
                let right_parts = &parts[dot_pos + 1..];

                // Join the parts back into expressions
                let left_expr = left_parts.join(" ");
                let right_expr = right_parts.join(" ");

                // Generate string concatenation using format!
                let result = match self.expression_type() {
                    ExpressionType::PrintConv | ExpressionType::ValueConv => format!(
                        "TagValue::String(format!(\"{{}}{{}}\", {}, {}))",
                        left_expr, right_expr
                    ),
                    _ => format!("format!(\"{{}}{{}}\", {}, {})", left_expr, right_expr),
                };
                return Ok(Some(result));
            }
        }
        Ok(None)
    }

    /// Handle regex operations in binary operations
    /// Supports both matching (/pattern/) and substitution (s/pattern/replacement/flags)
    fn handle_regex_operation(
        &self,
        left: &str,
        op: &str,
        right: &str,
    ) -> Result<String, CodeGenError> {
        // Check if the right side is already processed Rust code from visit_regexp_substitute or visit_transliterate
        if right.starts_with("TagValue::String(")
            || right.starts_with("val.to_string().chars().map(")
        {
            // This is already processed transformation code - just return it directly
            // For =~ operations with transformations, the transformation result IS the result
            return Ok(right.to_string());
        }

        // Check if this is a substitution operation (s/pattern/replacement/flags)
        if right.starts_with("s/") || right.starts_with("s#") {
            return self.handle_regex_substitution(left, op, right);
        }

        // Extract the regex pattern from right side (e.g., "/\\d/" -> "\\d")
        let pattern = if right.starts_with('/') && right.ends_with('/') {
            &right[1..right.len() - 1]
        } else {
            right
        };

        // Check for capture groups in the pattern
        if pattern.contains('(') && pattern.contains(')') {
            // This is a capture group pattern - generate proper regex match code
            // Following Trust ExifTool: preserve exact semantics of regex capture matching
            return self.generate_regex_capture_code(left, op, pattern);
        }

        // For simple pattern matching, use contains or regex
        // \d means contains a digit
        if pattern == "\\d" {
            if op == "=~" {
                return Ok(format!(
                    "{}.to_string().chars().any(|c| c.is_ascii_digit())",
                    left
                ));
            } else {
                return Ok(format!(
                    "!{}.to_string().chars().any(|c| c.is_ascii_digit())",
                    left
                ));
            }
        }

        // For other patterns, use a simple contains check
        // This is a simplification - full regex support would need the regex crate
        if op == "=~" {
            return Ok(format!("{}.to_string().contains(r\"{}\")", left, pattern));
        } else {
            return Ok(format!("!{}.to_string().contains(r\"{}\")", left, pattern));
        }
    }

    /// Generate regex capture code for patterns with capture groups
    /// Handles patterns like (\d{2})(\d{2})(\d{2}) by generating proper Rust regex matching
    fn generate_regex_capture_code(
        &self,
        left: &str,
        op: &str,
        pattern: &str,
    ) -> Result<String, CodeGenError> {
        // Count capture groups by counting opening parentheses (not escaped)
        // Generate regex static variable name based on pattern hash
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        pattern.hash(&mut hasher);
        let regex_id = format!("REGEX_{:x}", hasher.finish());

        // Convert Perl regex pattern to Rust regex pattern
        // Trust ExifTool: preserve exact pattern semantics but translate syntax
        let rust_pattern = pattern
            .replace("\\d", "\\d") // \d works the same in Rust
            .replace("\\w", "\\w") // \w works the same in Rust
            .replace("\\s", "\\s"); // \s works the same in Rust

        // Generate the regex match code
        if op == "=~" {
            // Positive match - return true if regex matches, false otherwise
            // Note: This generates a boolean result for truthiness testing
            // The actual capture extraction happens in the ternary context
            Ok(format!(
                "{{ use regex::Regex; use std::sync::LazyLock; static {}: LazyLock<Regex> = LazyLock::new(|| Regex::new(r\"{}\").unwrap()); {}.captures(&{}.to_string()).is_some() }}",
                regex_id, rust_pattern, regex_id, left
            ))
        } else {
            // Negative match (!~)
            Ok(format!(
                "{{ use regex::Regex; use std::sync::LazyLock; static {}: LazyLock<Regex> = LazyLock::new(|| Regex::new(r\"{}\").unwrap()); {}.captures(&{}.to_string()).is_none() }}",
                regex_id, rust_pattern, regex_id, left
            ))
        }
    }

    /// Handle regex substitution operations (s/pattern/replacement/flags)
    /// Generates appropriate string replacement code based on pattern complexity
    fn handle_regex_substitution(
        &self,
        left: &str,
        _op: &str,
        substitution: &str,
    ) -> Result<String, CodeGenError> {
        // Parse s/pattern/replacement/flags
        if !substitution.starts_with("s/") && !substitution.starts_with("s#") {
            return Err(CodeGenError::UnsupportedStructure(format!(
                "Invalid substitution pattern: {}",
                substitution
            )));
        }

        // Determine delimiter
        let delimiter = if substitution.starts_with("s/") {
            '/'
        } else {
            '#'
        };
        let parts: Vec<&str> = substitution[2..].split(delimiter).collect();

        if parts.len() < 2 {
            return Err(CodeGenError::UnsupportedStructure(format!(
                "Invalid substitution format: {}",
                substitution
            )));
        }

        let pattern = parts[0];
        let replacement = if parts.len() > 1 { parts[1] } else { "" };
        let flags = if parts.len() > 2 { parts[2] } else { "" };

        // Check for global flag
        let is_global = flags.contains('g');

        // Generate substitution code based on expression type
        let substitution_result = if pattern
            .chars()
            .all(|c| c.is_alphanumeric() || c.is_whitespace())
        {
            // Simple string replacement
            if is_global {
                format!(
                    "{}.to_string().replace(\"{}\", \"{}\")",
                    left, pattern, replacement
                )
            } else {
                format!(
                    "{}.to_string().replacen(\"{}\", \"{}\", 1)",
                    left, pattern, replacement
                )
            }
        } else {
            // Complex pattern - use regex replacement functions
            let safe_pattern = self.make_pattern_safe_for_rust(pattern);
            let escaped_replacement = self.escape_replacement_string(replacement);

            if is_global {
                format!(
                    "crate::fmt::regex_replace_all(\"{}\", &{}.to_string(), \"{}\")",
                    safe_pattern, left, escaped_replacement
                )
            } else {
                format!(
                    "crate::fmt::regex_replace(\"{}\", &{}.to_string(), \"{}\")",
                    safe_pattern, left, escaped_replacement
                )
            }
        };

        // Note: For =~ operations with substitutions, this should typically be used in an assignment context
        // The caller should handle whether to wrap in TagValue or not
        match self.expression_type() {
            ExpressionType::PrintConv | ExpressionType::ValueConv => {
                Ok(format!("TagValue::String({})", substitution_result))
            }
            _ => Ok(substitution_result),
        }
    }

    /// Make regex patterns safe for Rust code generation
    /// Handles patterns that might contain non-UTF8 bytes like ExifTool patterns
    fn make_pattern_safe_for_rust(&self, pattern: &str) -> String {
        // Escape backslashes and quotes for string literals
        // This delegates to crate::fmt functions that handle bytes regex properly
        pattern.replace("\\", "\\\\").replace("\"", "\\\"")
    }

    /// Escape replacement strings for proper Rust string literals
    fn escape_replacement_string(&self, replacement: &str) -> String {
        // Escape special characters in replacement strings for Rust string literals
        // Note: $ signs should remain as literal $ for regex backreferences like $1, $2, etc.
        replacement.replace("\\", "\\\\").replace("\"", "\\\"")
        // Do NOT escape $ signs - they are needed for regex backreferences ($1, $2, etc.)
    }

    /// Handle sprintf function call within a concatenation context
    fn handle_sprintf_in_concat(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        // Extract sprintf arguments from the PPI::Structure::List
        if node.children.len() >= 2 && node.children[1].class == "PPI::Structure::List" {
            let list_node = &node.children[1];
            if let Some(expr_node) = list_node.children.first() {
                if expr_node.class == "PPI::Statement::Expression" {
                    // Extract format string and arguments
                    let mut format_str = String::new();
                    let mut args = Vec::new();

                    for child in &expr_node.children {
                        if let Some(ref string_value) = child.string_value {
                            if format_str.is_empty() {
                                format_str = string_value.clone();
                            } else {
                                args.push(format!("\"{}\"", string_value));
                            }
                        } else if let Some(ref content) = child.content {
                            if content != "," && !format_str.is_empty() {
                                args.push(content.clone());
                            }
                        }
                    }

                    // Generate sprintf call
                    let args_str = if args.is_empty() {
                        "val".to_string()
                    } else {
                        args.join(", ")
                    };

                    return Ok(format!("format!(\"{}\", {})", format_str, args_str));
                }
            }
        }

        Err(CodeGenError::UnsupportedStructure(
            "Could not parse sprintf in concat context".to_string(),
        ))
    }

    /// Handle sprintf statement directly by processing arguments without recursion
    fn handle_sprintf_statement_directly(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        // node is a PPI::Statement with children: [sprintf, PPI::Structure::List]
        if node.children.len() < 2 {
            return Err(CodeGenError::UnsupportedStructure(
                "sprintf statement needs arguments".to_string(),
            ));
        }

        // Extract the arguments from the Structure::List
        let args_node = &node.children[1];
        if args_node.class != "PPI::Structure::List" {
            return Err(CodeGenError::UnsupportedStructure(
                "sprintf arguments must be in parentheses".to_string(),
            ));
        }

        // Get the expression inside the parentheses
        if let Some(expr) = args_node.children.first() {
            if expr.class == "PPI::Statement::Expression" {
                // Extract format string and arguments directly from the AST
                let mut format_str = None;
                let mut args = Vec::new();
                let mut found_comma = false;

                for child in &expr.children {
                    if child.class == "PPI::Token::Operator"
                        && child.content.as_deref() == Some(",")
                    {
                        found_comma = true;
                        continue;
                    }

                    if !found_comma {
                        // This is the format string
                        if let Some(ref string_value) = child.string_value {
                            format_str = Some(string_value.clone());
                        }
                    } else {
                        // These are the arguments
                        if let Some(ref content) = child.content {
                            args.push(content.clone());
                        } else if let Some(ref string_value) = child.string_value {
                            args.push(format!("\"{}\"", string_value));
                        }
                    }
                }

                let format_str = format_str.unwrap_or_else(|| "".to_string());
                let args_str = if args.is_empty() {
                    "val".to_string()
                } else {
                    args.join(", ").replace("$val", "val")
                };

                // Convert Perl format to Rust format
                let rust_format = self.convert_perl_sprintf_format(&format_str);

                return Ok(format!("format!(\"{}\", {})", rust_format, args_str));
            }
        }

        Err(CodeGenError::UnsupportedStructure(
            "Could not parse sprintf arguments".to_string(),
        ))
    }

    /// Handle sprintf function call with explicit arguments
    fn handle_sprintf_function_call(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        if node.children.is_empty() {
            return Err(CodeGenError::UnsupportedStructure(
                "sprintf needs arguments".to_string(),
            ));
        }

        // Process arguments
        let args = self.process_function_args(&node.children)?;
        if args.is_empty() {
            return Err(CodeGenError::UnsupportedStructure(
                "sprintf needs format string".to_string(),
            ));
        }

        let format_str = &args[0];
        let format_args = if args.len() > 1 {
            args[1..].join(", ")
        } else {
            "val".to_string()
        };

        Ok(format!("format!({}, {})", format_str, format_args))
    }

    /// Check if a list of children contains ternary operators (? and :)
    fn contains_ternary_operators(&self, children: &[PpiNode]) -> bool {
        children.iter().any(|child| {
            child.content.as_deref() == Some("?") || child.content.as_deref() == Some(":")
        })
    }

    /// Handle a parenthesized ternary expression
    fn handle_parenthesized_ternary(&self, expr: &PpiNode) -> Result<String, CodeGenError> {
        // Find the question mark and colon positions
        let mut question_pos = None;
        let mut colon_pos = None;

        for (i, child) in expr.children.iter().enumerate() {
            if child.content.as_deref() == Some("?") && question_pos.is_none() {
                question_pos = Some(i);
            } else if child.content.as_deref() == Some(":") && colon_pos.is_none() {
                colon_pos = Some(i);
            }
        }

        if let (Some(q_pos), Some(c_pos)) = (question_pos, colon_pos) {
            if q_pos < c_pos {
                // Extract condition, true branch, and false branch
                let condition_parts: Vec<String> = expr.children[..q_pos]
                    .iter()
                    .map(|child| self.extract_node_content(child))
                    .collect::<Result<Vec<_>, _>>()?;

                let true_parts: Vec<String> = expr.children[q_pos + 1..c_pos]
                    .iter()
                    .map(|child| self.extract_node_content(child))
                    .collect::<Result<Vec<_>, _>>()?;

                let false_parts: Vec<String> = expr.children[c_pos + 1..]
                    .iter()
                    .map(|child| self.extract_node_content(child))
                    .collect::<Result<Vec<_>, _>>()?;

                let condition = condition_parts.join(" ");
                let true_branch = true_parts.join(" ");
                let false_branch = false_parts.join(" ");

                return Ok(format!(
                    "if {} {{ {} }} else {{ {} }}",
                    condition, true_branch, false_branch
                ));
            }
        }

        // Fallback: treat as regular expression
        self.combine_statement_parts(&[], &[expr.clone()])
    }

    /// Extract content from a node (handling strings, numbers, symbols, etc.)
    fn extract_node_content(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        if let Some(ref content) = node.content {
            Ok(content.clone())
        } else if let Some(ref string_value) = node.string_value {
            Ok(format!("\"{}\"", string_value))
        } else if let Some(num) = node.numeric_value {
            Ok(num.to_string())
        } else {
            // For complex nodes, recursively process
            self.combine_statement_parts(&[], &[node.clone()])
        }
    }

    /// Check if operator is a string operation
    fn is_string_operator(&self, op: &str) -> bool {
        matches!(op, "." | "=~" | "!~")
    }

    /// Legacy method compatibility - delegate to main combiner
    fn combine_statement_parts(
        &self,
        parts: &[String],
        children: &[PpiNode],
    ) -> Result<String, CodeGenError>;

    /// Process function arguments from child nodes
    fn process_function_args(&self, children: &[PpiNode]) -> Result<Vec<String>, CodeGenError>;

    /// Convert Perl sprintf format string to Rust format! compatible format
    fn convert_perl_sprintf_format(&self, perl_format: &str) -> String {
        let mut rust_format = perl_format.to_string();

        // Float precision: %.3f -> {:.3}, %.2f -> {:.2}, %.0f -> {:.0}
        for precision in (0..=10).rev() {
            let perl_pattern = format!("%.{}f", precision);
            let rust_pattern = format!("{{:.{}}}", precision);
            rust_format = rust_format.replace(&perl_pattern, &rust_pattern);
        }

        // Integer padding: %.3d -> {:03}, %.5d -> {:05}
        for width in (1..=10).rev() {
            let perl_pattern = format!("%.{}d", width);
            let rust_pattern = format!("{{:0{}}}", width);
            rust_format = rust_format.replace(&perl_pattern, &rust_pattern);
        }

        // Hex with padding: %.8x -> {:08x}, %.4X -> {:04X}
        for width in (1..=10).rev() {
            let perl_pattern_lower = format!("%.{}x", width);
            let rust_pattern_lower = format!("{{:0{}x}}", width);
            rust_format = rust_format.replace(&perl_pattern_lower, &rust_pattern_lower);

            let perl_pattern_upper = format!("%.{}X", width);
            let rust_pattern_upper = format!("{{:0{}X}}", width);
            rust_format = rust_format.replace(&perl_pattern_upper, &rust_pattern_upper);
        }

        // Handle generic formats (no precision/padding)
        rust_format = rust_format.replace("%d", "{}");
        rust_format = rust_format.replace("%s", "{}");
        rust_format = rust_format.replace("%f", "{}");
        rust_format = rust_format.replace("%x", "{:x}");
        rust_format = rust_format.replace("%X", "{:X}");
        rust_format = rust_format.replace("%o", "{:o}");

        // Handle escaped percent: %% -> %
        rust_format = rust_format.replace("%%", "%");

        rust_format
    }
}
