//! PPI AST visitor pattern implementation
//!
//! This module contains the visitor pattern implementation for traversing
//! PPI AST nodes and generating Rust code from them.

use super::errors::CodeGenError;
use crate::ppi::types::*;
use indoc::formatdoc;

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
            // Task D: Control Flow & Advanced Features (Phase 3)
            "PPI::Token::Magic" => self.visit_magic(node),
            "PPI::Statement::Break" => self.visit_break(node),
            "PPI::Token::Regexp::Transliterate" => self.visit_transliterate(node),
            "PPI::Structure::Block" => self.visit_block(node),
            // Existing supported tokens
            "PPI::Token::Symbol" => self.visit_symbol(node),
            "PPI::Token::Operator" => self.visit_operator(node),
            "PPI::Token::Number" => self.visit_number(node),
            "PPI::Token::Quote::Double" | "PPI::Token::Quote::Single" => self.visit_string(node),
            "PPI::Token::Word" => self.visit_word(node),
            "PPI::Structure::List" => self.visit_list(node),
            "PPI::Token::Structure" => self.visit_structure(node),
            // Normalized AST node types (created by normalizer)
            "FunctionCall" => self.visit_normalized_function_call(node),
            "StringConcat" => self.visit_normalized_string_concat(node),
            "StringRepeat" => self.visit_normalized_string_repeat(node),
            _ => Err(CodeGenError::UnsupportedToken(node.class.clone())),
        }
    }

    /// Visit document node (top level)
    fn visit_document(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        if node.children.is_empty() {
            return Ok("".to_string());
        }

        if node.children.len() == 1 {
            // Simple case: single statement
            return self.visit_node(&node.children[0]);
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
            // Multiple statements: check if they are assignments that need a mutable variable
            let has_assignments = results.iter().any(|s| s.contains(" = "));

            if has_assignments {
                // Create a block with mutable local variable for assignment operations
                results.push(last_result);
                let statements = results[..results.len() - 1].join(";\n    ");
                let final_result = &results[results.len() - 1];
                Ok(formatdoc! {r#"
                    {{
                        let mut val = val.clone();
                        {statements};
                        {final_result}
                    }}
                "#})
            } else {
                // Regular multiple statements: combine them in a block expression
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

    /// Visit structure token - handles structural elements like parentheses, brackets
    fn visit_structure(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        let content = node
            .content
            .as_ref()
            .ok_or(CodeGenError::MissingContent("structure".to_string()))?;

        // For basic structure tokens, just return the content
        // More complex handling would go in specific structure types
        Ok(content.clone())
    }

    // Normalized AST node visitors (created by normalizer)

    /// Visit normalized function call nodes
    fn visit_normalized_function_call(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        let func_name = node
            .content
            .as_ref()
            .ok_or(CodeGenError::MissingContent("function_call".to_string()))?;

        // Process function arguments from children
        let mut args = Vec::new();
        for child in &node.children {
            args.push(self.visit_node(child)?);
        }

        // Handle special runtime functions
        match func_name.as_str() {
            "safe_reciprocal" => {
                if args.len() != 1 {
                    return Err(CodeGenError::UnsupportedStructure(
                        "safe_reciprocal requires exactly 1 argument".to_string(),
                    ));
                }
                Ok(format!("crate::fmt::safe_reciprocal(&{})", args[0]))
            }
            "safe_division" => {
                if args.len() != 2 {
                    return Err(CodeGenError::UnsupportedStructure(
                        "safe_division requires exactly 2 arguments".to_string(),
                    ));
                }
                Ok(format!(
                    "crate::fmt::safe_division({}.0, &{})",
                    args[0], args[1]
                ))
            }
            "log" => {
                if args.len() != 1 {
                    return Err(CodeGenError::UnsupportedStructure(
                        "log requires exactly 1 argument".to_string(),
                    ));
                }
                Ok(format!("({} as f64).ln()", args[0]))
            }
            "length" => {
                if args.len() != 1 {
                    return Err(CodeGenError::UnsupportedStructure(
                        "length requires exactly 1 argument".to_string(),
                    ));
                }
                let var = &args[0];
                match self.expression_type() {
                    ExpressionType::PrintConv => Ok(format!(
                        "TagValue::String(match {} {{ TagValue::String(s) => s.len().to_string(), _ => \"0\".to_string() }})",
                        var
                    )),
                    ExpressionType::ValueConv => Ok(format!(
                        "TagValue::I32(match {} {{ TagValue::String(s) => s.len() as i32, _ => 0 }})",
                        var
                    )),
                    _ => Ok(format!("match {} {{ TagValue::String(s) => s.len() as i32, _ => 0 }}", var)),
                }
            }
            _ => {
                // Generic function call
                Ok(format!("{}({})", func_name, args.join(", ")))
            }
        }
    }

    /// Visit normalized string concatenation nodes
    fn visit_normalized_string_concat(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        let mut parts = Vec::new();
        for child in &node.children {
            parts.push(self.visit_node(child)?);
        }

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

    /// Visit normalized string repetition nodes
    fn visit_normalized_string_repeat(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        if node.children.len() != 2 {
            return Err(CodeGenError::UnsupportedStructure(
                "StringRepeat requires exactly 2 children (string, count)".to_string(),
            ));
        }

        let string_part = self.visit_node(&node.children[0])?;
        let count = self.visit_node(&node.children[1])?;

        match self.expression_type() {
            ExpressionType::PrintConv | ExpressionType::ValueConv => Ok(format!(
                "TagValue::String({}.repeat({} as usize))",
                string_part, count
            )),
            _ => Ok(format!("{}.repeat({} as usize)", string_part, count)),
        }
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
        // Subscript nodes might have content or might have children
        // Check for content first, then fall back to processing children
        if let Some(content) = node.content.as_ref() {
            // Direct content - parse it
            return self.parse_subscript_content(content);
        }

        // No content - process children to build the subscript
        if node.children.is_empty() {
            return Err(CodeGenError::MissingContent("subscript".to_string()));
        }

        // Build subscript from children
        let mut parts = Vec::new();
        for child in &node.children {
            parts.push(self.visit_node(child)?);
        }
        let reconstructed = parts.join("");
        self.parse_subscript_content(&reconstructed)
    }

    fn parse_subscript_content(&self, content: &str) -> Result<String, CodeGenError> {
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

        // Parse regex patterns: /Canon/, /EOS D30\b/, /../g etc.
        if content.starts_with('/') {
            // Handle patterns with flags like /../g
            let end_slash = content.rfind('/').unwrap_or(content.len() - 1);
            if end_slash <= 1 {
                // Empty pattern or malformed
                return Ok(format!("/{}/", content.trim_matches('/')));
            }

            let pattern = &content[1..end_slash]; // Extract pattern between slashes
            let flags = &content[end_slash + 1..]; // Extract flags after last slash

            // Handle special case of /../ which means "any two characters" in Perl
            if pattern == ".." {
                // Return a pattern that matches any two characters
                return Ok("/./".to_string()); // Simplified - actual regex would be more complex
            }

            // Escape Rust regex special characters and convert Perl patterns
            let rust_pattern = pattern
                .replace("\\b", "\\b") // Word boundaries work the same
                .replace("\\0", "\\x00") // Null bytes
                .replace("\\xff", "\\xFF"); // Hex escapes

            // When this is just a regex pattern (not part of =~ or !~),
            // we just return the pattern itself for later combination
            // The actual matching will be handled when combined with =~ or !~
            if flags.is_empty() {
                Ok(format!("/{}/", rust_pattern))
            } else {
                Ok(format!("/{}/{}", rust_pattern, flags))
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
            // Following the pattern from magic_numbers.rs strategy
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

    // Task D: Control Flow & Advanced Features (Phase 3) - P07: PPI Enhancement

    /// Visit magic variable node - handles special variables like $_ and $@
    /// PPI::Token::Magic (174 occurrences) - Used in string manipulation patterns
    fn visit_magic(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        let content = node
            .content
            .as_ref()
            .ok_or(CodeGenError::MissingContent("magic variable".to_string()))?;

        match content.as_str() {
            "$_" => {
                // $_ is the default variable - in our context it's the current value being processed
                // Example: $_=$val,s/(\d+)(\d{4})/$1-$2/,$_
                // In ExifTool expressions, $_ typically refers to val
                Ok("val".to_string())
            }
            "$@" => {
                // $@ is the error variable in Perl
                Ok("error_val".to_string())
            }
            "$!" => {
                // $! is the system error
                Ok("sys_error".to_string())
            }
            "$?" => {
                // $? is the exit status
                Ok("exit_status".to_string())
            }
            _ => {
                // Other magic variables - generate a placeholder
                Ok(format!("magic_var_{}", content.trim_start_matches('$')))
            }
        }
    }

    /// Visit break statement node - handles return, last, next control flow
    /// PPI::Statement::Break (145 occurrences) - Critical for early returns
    fn visit_break(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        // Break statements typically have structure: [Word(return/last/next), Expression]
        if node.children.is_empty() {
            return Err(CodeGenError::UnsupportedStructure(
                "Empty break statement".to_string(),
            ));
        }

        let keyword = if node.children[0].class == "PPI::Token::Word" {
            node.children[0]
                .content
                .as_ref()
                .ok_or(CodeGenError::MissingContent("break keyword".to_string()))?
        } else {
            return Err(CodeGenError::UnsupportedStructure(
                "Invalid break statement structure".to_string(),
            ));
        };

        // Process the value/expression after the keyword
        let value = if node.children.len() > 1 {
            // Skip whitespace and process the rest
            let mut expr_parts = Vec::new();
            for i in 1..node.children.len() {
                if node.children[i].class != "PPI::Token::Whitespace" {
                    expr_parts.push(self.visit_node(&node.children[i])?);
                }
            }
            if expr_parts.is_empty() {
                "".to_string()
            } else {
                expr_parts.join(" ")
            }
        } else {
            "".to_string()
        };

        // Generate appropriate Rust control flow
        match keyword.as_str() {
            "return" => {
                // return $val => return val
                if value.is_empty() {
                    Ok("return".to_string())
                } else {
                    // Wrap in appropriate type based on expression type
                    match self.expression_type() {
                        ExpressionType::ValueConv => Ok(format!("return Ok({})", value)),
                        ExpressionType::PrintConv => Ok(format!("return {}", value)),
                        ExpressionType::Condition => Ok(format!("return {}", value)),
                    }
                }
            }
            "last" => {
                // Perl's "last" is Rust's "break"
                Ok("break".to_string())
            }
            "next" => {
                // Perl's "next" is Rust's "continue"
                Ok("continue".to_string())
            }
            _ => Err(CodeGenError::UnsupportedStructure(format!(
                "Unknown break keyword: {}",
                keyword
            ))),
        }
    }

    /// Visit transliterate node - handles tr/// character replacement operations
    /// PPI::Token::Regexp::Transliterate (likely <100 occurrences) - String character mapping
    fn visit_transliterate(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        let content = node.content.as_ref().ok_or(CodeGenError::MissingContent(
            "transliterate pattern".to_string(),
        ))?;

        // Parse tr/pattern/replacement/flags or tr#pattern#replacement#flags
        if !content.starts_with("tr/") && !content.starts_with("tr#") {
            return Err(CodeGenError::UnsupportedStructure(format!(
                "Invalid transliterate pattern: {}",
                content
            )));
        }

        // Determine delimiter
        let delimiter = if content.starts_with("tr/") { '/' } else { '#' };
        let parts: Vec<&str> = content[3..].split(delimiter).collect();

        if parts.len() < 2 {
            return Err(CodeGenError::UnsupportedStructure(format!(
                "Invalid transliterate format: {}",
                content
            )));
        }

        let search_chars = parts[0];
        let replace_chars = if parts.len() > 1 { parts[1] } else { "" };
        let flags = if parts.len() > 2 { parts[2] } else { "" };

        // Check for delete flag (d) and complement flag (c)
        let is_delete = flags.contains('d');
        let is_complement = flags.contains('c');

        if is_delete && !is_complement {
            // tr/chars//d - delete specified characters
            // Example: tr/()K//d removes parentheses and K
            let chars_to_remove: Vec<String> =
                search_chars.chars().map(|c| format!("'{}'", c)).collect();
            Ok(format!(
                "val.to_string().chars().filter(|c| ![{}].contains(c)).collect::<String>()",
                chars_to_remove.join(", ")
            ))
        } else if is_delete && is_complement {
            // tr/chars//dc - delete all EXCEPT specified characters
            // Example: tr/a-fA-F0-9//dc keeps only hex digits
            if search_chars.contains('-') {
                // Handle character ranges like a-f, A-F, 0-9
                let mut keep_chars = Vec::new();
                let chars: Vec<char> = search_chars.chars().collect();
                let mut i = 0;
                while i < chars.len() {
                    if i + 2 < chars.len() && chars[i + 1] == '-' {
                        // Character range
                        let start = chars[i] as u8;
                        let end = chars[i + 2] as u8;
                        for c in start..=end {
                            keep_chars.push(c as char);
                        }
                        i += 3;
                    } else if chars[i] != '-' {
                        // Single character
                        keep_chars.push(chars[i]);
                        i += 1;
                    } else {
                        i += 1;
                    }
                }
                let keep_list: Vec<String> =
                    keep_chars.iter().map(|c| format!("'{}'", c)).collect();
                Ok(format!(
                    "val.to_string().chars().filter(|c| [{}].contains(c)).collect::<String>()",
                    keep_list.join(", ")
                ))
            } else {
                // Simple character list
                let keep_chars: Vec<String> =
                    search_chars.chars().map(|c| format!("'{}'", c)).collect();
                Ok(format!(
                    "val.to_string().chars().filter(|c| [{}].contains(c)).collect::<String>()",
                    keep_chars.join(", ")
                ))
            }
        } else {
            // Character-by-character replacement
            // Build a replacement map
            let search_vec: Vec<char> = search_chars.chars().collect();
            let replace_vec: Vec<char> = replace_chars.chars().collect();

            if search_vec.len() != replace_vec.len() {
                return Err(CodeGenError::UnsupportedStructure(format!(
                    "Transliterate pattern length mismatch: {} vs {}",
                    search_chars, replace_chars
                )));
            }

            // Generate character mapping code
            let mut mappings = Vec::new();
            for (s, r) in search_vec.iter().zip(replace_vec.iter()) {
                mappings.push(format!("'{}' => '{}'", s, r));
            }

            Ok(format!(
                "val.to_string().chars().map(|c| match c {{ {} , _ => c }}).collect::<String>()",
                mappings.join(", ")
            ))
        }
    }

    /// Visit block node - handles closures and anonymous blocks
    /// PPI::Structure::Block (103 occurrences) - Used in map/grep operations
    fn visit_block(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        // Blocks contain statements that form closures
        // Example: map { $_ * 2 } @array

        if node.children.is_empty() {
            // Empty block
            return Ok("{ }".to_string());
        }

        // Process the block contents
        let mut block_parts = Vec::new();
        for child in &node.children {
            if child.class != "PPI::Token::Whitespace" {
                block_parts.push(self.visit_node(child)?);
            }
        }

        // Generate closure-like code
        // For now, generate a simple block - can be enhanced based on context
        if block_parts.len() == 1 {
            // Single expression block
            Ok(format!("|item| {}", block_parts[0]))
        } else {
            // Multi-statement block
            Ok(format!("|item| {{ {} }}", block_parts.join("; ")))
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
}
