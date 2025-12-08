//! Function generation helpers for PPI code generation
//!
//! This module contains specialized generators for different types of
//! Perl function calls and their Rust equivalents.

#![allow(dead_code)]

use super::errors::CodeGenError;
use super::visitor::PpiVisitor;
use crate::ppi::types::*;
use indoc::formatdoc;

/// Trait for generating function calls and related constructs
pub trait FunctionGenerator: PpiVisitor {
    /// Generate multi-argument function call using AST traversal (e.g., "join ' ', unpack 'H2H2', val")
    fn generate_multi_arg_function_call(
        &self,
        function_name: &str,
        node: &PpiNode,
    ) -> Result<String, CodeGenError> {
        match function_name {
            "join" => {
                // join " ", @array -> array.join(" ") - using AST traversal
                if node.children.len() >= 2 {
                    // Extract separator from first child node
                    let separator = if let Some(ref string_value) = node.children[0].string_value {
                        string_value.clone()
                    } else if let Some(ref content) = node.children[0].content {
                        content.trim_matches('"').trim_matches('\'').to_string()
                    } else {
                        " ".to_string() // Default separator
                    };

                    // Check for unpack function call in second argument using AST
                    let array_expr = if node.children.len() >= 3
                        && node.children[1].content.as_deref() == Some("unpack")
                    {
                        // Handle multi-argument unpack call using structured AST
                        format!(
                            "codegen_runtime::unpack_binary(\"{}\", &{})",
                            node.children[1].string_value.as_deref().unwrap_or("H2H2"),
                            node.children[2].content.as_deref().unwrap_or("val")
                        )
                    } else if node.children[1].content.as_deref() == Some("unpack") {
                        // Handle nested unpack call using AST
                        format!(
                            "codegen_runtime::unpack_binary(\"{}\", &val)",
                            node.children[1].string_value.as_deref().unwrap_or("H2H2")
                        )
                    } else {
                        // Process second argument normally using visitor pattern
                        if let Some(ref content) = node.children[1].content {
                            content.clone()
                        } else {
                            "val".to_string() // Default fallback
                        }
                    };

                    match self.expression_type() {
                        ExpressionType::PrintConv | ExpressionType::ValueConv => Ok(format!(
                            "TagValue::String({array_expr}.join(\"{separator}\"))"
                        )),
                        _ => Ok(format!("{array_expr}.join(\"{separator}\")")),
                    }
                } else {
                    Err(CodeGenError::UnsupportedFunction(
                        "join with insufficient args".to_string(),
                    ))
                }
            }
            "unpack" => {
                // unpack "H2H2", val -> decode hex bytes
                if node.children.len() >= 2 {
                    let format = if let Some(ref content) = node.children[0].content {
                        content.trim_matches('"').trim_matches('\'')
                    } else {
                        "H2H2"
                    };
                    let data = if let Some(ref content) = node.children[1].content {
                        content
                    } else {
                        "val"
                    };

                    match format {
                        "H2H2" => {
                            // Unpack two hex bytes
                            match self.expression_type() {
                                ExpressionType::PrintConv | ExpressionType::ValueConv => {
                                    Ok(formatdoc! {r#"
                                        TagValue::String(format!("{{}} {{}}",
                                            u8::from_str_radix(&{data}.to_string()[0..2], 16).unwrap_or(0),
                                            u8::from_str_radix(&{data}.to_string()[2..4], 16).unwrap_or(0)
                                        ))
                                    "#})
                                }
                                _ => Ok(format!("unpack_h2h2({data})")),
                            }
                        }
                        _ => Err(CodeGenError::UnsupportedFunction(format!(
                            "unpack format: {format}"
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
                if node.children.len() >= 2 {
                    let separator = if let Some(ref content) = node.children[0].content {
                        content.trim_matches('"').trim_matches('\'')
                    } else {
                        " "
                    };
                    let data = if let Some(ref content) = node.children[1].content {
                        content
                    } else {
                        "val"
                    };

                    match self.expression_type() {
                        ExpressionType::PrintConv | ExpressionType::ValueConv => {
                            Ok(format!(
                                "TagValue::Array({data}.to_string().split(\"{separator}\").map(|s| TagValue::String(s.to_string())).collect())"
                            ))
                        }
                        _ => Ok(format!("{data}.to_string().split(\"{separator}\").collect::<Vec<_>>()")),
                    }
                } else {
                    Err(CodeGenError::UnsupportedFunction(
                        "split with insufficient args".to_string(),
                    ))
                }
            }
            _ => Err(CodeGenError::UnsupportedFunction(format!(
                "multi-arg function: {function_name}"
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
                match self.expression_type() {
                    ExpressionType::PrintConv => Ok(format!(
                        "TagValue::String(match {arg} {{ TagValue::String(s) => s.len().to_string(), _ => \"0\".to_string() }})"
                    )),
                    ExpressionType::ValueConv => Ok(format!(
                        "TagValue::I32(match {arg} {{ TagValue::String(s) => s.len() as i32, _ => 0 }})"
                    )),
                    _ => Ok(format!("match {arg} {{ TagValue::String(s) => s.len() as i32, _ => 0 }}"))
                }
            }
            "int" => Ok(format!("({arg}).trunc() as i32")),
            "abs" => Ok(format!("({arg}).abs()")),
            "log" => Ok(format!("codegen_runtime::log({arg})")),
            "defined" => {
                // Use the cleaner runtime helper
                Ok(format!("codegen_runtime::string::is_defined(&{arg})"))
            }
            _ => Err(CodeGenError::UnsupportedFunction(function_name.to_string())),
        }
    }

    /// Generate function call from AST node containing function name and arguments
    fn generate_function_call_from_parts(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        // Extract function name from the AST node
        let function_name = if let Some(ref content) = node.content {
            content.clone()
        } else if !node.children.is_empty() {
            if let Some(ref content) = node.children[0].content {
                content.clone()
            } else {
                return Err(CodeGenError::UnsupportedStructure(
                    "function call missing function name".to_string(),
                ));
            }
        } else {
            return Err(CodeGenError::UnsupportedStructure(
                "function call missing function name".to_string(),
            ));
        };
        match function_name.as_str() {
            "sprintf" => self.generate_sprintf_call(node),
            "split" => {
                // Extract arguments from AST for split function
                if node.children.len() >= 2 {
                    let separator = if let Some(ref content) = node.children[0].content {
                        content.trim_matches('"').trim_matches('\'')
                    } else {
                        " "
                    };
                    let data = if let Some(ref content) = node.children[1].content {
                        content
                    } else {
                        "val"
                    };

                    Ok(format!(
                        "crate::types::split_tagvalue(&{data}, \"{separator}\")"
                    ))
                } else {
                    Err(CodeGenError::UnsupportedFunction(
                        "split with insufficient args".to_string(),
                    ))
                }
            }
            "int" => {
                // Extract first argument from AST
                let first_arg = if !node.children.is_empty() {
                    if let Some(ref content) = node.children[0].content {
                        content.clone()
                    } else {
                        "val".to_string()
                    }
                } else {
                    "val".to_string()
                };
                Ok(format!("({first_arg}.trunc() as i32)"))
            }
            "abs" => {
                // Extract first argument from AST
                let first_arg = if !node.children.is_empty() {
                    if let Some(ref content) = node.children[0].content {
                        content.clone()
                    } else {
                        "val".to_string()
                    }
                } else {
                    "val".to_string()
                };
                Ok(format!("({first_arg}).abs()"))
            }
            "log" => {
                // Extract first argument from AST
                let first_arg = if !node.children.is_empty() {
                    if let Some(ref content) = node.children[0].content {
                        content.clone()
                    } else {
                        "val".to_string()
                    }
                } else {
                    "val".to_string()
                };
                Ok(format!("(Into::<f64>::into({first_arg})).ln()"))
            }
            "length" => {
                // Perl length function - get string length
                match self.expression_type() {
                    ExpressionType::PrintConv => Ok("TagValue::String(match val { TagValue::String(s) => s.len().to_string(), _ => \"0\".to_string() })".to_string()),
                    ExpressionType::ValueConv => Ok("TagValue::I32(match val { TagValue::String(s) => s.len() as i32, _ => 0 })".to_string()),
                    _ => Ok("match val { TagValue::String(s) => s.len() as i32, _ => 0 }".to_string())
                }
            }
            "unpack" => {
                // Perl unpack function - binary data extraction using AST
                if node.children.len() >= 2 {
                    let format = if let Some(ref content) = node.children[0].content {
                        content.trim_matches('"').trim_matches('\'')
                    } else {
                        "H2H2"
                    };
                    let data = if let Some(ref content) = node.children[1].content {
                        content
                    } else {
                        "val"
                    };

                    match self.expression_type() {
                        ExpressionType::PrintConv | ExpressionType::ValueConv => Ok(format!(
                            "codegen_runtime::unpack_binary(\"{format}\", &{data})"
                        )),
                        _ => Ok(format!(
                            "codegen_runtime::unpack_binary(\"{format}\", &{data})"
                        )),
                    }
                } else {
                    Err(CodeGenError::UnsupportedFunction(
                        "unpack with insufficient args".to_string(),
                    ))
                }
            }
            _ => Err(CodeGenError::UnsupportedFunction(function_name.to_string())),
        }
    }

    /// Generate sprintf function call with proper AST handling
    fn generate_sprintf_call(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        if node.children.is_empty() {
            return Err(CodeGenError::UnsupportedStructure(
                "sprintf needs arguments".to_string(),
            ));
        }

        // The children might be wrapped in a Statement::Expression
        // Check if first child is a Statement::Expression and unwrap if needed
        let actual_args =
            if node.children.len() == 1 && node.children[0].class == "PPI::Statement::Expression" {
                &node.children[0].children
            } else {
                &node.children
            };

        if actual_args.is_empty() {
            return Err(CodeGenError::UnsupportedStructure(
                "sprintf needs arguments".to_string(),
            ));
        }

        // Extract format string from first argument, skipping commas
        let mut arg_index = 0;
        let format_str = loop {
            if arg_index >= actual_args.len() {
                return Err(CodeGenError::UnsupportedStructure(
                    "sprintf missing format string".to_string(),
                ));
            }

            let arg = &actual_args[arg_index];
            // Skip comma operators
            if arg.class == "PPI::Token::Operator" && arg.content.as_deref() == Some(",") {
                arg_index += 1;
                continue;
            }

            // Extract the format string
            let fmt = if let Some(ref string_value) = arg.string_value {
                string_value.clone()
            } else if let Some(ref content) = arg.content {
                content.trim_matches('"').trim_matches('\'').to_string()
            } else {
                return Err(CodeGenError::UnsupportedStructure(
                    "sprintf missing format string".to_string(),
                ));
            };
            arg_index += 1;
            break fmt;
        };

        // Process remaining arguments, skipping commas
        let mut args = Vec::new();
        for child in actual_args.iter().skip(arg_index) {
            // Skip comma operators
            if child.class == "PPI::Token::Operator" && child.content.as_deref() == Some(",") {
                continue;
            }

            let arg = if let Some(ref content) = child.content {
                content.replace("$val", "val")
            } else if let Some(ref string_value) = child.string_value {
                format!("\"{string_value}\"")
            } else {
                "val".to_string() // Fallback
            };
            args.push(arg);
        }

        // Handle different sprintf patterns based on arguments
        if args.is_empty() {
            // No arguments provided, use val as default
            args.push("val".to_string());
        }

        if args.len() == 1 {
            // Simple case: sprintf("format", single_arg)
            let arg = &args[0];

            // Handle division operations in the argument properly
            let processed_arg = if arg.contains('/') && !arg.contains("crate::") {
                // Simple division like "$val/100" or "val/100"
                let div_parts: Vec<&str> = arg.split('/').collect();
                if div_parts.len() == 2 {
                    let left = div_parts[0].trim().replace("$val", "val");
                    let right = div_parts[1].trim();
                    format!("(Into::<f64>::into({left})) / (Into::<f64>::into({right}))")
                } else {
                    arg.replace("$val", "val")
                }
            } else {
                arg.replace("$val", "val")
            };

            // Try to convert to native Rust formatting for simple cases
            if let Ok(rust_format) = self.convert_perl_format_to_rust(&format_str) {
                // Use native Rust format! for simple patterns
                match self.expression_type() {
                    ExpressionType::PrintConv | ExpressionType::ValueConv => Ok(format!(
                        "TagValue::String(format!(\"{rust_format}\", {processed_arg}))"
                    )),
                    _ => Ok(format!("format!(\"{rust_format}\", {processed_arg})")),
                }
            } else {
                // Fallback to sprintf_perl for complex patterns
                match self.expression_type() {
                    ExpressionType::PrintConv | ExpressionType::ValueConv => Ok(format!(
                        "TagValue::String(codegen_runtime::sprintf_perl(\"{format_str}\", &[{processed_arg}]))"
                    )),
                    _ => Ok(format!(
                        "codegen_runtime::sprintf_perl(\"{format_str}\", &[{processed_arg}])"
                    )),
                }
            }
        } else {
            // Multiple arguments: sprintf("format", arg1, arg2, ...)
            let args_formatted = args.join(", ");

            match self.expression_type() {
                ExpressionType::PrintConv | ExpressionType::ValueConv => Ok(format!(
                    "TagValue::String(codegen_runtime::sprintf_perl(\"{format_str}\", &[{args_formatted}]))"
                )),
                _ => Ok(format!(
                    "codegen_runtime::sprintf_perl(\"{format_str}\", &[{args_formatted}])"
                )),
            }
        }
    }

    /// Generate split function call with proper argument handling
    fn generate_split_call(&self, args: &str) -> Result<String, CodeGenError> {
        // Use the helper function from src/types/values.rs
        let args_inner = args.trim_start_matches('(').trim_end_matches(')');
        let parts: Vec<&str> = args_inner.split(',').map(|s| s.trim()).collect();

        if parts.len() >= 2 {
            let separator = parts[0].trim_matches('"').trim_matches('\'');
            let data = parts[1];

            // Use the helper function for consistent behavior
            Ok(format!(
                "crate::types::split_tagvalue(&{data}, \"{separator}\")"
            ))
        } else {
            Err(CodeGenError::UnsupportedFunction(
                "split with insufficient args".to_string(),
            ))
        }
    }

    /// Convert Perl sprintf format to Rust format!
    fn convert_perl_format_to_rust(&self, perl_format: &str) -> Result<String, CodeGenError> {
        let mut rust_format = perl_format.to_string();

        // Handle formats with precision/padding - must process these before generic patterns

        // Hex with padding: %.8x -> {:08x}, %.4X -> {:04X}
        for width in (1..=10).rev() {
            let perl_pattern_lower = format!("%.{width}x");
            let rust_pattern_lower = format!("{{:0{width}x}}");
            rust_format = rust_format.replace(&perl_pattern_lower, &rust_pattern_lower);

            let perl_pattern_upper = format!("%.{width}X");
            let rust_pattern_upper = format!("{{:0{width}X}}");
            rust_format = rust_format.replace(&perl_pattern_upper, &rust_pattern_upper);
        }

        // Integer padding: %.3d -> {:03}, %.5d -> {:05}
        for width in (1..=10).rev() {
            let perl_pattern = format!("%.{width}d");
            let rust_pattern = format!("{{:0{width}}}");
            rust_format = rust_format.replace(&perl_pattern, &rust_pattern);
        }

        // Float precision: %.3f -> {:.3}, %.2f -> {:.2}, %.0f -> {:.0}
        for precision in (0..=10).rev() {
            // Include 0 for %.0f
            let perl_pattern = format!("%.{precision}f");
            let rust_pattern = format!("{{:.{precision}}}");
            rust_format = rust_format.replace(&perl_pattern, &rust_pattern);
        }

        // Width without padding: %6d -> {:6}, %10s -> {:10}
        for width in (1..=20).rev() {
            let perl_pattern_d = format!("%{width}d");
            let rust_pattern_d = format!("{{:{width}}}");
            rust_format = rust_format.replace(&perl_pattern_d, &rust_pattern_d);

            let perl_pattern_s = format!("%{width}s");
            let rust_pattern_s = format!("{{:{width}}}");
            rust_format = rust_format.replace(&perl_pattern_s, &rust_pattern_s);
        }

        // Handle generic formats (no precision/padding)
        rust_format = rust_format.replace("%d", "{}");
        rust_format = rust_format.replace("%s", "{}");
        rust_format = rust_format.replace("%f", "{}");
        rust_format = rust_format.replace("%x", "{:x}"); // lowercase hex
        rust_format = rust_format.replace("%X", "{:X}"); // uppercase hex
        rust_format = rust_format.replace("%o", "{:o}"); // octal

        // Handle escaped percent: %% -> %
        rust_format = rust_format.replace("%%", "%");

        Ok(rust_format)
    }

    /// Extract first argument from function argument list
    fn extract_first_arg(&self, args: &str) -> Result<String, CodeGenError> {
        let args_inner = args.trim_start_matches('(').trim_end_matches(')');
        let first_arg = args_inner.split(',').next().unwrap_or(args_inner).trim();
        Ok(first_arg.to_string())
    }

    /// Check if a PpiNode represents an unpack function call using AST structure
    fn is_unpack_function_call(&self, node: &PpiNode) -> bool {
        // Check if this is a normalized function call with name "unpack"
        if node.class == "FunctionCall" {
            if let Some(ref content) = node.content {
                return content == "unpack";
            }
        }

        // Check if this is a PPI::Statement containing unpack
        if node.class == "PPI::Statement" && !node.children.is_empty() {
            if let Some(ref content) = node.children[0].content {
                return content == "unpack";
            }
        }

        // Check if content directly contains "unpack"
        if let Some(ref content) = node.content {
            return content.starts_with("unpack");
        }

        false
    }

    /// Generate unpack call from AST nodes instead of string parsing
    fn generate_unpack_from_node(
        &self,
        unpack_node: &PpiNode,
        data_node: &PpiNode,
    ) -> Result<String, CodeGenError> {
        // Extract format from unpack node
        let format = if unpack_node.children.len() >= 2 {
            if let Some(ref string_value) = unpack_node.children[1].string_value {
                string_value.clone()
            } else if let Some(ref content) = unpack_node.children[1].content {
                content.trim_matches('"').trim_matches('\'').to_string()
            } else {
                "H2H2".to_string() // Default fallback
            }
        } else {
            // Try to extract from content if children aren't available
            // Default to H2H2 format which is common for hex unpacking
            "H2H2".to_string()
        };

        // Extract data reference
        let data = if !data_node
            .content
            .as_ref()
            .unwrap_or(&String::new())
            .is_empty()
        {
            if let Some(ref content) = data_node.content {
                content.clone()
            } else {
                "val".to_string()
            }
        } else if unpack_node.children.len() >= 3 {
            // Data might be third child of unpack node
            if let Some(ref content) = unpack_node.children[2].content {
                content.clone()
            } else {
                "val".to_string()
            }
        } else {
            "val".to_string()
        };

        match format.as_str() {
            "H2H2" => Ok(formatdoc! {r#"
                    {{
                        let bytes = {data}.as_binary_data();
                        if bytes.len() >= 2 {{
                            vec![format!("{{:02x}}", bytes[0]), format!("{{:02x}}", bytes[1])]
                        }} else {{
                            vec!["00".to_string(), "00".to_string()]
                        }}
                    }}
                "#}),
            _ => {
                // Use generic unpack function for other formats
                Ok(format!(
                    "codegen_runtime::unpack_binary(\"{format}\", {data})?"
                ))
            }
        }
    }
}
