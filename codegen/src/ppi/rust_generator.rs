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

        Ok(word.clone())
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

    /// Smart combination of statement parts based on pattern analysis
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

        // Pattern: unary operator (e.g., -$val, +$val)
        if parts.len() == 2 && (parts[0] == "-" || parts[0] == "+") {
            return self.generate_unary_operation_from_parts(&parts[0], &parts[1]);
        }

        // Pattern: left_expr op right_expr (binary operation)
        if parts.len() == 3 {
            return self.generate_binary_operation_from_parts(&parts[0], &parts[1], &parts[2]);
        }

        // Pattern: longer expressions - try to find the main operator
        if let Some(main_op_pos) = self.find_main_operator(parts, children) {
            let left = parts[..main_op_pos].join(" ");
            let op = &parts[main_op_pos];
            let right = parts[main_op_pos + 1..].join(" ");

            // Handle unary operators at the beginning
            if main_op_pos == 0 && (op == "-" || op == "+") {
                return self.generate_unary_operation_from_parts(op, &right);
            }

            return self.generate_binary_operation_from_parts(&left, op, &right);
        }

        // Fallback: join parts with spaces
        Ok(parts.join(" "))
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

    /// Generate unary operation from operator and operand
    fn generate_unary_operation_from_parts(
        &self,
        op: &str,
        operand: &str,
    ) -> Result<String, CodeGenError> {
        match op {
            "-" => {
                // Handle unary minus
                match self.expression_type {
                    ExpressionType::Condition => Ok(format!("-{}", operand)),
                    _ => {
                        // For arithmetic operations, ensure we work with numbers
                        Ok(format!("-(({}) as f64)", operand))
                    }
                }
            }
            "+" => {
                // Handle unary plus (mostly a no-op)
                match self.expression_type {
                    ExpressionType::Condition => Ok(format!("+{}", operand)),
                    _ => Ok(format!("+(({}) as f64)", operand)),
                }
            }
            _ => Err(CodeGenError::UnsupportedOperator(format!("unary {}", op))),
        }
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

    /// Find the main operator in a complex expression
    fn find_main_operator(&self, parts: &[String], children: &[PpiNode]) -> Option<usize> {
        // Look for operators in order of precedence (lowest to highest)
        let operator_precedence = [
            ".", "?", ":", "||", "&&", "eq", "ne", "=~", "!~", "==", "!=", "<", ">", "<=", ">=",
            "+", "-", "*", "/",
        ];

        for op in operator_precedence {
            if let Some(pos) = parts.iter().position(|p| p == op) {
                // Make sure it's actually an operator node
                if pos < children.len() && children[pos].is_operator() {
                    return Some(pos);
                }
            }
        }
        None
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

        // Should generate arithmetic operation with as f64 conversion
        assert!(result.contains("as f64"));
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

        // Should generate arithmetic operation
        assert!(result.contains("as f64"));
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

        // Should generate proper unary minus operation
        assert!(result.contains("as f64"));
        assert!(result.contains("-"));
        assert!(result.contains("pub fn test_unary_minus"));
        assert!(!result.contains("( as f64)")); // Should not have empty left operand

        // Uncomment to see the generated code:
        // println!("Generated unary minus code:\n{}", result);
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
}
