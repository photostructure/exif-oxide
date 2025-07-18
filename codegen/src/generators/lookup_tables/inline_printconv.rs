//! Inline PrintConv table generation
//!
//! This module handles generation of lookup tables from inline PrintConv
//! definitions extracted from ExifTool tag tables.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Deserialize, Serialize)]
pub struct InlinePrintConvData {
    pub source: InlinePrintConvSource,
    pub metadata: InlinePrintConvMetadata,
    pub inline_printconvs: Vec<InlinePrintConvEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InlinePrintConvSource {
    pub module: String,
    pub table: String,
    pub extracted_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InlinePrintConvMetadata {
    pub total_tags_scanned: usize,
    pub inline_printconvs_found: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InlinePrintConvEntry {
    pub tag_id: String,
    pub tag_name: String,
    pub key_type: String,
    pub entries: HashMap<String, String>,
    pub entry_count: usize,
}

/// Generate lookup table code for an inline PrintConv entry with custom tag name
pub fn generate_inline_printconv_lookup_with_name(
    table_name: &str,
    tag_name: &str,
    entry: &InlinePrintConvEntry,
) -> Result<String> {
    let mut code = String::new();
    
    // Generate constant name - combine table and tag name
    let const_name = format!(
        "{}_{}", 
        to_screaming_snake_case(table_name),
        to_screaming_snake_case(&tag_name)
    );
    
    // Determine Rust type based on key_type
    let rust_key_type = match entry.key_type.as_str() {
        "u8" => "u8",
        "u16" => "u16",
        "u32" => "u32",
        "i8" => "i8",
        "i16" => "i16",
        "i32" => "i32",
        "String" => "&str",
        _ => "i32", // Default to i32 for unknown types
    };
    
    // For string keys, we need to handle them differently
    let is_string_key = entry.key_type == "String";
    
    // Generate data array
    code.push_str(&format!("/// Raw data ({} entries)\n", entry.entry_count));
    
    if is_string_key {
        code.push_str(&format!("static {}_DATA: &[(&'static str, &'static str)] = &[\n", const_name));
        
        // Sort entries for consistent output
        let mut sorted_entries: Vec<_> = entry.entries.iter().collect();
        sorted_entries.sort_by_key(|(k, _)| k.as_str());
        
        for (key, value) in sorted_entries {
            code.push_str(&format!("    (\"{}\", \"{}\"),\n", escape_string(key), escape_string(value)));
        }
    } else {
        code.push_str(&format!("static {}_DATA: &[({}, &'static str)] = &[\n", const_name, rust_key_type));
        
        // Sort entries numerically/lexically
        let mut sorted_entries: Vec<_> = entry.entries.iter().collect();
        if is_numeric_type(rust_key_type) {
            sorted_entries.sort_by_key(|(k, _)| {
                k.parse::<i64>().unwrap_or(i64::MAX)
            });
        } else {
            sorted_entries.sort_by_key(|(k, _)| k.as_str());
        }
        
        for (key, value) in sorted_entries {
            code.push_str(&format!("    ({}, \"{}\"),\n", key, escape_string(value)));
        }
    }
    
    code.push_str("];\n\n");
    
    // Generate lazy HashMap
    code.push_str("/// Lookup table (lazy-initialized)\n");
    
    if is_string_key {
        code.push_str(&format!(
            "pub static {}: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {{\n",
            const_name
        ));
        code.push_str(&format!("    {}_DATA.iter().cloned().collect()\n", const_name));
        code.push_str("});\n\n");
        
        // Generate lookup function
        code.push_str("/// Look up value by key\n");
        code.push_str(&format!(
            "pub fn lookup_{}(key: &str) -> Option<&'static str> {{\n",
            to_snake_case(&format!("{}_{}", table_name, tag_name))
        ));
        code.push_str(&format!("    {}.get(key).copied()\n", const_name));
        code.push_str("}\n");
    } else {
        code.push_str(&format!(
            "pub static {}: LazyLock<HashMap<{}, &'static str>> = LazyLock::new(|| {{\n",
            const_name, rust_key_type
        ));
        code.push_str(&format!("    {}_DATA.iter().cloned().collect()\n", const_name));
        code.push_str("});\n\n");
        
        // Generate lookup function
        code.push_str("/// Look up value by key\n");
        code.push_str(&format!(
            "pub fn lookup_{}(key: {}) -> Option<&'static str> {{\n",
            to_snake_case(&format!("{}_{}", table_name, tag_name)),
            rust_key_type
        ));
        code.push_str(&format!("    {}.get(&key).copied()\n", const_name));
        code.push_str("}\n");
    }
    
    Ok(code)
}

/// Generate lookup table code for an inline PrintConv entry
pub fn generate_inline_printconv_lookup(
    table_name: &str,
    tag_name: &str,
    entry: &InlinePrintConvEntry,
) -> Result<String> {
    generate_inline_printconv_lookup_with_name(table_name, tag_name, entry)
}

/// Generate all inline PrintConv lookups for a file
pub fn generate_inline_printconv_file(
    data: &InlinePrintConvData,
    table_name: &str,
) -> Result<String> {
    let mut code = String::new();
    let mut used_names = HashSet::new();
    
    // Generate lookup for each inline PrintConv
    for (i, entry) in data.inline_printconvs.iter().enumerate() {
        if i > 0 {
            code.push_str("\n");
        }
        
        // Handle duplicate tag names by adding tag_id suffix
        let mut unique_tag_name = entry.tag_name.clone();
        let base_const_name = format!(
            "{}_{}", 
            to_screaming_snake_case(table_name),
            to_screaming_snake_case(&unique_tag_name)
        );
        
        if used_names.contains(&base_const_name) {
            // Add tag_id to make it unique
            unique_tag_name = format!("{}_{}", entry.tag_name, entry.tag_id);
        }
        
        let final_const_name = format!(
            "{}_{}", 
            to_screaming_snake_case(table_name),
            to_screaming_snake_case(&unique_tag_name)
        );
        
        used_names.insert(final_const_name);
        
        code.push_str(&generate_inline_printconv_lookup_with_name(table_name, &unique_tag_name, entry)?);
    }
    
    Ok(code)
}

/// Check if a type is numeric
fn is_numeric_type(rust_type: &str) -> bool {
    matches!(rust_type, "u8" | "u16" | "u32" | "i8" | "i16" | "i32" | "u64" | "i64")
}

/// Convert string to SCREAMING_SNAKE_CASE with proper Rust identifier handling
fn to_screaming_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && ch.is_uppercase() {
            result.push('_');
        }
        
        // Convert non-alphanumeric characters to underscores, but avoid consecutive underscores
        if ch.is_alphanumeric() {
            result.push(ch.to_uppercase().next().unwrap_or(ch));
        } else if !result.ends_with('_') {
            result.push('_');
        }
    }
    
    // Remove trailing underscore
    if result.ends_with('_') {
        result.pop();
    }
    
    result
}

/// Convert string to snake_case with proper Rust identifier handling
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && ch.is_uppercase() {
            result.push('_');
        }
        
        // Convert non-alphanumeric characters to underscores, but avoid consecutive underscores
        if ch.is_alphanumeric() {
            result.push(ch.to_lowercase().next().unwrap_or(ch));
        } else if !result.ends_with('_') {
            result.push('_');
        }
    }
    
    // Remove trailing underscore
    if result.ends_with('_') {
        result.pop();
    }
    
    result
}

/// Escape a string for use in Rust string literals
fn escape_string(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '"' => r#"\""#.to_string(),
            '\\' => r"\\".to_string(),
            '\n' => r"\n".to_string(),
            '\r' => r"\r".to_string(),
            '\t' => r"\t".to_string(),
            c if c.is_control() => format!("\\u{{{:04x}}}", c as u32),
            c => c.to_string(),
        })
        .collect()
}