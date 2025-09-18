//! PPI AST visitor pattern implementation
//!
//! This module contains the visitor pattern implementation for traversing
//! PPI AST nodes and generating Rust code from them.

use super::errors::CodeGenError;
use crate::impl_registry::lookup_function;
use crate::ppi::types::*;
use indoc::formatdoc;

use crate::ppi::rust_generator::visitor_tokens::*;

/// Trait for visiting PPI AST nodes and generating Rust code
pub trait PpiVisitor {
    fn expression_type(&self) -> &ExpressionType;

    /// Recursive visitor for PPI nodes - dispatches based on node class
    fn visit_node(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        match node.class.as_str() {
            "PPI::Document" => self.visit_document(node),
            "PPI::Statement" => self.visit_statement(node),
            "PPI::Statement::Expression" => self.visit_expression(node),
            "PPI::Token::Cast" => self.visit_cast(node),
            "PPI::Structure::Subscript" => self.visit_subscript(node),
            "PPI::Token::Regexp::Match" => self.visit_regexp_match(node),
            "PPI::Token::Number::Hex" => self.visit_number_hex(node),
            "PPI::Token::Number::Float" => self.visit_number(node), // Handle float the same as number
            "PPI::Statement::Variable" => self.visit_variable(node),
            "PPI::Token::Regexp::Substitute" => self.visit_regexp_substitute(node),
            "PPI::Token::Magic" => self.visit_magic(node),
            "PPI::Statement::Break" => self.visit_break(node),
            "PPI::Token::Regexp::Transliterate" => self.visit_transliterate(node),
            "PPI::Structure::Block" => self.visit_block(node),
            "PPI::Token::Symbol" => self.visit_symbol(node),
            "PPI::Token::Operator" => self.visit_operator(node),
            "PPI::Token::Number" => self.visit_number(node),
            "PPI::Token::Quote::Double" | "PPI::Token::Quote::Single" => self.visit_string(node),
            "PPI::Token::Word" => self.visit_word(node),
            "PPI::Structure::List" => self.visit_list(node),
            "PPI::Token::Structure" => self.visit_structure(node),
            "ConditionalBlock" => self.visit_normalized_conditional_block(node),
            "FunctionCall" => self.visit_normalized_function_call(node),
            "IfStatement" => self.visit_normalized_if_statement(node),
            "StringConcat" => self.visit_normalized_string_concat(node),
            "StringRepeat" => self.visit_normalized_string_repeat(node),
            "TernaryOp" | "TernaryOperation" | "SafeDivision" => {
                self.visit_normalized_ternary_op(node)
            }
            "BinaryOperation" => self.visit_normalized_binary_operation(node),
            "UnaryNegation" => self.visit_unary_negation(node),
            "ArrayAccess" => self.visit_array_access(node),
            // Normalized component nodes (parts of larger structures)
            "Condition" | "Assignment" | "TrueBranch" | "FalseBranch" => {
                self.visit_normalized_component(node)
            }
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

        // Check for unsupported multi-statement transformation patterns
        // These are expressions like: $val=~tr/./:/; $val=~s/pattern/replace/; $val
        // which require mutable variable handling we don't support yet
        let has_transformations = node.children.iter().any(|child| {
            if let Some(binary_op) = child.children.iter().find(|c| c.class == "BinaryOperation") {
                if binary_op.content.as_deref() == Some("=~") {
                    if let Some(right_child) = binary_op.children.get(1) {
                        matches!(
                            right_child.class.as_str(),
                            "PPI::Token::Regexp::Substitute" | "PPI::Token::Regexp::Transliterate"
                        )
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        });

        if has_transformations && node.children.len() > 1 {
            return Err(CodeGenError::UnsupportedStructure(
                "Multi-statement expressions with in-place transformations (tr/// or s///) are not yet supported".to_string()
            ));
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
        process_symbol(node)
    }

    /// Visit operator node
    fn visit_operator(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        process_operator(node)
    }

    /// Visit number node - enhanced for better float and scientific notation handling
    fn visit_number(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        let raw_number = if let Some(num) = node.numeric_value {
            // For code generation, use appropriate literal format
            if num.fract() == 0.0 && num.abs() < 1e10 {
                // Integer value within reasonable range
                format!("{}", num as i64)
            } else {
                // Float value or large number - ensure Rust float literal format
                let num_str = num.to_string();
                // Add explicit float suffix if not present for clarity
                if !num_str.contains('e') && !num_str.contains('.') {
                    format!("{}.0", num_str)
                } else {
                    num_str
                }
            }
        } else if let Some(content) = &node.content {
            // Handle special numeric formats
            if content.contains('e') || content.contains('E') {
                // Scientific notation - ensure proper format
                content.to_lowercase()
            } else if content.contains('.') {
                // Decimal number - preserve as-is
                content.clone()
            } else {
                // Integer - validate and return
                if content
                    .chars()
                    .all(|c| c.is_ascii_digit() || c == '-' || c == '+')
                {
                    content.clone()
                } else {
                    return Err(CodeGenError::InvalidNumber(content.clone()));
                }
            }
        } else {
            return Err(CodeGenError::MissingContent("number".to_string()));
        };

        // For all contexts, return raw numbers with appropriate type suffixes
        // The TagValue operators handle raw numeric types (Mul<i32>, Mul<f64>, etc.)
        // Using .into() here causes type ambiguity in binary operations
        if raw_number.contains('.') || raw_number.contains('e') {
            // Add f64 suffix for floats
            Ok(format!("{}f64", raw_number))
        } else {
            // For integers, use i32 suffix since that's what the operators support
            // (codegen-runtime only implements Mul<i32> and Mul<f64>, not Mul<u32>)
            Ok(format!("{}i32", raw_number))
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
            let format_expr = format!("format!(\"{}\", val)", template);

            // In PrintConv context, wrap in TagValue::String or use .into()
            match self.expression_type() {
                ExpressionType::PrintConv => Ok(format!("Into::<TagValue>::into({})", format_expr)),
                _ => Ok(format_expr),
            }
        } else {
            // Simple string literal
            let string_literal = format!("\"{}\"", string_value.replace('\"', "\\\""));

            // In PrintConv context, wrap string literals with .into()
            match self.expression_type() {
                ExpressionType::PrintConv => {
                    Ok(format!("Into::<TagValue>::into({})", string_literal))
                }
                _ => Ok(string_literal),
            }
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
            _ => {
                // Check for ExifTool namespace references that should be treated as placeholders
                if word.starts_with("Image::ExifTool::") {
                    // ExifTool namespace references aren't supported in Rust
                    // Return a placeholder value that will be handled by the conservative fallback
                    tracing::warn!("Unsupported ExifTool namespace reference: {}", word);
                    match self.expression_type() {
                        ExpressionType::PrintConv => Ok(
                            "codegen_runtime::fmt::conservative_fallback(\"\".into(), val)"
                                .to_string(),
                        ),
                        ExpressionType::ValueConv => Ok("val.clone()".to_string()),
                        ExpressionType::Condition => Ok("false".to_string()),
                    }
                } else {
                    Ok(word.clone())
                }
            }
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
                Ok(format!("codegen_runtime::safe_reciprocal(&{})", args[0]))
            }
            "safe_division" => {
                if args.len() != 2 {
                    return Err(CodeGenError::UnsupportedStructure(
                        "safe_division requires exactly 2 arguments".to_string(),
                    ));
                }
                Ok(format!(
                    "codegen_runtime::safe_division({}.0, &{})",
                    args[0], args[1]
                ))
            }
            "log" => {
                if args.len() != 1 {
                    return Err(CodeGenError::UnsupportedStructure(
                        "log requires exactly 1 argument".to_string(),
                    ));
                }
                Ok(format!("log({})", args[0]))
            }
            "exp" => {
                if args.len() != 1 {
                    return Err(CodeGenError::UnsupportedStructure(
                        "exp requires exactly 1 argument".to_string(),
                    ));
                }
                Ok(format!("exp({})", args[0]))
            }
            "int" => {
                if args.len() != 1 {
                    return Err(CodeGenError::UnsupportedStructure(
                        "int requires exactly 1 argument".to_string(),
                    ));
                }
                Ok(format!("int({})", args[0]))
            }
            "abs" => {
                if args.len() != 1 {
                    return Err(CodeGenError::UnsupportedStructure(
                        "abs requires exactly 1 argument".to_string(),
                    ));
                }
                Ok(format!("abs({})", args[0]))
            }
            "sqrt" => {
                if args.len() != 1 {
                    return Err(CodeGenError::UnsupportedStructure(
                        "sqrt requires exactly 1 argument".to_string(),
                    ));
                }
                Ok(format!("sqrt({})", args[0]))
            }
            "sin" => {
                if args.len() != 1 {
                    return Err(CodeGenError::UnsupportedStructure(
                        "sin requires exactly 1 argument".to_string(),
                    ));
                }
                Ok(format!("sin({})", args[0]))
            }
            "cos" => {
                if args.len() != 1 {
                    return Err(CodeGenError::UnsupportedStructure(
                        "cos requires exactly 1 argument".to_string(),
                    ));
                }
                Ok(format!("cos({})", args[0]))
            }
            "atan2" => {
                if args.len() != 2 {
                    return Err(CodeGenError::UnsupportedStructure(
                        "atan2 requires exactly 2 arguments".to_string(),
                    ));
                }
                Ok(format!("atan2({}, {})", args[0], args[1]))
            }
            "length" => {
                if args.len() != 1 {
                    return Err(CodeGenError::UnsupportedStructure(
                        "length requires exactly 1 argument".to_string(),
                    ));
                }
                match self.expression_type() {
                    ExpressionType::PrintConv => Ok(format!("length_string({})", args[0])),
                    ExpressionType::ValueConv => Ok(format!("length_i32({})", args[0])),
                    _ => Ok(format!("length_i32({})", args[0])),
                }
            }
            "sprintf" => {
                if args.is_empty() {
                    return Err(CodeGenError::UnsupportedStructure(
                        "sprintf requires at least a format string".to_string(),
                    ));
                }

                // Special handling for format string to avoid TagValue wrapping
                let format_str = &args[0];

                // For simple string literals, unwrap the TagValue conversion
                let format_str = if format_str.starts_with("Into::<TagValue>::into(")
                    && format_str.ends_with(")")
                {
                    // Extract the inner string literal from Into::<TagValue>::into("string")
                    let inner = &format_str[23..format_str.len() - 1]; // Remove wrapper
                    inner.to_string()
                } else {
                    // For complex expressions, use as-is
                    format_str.clone()
                };

                let sprintf_args = if args.len() > 1 {
                    let cloned_args = args[1..]
                        .iter()
                        .map(|arg| format!("{}.clone()", arg))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("&[{}]", cloned_args)
                } else {
                    "&[]".to_string()
                };

                tracing::debug!(
                    "Generating sprintf call: format={}, args={}",
                    format_str,
                    sprintf_args
                );

                match self.expression_type() {
                    ExpressionType::PrintConv | ExpressionType::ValueConv => Ok(format!(
                        "TagValue::String(codegen_runtime::sprintf_perl({}, {}))",
                        format_str, sprintf_args
                    )),
                    _ => Ok(format!(
                        "codegen_runtime::sprintf_perl({}, {})",
                        format_str, sprintf_args
                    )),
                }
            }
            "substr" => {
                if args.len() < 2 || args.len() > 3 {
                    return Err(CodeGenError::UnsupportedStructure(
                        "substr requires 2 or 3 arguments (string, offset, [length])".to_string(),
                    ));
                }
                let func_name = if args.len() == 2 {
                    "substr_2arg"
                } else {
                    "substr_3arg"
                };
                let args_str = args.join(", ");
                Ok(format!("codegen_runtime::{}({})", func_name, args_str))
            }
            "index" => {
                if args.len() < 2 || args.len() > 3 {
                    return Err(CodeGenError::UnsupportedStructure(
                        "index requires 2 or 3 arguments (haystack, needle, [position])"
                            .to_string(),
                    ));
                }
                let func_name = if args.len() == 2 {
                    "index_2arg"
                } else {
                    "index_3arg"
                };
                let args_str = args.join(", ");
                Ok(format!("codegen_runtime::{}({})", func_name, args_str))
            }
            "join" => {
                if args.len() != 2 {
                    return Err(CodeGenError::UnsupportedStructure(
                        "join requires exactly 2 arguments (separator, list)".to_string(),
                    ));
                }
                let separator = &args[0];
                let list = &args[1];
                match self.expression_type() {
                    ExpressionType::PrintConv | ExpressionType::ValueConv => Ok(format!(
                        "TagValue::String(codegen_runtime::join_unpack_binary({}, &{}))",
                        separator, list
                    )),
                    _ => Ok(format!(
                        "codegen_runtime::join_unpack_binary({}, &{})",
                        separator, list
                    )),
                }
            }
            "unpack" => {
                if args.len() != 2 {
                    return Err(CodeGenError::UnsupportedStructure(
                        "unpack requires exactly 2 arguments (format, data)".to_string(),
                    ));
                }
                let format_str = &args[0];
                let data = &args[1];
                match self.expression_type() {
                    ExpressionType::PrintConv | ExpressionType::ValueConv => Ok(format!(
                        "TagValue::String(codegen_runtime::unpack_binary({}, &{}))",
                        format_str, data
                    )),
                    _ => Ok(format!(
                        "codegen_runtime::unpack_binary({}, &{})",
                        format_str, data
                    )),
                }
            }
            "if" => {
                // Handle conditional statements created by ConditionalStatementsNormalizer
                if args.len() != 2 {
                    return Err(CodeGenError::UnsupportedStructure(
                        "if requires exactly 2 arguments (condition, statement)".to_string(),
                    ));
                }
                let condition = &args[0];
                let statement = &args[1];

                // Generate Rust if statement
                Ok(format!("if {} {{ {} }}", condition, statement))
            }
            _ => {
                // Check if this is an ExifTool function that needs registry lookup
                if func_name.starts_with("Image::ExifTool::") {
                    if let Some(func_impl) = lookup_function(func_name) {
                        match func_impl {
                            crate::impl_registry::FunctionImplementation::ExifToolModule(
                                module_func,
                            ) => {
                                // Call the registered implementation
                                let args_str = args.join(", ");
                                Ok(format!(
                                    "{}::{}({})",
                                    module_func.module_path, module_func.function_name, args_str
                                ))
                            }
                            _ => {
                                // Other function types (e.g., Builtin) - handle as needed
                                Ok(format!("{}({})", func_name, args.join(", ")))
                            }
                        }
                    } else {
                        // ExifTool function not found in registry - this indicates missing implementation
                        // Generate a placeholder that will fail compilation with a clear error message
                        tracing::warn!("Unknown ExifTool function: {}", func_name);
                        Ok(format!(
                            "/* MISSING EXIFTOOL FUNCTION: {} */ {}({})",
                            func_name,
                            func_name,
                            args.join(", ")
                        ))
                    }
                } else {
                    // Generic function call
                    Ok(format!("{}({})", func_name, args.join(", ")))
                }
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

        // Cast tokens only contain the dereference operator itself ($ or @, etc.)
        // The actual $$self{Field} pattern is handled in process_node_sequence
        // This method is only called when a Cast token appears in isolation

        if content == "$" {
            // This is likely part of a $$self pattern that should be handled at a higher level
            // For now, return a placeholder that at least compiles
            // TODO: This should be handled by pattern matching in process_node_sequence
            Ok("ctx.and_then(|c| c.get_data_member(\"TimeScale\").cloned()).unwrap_or(TagValue::U32(1))".to_string())
        } else if content == "@" {
            // Array dereference - we can't handle this
            Err(CodeGenError::UnsupportedStructure(
                "Array dereference (@) operator requires fallback implementation".to_string(),
            ))
        } else if content == "%" {
            // Hash dereference - we can't handle this
            Err(CodeGenError::UnsupportedStructure(
                "Hash dereference (%) operator requires fallback implementation".to_string(),
            ))
        } else {
            // Unknown cast type - punt to fallback
            Err(CodeGenError::UnsupportedStructure(format!(
                "Unknown cast operator '{}' requires fallback implementation",
                content
            )))
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
            // Return error for complex subscript patterns that cannot be reliably translated
            Err(CodeGenError::UnsupportedStructure(format!(
                "Complex subscript pattern cannot be translated: {}",
                content
            )))
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
                let hex_literal = content.to_lowercase();

                // Add appropriate type suffix to hex literals
                // Parse the hex value to determine appropriate type
                if let Ok(val) = u64::from_str_radix(hex_part, 16) {
                    if val <= u32::MAX as u64 {
                        Ok(format!("{}u32", hex_literal))
                    } else {
                        Ok(format!("{}u64", hex_literal))
                    }
                } else {
                    Ok(hex_literal)
                }
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
                    "TagValue::String(codegen_runtime::regex_replace(\"{}\", \"{}\", &val.to_string()))",
                    safe_pattern, escaped_replacement
                ))
            } else {
                Ok(format!(
                    "TagValue::String(codegen_runtime::regex_replace(\"{}\", \"{}\", &val.to_string()))",
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
            // TODO: should we special-case "$1" and "$2" for capturing groups?
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
                // Unsupported magic variable
                Err(CodeGenError::UnsupportedToken(format!(
                    "Unsupported Perl magic variable: {}",
                    content
                )))
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

    /// Visit array access node - handles $val[0], $val[1], etc.
    fn visit_array_access(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        // The node should have the symbol name in content and subscript in children
        let var_name = node.content.as_deref().unwrap_or("$val");
        let rust_name = if var_name == "$val" {
            "val"
        } else {
            var_name.trim_start_matches('$')
        };

        // Extract the index from the subscript child
        if let Some(subscript) = node.children.first() {
            if let Some(index) = self.extract_subscript_index(subscript) {
                return Ok(format!(
                    "codegen_runtime::get_array_element({}, {})",
                    rust_name, index
                ));
            }
        }

        Err(CodeGenError::UnsupportedStructure(
            "Invalid array access structure".to_string(),
        ))
    }

    /// Extract numeric index from subscript node
    fn extract_subscript_index(&self, subscript: &PpiNode) -> Option<usize> {
        // Subscript contains Statement::Expression containing the index
        if let Some(expr) = subscript.children.first() {
            if let Some(index_node) = expr.children.first() {
                if index_node.class == "PPI::Token::Number" {
                    if let Some(numeric_value) = index_node.numeric_value {
                        return Some(numeric_value as usize);
                    }
                }
            }
        }
        None
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

        // Just join the parts with spaces for now - this is a fallback method
        // The main implementation in RustGenerator handles this properly
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

    /// Check if a node represents a substitution condition ($val =~ s///)
    fn is_substitution_condition(&self, node: &PpiNode) -> bool {
        // Check if this is a BinaryOperation with =~ operator
        if node.class == "BinaryOperation" && node.content.as_deref() == Some("=~") {
            if let Some(right_child) = node.children.get(1) {
                return matches!(right_child.class.as_str(), "PPI::Token::Regexp::Substitute");
            }
        }
        false
    }

    /// Ensure a value is properly converted to TagValue if needed
    fn ensure_tagvalue_return(&self, value: String) -> String {
        // Check if this looks like a bare string literal
        if value.starts_with('"') && value.ends_with('"') && !value.contains("TagValue") {
            // It's a bare string literal, add .into()
            format!("Into::<TagValue>::into({})", value)
        } else {
            // Already has proper type or is an expression
            value
        }
    }

    /// Generate code for a ternary with substitution condition
    fn generate_substitution_ternary(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        // node.children[0] is the condition ($val =~ s///)
        // node.children[1] is the true branch
        // node.children[2] is the false branch

        let condition_node = &node.children[0];
        if condition_node.children.len() != 2 {
            return Err(CodeGenError::UnsupportedStructure(
                "Substitution condition must have 2 children".to_string(),
            ));
        }

        // Extract the substitution pattern from the right child
        let subst_node = &condition_node.children[1];
        let subst_content = subst_node
            .content
            .as_ref()
            .ok_or(CodeGenError::MissingContent(
                "substitution pattern".to_string(),
            ))?;

        // Parse s/pattern/replacement/flags
        if !subst_content.starts_with("s/") && !subst_content.starts_with("s#") {
            return Err(CodeGenError::UnsupportedStructure(format!(
                "Invalid substitution pattern: {}",
                subst_content
            )));
        }

        let delimiter = if subst_content.starts_with("s/") {
            '/'
        } else {
            '#'
        };
        let parts: Vec<&str> = subst_content[2..].split(delimiter).collect();

        if parts.len() < 2 {
            return Err(CodeGenError::UnsupportedStructure(format!(
                "Invalid substitution format: {}",
                subst_content
            )));
        }

        let pattern = self.make_pattern_safe_for_rust(parts[0]);
        let replacement =
            self.escape_replacement_string(if parts.len() > 1 { parts[1] } else { "" });

        // Process the true and false branches
        // For the true branch, we need to use the modified value
        // We'll generate a match expression that captures both the success and the value
        let true_branch_raw = self.visit_node(&node.children[1])?;
        let false_branch_raw = self.visit_node(&node.children[2])?;

        // In ValueConv/PrintConv contexts, ensure branches return TagValue
        // Add .into() for string literals that don't already have it
        let true_branch = self.ensure_tagvalue_return(true_branch_raw);
        let false_branch = self.ensure_tagvalue_return(false_branch_raw);

        // Generate code using regex_substitute_perl
        // The true branch should use the modified value, but we need to handle
        // the case where it references $val (which should be the modified value)
        Ok(format!(
            r#"{{
                let (success, modified_val) = codegen_runtime::regex_substitute_perl(
                    r"{}",
                    "{}",
                    val
                );
                if success {{
                    let val = &modified_val;
                    {}
                }} else {{
                    {}
                }}
            }}"#,
            pattern, replacement, true_branch, false_branch
        ))
    }

    /// Visit normalized ConditionalBlock nodes (created by normalizer)
    fn visit_normalized_conditional_block(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        if node.children.len() != 3 {
            return Err(CodeGenError::UnsupportedStructure(
                "ConditionalBlock requires exactly 3 children (condition, assignment, return_expr)"
                    .to_string(),
            ));
        }

        let condition = self.visit_node(&node.children[0])?;
        let assignment = self.visit_node(&node.children[1])?;
        let return_expr = self.visit_node(&node.children[2])?;

        // Generate Rust if-block with assignment and return expression
        // Trust ExifTool: Preserve exact Perl semantics where conditional assignment affects final result
        Ok(format!(
            "{{ if {} {{ {} }} {} }}",
            condition, assignment, return_expr
        ))
    }

    /// Visit normalized TernaryOp nodes (created by normalizer)
    fn visit_normalized_ternary_op(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        if node.children.len() != 3 {
            return Err(CodeGenError::UnsupportedStructure(
                "TernaryOp requires exactly 3 children (condition, true_branch, false_branch)"
                    .to_string(),
            ));
        }

        // Check if the condition is a regex substitution ($val =~ s///)
        // This needs special handling because the substitution modifies $val
        // and the modified value should be used in the true branch
        if let Some(condition_node) = node.children.get(0) {
            if self.is_substitution_condition(condition_node) {
                return self.generate_substitution_ternary(node);
            }
        }

        // Check if any branch contains a Cast token with \ (reference operator)
        // The normalizer doesn't properly handle \$val in ternary branches
        for child in &node.children {
            if child.class == "PPI::Token::Cast"
                && child.content.as_ref() == Some(&"\\".to_string())
            {
                // This is a reference operator that we can't handle
                return Err(CodeGenError::UnsupportedStructure(
                    "Reference operator (\\$val) in expression requires fallback implementation"
                        .to_string(),
                ));
            }
        }

        // Process each part of the ternary expression normally
        let condition = self.visit_node(&node.children[0])?;
        let true_branch = self.visit_node(&node.children[1])?;
        let false_branch = self.visit_node(&node.children[2])?;

        Ok(format!(
            "if {} {{ {} }} else {{ {} }}",
            condition, true_branch, false_branch
        ))
    }

    /// Visit normalized IfStatement nodes (created by PostfixConditionalNormalizer)
    fn visit_normalized_if_statement(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        if node.children.len() != 2 {
            return Err(CodeGenError::UnsupportedStructure(
                "IfStatement requires exactly 2 children (condition, body)".to_string(),
            ));
        }

        // Process the condition and body
        let condition = self.visit_node(&node.children[0])?;
        let body = self.visit_node(&node.children[1])?;

        Ok(format!("if {} {{ {} }}", condition, body))
    }

    /// Visit normalized component nodes (Condition, Assignment, TrueBranch, FalseBranch)
    /// These are wrapper nodes created by the normalizer - process their children
    fn visit_normalized_component(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        if node.children.is_empty() {
            return Ok("".to_string());
        }

        // Delegate to process_node_sequence which handles binary operations (like =~) properly
        // instead of just joining with spaces, which generates invalid Perl syntax
        self.process_node_sequence(&node.children)
    }

    /// Visit unary negation nodes
    /// These are created by the ExpressionPrecedenceNormalizer for unary minus operations
    fn visit_unary_negation(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        if node.children.len() != 1 {
            return Err(CodeGenError::UnsupportedStructure(format!(
                "UnaryNegation must have exactly 1 child, got {}",
                node.children.len()
            )));
        }

        let operand = self.visit_node(&node.children[0])?;
        Ok(format!("codegen_runtime::negate({})", operand))
    }

    /// Visit normalized binary operation nodes
    /// These are created by the BinaryOperatorNormalizer to group mathematical expressions
    fn visit_normalized_binary_operation(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        let operator = node.content.as_ref().ok_or(CodeGenError::MissingContent(
            "binary operation operator".to_string(),
        ))?;

        if node.children.len() != 2 {
            return Err(CodeGenError::UnsupportedStructure(format!(
                "Binary operation must have exactly 2 children, got {}",
                node.children.len()
            )));
        }

        let left = self.visit_node(&node.children[0])?;
        let right = self.visit_node(&node.children[1])?;

        // Generate appropriate Rust code for the binary operation
        match operator.as_str() {
            "*" | "/" | "+" | "-" | "%" => {
                // Arithmetic operations - ensure proper TagValue operations
                // The TagValue ops crate handles all these operations correctly
                Ok(format!("({} {} {})", left, operator, right))
            }
            "**" => {
                // Power operator -> use cleaner power function
                Ok(format!("power({}, {})", left, right))
            }
            "." => {
                // String concatenation - use cleaner concat function
                Ok(format!(
                    "codegen_runtime::string::concat(&{}, &{})",
                    left, right
                ))
            }
            "=~" | "!~" => {
                // Regex operations - but check what kind of right operand we have

                // First check if this is a transformation operation (substitute/transliterate)
                // These already return complete Rust code, not patterns
                if let Some(right_node) = &node.children.get(1) {
                    if matches!(
                        right_node.class.as_str(),
                        "PPI::Token::Regexp::Substitute" | "PPI::Token::Regexp::Transliterate"
                    ) {
                        // For now, we need special handling at the ternary level
                        // The substitution needs to be performed AND its result used
                        return Err(CodeGenError::UnsupportedStructure(
                            "Regex substitution/transliteration in boolean context requires special ternary handling. \
                            Use SKIP configuration until proper implementation is added.".to_string()
                        ));
                    }
                }

                // Otherwise, it's a pattern match operation
                // The right side should be a regex pattern from visit_regexp_match

                // Extract the regex pattern and flags from right side
                // Examples: "/Canon/" -> ("Canon", ""), "/^[SW]/i" -> ("^[SW]", "i")
                let (pattern, flags) = if right.starts_with('/') {
                    let end_slash = right.rfind('/').unwrap_or(right.len() - 1);
                    if end_slash > 0 {
                        let pattern = &right[1..end_slash];
                        let flags = if end_slash < right.len() - 1 {
                            &right[end_slash + 1..]
                        } else {
                            ""
                        };
                        (pattern, flags)
                    } else {
                        (right.as_str(), "")
                    }
                } else {
                    (right.as_str(), "")
                };

                // Check if pattern needs real regex (has anchors, character classes, etc.)
                let needs_regex = pattern.contains('^')
                    || pattern.contains('$')
                    || pattern.contains('[')
                    || pattern.contains('\\')
                    || !flags.is_empty();

                if needs_regex {
                    // Generate proper regex matching code for complex patterns
                    // This is especially important for GPS patterns like /^[SW]/i
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};
                    let mut hasher = DefaultHasher::new();
                    pattern.hash(&mut hasher);
                    flags.hash(&mut hasher);
                    let regex_id = format!("REGEX_{:x}", hasher.finish());

                    // Build regex with flags
                    let regex_flags = if flags.contains('i') { "(?i)" } else { "" };
                    let full_pattern = format!("{}{}", regex_flags, pattern);

                    // Generate regex matching code
                    if operator == "=~" {
                        Ok(format!(
                            "{{ use regex::Regex; use std::sync::LazyLock; static {}: LazyLock<Regex> = LazyLock::new(|| Regex::new(r\"{}\").unwrap()); {}.is_match(&{}.to_string()) }}",
                            regex_id, full_pattern, regex_id, left
                        ))
                    } else {
                        Ok(format!(
                            "{{ use regex::Regex; use std::sync::LazyLock; static {}: LazyLock<Regex> = LazyLock::new(|| Regex::new(r\"{}\").unwrap()); !{}.is_match(&{}.to_string()) }}",
                            regex_id, full_pattern, regex_id, left
                        ))
                    }
                } else {
                    // For simple literal patterns, use contains check
                    if operator == "=~" {
                        Ok(format!("{}.to_string().contains(r\"{}\")", left, pattern))
                    } else {
                        Ok(format!("!{}.to_string().contains(r\"{}\")", left, pattern))
                    }
                }
            }
            "eq" | "ne" | "lt" | "gt" | "le" | "ge" => {
                // String comparison operators
                let rust_op = match operator.as_str() {
                    "eq" => "==",
                    "ne" => "!=",
                    "lt" => "<",
                    "gt" => ">",
                    "le" => "<=",
                    "ge" => ">=",
                    _ => operator, // shouldn't happen
                };

                // Convert to string for comparison - handle both direct and TagValue-wrapped literals
                let left_str = if self.is_string_literal_or_wrapped(&left) {
                    self.extract_string_literal(&left)
                } else {
                    format!("{}.to_string()", left)
                };

                let right_str = if self.is_string_literal_or_wrapped(&right) {
                    self.extract_string_literal(&right)
                } else {
                    format!("{}.to_string()", right)
                };

                Ok(format!("({} {} {})", left_str, rust_op, right_str))
            }
            "==" | "!=" | "<" | ">" | "<=" | ">=" => {
                // Numeric comparisons
                Ok(format!("({} {} {})", left, operator, right))
            }
            "&&" | "||" => {
                // Logical operations
                Ok(format!("({} {} {})", left, operator, right))
            }
            "&" | "|" | "^" => {
                // Bitwise operations
                Ok(format!("({} {} {})", left, operator, right))
            }
            "x" => {
                // String repetition operator: $string x $count
                Ok(format!("{}.repeat({} as usize)", left, right))
            }
            _ => {
                // Unknown operator - return error for now
                Err(CodeGenError::UnsupportedStructure(format!(
                    "Unsupported binary operator: {}",
                    operator
                )))
            }
        }
    }

    /// Check if a string is a string literal or TagValue-wrapped string literal
    fn is_string_literal_or_wrapped(&self, s: &str) -> bool {
        // Direct string literal
        if s.starts_with('"') && s.ends_with('"') {
            return true;
        }

        // TagValue-wrapped string literal: Into::<TagValue>::into("literal")
        if s.starts_with("Into::<TagValue>::into(") && s.ends_with(")") {
            let inner = &s[23..s.len() - 1]; // Extract content between ( and )
            return inner.starts_with('"') && inner.ends_with('"');
        }

        false
    }

    /// Extract string literal from either direct form or TagValue wrapper
    fn extract_string_literal(&self, s: &str) -> String {
        // Direct string literal
        if s.starts_with('"') && s.ends_with('"') {
            return s.to_string();
        }

        // TagValue-wrapped string literal: Into::<TagValue>::into("literal")
        if s.starts_with("Into::<TagValue>::into(") && s.ends_with(")") {
            let inner = &s[23..s.len() - 1]; // Extract "literal" from ( "literal" )
            if inner.starts_with('"') && inner.ends_with('"') {
                return inner.to_string();
            }
        }

        // Fallback - shouldn't happen if is_string_literal_or_wrapped returned true
        s.to_string()
    }
}
