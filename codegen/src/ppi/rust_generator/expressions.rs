//! Simplified expression combination logic (post-normalization)
//!
//! This module handles combining parsed PPI tokens into Rust expressions.
//! Most complex patterns are handled by the AST normalizer, so this only
//! needs to handle basic cases and normalized forms.

use super::errors::CodeGenError;
use crate::ppi::types::*;

/// Trait for combining expression parts into coherent Rust code
pub trait ExpressionCombiner {
    fn expression_type(&self) -> &ExpressionType;

    /// Combine statement parts, handling normalized AST nodes and basic patterns
    fn combine_statement_parts(
        &self,
        parts: &[String],
        children: &[PpiNode],
    ) -> Result<String, CodeGenError> {
        if parts.is_empty() {
            return Ok("".to_string());
        }

        // Handle normalized AST nodes first
        if !children.is_empty() {
            match children[0].class.as_str() {
                "FunctionCall" => {
                    return self.handle_normalized_function_call(&children[0]);
                }
                "StringConcat" => {
                    return self.handle_normalized_string_concat(&children[0]);
                }
                "StringRepeat" => {
                    return self.handle_normalized_string_repeat(&children[0]);
                }
                _ => {}
            }
        }

        if parts.len() == 1 {
            return Ok(parts[0].clone());
        }

        // Handle remaining essential patterns that aren't normalized

        // Pattern: join separator, unpack format, data (common ExifTool pattern)
        if parts.len() >= 5 && parts[0] == "join" && parts.contains(&"unpack".to_string()) {
            // Find the positions of join arguments and unpack call
            if let Some(unpack_pos) = parts.iter().position(|p| p == "unpack") {
                if unpack_pos >= 3 && unpack_pos + 2 < parts.len() {
                    // Expected structure: join "separator", unpack "format", data
                    let separator = parts[1].trim_matches('"').trim_matches('\'');
                    let format = parts[unpack_pos + 1].trim_matches('"').trim_matches('\'');
                    let data = if unpack_pos + 2 < parts.len() {
                        parts[unpack_pos + 2..]
                            .join(" ")
                            .trim_matches(',')
                            .trim()
                            .to_string()
                    } else {
                        "val".to_string()
                    };
                    return Ok(format!(
                        "crate::fmt::join_unpack_binary(\"{}\", \"{}\", &{})",
                        separator, format, data
                    ));
                }
            }
        }

        // Pattern: handle standalone 'unpack' calls
        if parts.len() >= 2 && parts[0] == "unpack" {
            // unpack "H2H2", val -> crate::fmt::unpack_binary("H2H2", val)
            let format = parts[1].trim_matches('"').trim_matches('\'');
            let data = if parts.len() > 2 {
                parts[2..].join(" ").trim_matches(',').trim().to_string()
            } else {
                "val".to_string()
            };
            return Ok(format!(
                "crate::fmt::unpack_binary(\"{}\", &{})",
                format, data
            ));
        }

        // Pattern: log function without parentheses (log $val -> val.ln())
        if parts.len() == 2 && parts[0] == "log" {
            let var = &parts[1];
            return Ok(format!("({} as f64).ln()", var));
        }

        // Pattern: length function without parentheses (length $val -> val.len())
        if parts.len() == 2 && parts[0] == "length" {
            let var = &parts[1];
            match self.expression_type() {
                ExpressionType::PrintConv => {
                    return Ok(format!(
                        "TagValue::String(match {} {{ TagValue::String(s) => s.len().to_string(), _ => \"0\".to_string() }})",
                        var
                    ));
                }
                ExpressionType::ValueConv => {
                    return Ok(format!(
                        "TagValue::I32(match {} {{ TagValue::String(s) => s.len() as i32, _ => 0 }})",
                        var
                    ));
                }
                _ => {
                    return Ok(format!(
                        "match {} {{ TagValue::String(s) => s.len() as i32, _ => 0 }}",
                        var
                    ));
                }
            }
        }

        // Pattern: pack "C*", map { bit extraction } numbers... (specific case)
        // From ExifTool Canon.pm line 1847: pack "C*", map { (($_>>$_)&0x1f)+0x60 } 10, 5, 0
        if parts.len() >= 8
            && parts[0] == "pack"
            && parts[1] == "\"C*\""
            && parts.contains(&"map".to_string())
        {
            // Enhanced pattern recognition: extract mask and offset from the map block
            if let Some((mask, offset, shifts)) = self.extract_pack_map_pattern(parts, children)? {
                return Ok(format!(
                    "crate::fmt::pack_c_star_bit_extract(val, &{:?}, {}, {})",
                    shifts, mask, offset
                ));
            }

            // Fallback: Extract all numbers from the parts (these are the shift values)
            let numbers: Vec<i32> = parts
                .iter()
                .filter_map(|s| s.parse::<i32>().ok())
                .filter(|&n| n >= 0 && n <= 32) // Only reasonable shift values
                .collect();

            // Use common ExifTool patterns as fallback
            let mask = 0x1f; // Common mask for 5-bit extraction
            let offset = 0x60; // Common offset for ASCII range

            if !numbers.is_empty() {
                return Ok(format!(
                    "crate::fmt::pack_c_star_bit_extract(val, &{:?}, {}, {})",
                    numbers, mask, offset
                ));
            }
        }

        // Pattern: sprintf with string concatenation and repetition operations
        // From ExifTool Canon.pm line 763: sprintf("%19d %4d %6d" . " %3d %4d %6d" x 8, split(" ",$val))
        if parts.len() >= 8
            && parts[0] == "sprintf"
            && parts.contains(&".".to_string())
            && parts.contains(&"x".to_string())
        {
            return self.handle_sprintf_with_string_operations(parts, children);
        }

        // Ternary operator (? :) - Enhanced with safe division pattern recognition
        if let Some(question_pos) = parts.iter().position(|p| p == "?") {
            if let Some(colon_pos) = parts.iter().position(|p| p == ":") {
                if question_pos < colon_pos {
                    return self.generate_ternary_from_parts(parts, question_pos, colon_pos);
                }
            }
        }

        // String concatenation pattern: expr . expr
        if let Some(dot_pos) = parts.iter().position(|p| p == ".") {
            if dot_pos > 0 && dot_pos < parts.len() - 1 {
                let left_parts = &parts[..dot_pos];
                let right_parts = &parts[dot_pos + 1..];

                // Join the parts back into expressions
                let left_expr = left_parts.join(" ");
                let right_expr = right_parts.join(" ");

                // Generate string concatenation using format!
                match self.expression_type() {
                    ExpressionType::PrintConv | ExpressionType::ValueConv => {
                        return Ok(format!(
                            "TagValue::String(format!(\"{{}}{{}}\", {}, {}))",
                            left_expr, right_expr
                        ));
                    }
                    _ => {
                        return Ok(format!(
                            "format!(\"{{}}{{}}\", {}, {})",
                            left_expr, right_expr
                        ));
                    }
                }
            }
        }

        // Binary operations
        for (i, part) in parts.iter().enumerate() {
            if self.is_binary_operator(part) {
                if i > 0 && i < parts.len() - 1 {
                    let left = parts[..i].join(" ");
                    let right = parts[i + 1..].join(" ");
                    return self.generate_binary_operation_from_parts(&left, part, &right);
                }
            }
        }

        // Default: join parts
        Ok(parts.join(" "))
    }

    /// Handle normalized FunctionCall nodes
    fn handle_normalized_function_call(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        let func_name = node.content.as_deref().unwrap_or("");

        // Handle special runtime functions
        match func_name {
            "safe_reciprocal" | "safe_division" => {
                let args = self.process_function_args(&node.children)?;
                Ok(format!("crate::fmt::{}({})", func_name, args.join(", ")))
            }
            "log" => {
                let args = self.process_function_args(&node.children)?;
                Ok(format!("({} as f64).ln()", args[0]))
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
                    "crate::fmt::unpack_binary({}, &{})",
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
                    self.combine_statement_parts(&[], &[child.clone()])
                }
            })
            .collect()
    }

    /// Handle normalized StringConcat nodes
    fn handle_normalized_string_concat(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        // Process each child node properly - they may be complex expressions
        let mut parts = Vec::new();

        for child in &node.children {
            let part = if let Some(ref content) = child.content {
                // Simple content
                content.clone()
            } else if let Some(ref string_value) = child.string_value {
                // String literal
                format!("\"{}\"", string_value)
            } else {
                // Complex expression - recursively process it
                self.combine_statement_parts(&[], &[child.clone()])?
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
                "TagValue::String(crate::fmt::sprintf_with_string_concat_repeat(\"{}\", \"{}\", {}, &{}))",
                base_format, concat_part, repeat_count, remaining_args
            )),
            ExpressionType::ValueConv => Ok(format!(
                "TagValue::String(crate::fmt::sprintf_with_string_concat_repeat(\"{}\", \"{}\", {}, &{}))",
                base_format, concat_part, repeat_count, remaining_args
            )),
            _ => Ok(format!(
                "crate::fmt::sprintf_with_string_concat_repeat(\"{}\", \"{}\", {}, &{})",
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

    /// Generate ternary conditional with safe division pattern recognition
    /// From ExifTool Canon.pm line 1234: $val ? 1/$val : 0
    fn generate_ternary_from_parts(
        &self,
        parts: &[String],
        question_pos: usize,
        colon_pos: usize,
    ) -> Result<String, CodeGenError> {
        let condition_parts: Vec<&str> = parts[..question_pos].iter().map(|s| s.as_str()).collect();
        let true_branch_parts: Vec<&str> = parts[question_pos + 1..colon_pos]
            .iter()
            .map(|s| s.as_str())
            .collect();
        let false_branch_parts: Vec<&str> =
            parts[colon_pos + 1..].iter().map(|s| s.as_str()).collect();

        // Pattern: $val ? 1 / $val : 0 (safe reciprocal)
        // Pattern: $val ? N / $val : 0 (safe division)
        if condition_parts.len() == 1
            && true_branch_parts.len() == 3
            && true_branch_parts[1] == "/"
            && false_branch_parts.len() == 1
            && false_branch_parts[0] == "0"
            && condition_parts[0] == true_branch_parts[2]
        // Same variable
        {
            let numerator = true_branch_parts[0];
            let variable = condition_parts[0];

            // If numerator is 1, use safe_reciprocal
            if numerator == "1" {
                return Ok(format!("crate::fmt::safe_reciprocal(&{})", variable));
            }
            // If numerator is a constant, use safe_division
            else if numerator.parse::<f64>().is_ok() {
                return Ok(format!(
                    "crate::fmt::safe_division({}.0, &{})",
                    numerator, variable
                ));
            }
        }

        // Default ternary handling
        let condition = parts[..question_pos].join(" ");
        let true_branch = parts[question_pos + 1..colon_pos].join(" ");
        let false_branch = parts[colon_pos + 1..].join(" ");

        Ok(format!(
            "if {} {{ {} }} else {{ {} }}",
            condition, true_branch, false_branch
        ))
    }

    /// Generate binary operation  
    fn generate_binary_operation_from_parts(
        &self,
        left: &str,
        op: &str,
        right: &str,
    ) -> Result<String, CodeGenError> {
        // Handle power operator specially
        if op == "**" {
            // Perl's ** operator is exponentiation, use powf in Rust
            return Ok(format!("({} as f64).powf({} as f64)", left, right));
        }

        // Handle string concatenation operator
        if op == "." {
            // Perl's . operator is string concatenation, use format! in Rust
            match self.expression_type() {
                ExpressionType::PrintConv | ExpressionType::ValueConv => {
                    return Ok(format!(
                        "TagValue::String(format!(\"{{}}{{}}\", {}, {}))",
                        left, right
                    ));
                }
                _ => {
                    return Ok(format!("format!(\"{{}}{{}}\", {}, {})", left, right));
                }
            }
        }

        // Handle regex match operators
        if op == "=~" || op == "!~" {
            // Extract the regex pattern from right side (e.g., "/\\d/" -> "\\d")
            let pattern = if right.starts_with('/') && right.ends_with('/') {
                &right[1..right.len() - 1]
            } else {
                right
            };

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

        let rust_op = self.perl_to_rust_operator(op)?;
        Ok(format!("{} {} {}", left, rust_op, right))
    }

    /// Check if a string is a binary operator
    fn is_binary_operator(&self, s: &str) -> bool {
        matches!(
            s,
            "+" | "-"
                | "*"
                | "**"  // Power operator
                | "/"
                | "%"
                | "=="
                | "!="
                | "<"
                | ">"
                | "<="
                | ">="
                | "&&"
                | "||"
                | "&"
                | "|"
                | "^"
                | "<<"
                | ">>"
                | "eq"
                | "ne"
                | "lt"
                | "gt"
                | "le"
                | "ge"
                | "and"
                | "or"
                | "xor"
                | "=~"  // Regex match
                | "!~" // Regex no-match
                | "." // String concatenation
        )
    }

    /// Convert Perl operators to Rust
    fn perl_to_rust_operator(&self, op: &str) -> Result<String, CodeGenError> {
        Ok(match op {
            "eq" => "==",
            "ne" => "!=",
            "lt" => "<",
            "gt" => ">",
            "le" => "<=",
            "ge" => ">=",
            "and" => "&&",
            "or" => "||",
            "xor" => "^",
            _ => op,
        }
        .to_string())
    }

    /// Generate sprintf call using proper AST traversal instead of string parsing
    fn generate_sprintf_call_from_node(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        if node.children.is_empty() {
            return Err(CodeGenError::UnsupportedStructure(
                "sprintf needs arguments".to_string(),
            ));
        }

        // Extract format string from first child node
        let format_str = if let Some(ref string_value) = node.children[0].string_value {
            format!("\"{}\"", string_value)
        } else if let Some(ref content) = node.children[0].content {
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

    /// Generate sprintf call (string-based interface for compatibility)
    fn generate_sprintf_call(&self, args: &[String]) -> Result<String, CodeGenError> {
        if args.is_empty() {
            return Err(CodeGenError::UnsupportedStructure(
                "sprintf needs arguments".to_string(),
            ));
        }

        let format_str = &args[0];
        let format_args = args[1..].join(", ");

        match self.expression_type() {
            ExpressionType::PrintConv | ExpressionType::ValueConv => Ok(format!(
                "TagValue::String(format!({}, {}))",
                format_str, format_args
            )),
            _ => Ok(format!("format!({}, {})", format_str, format_args)),
        }
    }

    // Legacy methods - will be removed after full normalizer integration
    fn generate_function_call_from_parts(&self, _node: &PpiNode) -> Result<String, CodeGenError> {
        Err(CodeGenError::UnsupportedStructure(
            "Use normalizer for function calls".to_string(),
        ))
    }

    fn generate_multi_arg_function_call(
        &self,
        _func: &str,
        _node: &PpiNode,
    ) -> Result<String, CodeGenError> {
        Err(CodeGenError::UnsupportedStructure(
            "Use normalizer for multi-arg function calls".to_string(),
        ))
    }

    fn generate_function_call_without_parens(
        &self,
        _func: &str,
        _arg: &str,
    ) -> Result<String, CodeGenError> {
        Err(CodeGenError::UnsupportedStructure(
            "Use normalizer for function calls without parens".to_string(),
        ))
    }

    fn handle_sprintf_with_string_operations(
        &self,
        parts: &[String],
        _children: &[PpiNode],
    ) -> Result<String, CodeGenError> {
        // Pattern: sprintf("format1" . "format2" x count, args...)
        // From ExifTool Canon.pm: sprintf("%19d %4d %6d" . " %3d %4d %6d" x 8, split(" ",$val))

        // Find the main format string (between sprintf and first .)
        let mut format_parts = Vec::new();
        let mut in_format = false;
        let mut current_part = String::new();

        for (i, part) in parts.iter().enumerate() {
            if i == 0 && part == "sprintf" {
                in_format = true;
                continue;
            }

            if in_format {
                if part == "." {
                    // Found concatenation, end first format part
                    if !current_part.is_empty() {
                        format_parts.push(current_part.clone());
                        current_part.clear();
                    }
                    break;
                }
                if part.starts_with('"') || current_part.is_empty() {
                    current_part = part.clone();
                } else {
                    current_part.push(' ');
                    current_part.push_str(part);
                }
            }
        }

        // For now, use a simplified approach that delegates to regular sprintf
        // This maintains ExifTool compatibility while avoiding complex string parsing
        let args_part = parts[1..].join(" ");

        match self.expression_type() {
            ExpressionType::PrintConv | ExpressionType::ValueConv => Ok(format!(
                "TagValue::String(crate::fmt::sprintf_with_string_ops(\"{}\", &{}))",
                args_part.trim_matches('(').trim_matches(')'),
                "val"
            )),
            _ => Ok(format!(
                "crate::fmt::sprintf_with_string_ops(\"{}\", &{})",
                args_part.trim_matches('(').trim_matches(')'),
                "val"
            )),
        }
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
}
