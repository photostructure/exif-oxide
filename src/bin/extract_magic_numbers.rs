#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! regex = "1"
//! lazy_static = "1"
//! serde = { version = "1", features = ["derive"] }
//! toml = "0.8"
//! chrono = "0.4"
//! ```
//!
//! Tool to extract magic number patterns from ExifTool's Perl source

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool.pm"]

use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io::Write;

#[derive(Debug)]
struct MagicPattern {
    file_type: String,
    pattern: String,
    perl_pattern: String,
    #[allow(dead_code)]
    description: Option<String>,
    weak: bool,
}

#[derive(Debug)]
struct MimeMapping {
    file_type: String,
    mime_type: String,
    canonical: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let exiftool_path = "../exiftool/lib/Image/ExifTool.pm";

    println!("Extracting magic numbers from ExifTool...");

    let content = fs::read_to_string(exiftool_path)?;

    // Extract magic number patterns
    let magic_patterns = extract_magic_numbers(&content)?;

    // Extract MIME type mappings
    let mime_mappings = extract_mime_types(&content)?;

    // Extract file type descriptions
    let file_type_lookup = extract_file_type_lookup(&content)?;

    // Generate Rust code
    generate_rust_module(&magic_patterns, &mime_mappings, &file_type_lookup)?;

    println!("Successfully generated src/detection/magic_numbers.rs");
    println!("Extracted {} magic patterns", magic_patterns.len());
    println!("Extracted {} MIME type mappings", mime_mappings.len());

    Ok(())
}

fn extract_magic_numbers(content: &str) -> Result<Vec<MagicPattern>, Box<dyn std::error::Error>> {
    let mut patterns = Vec::new();

    // Find the %magicNumber hash
    let magic_start = content
        .find("%magicNumber = (")
        .ok_or("Could not find %magicNumber hash")?;

    let magic_section = &content[magic_start..];
    let magic_end = magic_section
        .find(");")
        .ok_or("Could not find end of %magicNumber hash")?;

    let magic_content = &magic_section[..magic_end];

    // Parse each line - handle multi-line patterns
    let lines: Vec<&str> = magic_content.lines().collect();
    let mut i = 1; // Skip the opening line

    while i < lines.len() {
        let line = lines[i].trim();

        // Skip comments and empty lines
        if line.starts_with('#') || line.is_empty() {
            i += 1;
            continue;
        }

        // Match pattern: FileType => 'pattern',
        if let Some(caps) = Regex::new(r"^\s*(\w+)\s*=>\s*'(.+)'(?:,\s*)?$")?.captures(line) {
            let file_type = caps[1].to_string();
            let mut pattern = caps[2].to_string();

            // Handle multi-line patterns
            if pattern.ends_with("\\") {
                pattern.pop(); // Remove trailing backslash
                i += 1;
                while i < lines.len() && lines[i].trim().ends_with("\\") {
                    let continuation = lines[i].trim();
                    pattern.push_str(&continuation[..continuation.len() - 1]);
                    i += 1;
                }
                if i < lines.len() {
                    pattern.push_str(lines[i].trim().trim_end_matches("',"));
                }
            }

            patterns.push(MagicPattern {
                file_type: file_type.clone(),
                perl_pattern: pattern.clone(),
                pattern: convert_perl_to_rust_pattern(&pattern)?,
                description: None,
                weak: is_weak_magic(&file_type),
            });
        }

        i += 1;
    }

    Ok(patterns)
}

fn extract_mime_types(content: &str) -> Result<Vec<MimeMapping>, Box<dyn std::error::Error>> {
    let mut mappings = Vec::new();

    // Find the %mimeType hash
    let mime_start = content
        .find("%mimeType = (")
        .ok_or("Could not find %mimeType hash")?;

    let mime_section = &content[mime_start..];
    let mime_end = mime_section
        .find(");")
        .ok_or("Could not find end of %mimeType hash")?;

    let mime_content = &mime_section[..mime_end];

    // Parse each line
    for line in mime_content.lines().skip(1) {
        let line = line.trim();

        // Skip comments and empty lines
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        // Match pattern: FileType => 'mime/type', # comment
        if let Some(caps) = Regex::new(r"^\s*(\w+)\s*=>\s*'([^']+)'(?:,\s*#.*)?")?.captures(line) {
            let file_type = caps[1].to_string();
            let mime_type = caps[2].to_string();
            let canonical = !line.contains("#PH (NC)");

            mappings.push(MimeMapping {
                file_type,
                mime_type,
                canonical,
            });
        }
    }

    Ok(mappings)
}

type FileTypeLookup = HashMap<String, (String, Option<String>)>;

fn extract_file_type_lookup(content: &str) -> Result<FileTypeLookup, Box<dyn std::error::Error>> {
    let mut lookup = HashMap::new();

    // Find the %fileTypeLookup hash
    let lookup_start = content
        .find("%fileTypeLookup = (")
        .ok_or("Could not find %fileTypeLookup hash")?;

    let lookup_section = &content[lookup_start..];
    let lookup_end = lookup_section
        .find(");")
        .ok_or("Could not find end of %fileTypeLookup hash")?;

    let lookup_content = &lookup_section[..lookup_end];

    // Parse each line
    for line in lookup_content.lines().skip(1) {
        let line = line.trim();

        // Skip comments and empty lines
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        // Match patterns
        // Simple: 'EXT' => 'TYPE',
        if let Some(caps) = Regex::new(r"^'(\w+)'\s*=>\s*'(\w+)'")?.captures(line) {
            let ext = caps[1].to_string();
            let file_type = caps[2].to_string();
            lookup.insert(ext, (file_type, None));
        }
        // Array: 'EXT' => ['TYPE', 'Description'],
        else if let Some(caps) =
            Regex::new(r"^'(\w+)'\s*=>\s*\['(\w+)',\s*'([^']+)'\]")?.captures(line)
        {
            let ext = caps[1].to_string();
            let file_type = caps[2].to_string();
            let description = caps[3].to_string();
            lookup.insert(ext, (file_type, Some(description)));
        }
    }

    Ok(lookup)
}

fn convert_perl_to_rust_pattern(perl_pattern: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Convert Perl regex to Rust byte pattern
    let mut rust_pattern = String::new();
    let chars: Vec<char> = perl_pattern.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            '\\' if i + 1 < chars.len() => {
                match chars[i + 1] {
                    'x' if i + 3 < chars.len() => {
                        // Handle \xFF patterns
                        let hex = &perl_pattern[i + 2..i + 4];
                        rust_pattern.push_str(&format!("0x{}", hex));
                        i += 4;
                        continue;
                    }
                    '0' => {
                        rust_pattern.push_str("0x00");
                        i += 2;
                        continue;
                    }
                    'n' => {
                        rust_pattern.push_str("0x0a");
                        i += 2;
                        continue;
                    }
                    'r' => {
                        rust_pattern.push_str("0x0d");
                        i += 2;
                        continue;
                    }
                    _ => {
                        rust_pattern.push('\\');
                        i += 1;
                    }
                }
            }
            '.' => {
                rust_pattern.push('_'); // Any byte
                i += 1;
            }
            '[' => {
                // Convert character class
                let class_end = chars[i..].iter().position(|&c| c == ']').unwrap_or(0);
                let class = &perl_pattern[i..i + class_end + 1];
                rust_pattern.push_str(&convert_char_class(class));
                i += class_end + 1;
            }
            '(' | ')' | '|' | '{' | '}' | '*' | '+' | '?' => {
                // Keep regex operators
                rust_pattern.push(chars[i]);
                i += 1;
            }
            c if c.is_ascii() && !c.is_control() => {
                // Regular ASCII character
                rust_pattern.push_str(&format!("0x{:02x}", c as u8));
                i += 1;
            }
            _ => {
                i += 1;
            }
        }

        if i < chars.len()
            && chars[i] != '|'
            && chars[i] != '('
            && chars[i] != ')'
            && chars[i] != '{'
            && chars[i] != '}'
            && chars[i] != '*'
            && chars[i] != '+'
            && chars[i] != '?'
        {
            rust_pattern.push_str(", ");
        }
    }

    Ok(rust_pattern)
}

fn convert_char_class(class: &str) -> String {
    // Simple character class conversion
    if class == "[\\xf0\\xf1]" {
        return "(0xf0 | 0xf1)".to_string();
    }
    if class == "[\\x02\\x04\\x06\\x08]" {
        return "(0x02 | 0x04 | 0x06 | 0x08)".to_string();
    }
    if class == "[\\0-\\x20]" {
        return "(0x00..=0x20)".to_string();
    }
    // Default: keep as comment
    format!("/* {} */", class)
}

fn is_weak_magic(file_type: &str) -> bool {
    // File types that need additional validation beyond magic numbers
    matches!(file_type, "MP3" | "ID3" | "APE" | "FLAC" | "OGG" | "MPC")
}

fn generate_rust_module(
    patterns: &[MagicPattern],
    mime_mappings: &[MimeMapping],
    file_type_lookup: &HashMap<String, (String, Option<String>)>,
) -> Result<(), Box<dyn std::error::Error>> {
    let output_path = "src/detection/magic_numbers.rs";

    // Ensure directory exists
    fs::create_dir_all("src/detection")?;

    let mut file = fs::File::create(output_path)?;

    // Write header
    writeln!(file, "// AUTO-GENERATED from ExifTool v12.65")?;
    writeln!(
        file,
        "// Source: lib/Image/ExifTool.pm (%magicNumber, %mimeType, %fileTypeLookup)"
    )?;
    writeln!(file, "// Generated: 2025-06-23 by extract_magic_numbers")?;
    writeln!(
        file,
        "// DO NOT EDIT - Regenerate with `cargo run --bin extract_magic_numbers`"
    )?;
    writeln!(file)?;
    writeln!(file, "use std::collections::HashMap;")?;
    writeln!(file, "use lazy_static::lazy_static;")?;
    writeln!(file)?;

    // File type enum
    writeln!(file, "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]")?;
    writeln!(file, "pub enum FileType {{")?;
    let mut file_types = patterns.iter().map(|p| &p.file_type).collect::<Vec<_>>();
    file_types.sort();
    file_types.dedup();
    for ft in &file_types {
        writeln!(file, "    {},", ft)?;
    }
    writeln!(file, "    Unknown,")?;
    writeln!(file, "}}")?;
    writeln!(file)?;

    // Magic pattern structure
    writeln!(file, "#[derive(Debug)]")?;
    writeln!(file, "pub struct MagicPattern {{")?;
    writeln!(file, "    pub pattern: &'static [u8],")?;
    writeln!(file, "    pub regex: Option<&'static str>,")?;
    writeln!(file, "    pub offset: usize,")?;
    writeln!(file, "    pub weak: bool,")?;
    writeln!(file, "    pub test_len: usize,")?;
    writeln!(file, "}}")?;
    writeln!(file)?;

    // Magic number patterns
    writeln!(file, "lazy_static! {{")?;
    writeln!(
        file,
        "    pub static ref MAGIC_NUMBERS: HashMap<FileType, Vec<MagicPattern>> = {{"
    )?;
    writeln!(file, "        let mut map = HashMap::new();")?;
    writeln!(file)?;

    for pattern in patterns {
        writeln!(
            file,
            "        // {} - perl: '{}'",
            pattern.file_type, pattern.perl_pattern
        )?;
        writeln!(
            file,
            "        map.insert(FileType::{}, vec![MagicPattern {{",
            pattern.file_type
        )?;
        writeln!(file, "            pattern: &[{}],", pattern.pattern)?;
        writeln!(
            file,
            "            regex: Some(\"{}\"),",
            pattern.perl_pattern.replace('"', "\\\"")
        )?;
        writeln!(file, "            offset: 0,")?;
        writeln!(file, "            weak: {},", pattern.weak)?;
        writeln!(file, "            test_len: 1024,")?;
        writeln!(file, "        }}]);")?;
        writeln!(file)?;
    }

    writeln!(file, "        map")?;
    writeln!(file, "    }};")?;
    writeln!(file)?;

    // MIME type mappings
    writeln!(
        file,
        "    pub static ref MIME_TYPES: HashMap<FileType, &'static str> = {{"
    )?;
    writeln!(file, "        let mut map = HashMap::new();")?;
    writeln!(file)?;

    for mapping in mime_mappings {
        if let Some(file_type) = file_types.iter().find(|ft| **ft == &mapping.file_type) {
            writeln!(
                file,
                "        map.insert(FileType::{}, \"{}\");{}",
                file_type,
                mapping.mime_type,
                if !mapping.canonical {
                    " // Non-canonical"
                } else {
                    ""
                }
            )?;
        }
    }

    writeln!(file, "        map")?;
    writeln!(file, "    }};")?;
    writeln!(file)?;

    // Extension lookup
    writeln!(file, "    pub static ref EXTENSION_LOOKUP: HashMap<&'static str, (FileType, Option<&'static str>)> = {{")?;
    writeln!(file, "        let mut map = HashMap::new();")?;
    writeln!(file)?;

    for (ext, (ft, desc)) in file_type_lookup {
        if let Some(file_type) = file_types.iter().find(|t| **t == ft) {
            write!(
                file,
                "        map.insert(\"{}\", (FileType::{}, ",
                ext.to_uppercase(),
                file_type
            )?;
            if let Some(d) = desc {
                write!(file, "Some(\"{}\")))", d)?;
            } else {
                write!(file, "None))")?;
            }
            writeln!(file, ";")?;
        }
    }

    writeln!(file, "        map")?;
    writeln!(file, "    }};")?;
    writeln!(file, "}}")?;

    Ok(())
}
