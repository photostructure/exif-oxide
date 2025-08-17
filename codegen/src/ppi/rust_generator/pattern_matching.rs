//! Pattern matching and recognition for Perl constructs
//!
//! Handles detection and extraction of common Perl patterns that need special
//! handling during Rust code generation. Includes complexity checking to identify
//! constructs that cannot be reliably translated.
//!
//! Trust ExifTool: Pattern recognition preserves Perl semantics exactly.

use crate::ppi::rust_generator::errors::CodeGenError;
use crate::ppi::types::PpiNode;

/// Check if a node tree contains constructs that are too complex to translate reliably
pub fn check_node_complexity(node: &PpiNode) -> Result<(), CodeGenError> {
    // Check this node for problematic patterns
    match node.class.as_str() {
        // Variable declarations in multi-statement blocks are complex
        "PPI::Statement::Variable" => {
            return Err(CodeGenError::UnsupportedStructure(
                "Variable declarations in multi-statement blocks not supported".to_string(),
            ));
        }
        // Check for magic variable assignments at statement level
        "PPI::Statement" => {
            if has_magic_variable_assignment(node) {
                return Err(CodeGenError::UnsupportedStructure(
                    "Assignments to magic variables ($_) not supported".to_string(),
                ));
            }
        }
        // Complex regex patterns with flags
        "PPI::Token::Regexp::Match" => {
            if let Some(content) = &node.content {
                // Reject regex patterns with global flags or complex modifiers
                if content.contains("/g") || content.contains("/m") || content.contains("/s") {
                    return Err(CodeGenError::UnsupportedStructure(
                        "Regex patterns with global or multiline flags not supported".to_string(),
                    ));
                }
            }
        }
        // Perl references (backslash syntax) are not translatable
        _ => {
            if let Some(content) = &node.content {
                // Check for Perl reference syntax
                if content.contains("\\%") || content.contains("\\@") || content.contains("\\$") {
                    return Err(CodeGenError::UnsupportedStructure(
                        "Perl reference syntax (\\%) not supported".to_string(),
                    ));
                }
                // Check for foreach with special variables
                if content.contains("foreach") && content.contains("$_") {
                    return Err(CodeGenError::UnsupportedStructure(
                        "Foreach loops with special variables not supported".to_string(),
                    ));
                }
                // Check for assignments to magic variables like $_=$val
                if content.contains("$_=") || content.contains("$_ =") {
                    return Err(CodeGenError::UnsupportedStructure(
                        "Assignments to magic variables ($_) not supported".to_string(),
                    ));
                }
                // Check for complex array operations
                if content.contains("reverse @") {
                    return Err(CodeGenError::UnsupportedStructure(
                        "Complex array operations (reverse @) not supported".to_string(),
                    ));
                }
            }
        }
    }

    // Recursively check children
    for child in &node.children {
        check_node_complexity(child)?;
    }

    Ok(())
}

/// Extract pack/map pattern for bit extraction operations  
/// From ExifTool Canon.pm line 1847: pack "C*", map { (($_>>$_)&0x1f)+0x60 } 10, 5, 0
/// Returns: Some((mask, offset, shifts)) if pattern matches, None if not recognized
pub fn extract_pack_map_pattern(
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

/// Check if two statements form an operation-return pattern
/// Patterns: "$var =~ s/pattern// ; return $var" or "$var =~ s/pattern// ; $var"
pub fn is_operation_return_pattern(
    first_stmt: &PpiNode,
    second_stmt: &PpiNode,
) -> Result<bool, CodeGenError> {
    // Pattern 1: $var =~ s/pattern// ; return $var
    if second_stmt.class == "PPI::Statement::Break" {
        // Check if it's "return $var"
        if second_stmt.children.len() == 2
            && second_stmt.children[0].content.as_ref() == Some(&"return".to_string())
            && second_stmt.children[1].class == "PPI::Token::Symbol"
        {
            let return_var = second_stmt.children[1].content.as_ref();
            // Check if first statement operates on the same variable
            if let Some(first_var) = extract_operated_variable(first_stmt) {
                return Ok(return_var == Some(&first_var));
            }
        }
    }

    // Pattern 2: $var =~ s/pattern// ; $var
    if second_stmt.class == "PPI::Statement"
        && second_stmt.children.len() == 1
        && second_stmt.children[0].class == "PPI::Token::Symbol"
    {
        let result_var = second_stmt.children[0].content.as_ref();
        // Check if first statement operates on the same variable
        if let Some(first_var) = extract_operated_variable(first_stmt) {
            return Ok(result_var == Some(&first_var));
        }
    }

    Ok(false)
}

/// Extract the variable being operated on from a statement like "$var =~ s/pattern//"
pub fn extract_operated_variable(stmt: &PpiNode) -> Option<String> {
    if stmt.class == "PPI::Statement" && !stmt.children.is_empty() {
        // Look for $var =~ ... pattern
        if stmt.children.len() >= 3
            && stmt.children[0].class == "PPI::Token::Symbol"
            && stmt.children[1].class == "PPI::Token::Operator"
            && stmt.children[1].content.as_ref() == Some(&"=~".to_string())
        {
            return stmt.children[0].content.clone();
        }
    }
    None
}

/// Check if a statement node contains assignment to magic variable $_
fn has_magic_variable_assignment(stmt: &PpiNode) -> bool {
    if stmt.class != "PPI::Statement" || stmt.children.len() < 3 {
        return false;
    }

    // Look for pattern: $_ = value
    // First child should be magic variable $_
    if stmt.children[0].class == "PPI::Token::Magic" {
        if let Some(content) = &stmt.children[0].content {
            if content == "$_" {
                // Second child should be assignment operator
                if stmt.children.len() > 1 && stmt.children[1].class == "PPI::Token::Operator" {
                    if let Some(op) = &stmt.children[1].content {
                        if op == "=" {
                            return true;
                        }
                    }
                }
            }
        }
    }

    false
}
