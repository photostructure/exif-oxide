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
                tokens.push(parse_function(ch, &mut chars)?);
            }
            
            '+' => tokens.push(ParseToken::Operator(Operator::new(OpType::Add, 1, true))),
            '-' => tokens.push(ParseToken::Operator(Operator::new(OpType::Subtract, 1, true))),
            '*' => tokens.push(ParseToken::Operator(Operator::new(OpType::Multiply, 2, true))),
            '/' => tokens.push(ParseToken::Operator(Operator::new(OpType::Divide, 2, true))),
            '(' => tokens.push(ParseToken::LeftParen),
            ')' => tokens.push(ParseToken::RightParen),
            
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

/// Parse function name token
fn parse_function(first_char: char, chars: &mut Peekable<Chars>) -> Result<ParseToken, String> {
    let mut func_name = String::new();
    func_name.push(first_char);
    
    while let Some(&next_ch) = chars.peek() {
        if next_ch.is_ascii_alphabetic() {
            func_name.push(chars.next().unwrap());
        } else {
            break;
        }
    }
    
    // Expect opening parenthesis after function name
    if chars.peek() != Some(&'(') {
        return Err(format!("Expected '(' after function name '{}'", func_name));
    }
    
    let func_type = match func_name.as_str() {
        "int" => FuncType::Int,
        "exp" => FuncType::Exp,
        "log" => FuncType::Log,
        _ => return Err(format!("Unknown function: '{}'", func_name)),
    };
    
    Ok(ParseToken::Function(func_type))
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
}