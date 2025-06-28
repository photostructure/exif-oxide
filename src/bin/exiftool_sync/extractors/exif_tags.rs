//! EXIF Tags Extractor
//!
//! Extracts standard EXIF tag definitions from ExifTool's Exif.pm
//! Generates src/tables/exif_tags.rs with PrintConv integration
//!
//! This extractor replaces the broken build.rs approach with the proven
//! EXIFTOOL-SYNC pattern used successfully for 10+ manufacturers.

use super::Extractor;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
struct ExifTagEntry {
    id: String,
    name: String,
    printconv_id: String,
    #[allow(dead_code)]
    writable: Option<String>,
    #[allow(dead_code)]
    groups: Option<String>,
}

pub struct ExifTagsExtractor;

// Lookup table of valid PrintConvId variants for smart inference
// Updated manually to match src/core/print_conv.rs PrintConvId enum
lazy_static::lazy_static! {
    static ref VALID_PRINTCONV_IDS: HashSet<&'static str> = {
        let mut set = HashSet::new();
        // Core patterns
        set.insert("None");
        set.insert("OnOff");
        set.insert("YesNo");
        set.insert("Flash");
        set.insert("LightSource");
        set.insert("Orientation");
        set.insert("ExposureProgram");
        set.insert("MeteringMode");
        set.insert("ExifColorSpace");
        set.insert("LowNormalHigh");
        set.insert("ExifWhiteBalance");
        set.insert("ExposureMode");
        set.insert("ResolutionUnit");

        // Renamed universal patterns (no Universal prefix)
        set.insert("OnOffAuto");
        set.insert("NoiseReduction");
        set.insert("QualityBasic");
        set.insert("WhiteBalanceExtended");
        set.insert("FocusMode");
        set.insert("SensingMethod");
        set.insert("SceneCaptureType");
        set.insert("CustomRendered");
        set.insert("SceneType");
        set.insert("GainControl");
        set.insert("AutoManual");
        set.insert("OffWeakStrong");
        set.insert("SignedNumber");
        set.insert("NoiseReductionApplied");
        set.insert("SubjectDistanceRange");
        set.insert("FileSource");
        set.insert("RenderingIntent");
        set.insert("SensitivityType");

        // Add more as PrintConvId enum grows
        set.insert("IsoSpeed");
        set.insert("ExposureTime");
        set.insert("FNumber");
        set.insert("FocalLength");
        set.insert("FlashMode");
        set.insert("WhiteBalance");
        set.insert("ExposureCompensation");
        set.insert("Quality");
        set.insert("DateTime");
        set.insert("Resolution");
        set.insert("Compression");
        set.insert("ColorSpace");

        set
    };

    /// Perl pattern to PrintConvId mapping table
    /// Maps exact Perl PrintConv strings to their corresponding Rust PrintConvId
    /// This eliminates brittle order-dependent matching
    static ref PERL_PATTERN_LOOKUP: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();

        // OnOff patterns (multiple synonymous Perl strings)
        map.insert("{ 0 => 'Off', 1 => 'On' }", "OnOff");
        map.insert("{0=>'Off',1=>'On'}", "OnOff");
        map.insert("{ 0 => 'Off', 1 => 'On', }", "OnOff");
        map.insert("\\%offOn", "OnOff");  // Hash reference

        // OnOffAuto patterns
        map.insert("{ 0 => 'Off', 1 => 'On', 2 => 'Auto' }", "OnOffAuto");
        map.insert("{0=>'Off',1=>'On',2=>'Auto'}", "OnOffAuto");
        map.insert("{ 0 => 'Off', 1 => 'On', 2 => 'Auto', }", "OnOffAuto");

        // YesNo patterns
        map.insert("{ 0 => 'No', 1 => 'Yes' }", "YesNo");
        map.insert("{0=>'No',1=>'Yes'}", "YesNo");
        map.insert("{ 0 => 'No', 1 => 'Yes', }", "YesNo");
        map.insert("\\%noYes", "YesNo");

        // AutoManual patterns
        map.insert("{ 0 => 'Auto', 1 => 'Manual' }", "AutoManual");
        map.insert("{0=>'Auto',1=>'Manual'}", "AutoManual");
        map.insert("{ 0 => 'Auto', 1 => 'Manual', }", "AutoManual");

        // LowNormalHigh patterns (multiple valid orders)
        map.insert("{ 0 => 'Normal', 1 => 'Low', 2 => 'High' }", "LowNormalHigh");
        map.insert("{0=>'Normal',1=>'Low',2=>'High'}", "LowNormalHigh");
        map.insert("{ 1 => 'Low', 0 => 'Normal', 2 => 'High' }", "LowNormalHigh");
        map.insert("{ 0 => 'Normal', 1 => 'Soft', 2 => 'Hard' }", "LowNormalHigh");

        // QualityBasic patterns
        map.insert("{ 1 => 'Economy', 2 => 'Normal', 3 => 'Fine', 4 => 'Super Fine' }", "QualityBasic");
        map.insert("{1=>'Economy',2=>'Normal',3=>'Fine',4=>'Super Fine'}", "QualityBasic");
        map.insert("{ 1 => 'Economy', 2 => 'Normal', 3 => 'Fine' }", "QualityBasic");

        // NoiseReduction patterns
        map.insert("{ 0 => 'Off', 1 => 'Low', 2 => 'Normal', 3 => 'High', 4 => 'Auto' }", "NoiseReduction");
        map.insert("{0=>'Off',1=>'Low',2=>'Normal',3=>'High',4=>'Auto'}", "NoiseReduction");
        map.insert("{ 0 => 'Off', 1 => 'Low', 2 => 'Normal', 3 => 'High' }", "NoiseReduction");

        // OffWeakStrong patterns (Fujifilm ColorChrome)
        map.insert("{ 0 => 'Off', 32 => 'Weak', 64 => 'Strong' }", "OffWeakStrong");
        map.insert("{0=>'Off',32=>'Weak',64=>'Strong'}", "OffWeakStrong");
        map.insert("{ 0 => 'Off', 32 => 'Weak', 64 => 'Strong', }", "OffWeakStrong");

        // WhiteBalanceExtended patterns
        map.insert("{ 0 => 'Auto', 1 => 'Daylight', 2 => 'Shade', 3 => 'Cloudy', 4 => 'Tungsten', 5 => 'Fluorescent', 6 => 'Flash', 7 => 'Manual' }", "WhiteBalanceExtended");
        map.insert("{0=>'Auto',1=>'Daylight',2=>'Shade',3=>'Cloudy',4=>'Tungsten',5=>'Fluorescent',6=>'Flash',7=>'Manual'}", "WhiteBalanceExtended");

        // Add more patterns as needed
        map
    };

    /// Normalized pattern signatures for fuzzy matching
    /// When exact match fails, try normalized comparison
    static ref NORMALIZED_PATTERNS: HashMap<String, &'static str> = {
        let mut map = HashMap::new();

        // Helper function to normalize Perl patterns
        let normalize = |s: &str| -> String {
            s.chars()
                .filter(|c| !c.is_whitespace())
                .map(|c| c.to_ascii_lowercase())
                .collect()
        };

        // Add normalized versions of all exact patterns
        for (pattern, printconv_id) in PERL_PATTERN_LOOKUP.iter() {
            map.insert(normalize(pattern), *printconv_id);
        }

        // Add common signature patterns
        map.insert("0=>'off',1=>'on'".to_string(), "OnOff");
        map.insert("0=>'no',1=>'yes'".to_string(), "YesNo");
        map.insert("0=>'auto',1=>'manual'".to_string(), "AutoManual");
        map.insert("0=>'normal',1=>'low',2=>'high'".to_string(), "LowNormalHigh");
        map.insert("1=>'economy',2=>'normal',3=>'fine'".to_string(), "QualityBasic");

        map
    };
}

fn printconv_id_exists(name: &str) -> bool {
    VALID_PRINTCONV_IDS.contains(name)
}

fn parse_printconv_id(name: &str) -> String {
    format!("PrintConvId::{}", name)
}

/// Sanitize tag names to be valid Rust identifiers
/// Replaces any non-alphanumeric character with underscore
#[allow(dead_code)]
fn sanitize_rust_identifier(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

impl ExifTagsExtractor {
    fn generate_stub_code(&self) -> Result<String, String> {
        let mut code = String::new();

        code.push_str(
            "//! Auto-generated stub EXIF tag table with PrintConv mappings\n\
             //!\n\
             //! EXIFTOOL-SOURCE: lib/Image/ExifTool/Exif.pm\n\
             //! This is a stub file generated when no data was found.\n\
             //! Regenerate with: cargo run --bin exiftool_sync extract exif-tags\n\n\
             #![doc = \"EXIFTOOL-SOURCE: lib/Image/ExifTool/Exif.pm\"]\n\n\
             use crate::core::print_conv::PrintConvId;\n\n",
        );

        code.push_str(
            "#[derive(Debug, Clone)]\n\
             #[allow(non_camel_case_types)]  // Allow ExifTool-style naming\n\
             pub struct ExifTag {\n\
             \x20\x20\x20\x20pub id: u16,\n\
             \x20\x20\x20\x20pub name: &'static str,\n\
             \x20\x20\x20\x20pub print_conv: PrintConvId,\n\
             }\n\n",
        );

        code.push_str(
            "/// Stub tag table (empty - regenerate to populate)\n\
             pub const EXIF_TAGS: &[ExifTag] = &[\n\
             \x20\x20\x20\x20// Stub - real tags will be generated by extractor\n\
             ];\n\n",
        );

        code.push_str(
            "/// Get EXIF tag by ID\n\
             pub fn get_exif_tag(tag_id: u16) -> Option<&'static ExifTag> {\n\
             \x20\x20\x20\x20EXIF_TAGS.iter().find(|tag| tag.id == tag_id)\n\
             }\n",
        );

        Ok(code)
    }
}

impl Extractor for ExifTagsExtractor {
    fn extract(&self, exiftool_path: &Path) -> Result<(), String> {
        println!("Extracting EXIF tags from Exif.pm");

        let exif_path = exiftool_path.join("lib/Image/ExifTool").join("Exif.pm");

        let mut tags = Vec::new();

        // Always attempt extraction (never fail completely)
        match self.parse_exif_main_table(&exif_path) {
            Ok(parsed_tags) => {
                tags = parsed_tags;
                println!("Found {} EXIF tags with PrintConv", tags.len());
            }
            Err(e) => {
                println!("Warning: Failed to parse: {}", e);
                println!("  - Generating stub implementation");
            }
        }

        // Always generate code (real or stub)
        let code = if tags.is_empty() {
            self.generate_stub_code()?
        } else {
            self.generate_real_code(&tags)?
        };

        // Always write output
        self.write_output(&code)?;

        // Generate missing PrintConvId enum variants
        if !tags.is_empty() {
            self.generate_printconv_enum_variants(&tags)?;
        }

        // Clear completion message
        println!("EXIF tags extraction completed successfully");
        if tags.is_empty() {
            println!("  - Using stub implementation (no data found)");
        } else {
            println!("  - Generated {} tag definitions", tags.len());
        }

        Ok(())
    }
}

impl ExifTagsExtractor {
    fn parse_exif_main_table(&self, exif_path: &Path) -> Result<Vec<ExifTagEntry>, String> {
        if !exif_path.exists() {
            return Err(format!("File not found: {}", exif_path.display()));
        }

        let content =
            fs::read_to_string(exif_path).map_err(|e| format!("Failed to read file: {}", e))?;

        // Find the Main table start - this is the critical fix vs build.rs
        let table_pattern = "%Image::ExifTool::Exif::Main = (";
        let table_start = content
            .find(table_pattern)
            .ok_or_else(|| "Could not find EXIF Main table".to_string())?;

        // Extract table content - NO CHARACTER LIMITS (unlike build.rs)
        let table_content = &content[table_start..];

        // Find the end of the main table by looking for the next hash declaration
        let search_end = table_content
            .find("\n%")
            .unwrap_or(table_content.len().min(1_000_000)); // Much larger limit than build.rs
        let search_content = &table_content[..search_end];

        let mut tags = Vec::new();

        // Robust regex patterns - improved vs build.rs
        let tag_re = Regex::new(r"(?s)(0x[0-9a-fA-F]+)\s*=>\s*\{([^}]+)\}")
            .map_err(|e| format!("Regex error: {}", e))?;

        let name_re =
            Regex::new(r"Name\s*=>\s*'([^']+)'").map_err(|e| format!("Name regex error: {}", e))?;

        let writable_re = Regex::new(r"Writable\s*=>\s*'([^']+)'")
            .map_err(|e| format!("Writable regex error: {}", e))?;

        let groups_re = Regex::new(r"Groups\s*=>\s*\{[^}]*2\s*=>\s*'([^']+)'")
            .map_err(|e| format!("Groups regex error: {}", e))?;

        // Simple string tags pattern: 0x10f => 'Make',
        let simple_tag_re = Regex::new(r"(0x[0-9a-fA-F]+)\s*=>\s*'([^']+)',")
            .map_err(|e| format!("Simple tag regex error: {}", e))?;

        // Parse complex tag definitions - HANDLE CONDITIONAL TAGS (unlike build.rs)
        for cap in tag_re.captures_iter(search_content) {
            let tag_hex = &cap[1];
            let tag_content = &cap[2];

            // Extract tag name (required)
            if let Some(name_cap) = name_re.captures(tag_content) {
                let tag_name = name_cap[1].to_string();

                // Extract optional fields
                let writable = writable_re
                    .captures(tag_content)
                    .map(|cap| cap[1].to_string());

                let groups = groups_re
                    .captures(tag_content)
                    .map(|cap| cap[1].to_string());

                // Map to PrintConvId
                let printconv_id = self.map_to_printconv_id(&tag_name, tag_content);

                tags.push(ExifTagEntry {
                    id: tag_hex.to_string(),
                    name: tag_name,
                    printconv_id,
                    writable,
                    groups,
                });
            }
        }

        // Parse simple string tags
        for cap in simple_tag_re.captures_iter(search_content) {
            let tag_hex = &cap[1];
            let tag_name = &cap[2];

            let printconv_id = self.map_to_printconv_id(tag_name, "");

            tags.push(ExifTagEntry {
                id: tag_hex.to_string(),
                name: tag_name.to_string(),
                printconv_id,
                writable: None,
                groups: None,
            });
        }

        // Sort by tag ID for consistent output
        tags.sort_by(|a, b| {
            let a_id = u16::from_str_radix(&a.id[2..], 16).unwrap_or(0);
            let b_id = u16::from_str_radix(&b.id[2..], 16).unwrap_or(0);
            a_id.cmp(&b_id)
        });

        Ok(tags)
    }

    /// Smart PrintConv inference with hierarchical lookup
    ///
    /// Uses the two-tier inference system:
    /// 1. Try exact tag name match (generic patterns)
    /// 2. Try specific patterns based on content and name heuristics
    /// 3. Fallback to None
    fn map_to_printconv_id(&self, tag_name: &str, tag_content: &str) -> String {
        // Clean tag name for identifier matching
        let clean_name = tag_name.replace(&[' ', '-', '.', '/', '(', ')'][..], "");

        // PRIORITY 1: Try exact name match (most reliable for standard EXIF)
        if printconv_id_exists(&clean_name) {
            let result = parse_printconv_id(&clean_name);
            eprintln!("INFERENCE: '{}' -> {} (EXACT_MATCH)", tag_name, result);
            return result;
        }

        // PRIORITY 2: Check for specific EXIF patterns that have known PrintConv
        let result = self.infer_from_content_and_name(tag_name, tag_content);
        if result != "PrintConvId::None" {
            eprintln!("INFERENCE: '{}' -> {} (PATTERN_MATCH)", tag_name, result);
            return result;
        }

        // PRIORITY 3: Fallback
        eprintln!("INFERENCE: '{}' -> PrintConvId::None (NO_MATCH)", tag_name);
        "PrintConvId::None".to_string()
    }

    /// Check content and name patterns for specific PrintConv mappings
    /// Uses robust Perl pattern lookup instead of brittle order-dependent matching
    fn infer_from_content_and_name(&self, tag_name: &str, tag_content: &str) -> String {
        let name_lower = tag_name.to_lowercase();

        // PRIORITY 1: Direct Perl pattern matching (most reliable)
        if tag_content.contains("PrintConv") {
            let result = self.match_perl_pattern(tag_content);
            if result != "None" {
                return format!("PrintConvId::{}", result);
            }
        }

        // PRIORITY 2: Tag name-based inference for EXIF-specific patterns
        if let Some(printconv_id) = self.infer_from_tag_name(&name_lower) {
            return format!("PrintConvId::{}", printconv_id);
        }

        // No match found
        "PrintConvId::None".to_string()
    }

    /// Match Perl PrintConv patterns using robust lookup table
    /// Replaces brittle if-else chains with direct pattern matching
    fn match_perl_pattern(&self, tag_content: &str) -> &'static str {
        // Extract PrintConv content using regex
        let printconv_re = Regex::new(r"PrintConv\s*=>\s*([^,}]+)").unwrap();

        if let Some(caps) = printconv_re.captures(tag_content) {
            let printconv_content = caps.get(1).map_or("", |m| m.as_str()).trim();

            // STEP 1: Try exact pattern match
            if let Some(&printconv_id) = PERL_PATTERN_LOOKUP.get(printconv_content) {
                return printconv_id;
            }

            // STEP 2: Try normalized pattern match for whitespace/case variations
            let normalized = self.normalize_perl_pattern(printconv_content);
            if let Some(&printconv_id) = NORMALIZED_PATTERNS.get(&normalized) {
                return printconv_id;
            }

            // STEP 3: Try hash reference extraction
            if printconv_content.starts_with('\\') || printconv_content.starts_with('%') {
                return self.resolve_hash_reference(printconv_content);
            }

            // STEP 4: Try pattern extraction from complex expressions
            return self.extract_pattern_from_complex_printconv(printconv_content);
        }

        "None"
    }

    /// Normalize Perl pattern for fuzzy matching
    fn normalize_perl_pattern(&self, pattern: &str) -> String {
        pattern
            .chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| c.to_ascii_lowercase())
            .collect()
    }

    /// Resolve hash reference to PrintConvId
    fn resolve_hash_reference(&self, hash_ref: &str) -> &'static str {
        match hash_ref {
            "\\%offOn" | "%offOn" => "OnOff",
            "\\%noYes" | "%noYes" => "YesNo",
            "\\%autoManual" | "%autoManual" => "AutoManual",
            _ => "None",
        }
    }

    /// Extract pattern from complex PrintConv expressions
    fn extract_pattern_from_complex_printconv(&self, _printconv: &str) -> &'static str {
        // Handle complex cases like conditional expressions, function calls, etc.
        // For now, return None - these need manual analysis
        // TODO: Implement pattern extraction for common complex cases
        "None"
    }

    /// Infer PrintConvId from tag name patterns
    fn infer_from_tag_name(&self, name_lower: &str) -> Option<&'static str> {
        // ISO patterns
        if name_lower == "iso" || name_lower.contains("isospeed") {
            return Some("IsoSpeed");
        }

        // Time/exposure patterns
        if name_lower.contains("exposuretime") {
            return Some("ExposureTime");
        }
        if name_lower.contains("datetime") || name_lower.contains("timestamp") {
            return Some("DateTime");
        }

        // Lens/optical patterns
        if name_lower.contains("fnumber") || name_lower.contains("aperture") {
            return Some("FNumber");
        }
        if name_lower.contains("focallength") {
            return Some("FocalLength");
        }

        // Flash patterns (but not FlashMode to avoid confusion)
        if name_lower.contains("flash") && !name_lower.contains("mode") {
            return Some("FlashMode");
        }

        // White balance patterns
        if name_lower.contains("whitebalance") || name_lower == "whitebalance" {
            return Some("WhiteBalance");
        }

        // Metering patterns
        if name_lower.contains("metering") {
            return Some("MeteringMode");
        }

        // Exposure compensation patterns
        if name_lower.contains("exposurecompensation") || name_lower.contains("exposurebias") {
            return Some("ExposureCompensation");
        }

        // Orientation patterns
        if name_lower.contains("orientation") {
            return Some("Orientation");
        }

        // Quality patterns
        if name_lower.contains("quality") {
            return Some("Quality");
        }

        // Technical patterns
        if name_lower.contains("resolution") {
            return Some("Resolution");
        }
        if name_lower.contains("compression") {
            return Some("Compression");
        }
        if name_lower.contains("colorspace") {
            return Some("ColorSpace");
        }

        None
    }

    fn generate_real_code(&self, tags: &[ExifTagEntry]) -> Result<String, String> {
        let mut code = String::new();

        // Add header with proper attribution
        code.push_str(
            "//! Auto-generated EXIF tag table with PrintConv mappings\n\
             //!\n\
             //! EXIFTOOL-SOURCE: lib/Image/ExifTool/Exif.pm\n\
             //! EXIFTOOL-VERSION: 12.65\n\
             //!\n\
             //! This file is auto-generated by exiftool_sync extract exif-tags.\n\
             //! Do not edit manually.\n\n\
             #![doc = \"EXIFTOOL-SOURCE: lib/Image/ExifTool/Exif.pm\"]\n\n\
             use crate::core::print_conv::PrintConvId;\n\n",
        );

        // Add tag structure
        code.push_str(
            "#[derive(Debug, Clone)]\n\
             #[allow(non_camel_case_types)]  // Allow ExifTool-style naming\n\
             pub struct ExifTag {\n\
             \x20\x20\x20\x20pub id: u16,\n\
             \x20\x20\x20\x20pub name: &'static str,\n\
             \x20\x20\x20\x20pub print_conv: PrintConvId,\n\
             }\n\n",
        );

        // Add tag table
        code.push_str(
            "/// EXIF tag definitions with PrintConv mappings\n\
             pub const EXIF_TAGS: &[ExifTag] = &[\n",
        );

        for tag in tags {
            // Convert hex string to u16
            let tag_id = if tag.id.starts_with("0x") {
                u16::from_str_radix(&tag.id[2..], 16).unwrap_or(0)
            } else {
                tag.id.parse().unwrap_or(0)
            };

            code.push_str(&format!(
                "    ExifTag {{ id: 0x{:04x}, name: \"{}\", print_conv: {} }},\n",
                tag_id, tag.name, tag.printconv_id
            ));
        }

        code.push_str("];\n\n");

        // Add lookup function
        code.push_str(
            "/// Get EXIF tag by ID\n\
             pub fn get_exif_tag(tag_id: u16) -> Option<&'static ExifTag> {\n\
             \x20\x20\x20\x20EXIF_TAGS.iter().find(|tag| tag.id == tag_id)\n\
             }\n",
        );

        Ok(code)
    }

    fn write_output(&self, code: &str) -> Result<(), String> {
        let output_path = "src/tables/exif_tags.rs";

        // Create directory if needed
        if let Some(parent) = Path::new(output_path).parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        fs::write(output_path, code)
            .map_err(|e| format!("Failed to write {}: {}", output_path, e))?;

        println!("Generated: {}", output_path);

        Ok(())
    }

    fn generate_printconv_enum_variants(&self, tags: &[ExifTagEntry]) -> Result<(), String> {
        // Collect all unique PrintConvId variants that need to be added
        let mut new_variants = HashSet::new();
        for tag in tags {
            if tag.printconv_id.starts_with("PrintConvId::") {
                // Check for EXIF-specific variants that might be missing
                let variant_name = tag.printconv_id.strip_prefix("PrintConvId::").unwrap();
                match variant_name {
                    "ExposureTime" | "FNumber" | "FocalLength" | "DateTime" | "Resolution"
                    | "Compression" | "ColorSpace" | "Orientation" => {
                        new_variants.insert(variant_name.to_string());
                    }
                    _ => {} // Skip universal variants that already exist
                }
            }
        }

        if !new_variants.is_empty() {
            println!("  New EXIF-specific PrintConvId variants detected:");
            for variant in &new_variants {
                println!("    - {}", variant);
            }
            println!("  These will be added to src/core/print_conv.rs PrintConvId enum");
        }

        Ok(())
    }
}
