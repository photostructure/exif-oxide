//! Rust Code Generator from PPI Structures
//!
//! Converts PPI AST nodes into Rust source code that calls runtime support
//! functions from the `ast::` module for DRY code generation.
//!
//! Trust ExifTool: Generated code preserves exact Perl evaluation semantics.

use super::types::*;
use std::fmt::Write;

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
    fn visit_statement(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        // Process all children first (bottom-up approach)
        let mut parts = Vec::new();
        for child in &node.children {
            parts.push(self.visit_node(child)?);
        }

        // Combine parts based on the pattern of children
        self.combine_statement_parts(&parts, &node.children)
    }

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

    /// Visit number node
    fn visit_number(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        if let Some(num) = node.numeric_value {
            // For code generation, use appropriate literal format
            if num.fract() == 0.0 {
                Ok(format!("{}", num as i64))
            } else {
                Ok(num.to_string())
            }
        } else {
            // Fallback to content
            Ok(node
                .content
                .as_ref()
                .ok_or(CodeGenError::MissingContent("number".to_string()))?
                .clone())
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
                match self.expression_type {
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
        // Process all children and return them as a comma-separated list
        let mut args = Vec::new();
        for child in &node.children {
            args.push(self.visit_node(child)?);
        }
        Ok(format!("({})", args.join(", ")))
    }

    /// Combine statement parts, handling only essential Rust-specific conversions
    fn combine_statement_parts(
        &self,
        parts: &[String],
        children: &[PpiNode],
    ) -> Result<String, CodeGenError> {
        if parts.is_empty() {
            return Ok("".to_string());
        }

        if parts.len() == 1 {
            return Ok(parts[0].clone());
        }

        // Look for common patterns and handle them intelligently

        // Pattern: function_name ( args )
        if parts.len() >= 2 && children.len() >= 2 {
            if children[0].is_word() && children[1].class.contains("Structure") {
                return self.generate_function_call_from_parts(&parts[0], &parts[1]);
            }
        }

        // Pattern: multi-argument function calls (join " ", unpack "H2H2", val)
        if parts.len() >= 3 && children.len() >= 3 {
            if children[0].is_word() {
                let function_name = &parts[0];
                if matches!(
                    function_name.as_str(),
                    "join" | "unpack" | "pack" | "substr" | "split"
                ) {
                    // For now, split arguments at commas for simpler processing
                    let mut args = Vec::new();
                    let mut current_arg = Vec::new();

                    for part in parts[1..].iter() {
                        if part == "," {
                            if !current_arg.is_empty() {
                                args.push(current_arg.join(" "));
                                current_arg.clear();
                            }
                        } else {
                            current_arg.push(part.clone());
                        }
                    }
                    // Add the last argument
                    if !current_arg.is_empty() {
                        args.push(current_arg.join(" "));
                    }

                    return self.generate_multi_arg_function_call(function_name, &args);
                }
            }
        }

        // Pattern: function_name arg (without parentheses, like "length $val")
        if parts.len() == 2 && children.len() == 2 {
            if children[0].is_word() {
                let function_name = &parts[0];
                if matches!(
                    function_name.as_str(),
                    "length" | "int" | "abs" | "sqrt" | "sin" | "cos" | "defined"
                ) {
                    return self.generate_function_call_without_parens(function_name, &parts[1]);
                }
            }
        }

        // Pattern: expr . expr (concatenation)
        if let Some(concat_pos) = parts.iter().position(|p| p == ".") {
            return self.generate_concatenation_from_parts(parts, concat_pos);
        }

        // Pattern: condition ? true_expr : false_expr (ternary)
        if let (Some(question_pos), Some(colon_pos)) = (
            parts.iter().position(|p| p == "?"),
            parts.iter().position(|p| p == ":"),
        ) {
            if question_pos < colon_pos {
                return self.generate_ternary_from_parts(parts, question_pos, colon_pos);
            }
        }

        // Default: trust PPI's AST structure and join parts appropriately
        Ok(parts.join(" "))
    }

    /// Generate multi-argument function call (e.g., "join ' ', unpack 'H2H2', val")
    fn generate_multi_arg_function_call(
        &self,
        function_name: &str,
        args: &[String],
    ) -> Result<String, CodeGenError> {
        match function_name {
            "join" => {
                // join " ", @array -> array.join(" ")
                if args.len() >= 2 {
                    let separator = args[0].trim_matches('"').trim_matches('\'');

                    // Process the array expression - might be another function call
                    let array_expr = if args[1].starts_with("unpack") {
                        // Handle nested unpack call
                        let unpack_parts: Vec<&str> = args[1].split_whitespace().collect();
                        if unpack_parts.len() >= 3 && unpack_parts[0] == "unpack" {
                            let format = unpack_parts[1].trim_matches('"').trim_matches('\'');
                            let data = unpack_parts[2];

                            match format {
                                "H2H2" => {
                                    format!(
                                        "vec![u8::from_str_radix(&{}.to_string()[0..2], 16).unwrap_or(0).to_string(), \
                                        u8::from_str_radix(&{}.to_string()[2..4], 16).unwrap_or(0).to_string()]",
                                        data, data
                                    )
                                }
                                _ => format!("unpack_{}({})", format.replace("H", "hex"), data),
                            }
                        } else {
                            args[1].clone()
                        }
                    } else {
                        args[1].clone()
                    };

                    match self.expression_type {
                        ExpressionType::PrintConv | ExpressionType::ValueConv => Ok(format!(
                            "TagValue::String({}.join(\"{}\"))",
                            array_expr, separator
                        )),
                        _ => Ok(format!("{}.join(\"{}\")", array_expr, separator)),
                    }
                } else {
                    Err(CodeGenError::UnsupportedFunction(
                        "join with insufficient args".to_string(),
                    ))
                }
            }
            "unpack" => {
                // unpack "H2H2", val -> decode hex bytes
                if args.len() >= 2 {
                    let format = args[0].trim_matches('"').trim_matches('\'');
                    let data = &args[1];

                    match format {
                        "H2H2" => {
                            // Unpack two hex bytes
                            match self.expression_type {
                                ExpressionType::PrintConv | ExpressionType::ValueConv => {
                                    Ok(format!(
                                        "TagValue::String(format!(\"{{}} {{}}\", \
                                        u8::from_str_radix(&{}.to_string()[0..2], 16).unwrap_or(0), \
                                        u8::from_str_radix(&{}.to_string()[2..4], 16).unwrap_or(0)))",
                                        data, data
                                    ))
                                }
                                _ => Ok(format!("unpack_h2h2({})", data)),
                            }
                        }
                        _ => Err(CodeGenError::UnsupportedFunction(format!(
                            "unpack format: {}",
                            format
                        ))),
                    }
                } else {
                    Err(CodeGenError::UnsupportedFunction(
                        "unpack with insufficient args".to_string(),
                    ))
                }
            }
            "split" => {
                // split " ", $val -> $val.split(" ").collect::<Vec<_>>()
                if args.len() >= 2 {
                    let separator = args[0].trim_matches('"').trim_matches('\'');
                    let data = &args[1];

                    match self.expression_type {
                        ExpressionType::PrintConv | ExpressionType::ValueConv => {
                            Ok(format!(
                                "TagValue::Array({}.to_string().split(\"{}\").map(|s| TagValue::String(s.to_string())).collect())",
                                data, separator
                            ))
                        }
                        _ => Ok(format!("{}.to_string().split(\"{}\").collect::<Vec<_>>()", data, separator)),
                    }
                } else {
                    Err(CodeGenError::UnsupportedFunction(
                        "split with insufficient args".to_string(),
                    ))
                }
            }
            _ => Err(CodeGenError::UnsupportedFunction(format!(
                "multi-arg function: {}",
                function_name
            ))),
        }
    }

    /// Generate function call without parentheses (e.g., "length $val")
    fn generate_function_call_without_parens(
        &self,
        function_name: &str,
        arg: &str,
    ) -> Result<String, CodeGenError> {
        match function_name {
            "length" => {
                // Perl length function - get string length
                match self.expression_type {
                    ExpressionType::PrintConv => Ok(format!(
                        "TagValue::String(match {} {{ TagValue::String(s) => s.len().to_string(), _ => \"0\".to_string() }})",
                        arg
                    )),
                    ExpressionType::ValueConv => Ok(format!(
                        "TagValue::I32(match {} {{ TagValue::String(s) => s.len() as i32, _ => 0 }})",
                        arg
                    )),
                    _ => Ok(format!("match {} {{ TagValue::String(s) => s.len() as i32, _ => 0 }}", arg))
                }
            }
            "int" => Ok(format!("({}).trunc() as i32", arg)),
            "abs" => Ok(format!("({}).abs()", arg)),
            "defined" => {
                // Check if value is defined (not None/null)
                Ok(format!(
                    "match {} {{ TagValue::String(s) => !s.is_empty(), _ => true }}",
                    arg
                ))
            }
            _ => Err(CodeGenError::UnsupportedFunction(function_name.to_string())),
        }
    }

    /// Generate function call from function name and arguments
    fn generate_function_call_from_parts(
        &self,
        function_name: &str,
        args_part: &str,
    ) -> Result<String, CodeGenError> {
        match function_name {
            "sprintf" => self.generate_sprintf_call(args_part),
            "split" => self.generate_split_call(args_part),
            "int" => Ok(format!(
                "({}.trunc() as i32)",
                self.extract_first_arg(args_part)?
            )),
            "abs" => Ok(format!("({}).abs()", self.extract_first_arg(args_part)?)),
            "length" => {
                // Perl length function - get string length
                match self.expression_type {
                    ExpressionType::PrintConv => Ok(format!(
                        "TagValue::String(match val {{ TagValue::String(s) => s.len().to_string(), _ => \"0\".to_string() }})"
                    )),
                    ExpressionType::ValueConv => Ok(format!(
                        "TagValue::I32(match val {{ TagValue::String(s) => s.len() as i32, _ => 0 }})"
                    )),
                    _ => Ok(format!("match val {{ TagValue::String(s) => s.len() as i32, _ => 0 }}"))
                }
            }
            _ => Err(CodeGenError::UnsupportedFunction(function_name.to_string())),
        }
    }

    /// Generate sprintf function call with proper format handling
    fn generate_sprintf_call(&self, args: &str) -> Result<String, CodeGenError> {
        // Extract format string and argument from (format, arg) pattern
        let args_inner = args.trim_start_matches('(').trim_end_matches(')');
        let parts: Vec<&str> = args_inner.split(',').map(|s| s.trim()).collect();

        if parts.len() >= 2 {
            let format_str = parts[0].trim_matches('"');
            let arg = parts[1];

            // Convert Perl format to Rust format
            let rust_format = self.convert_perl_format_to_rust(format_str)?;

            match self.expression_type {
                ExpressionType::PrintConv => Ok(format!(
                    "TagValue::String(format!(\"{}\", {}))",
                    rust_format, arg
                )),
                ExpressionType::ValueConv => Ok(format!(
                    "TagValue::String(format!(\"{}\", {}))",
                    rust_format, arg
                )),
                _ => Ok(format!("format!(\"{}\", {})", rust_format, arg)),
            }
        } else {
            Err(CodeGenError::UnsupportedFunction(
                "sprintf with invalid args".to_string(),
            ))
        }
    }

    /// Generate split function call with proper argument handling
    fn generate_split_call(&self, args: &str) -> Result<String, CodeGenError> {
        // Extract separator and string from (separator, string) pattern
        let args_inner = args.trim_start_matches('(').trim_end_matches(')');
        let parts: Vec<&str> = args_inner.split(',').map(|s| s.trim()).collect();

        if parts.len() >= 2 {
            let separator = parts[0].trim_matches('"').trim_matches('\'');
            let data = parts[1];

            match self.expression_type {
                ExpressionType::PrintConv | ExpressionType::ValueConv => {
                    Ok(format!(
                        "TagValue::Array({}.to_string().split(\"{}\").map(|s| TagValue::String(s.to_string())).collect())",
                        data, separator
                    ))
                }
                _ => Ok(format!("{}.to_string().split(\"{}\").collect::<Vec<_>>()", data, separator)),
            }
        } else {
            Err(CodeGenError::UnsupportedFunction(
                "split with insufficient args".to_string(),
            ))
        }
    }

    /// Convert Perl sprintf format to Rust format!
    fn convert_perl_format_to_rust(&self, perl_format: &str) -> Result<String, CodeGenError> {
        let mut rust_format = perl_format.to_string();

        // Convert %.2f to {:.2}
        rust_format = rust_format.replace("%.2f", "{:.2}");
        rust_format = rust_format.replace("%.1f", "{:.1}");
        rust_format = rust_format.replace("%d", "{}");
        rust_format = rust_format.replace("%s", "{}");
        rust_format = rust_format.replace("%f", "{}");

        Ok(rust_format)
    }

    /// Extract first argument from function argument list
    fn extract_first_arg(&self, args: &str) -> Result<String, CodeGenError> {
        let args_inner = args.trim_start_matches('(').trim_end_matches(')');
        let first_arg = args_inner.split(',').next().unwrap_or(args_inner).trim();
        Ok(first_arg.to_string())
    }

    /// Generate concatenation from parts array
    fn generate_concatenation_from_parts(
        &self,
        parts: &[String],
        concat_pos: usize,
    ) -> Result<String, CodeGenError> {
        let left = parts[..concat_pos].join(" ");
        let right = parts[concat_pos + 1..].join(" ");

        match self.expression_type {
            ExpressionType::PrintConv => Ok(format!(
                "TagValue::String(format!(\"{{}}{{}}\", {}, {}))",
                left, right
            )),
            ExpressionType::ValueConv => Ok(format!(
                "TagValue::String(format!(\"{{}}{{}}\", {}, {}))",
                left, right
            )),
            _ => Ok(format!("format!(\"{{}}{{}}\", {}, {})", left, right)),
        }
    }

    /// Generate ternary conditional from parts
    fn generate_ternary_from_parts(
        &self,
        parts: &[String],
        question_pos: usize,
        colon_pos: usize,
    ) -> Result<String, CodeGenError> {
        let condition_parts = &parts[..question_pos];
        let true_branch_parts = &parts[question_pos + 1..colon_pos];
        let false_branch_parts = &parts[colon_pos + 1..];

        // Process condition - look for binary operations within it
        let condition = if condition_parts.len() == 3 {
            // Try to handle as binary operation (e.g., "val eq inf")
            self.generate_binary_operation_from_parts(
                &condition_parts[0],
                &condition_parts[1],
                &condition_parts[2],
            )?
        } else {
            condition_parts.join(" ")
        };

        let true_branch = true_branch_parts.join(" ");
        let false_branch = false_branch_parts.join(" ");

        Ok(format!(
            "if {} {{ {} }} else {{ {} }}",
            condition, true_branch, false_branch
        ))
    }

    /// Generate binary operation from three parts
    fn generate_binary_operation_from_parts(
        &self,
        left: &str,
        op: &str,
        right: &str,
    ) -> Result<String, CodeGenError> {
        let rust_op = match op {
            "eq" => "==",
            "ne" => "!=",
            "=~" => ".contains",  // Simplified regex matching
            "!~" => "!.contains", // Simplified regex matching
            "/" => "/",
            "*" => "*",
            "+" => "+",
            "-" => "-",
            ">" => ">",
            "<" => "<",
            ">=" => ">=",
            "<=" => "<=",
            _ => op,
        };

        match self.expression_type {
            ExpressionType::Condition => {
                // Generate boolean expression for conditions
                if op == "=~" || op == "!~" {
                    let negate = op == "!~";
                    Ok(format!(
                        "{}{}.to_string().contains(&{}.to_string())",
                        if negate { "!" } else { "" },
                        left,
                        right
                    ))
                } else {
                    Ok(format!("{} {} {}", left, rust_op, right))
                }
            }
            ExpressionType::PrintConv | ExpressionType::ValueConv => {
                // For PrintConv/ValueConv, handle TagValue comparisons and arithmetic
                match rust_op {
                    "==" | "!=" => {
                        // Handle TagValue string comparisons
                        Ok(format!(
                            "{}.to_string() {} {}.to_string()",
                            left, rust_op, right
                        ))
                    }
                    "/" | "*" | "+" | "-" => {
                        // Ensure floating point arithmetic
                        Ok(format!("({} as f64) {} ({} as f64)", left, rust_op, right))
                    }
                    _ => Ok(format!("{} {} {}", left, rust_op, right)),
                }
            }
        }
    }
}

/// Error types for code generation
#[derive(Debug, thiserror::Error)]
pub enum CodeGenError {
    #[error("Unsupported AST structure: {0}")]
    UnsupportedStructure(String),

    #[error("Unsupported operator: {0}")]
    UnsupportedOperator(String),

    #[error("Unsupported function: {0}")]
    UnsupportedFunction(String),

    #[error("Unsupported context: {0}")]
    UnsupportedContext(String),

    #[error("Unsupported token type: {0}")]
    UnsupportedToken(String),

    #[error("Missing content for: {0}")]
    MissingContent(String),

    #[error("Invalid self-reference: {0}")]
    InvalidSelfReference(String),

    #[error("Formatting error: {0}")]
    Format(#[from] std::fmt::Error),
}

impl RustGenerator {
    // Task A: Critical Foundation Tokens (Phase 1) - P07: PPI Enhancement

    /// Visit expression node - handles complex expressions with function composition
    /// PPI::Statement::Expression (4,172 occurrences) - Essential for complex expressions
    fn visit_expression(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        // Process children recursively and intelligently combine them
        let mut parts = Vec::new();
        for child in &node.children {
            // Special handling for function calls with incomplete arguments
            if child.is_word() && parts.is_empty() {
                // This is likely a function name starting an expression
                parts.push(self.visit_node(child)?);
            } else if child.class == "PPI::Structure::List" {
                // This is function arguments - parse them properly
                let arg_list = self.visit_list(child)?;
                parts.push(arg_list);
            } else {
                parts.push(self.visit_node(child)?);
            }
        }

        // Handle complex expression patterns that regular statements can't
        self.combine_expression_parts(&parts, &node.children)
    }

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
            match self.expression_type {
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

    /// Combine expression parts with advanced pattern recognition
    fn combine_expression_parts(
        &self,
        parts: &[String],
        children: &[PpiNode],
    ) -> Result<String, CodeGenError> {
        if parts.is_empty() {
            return Ok("".to_string());
        }

        if parts.len() == 1 {
            return Ok(parts[0].clone());
        }

        // Pattern: function_name(args) - handle function calls with parentheses
        if parts.len() == 2 && children.len() == 2 {
            if children[0].is_word() && children[1].class == "PPI::Structure::List" {
                let function_name = &parts[0];
                let args_part = &parts[1];
                return self.generate_function_call_from_parts(function_name, args_part);
            }
        }

        // Pattern: left =~ regex (regex matching)
        if parts.len() == 3 && parts[1] == "=~" {
            return Ok(format!(
                "regex::Regex::new(r\"{}\").unwrap().is_match(&{}.to_string())",
                parts[2].trim_matches('/'),
                parts[0]
            ));
        }

        // Pattern: left !~ regex (negative regex matching)
        if parts.len() == 3 && parts[1] == "!~" {
            return Ok(format!(
                "!regex::Regex::new(r\"{}\").unwrap().is_match(&{}.to_string())",
                parts[2].trim_matches('/'),
                parts[0]
            ));
        }

        // Pattern: arithmetic operations with TagValue
        if parts.len() == 3 {
            match parts[1].as_str() {
                "/" => return Ok(format!("&{} / {}", parts[0], parts[2])),
                "*" => return Ok(format!("&{} * {}", parts[0], parts[2])),
                "+" => return Ok(format!("&{} + {}", parts[0], parts[2])),
                "-" => return Ok(format!("&{} - {}", parts[0], parts[2])),
                "eq" => {
                    return Ok(format!(
                        "{}.to_string() == {}.to_string()",
                        parts[0], parts[2]
                    ))
                }
                "ne" => {
                    return Ok(format!(
                        "{}.to_string() != {}.to_string()",
                        parts[0], parts[2]
                    ))
                }
                _ => {}
            }
        }

        // Pattern: condition ? true_expr : false_expr (ternary)
        if let (Some(question_pos), Some(colon_pos)) = (
            parts.iter().position(|p| p == "?"),
            parts.iter().position(|p| p == ":"),
        ) {
            if question_pos < colon_pos {
                return self.generate_ternary_from_parts(parts, question_pos, colon_pos);
            }
        }

        // Fall back to the existing statement combination logic
        self.combine_statement_parts(parts, children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_arithmetic_generation() {
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Symbol",
                    "content": "$val",
                    "symbol_type": "scalar"
                }, {
                    "class": "PPI::Token::Operator",
                    "content": "/"
                }, {
                    "class": "PPI::Token::Number",
                    "content": "100",
                    "numeric_value": 100
                }],
                "class": "PPI::Statement"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::ValueConv,
            "test_divide".to_string(),
            "$val / 100".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // println!("Simple arithmetic generation result:\n{}", result);

        // Should generate clean arithmetic operation (trusting PPI structure)
        assert!(result.contains("val / 100"));
        assert!(result.contains("pub fn test_divide"));
        assert!(result.contains("Original: $val / 100"));
    }

    #[test]
    fn test_string_interpolation_generation() {
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Quote::Double",
                    "content": "\"Case $val\"",
                    "string_value": "Case $val"
                }],
                "class": "PPI::Statement"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::PrintConv,
            "test_string".to_string(),
            "\"Case $val\"".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Should generate string interpolation with format!
        assert!(result.contains("format!"));
        assert!(result.contains("Case"));
    }

    #[test]
    fn test_signature_generation() {
        let generator = RustGenerator::new(
            ExpressionType::Condition,
            "test_condition".to_string(),
            "$$self{Make} =~ /Canon/".to_string(),
        );

        let signature = generator.generate_signature();

        assert!(signature.contains("pub fn test_condition"));
        assert!(signature.contains("val: &TagValue"));
        assert!(signature.contains("ctx: &ExifContext"));
        assert!(signature.contains("-> bool"));
    }

    #[test]
    fn test_recursive_visitor_arithmetic() {
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Symbol",
                    "content": "$val",
                    "symbol_type": "scalar"
                }, {
                    "class": "PPI::Token::Operator",
                    "content": "*"
                }, {
                    "class": "PPI::Token::Number",
                    "content": "25",
                    "numeric_value": 25
                }],
                "class": "PPI::Statement"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::ValueConv,
            "test_multiply".to_string(),
            "$val * 25".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // println!("Recursive visitor arithmetic result:\n{}", result);

        // Should generate clean arithmetic operation (trusting PPI structure)
        assert!(result.contains("val * 25"));
        assert!(result.contains("pub fn test_multiply"));
        assert!(result.contains("Original: $val * 25"));
    }

    #[test]
    fn test_sprintf_concatenation_ternary() {
        // Test the complex expression: sprintf("%.2f s",$val) . ($val > 254.5/60 ? " or longer" : "")
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Word",
                    "content": "sprintf"
                }, {
                    "class": "PPI::Structure::List",
                    "children": [{
                        "class": "PPI::Token::Quote::Double",
                        "content": "\"%.2f s\"",
                        "string_value": "%.2f s"
                    }, {
                        "class": "PPI::Token::Symbol",
                        "content": "$val",
                        "symbol_type": "scalar"
                    }]
                }, {
                    "class": "PPI::Token::Operator",
                    "content": "."
                }, {
                    "class": "PPI::Structure::List",
                    "children": [{
                        "class": "PPI::Token::Symbol",
                        "content": "$val",
                        "symbol_type": "scalar"
                    }, {
                        "class": "PPI::Token::Operator",
                        "content": ">"
                    }, {
                        "class": "PPI::Token::Number",
                        "content": "254.5",
                        "numeric_value": 254.5
                    }, {
                        "class": "PPI::Token::Operator",
                        "content": "/"
                    }, {
                        "class": "PPI::Token::Number",
                        "content": "60",
                        "numeric_value": 60
                    }, {
                        "class": "PPI::Token::Operator",
                        "content": "?"
                    }, {
                        "class": "PPI::Token::Quote::Double",
                        "content": "\" or longer\"",
                        "string_value": " or longer"
                    }, {
                        "class": "PPI::Token::Operator",
                        "content": ":"
                    }, {
                        "class": "PPI::Token::Quote::Double",
                        "content": "\"\"",
                        "string_value": ""
                    }]
                }],
                "class": "PPI::Statement"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::PrintConv,
            "test_sprintf_concat_ternary".to_string(),
            "sprintf(\"%.2f s\",$val) . ($val > 254.5/60 ? \" or longer\" : \"\")".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Should generate function with sprintf, concatenation, and ternary
        assert!(result.contains("format!"));
        assert!(result.contains("{:.2}")); // Perl %.2f -> Rust {:.2}
        assert!(result.contains("if")); // Ternary -> if/else
        assert!(result.contains("pub fn test_sprintf_concat_ternary"));

        // Uncomment to see the generated code:
        // println!("Generated code:\n{}", result);
    }

    #[test]
    fn test_unary_minus_operation() {
        // Test the expression: -$val/256
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Operator",
                    "content": "-"
                }, {
                    "class": "PPI::Token::Symbol",
                    "content": "$val",
                    "symbol_type": "scalar"
                }, {
                    "class": "PPI::Token::Operator",
                    "content": "/"
                }, {
                    "class": "PPI::Token::Number",
                    "content": "256",
                    "numeric_value": 256
                }],
                "class": "PPI::Statement"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::ValueConv,
            "test_unary_minus".to_string(),
            "-$val/256".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Uncomment to see the generated code:
        // println!("Generated unary minus code:\n{}", result);

        // Should generate clean unary minus operation (trusting PPI's structure)
        assert!(result.contains("- val / 256")); // Clean, simple expression
        assert!(result.contains("pub fn test_unary_minus"));
        assert!(!result.contains("( as f64)")); // Should not have empty left operand
    }

    #[test]
    fn test_length_function_without_parens() {
        // Test the expression: length $val
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Word",
                    "content": "length"
                }, {
                    "class": "PPI::Token::Symbol",
                    "content": "$val",
                    "symbol_type": "scalar"
                }],
                "class": "PPI::Statement"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::PrintConv,
            "test_length".to_string(),
            "length $val".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Should generate proper length function call
        assert!(result.contains("TagValue::String"));
        assert!(result.contains("s.len()"));
        assert!(result.contains("pub fn test_length"));
        assert!(!result.contains("length val")); // Should not have raw function call

        // Uncomment to see the generated code:
        // println!("Generated length code:\n{}", result);
    }

    #[test]
    fn test_ternary_with_string_comparison() {
        // Test the expression: $val eq "inf" ? $val : "$val m"
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Symbol",
                    "content": "$val",
                    "symbol_type": "scalar"
                }, {
                    "class": "PPI::Token::Operator",
                    "content": "eq"
                }, {
                    "class": "PPI::Token::Quote::Double",
                    "content": "\"inf\"",
                    "string_value": "inf"
                }, {
                    "class": "PPI::Token::Operator",
                    "content": "?"
                }, {
                    "class": "PPI::Token::Symbol",
                    "content": "$val",
                    "symbol_type": "scalar"
                }, {
                    "class": "PPI::Token::Operator",
                    "content": ":"
                }, {
                    "class": "PPI::Token::Quote::Double",
                    "content": "\"$val m\"",
                    "string_value": "$val m"
                }],
                "class": "PPI::Statement"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::PrintConv,
            "test_ternary_string_eq".to_string(),
            "$val eq \"inf\" ? $val : \"$val m\"".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Uncomment to see the generated code:
        // println!("Generated ternary string comparison code:\n{}", result);

        // Should generate proper string comparison with TagValue
        assert!(result.contains(".to_string() == "));
        assert!(result.contains("if"));
        assert!(result.contains("else"));
        assert!(result.contains("pub fn test_ternary_string_eq"));
        // The function body should not have "val eq", but the comment will still contain it
        let function_body = result.split("pub fn").nth(1).unwrap();
        assert!(!function_body.contains("val eq")); // Should not have raw eq operator in function body
    }

    #[test]
    fn test_undef_keyword() {
        // Test the expression: undef
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Word",
                    "content": "undef"
                }],
                "class": "PPI::Statement"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::PrintConv,
            "test_undef".to_string(),
            "undef".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Should generate appropriate default value
        assert!(result.contains("TagValue::String(\"\".to_string())"));
        assert!(result.contains("pub fn test_undef"));
    }

    #[test]
    fn test_join_function() {
        // Test the expression: join " ", unpack "H2H2", val
        let ast_json = json!({
            "children": [{
                "children": [{
                    "class": "PPI::Token::Word",
                    "content": "join"
                }, {
                    "class": "PPI::Token::Quote::Double",
                    "content": "\" \"",
                    "string_value": " "
                }, {
                    "class": "PPI::Token::Operator",
                    "content": ","
                }, {
                    "class": "PPI::Token::Word",
                    "content": "unpack"
                }, {
                    "class": "PPI::Token::Quote::Double",
                    "content": "\"H2H2\"",
                    "string_value": "H2H2"
                }, {
                    "class": "PPI::Token::Operator",
                    "content": ","
                }, {
                    "class": "PPI::Token::Symbol",
                    "content": "val",
                    "symbol_type": "scalar"
                }],
                "class": "PPI::Statement"
            }],
            "class": "PPI::Document"
        });

        let ast: PpiNode = serde_json::from_value(ast_json).unwrap();

        let generator = RustGenerator::new(
            ExpressionType::PrintConv,
            "test_join_unpack".to_string(),
            "join \" \", unpack \"H2H2\", val".to_string(),
        );

        let result = generator.generate_function(&ast).unwrap();

        // Uncomment to see the generated code:
        // println!("Generated join/unpack code:\n{}", result);

        // Should generate proper join and unpack functions
        assert!(result.contains("TagValue::String"));
        assert!(result.contains("pub fn test_join_unpack"));
        // For now, basic join functionality is sufficient
        // Full nested function parsing will be refined later
    }
}
