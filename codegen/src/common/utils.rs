//! Common utility functions for code generation

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Find the repository root by walking up from the given path until we find Cargo.toml
#[allow(dead_code)]
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
    u32::from_str_radix(cleaned, 16).with_context(|| format!("Failed to parse hex ID: {hex_str}"))
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

/// Check if a string contains characters that require raw string literal in Rust
pub fn needs_raw_string(s: &str) -> bool {
    // Check for backslash followed by common escape sequences that aren't valid in Rust
    s.contains("\\s") || s.contains("\\S") || s.contains("\\d") || s.contains("\\D") ||
    s.contains("\\w") || s.contains("\\W") || s.contains("\\b") || s.contains("\\B") ||
    s.contains("\\A") || s.contains("\\z") || s.contains("\\Z") ||
    // Also check for regex patterns
    s.contains("=~") || s.contains("!~")
}

/// Format a string for use in Rust code, using raw string if needed
pub fn format_rust_string(s: &str) -> String {
    if needs_raw_string(s) {
        // Use raw string literal
        if s.contains('"') && !s.contains('#') {
            // Use r#"..."# format
            format!("r#\"{}\"#", s)
        } else if s.contains('"') && s.contains('#') {
            // Need to use more # symbols if the string contains #
            let mut hashes = "#".to_string();
            while s.contains(&format!("\"{}", hashes)) {
                hashes.push('#');
            }
            format!("r{}\"{}\"{}", hashes, s, hashes)
        } else {
            // Simple raw string
            format!("r\"{}\"", s)
        }
    } else {
        // Use regular escaped string
        format!("\"{}\"", escape_string(s))
    }
}
