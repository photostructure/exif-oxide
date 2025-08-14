//! Function generation helpers for PPI code generation
//!
//! This module contains specialized generators for different types of
//! Perl function calls and their Rust equivalents.

use super::errors::CodeGenError;
use crate::ppi::types::*;

/// Trait for generating function calls and related constructs
pub trait FunctionGenerator {
    fn expression_type(&self) -> &ExpressionType;

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

                    match self.expression_type() {
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
                            match self.expression_type() {
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

                    match self.expression_type() {
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
                match self.expression_type() {
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
                match self.expression_type() {
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
        // Extract format string and arguments from (format, arg1, arg2, ...) pattern
        // Only remove the outer parentheses, not nested ones
        let args_inner = if args.starts_with('(') && args.ends_with(')') {
            &args[1..args.len()-1]
        } else {
            args
        };

        // Check if we have a nested function call like split
        if args_inner.contains("crate::types::split_tagvalue") {
            // This is the result from our improved visitor - handle it properly
            // Pattern: sprintf("%.3f x %.3f mm", crate::types::split_tagvalue(&val, " "))

            // Extract format string
            let format_start = args_inner.find('"').unwrap_or(0);
            let format_end = args_inner[format_start + 1..]
                .find('"')
                .map(|i| i + format_start + 1)
                .unwrap_or(args_inner.len());

            if format_start < format_end {
                let format_str = &args_inner[format_start + 1..format_end];

                // Find where the split call starts (after the format string and comma)
                let split_start = if let Some(comma_pos) = args_inner[format_end..].find(',') {
                    format_end + comma_pos + 1
                } else {
                    return Err(CodeGenError::UnsupportedFunction(
                        "sprintf missing arguments".to_string(),
                    ));
                };

                // Get everything after the comma as the split call
                let split_call = args_inner[split_start..].trim();

                // Use our new helper function for cleaner code generation
                return match self.expression_type() {
                    ExpressionType::PrintConv | ExpressionType::ValueConv => Ok(format!(
                        "{{
    let values = {};
    TagValue::String(crate::fmt::sprintf_split_values(\"{}\", &values))
}}",
                        split_call, format_str
                    )),
                    _ => Ok(format!(
                        "{{
    let values = {};
    crate::fmt::sprintf_split_values(\"{}\", &values)
}}",
                        split_call, format_str
                    )),
                };
            }
        }

        // Fallback: handle simple sprintf case: sprintf("format", single_arg)
        let parts: Vec<&str> = args_inner.split(',').map(|s| s.trim()).collect();
        if parts.len() >= 2 {
            let format_str = parts[0].trim_matches('"');
            let arg = parts[1];

            // Convert Perl format to Rust format
            let rust_format = self.convert_perl_format_to_rust(format_str)?;

            match self.expression_type() {
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
        // Use the helper function from src/types/values.rs
        let args_inner = args.trim_start_matches('(').trim_end_matches(')');
        let parts: Vec<&str> = args_inner.split(',').map(|s| s.trim()).collect();

        if parts.len() >= 2 {
            let separator = parts[0].trim_matches('"').trim_matches('\'');
            let data = parts[1];

            // Use the helper function for consistent behavior
            Ok(format!(
                "crate::types::split_tagvalue(&{}, \"{}\")",
                data, separator
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
            let perl_pattern_lower = format!("%.{}x", width);
            let rust_pattern_lower = format!("{{:0{}x}}", width);
            rust_format = rust_format.replace(&perl_pattern_lower, &rust_pattern_lower);

            let perl_pattern_upper = format!("%.{}X", width);
            let rust_pattern_upper = format!("{{:0{}X}}", width);
            rust_format = rust_format.replace(&perl_pattern_upper, &rust_pattern_upper);
        }

        // Integer padding: %.3d -> {:03}, %.5d -> {:05}
        for width in (1..=10).rev() {
            let perl_pattern = format!("%.{}d", width);
            let rust_pattern = format!("{{:0{}}}", width);
            rust_format = rust_format.replace(&perl_pattern, &rust_pattern);
        }

        // Float precision: %.3f -> {:.3}, %.2f -> {:.2}, %.0f -> {:.0}
        for precision in (0..=10).rev() {
            // Include 0 for %.0f
            let perl_pattern = format!("%.{}f", precision);
            let rust_pattern = format!("{{:.{}}}", precision);
            rust_format = rust_format.replace(&perl_pattern, &rust_pattern);
        }

        // Width without padding: %6d -> {:6}, %10s -> {:10}
        for width in (1..=20).rev() {
            let perl_pattern_d = format!("%{}d", width);
            let rust_pattern_d = format!("{{:{}}}", width);
            rust_format = rust_format.replace(&perl_pattern_d, &rust_pattern_d);

            let perl_pattern_s = format!("%{}s", width);
            let rust_pattern_s = format!("{{:{}}}", width);
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
}
