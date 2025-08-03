//! Rust code generation for compiled expressions
//!
//! This module generates Rust code from RPN token sequences,
//! optimizing simple cases and using stack-based evaluation for complex ones.

use super::types::*;

impl CompiledExpression {
    /// Generate Rust code that evaluates this expression using AST
    /// 
    /// Returns code like: `match value.as_f64() { Some(val) => Ok(TagValue::F64(val / 8.0)), None => Ok(value.clone()) }`
    pub fn generate_rust_code(&self) -> String {
        self.generate_ast_code(&self.ast)
    }
    
    /// Generate Rust code from AST node
    fn generate_ast_code(&self, node: &AstNode) -> String {
        format!(
            "match value.as_f64() {{\n        Some(val) => Ok({}),\n        None => Ok(value.clone()),\n    }}",
            self.generate_ast_expression(node)
        )
    }
    
    /// Generate the expression part of AST node
    fn generate_ast_expression(&self, node: &AstNode) -> String {
        match node {
            AstNode::Variable => "TagValue::F64(val)".to_string(),
            AstNode::Number(n) => format!("TagValue::F64({})", format_number(*n)),
            AstNode::String { value, has_interpolation } => {
                if *has_interpolation {
                    // Handle variable interpolation using format! macro
                    let formatted_value = value.replace("$val", "{}");
                    format!("TagValue::String(format!(\"{}\", val))", formatted_value)
                } else {
                    // Simple string literal
                    format!("TagValue::String(\"{}\".to_string())", value)
                }
            }
            AstNode::Undefined => "value.clone()".to_string(), // ExifTool undef - return original value
            AstNode::BinaryOp { op, left, right } => {
                let left_expr = self.generate_value_expression(left);
                let right_expr = self.generate_value_expression(right);
                match op {
                    OpType::Add => format!("TagValue::F64({} + {})", left_expr, right_expr),
                    OpType::Subtract => format!("TagValue::F64({} - {})", left_expr, right_expr),
                    OpType::Multiply => format!("TagValue::F64({} * {})", left_expr, right_expr),
                    OpType::Divide => format!("TagValue::F64({} / {})", left_expr, right_expr),
                    OpType::Power => format!("TagValue::F64({}.powf({}))", left_expr, right_expr),
                    OpType::Concatenate => {
                        // Use concatenation chain to avoid nested format! calls
                        let temp_node = AstNode::BinaryOp { 
                            op: *op, 
                            left: left.clone(), 
                            right: right.clone() 
                        };
                        let chain = self.collect_concatenation_chain(&temp_node);
                        let (format_string, arguments) = self.generate_concatenation_format(&chain, false);
                        if arguments.is_empty() {
                            // All string literals
                            format!("TagValue::String(\"{}\".to_string())", format_string)
                        } else {
                            format!("TagValue::String(format!(\"{}\", {}))", format_string, arguments.join(", "))
                        }
                    }
                    // Bitwise operations - convert to integers first, then back to F64
                    OpType::BitwiseAnd => format!("TagValue::F64((({} as i64) & ({} as i64)) as f64)", left_expr, right_expr),
                    OpType::BitwiseOr => format!("TagValue::F64((({} as i64) | ({} as i64)) as f64)", left_expr, right_expr),
                    OpType::LeftShift => format!("TagValue::F64((({} as i64) << ({} as i64)) as f64)", left_expr, right_expr),
                    OpType::RightShift => format!("TagValue::F64((({} as i64) >> ({} as i64)) as f64)", left_expr, right_expr),
                }
            }
            AstNode::ComparisonOp { op, left, right } => {
                let left_expr = self.generate_value_expression(left);
                let right_expr = self.generate_value_expression(right);
                let rust_op = match op {
                    CompType::GreaterEq => format!("{} >= {}", left_expr, right_expr),
                    CompType::Greater => format!("{} > {}", left_expr, right_expr),
                    CompType::LessEq => format!("{} <= {}", left_expr, right_expr),
                    CompType::Less => format!("{} < {}", left_expr, right_expr),
                    CompType::Equal => format!("{} == {}", left_expr, right_expr),
                    CompType::NotEqual => format!("{} != {}", left_expr, right_expr),
                };
                format!("TagValue::U8(if {} {{ 1 }} else {{ 0 }})", rust_op)
            }
            AstNode::TernaryOp { condition, true_expr, false_expr } => {
                // Generate short-circuiting conditional expression
                match condition.as_ref() {
                    AstNode::ComparisonOp { op, left, right } => {
                        let left_expr = self.generate_value_expression(left);
                        let right_expr = self.generate_value_expression(right);
                        let condition_code = match op {
                            CompType::GreaterEq => format!("{} >= {}", left_expr, right_expr),
                            CompType::Greater => format!("{} > {}", left_expr, right_expr),
                            CompType::LessEq => format!("{} <= {}", left_expr, right_expr),
                            CompType::Less => format!("{} < {}", left_expr, right_expr),
                            CompType::Equal => format!("{} == {}", left_expr, right_expr),
                            CompType::NotEqual => format!("{} != {}", left_expr, right_expr),
                        };
                        
                        let true_branch = self.generate_ast_expression(true_expr);
                        let false_branch = self.generate_ast_expression(false_expr);
                        
                        format!(
                            "if {} {{ {} }} else {{ {} }}",
                            condition_code, true_branch, false_branch
                        )
                    }
                    _ => {
                        // For non-comparison conditions, convert to boolean first
                        let condition_expr = self.generate_value_expression(condition);
                        let true_branch = self.generate_ast_expression(true_expr);
                        let false_branch = self.generate_ast_expression(false_expr);
                        
                        format!(
                            "if {} != 0.0 {{ {} }} else {{ {} }}",
                            condition_expr, true_branch, false_branch
                        )
                    }
                }
            }
            AstNode::FunctionCall { func, arg } => {
                let arg_expr = self.generate_value_expression(arg);
                let rust_func = match func {
                    FuncType::Int => format!("{}.trunc()", arg_expr),
                    FuncType::Exp => format!("{}.exp()", arg_expr),
                    FuncType::Log => format!("{}.ln()", arg_expr),
                };
                format!("TagValue::F64({})", rust_func)
            }
            AstNode::ExifToolFunction { name, arg } => {
                // Generate ExifTool function call with conv_registry lookup
                self.generate_exiftool_function_call(name, arg)
            }
            AstNode::Sprintf { format_string, args } => {
                // Convert Perl sprintf format to Rust format! syntax
                let rust_format = convert_perl_sprintf_to_rust(format_string);
                let arg_exprs: Vec<String> = args.iter()
                    .map(|arg| self.generate_value_expression(arg))
                    .collect();
                format!("TagValue::String(format!(\"{}\", {}))", 
                       rust_format, 
                       arg_exprs.join(", "))
            }
            AstNode::UnaryMinus { operand } => {
                let operand_expr = self.generate_value_expression(operand);
                format!("TagValue::F64(-{})", operand_expr)
            }
            AstNode::RegexSubstitution { target, pattern, replacement, flags } => {
                let target_expr = self.generate_ast_expression(target);
                self.generate_regex_substitution(&target_expr, pattern, replacement, flags)
            }
            AstNode::Transliteration { target, search_list, replace_list, flags } => {
                let target_expr = self.generate_ast_expression(target);
                self.generate_transliteration(&target_expr, search_list, replace_list, flags)
            }
        }
    }
    
    /// Generate a numeric value expression (not wrapped in TagValue)
    fn generate_value_expression(&self, node: &AstNode) -> String {
        match node {
            AstNode::Variable => "val".to_string(),
            AstNode::Number(n) => format_number(*n),
            AstNode::BinaryOp { op, left, right } => {
                let left_expr = self.generate_value_expression(left);
                let right_expr = self.generate_value_expression(right);
                match op {
                    OpType::Add => format!("({} + {})", left_expr, right_expr),
                    OpType::Subtract => format!("({} - {})", left_expr, right_expr),
                    OpType::Multiply => format!("({} * {})", left_expr, right_expr),
                    OpType::Divide => format!("({} / {})", left_expr, right_expr),
                    OpType::Power => format!("({}.powf({}))", left_expr, right_expr),
                    OpType::Concatenate => {
                        // Use concatenation chain to avoid nested format! calls
                        let temp_node = AstNode::BinaryOp { 
                            op: *op, 
                            left: left.clone(), 
                            right: right.clone() 
                        };
                        let chain = self.collect_concatenation_chain(&temp_node);
                        let (format_string, arguments) = self.generate_concatenation_format(&chain, true);
                        if arguments.is_empty() {
                            // All string literals
                            format!("\"{}\"", format_string)
                        } else {
                            format!("format!(\"{}\", {})", format_string, arguments.join(", "))
                        }
                    }
                    // Bitwise operations - convert to integers, operate, then back to f64
                    OpType::BitwiseAnd => format!("((({} as i64) & ({} as i64)) as f64)", left_expr, right_expr),
                    OpType::BitwiseOr => format!("((({} as i64) | ({} as i64)) as f64)", left_expr, right_expr),
                    OpType::LeftShift => format!("((({} as i64) << ({} as i64)) as f64)", left_expr, right_expr),
                    OpType::RightShift => format!("((({} as i64) >> ({} as i64)) as f64)", left_expr, right_expr),
                }
            }
            AstNode::FunctionCall { func, arg } => {
                let arg_expr = self.generate_value_expression(arg);
                match func {
                    FuncType::Int => format!("{}.trunc()", arg_expr),
                    FuncType::Exp => format!("{}.exp()", arg_expr),
                    FuncType::Log => format!("{}.ln()", arg_expr),
                }
            }
            AstNode::ExifToolFunction { .. } => {
                // ExifTool functions produce TagValue, not raw numeric values
                // In value context, we'll return 0.0 as fallback
                "0.0".to_string()
            }
            AstNode::String { value, has_interpolation } => {
                if *has_interpolation {
                    // For value expressions in arithmetic context, string interpolation doesn't make sense
                    // This is likely an error case, but we'll return 0.0 as fallback
                    "0.0".to_string()
                } else {
                    // Try to parse string as number, fallback to 0.0
                    if let Ok(num) = value.parse::<f64>() {
                        format_number(num)
                    } else {
                        "0.0".to_string()
                    }
                }
            }
            AstNode::ComparisonOp { op, left, right } => {
                let left_expr = self.generate_value_expression(left);
                let right_expr = self.generate_value_expression(right);
                let comparison = match op {
                    CompType::GreaterEq => format!("{} >= {}", left_expr, right_expr),
                    CompType::Greater => format!("{} > {}", left_expr, right_expr),
                    CompType::LessEq => format!("{} <= {}", left_expr, right_expr),
                    CompType::Less => format!("{} < {}", left_expr, right_expr),
                    CompType::Equal => format!("{} == {}", left_expr, right_expr),
                    CompType::NotEqual => format!("{} != {}", left_expr, right_expr),
                };
                format!("if {} {{ 1.0 }} else {{ 0.0 }}", comparison)
            }
            AstNode::TernaryOp { condition, true_expr, false_expr } => {
                // For value context, we need to ensure both branches return numeric values
                let condition_expr = match condition.as_ref() {
                    AstNode::ComparisonOp { op, left, right } => {
                        let left_expr = self.generate_value_expression(left);
                        let right_expr = self.generate_value_expression(right);
                        match op {
                            CompType::GreaterEq => format!("{} >= {}", left_expr, right_expr),
                            CompType::Greater => format!("{} > {}", left_expr, right_expr),
                            CompType::LessEq => format!("{} <= {}", left_expr, right_expr),
                            CompType::Less => format!("{} < {}", left_expr, right_expr),
                            CompType::Equal => format!("{} == {}", left_expr, right_expr),
                            CompType::NotEqual => format!("{} != {}", left_expr, right_expr),
                        }
                    }
                    _ => {
                        let cond_val = self.generate_value_expression(condition); 
                        format!("{} != 0.0", cond_val)
                    }
                };
                
                let true_val = self.generate_value_expression(true_expr);
                let false_val = self.generate_value_expression(false_expr);
                
                format!(
                    "if {} {{ {} }} else {{ {} }}",
                    condition_expr, true_val, false_val
                )
            }
            AstNode::Undefined => "0.0".to_string(), // undef in numeric context is 0
            AstNode::Sprintf { .. } => "0.0".to_string(), // sprintf produces strings, not numbers
            AstNode::UnaryMinus { operand } => {
                let operand_expr = self.generate_value_expression(operand);
                format!("(-{})", operand_expr)
            }
            AstNode::RegexSubstitution { .. } => {
                // RegexSubstitution produces strings, not numeric values
                // This shouldn't be called for string-producing operations
                "0.0".to_string()
            }
            AstNode::Transliteration { .. } => {
                // Transliteration produces strings, not numeric values
                // This shouldn't be called for string-producing operations
                "0.0".to_string()
            }
        }
    }
    
    /// Generate ExifTool function call with conv_registry lookup
    fn generate_exiftool_function_call(&self, name: &str, arg: &AstNode) -> String {
        let arg_expr = self.generate_value_expression(arg);
        
        // Try to look up the function in conv_registry
        // For now, we'll generate a lookup call - this will be enhanced with actual conv_registry integration
        let function_expr = format!("{}($val)", name);
        
        // Check if this is a known function pattern in conv_registry
        match name {
            "Image::ExifTool::Exif::PrintExposureTime" => {
                format!("crate::implementations::print_conv::exposuretime_print_conv(&TagValue::F64({}))", arg_expr)
            }
            "Image::ExifTool::Exif::PrintFNumber" => {
                format!("crate::implementations::print_conv::fnumber_print_conv(&TagValue::F64({}))", arg_expr)
            }
            "Image::ExifTool::Exif::PrintFraction" => {
                format!("crate::implementations::print_conv::print_fraction(&TagValue::F64({}))", arg_expr)
            }
            _ => {
                // For unknown functions, generate a fallback call to missing_print_conv
                // This maintains the --show-missing functionality
                format!(
                    "crate::implementations::missing::missing_print_conv(\
                        0, \"{}\", \"Expression\", \"{}\", &TagValue::F64({}))",
                    name, function_expr, arg_expr
                )
            }
        }
    }
    
    /// Generate regex substitution code
    fn generate_regex_substitution(&self, target_expr: &str, pattern: &str, replacement: &str, flags: &str) -> String {
        // For now, generate basic regex substitution using the regex crate
        // This is a simplified implementation - full Perl regex compatibility would need more work
        let case_insensitive = flags.contains('i');
        let global = flags.contains('g');
        
        if global {
            format!(
                "{{
                    use regex::Regex;
                    let re = Regex::new({}).unwrap();
                    let target_str = match {} {{
                        TagValue::String(s) => s,
                        TagValue::F64(f) => f.to_string(),
                        _ => String::new(),
                    }};
                    TagValue::String(re.replace_all(&target_str, \"{}\").to_string())
                }}",
                if case_insensitive {
                    format!("r\"(?i){}\"", escape_regex_pattern(pattern))
                } else {
                    format!("r\"{}\"", escape_regex_pattern(pattern))
                },
                target_expr,
                escape_replacement_string(replacement)
            )
        } else {
            format!(
                "{{
                    use regex::Regex;
                    let re = Regex::new({}).unwrap();
                    let target_str = match {} {{
                        TagValue::String(s) => s,
                        TagValue::F64(f) => f.to_string(),
                        _ => String::new(),
                    }};
                    TagValue::String(re.replace(&target_str, \"{}\").to_string())
                }}",
                if case_insensitive {
                    format!("r\"(?i){}\"", escape_regex_pattern(pattern))
                } else {
                    format!("r\"{}\"", escape_regex_pattern(pattern))
                },
                target_expr,
                escape_replacement_string(replacement)
            )
        }
    }
    
    /// Collect a chain of concatenations into a flat list
    /// This converts nested BinaryOp(Concatenate) into a single list to avoid nested format! calls
    fn collect_concatenation_chain<'a>(&self, node: &'a AstNode) -> Vec<&'a AstNode> {
        match node {
            AstNode::BinaryOp { op: OpType::Concatenate, left, right } => {
                let mut chain = self.collect_concatenation_chain(left);
                chain.extend(self.collect_concatenation_chain(right));
                chain
            }
            _ => vec![node]
        }
    }
    
    /// Generate a format string and arguments for concatenation chain
    /// Returns (format_string, arguments)
    fn generate_concatenation_format<'a>(&self, chain: &[&'a AstNode], for_value_context: bool) -> (String, Vec<String>) {
        let mut format_string = String::new();
        let mut arguments = Vec::new();
        
        for node in chain {
            match node {
                AstNode::String { value, has_interpolation } => {
                    if *has_interpolation {
                        // Variable interpolation: "$val mm" becomes "{} mm" with val argument
                        let formatted_value = value.replace("$val", "{}");
                        format_string.push_str(&formatted_value);
                        arguments.push("val".to_string());
                    } else {
                        // String literal: just append to format string
                        format_string.push_str(value);
                    }
                }
                _ => {
                    // Non-string expression: add {} placeholder and generate argument
                    format_string.push_str("{}");
                    if for_value_context {
                        arguments.push(self.generate_value_expression(node));
                    } else {
                        // For non-value context, we need to extract the actual value from TagValue
                        let expr = self.generate_value_expression(node);
                        arguments.push(expr);
                    }
                }
            }
        }
        
        (format_string, arguments)
    }
    
    /// Generate transliteration code
    fn generate_transliteration(&self, target_expr: &str, search_list: &str, replace_list: &str, flags: &str) -> String {
        let complement = flags.contains('c');
        let delete = flags.contains('d');
        
        if delete && complement {
            // tr/a-fA-F0-9//dc - delete all characters NOT in the search list
            format!(
                "{{
                    let target_str = match {} {{
                        TagValue::String(s) => s,
                        TagValue::F64(f) => f.to_string(),
                        _ => String::new(),
                    }};
                    let keep_chars: std::collections::HashSet<char> = \"{}\".chars().collect();
                    let result: String = target_str.chars()
                        .filter(|c| keep_chars.contains(c))
                        .collect();
                    TagValue::String(result)
                }}",
                target_expr,
                escape_char_class(search_list)
            )
        } else if delete {
            // tr/chars//d - delete specified characters
            format!(
                "{{
                    let target_str = match {} {{
                        TagValue::String(s) => s,
                        TagValue::F64(f) => f.to_string(),
                        _ => String::new(),
                    }};
                    let delete_chars: std::collections::HashSet<char> = \"{}\".chars().collect();
                    let result: String = target_str.chars()
                        .filter(|c| !delete_chars.contains(c))
                        .collect();
                    TagValue::String(result)
                }}",
                target_expr,
                escape_char_class(search_list)
            )
        } else {
            // Basic character replacement
            format!(
                "{{
                    let target_str = match {} {{
                        TagValue::String(s) => s,
                        TagValue::F64(f) => f.to_string(),
                        _ => String::new(),
                    }};
                    let search_chars: Vec<char> = \"{}\".chars().collect();
                    let replace_chars: Vec<char> = \"{}\".chars().collect();
                    let result: String = target_str.chars()
                        .map(|c| {{
                            if let Some(pos) = search_chars.iter().position(|&sc| sc == c) {{
                                replace_chars.get(pos).copied().unwrap_or(c)
                            }} else {{
                                c
                            }}
                        }})
                        .collect();
                    TagValue::String(result)
                }}",
                target_expr,
                escape_char_class(search_list),
                escape_char_class(replace_list)
            )
        }
    }
}

/// Format number as floating-point literal to ensure proper f64 arithmetic
fn format_number(n: Number) -> String {
    if n.fract() == 0.0 {
        format!("{:.1}_f64", n) // Add _f64 to integers like 8 -> 8.0_f64
    } else {
        format!("{}_f64", n) // Add _f64 to decimals like 25.4 -> 25.4_f64
    }
}

/// Convert Perl sprintf format string to Rust format! syntax
/// Examples: "%.1f mm" -> "{:.1} mm", "%d" -> "{}", "%.2f" -> "{:.2}"
fn convert_perl_sprintf_to_rust(perl_format: &str) -> String {
    let mut result = String::new();
    let mut chars = perl_format.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '%' {
            // Handle format specifiers
            let mut format_spec = String::new();
            
            // Skip optional flags like '+', '-', ' ', '#', '0'
            while let Some(&next_ch) = chars.peek() {
                if matches!(next_ch, '+' | '-' | ' ' | '#' | '0') {
                    chars.next();
                } else {
                    break;
                }
            }
            
            // Parse width (optional)
            while let Some(&next_ch) = chars.peek() {
                if next_ch.is_ascii_digit() {
                    chars.next();
                } else {
                    break;
                }
            }
            
            // Parse precision (optional)
            if chars.peek() == Some(&'.') {
                chars.next(); // consume '.'
                format_spec.push(':');
                format_spec.push('.');
                while let Some(&next_ch) = chars.peek() {
                    if next_ch.is_ascii_digit() {
                        format_spec.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
            }
            
            // Parse conversion specifier
            if let Some(spec_char) = chars.next() {
                match spec_char {
                    'f' => {
                        // Floating point: %.1f -> {:.1}
                        result.push('{');
                        result.push_str(&format_spec);
                        result.push('}');
                    }
                    'd' => {
                        // Integer: %d -> {}
                        result.push_str("{}");
                    }
                    'x' => {
                        // Hexadecimal: %x -> {:x}
                        result.push('{');
                        if !format_spec.is_empty() {
                            result.push_str(&format_spec);
                        } else {
                            result.push(':');
                        }
                        result.push('x');
                        result.push('}');
                    }
                    's' => {
                        // String: %s -> {}
                        result.push_str("{}");
                    }
                    _ => {
                        // Unsupported format specifier, pass through
                        result.push('%');
                        result.push(spec_char);
                    }
                }
            }
        } else {
            result.push(ch);
        }
    }
    
    result
}

/// Escape regex pattern for use in Rust regex::Regex
fn escape_regex_pattern(pattern: &str) -> String {
    // Basic escaping - in full implementation would need comprehensive Perl->Rust regex translation
    pattern.replace("\\", "\\\\").replace("\"", "\\\"")
}

/// Escape replacement string for use in regex replacement
fn escape_replacement_string(replacement: &str) -> String {
    replacement.replace("\\", "\\\\").replace("\"", "\\\"")
}

/// Escape character class for use in transliteration
fn escape_char_class(chars: &str) -> String {
    // Handle character ranges like a-z, A-Z, 0-9
    // For now, just basic escaping - full implementation would expand ranges
    chars.replace("\\", "\\\\").replace("\"", "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_function_codegen() {
        let expr = CompiledExpression {
            original_expr: "int($val)".to_string(),
            ast: Box::new(AstNode::FunctionCall {
                func: FuncType::Int,
                arg: Box::new(AstNode::Variable)
            })
        };
        
        let code = expr.generate_rust_code();
        assert!(code.contains("val.trunc()"));
        assert!(code.contains("match value.as_f64()"));
    }

    #[test]
    fn test_simple_arithmetic_codegen() {
        let expr = CompiledExpression {
            original_expr: "$val / 8".to_string(),
            ast: Box::new(AstNode::BinaryOp {
                op: OpType::Divide,
                left: Box::new(AstNode::Variable),
                right: Box::new(AstNode::Number(8.0))
            })
        };
        
        let code = expr.generate_rust_code();
        assert!(code.contains("(val / 8.0_f64)"));
        assert!(code.contains("match value.as_f64()"));
    }

    #[test]
    fn test_complex_expression_codegen() {
        let expr = CompiledExpression {
            original_expr: "($val - 104) / 8".to_string(),
            ast: Box::new(AstNode::BinaryOp {
                op: OpType::Divide,
                left: Box::new(AstNode::BinaryOp {
                    op: OpType::Subtract,
                    left: Box::new(AstNode::Variable),
                    right: Box::new(AstNode::Number(104.0))
                }),
                right: Box::new(AstNode::Number(8.0))
            })
        };
        
        let code = expr.generate_rust_code();
        assert!(code.contains("((val - 104.0_f64) / 8.0_f64)"));
        assert!(code.contains("match value.as_f64()"));
    }

    #[test]
    fn test_function_in_complex_expression() {
        let expr = CompiledExpression {
            original_expr: "int($val * 2)".to_string(),
            ast: Box::new(AstNode::FunctionCall {
                func: FuncType::Int,
                arg: Box::new(AstNode::BinaryOp {
                    op: OpType::Multiply,
                    left: Box::new(AstNode::Variable),
                    right: Box::new(AstNode::Number(2.0))
                })
            })
        };
        
        let code = expr.generate_rust_code();
        assert!(code.contains("(val * 2.0_f64).trunc()"));
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(8.0), "8.0_f64");
        assert_eq!(format_number(25.4), "25.4_f64");
        assert_eq!(format_number(0.0), "0.0_f64");
        assert_eq!(format_number(1000.0), "1000.0_f64");
    }
    
    #[test]
    fn test_comparison_codegen() {
        let expr = CompiledExpression {
            original_expr: "$val >= 0".to_string(),
            ast: Box::new(AstNode::ComparisonOp {
                op: CompType::GreaterEq,
                left: Box::new(AstNode::Variable),
                right: Box::new(AstNode::Number(0.0))
            })
        };
        
        let code = expr.generate_rust_code();
        assert!(code.contains("val >= 0.0"));
        assert!(code.contains("TagValue::U8(if"));
        assert!(code.contains("{ 1 } else { 0 }"));
    }
    
    #[test]
    fn test_simple_ternary_codegen() {
        let expr = CompiledExpression {
            original_expr: "$val >= 0 ? $val : undef".to_string(),
            ast: Box::new(AstNode::TernaryOp {
                condition: Box::new(AstNode::ComparisonOp {
                    op: CompType::GreaterEq,
                    left: Box::new(AstNode::Variable),
                    right: Box::new(AstNode::Number(0.0))
                }),
                true_expr: Box::new(AstNode::Variable),
                false_expr: Box::new(AstNode::Undefined)
            })
        };
        
        let code = expr.generate_rust_code();
        assert!(code.contains("if val >= 0.0"));
        assert!(code.contains("TagValue::F64(val)"));
        assert!(code.contains("value.clone()"));
    }
    
    #[test]
    fn test_string_ternary_codegen() {
        let expr = CompiledExpression {
            original_expr: r#"$val > 655.345 ? "inf" : "$val m""#.to_string(),
            ast: Box::new(AstNode::TernaryOp {
                condition: Box::new(AstNode::ComparisonOp {
                    op: CompType::Greater,
                    left: Box::new(AstNode::Variable),
                    right: Box::new(AstNode::Number(655.345))
                }),
                true_expr: Box::new(AstNode::String { 
                    value: "inf".to_string(), 
                    has_interpolation: false 
                }),
                false_expr: Box::new(AstNode::String { 
                    value: "$val m".to_string(), 
                    has_interpolation: true 
                })
            })
        };
        
        let code = expr.generate_rust_code();
        assert!(code.contains("if val > 655.345"));
        assert!(code.contains(r#"TagValue::String("inf".to_string())"#));
        assert!(code.contains("format!(\"{} m\", val)"));
    }
    
    #[test]
    fn test_function_with_ternary_codegen() {
        let expr = CompiledExpression {
            original_expr: "int($val >= 0 ? $val : 0)".to_string(),
            ast: Box::new(AstNode::FunctionCall {
                func: FuncType::Int,
                arg: Box::new(AstNode::TernaryOp {
                    condition: Box::new(AstNode::ComparisonOp {
                        op: CompType::GreaterEq,
                        left: Box::new(AstNode::Variable),
                        right: Box::new(AstNode::Number(0.0))
                    }),
                    true_expr: Box::new(AstNode::Variable),
                    false_expr: Box::new(AstNode::Number(0.0))
                })
            })
        };
        
        let code = expr.generate_rust_code();
        assert!(code.contains("if val >= 0.0_f64 { val } else { 0.0_f64 }"));
        assert!(code.contains(".trunc()"));
    }
}