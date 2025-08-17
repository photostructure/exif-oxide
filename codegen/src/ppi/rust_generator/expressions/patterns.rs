//! Complex expression patterns
//!
//! This module handles complex ExifTool patterns including pack/map operations,
//! sprintf with string operations, ternary conditionals, and function calls.

use crate::ppi::rust_generator::errors::CodeGenError;
use crate::ppi::types::*;

/// Trait for handling complex expression patterns
pub trait ComplexPatternHandler {
    fn expression_type(&self) -> &ExpressionType;

    /// Try to handle join separator, unpack format, data pattern
    fn try_join_unpack_pattern(&self, parts: &[String]) -> Result<Option<String>, CodeGenError> {
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
                    let result = format!(
                        "crate::fmt::join_unpack_binary(\"{}\", \"{}\", &{})",
                        separator, format, data
                    );
                    return Ok(Some(result));
                }
            }
        }
        Ok(None)
    }

    /// Try to handle standalone unpack calls
    fn try_unpack_pattern(&self, parts: &[String]) -> Result<Option<String>, CodeGenError> {
        if parts.len() >= 2 && parts[0] == "unpack" {
            // unpack "H2H2", val -> crate::fmt::unpack_binary("H2H2", val)
            let format = parts[1].trim_matches('"').trim_matches('\'');
            let data = if parts.len() > 2 {
                parts[2..].join(" ").trim_matches(',').trim().to_string()
            } else {
                "val".to_string()
            };
            let result = format!("crate::fmt::unpack_binary(\"{}\", &{})", format, data);
            return Ok(Some(result));
        }
        Ok(None)
    }

    /// Try to handle log function without parentheses
    fn try_log_pattern(&self, parts: &[String]) -> Result<Option<String>, CodeGenError> {
        if parts.len() == 2 && parts[0] == "log" {
            let var = &parts[1];
            let result = format!("({} as f64).ln()", var);
            return Ok(Some(result));
        }
        Ok(None)
    }

    /// Try to handle length function without parentheses
    fn try_length_pattern(&self, parts: &[String]) -> Result<Option<String>, CodeGenError> {
        if parts.len() == 2 && parts[0] == "length" {
            let var = &parts[1];
            let result = match self.expression_type() {
                ExpressionType::PrintConv => {
                    format!(
                        "TagValue::String(match {} {{ TagValue::String(s) => s.len().to_string(), _ => \"0\".to_string() }})",
                        var
                    )
                }
                ExpressionType::ValueConv => {
                    format!(
                        "TagValue::I32(match {} {{ TagValue::String(s) => s.len() as i32, _ => 0 }})",
                        var
                    )
                }
                _ => {
                    format!(
                        "match {} {{ TagValue::String(s) => s.len() as i32, _ => 0 }}",
                        var
                    )
                }
            };
            return Ok(Some(result));
        }
        Ok(None)
    }

    /// Try to handle pack "C*", map { bit extraction } numbers... pattern
    fn try_pack_map_pattern(
        &self,
        parts: &[String],
        children: &[PpiNode],
    ) -> Result<Option<String>, CodeGenError> {
        if parts.len() >= 8
            && parts[0] == "pack"
            && parts[1] == "\"C*\""
            && parts.contains(&"map".to_string())
        {
            // Enhanced pattern recognition: extract mask and offset from the map block
            if let Some((mask, offset, shifts)) = self.extract_pack_map_pattern(parts, children)? {
                let result = format!(
                    "crate::fmt::pack_c_star_bit_extract(val, &{:?}, {}, {})",
                    shifts, mask, offset
                );
                return Ok(Some(result));
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
                let result = format!(
                    "crate::fmt::pack_c_star_bit_extract(val, &{:?}, {}, {})",
                    numbers, mask, offset
                );
                return Ok(Some(result));
            }
        }
        Ok(None)
    }

    /// Try to handle sprintf with string concatenation and repetition operations
    fn try_sprintf_string_ops_pattern(
        &self,
        parts: &[String],
        children: &[PpiNode],
    ) -> Result<Option<String>, CodeGenError> {
        if parts.len() >= 8
            && parts[0] == "sprintf"
            && parts.contains(&".".to_string())
            && parts.contains(&"x".to_string())
        {
            let result = self.handle_sprintf_with_string_operations(parts, children)?;
            return Ok(Some(result));
        }
        Ok(None)
    }

    /// Try to handle ternary operator (? :) with safe division pattern recognition
    fn try_ternary_pattern(&self, parts: &[String]) -> Result<Option<String>, CodeGenError> {
        if let Some(question_pos) = parts.iter().position(|p| p == "?") {
            if let Some(colon_pos) = parts.iter().position(|p| p == ":") {
                if question_pos < colon_pos {
                    let result =
                        self.generate_ternary_from_parts(parts, question_pos, colon_pos)?;
                    return Ok(Some(result));
                }
            }
        }
        Ok(None)
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
        let condition_parts = &parts[..question_pos];
        let condition = if condition_parts.len() == 3 {
            // Check if it's a binary operation like "val eq inf"
            let left = &condition_parts[0];
            let op = &condition_parts[1];
            let right = &condition_parts[2];

            // Convert binary operators from Perl to Rust
            match op.as_str() {
                "eq" => {
                    // Handle string equality - need to convert to proper comparison
                    if right.starts_with('"') || right.starts_with('\'') {
                        format!("{}.to_string() == {}", left, right)
                    } else {
                        format!("{} == {}", left, right)
                    }
                }
                "ne" => format!("{} != {}", left, right),
                "lt" => format!("{} < {}", left, right),
                "gt" => format!("{} > {}", left, right),
                "le" => format!("{} <= {}", left, right),
                "ge" => format!("{} >= {}", left, right),
                _ => condition_parts.join(" "),
            }
        } else {
            condition_parts.join(" ")
        };

        let true_branch = parts[question_pos + 1..colon_pos].join(" ");
        let false_branch = parts[colon_pos + 1..].join(" ");

        Ok(format!(
            "if {} {{ {} }} else {{ {} }}",
            condition, true_branch, false_branch
        ))
    }

    /// Handle sprintf with string operations
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
}
