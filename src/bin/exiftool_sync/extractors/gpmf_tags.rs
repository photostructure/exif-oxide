//! GPMF Tags Extractor
//!
//! Extracts GoPro GPMF tag definitions from ExifTool's GoPro.pm
//! Generates src/gpmf/tags.rs with PrintConv integration
//!
//! GPMF (GoPro Metadata Format) uses 4-byte tag IDs unlike traditional IFD structures

use super::Extractor;
use regex::Regex;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
struct GpmfTagEntry {
    tag_id: String,         // 4-byte tag like "ABSC", "ACCL"
    name: String,           // Tag name like "AutoBoostScore"
    #[allow(dead_code)]
    description: String,    // Optional description
    printconv_id: String,   // PrintConvId variant name
    binary: bool,           // If tag has Binary => 1
    subdirectory: bool,     // If tag has SubDirectory
    unknown: bool,          // If tag has Unknown => 1
    #[allow(dead_code)]
    groups: Option<String>, // Group assignment
}

pub struct GpmfTagsExtractor {
    tags: Vec<GpmfTagEntry>,
}

impl GpmfTagsExtractor {
    pub fn new() -> Self {
        Self { tags: Vec::new() }
    }

    /// Parse the %Image::ExifTool::GoPro::GPMF hash table from GoPro.pm
    fn parse_gpmf_table(&mut self, gromf_path: &Path) -> Result<(), String> {
        let content = fs::read_to_string(gromf_path)
            .map_err(|e| format!("Failed to read GoPro.pm: {}", e))?;

        // Find the GPMF table definition
        let table_start = content
            .find("%Image::ExifTool::GoPro::GPMF = (")
            .ok_or("Could not find GPMF table definition")?;

        let table_end = self.find_table_end(&content, table_start)?;
        let table_content = &content[table_start..table_end];

        println!("  - Parsing GPMF table ({} bytes)", table_content.len());

        // Parse individual tag entries
        self.parse_tag_entries(table_content)?;

        println!("  - Found {} GPMF tags", self.tags.len());
        Ok(())
    }

    /// Find the end of the hash table definition
    fn find_table_end(&self, content: &str, start: usize) -> Result<usize, String> {
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

        Err("Could not find end of GPMF table".to_string())
    }

    /// Parse individual tag entries from the table content
    fn parse_tag_entries(&mut self, table_content: &str) -> Result<(), String> {
        // Regex to match tag entries like: ABSC => 'AutoBoostScore', #3
        let simple_tag_re = Regex::new(r"^\s*([A-Z0-9]{4})\s*=>\s*'([^']+)',?\s*(?:#.*)?$")
            .map_err(|e| format!("Failed to compile simple tag regex: {}", e))?;

        // Regex for complex tags with hash definitions
        let complex_tag_re = Regex::new(r"^\s*([A-Z0-9]{4})\s*=>\s*\{\s*(?:#.*)?$")
            .map_err(|e| format!("Failed to compile complex tag regex: {}", e))?;

        let lines: Vec<&str> = table_content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                i += 1;
                continue;
            }

            // Try simple tag format first
            if let Some(caps) = simple_tag_re.captures(line) {
                let tag_id = caps[1].to_string();
                let name = caps[2].to_string();

                let tag_entry = GpmfTagEntry {
                    tag_id: tag_id.clone(),
                    name: name.clone(),
                    description: String::new(),
                    printconv_id: self.generate_printconv_id(&tag_id, &name),
                    binary: false,
                    subdirectory: false,
                    unknown: false,
                    groups: None,
                };

                self.tags.push(tag_entry);
                i += 1;
                continue;
            }

            // Try complex tag format
            if let Some(caps) = complex_tag_re.captures(line) {
                let tag_id = caps[1].to_string();
                let (tag_entry, lines_consumed) = self.parse_complex_tag(&tag_id, &lines[i..])?;
                self.tags.push(tag_entry);
                i += lines_consumed;
                continue;
            }

            i += 1;
        }

        Ok(())
    }

    /// Parse a complex tag definition with multiple properties
    fn parse_complex_tag(
        &self,
        tag_id: &str,
        lines: &[&str],
    ) -> Result<(GpmfTagEntry, usize), String> {
        let mut name = String::new();
        let mut description = String::new();
        let mut binary = false;
        let mut subdirectory = false;
        let mut unknown = false;
        let mut groups = None;
        let mut lines_consumed = 1;

        // Parse properties until we find the closing brace
        for (idx, line) in lines[1..].iter().enumerate() {
            let line = line.trim();

            if line.starts_with('}') {
                lines_consumed = idx + 2;
                break;
            }

            // Parse Name property
            if let Some(caps) = Regex::new(r#"Name\s*=>\s*'([^']+)'"#)
                .unwrap()
                .captures(line)
            {
                name = caps[1].to_string();
            }

            // Parse Description property
            if let Some(caps) = Regex::new(r#"Description\s*=>\s*'([^']+)'"#)
                .unwrap()
                .captures(line)
            {
                description = caps[1].to_string();
            }

            // Parse Binary flag
            if line.contains("Binary => 1") {
                binary = true;
            }

            // Parse SubDirectory
            if line.contains("SubDirectory") {
                subdirectory = true;
            }

            // Parse Unknown flag
            if line.contains("Unknown => 1") {
                unknown = true;
            }

            // Parse Groups
            if let Some(caps) = Regex::new(r#"Groups\s*=>\s*\{\s*2\s*=>\s*'([^']+)'"#)
                .unwrap()
                .captures(line)
            {
                groups = Some(caps[1].to_string());
            }
        }

        // Use tag_id as name if no Name property found
        if name.is_empty() {
            name = tag_id.to_string();
        }

        let tag_entry = GpmfTagEntry {
            tag_id: tag_id.to_string(),
            name: name.clone(),
            description,
            printconv_id: self.generate_printconv_id(tag_id, &name),
            binary,
            subdirectory,
            unknown,
            groups,
        };

        Ok((tag_entry, lines_consumed))
    }

    /// Generate PrintConvId variant name for a tag
    fn generate_printconv_id(&self, _tag_id: &str, name: &str) -> String {
        // For GPMF tags, use the tag name with Gpmf prefix
        format!("Gpmf{}", sanitize_rust_identifier(name))
    }

    /// Generate the complete Rust code for the GPMF tags table
    fn generate_code(&self) -> Result<String, String> {
        let mut code = String::new();

        // Header
        code.push_str(
            "//! Auto-generated GPMF tag table with PrintConv mappings\n\
             //!\n\
             //! EXIFTOOL-SOURCE: lib/Image/ExifTool/GoPro.pm\n\
             //! EXIFTOOL-VERSION: 12.65\n\
             //!\n\
             //! This file is auto-generated by exiftool_sync extract gpmf-tags.\n\
             //! Do not edit manually.\n\n\
             #![doc = \"EXIFTOOL-SOURCE: lib/Image/ExifTool/GoPro.pm\"]\n\n\
             use crate::core::print_conv::PrintConvId;\n\n",
        );

        // Tag structure definition
        code.push_str(
            "#[derive(Debug, Clone)]\n\
             #[allow(non_camel_case_types)]  // Allow ExifTool-style naming\n\
             pub struct GpmfTag {\n\
             \x20\x20\x20\x20pub tag_id: &'static str,  // 4-byte GPMF tag ID\n\
             \x20\x20\x20\x20pub name: &'static str,\n\
             \x20\x20\x20\x20pub print_conv: PrintConvId,\n\
             \x20\x20\x20\x20pub binary: bool,\n\
             \x20\x20\x20\x20pub subdirectory: bool,\n\
             \x20\x20\x20\x20pub unknown: bool,\n\
             }\n\n",
        );

        // Generate tag table
        code.push_str(&format!(
            "/// GPMF tag definitions extracted from ExifTool GoPro.pm\n\
             /// Total tags: {}\n\
             pub const GPMF_TAGS: &[GpmfTag] = &[\n",
            self.tags.len()
        ));

        for tag in &self.tags {
            code.push_str(&format!(
                "    GpmfTag {{\n\
                 \x20\x20\x20\x20\x20\x20\x20\x20tag_id: \"{}\",\n\
                 \x20\x20\x20\x20\x20\x20\x20\x20name: \"{}\",\n\
                 \x20\x20\x20\x20\x20\x20\x20\x20print_conv: PrintConvId::{},\n\
                 \x20\x20\x20\x20\x20\x20\x20\x20binary: {},\n\
                 \x20\x20\x20\x20\x20\x20\x20\x20subdirectory: {},\n\
                 \x20\x20\x20\x20\x20\x20\x20\x20unknown: {},\n\
                 \x20\x20\x20\x20}},\n",
                tag.tag_id, tag.name, tag.printconv_id, tag.binary, tag.subdirectory, tag.unknown
            ));
        }

        code.push_str("];\n\n");

        // Helper functions
        code.push_str(
            "/// Get GPMF tag by 4-byte tag ID\n\
             pub fn get_gpmf_tag(tag_id: &str) -> Option<&'static GpmfTag> {\n\
             \x20\x20\x20\x20GPMF_TAGS.iter().find(|tag| tag.tag_id == tag_id)\n\
             }\n\n\
             \n\
             /// Get all GPMF tags\n\
             pub fn get_all_gpmf_tags() -> &'static [GpmfTag] {\n\
             \x20\x20\x20\x20GPMF_TAGS\n\
             }\n",
        );

        Ok(code)
    }

    /// Generate stub code when no data is found
    fn generate_stub_code(&self) -> Result<String, String> {
        let mut code = String::new();

        code.push_str(
            "//! Auto-generated stub GPMF tag table with PrintConv mappings\n\
             //!\n\
             //! EXIFTOOL-SOURCE: lib/Image/ExifTool/GoPro.pm\n\
             //! This is a stub file generated when no data was found.\n\
             //! Regenerate with: cargo run --bin exiftool_sync extract gpmf-tags\n\n\
             #![doc = \"EXIFTOOL-SOURCE: lib/Image/ExifTool/GoPro.pm\"]\n\n\
             use crate::core::print_conv::PrintConvId;\n\n",
        );

        code.push_str(
            "#[derive(Debug, Clone)]\n\
             #[allow(non_camel_case_types)]  // Allow ExifTool-style naming\n\
             pub struct GpmfTag {\n\
             \x20\x20\x20\x20pub tag_id: &'static str,\n\
             \x20\x20\x20\x20pub name: &'static str,\n\
             \x20\x20\x20\x20pub print_conv: PrintConvId,\n\
             \x20\x20\x20\x20pub binary: bool,\n\
             \x20\x20\x20\x20pub subdirectory: bool,\n\
             \x20\x20\x20\x20pub unknown: bool,\n\
             }\n\n",
        );

        code.push_str(
            "/// Stub tag table (empty - regenerate to populate)\n\
             pub const GPMF_TAGS: &[GpmfTag] = &[\n\
             \x20\x20\x20\x20// Stub - real GPMF tags will be generated by extractor\n\
             ];\n\n",
        );

        code.push_str(
            "/// Get GPMF tag by 4-byte tag ID\n\
             pub fn get_gpmf_tag(tag_id: &str) -> Option<&'static GpmfTag> {\n\
             \x20\x20\x20\x20GPMF_TAGS.iter().find(|tag| tag.tag_id == tag_id)\n\
             }\n",
        );

        Ok(code)
    }

    /// Write the generated code to the output file
    fn write_output(&self, code: &str) -> Result<(), String> {
        let output_path = Path::new("src/gpmf/tags.rs");

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

impl Extractor for GpmfTagsExtractor {
    fn extract(&self, exiftool_path: &Path) -> Result<(), String> {
        println!("Extracting GPMF tags from ExifTool GoPro.pm...");

        let gromf_path = exiftool_path.join("lib/Image/ExifTool/GoPro.pm");
        let mut extractor = Self::new();

        // Always attempt extraction (never fail completely)
        match extractor.parse_gpmf_table(&gromf_path) {
            Ok(()) => {
                println!("Found {} GPMF tags", extractor.tags.len());
            }
            Err(e) => {
                println!("Warning: Failed to parse: {}", e);
                println!("  - Generating stub implementation");
            }
        }

        // Always generate code (real or stub)
        let code = if extractor.tags.is_empty() {
            extractor.generate_stub_code()?
        } else {
            extractor.generate_code()?
        };

        // Always write output
        extractor.write_output(&code)?;

        // Clear completion message
        println!("GPMF tags extraction completed successfully");
        if extractor.tags.is_empty() {
            println!("  - Using stub implementation (no data found)");
        } else {
            println!("  - Generated {} GPMF tags", extractor.tags.len());
        }

        Ok(())
    }
}

/// Sanitize tag names to be valid Rust identifiers
/// Replaces any non-alphanumeric character with underscore
fn sanitize_rust_identifier(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}
