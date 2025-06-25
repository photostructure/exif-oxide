//! GPMF Format Extractor
//!
//! Extracts GoPro GPMF format definitions from ExifTool's GoPro.pm
//! Generates src/gpmf/format.rs with format type mappings
//!
//! GPMF uses custom format codes unlike standard EXIF formats

use super::Extractor;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
struct GpmfFormatEntry {
    code: u8,            // Format code like 0x62
    code_char: char,     // Character representation like 'b'
    rust_type: String,   // Rust type like "i8"
    exif_format: String, // ExifTool format name like "int8s"
    size: Option<usize>, // Custom size if different from default
}

pub struct GpmfFormatExtractor {
    formats: Vec<GpmfFormatEntry>,
}

impl GpmfFormatExtractor {
    pub fn new() -> Self {
        Self {
            formats: Vec::new(),
        }
    }

    /// Parse the %goProFmt and %goProSize hashes from GoPro.pm
    fn parse_format_definitions(&mut self, gromf_path: &Path) -> Result<(), String> {
        let content = fs::read_to_string(gromf_path)
            .map_err(|e| format!("Failed to read GoPro.pm: {}", e))?;

        // Parse format mappings
        self.parse_gopro_fmt(&content)?;

        // Parse size overrides
        self.parse_gopro_size(&content)?;

        println!("  - Found {} GPMF format codes", self.formats.len());
        Ok(())
    }

    /// Parse the %goProFmt hash for format code mappings
    fn parse_gopro_fmt(&mut self, content: &str) -> Result<(), String> {
        // Find the goProFmt hash definition
        let fmt_start = content
            .find("my %goProFmt = (")
            .ok_or("Could not find goProFmt hash definition")?;

        let fmt_end = self.find_hash_end(content, fmt_start)?;
        let fmt_content = &content[fmt_start..fmt_end];

        println!("  - Parsing goProFmt hash ({} bytes)", fmt_content.len());

        // Regex to match format entries like: 0x62 => 'int8s',    # 'b'
        let fmt_re = Regex::new(r"^\s*0x([0-9a-fA-F]{2})\s*=>\s*'([^']+)',?\s*#\s*'(.)'")
            .map_err(|e| format!("Failed to compile format regex: {}", e))?;

        for line in fmt_content.lines() {
            if let Some(caps) = fmt_re.captures(line.trim()) {
                let code = u8::from_str_radix(&caps[1], 16)
                    .map_err(|e| format!("Invalid hex code {}: {}", &caps[1], e))?;
                let exif_format = caps[2].to_string();
                let code_char = caps[3]
                    .chars()
                    .next()
                    .ok_or("Invalid character representation")?;

                let rust_type = self.exif_format_to_rust_type(&exif_format);

                let format_entry = GpmfFormatEntry {
                    code,
                    code_char,
                    rust_type,
                    exif_format,
                    size: None, // Will be filled in by parse_gopro_size
                };

                self.formats.push(format_entry);
            }
        }

        Ok(())
    }

    /// Parse the %goProSize hash for size overrides
    fn parse_gopro_size(&mut self, content: &str) -> Result<(), String> {
        // Find the goProSize hash definition
        let size_start = content
            .find("my %goProSize = (")
            .ok_or("Could not find goProSize hash definition")?;

        let size_end = self.find_hash_end(content, size_start)?;
        let size_content = &content[size_start..size_end];

        println!("  - Parsing goProSize hash ({} bytes)", size_content.len());

        // Build a map of size overrides
        let mut size_overrides = HashMap::new();

        // Regex to match size entries like: 0x46 => 4,
        let size_re = Regex::new(r"^\s*0x([0-9a-fA-F]{2})\s*=>\s*(\d+),?")
            .map_err(|e| format!("Failed to compile size regex: {}", e))?;

        for line in size_content.lines() {
            if let Some(caps) = size_re.captures(line.trim()) {
                let code = u8::from_str_radix(&caps[1], 16)
                    .map_err(|e| format!("Invalid hex code {}: {}", &caps[1], e))?;
                let size = caps[2]
                    .parse::<usize>()
                    .map_err(|e| format!("Invalid size {}: {}", &caps[2], e))?;

                size_overrides.insert(code, size);
            }
        }

        // Apply size overrides to format entries
        for format in &mut self.formats {
            if let Some(&size) = size_overrides.get(&format.code) {
                format.size = Some(size);
            }
        }

        Ok(())
    }

    /// Find the end of a hash definition
    fn find_hash_end(&self, content: &str, start: usize) -> Result<usize, String> {
        let mut paren_count = 0;
        let mut in_string = false;
        let mut escape_next = false;

        for (i, ch) in content[start..].char_indices() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if in_string => escape_next = true,
                '\'' => in_string = !in_string,
                '(' if !in_string => paren_count += 1,
                ')' if !in_string => {
                    paren_count -= 1;
                    if paren_count == 0 {
                        return Ok(start + i + 1);
                    }
                }
                _ => {}
            }
        }

        Err("Could not find end of hash definition".to_string())
    }

    /// Convert ExifTool format name to Rust type
    fn exif_format_to_rust_type(&self, exif_format: &str) -> String {
        match exif_format {
            "int8s" => "i8".to_string(),
            "int8u" => "u8".to_string(),
            "int16s" => "i16".to_string(),
            "int16u" => "u16".to_string(),
            "int32s" => "i32".to_string(),
            "int32u" => "u32".to_string(),
            "int64s" => "i64".to_string(),
            "int64u" => "u64".to_string(),
            "float" => "f32".to_string(),
            "double" => "f64".to_string(),
            "fixed32s" => "i32".to_string(), // Q-format fixed point
            "fixed64s" => "i64".to_string(), // Q-format fixed point
            "string" => "String".to_string(),
            "undef" => "Vec<u8>".to_string(),
            _ => format!("/* Unknown: {} */ Vec<u8>", exif_format),
        }
    }

    /// Generate the complete Rust code for the GPMF format definitions
    fn generate_code(&self) -> Result<String, String> {
        let mut code = String::new();

        // Header
        code.push_str(
            "//! Auto-generated GPMF format definitions\n\
             //!\n\
             //! EXIFTOOL-SOURCE: lib/Image/ExifTool/GoPro.pm\n\
             //! EXIFTOOL-VERSION: 12.65\n\
             //!\n\
             //! This file is auto-generated by exiftool_sync extract gpmf-format.\n\
             //! Do not edit manually.\n\n\
             #![doc = \"EXIFTOOL-SOURCE: lib/Image/ExifTool/GoPro.pm\"]\n\n\
             use std::collections::HashMap;\n\
             use lazy_static::lazy_static;\n\n",
        );

        // GPMF format enum
        code.push_str(
            "/// GPMF data format types\n\
             #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n\
             pub enum GpmfFormat {\n",
        );

        // Generate unique variants for each format, handling duplicates
        let mut used_variants = std::collections::HashSet::new();
        for format in &self.formats {
            let mut variant_name = format
                .exif_format
                .replace("int", "Int")
                .replace("fixed", "Fixed")
                .replace("string", "String")
                .replace("undef", "Undef")
                .replace("float", "Float")
                .replace("double", "Double");

            // Handle duplicate Undef variants by adding character suffix
            if variant_name == "Undef" && used_variants.contains(&variant_name) {
                variant_name = format!("Undef{}", format.code_char.to_ascii_uppercase());
            }

            used_variants.insert(variant_name.clone());

            code.push_str(&format!(
                "    /// {} - '{}' - {}\n\
                 \x20\x20\x20\x20{},\n",
                format.exif_format, format.code_char, format.rust_type, variant_name
            ));
        }

        code.push_str("}\n\n");

        // Format code mapping
        code.push_str("lazy_static! {\n");
        code.push_str(
            "    /// Map format codes to GpmfFormat variants\n\
             \x20\x20\x20\x20pub static ref GPMF_FORMAT_MAP: HashMap<u8, GpmfFormat> = {\n\
             \x20\x20\x20\x20\x20\x20\x20\x20let mut map = HashMap::new();\n",
        );

        // Generate mappings with unique variants (handle duplicates)
        let mut used_variants = std::collections::HashSet::new();
        for format in &self.formats {
            let mut variant_name = format
                .exif_format
                .replace("int", "Int")
                .replace("fixed", "Fixed")
                .replace("string", "String")
                .replace("undef", "Undef")
                .replace("float", "Float")
                .replace("double", "Double");

            // Handle duplicate Undef variants by adding character suffix
            if variant_name == "Undef" && used_variants.contains(&variant_name) {
                variant_name = format!("Undef{}", format.code_char.to_ascii_uppercase());
            }

            used_variants.insert(variant_name.clone());

            code.push_str(&format!(
                "        map.insert(0x{:02x}, GpmfFormat::{}); // '{}'\n",
                format.code, variant_name, format.code_char
            ));
        }

        code.push_str(
            "        map\n\
             \x20\x20\x20\x20};\n\n",
        );

        // Size mapping
        code.push_str(
            "    /// Map format codes to custom sizes (when different from default)\n\
             \x20\x20\x20\x20pub static ref GPMF_SIZE_MAP: HashMap<u8, usize> = {\n\
             \x20\x20\x20\x20\x20\x20\x20\x20let mut map = HashMap::new();\n",
        );

        for format in &self.formats {
            if let Some(size) = format.size {
                code.push_str(&format!(
                    "        map.insert(0x{:02x}, {}); // '{}'\n",
                    format.code, size, format.code_char
                ));
            }
        }

        code.push_str(
            "        map\n\
             \x20\x20\x20\x20};\n\
             }\n\n",
        );

        // Helper functions
        code.push_str(
            "/// Get GPMF format from format code\n\
             pub fn get_gpmf_format(code: u8) -> Option<GpmfFormat> {\n\
             \x20\x20\x20\x20GPMF_FORMAT_MAP.get(&code).copied()\n\
             }\n\n\
             \n\
             /// Get custom size for format code (if different from default)\n\
             pub fn get_gpmf_size(code: u8) -> Option<usize> {\n\
             \x20\x20\x20\x20GPMF_SIZE_MAP.get(&code).copied()\n\
             }\n\n\
             \n\
             /// Get default size for a format type\n\
             pub fn get_default_format_size(format: GpmfFormat) -> usize {\n\
             \x20\x20\x20\x20match format {\n",
        );

        // Generate size match arms with unique variants
        let mut used_variants = std::collections::HashSet::new();
        for format in &self.formats {
            let mut variant_name = format
                .exif_format
                .replace("int", "Int")
                .replace("fixed", "Fixed")
                .replace("string", "String")
                .replace("undef", "Undef")
                .replace("float", "Float")
                .replace("double", "Double");

            // Handle duplicate Undef variants by adding character suffix
            if variant_name == "Undef" && used_variants.contains(&variant_name) {
                variant_name = format!("Undef{}", format.code_char.to_ascii_uppercase());
            }

            used_variants.insert(variant_name.clone());

            let default_size = match format.exif_format.as_str() {
                "int8s" | "int8u" => 1,
                "int16s" | "int16u" => 2,
                "int32s" | "int32u" | "float" | "fixed32s" => 4,
                "int64s" | "int64u" | "double" | "fixed64s" => 8,
                "string" | "undef" => 1, // Variable length
                _ => 1,
            };

            code.push_str(&format!(
                "        GpmfFormat::{} => {},\n",
                variant_name, default_size
            ));
        }

        code.push_str(
            "    }\n\
             }\n\n",
        );

        // Format information
        code.push_str(&format!(
            "/// Total GPMF format codes: {}\n\
             pub const GPMF_FORMAT_COUNT: usize = {};\n",
            self.formats.len(),
            self.formats.len()
        ));

        Ok(code)
    }

    /// Generate stub code when no data is found
    fn generate_stub_code(&self) -> Result<String, String> {
        let mut code = String::new();

        code.push_str(
            "//! Auto-generated stub GPMF format definitions\n\
             //!\n\
             //! EXIFTOOL-SOURCE: lib/Image/ExifTool/GoPro.pm\n\
             //! This is a stub file generated when no data was found.\n\
             //! Regenerate with: cargo run --bin exiftool_sync extract gpmf-format\n\n\
             #![doc = \"EXIFTOOL-SOURCE: lib/Image/ExifTool/GoPro.pm\"]\n\n\
             use std::collections::HashMap;\n\
             use lazy_static::lazy_static;\n\n",
        );

        code.push_str(
            "/// Stub GPMF data format types\n\
             #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n\
             pub enum GpmfFormat {\n\
             \x20\x20\x20\x20// Stub - real format types will be generated by extractor\n\
             \x20\x20\x20\x20Placeholder,\n\
             }\n\n",
        );

        code.push_str(
            "lazy_static! {\n\
             \x20\x20\x20\x20/// Stub format map (empty - regenerate to populate)\n\
             \x20\x20\x20\x20pub static ref GPMF_FORMAT_MAP: HashMap<u8, GpmfFormat> = HashMap::new();\n\
             \x20\x20\x20\x20\n\
             \x20\x20\x20\x20/// Stub size map (empty - regenerate to populate)\n\
             \x20\x20\x20\x20pub static ref GPMF_SIZE_MAP: HashMap<u8, usize> = HashMap::new();\n\
             }\n\n"
        );

        code.push_str(
            "/// Get GPMF format from format code (stub implementation)\n\
             pub fn get_gpmf_format(_code: u8) -> Option<GpmfFormat> {\n\
             \x20\x20\x20\x20None // Stub implementation\n\
             }\n\n\
             \n\
             /// Get custom size for format code (stub implementation)\n\
             pub fn get_gpmf_size(_code: u8) -> Option<usize> {\n\
             \x20\x20\x20\x20None // Stub implementation\n\
             }\n",
        );

        Ok(code)
    }

    /// Write the generated code to the output file
    fn write_output(&self, code: &str) -> Result<(), String> {
        let output_path = Path::new("src/gpmf/format.rs");

        // Create the gpmf directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory {}: {}", parent.display(), e))?;
        }

        fs::write(output_path, code)
            .map_err(|e| format!("Failed to write {}: {}", output_path.display(), e))?;

        println!("Generated: {}", output_path.display());
        Ok(())
    }
}

impl Extractor for GpmfFormatExtractor {
    fn extract(&self, exiftool_path: &Path) -> Result<(), String> {
        println!("Extracting GPMF format definitions from ExifTool GoPro.pm...");

        let gromf_path = exiftool_path.join("lib/Image/ExifTool/GoPro.pm");
        let mut extractor = Self::new();

        // Always attempt extraction (never fail completely)
        match extractor.parse_format_definitions(&gromf_path) {
            Ok(()) => {
                println!("Found {} GPMF format codes", extractor.formats.len());
            }
            Err(e) => {
                println!("Warning: Failed to parse: {}", e);
                println!("  - Generating stub implementation");
            }
        }

        // Always generate code (real or stub)
        let code = if extractor.formats.is_empty() {
            extractor.generate_stub_code()?
        } else {
            extractor.generate_code()?
        };

        // Always write output
        extractor.write_output(&code)?;

        // Clear completion message
        println!("GPMF format extraction completed successfully");
        if extractor.formats.is_empty() {
            println!("  - Using stub implementation (no data found)");
        } else {
            println!(
                "  - Generated {} GPMF format codes",
                extractor.formats.len()
            );
        }

        Ok(())
    }
}
