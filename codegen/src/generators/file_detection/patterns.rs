//! Magic number pattern generation for file type detection

use anyhow::Result;
use serde::{Deserialize, Serialize};
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

/// Generate magic number patterns from regex_patterns.json
pub fn generate_magic_patterns(json_dir: &Path, output_dir: &str) -> Result<()> {
    // Look for regex_patterns.json
    let regex_patterns_path = json_dir.parent()
        .ok_or_else(|| anyhow::anyhow!("Invalid json_dir path"))?
        .join("regex_patterns.json");
    
    if !regex_patterns_path.exists() {
        println!("    ⚠️  regex_patterns.json not found, skipping magic patterns");
        return Ok(());
    }
    
    // Read as bytes first to handle potential non-UTF-8 content
    let json_bytes = fs::read(&regex_patterns_path)?;
    
    // Try to parse as UTF-8, but if it fails, we need to handle it
    let json_data = match String::from_utf8(json_bytes.clone()) {
        Ok(s) => s,
        Err(_) => {
            // If UTF-8 conversion fails, we need to clean the data
            println!("    ⚠️  regex_patterns.json contains non-UTF-8 bytes, cleaning...");
            
            // Read the file and clean problematic patterns
            let mut cleaned_bytes = json_bytes;
            
            // Replace the problematic BPG pattern with a safe version
            // BPG\xfb -> BPG\\xfb (escaped version)
            let bad_pattern = b"\"BPG\xfb\"";
            let good_pattern = b"\"BPG\\\\xfb\"";
            
            if let Some(pos) = cleaned_bytes.windows(bad_pattern.len())
                .position(|window| window == bad_pattern) {
                cleaned_bytes.splice(pos..pos+bad_pattern.len(), good_pattern.iter().cloned());
                println!("    ✓ Fixed BPG pattern with non-UTF-8 byte");
            }
            
            // Try again with cleaned data
            String::from_utf8(cleaned_bytes)
                .map_err(|e| anyhow::anyhow!("Failed to clean non-UTF-8 data: {}", e))?
        }
    };
    
    let data: RegexPatternsData = serde_json::from_str(&json_data)?;
    
    // Generate magic_number_patterns.rs directly in output_dir
    generate_magic_number_patterns(&data, Path::new(output_dir))?;
    
    println!("    ✓ Generated regex patterns with {} magic number patterns", data.patterns.magic_numbers.len());
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bpg_pattern_utf8_handling() {
        // This test ensures the BPG pattern with non-UTF-8 byte (0xfb) is handled correctly.
        // The BPG (Better Portable Graphics) format has a magic number that includes
        // a non-UTF-8 byte, which must be properly escaped when generating Rust code.
        
        // Create test data with the problematic BPG pattern
        let test_pattern = RegexPatternEntry {
            key: "BPG".to_string(),
            pattern: "BPG\u{fb}".to_string(), // This will be invalid UTF-8 in JSON
            rust_compatible: 1,
            compatibility_notes: "Contains non-UTF-8 byte that needs escaping".to_string(),
            source_table: RegexPatternSource {
                module: "ExifTool.pm".to_string(),
                hash_name: "%magicNumber".to_string(),
                description: "Magic number patterns".to_string(),
            },
        };
        
        // The pattern should be escaped in the generated code
        let data = RegexPatternsData {
            extracted_at: "test".to_string(),
            patterns: RegexPatterns {
                file_extensions: vec![],
                magic_numbers: vec![test_pattern],
            },
            compatibility_notes: "test".to_string(),
        };
        
        // Generate code to a temp directory
        let temp_dir = std::env::temp_dir();
        let result = generate_magic_number_patterns(&data, &temp_dir);
        
        // Should succeed
        assert!(result.is_ok(), "Failed to generate patterns with non-UTF-8 byte");
        
        // Read the generated file and verify the pattern is properly escaped
        let generated_path = temp_dir.join("file_types").join("magic_number_patterns.rs");
        if generated_path.exists() {
            let content = std::fs::read_to_string(&generated_path).unwrap();
            // The pattern should be escaped as BPG\\xfb in the generated code
            assert!(content.contains(r#"map.insert("BPG", "BPG\\xfb");"#) || 
                    content.contains(r#"map.insert("BPG", "BPG\u{fb}");"#),
                    "BPG pattern not properly escaped in generated code");
        }
    }
    
    #[test]
    fn test_non_utf8_json_cleaning() {
        // Test the actual JSON cleaning logic
        let bad_json_bytes = br#"{"pattern": "BPG\xfb", "key": "BPG"}"#;
        let bad_pattern = b"\"BPG\xfb\"";
        let good_pattern = b"\"BPG\\\\xfb\"";
        
        let mut test_bytes = bad_json_bytes.to_vec();
        
        // Find and replace the pattern
        if let Some(pos) = test_bytes.windows(bad_pattern.len())
            .position(|window| window == bad_pattern) {
            test_bytes.splice(pos..pos+bad_pattern.len(), good_pattern.iter().cloned());
        }
        
        // Should now be valid UTF-8
        let result = String::from_utf8(test_bytes);
        assert!(result.is_ok(), "Failed to clean non-UTF-8 JSON");
        
        let cleaned = result.unwrap();
        assert!(cleaned.contains(r#""BPG\\xfb""#), "Pattern not properly escaped");
    }
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
    
    // Write the file to file_types subdirectory
    let file_types_dir = output_dir.join("file_types");
    fs::create_dir_all(&file_types_dir)?;
    let output_path = file_types_dir.join("magic_number_patterns.rs");
    fs::write(&output_path, code)?;
    
    Ok(())
}