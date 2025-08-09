//! Tokenization logic for arithmetic expressions
//!
//! This module handles parsing expression strings into tokens that can
//! be processed by the shunting yard algorithm.

use super::types::*;
use std::iter::Peekable;
use std::str::Chars;

/// Tokenize an expression string into parse tokens
pub fn tokenize(expr: &str) -> Result<Vec<ParseToken>, String> {
    let mut tokens = Vec::new();
    let mut chars = expr.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            ' ' | '\t' => continue, // Skip whitespace

            '$' => {
                tokens.push(parse_variable(&mut chars)?);
            }

            '0'..='9' => {
                tokens.push(parse_number(ch, &mut chars)?);
            }

            'a'..='z' | 'A'..='Z' => {
                // Special case for regex operations
                if ch == 's' || ch == 't' {
                    if let Some(token) = try_parse_regex(ch, &mut chars)? {
                        tokens.push(token);
                    } else {
                        tokens.push(parse_identifier(ch, &mut chars)?);
                    }
                } else {
                    tokens.push(parse_identifier(ch, &mut chars)?);
                }
            }

            '"' => {
                tokens.push(parse_string_literal(&mut chars)?);
            }

            '+' => tokens.push(ParseToken::Operator(Operator::new(OpType::Add, 4, true))),
            '-' => {
                // Check if this is unary minus based on context
                let is_unary = match tokens.last() {
                    None => true,                            // Start of expression
                    Some(ParseToken::LeftParen) => true,     // After opening parenthesis
                    Some(ParseToken::Operator(_)) => true,   // After another operator
                    Some(ParseToken::Comparison(_)) => true, // After comparison operator
                    Some(ParseToken::Question) => true,      // After ? in ternary
                    Some(ParseToken::Colon) => true,         // After : in ternary
                    Some(ParseToken::Comma) => true,         // After comma in function call
                    _ => false, // After operand (number, variable, etc.)
                };

                if is_unary {
                    tokens.push(ParseToken::UnaryMinus);
                } else {
                    tokens.push(ParseToken::Operator(Operator::new(
                        OpType::Subtract,
                        4,
                        true,
                    )));
                }
            }
            '*' => {
                // Check for ** power operator
                if chars.peek() == Some(&'*') {
                    chars.next(); // consume second '*'
                    tokens.push(ParseToken::Operator(Operator::new(OpType::Power, 6, false)));
                // Right-associative, highest precedence
                } else {
                    tokens.push(ParseToken::Operator(Operator::new(
                        OpType::Multiply,
                        5,
                        true,
                    )));
                }
            }
            '/' => tokens.push(ParseToken::Operator(Operator::new(OpType::Divide, 5, true))),
            '.' => tokens.push(ParseToken::Operator(Operator::new(
                OpType::Concatenate,
                4,
                true,
            ))),

            // Comparison operators and bitwise shifts
            '>' => {
                if chars.peek() == Some(&'=') {
                    chars.next(); // consume '='
                    tokens.push(ParseToken::Comparison(CompOperator::new(
                        CompType::GreaterEq,
                        3,
                    )));
                } else if chars.peek() == Some(&'>') {
                    chars.next(); // consume second '>'
                    tokens.push(ParseToken::Operator(Operator::new(
                        OpType::RightShift,
                        3,
                        true,
                    ))); // Between arithmetic and comparison
                } else {
                    tokens.push(ParseToken::Comparison(CompOperator::new(
                        CompType::Greater,
                        3,
                    )));
                }
            }
            '<' => {
                if chars.peek() == Some(&'=') {
                    chars.next(); // consume '='
                    tokens.push(ParseToken::Comparison(CompOperator::new(
                        CompType::LessEq,
                        3,
                    )));
                } else if chars.peek() == Some(&'<') {
                    chars.next(); // consume second '<'
                    tokens.push(ParseToken::Operator(Operator::new(
                        OpType::LeftShift,
                        3,
                        true,
                    ))); // Between arithmetic and comparison
                } else {
                    tokens.push(ParseToken::Comparison(CompOperator::new(CompType::Less, 3)));
                }
            }
            '=' => {
                if chars.peek() == Some(&'=') {
                    chars.next(); // consume second '='
                    tokens.push(ParseToken::Comparison(CompOperator::new(
                        CompType::Equal,
                        3,
                    )));
                } else {
                    return Err("Single '=' not supported - use '==' for equality".to_string());
                }
            }
            '!' => {
                if chars.peek() == Some(&'=') {
                    chars.next(); // consume '='
                    tokens.push(ParseToken::Comparison(CompOperator::new(
                        CompType::NotEqual,
                        3,
                    )));
                } else {
                    return Err("Single '!' not supported - use '!=' for inequality".to_string());
                }
            }

            // Ternary operators
            '?' => tokens.push(ParseToken::Question),
            ':' => tokens.push(ParseToken::Colon),

            // Bitwise operators
            '&' => tokens.push(ParseToken::Operator(Operator::new(
                OpType::BitwiseAnd,
                2,
                true,
            ))), // Lower precedence than shifts
            '|' => tokens.push(ParseToken::Operator(Operator::new(
                OpType::BitwiseOr,
                1,
                true,
            ))), // Lowest precedence

            '(' => tokens.push(ParseToken::LeftParen),
            ')' => tokens.push(ParseToken::RightParen),
            ',' => tokens.push(ParseToken::Comma), // For parsing argument lists

            _ => return Err(format!("Unexpected character: '{ch}'")),
        }
    }

    Ok(tokens)
}

/// Parse $val variable token
fn parse_variable(chars: &mut Peekable<Chars>) -> Result<ParseToken, String> {
    // Expect "val" after $
    let val_chars: String = chars.by_ref().take(3).collect();
    if val_chars == "val" {
        // Check if this is indexed variable ($val[n])
        if chars.peek() == Some(&'[') {
            chars.next(); // consume '['

            // Parse the index number
            let mut index_str = String::new();
            while let Some(&next_ch) = chars.peek() {
                if next_ch.is_ascii_digit() {
                    index_str.push(chars.next().unwrap());
                } else {
                    break;
                }
            }

            if index_str.is_empty() {
                return Err("Expected digit after '$val['".to_string());
            }

            // Expect closing bracket
            if chars.next() != Some(']') {
                return Err("Expected ']' after index in $val[n]".to_string());
            }

            // Parse index as usize
            let index: usize = index_str
                .parse()
                .map_err(|_| format!("Invalid index in $val[{index_str}]"))?;

            Ok(ParseToken::ValIndex(index))
        } else {
            // Regular $val variable
            Ok(ParseToken::Variable)
        }
    } else {
        Err(format!("Expected 'val' after '$', found '{val_chars}'"))
    }
}

/// Parse numeric literal token (including hex numbers like 0xffff)
fn parse_number(first_digit: char, chars: &mut Peekable<Chars>) -> Result<ParseToken, String> {
    let mut number_str = String::new();
    number_str.push(first_digit);

    // Check for hex numbers (0x...)
    if first_digit == '0' && chars.peek() == Some(&'x') {
        number_str.push(chars.next().unwrap()); // consume 'x'

        // Parse hex digits
        while let Some(&next_ch) = chars.peek() {
            if next_ch.is_ascii_hexdigit() {
                number_str.push(chars.next().unwrap());
            } else {
                break;
            }
        }

        // Convert hex string to decimal
        let hex_part = &number_str[2..]; // Skip "0x"
        let number = i64::from_str_radix(hex_part, 16)
            .map_err(|_| format!("Invalid hex number: {number_str}"))? as f64;
        return Ok(ParseToken::Number(number));
    }

    // Parse regular decimal numbers
    while let Some(&next_ch) = chars.peek() {
        if next_ch.is_ascii_digit() || next_ch == '.' {
            number_str.push(chars.next().unwrap());
        } else {
            break;
        }
    }

    let number: f64 = number_str
        .parse()
        .map_err(|_| format!("Invalid number: {number_str}"))?;
    Ok(ParseToken::Number(number))
}

/// Parse string literal token
fn parse_string_literal(chars: &mut Peekable<Chars>) -> Result<ParseToken, String> {
    let mut string_value = String::new();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                // End of string
                return Ok(ParseToken::String(string_value));
            }
            '$' => {
                // Variable interpolation detected
                string_value.push(ch);
            }
            '\\' => {
                // Handle escape sequences
                if let Some(escaped) = chars.next() {
                    match escaped {
                        'n' => string_value.push('\n'),
                        't' => string_value.push('\t'),
                        'r' => string_value.push('\r'),
                        '\\' => string_value.push('\\'),
                        '"' => string_value.push('"'),
                        _ => {
                            string_value.push('\\');
                            string_value.push(escaped);
                        }
                    }
                } else {
                    return Err("Unexpected end of string after escape character".to_string());
                }
            }
            _ => string_value.push(ch),
        }
    }

    Err("Unterminated string literal".to_string())
}

/// Parse identifier (function name or undef keyword)
fn parse_identifier(first_char: char, chars: &mut Peekable<Chars>) -> Result<ParseToken, String> {
    let mut identifier = String::new();
    identifier.push(first_char);

    // Parse the full identifier including :: separators for ExifTool functions
    while let Some(&next_ch) = chars.peek() {
        if next_ch.is_ascii_alphabetic() || next_ch == ':' {
            identifier.push(chars.next().unwrap());
        } else {
            break;
        }
    }

    // Check if it's the undef keyword
    if identifier == "undef" {
        return Ok(ParseToken::Undefined);
    }

    // Check if it's a function (must be followed by opening parenthesis)
    if chars.peek() == Some(&'(') {
        match identifier.as_str() {
            "sprintf" => return Ok(ParseToken::Sprintf),
            "int" => return Ok(ParseToken::Function(FuncType::Int)),
            "exp" => return Ok(ParseToken::Function(FuncType::Exp)),
            "log" => return Ok(ParseToken::Function(FuncType::Log)),
            _ => {
                // Check if it's an ExifTool function pattern
                if identifier.starts_with("Image::ExifTool::") {
                    return Ok(ParseToken::ExifToolFunction(identifier));
                }
                return Err(format!("Unknown function: '{identifier}'"));
            }
        };
    }

    // If it's not undef or a function, it's an error for now
    Err(format!("Unknown identifier: '{identifier}'"))
}

/// Try to parse regex operations (s/// or tr///)
/// Returns Some(token) if it's a valid regex, None if it should be parsed as identifier
fn try_parse_regex(
    first_char: char,
    chars: &mut Peekable<Chars>,
) -> Result<Option<ParseToken>, String> {
    match first_char {
        's' => {
            // Look for s/pattern/replacement/flags
            if chars.peek() == Some(&'/') {
                chars.next(); // consume '/'
                let (pattern, replacement, flags) = parse_substitution_parts(chars)?;
                return Ok(Some(ParseToken::RegexSubstitution {
                    pattern,
                    replacement,
                    flags,
                }));
            }
        }
        't' => {
            // Look for tr/searchlist/replacelist/flags
            if chars.peek() == Some(&'r') {
                chars.next(); // consume 'r'
                if chars.peek() == Some(&'/') {
                    chars.next(); // consume '/'
                    let (search_list, replace_list, flags) = parse_transliteration_parts(chars)?;
                    return Ok(Some(ParseToken::Transliteration {
                        search_list,
                        replace_list,
                        flags,
                    }));
                }
                // If we consumed 'r' but it's not followed by '/', this is an error
                // for now, but we could implement a more sophisticated backtracking
                return Err("Expected '/' after 'tr'".to_string());
            }
        }
        _ => {}
    }

    // Not a regex pattern, let identifier parser handle it
    Ok(None)
}

/// Parse s/pattern/replacement/flags parts
fn parse_substitution_parts(
    chars: &mut Peekable<Chars>,
) -> Result<(String, String, String), String> {
    let pattern = parse_regex_part(chars, '/')?;
    let replacement = parse_regex_part(chars, '/')?;
    let flags = parse_regex_flags(chars);
    Ok((pattern, replacement, flags))
}

/// Parse tr/searchlist/replacelist/flags parts
fn parse_transliteration_parts(
    chars: &mut Peekable<Chars>,
) -> Result<(String, String, String), String> {
    let search_list = parse_regex_part(chars, '/')?;
    let replace_list = parse_regex_part(chars, '/')?;
    let flags = parse_regex_flags(chars);
    Ok((search_list, replace_list, flags))
}

/// Parse a regex part up to delimiter
fn parse_regex_part(chars: &mut Peekable<Chars>, delimiter: char) -> Result<String, String> {
    let mut part = String::new();
    let mut escaped = false;

    while let Some(&ch) = chars.peek() {
        if escaped {
            part.push('\\');
            part.push(ch);
            chars.next();
            escaped = false;
        } else if ch == '\\' {
            chars.next();
            escaped = true;
        } else if ch == delimiter {
            chars.next(); // consume delimiter
            break;
        } else {
            part.push(ch);
            chars.next();
        }
    }

    Ok(part)
}

/// Parse regex flags after the final /
fn parse_regex_flags(chars: &mut Peekable<Chars>) -> String {
    let mut flags = String::new();

    while let Some(&ch) = chars.peek() {
        if ch.is_ascii_alphabetic() {
            flags.push(ch);
            chars.next();
        } else {
            break;
        }
    }

    flags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple() {
        let tokens = tokenize("$val / 8").unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], ParseToken::Variable));
        assert!(matches!(tokens[1], ParseToken::Operator(_)));
        assert!(matches!(tokens[2], ParseToken::Number(8.0)));
    }

    #[test]
    fn test_tokenize_function() {
        let tokens = tokenize("int($val)").unwrap();
        assert_eq!(tokens.len(), 4);
        assert!(matches!(tokens[0], ParseToken::Function(FuncType::Int)));
        assert!(matches!(tokens[1], ParseToken::LeftParen));
        assert!(matches!(tokens[2], ParseToken::Variable));
        assert!(matches!(tokens[3], ParseToken::RightParen));
    }

    #[test]
    fn test_tokenize_complex() {
        let tokens = tokenize("exp($val/32*log(2))*100").unwrap();
        assert!(tokens.len() > 10); // Should have many tokens
        assert!(matches!(tokens[0], ParseToken::Function(FuncType::Exp)));
    }

    #[test]
    fn test_invalid_function() {
        let result = tokenize("unknown($val)");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown function"));
    }

    #[test]
    fn test_invalid_variable() {
        let result = tokenize("$foo");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Expected 'val'"));
    }

    #[test]
    fn test_comparison_operators() {
        let tokens = tokenize("$val >= 0").unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], ParseToken::Variable));
        assert!(matches!(tokens[1], ParseToken::Comparison(_)));
        assert!(matches!(tokens[2], ParseToken::Number(0.0)));

        if let ParseToken::Comparison(comp_op) = &tokens[1] {
            assert_eq!(comp_op.comp_type, CompType::GreaterEq);
        }
    }

    #[test]
    fn test_ternary_operators() {
        let tokens = tokenize("$val ? 1 : 0").unwrap();
        assert_eq!(tokens.len(), 5);
        assert!(matches!(tokens[0], ParseToken::Variable));
        assert!(matches!(tokens[1], ParseToken::Question));
        assert!(matches!(tokens[2], ParseToken::Number(1.0)));
        assert!(matches!(tokens[3], ParseToken::Colon));
        assert!(matches!(tokens[4], ParseToken::Number(0.0)));
    }

    #[test]
    fn test_string_literals() {
        let tokens = tokenize("\"hello world\"").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], ParseToken::String(_)));

        if let ParseToken::String(s) = &tokens[0] {
            assert_eq!(s, "hello world");
        }
    }

    #[test]
    fn test_string_with_interpolation() {
        let tokens = tokenize("\"$val m\"").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], ParseToken::String(_)));

        if let ParseToken::String(s) = &tokens[0] {
            assert_eq!(s, "$val m");
        }
    }

    #[test]
    fn test_undef_keyword() {
        let tokens = tokenize("undef").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], ParseToken::Undefined));
    }

    #[test]
    fn test_ternary_expression() {
        let tokens = tokenize("$val >= 0 ? $val : undef").unwrap();
        assert_eq!(tokens.len(), 7);
        assert!(matches!(tokens[0], ParseToken::Variable));
        assert!(matches!(tokens[1], ParseToken::Comparison(_)));
        assert!(matches!(tokens[2], ParseToken::Number(0.0)));
        assert!(matches!(tokens[3], ParseToken::Question));
        assert!(matches!(tokens[4], ParseToken::Variable));
        assert!(matches!(tokens[5], ParseToken::Colon));
        assert!(matches!(tokens[6], ParseToken::Undefined));
    }

    #[test]
    fn test_sprintf_expression() {
        let tokens = tokenize("sprintf(\"%.1f mm\", $val)").unwrap();
        assert_eq!(tokens.len(), 6);
        assert!(matches!(tokens[0], ParseToken::Sprintf));
        assert!(matches!(tokens[1], ParseToken::LeftParen));
        assert!(matches!(tokens[2], ParseToken::String(_)));
        assert!(matches!(tokens[3], ParseToken::Comma));
        assert!(matches!(tokens[4], ParseToken::Variable));
        assert!(matches!(tokens[5], ParseToken::RightParen));

        if let ParseToken::String(s) = &tokens[2] {
            assert_eq!(s, "%.1f mm");
        }
    }

    #[test]
    fn test_exiftool_function_expression() {
        let tokens = tokenize("Image::ExifTool::Exif::PrintExposureTime($val)").unwrap();
        assert_eq!(tokens.len(), 4);
        assert!(matches!(tokens[0], ParseToken::ExifToolFunction(_)));
        assert!(matches!(tokens[1], ParseToken::LeftParen));
        assert!(matches!(tokens[2], ParseToken::Variable));
        assert!(matches!(tokens[3], ParseToken::RightParen));

        if let ParseToken::ExifToolFunction(func_name) = &tokens[0] {
            assert_eq!(func_name, "Image::ExifTool::Exif::PrintExposureTime");
        }
    }

    #[test]
    fn test_various_exiftool_functions() {
        let test_cases = vec![
            "Image::ExifTool::Exif::PrintFNumber($val)",
            "Image::ExifTool::GPS::ToDegrees($val)",
            "Image::ExifTool::Canon::LensType($val)",
        ];

        for expr in test_cases {
            let tokens = tokenize(expr).unwrap();
            assert_eq!(tokens.len(), 4);
            assert!(matches!(tokens[0], ParseToken::ExifToolFunction(_)));

            if let ParseToken::ExifToolFunction(func_name) = &tokens[0] {
                assert!(func_name.starts_with("Image::ExifTool::"));
            }
        }
    }
}
