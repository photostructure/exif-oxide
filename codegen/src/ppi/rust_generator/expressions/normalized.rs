//! Normalized AST node handling
//!
//! This module handles processing of normalized AST nodes created by the normalizer.
//! These are canonical forms that replace complex patterns with structured representations.

use crate::ppi::rust_generator::errors::CodeGenError;
use crate::ppi::types::*;

/// Trait for handling normalized AST nodes
pub trait NormalizedAstHandler {
    fn expression_type(&self) -> &ExpressionType;

    /// Handle normalized ConditionalBlock nodes from SneakyConditionalAssignmentNormalizer
    fn handle_normalized_conditional_block(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        if node.children.len() != 3 {
            return Err(CodeGenError::UnsupportedStructure(
                "ConditionalBlock requires exactly 3 children (condition, assignment, return_expr)"
                    .to_string(),
            ));
        }

        let condition = self.process_conditional_part(&node.children[0])?;
        let assignment = self.process_conditional_part(&node.children[1])?;
        let return_expr = self.process_conditional_part(&node.children[2])?;

        // Generate Rust if-block with assignment and return expression
        // Trust ExifTool: Preserve exact Perl semantics where conditional assignment affects final result
        Ok(format!(
            "{{ if {} {{ {} }} {} }}",
            condition, assignment, return_expr
        ))
    }

    /// Process a part of a conditional block (condition, assignment, or return expression)
    fn process_conditional_part(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        match node.class.as_str() {
            "Condition" | "Assignment" => {
                // These are wrapper nodes - process their children
                let parts: Result<Vec<String>, CodeGenError> = node
                    .children
                    .iter()
                    .map(|child| self.combine_statement_parts(&[], &[child.clone()]))
                    .collect();
                Ok(parts?.join(" "))
            }
            _ => {
                // Direct node - process it (return expression)
                self.combine_statement_parts(&[], &[node.clone()])
            }
        }
    }

    /// Handle normalized FunctionCall nodes
    fn handle_normalized_function_call(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        let func_name = node.content.as_deref().unwrap_or("");

        // Handle special runtime functions
        match func_name {
            "safe_reciprocal" | "safe_division" => {
                let args = self.process_function_args(&node.children)?;
                Ok(format!(
                    "codegen_runtime::{}({})",
                    func_name,
                    args.join(", ")
                ))
            }
            "log" => {
                let args = self.process_function_args(&node.children)?;
                Ok(format!("log({})", args[0]))
            }
            "length" => {
                let args = self.process_function_args(&node.children)?;
                let var = &args[0];
                match self.expression_type() {
                    ExpressionType::PrintConv => {
                        Ok(format!(
                            "match {} {{ TagValue::String(s) => s.len().to_string(), _ => \"0\".to_string() }}",
                            var
                        ))
                    }
                    ExpressionType::ValueConv => {
                        Ok(format!(
                            "TagValue::I32(match {} {{ TagValue::String(s) => s.len() as i32, _ => 0 }})",
                            var
                        ))
                    }
                    _ => {
                        Ok(format!("match {} {{ TagValue::String(s) => s.len() as i32, _ => 0 }}", var))
                    }
                }
            }
            "sprintf" => self.generate_sprintf_call_from_node(node),
            "sprintf_with_string_concat_repeat" => self.generate_sprintf_concat_repeat_call(node),
            "unpack" => {
                let args = self.process_function_args(&node.children)?;
                Ok(format!(
                    "codegen_runtime::unpack_binary({}, &{})",
                    args[0],
                    args.get(1).unwrap_or(&"val".to_string())
                ))
            }
            _ => {
                // Generic function call
                let args = self.process_function_args(&node.children)?;
                Ok(format!("{}({})", func_name, args.join(", ")))
            }
        }
    }

    /// Process function arguments from child nodes
    fn process_function_args(&self, children: &[PpiNode]) -> Result<Vec<String>, CodeGenError> {
        children
            .iter()
            .map(|child| {
                if let Some(ref content) = child.content {
                    Ok(content.clone())
                } else if let Some(ref string_value) = child.string_value {
                    Ok(format!("\"{}\"", string_value))
                } else {
                    // For complex expressions, delegate back to main combiner
                    self.combine_statement_parts(&[], &[child.clone()])
                }
            })
            .collect()
    }

    /// Generate sprintf with string concatenation and repetition call
    ///
    /// Handles normalized sprintf_with_string_concat_repeat calls from SprintfNormalizer
    /// Example: sprintf("%19d %4d %6d" . " %3d %4d %6d" x 8, split(" ",$val))
    ///
    /// Arguments: [base_format, concat_part, repeat_count, ...args]
    fn generate_sprintf_concat_repeat_call(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        if node.children.len() < 3 {
            return Err(CodeGenError::UnsupportedStructure(
                "sprintf_with_string_concat_repeat needs at least 3 arguments (base_format, concat_part, repeat_count)".to_string(),
            ));
        }

        // Extract the components
        let base_format = self.extract_string_literal(&node.children[0])?;
        let concat_part = self.extract_string_literal(&node.children[1])?;
        let repeat_count = self.extract_numeric_literal(&node.children[2])?;

        // Extract remaining arguments (data to format)
        let remaining_args = if node.children.len() > 3 {
            let args: Vec<String> = node.children[3..]
                .iter()
                .map(|child| self.process_function_args(&[child.clone()]))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .collect();
            format!("&[{}]", args.join(", "))
        } else {
            "val".to_string()
        };

        // Generate call to the helper function
        match self.expression_type() {
            ExpressionType::PrintConv => Ok(format!(
                "TagValue::String(codegen_runtime::sprintf_with_string_concat_repeat(\"{}\", \"{}\", {}, &{}))",
                base_format, concat_part, repeat_count, remaining_args
            )),
            ExpressionType::ValueConv => Ok(format!(
                "TagValue::String(codegen_runtime::sprintf_with_string_concat_repeat(\"{}\", \"{}\", {}, &{}))",
                base_format, concat_part, repeat_count, remaining_args
            )),
            _ => Ok(format!(
                "codegen_runtime::sprintf_with_string_concat_repeat(\"{}\", \"{}\", {}, &{})",
                base_format, concat_part, repeat_count, remaining_args
            )),
        }
    }

    /// Extract string literal value from PPI node
    fn extract_string_literal(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        if let Some(ref string_value) = node.string_value {
            Ok(string_value.clone())
        } else if let Some(ref content) = node.content {
            // Remove quotes if present
            let unquoted = content.trim_matches('"').trim_matches('\'');
            Ok(unquoted.to_string())
        } else {
            Err(CodeGenError::UnsupportedStructure(
                "Expected string literal".to_string(),
            ))
        }
    }

    /// Extract numeric literal value from PPI node
    fn extract_numeric_literal(&self, node: &PpiNode) -> Result<usize, CodeGenError> {
        if let Some(num) = node.numeric_value {
            Ok(num as usize)
        } else if let Some(ref content) = node.content {
            content.parse().map_err(|_| {
                CodeGenError::UnsupportedStructure("Expected numeric literal".to_string())
            })
        } else {
            Err(CodeGenError::UnsupportedStructure(
                "Expected numeric literal".to_string(),
            ))
        }
    }

    /// Generate sprintf call using proper AST traversal instead of string parsing
    fn generate_sprintf_call_from_node(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        if node.children.is_empty() {
            return Err(CodeGenError::UnsupportedStructure(
                "sprintf needs arguments".to_string(),
            ));
        }

        let first_arg = &node.children[0];

        // Check if first argument is a normalized string operation
        match first_arg.class.as_str() {
            "StringConcat" => {
                // Handle sprintf with string concatenation
                return self.handle_sprintf_with_string_operations(node);
            }
            "StringRepeat" => {
                // Handle sprintf with string repetition
                return self.handle_sprintf_with_string_operations(node);
            }
            _ => {
                // Standard sprintf handling
            }
        }

        // Extract format string from first child node
        let format_str = if let Some(ref string_value) = first_arg.string_value {
            format!("\"{}\"", string_value)
        } else if let Some(ref content) = first_arg.content {
            content.clone()
        } else {
            "\"\"".to_string() // Fallback
        };

        // Process remaining arguments using visitor pattern
        let format_args: Result<Vec<String>, CodeGenError> = node.children[1..]
            .iter()
            .map(|child| {
                if let Some(ref content) = child.content {
                    Ok(content.clone())
                } else if let Some(ref string_value) = child.string_value {
                    Ok(format!("\"{}\"", string_value))
                } else {
                    Ok("val".to_string()) // Fallback
                }
            })
            .collect();

        let args_formatted = format_args?.join(", ");

        match self.expression_type() {
            ExpressionType::PrintConv | ExpressionType::ValueConv => Ok(format!(
                "TagValue::String(format!({}, {}))",
                format_str, args_formatted
            )),
            _ => Ok(format!("format!({}, {})", format_str, args_formatted)),
        }
    }

    /// Handle sprintf with normalized string operations (StringConcat, StringRepeat)
    fn handle_sprintf_with_string_operations(
        &self,
        node: &PpiNode,
    ) -> Result<String, CodeGenError> {
        let format_arg = &node.children[0];

        // Try to convert normalized string operations back to sprintf_with_string_concat_repeat pattern
        match format_arg.class.as_str() {
            "StringConcat" => {
                // Check if this is a StringConcat containing a StringRepeat (the full pattern)
                if format_arg.children.len() == 2 && format_arg.children[1].class == "StringRepeat"
                {
                    // This is: StringConcat(base, StringRepeat(part, count))
                    let base_format = self.extract_string_from_node(&format_arg.children[0])?;
                    let repeat_node = &format_arg.children[1];
                    if repeat_node.children.len() >= 2 {
                        let concat_part =
                            self.extract_string_from_node(&repeat_node.children[0])?;
                        let repeat_count =
                            self.extract_number_from_node(&repeat_node.children[1])?;

                        // Generate sprintf_with_string_concat_repeat call
                        let remaining_args = if node.children.len() > 1 {
                            let args: Result<Vec<String>, CodeGenError> = node.children[1..]
                                .iter()
                                .map(|child| self.combine_statement_parts(&[], &[child.clone()]))
                                .collect();
                            args?.join(", ")
                        } else {
                            "val".to_string()
                        };

                        return match self.expression_type() {
                            ExpressionType::PrintConv | ExpressionType::ValueConv => Ok(format!(
                                "TagValue::String(codegen_runtime::sprintf_with_string_concat_repeat(\"{}\", \"{}\", {}, &{}))",
                                base_format, concat_part, repeat_count, remaining_args
                            )),
                            _ => Ok(format!(
                                "codegen_runtime::sprintf_with_string_concat_repeat(\"{}\", \"{}\", {}, &{})",
                                base_format, concat_part, repeat_count, remaining_args
                            )),
                        };
                    }
                }

                // Simple string concatenation - build the format string manually
                let format_parts: Result<Vec<String>, CodeGenError> = format_arg
                    .children
                    .iter()
                    .map(|child| self.extract_string_from_node(child))
                    .collect();
                let combined_format = format_parts?.join("");

                let remaining_args = if node.children.len() > 1 {
                    let args: Result<Vec<String>, CodeGenError> = node.children[1..]
                        .iter()
                        .map(|child| self.combine_statement_parts(&[], &[child.clone()]))
                        .collect();
                    args?.join(", ")
                } else {
                    "val".to_string()
                };

                return match self.expression_type() {
                    ExpressionType::PrintConv | ExpressionType::ValueConv => Ok(format!(
                        "TagValue::String(codegen_runtime::sprintf_perl(\"{}\", &{}))",
                        combined_format, remaining_args
                    )),
                    _ => Ok(format!(
                        "codegen_runtime::sprintf_perl(\"{}\", &{})",
                        combined_format, remaining_args
                    )),
                };
            }
            "StringRepeat" => {
                // Simple string repetition: repeat(string, count)
                if format_arg.children.len() >= 2 {
                    let repeat_string = self.extract_string_from_node(&format_arg.children[0])?;
                    let repeat_count = self.extract_number_from_node(&format_arg.children[1])?;
                    let combined_format = repeat_string.repeat(repeat_count);

                    let remaining_args = if node.children.len() > 1 {
                        let args: Result<Vec<String>, CodeGenError> = node.children[1..]
                            .iter()
                            .map(|child| self.combine_statement_parts(&[], &[child.clone()]))
                            .collect();
                        args?.join(", ")
                    } else {
                        "val".to_string()
                    };

                    return match self.expression_type() {
                        ExpressionType::PrintConv | ExpressionType::ValueConv => Ok(format!(
                            "TagValue::String(codegen_runtime::sprintf_perl(\"{}\", &{}))",
                            combined_format, remaining_args
                        )),
                        _ => Ok(format!(
                            "codegen_runtime::sprintf_perl(\"{}\", &{})",
                            combined_format, remaining_args
                        )),
                    };
                }
            }
            _ => {}
        }

        // Fallback: couldn't handle the pattern, return error
        Err(CodeGenError::UnsupportedStructure(
            "sprintf with unsupported string operations".to_string(),
        ))
    }

    /// Extract a string value from a normalized string node
    fn extract_string_from_node(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        if let Some(ref string_value) = node.string_value {
            Ok(string_value.clone())
        } else if let Some(ref content) = node.content {
            // Remove quotes if present
            let content = content.trim_matches('"').trim_matches('\'');
            Ok(content.to_string())
        } else {
            Err(CodeGenError::UnsupportedStructure(
                "Cannot extract string from node".to_string(),
            ))
        }
    }

    /// Extract a numeric value from a normalized number node
    fn extract_number_from_node(&self, node: &PpiNode) -> Result<usize, CodeGenError> {
        if let Some(num) = node.numeric_value {
            Ok(num as usize)
        } else if let Some(ref content) = node.content {
            content.parse().map_err(|_| {
                CodeGenError::UnsupportedStructure("Cannot parse number from node".to_string())
            })
        } else {
            Err(CodeGenError::UnsupportedStructure(
                "Cannot extract number from node".to_string(),
            ))
        }
    }

    /// Legacy method compatibility - delegate to main combiner
    fn combine_statement_parts(
        &self,
        parts: &[String],
        children: &[PpiNode],
    ) -> Result<String, CodeGenError>;
}
