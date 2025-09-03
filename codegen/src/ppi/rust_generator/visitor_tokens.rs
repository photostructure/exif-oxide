//! Helper functions for basic token processing
//!
//! This module contains standalone helper functions for processing simple PPI tokens
//! that don't require recursion or complex traversal logic.

use crate::ppi::rust_generator::errors::CodeGenError;
use crate::ppi::types::*;

/// Process symbol nodes (variables like $val, $$self{Field})
pub fn process_symbol(node: &PpiNode) -> Result<String, CodeGenError> {
    let content = node
        .content
        .as_ref()
        .ok_or(CodeGenError::MissingContent("symbol".to_string()))?;

    if node.is_self_reference() {
        if let Some(field) = node.extract_self_field() {
            Ok(format!("ctx.get(\"{field}\").unwrap_or_default()"))
        } else {
            Err(CodeGenError::InvalidSelfReference(content.clone()))
        }
    } else if content == "$val" {
        Ok("val".to_string())
    } else if content == "$valPt" {
        Ok("val_pt".to_string())
    } else if content == "$self" {
        // $self method calls are not supported in standalone functions
        // Following explicit failure semantics from CODEGEN.md
        Err(CodeGenError::UnsupportedStructure(
            "$self method calls are not supported in standalone functions".to_string(),
        ))
    } else {
        // Generic variable
        Ok(content.trim_start_matches('$').to_string())
    }
}

/// Process operator nodes
pub fn process_operator(node: &PpiNode) -> Result<String, CodeGenError> {
    let op = node
        .content
        .as_ref()
        .ok_or(CodeGenError::MissingContent("operator".to_string()))?;

    // Convert Perl operators to Rust equivalents
    // Trust ExifTool: preserve original logic but translate to valid Rust syntax
    let rust_op = match op.as_str() {
        "eq" => "==",
        "ne" => "!=",
        "lt" => "<",
        "gt" => ">",
        "le" => "<=",
        "ge" => ">=",
        "and" => "&&",
        "or" => "||",
        "xor" => "^",
        // Keep other operators as-is
        _ => op,
    };

    Ok(rust_op.to_string())
}
