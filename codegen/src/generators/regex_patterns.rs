//! Generator for regex patterns from ExifTool's magicNumber hash

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct RegexPatternsData {
    pub extracted_at: String,
    pub patterns: RegexPatterns,
    pub compatibility_notes: String,
}

#[derive(Debug, Deserialize)]
pub struct RegexPatterns {
    pub file_extensions: Vec<RegexPatternEntry>,
    pub magic_numbers: Vec<RegexPatternEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RegexPatternEntry {
    pub key: String,
    pub pattern: String,
    pub rust_compatible: i32,
    pub compatibility_notes: String,
    pub source_table: RegexPatternSource,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RegexPatternSource {
    pub module: String,
    pub hash_name: String,
    pub description: String,
}

/// Generate regex patterns from ExifTool's magicNumber data
pub fn generate_regex_patterns(json_path: &Path, output_dir: &str) -> Result<()> {
    let json_data = fs::read_to_string(json_path)?;
    let data: RegexPatternsData = serde_json::from_str(&json_data)?;
    
    // Create file_types directory
    let file_types_dir = Path::new(output_dir).join("simple_tables").join("file_types");
    fs::create_dir_all(&file_types_dir)?;
    
    // Generate magic_number_patterns.rs
    generate_magic_number_patterns(&data, &file_types_dir)?;
    
    println!("Generated regex patterns with {} magic number patterns", data.patterns.magic_numbers.len());
    
    Ok(())
}

fn generate_magic_number_patterns(data: &RegexPatternsData, output_dir: &Path) -> Result<()> {
    let mut code = String::new();
    
    // File header
    code.push_str("//! Magic number regex patterns generated from ExifTool's magicNumber hash\n");
    code.push_str("//!\n");
    code.push_str(&format!("//! Generated at: {}\n", data.extracted_at));
    code.push_str(&format!("//! Total patterns: {}\n", data.patterns.magic_numbers.len()));
    code.push_str(&format!("//! Compatibility: {}\n", data.compatibility_notes));
    code.push_str("\n");
    code.push_str("use std::collections::HashMap;\n");
    code.push_str("use once_cell::sync::Lazy;\n");
    code.push_str("\n");
    
    // Generate magic number patterns
    code.push_str("/// Magic number regex patterns for file type detection\n");
    code.push_str("/// These patterns are validated to be compatible with the Rust regex crate\n");
    code.push_str("static MAGIC_NUMBER_PATTERNS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {\n");
    code.push_str("    let mut map = HashMap::new();\n");
    
    for entry in &data.patterns.magic_numbers {
        if entry.rust_compatible == 1 {
            // Escape pattern for Rust string literal
            let escaped_pattern = entry.pattern.replace('\\', "\\\\").replace('"', "\\\"");
            code.push_str(&format!("    map.insert(\"{}\", \"{}\");\n", entry.key, escaped_pattern));
        }
    }
    
    code.push_str("    map\n");
    code.push_str("});\n");
    code.push_str("\n");
    
    // Generate compatibility map
    code.push_str("/// Compatibility status for each magic number pattern\n");
    code.push_str("static PATTERN_COMPATIBILITY: Lazy<HashMap<&'static str, bool>> = Lazy::new(|| {\n");
    code.push_str("    let mut map = HashMap::new();\n");
    
    for entry in &data.patterns.magic_numbers {
        code.push_str(&format!("    map.insert(\"{}\", {});\n", entry.key, entry.rust_compatible == 1));
    }
    
    code.push_str("    map\n");
    code.push_str("});\n");
    code.push_str("\n");
    
    // Generate public API
    code.push_str("/// Get magic number pattern for a file type\n");
    code.push_str("pub fn get_magic_number_pattern(file_type: &str) -> Option<&'static str> {\n");
    code.push_str("    MAGIC_NUMBER_PATTERNS.get(file_type).copied()\n");
    code.push_str("}\n");
    code.push_str("\n");
    
    code.push_str("/// Check if a file type has a Rust-compatible magic number pattern\n");
    code.push_str("pub fn is_pattern_compatible(file_type: &str) -> bool {\n");
    code.push_str("    PATTERN_COMPATIBILITY.get(file_type).copied().unwrap_or(false)\n");
    code.push_str("}\n");
    code.push_str("\n");
    
    code.push_str("/// Get all file types with magic number patterns\n");
    code.push_str("pub fn get_magic_file_types() -> Vec<&'static str> {\n");
    code.push_str("    MAGIC_NUMBER_PATTERNS.keys().copied().collect()\n");
    code.push_str("}\n");
    code.push_str("\n");
    
    code.push_str("/// Get all file types with Rust-compatible patterns\n");
    code.push_str("pub fn get_compatible_file_types() -> Vec<&'static str> {\n");
    code.push_str("    PATTERN_COMPATIBILITY.iter()\n");
    code.push_str("        .filter(|(_, &compatible)| compatible)\n");
    code.push_str("        .map(|(&file_type, _)| file_type)\n");
    code.push_str("        .collect()\n");
    code.push_str("}\n");
    
    // Write the file
    let output_path = output_dir.join("magic_number_patterns.rs");
    fs::write(&output_path, code)?;
    
    println!("Generated magic_number_patterns.rs");
    
    Ok(())
}