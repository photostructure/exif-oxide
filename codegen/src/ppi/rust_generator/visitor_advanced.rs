//! Helper functions for advanced token processing
//!
//! This module contains standalone helper functions for processing advanced PPI tokens
//! that require specialized logic but don't need recursive traversal.

use crate::ppi::rust_generator::errors::CodeGenError;
use crate::ppi::types::*;

/// Process magic variable nodes (like $_, $@, $!)
pub fn process_magic(node: &PpiNode) -> Result<String, CodeGenError> {
    let content = node
        .content
        .as_ref()
        .ok_or(CodeGenError::MissingContent("magic variable".to_string()))?;

    match content.as_str() {
        "$_" => {
            // $_ is the default variable - in our context it's the current value being processed
            // Example: $_=$val,s/(\d+)(\d{4})/$1-$2/,$_
            // In ExifTool expressions, $_ typically refers to val
            Ok("val".to_string())
        }
        "$@" => {
            // $@ is the error variable in Perl
            Ok("error_val".to_string())
        }
        "$!" => {
            // $! is the system error
            Ok("sys_error".to_string())
        }
        "$?" => {
            // $? is the exit status
            Ok("exit_status".to_string())
        }
        _ => {
            // Other magic variables - generate a placeholder
            Ok(format!("magic_var_{}", content.trim_start_matches('$')))
        }
    }
}

/// Process break statement keywords (return, last, next) - needs expression type context
pub fn process_break_keyword(
    keyword: &str,
    value: &str,
    expression_type: &ExpressionType,
) -> Result<String, CodeGenError> {
    match keyword {
        "return" => {
            // return $val => return val
            if value.is_empty() {
                Ok("return".to_string())
            } else {
                // Wrap in appropriate type based on expression type
                match expression_type {
                    ExpressionType::ValueConv => Ok(format!("return Ok({})", value)),
                    ExpressionType::PrintConv => Ok(format!("return {}", value)),
                    ExpressionType::Condition => Ok(format!("return {}", value)),
                }
            }
        }
        "last" => {
            // Perl's "last" is Rust's "break"
            Ok("break".to_string())
        }
        "next" => {
            // Perl's "next" is Rust's "continue"
            Ok("continue".to_string())
        }
        _ => Err(CodeGenError::UnsupportedStructure(format!(
            "Unknown break keyword: {}",
            keyword
        ))),
    }
}

/// Process transliterate operations (tr/// character replacement)
pub fn process_transliterate(node: &PpiNode) -> Result<String, CodeGenError> {
    let content = node.content.as_ref().ok_or(CodeGenError::MissingContent(
        "transliterate pattern".to_string(),
    ))?;

    // Parse tr/pattern/replacement/flags or tr#pattern#replacement#flags
    if !content.starts_with("tr/") && !content.starts_with("tr#") {
        return Err(CodeGenError::UnsupportedStructure(format!(
            "Invalid transliterate pattern: {}",
            content
        )));
    }

    // Determine delimiter
    let delimiter = if content.starts_with("tr/") { '/' } else { '#' };
    let parts: Vec<&str> = content[3..].split(delimiter).collect();

    if parts.len() < 2 {
        return Err(CodeGenError::UnsupportedStructure(format!(
            "Invalid transliterate format: {}",
            content
        )));
    }

    let search_chars = parts[0];
    let replace_chars = if parts.len() > 1 { parts[1] } else { "" };
    let flags = if parts.len() > 2 { parts[2] } else { "" };

    // Check for delete flag (d) and complement flag (c)
    let is_delete = flags.contains('d');
    let is_complement = flags.contains('c');

    if is_delete && !is_complement {
        // tr/chars//d - delete specified characters
        // Example: tr/()K//d removes parentheses and K
        let chars_to_remove: Vec<String> =
            search_chars.chars().map(|c| format!("'{}'", c)).collect();
        Ok(format!(
            "val.to_string().chars().filter(|c| ![{}].contains(c)).collect::<String>()",
            chars_to_remove.join(", ")
        ))
    } else if is_delete && is_complement {
        // tr/chars//dc - delete all EXCEPT specified characters
        // Example: tr/a-fA-F0-9//dc keeps only hex digits
        if search_chars.contains('-') {
            // Handle character ranges like a-f, A-F, 0-9
            let mut keep_chars = Vec::new();
            let chars: Vec<char> = search_chars.chars().collect();
            let mut i = 0;
            while i < chars.len() {
                if i + 2 < chars.len() && chars[i + 1] == '-' {
                    // Character range
                    let start = chars[i] as u8;
                    let end = chars[i + 2] as u8;
                    for c in start..=end {
                        keep_chars.push(c as char);
                    }
                    i += 3;
                } else if chars[i] != '-' {
                    // Single character
                    keep_chars.push(chars[i]);
                    i += 1;
                } else {
                    i += 1;
                }
            }
            let keep_list: Vec<String> = keep_chars.iter().map(|c| format!("'{}'", c)).collect();
            Ok(format!(
                "val.to_string().chars().filter(|c| [{}].contains(c)).collect::<String>()",
                keep_list.join(", ")
            ))
        } else {
            // Simple character list
            let keep_chars: Vec<String> =
                search_chars.chars().map(|c| format!("'{}'", c)).collect();
            Ok(format!(
                "val.to_string().chars().filter(|c| [{}].contains(c)).collect::<String>()",
                keep_chars.join(", ")
            ))
        }
    } else {
        // Character-by-character replacement
        // Build a replacement map
        let search_vec: Vec<char> = search_chars.chars().collect();
        let replace_vec: Vec<char> = replace_chars.chars().collect();

        if search_vec.len() != replace_vec.len() {
            return Err(CodeGenError::UnsupportedStructure(format!(
                "Transliterate pattern length mismatch: {} vs {}",
                search_chars, replace_chars
            )));
        }

        // Generate character mapping code
        let mut mappings = Vec::new();
        for (s, r) in search_vec.iter().zip(replace_vec.iter()) {
            mappings.push(format!("'{}' => '{}'", s, r));
        }

        Ok(format!(
            "val.to_string().chars().map(|c| match c {{ {} , _ => c }}).collect::<String>()",
            mappings.join(", ")
        ))
    }
}

/// Extract character ranges from tr/// patterns (helper for transliterate processing)
pub fn extract_char_ranges(pattern: &str) -> Vec<char> {
    let mut chars = Vec::new();
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;

    while i < pattern_chars.len() {
        if i + 2 < pattern_chars.len() && pattern_chars[i + 1] == '-' {
            // Character range like a-z
            let start = pattern_chars[i] as u8;
            let end = pattern_chars[i + 2] as u8;
            for c in start..=end {
                chars.push(c as char);
            }
            i += 3;
        } else if pattern_chars[i] != '-' {
            // Single character
            chars.push(pattern_chars[i]);
            i += 1;
        } else {
            i += 1;
        }
    }

    chars
}
