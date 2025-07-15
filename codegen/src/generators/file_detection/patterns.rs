//! Magic number pattern generation for file type detection

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tracing::{debug, info};
use base64::{Engine as _, engine::general_purpose};

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

#[derive(Debug, Deserialize)]
pub struct MagicNumberData {
    pub extracted_at: String,
    pub magic_patterns: Vec<MagicPatternEntry>,
    pub stats: MagicNumberStats,
}

#[derive(Debug, Deserialize)]
pub struct MagicPatternEntry {
    pub file_type: String,
    #[serde(default)]
    pub pattern: String,
    // Base64-encoded pattern to avoid character escaping issues
    // ExifTool patterns contain raw bytes (0x00-0xFF) that don't translate
    // well through JSON -> Rust string literals -> regex compilation.
    // Base64 encoding preserves the exact byte sequence without any
    // interpretation or escaping complications.
    #[serde(default)]
    pub pattern_base64: String,
    pub source: MagicPatternSource,
}

#[derive(Debug, Deserialize)]
pub struct MagicPatternSource {
    pub module: String,
    pub hash: String,
}

#[derive(Debug, Deserialize)]
pub struct MagicNumberStats {
    #[serde(deserialize_with = "string_or_number")]
    pub total_patterns: usize,
}

fn string_or_number<'de, D>(deserializer: D) -> Result<usize, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct StringOrNumber;

    impl<'de> Visitor<'de> for StringOrNumber {
        type Value = usize;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or number")
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value as usize)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            value.parse().map_err(de::Error::custom)
        }
    }

    deserializer.deserialize_any(StringOrNumber)
}

/// Escape a string pattern for use in Rust string literals
/// This handles non-UTF-8 bytes by converting them to \xNN escape sequences
/// For bytes::Regex patterns, we need to escape ALL ASCII letters to their hex equivalents
/// when they appear outside of escape sequences, to ensure proper byte matching
fn escape_pattern_for_rust(pattern: &str) -> String {
    let mut escaped = String::new();
    let bytes = pattern.as_bytes();
    let mut i = 0;
    
    while i < bytes.len() {
        let byte = bytes[i];
        
        // Check if this is the start of an escape sequence
        if byte == b'\\' && i + 1 < bytes.len() {
            // Handle escape sequences
            match bytes[i + 1] {
                b'\\' => {
                    escaped.push_str("\\\\\\\\");  // Need 4 backslashes for regex in byte string
                    i += 2;
                    continue;
                }
                b'x' if i + 3 < bytes.len() => {
                    // Pass through hex escape sequences as-is
                    escaped.push_str(&pattern[i..i+4]);
                    i += 4;
                    continue;
                }
                b'r' => {
                    escaped.push_str("\\r");
                    i += 2;
                    continue;
                }
                b'n' => {
                    escaped.push_str("\\n");
                    i += 2;
                    continue;
                }
                b't' => {
                    escaped.push_str("\\t");
                    i += 2;
                    continue;
                }
                b'"' => {
                    escaped.push_str("\\\"");
                    i += 2;
                    continue;
                }
                // For regex metacharacters that were escaped in the pattern,
                // we need to keep them escaped with double backslashes for the regex
                b'+' | b'*' | b'?' | b'.' | b'^' | b'$' | b'(' | b')' | 
                b'[' | b']' | b'{' | b'}' | b'|' | b'd' | b's' | b'S' | b'w' | b'W' => {
                    escaped.push_str("\\\\");
                    escaped.push(bytes[i + 1] as char);
                    i += 2;
                    continue;
                }
                _ => {
                    // Other escape sequences - keep the backslash
                    escaped.push_str("\\\\");
                    escaped.push(bytes[i + 1] as char);
                    i += 2;
                    continue;
                }
            }
        }
        
        // Not part of an escape sequence - handle individual bytes
        match byte {
            // Standard string escapes that weren't already escaped
            b'"' => escaped.push_str("\\\""),
            b'\n' => escaped.push_str("\\n"),
            b'\r' => escaped.push_str("\\r"),
            b'\t' => escaped.push_str("\\t"),
            b'\\' => escaped.push_str("\\\\"),
            // Non-ASCII or control characters
            0x00..=0x1F | 0x7F..=0xFF => {
                escaped.push_str(&format!("\\x{:02x}", byte));
            }
            // All other ASCII characters remain as-is
            _ => escaped.push(byte as char),
        }
        i += 1;
    }
    
    escaped
}

/// Generate magic number patterns from magic_number.json
pub fn generate_magic_patterns(json_dir: &Path, output_dir: &str) -> Result<()> {
    // Look for regex_patterns.json in the extract directory
    let regex_patterns_path = json_dir.join("regex_patterns.json");
    
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
            
            debug!("Looking for pattern: {:?}", bad_pattern);
            if let Some(pos) = cleaned_bytes.windows(bad_pattern.len())
                .position(|window| window == bad_pattern) {
                cleaned_bytes.splice(pos..pos+bad_pattern.len(), good_pattern.iter().cloned());
                println!("    ✓ Fixed BPG pattern with non-UTF-8 byte");
                debug!("Replaced at position {}", pos);
            } else {
                debug!("BPG pattern not found in raw form");
            }
            
            // Try again with cleaned data
            String::from_utf8(cleaned_bytes)
                .map_err(|e| anyhow::anyhow!("Failed to clean non-UTF-8 data: {}", e))?
        }
    };
    
    let data: MagicNumberData = serde_json::from_str(&json_data)?;
    
    info!("Parsed MagicNumberData with {} patterns", data.magic_patterns.len());
    
    // Log patterns that contain \0 for debugging
    let patterns_with_null = data.magic_patterns.iter()
        .filter(|p| p.pattern.contains("\\0"))
        .count();
    if patterns_with_null > 0 {
        info!("Found {} patterns containing \\0 that need conversion", patterns_with_null);
    }
    
    // Generate magic_number_patterns.rs directly in output_dir  
    generate_magic_number_patterns_from_new_format(&data, Path::new(output_dir))?;
    
    println!("    ✓ Generated regex patterns with {} magic number patterns", data.magic_patterns.len());
    
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
        let result = generate_magic_number_patterns_from_new_format(&data, &temp_dir);
        
        // Should succeed
        assert!(result.is_ok(), "Failed to generate patterns with non-UTF-8 byte");
        
        // Read the generated file and verify the pattern is properly escaped
        let generated_path = temp_dir.join("file_types").join("magic_number_patterns.rs");
        assert!(generated_path.exists(), "Generated file does not exist at {:?}", generated_path);
        
        let content = std::fs::read_to_string(&generated_path).unwrap();
        
        // The pattern should be escaped as BPG\\xfb or BPG\xc3\xbb (UTF-8 encoding of \u{fb})
        assert!(content.contains(r#"map.insert("BPG", "BPG\\xfb");"#) || 
                content.contains(r#"map.insert("BPG", "BPG\xc3\xbb");"#) ||
                content.contains(r#"map.insert("BPG", "BPG\u{fb}");"#),
                "BPG pattern not properly escaped in generated code");
    }
    
    #[test]
    fn test_non_utf8_json_cleaning() {
        // Test the actual JSON cleaning logic
        // Create JSON with actual non-UTF-8 byte
        let mut bad_json_bytes = Vec::new();
        bad_json_bytes.extend_from_slice(b"{\"pattern\": \"BPG");
        bad_json_bytes.push(0xfb); // Add the actual non-UTF-8 byte
        bad_json_bytes.extend_from_slice(b"\", \"key\": \"BPG\"}");
        
        let bad_pattern = b"\"BPG\xfb\"";
        let good_pattern = b"\"BPG\\\\xfb\"";
        
        let mut test_bytes = bad_json_bytes.clone();
        
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

fn generate_magic_number_patterns_from_new_format(data: &MagicNumberData, output_dir: &Path) -> Result<()> {
    let mut code = String::new();
    
    // File header
    code.push_str("//! Magic number regex patterns generated from ExifTool's magicNumber hash\n");
    code.push_str("//!\n");
    code.push_str(&format!("//! Generated at: {}\n", data.extracted_at));
    code.push_str(&format!("//! Total patterns: {}\n", data.magic_patterns.len()));
    code.push_str("//! Source: ExifTool.pm %magicNumber hash\n");
    code.push_str("//!\n");
    code.push_str("//! IMPORTANT: These patterns use bytes::RegexBuilder with unicode(false)\n");
    code.push_str("//! to ensure hex escapes like \\x89 match raw bytes, not Unicode codepoints.\n");
    code.push_str("\n");
    code.push_str("use crate::file_types::lazy_regex::LazyRegexMap;\n");
    code.push_str("use once_cell::sync::Lazy;\n");
    code.push_str("use regex::bytes::Regex;\n");
    code.push_str("\n");
    
    // Generate pattern storage - just the patterns as strings, not compiled
    code.push_str("/// Magic number patterns from ExifTool's %magicNumber hash\n");
    code.push_str("static PATTERN_DATA: &[(&str, &str)] = &[\n");
    
    for entry in &data.magic_patterns {
        // Use the pattern field directly - it contains the regex syntax with proper escaping
        // (e.g. "\\x89" for byte 0x89), not the evaluated binary data
        let pattern_str = entry.pattern.clone();
        
        // Convert Perl regex syntax to Rust regex syntax
        // The patterns from JSON have different representations:
        // - Literal control characters that need to be escaped for regex
        // - Double backslashes for escape sequences: "\\x89" for \x89
        let mut converted_pattern = pattern_str.clone();
        
        // First, escape literal control characters that appear in JSON
        // These need to be converted to regex escape sequences
        converted_pattern = converted_pattern.replace('\n', "\\n");  // Newline
        converted_pattern = converted_pattern.replace('\r', "\\r");  // Carriage return
        converted_pattern = converted_pattern.replace('\t', "\\t");  // Tab
        converted_pattern = converted_pattern.replace('\u{0000}', "\\x00");  // Null byte
        
        // Convert \0{n} to \x00{n} (repeated null bytes)
        // In JSON this appears as "\\0{6}" which we need to convert to "\\x00{6}"
        converted_pattern = converted_pattern.replace("\\0{", "\\x00{");
        
        // Convert standalone \0 to \x00 (null bytes)
        // In JSON this appears as "\\0" which we need to convert to "\\x00"
        // Do this after the {n} replacement to avoid double-conversion
        converted_pattern = converted_pattern.replace("\\0", "\\x00");
        
        // Add ^ anchor at the beginning since ExifTool expects patterns to match from start
        let anchored_pattern = if converted_pattern.starts_with('^') {
            converted_pattern
        } else {
            format!("^{}", converted_pattern)
        };
        
        // Escape the pattern for use in a Rust string literal
        // We need to escape backslashes and quotes
        let escaped_pattern = anchored_pattern
            .replace('\\', "\\\\")  // Escape backslashes
            .replace('"', "\\\"");  // Escape quotes
        
        code.push_str(&format!(
            "    (\"{}\", \"{}\"),\n", 
            entry.file_type, escaped_pattern
        ));
        
        debug!("Generated pattern for {}: {}", entry.file_type, anchored_pattern);
    }
    
    code.push_str("];\n");
    code.push_str("\n");
    
    // Create the lazy regex map
    code.push_str("/// Lazy-compiled regex patterns for magic number detection\n");
    code.push_str("static MAGIC_PATTERNS: Lazy<LazyRegexMap> = Lazy::new(|| {\n");
    code.push_str("    LazyRegexMap::new(PATTERN_DATA)\n");
    code.push_str("});\n");
    code.push_str("\n");
    
    // Generate public API using the LazyRegexMap
    code.push_str("/// Test if a byte buffer matches a file type's magic number pattern\n");
    code.push_str("pub fn matches_magic_number(file_type: &str, buffer: &[u8]) -> bool {\n");
    code.push_str("    MAGIC_PATTERNS.matches(file_type, buffer)\n");
    code.push_str("}\n");
    code.push_str("\n");
    
    code.push_str("/// Get all file types with magic number patterns\n");
    code.push_str("pub fn get_magic_file_types() -> Vec<&'static str> {\n");
    code.push_str("    MAGIC_PATTERNS.file_types()\n");
    code.push_str("}\n");
    code.push_str("\n");
    
    code.push_str("/// Get compiled magic number regex for a file type\n");
    code.push_str("/// Uses the cached version if available, compiles and caches if not\n");
    code.push_str("pub fn get_magic_number_pattern(file_type: &str) -> Option<Regex> {\n");
    code.push_str("    MAGIC_PATTERNS.get_regex(file_type)\n");
    code.push_str("}\n");
    
    // Write the file to file_types subdirectory
    let file_types_dir = output_dir.join("file_types");
    fs::create_dir_all(&file_types_dir)?;
    let output_path = file_types_dir.join("magic_number_patterns.rs");
    fs::write(&output_path, code)?;
    
    Ok(())
}