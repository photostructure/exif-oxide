//! Extractor for composite binary tag definitions from ExifTool
//!
//! This extractor parses ExifTool's composite tag definitions to identify binary image tags
//! like ThumbnailImage, PreviewImage, JpgFromRaw that are constructed from offset/length pairs.

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Exif.pm (Composite tags 4858-5500)"]
#![doc = "EXIFTOOL-SOURCE: exiftool (ConvertBinary function 3891-3920)"]
#![allow(dead_code)]

use super::Extractor;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

pub struct BinaryTagsExtractor {
    composite_tags: Vec<CompositeTag>,
}

#[derive(Debug, Clone)]
struct CompositeTag {
    name: String,
    groups: HashMap<u8, String>,
    requires: Vec<RequiredTag>,
    desires: Vec<RequiredTag>,
    raw_conv: String,
    source_file: String,
    line_start: usize,
    line_end: usize,
    is_binary: bool,
}

#[derive(Debug, Clone)]
struct RequiredTag {
    index: u8,
    tag_name: String,
}

impl BinaryTagsExtractor {
    pub fn new() -> Self {
        Self {
            composite_tags: Vec::new(),
        }
    }

    /// Parse ExifTool's Exif.pm for composite tag definitions
    fn parse_composite_tags(&mut self, exiftool_path: &Path) -> Result<(), String> {
        let exif_pm_path = exiftool_path.join("lib/Image/ExifTool/Exif.pm");
        let content = fs::read_to_string(&exif_pm_path)
            .map_err(|e| format!("Failed to read Exif.pm: {}", e))?;

        println!("Parsing composite tags from Exif.pm...");

        // Find the %Composite hash definition
        let composite_start = content
            .find("%Image::ExifTool::Exif::Composite")
            .ok_or("Could not find %Composite hash in Exif.pm")?;

        // Parse composite tag definitions
        self.parse_composite_hash(&content, composite_start, "lib/Image/ExifTool/Exif.pm")?;

        println!(
            "Found {} composite tags, {} are binary tags",
            self.composite_tags.len(),
            self.composite_tags.iter().filter(|t| t.is_binary).count()
        );

        Ok(())
    }

    /// Parse the %Composite hash structure from Perl
    fn parse_composite_hash(
        &mut self,
        content: &str,
        start_pos: usize,
        source_file: &str,
    ) -> Result<(), String> {
        let lines: Vec<&str> = content.lines().collect();
        let start_line = content[..start_pos].lines().count();

        let mut current_tag: Option<CompositeTag> = None;
        let mut brace_depth = 0;
        let mut in_composite_hash = false;
        let mut tag_brace_depth = 0;
        let mut collecting_rawconv = false;
        let mut rawconv_content = String::new();

        for (line_idx, line) in lines.iter().enumerate().skip(start_line) {
            let trimmed = line.trim();

            // Skip comments and empty lines
            if trimmed.starts_with('#') || trimmed.is_empty() {
                continue;
            }

            // Track opening of %Composite hash
            if trimmed.contains("%Image::ExifTool::Exif::Composite") {
                in_composite_hash = true;
                brace_depth = 0;
                continue;
            }

            if !in_composite_hash {
                continue;
            }

            // Track brace depth
            brace_depth += trimmed.matches('{').count() as i32;
            brace_depth -= trimmed.matches('}').count() as i32;

            // End of composite hash
            if brace_depth < 0 {
                break;
            }

            // Check if this is a top-level tag definition (not a property)
            if brace_depth == 1 && trimmed.contains("=>") && trimmed.ends_with("{") {
                // This might be a tag definition, but only if it's not a known property
                if let Some(tag_name) = self.extract_tag_name(trimmed) {
                    // Filter out known properties that aren't tags
                    if ![
                        "Groups",
                        "Require",
                        "Desire",
                        "WriteAlso",
                        "Notes",
                        "Writable",
                        "WriteCheck",
                        "DelCheck",
                    ]
                    .contains(&tag_name.as_str())
                    {
                        // Save previous tag if it exists
                        if let Some(mut tag) = current_tag.take() {
                            // Finalize previous tag
                            if collecting_rawconv {
                                tag.raw_conv = rawconv_content.clone();
                                if tag.raw_conv.contains("ExtractImage") {
                                    tag.is_binary = true;
                                    // Binary tag found
                                }
                            }
                            self.composite_tags.push(tag);
                        }

                        // Found composite tag
                        current_tag = Some(CompositeTag {
                            name: tag_name,
                            groups: HashMap::new(),
                            requires: Vec::new(),
                            desires: Vec::new(),
                            raw_conv: String::new(),
                            source_file: source_file.to_string(),
                            line_start: line_idx + 1,
                            line_end: line_idx + 1,
                            is_binary: false,
                        });
                        tag_brace_depth = brace_depth;
                        collecting_rawconv = false;
                        rawconv_content.clear();
                    }
                }
            }

            // Parse tag properties
            if let Some(ref mut tag) = current_tag {
                tag.line_end = line_idx + 1;

                if trimmed.starts_with("Groups") {
                    self.parse_groups_block(&lines[line_idx..], &mut tag.groups)?;
                } else if trimmed.starts_with("Require") {
                    self.parse_require_desire_block(&lines[line_idx..], &mut tag.requires)?;
                } else if trimmed.starts_with("Desire") {
                    self.parse_require_desire_block(&lines[line_idx..], &mut tag.desires)?;
                } else if trimmed.starts_with("RawConv") {
                    collecting_rawconv = true;
                    rawconv_content = self.extract_perl_string(trimmed)?;
                } else if collecting_rawconv {
                    // Continue collecting multi-line RawConv
                    rawconv_content.push(' ');
                    rawconv_content.push_str(trimmed);
                    // Check if this line ends the RawConv (contains closing brace or comma)
                    if trimmed.contains("},")
                        || (trimmed.contains("}") && brace_depth <= tag_brace_depth)
                    {
                        collecting_rawconv = false;
                        tag.raw_conv = rawconv_content.clone();
                        if tag.raw_conv.contains("ExtractImage") {
                            tag.is_binary = true;
                        }
                    }
                }
            }
        }

        // Save final tag
        if let Some(mut tag) = current_tag {
            if collecting_rawconv {
                tag.raw_conv = rawconv_content;
                if tag.raw_conv.contains("ExtractImage") {
                    tag.is_binary = true;
                }
            }
            self.composite_tags.push(tag);
        }

        Ok(())
    }

    /// Extract tag name from a line like "ThumbnailImage => {"
    fn extract_tag_name(&self, line: &str) -> Option<String> {
        let re = Regex::new(r"(\w+)\s*=>\s*\{").ok()?;
        re.captures(line)?.get(1).map(|m| m.as_str().to_string())
    }

    /// Parse Groups => { 0 => 'EXIF', 1 => 'IFD1', 2 => 'Preview' }
    fn parse_groups(&self, line: &str, groups: &mut HashMap<u8, String>) -> Result<(), String> {
        let re = Regex::new(r"(\d+)\s*=>\s*'([^']+)'").unwrap();

        for cap in re.captures_iter(line) {
            let group_num: u8 = cap[1]
                .parse()
                .map_err(|_| format!("Invalid group number: {}", &cap[1]))?;
            let group_name = cap[2].to_string();
            groups.insert(group_num, group_name);
        }

        Ok(())
    }

    /// Parse multi-line Groups block: Groups => { 0 => 'EXIF', 1 => 'IFD1', 2 => 'Preview' }
    fn parse_groups_block(
        &self,
        lines: &[&str],
        groups: &mut HashMap<u8, String>,
    ) -> Result<(), String> {
        let mut content = String::new();
        let mut brace_count = 0;

        for line in lines {
            let trimmed = line.trim();
            content.push_str(trimmed);
            content.push(' ');

            brace_count += trimmed.matches('{').count() as i32;
            brace_count -= trimmed.matches('}').count() as i32;

            if brace_count <= 0 {
                break;
            }
        }

        self.parse_groups(&content, groups)
    }

    /// Parse multi-line Require/Desire block: Require => { 0 => 'TagName', 1 => 'OtherTag' }
    fn parse_require_desire_block(
        &self,
        lines: &[&str],
        tags: &mut Vec<RequiredTag>,
    ) -> Result<(), String> {
        let mut content = String::new();
        let mut brace_count = 0;

        for line in lines {
            let trimmed = line.trim();
            content.push_str(trimmed);
            content.push(' ');

            brace_count += trimmed.matches('{').count() as i32;
            brace_count -= trimmed.matches('}').count() as i32;

            if brace_count <= 0 {
                break;
            }
        }

        self.parse_require_desire(&content, tags)
    }

    /// Parse Require/Desire => { 0 => 'TagName', 1 => 'OtherTag' }
    fn parse_require_desire(
        &self,
        content: &str,
        tags: &mut Vec<RequiredTag>,
    ) -> Result<(), String> {
        let re = Regex::new(r"(\d+)\s*=>\s*'([^']+)'").unwrap();

        for cap in re.captures_iter(content) {
            let index: u8 = cap[1]
                .parse()
                .map_err(|_| format!("Invalid tag index: {}", &cap[1]))?;
            let tag_name = cap[2].to_string();
            tags.push(RequiredTag { index, tag_name });
        }

        Ok(())
    }

    /// Extract Perl string from RawConv => q{ ... } or RawConv => '...'
    fn extract_perl_string(&self, line: &str) -> Result<String, String> {
        if let Some(start) = line.find("q{") {
            // Multi-line q{...} string - this is simplified, real implementation
            // would need to handle proper brace matching
            if let Some(end) = line.rfind('}') {
                return Ok(line[start + 2..end].to_string());
            }
        } else if let Some(start) = line.find('\'') {
            if let Some(end) = line.rfind('\'') {
                if end > start {
                    return Ok(line[start + 1..end].to_string());
                }
            }
        }

        // Fallback - just return the line without RawConv prefix
        if let Some(arrow_pos) = line.find("=>") {
            Ok(line[arrow_pos + 2..].trim().to_string())
        } else {
            Ok(String::new())
        }
    }

    /// Map tag names to known tag IDs
    fn get_tag_id(&self, tag_name: &str) -> u16 {
        match tag_name {
            "ThumbnailOffset" => 0x0201,
            "ThumbnailLength" => 0x0202,
            "PreviewImageStart" => 0x0111, // StripOffsets in many contexts
            "PreviewImageLength" => 0x0117, // StripByteCounts in many contexts
            "JpgFromRawStart" => 0x0111,   // Same as PreviewImageStart in some contexts
            "JpgFromRawLength" => 0x0117,  // Same as PreviewImageLength in some contexts
            "PreviewImageValid" => 0x0000, // Placeholder - not a real EXIF tag
            _ => 0x0000,                   // Unknown tag
        }
    }

    /// Generate stub code when no tags are found
    fn generate_stub_code(&self) -> Result<String, String> {
        let mut code = String::new();

        code.push_str("//! Auto-generated stub composite binary tag definitions\n");
        code.push_str("//!\n");
        code.push_str("//! EXIFTOOL-SOURCE: lib/Image/ExifTool/Exif.pm (Composite tags)\n");
        code.push_str("//! EXIFTOOL-VERSION: 12.65\n");
        code.push_str("//!\n");
        code.push_str("//! This is a stub file generated when no tags were found.\n");
        code.push_str("//! Regenerate with: cargo run --bin exiftool_sync extract binary-tags\n\n");

        code.push_str("#![doc = \"EXIFTOOL-SOURCE: lib/Image/ExifTool/Exif.pm\"]\n\n");

        code.push_str("use std::collections::HashMap;\n");
        code.push_str("use crate::core::types::ExifValue;\n\n");

        code.push_str("#[derive(Debug, Clone)]\n");
        code.push_str("pub struct CompositeTagDefinition {\n");
        code.push_str("    pub name: &'static str,\n");
        code.push_str("    pub requires: &'static [(u16, &'static str)],\n");
        code.push_str("    pub desires: &'static [(u16, &'static str)],\n");
        code.push_str("    pub groups: &'static [(u8, &'static str)],\n");
        code.push_str("}\n\n");

        code.push_str("/// Stub composite tags (empty - regenerate to populate)\n");
        code.push_str("pub static BINARY_COMPOSITE_TAGS: &[CompositeTagDefinition] = &[\n");
        code.push_str("    // Stub - real tags will be generated by extractor\n");
        code.push_str("];\n\n");

        code.push_str("/// Extract binary data for composite tags (stub implementation)\n");
        code.push_str("pub fn extract_composite_binary_tag(\n");
        code.push_str("    _tag_name: &str,\n");
        code.push_str("    _tags: &HashMap<u16, ExifValue>,\n");
        code.push_str("    _file_data: &[u8]\n");
        code.push_str(") -> Option<Vec<u8>> {\n");
        code.push_str("    None // Stub implementation\n");
        code.push_str("}\n");

        Ok(code)
    }

    /// Generate Rust code for binary tag extraction
    fn generate_rust_code(&self) -> Result<String, String> {
        let mut code = String::new();

        code.push_str("//! Auto-generated composite binary tag definitions from ExifTool\n");
        code.push_str("//!\n");
        code.push_str("//! EXIFTOOL-SOURCE: lib/Image/ExifTool/Exif.pm (Composite tags)\n");
        code.push_str("//! EXIFTOOL-VERSION: 12.65\n");
        code.push_str("//!\n");
        code.push_str("//! This file is auto-generated by exiftool_sync extract binary-tags.\n");
        code.push_str("//! Do not edit manually.\n\n");

        code.push_str("use std::collections::HashMap;\n");
        code.push_str("use crate::core::types::ExifValue;\n\n");

        code.push_str("#[derive(Debug, Clone)]\n");
        code.push_str("pub struct CompositeTagDefinition {\n");
        code.push_str("    pub name: &'static str,\n");
        code.push_str("    pub requires: &'static [(u16, &'static str)],  // (tag_id, tag_name)\n");
        code.push_str("    pub desires: &'static [(u16, &'static str)],   // Optional tags\n");
        code.push_str("    pub groups: &'static [(u8, &'static str)],     // Group assignments\n");
        code.push_str("}\n\n");

        // Generate the static table of binary composite tags
        code.push_str("pub static BINARY_COMPOSITE_TAGS: &[CompositeTagDefinition] = &[\n");

        for tag in &self.composite_tags {
            if !tag.is_binary {
                continue;
            }

            code.push_str("    CompositeTagDefinition {\n");
            code.push_str(&format!("        name: \"{}\",\n", tag.name));

            // Generate requires array
            code.push_str("        requires: &[\n");
            for req in &tag.requires {
                let tag_id = self.get_tag_id(&req.tag_name);
                code.push_str(&format!(
                    "            (0x{:04X}, \"{}\"),\n",
                    tag_id, req.tag_name
                ));
            }
            code.push_str("        ],\n");

            // Generate desires array
            code.push_str("        desires: &[\n");
            for des in &tag.desires {
                let tag_id = self.get_tag_id(&des.tag_name);
                code.push_str(&format!(
                    "            (0x{:04X}, \"{}\"),\n",
                    tag_id, des.tag_name
                ));
            }
            code.push_str("        ],\n");

            // Generate groups array
            code.push_str("        groups: &[\n");
            for (group_num, group_name) in &tag.groups {
                code.push_str(&format!(
                    "            ({}, \"{}\"),\n",
                    group_num, group_name
                ));
            }
            code.push_str("        ],\n");

            code.push_str("    },\n");
        }

        code.push_str("];\n\n");

        // Generate helper functions
        code.push_str("/// Extract binary data for composite tags like ThumbnailImage\n");
        code.push_str("pub fn extract_composite_binary_tag(\n");
        code.push_str("    tag_name: &str,\n");
        code.push_str("    tags: &HashMap<u16, ExifValue>,\n");
        code.push_str("    file_data: &[u8]\n");
        code.push_str(") -> Option<Vec<u8>> {\n");
        code.push_str("    let composite_def = BINARY_COMPOSITE_TAGS.iter()\n");
        code.push_str("        .find(|def| def.name == tag_name)?;\n");
        code.push_str("    \n");
        code.push_str("    // Basic offset/length extraction pattern\n");
        code.push_str("    if composite_def.requires.len() >= 2 {\n");
        code.push_str("        let offset_tag_id = composite_def.requires[0].0;\n");
        code.push_str("        let length_tag_id = composite_def.requires[1].0;\n");
        code.push_str("        \n");
        code.push_str(
            "        if let (Some(ExifValue::U32(offset)), Some(ExifValue::U32(length))) = \n",
        );
        code.push_str("            (tags.get(&offset_tag_id), tags.get(&length_tag_id)) {\n");
        code.push_str("            \n");
        code.push_str("            let start = *offset as usize;\n");
        code.push_str("            let end = start + *length as usize;\n");
        code.push_str("            \n");
        code.push_str("            if end <= file_data.len() {\n");
        code.push_str("                return Some(file_data[start..end].to_vec());\n");
        code.push_str("            }\n");
        code.push_str("        }\n");
        code.push_str("    }\n");
        code.push_str("    \n");
        code.push_str("    None\n");
        code.push_str("}\n\n");

        // Generate individual extraction functions for known binary tags
        for tag in &self.composite_tags {
            if !tag.is_binary {
                continue;
            }

            let fn_name = tag.name.to_lowercase().replace("image", "_image");
            code.push_str(&format!("/// Extract {} binary data\n", tag.name));
            code.push_str(&format!("pub fn extract_{}(\n", fn_name));
            code.push_str("    tags: &HashMap<u16, ExifValue>,\n");
            code.push_str("    file_data: &[u8]\n");
            code.push_str(") -> Option<Vec<u8>> {\n");
            code.push_str(&format!(
                "    extract_composite_binary_tag(\"{}\", tags, file_data)\n",
                tag.name
            ));
            code.push_str("}\n\n");
        }

        Ok(code)
    }

    /// Write the generated code to the appropriate output file
    fn write_output(&self, code: &str) -> Result<(), String> {
        let output_path = Path::new("src/binary/composite_tags.rs");

        // Create directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create output directory: {}", e))?;
        }

        let mut file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        file.write_all(code.as_bytes())
            .map_err(|e| format!("Failed to write output file: {}", e))?;

        println!("Generated composite binary tags: {}", output_path.display());

        Ok(())
    }
}

impl Extractor for BinaryTagsExtractor {
    fn extract(&self, exiftool_path: &Path) -> Result<(), String> {
        println!("Extracting binary tag definitions from ExifTool...");

        let mut extractor = BinaryTagsExtractor::new();

        // Always attempt to parse composite tags
        match extractor.parse_composite_tags(exiftool_path) {
            Ok(()) => {
                println!(
                    "Found {} composite tags ({} binary tags)",
                    extractor.composite_tags.len(),
                    extractor
                        .composite_tags
                        .iter()
                        .filter(|t| t.is_binary)
                        .count()
                );
            }
            Err(e) => {
                println!("Warning: Failed to parse composite tags: {}", e);
                println!("  - Generating stub implementation");
            }
        }

        // Always generate code (real or stub)
        let code = if extractor.composite_tags.is_empty() {
            extractor.generate_stub_code()?
        } else {
            extractor.generate_rust_code()?
        };

        // Always write output
        extractor.write_output(&code)?;

        println!("Binary tags extraction completed successfully");
        if extractor.composite_tags.is_empty() {
            println!("  - Using stub implementation (no tags found)");
        } else {
            println!(
                "  - Generated {} composite tags ({} binary tags)",
                extractor.composite_tags.len(),
                extractor
                    .composite_tags
                    .iter()
                    .filter(|t| t.is_binary)
                    .count()
            );
        }

        Ok(())
    }
}
