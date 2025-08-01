//! Tokenization logic for arithmetic expressions
//!
//! This module handles parsing expression strings into tokens that can
//! be processed by the shunting yard algorithm.

use super::types::*;
use std::str::Chars;
use std::iter::Peekable;

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
                tokens.push(parse_identifier(ch, &mut chars)?);
            }
            
            '"' => {
                tokens.push(parse_string_literal(&mut chars)?);
            }
            
            '+' => tokens.push(ParseToken::Operator(Operator::new(OpType::Add, 1, true))),
            '-' => tokens.push(ParseToken::Operator(Operator::new(OpType::Subtract, 1, true))),
            '*' => tokens.push(ParseToken::Operator(Operator::new(OpType::Multiply, 2, true))),
            '/' => tokens.push(ParseToken::Operator(Operator::new(OpType::Divide, 2, true))),
            '.' => tokens.push(ParseToken::Operator(Operator::new(OpType::Concatenate, 1, true))),
            
            // Comparison operators
            '>' => {
                if chars.peek() == Some(&'=') {
                    chars.next(); // consume '='
                    tokens.push(ParseToken::Comparison(CompOperator::new(CompType::GreaterEq, 3)));
                } else {
                    tokens.push(ParseToken::Comparison(CompOperator::new(CompType::Greater, 3)));
                }
            }
            '<' => {
                if chars.peek() == Some(&'=') {
                    chars.next(); // consume '='
                    tokens.push(ParseToken::Comparison(CompOperator::new(CompType::LessEq, 3)));
                } else {
                    tokens.push(ParseToken::Comparison(CompOperator::new(CompType::Less, 3)));
                }
            }
            '=' => {
                if chars.peek() == Some(&'=') {
                    chars.next(); // consume second '='
                    tokens.push(ParseToken::Comparison(CompOperator::new(CompType::Equal, 3)));
                } else {
                    return Err("Single '=' not supported - use '==' for equality".to_string());
                }
            }
            '!' => {
                if chars.peek() == Some(&'=') {
                    chars.next(); // consume '='
                    tokens.push(ParseToken::Comparison(CompOperator::new(CompType::NotEqual, 3)));
                } else {
                    return Err("Single '!' not supported - use '!=' for inequality".to_string());
                }
            }
            
            // Ternary operators
            '?' => tokens.push(ParseToken::Question),
            ':' => tokens.push(ParseToken::Colon),
            
            '(' => tokens.push(ParseToken::LeftParen),
            ')' => tokens.push(ParseToken::RightParen),
            ',' => tokens.push(ParseToken::Comma), // For parsing argument lists
            
            _ => return Err(format!("Unexpected character: '{}'", ch)),
        }
    }
    
    Ok(tokens)
}

/// Parse $val variable token
fn parse_variable(chars: &mut Peekable<Chars>) -> Result<ParseToken, String> {
    // Expect "val" after $
    let val_chars: String = chars.by_ref().take(3).collect();
    if val_chars == "val" {
        Ok(ParseToken::Variable)
    } else {
        Err(format!("Expected 'val' after '$', found '{}'", val_chars))
    }
}

/// Parse numeric literal token
fn parse_number(first_digit: char, chars: &mut Peekable<Chars>) -> Result<ParseToken, String> {
    let mut number_str = String::new();
    number_str.push(first_digit);
    
    while let Some(&next_ch) = chars.peek() {
        if next_ch.is_ascii_digit() || next_ch == '.' {
            number_str.push(chars.next().unwrap());
        } else {
            break;
        }
    }
    
    let number: f64 = number_str.parse()
        .map_err(|_| format!("Invalid number: {}", number_str))?;
    Ok(ParseToken::Number(number))
}

/// Parse string literal token
fn parse_string_literal(chars: &mut Peekable<Chars>) -> Result<ParseToken, String> {
    let mut string_value = String::new();
    let mut has_interpolation = false;
    
    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                // End of string
                return Ok(ParseToken::String(string_value));
            }
            '$' => {
                // Variable interpolation detected
                has_interpolation = true;
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
    
    while let Some(&next_ch) = chars.peek() {
        if next_ch.is_ascii_alphabetic() {
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
            _ => return Err(format!("Unknown function: '{}'", identifier)),
        };
    }
    
    // If it's not undef or a function, it's an error for now
    Err(format!("Unknown identifier: '{}'", identifier))
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
}