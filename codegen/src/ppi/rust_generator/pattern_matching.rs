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
