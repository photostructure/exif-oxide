//! Common utility functions for code generation

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Find the repository root by walking up from the given path until we find Cargo.toml
pub fn find_repo_root(start_path: &Path) -> Result<PathBuf> {
    let mut current_path = start_path.canonicalize()?;
    while !current_path.join("Cargo.toml").exists() {
        current_path = current_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Could not find repository root (Cargo.toml)"))?
            .to_path_buf();
    }
    Ok(current_path)
}

/// Parse a hexadecimal tag ID string (e.g., "0x8769" -> 34665)
pub fn parse_hex_id(hex_str: &str) -> Result<u32> {
    let cleaned = hex_str.trim_start_matches("0x");
    u32::from_str_radix(cleaned, 16)
        .with_context(|| format!("Failed to parse hex ID: {}", hex_str))
}

/// Convert ExifTool format names to our Rust enum variant names
pub fn normalize_format(format: &str) -> String {
    match format {
        "int8u" => "U8",
        "int16u" => "U16",
        "int32u" => "U32",
        "int8s" => "I8",
        "int16s" => "I16",
        "int32s" => "I32",
        "rational64u" => "RationalU",
        "rational64s" => "RationalS",
        "string" => "String",
        "undef" | "binary" => "Undef",
        "float" => "Float",
        "double" => "Double",
        _ => "Undef", // Unknown formats default to undefined
    }
    .to_string()
}

/// Escape a string for use in Rust code
pub fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\") // Must be first to avoid double-escaping
        .replace('\"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Escape a string for use in Rust code (alias for escape_string)
pub fn escape_rust_string(s: &str) -> String {
    escape_string(s)
}

/// Escape a regex pattern for use in Rust raw strings
pub fn escape_regex_for_rust(pattern: &str) -> String {
    // For raw strings, we just need to handle quotes carefully
    // Most regex patterns work fine in raw strings r"..."
    // But if pattern contains quotes, we need to use regular strings
    if pattern.contains('"') {
        // Use regular string with minimal escaping
        pattern.replace('\"', "\\\"")
    } else {
        // Use the pattern as-is in raw string
        pattern.to_string()
    }
}