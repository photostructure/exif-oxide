//! Extractor for magic number patterns from ExifTool

use super::Extractor;
use regex::Regex;
use std::fs;
use std::io::Write;
use std::path::Path;

pub struct MagicNumbersExtractor;

#[derive(Debug)]
struct MagicPattern {
    file_type: String,
    pattern: String,
    is_weak: bool,
    comment: Option<String>,
}

impl MagicNumbersExtractor {
    pub fn new() -> Self {
        Self
    }

    /// Parse the %magicNumber hash from ExifTool.pm
    fn parse_magic_numbers(&self, content: &str) -> Result<Vec<MagicPattern>, String> {
        let mut patterns = Vec::new();

        // Find the start of %magicNumber hash
        let start_marker = "%magicNumber = (";
        let start_pos = content
            .find(start_marker)
            .ok_or_else(|| "Could not find %magicNumber hash".to_string())?;

        // Find the end by looking for the closing ');'
        let hash_start = start_pos + start_marker.len();
        let hash_end = content[hash_start..]
            .find(");")
            .ok_or_else(|| "Could not find end of %magicNumber hash".to_string())?;

        let hash_content = &content[hash_start..hash_start + hash_end];

        // Extract weak magic types
        let weak_magic = self.extract_weak_magic(content)?;

        // Parse each entry in the hash
        // Pattern: TYPE => 'regex',
        let entry_regex = Regex::new(r"(?m)^\s*(\w+)\s*=>\s*'([^'\\]*(?:\\.[^'\\]*)*)'").unwrap();

        for cap in entry_regex.captures_iter(hash_content) {
            let file_type = cap[1].to_string();
            let pattern = cap[2].to_string();
            let is_weak = weak_magic.contains(&file_type);

            // Check for comment on the same line
            let line_start = cap.get(0).unwrap().start();
            let line_end = hash_content[line_start..]
                .find('\n')
                .unwrap_or(hash_content.len() - line_start);
            let line = &hash_content[line_start..line_start + line_end];
            let comment = line
                .find('#')
                .map(|comment_pos| line[comment_pos + 1..].trim().to_string());

            patterns.push(MagicPattern {
                file_type,
                pattern,
                is_weak,
                comment,
            });
        }

        println!("    Found {} magic patterns", patterns.len());
        Ok(patterns)
    }

    /// Extract %weakMagic hash
    fn extract_weak_magic(&self, content: &str) -> Result<Vec<String>, String> {
        let mut weak_types = Vec::new();

        if let Some(weak_pos) = content.find("%weakMagic") {
            let weak_section = &content[weak_pos..weak_pos + 200];
            let weak_regex = Regex::new(r"(\w+)\s*=>\s*1").unwrap();

            for cap in weak_regex.captures_iter(weak_section) {
                weak_types.push(cap[1].to_string());
            }
        }

        Ok(weak_types)
    }

    /// Convert Perl regex to Rust pattern
    fn perl_to_rust_pattern(&self, perl_pattern: &str) -> String {
        let mut rust_pattern = String::new();
        let mut chars = perl_pattern.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '\\' => {
                    if let Some(&next_ch) = chars.peek() {
                        match next_ch {
                            'x' => {
                                // Hex escape: \xff -> 0xFF
                                chars.next(); // consume 'x'
                                let hex1 = chars.next().unwrap_or('0');
                                let hex2 = chars.next().unwrap_or('0');
                                rust_pattern.push_str(&format!("\\x{{{}{}}}", hex1, hex2));
                            }
                            'r' => {
                                chars.next();
                                rust_pattern.push_str("\\r");
                            }
                            'n' => {
                                chars.next();
                                rust_pattern.push_str("\\n");
                            }
                            't' => {
                                chars.next();
                                rust_pattern.push_str("\\t");
                            }
                            _ => {
                                rust_pattern.push('\\');
                            }
                        }
                    } else {
                        rust_pattern.push('\\');
                    }
                }
                '.' => {
                    // Check if this is inside a character class
                    let in_char_class = rust_pattern.contains('[') && !rust_pattern.contains(']');
                    if in_char_class {
                        rust_pattern.push('.');
                    } else {
                        // Any byte in binary regex
                        rust_pattern.push('.');
                    }
                }
                _ => rust_pattern.push(ch),
            }
        }

        rust_pattern
    }

    /// Generate Rust code for magic patterns
    fn generate_rust_code(&self, patterns: &[MagicPattern]) -> Result<(), String> {
        // Create output directory
        let output_dir = Path::new("src/detection");
        fs::create_dir_all(output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;

        let filepath = output_dir.join("magic_patterns.rs");
        let mut file = fs::File::create(&filepath)
            .map_err(|e| format!("Failed to create {}: {}", filepath.display(), e))?;

        // Write file header
        writeln!(
            file,
            "// AUTO-GENERATED by exiftool_sync extract magic-numbers"
        )
        .map_err(|e| format!("Write error: {}", e))?;
        writeln!(
            file,
            "// Source: third-party/exiftool/lib/Image/ExifTool.pm"
        )
        .map_err(|e| format!("Write error: {}", e))?;
        writeln!(file, "// Generated by exiftool_sync extract magic-numbers")
            .map_err(|e| format!("Write error: {}", e))?;
        writeln!(file, "// DO NOT EDIT - Regenerate with `cargo run --bin exiftool_sync extract magic-numbers`")
            .map_err(|e| format!("Write error: {}", e))?;
        writeln!(file).map_err(|e| format!("Write error: {}", e))?;
        writeln!(
            file,
            "#![doc = \"EXIFTOOL-SOURCE: lib/Image/ExifTool.pm:912-1027\"]"
        )
        .map_err(|e| format!("Write error: {}", e))?;
        writeln!(file).map_err(|e| format!("Write error: {}", e))?;

        // Write imports
        writeln!(file, "use regex::bytes::Regex;").map_err(|e| format!("Write error: {}", e))?;
        writeln!(file, "use lazy_static::lazy_static;")
            .map_err(|e| format!("Write error: {}", e))?;
        writeln!(file).map_err(|e| format!("Write error: {}", e))?;

        // Write pattern structure
        writeln!(file, "/// Magic number pattern for file type detection")
            .map_err(|e| format!("Write error: {}", e))?;
        writeln!(file, "#[derive(Debug, Clone)]").map_err(|e| format!("Write error: {}", e))?;
        writeln!(file, "pub struct MagicPattern {{").map_err(|e| format!("Write error: {}", e))?;
        writeln!(file, "    pub file_type: &'static str,")
            .map_err(|e| format!("Write error: {}", e))?;
        writeln!(file, "    pub pattern: &'static str,")
            .map_err(|e| format!("Write error: {}", e))?;
        writeln!(file, "    pub regex: &'static Regex,")
            .map_err(|e| format!("Write error: {}", e))?;
        writeln!(file, "    pub is_weak: bool,").map_err(|e| format!("Write error: {}", e))?;
        writeln!(file, "}}").map_err(|e| format!("Write error: {}", e))?;
        writeln!(file).map_err(|e| format!("Write error: {}", e))?;

        // Generate lazy_static regexes
        writeln!(file, "lazy_static! {{").map_err(|e| format!("Write error: {}", e))?;

        for pattern in patterns {
            let var_name = format!("REGEX_{}", pattern.file_type.to_uppercase());
            let rust_pattern = self.perl_to_rust_pattern(&pattern.pattern);

            writeln!(
                file,
                "    static ref {}: Regex = Regex::new(r#\"^{}\"#).unwrap();",
                var_name, rust_pattern
            )
            .map_err(|e| format!("Write error: {}", e))?;
        }

        writeln!(file, "}}").map_err(|e| format!("Write error: {}", e))?;
        writeln!(file).map_err(|e| format!("Write error: {}", e))?;

        // Generate patterns array
        writeln!(file, "/// All magic number patterns")
            .map_err(|e| format!("Write error: {}", e))?;
        writeln!(file, "pub static MAGIC_PATTERNS: &[MagicPattern] = &[")
            .map_err(|e| format!("Write error: {}", e))?;

        for pattern in patterns {
            let var_name = format!("REGEX_{}", pattern.file_type.to_uppercase());

            if let Some(comment) = &pattern.comment {
                writeln!(file, "    // {}", comment).map_err(|e| format!("Write error: {}", e))?;
            }

            writeln!(file, "    MagicPattern {{").map_err(|e| format!("Write error: {}", e))?;
            writeln!(file, "        file_type: \"{}\",", pattern.file_type)
                .map_err(|e| format!("Write error: {}", e))?;
            writeln!(file, "        pattern: r#\"{}\"#,", pattern.pattern)
                .map_err(|e| format!("Write error: {}", e))?;
            writeln!(file, "        regex: &{},", var_name)
                .map_err(|e| format!("Write error: {}", e))?;
            writeln!(file, "        is_weak: {},", pattern.is_weak)
                .map_err(|e| format!("Write error: {}", e))?;
            writeln!(file, "    }},").map_err(|e| format!("Write error: {}", e))?;
        }

        writeln!(file, "];").map_err(|e| format!("Write error: {}", e))?;
        writeln!(file).map_err(|e| format!("Write error: {}", e))?;

        // Add helper functions
        writeln!(
            file,
            "/// Test length for magic number detection (from ExifTool.pm line 906)"
        )
        .map_err(|e| format!("Write error: {}", e))?;
        writeln!(file, "pub const MAGIC_TEST_LENGTH: usize = 1024;")
            .map_err(|e| format!("Write error: {}", e))?;
        writeln!(file).map_err(|e| format!("Write error: {}", e))?;

        writeln!(file, "/// Detect file type from magic number")
            .map_err(|e| format!("Write error: {}", e))?;
        writeln!(
            file,
            "pub fn detect_file_type(data: &[u8]) -> Option<&'static str> {{"
        )
        .map_err(|e| format!("Write error: {}", e))?;
        writeln!(file, "    for pattern in MAGIC_PATTERNS {{")
            .map_err(|e| format!("Write error: {}", e))?;
        writeln!(file, "        if pattern.regex.is_match(data) {{")
            .map_err(|e| format!("Write error: {}", e))?;
        writeln!(file, "            return Some(pattern.file_type);")
            .map_err(|e| format!("Write error: {}", e))?;
        writeln!(file, "        }}").map_err(|e| format!("Write error: {}", e))?;
        writeln!(file, "    }}").map_err(|e| format!("Write error: {}", e))?;
        writeln!(file, "    None").map_err(|e| format!("Write error: {}", e))?;
        writeln!(file, "}}").map_err(|e| format!("Write error: {}", e))?;

        file.flush().map_err(|e| format!("Flush error: {}", e))?;

        println!("  Generated {}", filepath.display());
        Ok(())
    }
}

impl Extractor for MagicNumbersExtractor {
    fn extract(&self, exiftool_path: &Path) -> Result<(), String> {
        println!("Extracting magic numbers from ExifTool.pm...");

        // Read ExifTool.pm
        let exiftool_pm = exiftool_path.join("lib/Image/ExifTool.pm");
        if !exiftool_pm.exists() {
            return Err(format!(
                "ExifTool.pm not found at {}",
                exiftool_pm.display()
            ));
        }

        let content = fs::read_to_string(&exiftool_pm)
            .map_err(|e| format!("Failed to read ExifTool.pm: {}", e))?;

        // Parse magic numbers
        let patterns = self.parse_magic_numbers(&content)?;

        // Generate Rust code
        println!("\nGenerating Rust code...");
        self.generate_rust_code(&patterns)?;

        println!("\nExtracted {} magic number patterns", patterns.len());

        Ok(())
    }
}
