//! Extractor for writable tag information from ExifTool source files
//!
//! This extractor scans all ExifTool .pm files to identify which tags are writable,
//! their formats, and classifies them by safety level for write operations.

use super::Extractor;
use regex::Regex;
use std::fs;
use std::io::Write;
use std::path::Path;

pub struct WritableTagsExtractor {
    tags: Vec<WritableTagInfo>,
}

#[derive(Debug, Clone)]
pub struct WritableTagInfo {
    pub tag_name: String,
    pub tag_id: Option<u32>,        // Some tags use string IDs (XMP)
    pub tag_id_hex: Option<String>, // Store original hex representation
    pub group: String,              // EXIF, Canon, XMP, etc.
    pub writable_format: WritableFormat,
    pub write_proc: Option<String>, // Custom write function if any
    pub safety_level: SafetyLevel,  // Classification based on patterns
    pub source_file: String,        // Which .pm file it came from
}

#[derive(Debug, Clone, PartialEq)]
pub enum SafetyLevel {
    Safe,       // Basic metadata - always safe to write
    Restricted, // Maker notes - preserve existing structure only
    Dangerous,  // File structure - never write programmatically
}

#[derive(Debug, Clone)]
pub enum WritableFormat {
    String,          // ASCII text
    Int8u,           // 8-bit unsigned integer
    Int16u,          // 16-bit unsigned integer
    Int32u,          // 32-bit unsigned integer
    Int8s,           // 8-bit signed integer
    Int16s,          // 16-bit signed integer
    Int32s,          // 32-bit signed integer
    Rational64u,     // Fraction (unsigned)
    Rational64s,     // Fraction (signed)
    Float,           // 32-bit float
    Double,          // 64-bit double
    Undef,           // Undefined/binary data
    Binary,          // Binary data
    Unicode,         // Unicode string
    Ifd,             // IFD offset
    Unknown(String), // Unknown format type
}

impl WritableTagsExtractor {
    pub fn new() -> Self {
        Self { tags: Vec::new() }
    }

    /// Parse writable tags from all ExifTool .pm files
    fn parse_writable_tags(&mut self, exiftool_path: &Path) -> Result<(), String> {
        let lib_path = exiftool_path.join("lib/Image/ExifTool");
        if !lib_path.exists() {
            return Err(format!(
                "ExifTool lib directory not found: {}",
                lib_path.display()
            ));
        }

        let entries = fs::read_dir(&lib_path)
            .map_err(|e| format!("Failed to read ExifTool lib directory: {}", e))?;

        let mut pm_files = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            // Only process .pm files
            if let Some(ext) = path.extension() {
                if ext == "pm" && path.is_file() {
                    if let Some(file_name) = path.file_name() {
                        if let Some(file_str) = file_name.to_str() {
                            pm_files.push((path.clone(), file_str.to_string()));
                        }
                    }
                }
            }
        }

        // Sort for consistent processing order
        pm_files.sort_by(|(_, a), (_, b)| a.cmp(b));

        println!("  Found {} .pm files to scan", pm_files.len());

        let mut total_writable_tags = 0;
        for (pm_path, pm_name) in pm_files {
            match self.parse_pm_file(&pm_path, &pm_name) {
                Ok(count) => {
                    if count > 0 {
                        println!("    {}: {} writable tags", pm_name, count);
                        total_writable_tags += count;
                    }
                }
                Err(e) => {
                    println!("    Warning: Failed to parse {}: {}", pm_name, e);
                }
            }
        }

        println!("  Total writable tags found: {}", total_writable_tags);
        Ok(())
    }

    /// Parse a single .pm file for writable tag information
    fn parse_pm_file(&mut self, pm_path: &Path, pm_name: &str) -> Result<usize, String> {
        let content = fs::read_to_string(pm_path)
            .map_err(|e| format!("Failed to read {}: {}", pm_name, e))?;

        let mut count = 0;

        // Extract group name from filename (e.g., "Canon.pm" -> "Canon")
        let group = pm_name.trim_end_matches(".pm").to_string();

        // Parse individual writable tags: tag_id => { ... Writable => 'format' ... }
        count += self.parse_individual_writable_tags(&content, &group, pm_name)?;

        // Parse table-level writability: WRITABLE => 1
        count += self.parse_table_level_writable(&content, &group, pm_name)?;

        // Parse write procedures: WRITE_PROC => \&Function
        count += self.parse_write_procedures(&content, &group, pm_name)?;

        Ok(count)
    }

    /// Parse individual tags with Writable => 'format' attributes
    fn parse_individual_writable_tags(
        &mut self,
        content: &str,
        group: &str,
        source_file: &str,
    ) -> Result<usize, String> {
        let mut count = 0;

        // Regex to match tag definitions with Writable attribute
        // Handles both hex (0x1234) and string ('TagName') tag IDs
        let tag_regex = Regex::new(r"(?s)(0x[0-9a-fA-F]+|'[^']+')?\s*=>\s*\{([^}]+)\}").unwrap();

        let name_regex = Regex::new(r"Name\s*=>\s*'([^']+)'").unwrap();
        let writable_regex = Regex::new(r"Writable\s*=>\s*'([^']+)'").unwrap();
        let write_group_regex = Regex::new(r"WriteGroup\s*=>\s*'([^']+)'").unwrap();

        for cap in tag_regex.captures_iter(content) {
            let tag_id_str = cap.get(1).map(|m| m.as_str());
            let tag_content = &cap[2];

            // Must have both Name and Writable to be a writable tag
            if let (Some(name_cap), Some(writable_cap)) = (
                name_regex.captures(tag_content),
                writable_regex.captures(tag_content),
            ) {
                let tag_name = name_cap[1].to_string();
                let writable_format_str = &writable_cap[1];
                let writable_format = self.parse_writable_format(writable_format_str);

                // Parse tag ID if present
                let (tag_id, tag_id_hex) = if let Some(id_str) = tag_id_str {
                    if let Some(hex_part) = id_str.strip_prefix("0x") {
                        // Hex tag ID
                        match u32::from_str_radix(hex_part, 16) {
                            Ok(id) => (Some(id), Some(id_str.to_string())),
                            Err(_) => (None, Some(id_str.to_string())),
                        }
                    } else {
                        // String tag ID (like XMP tags)
                        (None, Some(id_str.to_string()))
                    }
                } else {
                    (None, None)
                };

                // Determine write group (may override main group)
                let final_group = write_group_regex
                    .captures(tag_content)
                    .map(|cap| cap[1].to_string())
                    .unwrap_or_else(|| group.to_string());

                // Classify safety level
                let safety_level = self.classify_safety_level(
                    &tag_name,
                    &final_group,
                    tag_content,
                    &writable_format,
                );

                self.tags.push(WritableTagInfo {
                    tag_name,
                    tag_id,
                    tag_id_hex,
                    group: final_group,
                    writable_format,
                    write_proc: None, // Individual tags don't have WRITE_PROC
                    safety_level,
                    source_file: source_file.to_string(),
                });

                count += 1;
            }
        }

        Ok(count)
    }

    /// Parse table-level WRITABLE => 1 declarations
    fn parse_table_level_writable(
        &mut self,
        content: &str,
        _group: &str,
        source_file: &str,
    ) -> Result<usize, String> {
        let mut count = 0;

        // Look for table declarations with WRITABLE => 1
        let table_regex =
            Regex::new(r"%Image::ExifTool::([^:]+)::([^\s=]+)\s*=\s*\([^)]*WRITABLE\s*=>\s*1")
                .unwrap();

        for cap in table_regex.captures_iter(content) {
            let table_group = &cap[1];
            let table_name = &cap[2];

            // For table-level writability, we create a general entry
            // This indicates the entire table supports writing
            self.tags.push(WritableTagInfo {
                tag_name: format!("{}Table", table_name),
                tag_id: None,
                tag_id_hex: None,
                group: table_group.to_string(),
                writable_format: WritableFormat::Unknown("table".to_string()),
                write_proc: None,
                safety_level: SafetyLevel::Restricted, // Table-level is usually maker notes
                source_file: source_file.to_string(),
            });

            count += 1;
        }

        Ok(count)
    }

    /// Parse WRITE_PROC => \&Function declarations
    fn parse_write_procedures(
        &mut self,
        content: &str,
        group: &str,
        source_file: &str,
    ) -> Result<usize, String> {
        let mut count = 0;

        // Look for WRITE_PROC declarations
        let write_proc_regex = Regex::new(r"WRITE_PROC\s*=>\s*\\&([^,\s}]+)").unwrap();

        for cap in write_proc_regex.captures_iter(content) {
            let proc_name = &cap[1];

            // Create an entry for the write procedure
            self.tags.push(WritableTagInfo {
                tag_name: format!("WriteProcedure_{}", proc_name),
                tag_id: None,
                tag_id_hex: None,
                group: group.to_string(),
                writable_format: WritableFormat::Unknown("procedure".to_string()),
                write_proc: Some(proc_name.to_string()),
                safety_level: SafetyLevel::Dangerous, // Write procedures are complex
                source_file: source_file.to_string(),
            });

            count += 1;
        }

        Ok(count)
    }

    /// Parse writable format string into enum
    fn parse_writable_format(&self, format_str: &str) -> WritableFormat {
        match format_str {
            "string" => WritableFormat::String,
            "int8u" => WritableFormat::Int8u,
            "int16u" => WritableFormat::Int16u,
            "int32u" => WritableFormat::Int32u,
            "int8s" => WritableFormat::Int8s,
            "int16s" => WritableFormat::Int16s,
            "int32s" => WritableFormat::Int32s,
            "rational64u" => WritableFormat::Rational64u,
            "rational64s" => WritableFormat::Rational64s,
            "float" => WritableFormat::Float,
            "double" => WritableFormat::Double,
            "undef" => WritableFormat::Undef,
            "binary" => WritableFormat::Binary,
            "unicode" => WritableFormat::Unicode,
            "ifd" => WritableFormat::Ifd,
            _ => WritableFormat::Unknown(format_str.to_string()),
        }
    }

    /// Classify safety level based on tag patterns
    /// Following ExifTool's approach: ref lib/Image/ExifTool/Writer.pl:2241-2400
    fn classify_safety_level(
        &self,
        tag_name: &str,
        group: &str,
        tag_content: &str,
        format: &WritableFormat,
    ) -> SafetyLevel {
        // Dangerous tags (file structure, never write programmatically)
        if tag_name.contains("Offset")
            || tag_name.contains("Length")
            || tag_name.contains("Size")
            || tag_name.contains("Count")
            || tag_name.contains("Pointer")
            || tag_content.contains("Protected => 1")
            || tag_content.contains("Permanent => 1")
        {
            return SafetyLevel::Dangerous;
        }

        // Restricted tags (maker notes, binary data - preserve structure only)
        if group != "EXIF" && group != "ExifIFD" && group != "GPS" && group != "InteropIFD"
            || matches!(format, WritableFormat::Binary | WritableFormat::Undef)
            || tag_content.contains("Binary => 1")
            || tag_content.contains("SubDirectory")
            || tag_content.contains("ProcessBinaryData")
        {
            return SafetyLevel::Restricted;
        }

        // Safe tags (basic metadata)
        SafetyLevel::Safe
    }

    /// Generate Rust code for the writable tags registry
    fn generate_rust_code(&self) -> Result<String, String> {
        let mut code = String::new();

        // Header with ExifTool attribution
        code.push_str("//! Auto-generated writable tags registry from ExifTool\n");
        code.push_str("//!\n");
        code.push_str("//! EXIFTOOL-SOURCE: lib/Image/ExifTool/*.pm (all modules)\n");
        code.push_str("//! This file is auto-generated by exiftool_sync extract writable-tags.\n");
        code.push_str("//! Do not edit manually.\n\n");

        code.push_str("#![doc = \"EXIFTOOL-SOURCE: lib/Image/ExifTool/*.pm\"]\n\n");

        // Imports
        code.push_str("use lazy_static::lazy_static;\n");
        code.push_str("use std::collections::HashMap;\n\n");

        // Enums
        code.push_str("/// Safety level for write operations\n");
        code.push_str("#[derive(Debug, Clone, PartialEq, Eq)]\n");
        code.push_str("pub enum SafetyLevel {\n");
        code.push_str("    /// Basic metadata - always safe to write\n");
        code.push_str("    Safe,\n");
        code.push_str("    /// Maker notes - preserve existing structure only\n");
        code.push_str("    Restricted,\n");
        code.push_str("    /// File structure - never write programmatically\n");
        code.push_str("    Dangerous,\n");
        code.push_str("}\n\n");

        code.push_str("/// Writable format types from ExifTool\n");
        code.push_str("#[derive(Debug, Clone, PartialEq, Eq)]\n");
        code.push_str("pub enum WritableFormat {\n");
        code.push_str("    String,\n");
        code.push_str("    Int8u,\n");
        code.push_str("    Int16u,\n");
        code.push_str("    Int32u,\n");
        code.push_str("    Int8s,\n");
        code.push_str("    Int16s,\n");
        code.push_str("    Int32s,\n");
        code.push_str("    Rational64u,\n");
        code.push_str("    Rational64s,\n");
        code.push_str("    Float,\n");
        code.push_str("    Double,\n");
        code.push_str("    Undef,\n");
        code.push_str("    Binary,\n");
        code.push_str("    Unicode,\n");
        code.push_str("    Ifd,\n");
        code.push_str("    Unknown(String),\n");
        code.push_str("}\n\n");

        // Main struct
        code.push_str("/// Information about a writable tag\n");
        code.push_str("#[derive(Debug, Clone)]\n");
        code.push_str("pub struct WritableTagInfo {\n");
        code.push_str("    pub tag_name: String,\n");
        code.push_str("    pub tag_id: Option<u32>,\n");
        code.push_str("    pub tag_id_hex: Option<String>,\n");
        code.push_str("    pub group: String,\n");
        code.push_str("    pub writable_format: WritableFormat,\n");
        code.push_str("    pub write_proc: Option<String>,\n");
        code.push_str("    pub safety_level: SafetyLevel,\n");
        code.push_str("    pub source_file: String,\n");
        code.push_str("}\n\n");

        // Generate lazy_static registry
        code.push_str("lazy_static! {\n");
        code.push_str("    /// Registry of all writable tags from ExifTool\n");
        code.push_str("    pub static ref WRITABLE_TAGS: HashMap<String, WritableTagInfo> = {\n");
        code.push_str("        let mut map = HashMap::new();\n\n");

        // Add all tags
        for tag in &self.tags {
            code.push_str(&format!(
                "        map.insert(\"{}\".to_string(), WritableTagInfo {{\n",
                tag.tag_name
            ));
            code.push_str(&format!(
                "            tag_name: \"{}\".to_string(),\n",
                tag.tag_name
            ));

            if let Some(id) = tag.tag_id {
                code.push_str(&format!("            tag_id: Some({}),\n", id));
            } else {
                code.push_str("            tag_id: None,\n");
            }

            if let Some(hex) = &tag.tag_id_hex {
                code.push_str(&format!(
                    "            tag_id_hex: Some(\"{}\".to_string()),\n",
                    hex
                ));
            } else {
                code.push_str("            tag_id_hex: None,\n");
            }

            code.push_str(&format!(
                "            group: \"{}\".to_string(),\n",
                tag.group
            ));
            code.push_str(&format!(
                "            writable_format: {},\n",
                self.format_to_rust(&tag.writable_format)
            ));

            if let Some(proc) = &tag.write_proc {
                code.push_str(&format!(
                    "            write_proc: Some(\"{}\".to_string()),\n",
                    proc
                ));
            } else {
                code.push_str("            write_proc: None,\n");
            }

            code.push_str(&format!(
                "            safety_level: {},\n",
                self.safety_level_to_rust(&tag.safety_level)
            ));
            code.push_str(&format!(
                "            source_file: \"{}\".to_string(),\n",
                tag.source_file
            ));
            code.push_str("        });\n\n");
        }

        code.push_str("        map\n");
        code.push_str("    };\n");
        code.push_str("}\n\n");

        // Helper functions
        code.push_str("/// Check if a tag is writable\n");
        code.push_str("pub fn is_tag_writable(tag_name: &str) -> bool {\n");
        code.push_str("    WRITABLE_TAGS.contains_key(tag_name)\n");
        code.push_str("}\n\n");

        code.push_str("/// Get writable tag information\n");
        code.push_str(
            "pub fn get_writable_tag_info(tag_name: &str) -> Option<&WritableTagInfo> {\n",
        );
        code.push_str("    WRITABLE_TAGS.get(tag_name)\n");
        code.push_str("}\n\n");

        code.push_str("/// Get tag safety level\n");
        code.push_str("pub fn get_tag_safety_level(tag_name: &str) -> Option<&SafetyLevel> {\n");
        code.push_str("    WRITABLE_TAGS.get(tag_name).map(|info| &info.safety_level)\n");
        code.push_str("}\n\n");

        code.push_str("/// Get all writable tags by safety level\n");
        code.push_str(
            "pub fn get_tags_by_safety_level(level: &SafetyLevel) -> Vec<&WritableTagInfo> {\n",
        );
        code.push_str(
            "    WRITABLE_TAGS.values().filter(|tag| tag.safety_level == *level).collect()\n",
        );
        code.push_str("}\n\n");

        code.push_str("/// Get writable tags count\n");
        code.push_str("pub fn get_writable_tags_count() -> usize {\n");
        code.push_str("    WRITABLE_TAGS.len()\n");
        code.push_str("}\n");

        Ok(code)
    }

    /// Convert WritableFormat to Rust code
    fn format_to_rust(&self, format: &WritableFormat) -> String {
        match format {
            WritableFormat::String => "WritableFormat::String".to_string(),
            WritableFormat::Int8u => "WritableFormat::Int8u".to_string(),
            WritableFormat::Int16u => "WritableFormat::Int16u".to_string(),
            WritableFormat::Int32u => "WritableFormat::Int32u".to_string(),
            WritableFormat::Int8s => "WritableFormat::Int8s".to_string(),
            WritableFormat::Int16s => "WritableFormat::Int16s".to_string(),
            WritableFormat::Int32s => "WritableFormat::Int32s".to_string(),
            WritableFormat::Rational64u => "WritableFormat::Rational64u".to_string(),
            WritableFormat::Rational64s => "WritableFormat::Rational64s".to_string(),
            WritableFormat::Float => "WritableFormat::Float".to_string(),
            WritableFormat::Double => "WritableFormat::Double".to_string(),
            WritableFormat::Undef => "WritableFormat::Undef".to_string(),
            WritableFormat::Binary => "WritableFormat::Binary".to_string(),
            WritableFormat::Unicode => "WritableFormat::Unicode".to_string(),
            WritableFormat::Ifd => "WritableFormat::Ifd".to_string(),
            WritableFormat::Unknown(s) => format!("WritableFormat::Unknown(\"{}\".to_string())", s),
        }
    }

    /// Convert SafetyLevel to Rust code
    fn safety_level_to_rust(&self, level: &SafetyLevel) -> String {
        match level {
            SafetyLevel::Safe => "SafetyLevel::Safe".to_string(),
            SafetyLevel::Restricted => "SafetyLevel::Restricted".to_string(),
            SafetyLevel::Dangerous => "SafetyLevel::Dangerous".to_string(),
        }
    }

    /// Generate stub code when no data is available
    fn generate_stub_code(&self) -> Result<String, String> {
        let mut code = String::new();

        code.push_str("//! Auto-generated stub writable tags registry\n");
        code.push_str("//!\n");
        code.push_str("//! EXIFTOOL-SOURCE: lib/Image/ExifTool/*.pm\n");
        code.push_str("//! This is a stub file generated when no data was found.\n");
        code.push_str(
            "//! Regenerate with: cargo run --bin exiftool_sync extract writable-tags\n\n",
        );

        code.push_str("#![doc = \"EXIFTOOL-SOURCE: lib/Image/ExifTool/*.pm\"]\n\n");

        code.push_str("use lazy_static::lazy_static;\n");
        code.push_str("use std::collections::HashMap;\n\n");

        // Stub enums and structs
        code.push_str("#[derive(Debug, Clone, PartialEq, Eq)]\n");
        code.push_str("pub enum SafetyLevel { Safe, Restricted, Dangerous }\n\n");

        code.push_str("#[derive(Debug, Clone, PartialEq, Eq)]\n");
        code.push_str("pub enum WritableFormat { String, Unknown(String) }\n\n");

        code.push_str("#[derive(Debug, Clone)]\n");
        code.push_str("pub struct WritableTagInfo {\n");
        code.push_str("    pub tag_name: String,\n");
        code.push_str("    pub tag_id: Option<u32>,\n");
        code.push_str("    pub tag_id_hex: Option<String>,\n");
        code.push_str("    pub group: String,\n");
        code.push_str("    pub writable_format: WritableFormat,\n");
        code.push_str("    pub write_proc: Option<String>,\n");
        code.push_str("    pub safety_level: SafetyLevel,\n");
        code.push_str("    pub source_file: String,\n");
        code.push_str("}\n\n");

        // Empty registry
        code.push_str("lazy_static! {\n");
        code.push_str("    pub static ref WRITABLE_TAGS: HashMap<String, WritableTagInfo> = HashMap::new();\n");
        code.push_str("}\n\n");

        // Stub functions
        code.push_str("pub fn is_tag_writable(_tag_name: &str) -> bool { false }\n");
        code.push_str(
            "pub fn get_writable_tag_info(_tag_name: &str) -> Option<&WritableTagInfo> { None }\n",
        );
        code.push_str(
            "pub fn get_tag_safety_level(_tag_name: &str) -> Option<&SafetyLevel> { None }\n",
        );
        code.push_str("pub fn get_tags_by_safety_level(_level: &SafetyLevel) -> Vec<&WritableTagInfo> { Vec::new() }\n");
        code.push_str("pub fn get_writable_tags_count() -> usize { 0 }\n");

        Ok(code)
    }

    /// Write output to file
    fn write_output(&self, code: &str) -> Result<(), String> {
        // Create output directory
        let output_dir = Path::new("src/write");
        fs::create_dir_all(output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;

        let filepath = output_dir.join("writable_tags.rs");
        let mut file = fs::File::create(&filepath)
            .map_err(|e| format!("Failed to create {}: {}", filepath.display(), e))?;

        file.write_all(code.as_bytes())
            .map_err(|e| format!("Failed to write to {}: {}", filepath.display(), e))?;

        file.flush()
            .map_err(|e| format!("Failed to flush {}: {}", filepath.display(), e))?;

        println!("  Generated {}", filepath.display());
        Ok(())
    }
}

impl Extractor for WritableTagsExtractor {
    fn extract(&self, exiftool_path: &Path) -> Result<(), String> {
        println!("Extracting writable tags from ExifTool source...");

        let mut extractor = Self::new();

        // Always attempt extraction (never fail completely)
        match extractor.parse_writable_tags(exiftool_path) {
            Ok(()) => {
                println!("  Found {} writable tags", extractor.tags.len());
            }
            Err(e) => {
                println!("  Warning: Failed to parse: {}", e);
                println!("  - Generating stub implementation");
            }
        }

        // Always generate code (real or stub)
        let code = if extractor.tags.is_empty() {
            extractor.generate_stub_code()?
        } else {
            extractor.generate_rust_code()?
        };

        // Always write output
        extractor.write_output(&code)?;

        // Clear completion message
        println!("Writable tags extraction completed successfully");
        if extractor.tags.is_empty() {
            println!("  - Using stub implementation (no data found)");
        } else {
            let safe_count = extractor
                .tags
                .iter()
                .filter(|t| t.safety_level == SafetyLevel::Safe)
                .count();
            let restricted_count = extractor
                .tags
                .iter()
                .filter(|t| t.safety_level == SafetyLevel::Restricted)
                .count();
            let dangerous_count = extractor
                .tags
                .iter()
                .filter(|t| t.safety_level == SafetyLevel::Dangerous)
                .count();

            println!(
                "  - Generated {} tags: {} safe, {} restricted, {} dangerous",
                extractor.tags.len(),
                safe_count,
                restricted_count,
                dangerous_count
            );
        }

        Ok(())
    }
}
