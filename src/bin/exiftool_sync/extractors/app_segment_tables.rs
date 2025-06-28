//! APP segment table extraction from ExifTool JPEG.pm
//!
//! Extracts APP0-APP15 segment definitions and generates static lookup tables
//! following the proven PrintConv table extraction pattern.

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/JPEG.pm"]

use super::Extractor;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct AppSegmentTablesExtractor;

#[derive(Debug)]
#[allow(dead_code)] // condition and subdirectory fields kept for debugging/future use
struct AppSegmentRule {
    segment: u8,        // APP0-APP15
    name: String,       // JFIF, XMP, Photoshop, etc.
    signature: Vec<u8>, // Detection signature
    condition: String,  // Original Perl condition
    condition_type: ConditionType,
    format_handler: FormatHandler,
    notes: Option<String>,
    subdirectory: Option<String>,
}

#[derive(Debug)]
enum ConditionType {
    StartsWith,
    Contains,
    Regex(String),
    Custom(String),
}

#[derive(Debug, Clone)]
#[allow(clippy::upper_case_acronyms)] // Industry standard format names should remain in caps
enum FormatHandler {
    JFIF,
    JFXX,
    XMP,
    ExtendedXMP,
    EXIF,
    Photoshop,
    Adobe,
    AdobeCM,
    IccProfile,
    MPF,
    GoPro,
    CIFF,
    AVI1,
    Ocad,
    QVCI,
    FLIR,
    Parrot,
    InfiRay,
    FPXR,
    RMETA,
    Samsung,
    EPPIM,
    NITF,
    HpTdhd,
    Pentax,
    Huawei,
    Qualcomm,
    DJI,
    SPIFF,
    SEAL,
    MediaJukebox,
    Comment,
    HDRGainCurve,
    JpegHdr,
    JUMBF,
    PictureInfo,
    Ducky,
    GraphicConverter,
    PreviewImage,
    Unknown,
}

impl AppSegmentTablesExtractor {
    pub fn new() -> Self {
        Self
    }

    /// Parse JPEG.pm Main table structure
    fn parse_jpeg_main_table(&self, content: &str) -> Result<Vec<AppSegmentRule>, String> {
        let mut rules = Vec::new();

        // Find the main table definition
        let table_start = content
            .find("%Image::ExifTool::JPEG::Main = (")
            .ok_or("Could not find JPEG Main table")?;

        let table_content = self.extract_table_content(content, table_start)?;

        // Parse each APP segment definition
        for app_num in 0..=15 {
            let app_key = format!("APP{}", app_num);
            if let Some(app_rules) = self.parse_app_segment(&table_content, app_num, &app_key)? {
                rules.extend(app_rules);
            }
        }

        // Parse trailer segments
        if let Some(trailer_rules) = self.parse_trailer_segment(&table_content)? {
            rules.extend(trailer_rules);
        }

        Ok(rules)
    }

    /// Extract table content between parentheses
    fn extract_table_content(&self, content: &str, start_pos: usize) -> Result<String, String> {
        let mut paren_count = 0;
        let mut start_found = false;
        let mut table_start = start_pos;
        let mut table_end = start_pos;

        for (i, ch) in content[start_pos..].char_indices() {
            match ch {
                '(' if !start_found => {
                    paren_count = 1;
                    start_found = true;
                    table_start = start_pos + i + 1;
                }
                '(' if start_found => paren_count += 1,
                ')' if start_found => {
                    paren_count -= 1;
                    if paren_count == 0 {
                        table_end = start_pos + i;
                        break;
                    }
                }
                _ => {}
            }
        }

        if !start_found || paren_count != 0 {
            return Err("Could not find complete table definition".to_string());
        }

        Ok(content[table_start..table_end].to_string())
    }

    /// Parse APP segment definitions
    fn parse_app_segment(
        &self,
        table_content: &str,
        app_num: u8,
        app_key: &str,
    ) -> Result<Option<Vec<AppSegmentRule>>, String> {
        // Look for APP segment definition with proper bracket balancing
        let pattern = format!(r"{}\s*=>\s*\[", regex::escape(app_key));
        let app_start_regex =
            Regex::new(&pattern).map_err(|e| format!("Invalid regex for {}: {}", app_key, e))?;

        let Some(app_match) = app_start_regex.find(table_content) else {
            return Ok(None);
        };

        // Find the matching closing bracket
        let start_pos = app_match.end();
        let app_content = self.extract_array_content(table_content, start_pos - 1)?;

        let mut rules = Vec::new();

        // Parse individual rules within the APP segment
        // Each rule is enclosed in curly braces { ... }
        let mut bracket_count = 0;
        let mut rule_start = None;
        let chars: Vec<char> = app_content.chars().collect();

        for (i, &ch) in chars.iter().enumerate() {
            match ch {
                '{' => {
                    if bracket_count == 0 {
                        rule_start = Some(i + 1);
                    }
                    bracket_count += 1;
                }
                '}' => {
                    bracket_count -= 1;
                    if bracket_count == 0 && rule_start.is_some() {
                        let start = rule_start.unwrap();
                        let rule_content: String = chars[start..i].iter().collect();
                        let parsed_rules = self.parse_single_rule(app_num, &rule_content)?;
                        rules.extend(parsed_rules);
                        rule_start = None;
                    }
                }
                _ => {}
            }
        }

        Ok(Some(rules))
    }

    /// Extract array content between brackets
    fn extract_array_content(&self, content: &str, start_pos: usize) -> Result<String, String> {
        let mut bracket_count = 0;
        let mut array_start = None;
        let mut array_end = None;

        for (i, ch) in content[start_pos..].char_indices() {
            match ch {
                '[' => {
                    if bracket_count == 0 {
                        array_start = Some(start_pos + i + 1);
                    }
                    bracket_count += 1;
                }
                ']' => {
                    bracket_count -= 1;
                    if bracket_count == 0 {
                        array_end = Some(start_pos + i);
                        break;
                    }
                }
                _ => {}
            }
        }

        match (array_start, array_end) {
            (Some(start), Some(end)) => Ok(content[start..end].to_string()),
            _ => Err("Could not find complete array definition".to_string()),
        }
    }

    /// Parse trailer segment definitions
    fn parse_trailer_segment(
        &self,
        table_content: &str,
    ) -> Result<Option<Vec<AppSegmentRule>>, String> {
        // Look for Trailer definition
        let trailer_regex = Regex::new(r"Trailer\s*=>\s*\[(.*?)\]")
            .map_err(|e| format!("Invalid trailer regex: {}", e))?;

        let Some(caps) = trailer_regex.captures(table_content) else {
            return Ok(None);
        };

        let trailer_content = &caps[1];
        let mut rules = Vec::new();

        // Parse individual trailer rules
        let rule_regex =
            Regex::new(r"\{\s*(.*?)\s*\}").map_err(|e| format!("Invalid rule regex: {}", e))?;

        for rule_match in rule_regex.captures_iter(trailer_content) {
            let rule_content = &rule_match[1];
            let parsed_rules = self.parse_single_rule(255, rule_content)?; // Use 255 to indicate trailer
            rules.extend(parsed_rules);
        }

        Ok(Some(rules))
    }

    /// Parse a single rule definition (may return multiple rules for OR patterns)
    fn parse_single_rule(
        &self,
        segment: u8,
        rule_content: &str,
    ) -> Result<Vec<AppSegmentRule>, String> {
        // Extract Name field
        let name_regex = Regex::new(r#"Name\s*=>\s*['"]([^'"]+)['"]"#)
            .map_err(|e| format!("Invalid name regex: {}", e))?;
        let Some(name_caps) = name_regex.captures(rule_content) else {
            return Ok(vec![]); // Skip rules without names
        };
        let name = name_caps[1].to_string();

        // Extract Condition field
        let condition_regex = Regex::new(r#"Condition\s*=>\s*['"]([^'"]+)['"]"#)
            .map_err(|e| format!("Invalid condition regex: {}", e))?;
        let condition = condition_regex
            .captures(rule_content)
            .map(|caps| caps[1].to_string())
            .unwrap_or_default();

        // Parse condition to extract signatures and types (may return multiple for OR patterns)
        let conditions = self.parse_condition_with_or(&condition)?;

        // Determine format handler from name
        let format_handler = self.determine_format_handler(&name);

        // Extract SubDirectory field
        let subdirectory_regex =
            Regex::new(r#"SubDirectory\s*=>\s*\{\s*TagTable\s*=>\s*['"]([^'"]+)['"]"#)
                .map_err(|e| format!("Invalid subdirectory regex: {}", e))?;
        let subdirectory = subdirectory_regex
            .captures(rule_content)
            .map(|caps| caps[1].to_string());

        // Extract Notes field
        let notes_regex = Regex::new(r#"Notes\s*=>\s*['"]([^'"]+)['"]"#)
            .map_err(|e| format!("Invalid notes regex: {}", e))?;
        let notes = notes_regex
            .captures(rule_content)
            .map(|caps| caps[1].to_string());

        // Generate a rule for each condition (handles OR patterns)
        let rules = conditions
            .into_iter()
            .map(|(signature, condition_type)| AppSegmentRule {
                segment,
                name: name.clone(),
                signature,
                condition: condition.clone(),
                condition_type,
                format_handler: format_handler.clone(),
                notes: notes.clone(),
                subdirectory: subdirectory.clone(),
            })
            .collect();

        Ok(rules)
    }

    /// Parse Perl condition with OR pattern support
    fn parse_condition_with_or(
        &self,
        condition: &str,
    ) -> Result<Vec<(Vec<u8>, ConditionType)>, String> {
        if condition.is_empty() {
            return Ok(vec![(
                Vec::new(),
                ConditionType::Custom("always".to_string()),
            )]);
        }

        // Check for OR patterns in StartsWith: $$valPt =~ /^(A|B|C)/
        if let Some(caps) = Regex::new(r"\$\$valPt\s*=~\s*/\^\(([^)]+)\)/")
            .unwrap()
            .captures(condition)
        {
            let alternatives = &caps[1];
            let parts = self.split_or_pattern(alternatives)?;
            let mut results = Vec::new();
            for part in parts {
                let signature = self.convert_perl_pattern_to_bytes(&part)?;
                results.push((signature, ConditionType::StartsWith));
            }
            return Ok(results);
        }

        // Check for OR patterns in Contains: $$valPt =~ /(A|B|C)$/
        if let Some(caps) = Regex::new(r"\$\$valPt\s*=~\s*/\(([^)]+)\)\$/")
            .unwrap()
            .captures(condition)
        {
            let alternatives = &caps[1];
            let parts = self.split_or_pattern(alternatives)?;
            let mut results = Vec::new();
            for part in parts {
                let signature = self.convert_perl_pattern_to_bytes(&part)?;
                results.push((signature, ConditionType::Contains));
            }
            return Ok(results);
        }

        // Fall back to single condition parsing
        let (signature, condition_type) = self.parse_condition(condition)?;
        Ok(vec![(signature, condition_type)])
    }

    /// Split OR pattern like "A|B|C" into vec!["A", "B", "C"]
    fn split_or_pattern(&self, pattern: &str) -> Result<Vec<String>, String> {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut escape = false;
        let mut paren_depth = 0;

        for ch in pattern.chars() {
            if escape {
                current.push(ch);
                escape = false;
                continue;
            }

            match ch {
                '\\' => {
                    current.push(ch);
                    escape = true;
                }
                '(' => {
                    current.push(ch);
                    paren_depth += 1;
                }
                ')' => {
                    current.push(ch);
                    paren_depth -= 1;
                }
                '|' if paren_depth == 0 => {
                    parts.push(current.trim().to_string());
                    current = String::new();
                }
                _ => {
                    current.push(ch);
                }
            }
        }

        if !current.is_empty() {
            parts.push(current.trim().to_string());
        }

        if parts.is_empty() {
            return Err("Empty OR pattern".to_string());
        }

        Ok(parts)
    }

    /// Parse Perl condition to extract signature and condition type
    fn parse_condition(&self, condition: &str) -> Result<(Vec<u8>, ConditionType), String> {
        if condition.is_empty() {
            return Ok((Vec::new(), ConditionType::Custom("always".to_string())));
        }

        // Pattern: $$valPt =~ /^pattern/
        if let Some(caps) = Regex::new(r"\$\$valPt\s*=~\s*/\^([^/]+)/")
            .unwrap()
            .captures(condition)
        {
            let pattern = &caps[1];
            let signature = self.convert_perl_pattern_to_bytes(pattern)?;
            return Ok((signature, ConditionType::StartsWith));
        }

        // Pattern: $$valPt =~ /pattern$/
        if let Some(caps) = Regex::new(r"\$\$valPt\s*=~\s*/([^/]+)\$/")
            .unwrap()
            .captures(condition)
        {
            let pattern = &caps[1];
            let signature = self.convert_perl_pattern_to_bytes(pattern)?;
            return Ok((signature, ConditionType::Contains));
        }

        // Pattern: $$valPt =~ /pattern/
        if let Some(caps) = Regex::new(r"\$\$valPt\s*=~\s*/([^/]+)/")
            .unwrap()
            .captures(condition)
        {
            let pattern = &caps[1];
            let signature = self.convert_perl_pattern_to_bytes(pattern)?;
            return Ok((signature, ConditionType::Regex(pattern.to_string())));
        }

        // Custom condition (complex logic)
        Ok((Vec::new(), ConditionType::Custom(condition.to_string())))
    }

    /// Convert Perl pattern to byte signature
    fn convert_perl_pattern_to_bytes(&self, pattern: &str) -> Result<Vec<u8>, String> {
        let mut bytes = Vec::new();
        let mut chars = pattern.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '\\' => {
                    // Handle escape sequences
                    if let Some(next_ch) = chars.next() {
                        match next_ch {
                            '0' => bytes.push(0),
                            'x' => {
                                // Hex escape: \x41
                                let hex_str: String = chars.by_ref().take(2).collect();
                                if hex_str.len() == 2 {
                                    let byte = u8::from_str_radix(&hex_str, 16).map_err(|_| {
                                        format!("Invalid hex escape: \\x{}", hex_str)
                                    })?;
                                    bytes.push(byte);
                                } else {
                                    return Err(format!("Invalid hex escape: \\x{}", hex_str));
                                }
                            }
                            _ => {
                                // Regular escape
                                bytes.push(next_ch as u8);
                            }
                        }
                    }
                }
                _ if ch.is_ascii() => {
                    bytes.push(ch as u8);
                }
                _ => {
                    return Err(format!("Non-ASCII character in pattern: {}", ch));
                }
            }
        }

        Ok(bytes)
    }

    /// Determine format handler from name
    fn determine_format_handler(&self, name: &str) -> FormatHandler {
        match name {
            "JFIF" => FormatHandler::JFIF,
            "JFXX" => FormatHandler::JFXX,
            "XMP" => FormatHandler::XMP,
            "ExtendedXMP" => FormatHandler::ExtendedXMP,
            "EXIF" => FormatHandler::EXIF,
            "Photoshop" => FormatHandler::Photoshop,
            "Adobe" => FormatHandler::Adobe,
            "Adobe_CM" => FormatHandler::AdobeCM,
            "ICC_Profile" => FormatHandler::IccProfile,
            "MPF" => FormatHandler::MPF,
            "GoPro" => FormatHandler::GoPro,
            "CIFF" => FormatHandler::CIFF,
            "AVI1" => FormatHandler::AVI1,
            "Ocad" => FormatHandler::Ocad,
            "QVCI" => FormatHandler::QVCI,
            "FLIR" => FormatHandler::FLIR,
            "RawThermalImage" => FormatHandler::Parrot,
            "FPXR" => FormatHandler::FPXR,
            "RMETA" => FormatHandler::RMETA,
            "SamsungUniqueID" => FormatHandler::Samsung,
            "EPPIM" => FormatHandler::EPPIM,
            "NITF" => FormatHandler::NITF,
            "HP_TDHD" => FormatHandler::HpTdhd,
            "Pentax" => FormatHandler::Pentax,
            "Huawei" => FormatHandler::Huawei,
            "Qualcomm" | "QualcommDualCamera" => FormatHandler::Qualcomm,
            "SPIFF" => FormatHandler::SPIFF,
            "SEAL" => FormatHandler::SEAL,
            "MediaJukebox" => FormatHandler::MediaJukebox,
            "Comment" => FormatHandler::Comment,
            "HDRGainCurve" => FormatHandler::HDRGainCurve,
            "JPEG-HDR" => FormatHandler::JpegHdr,
            "JUMBF" => FormatHandler::JUMBF,
            "PictureInfo" => FormatHandler::PictureInfo,
            "Ducky" => FormatHandler::Ducky,
            "GraphicConverter" => FormatHandler::GraphicConverter,
            "PreviewImage" => FormatHandler::PreviewImage,
            _ if name.contains("InfiRay") => FormatHandler::InfiRay,
            _ if name.contains("DJI") || name.contains("Thermal") => FormatHandler::DJI,
            _ => FormatHandler::Unknown,
        }
    }

    /// Generate Rust code for APP segment tables
    fn generate_app_segment_tables(&self, rules: &[AppSegmentRule]) -> String {
        let mut output = String::new();

        // Generate file header
        output.push_str(
            "// AUTO-GENERATED from ExifTool JPEG.pm\n\
             // Generated by exiftool_sync extract app-segment-tables\n\
             // DO NOT EDIT - Regenerate with `cargo run --bin exiftool_sync extract app-segment-tables`\n\n\
             #![doc = \"EXIFTOOL-SOURCE: lib/Image/ExifTool/JPEG.pm\"]\n\n"
        );

        // Generate enums
        self.generate_enums(&mut output);

        // Generate rule structures
        self.generate_rule_structures(&mut output);

        // Generate static tables
        self.generate_static_tables(&mut output, rules);

        // Generate lookup functions
        self.generate_lookup_functions(&mut output);

        output
    }

    /// Generate enum definitions
    fn generate_enums(&self, output: &mut String) {
        output.push_str(
            "/// Condition types for APP segment detection\n\
             #[derive(Debug, Clone, PartialEq)]\n\
             pub enum ConditionType {\n\
             \x20   StartsWith,\n\
             \x20   Contains,\n\
             \x20   Regex(&'static str),\n\
             \x20   Custom(&'static str),\n\
             }\n\n\
             /// Format handlers for different APP segment types\n\
             #[derive(Debug, Clone, PartialEq)]\n\
             pub enum FormatHandler {\n\
             \x20   JFIF,\n\
             \x20   JFXX,\n\
             \x20   XMP,\n\
             \x20   ExtendedXMP,\n\
             \x20   EXIF,\n\
             \x20   Photoshop,\n\
             \x20   Adobe,\n\
             \x20   AdobeCM,\n\
             \x20   IccProfile,\n\
             \x20   MPF,\n\
             \x20   GoPro,\n\
             \x20   CIFF,\n\
             \x20   AVI1,\n\
             \x20   Ocad,\n\
             \x20   QVCI,\n\
             \x20   FLIR,\n\
             \x20   Parrot,\n\
             \x20   InfiRay,\n\
             \x20   FPXR,\n\
             \x20   RMETA,\n\
             \x20   Samsung,\n\
             \x20   EPPIM,\n\
             \x20   NITF,\n\
             \x20   HpTdhd,\n\
             \x20   Pentax,\n\
             \x20   Huawei,\n\
             \x20   Qualcomm,\n\
             \x20   DJI,\n\
             \x20   SPIFF,\n\
             \x20   SEAL,\n\
             \x20   MediaJukebox,\n\
             \x20   Comment,\n\
             \x20   HDRGainCurve,\n\
             \x20   JpegHdr,\n\
             \x20   JUMBF,\n\
             \x20   PictureInfo,\n\
             \x20   Ducky,\n\
             \x20   GraphicConverter,\n\
             \x20   PreviewImage,\n\
             \x20   Unknown,\n\
             }\n\n",
        );
    }

    /// Generate rule structure definitions
    fn generate_rule_structures(&self, output: &mut String) {
        output.push_str(
            "/// APP segment identification rule\n\
             #[derive(Debug, Clone)]\n\
             pub struct AppSegmentRule {\n\
             \x20   pub name: &'static str,\n\
             \x20   pub signature: &'static [u8],\n\
             \x20   pub condition_type: ConditionType,\n\
             \x20   pub format_handler: FormatHandler,\n\
             \x20   pub notes: Option<&'static str>,\n\
             }\n\n",
        );
    }

    /// Generate static lookup tables
    fn generate_static_tables(&self, output: &mut String, rules: &[AppSegmentRule]) {
        // Group rules by segment number
        let mut segments: HashMap<u8, Vec<&AppSegmentRule>> = HashMap::new();
        for rule in rules {
            segments.entry(rule.segment).or_default().push(rule);
        }

        // Generate tables for each APP segment
        for app_num in 0..=15 {
            if let Some(app_rules) = segments.get(&app_num) {
                output.push_str(&format!(
                    "/// APP{} segment definitions\n\
                     pub static APP{}_SEGMENTS: &[AppSegmentRule] = &[\n",
                    app_num, app_num
                ));

                for rule in app_rules {
                    self.generate_rule_entry(output, rule);
                }

                output.push_str("];\n\n");
            } else {
                // Generate empty table for unused APP segments
                output.push_str(&format!(
                    "/// APP{} segment definitions (empty)\n\
                     pub static APP{}_SEGMENTS: &[AppSegmentRule] = &[];\n\n",
                    app_num, app_num
                ));
            }
        }

        // Generate trailer segments table (always generate, even if empty)
        output.push_str(
            "/// JPEG trailer segment definitions\n\
             pub static TRAILER_SEGMENTS: &[AppSegmentRule] = &[\n",
        );

        if let Some(trailer_rules) = segments.get(&255) {
            for rule in trailer_rules {
                self.generate_rule_entry(output, rule);
            }
        }

        output.push_str("];\n\n");

        // Generate main lookup table
        output.push_str(
            "/// Lookup table for all APP segments\n\
             pub static APP_SEGMENT_LOOKUP: &[&[AppSegmentRule]] = &[\n",
        );

        for app_num in 0..=15 {
            output.push_str(&format!(
                "    APP{}_SEGMENTS,   // APP{}\n",
                app_num, app_num
            ));
        }

        output.push_str("];\n\n");
    }

    /// Generate a single rule entry
    fn generate_rule_entry(&self, output: &mut String, rule: &AppSegmentRule) {
        let signature_bytes = rule
            .signature
            .iter()
            .map(|b| format!("0x{:02x}", b))
            .collect::<Vec<_>>()
            .join(", ");

        let condition_type = match &rule.condition_type {
            ConditionType::StartsWith => "ConditionType::StartsWith".to_string(),
            ConditionType::Contains => "ConditionType::Contains".to_string(),
            ConditionType::Regex(pattern) => {
                // Escape the pattern properly for Rust raw string
                let escaped_pattern = pattern.replace('\\', "\\\\").replace('"', "\\\"");
                format!("ConditionType::Regex(r\"{}\")", escaped_pattern)
            }
            ConditionType::Custom(cond) => {
                let escaped_cond = cond.replace('\\', "\\\\").replace('"', "\\\"");
                format!("ConditionType::Custom(r\"{}\")", escaped_cond)
            }
        };

        let format_handler = format!("FormatHandler::{:?}", rule.format_handler);

        let notes = rule
            .notes
            .as_ref()
            .map(|n| format!("Some(\"{}\")", n))
            .unwrap_or_else(|| "None".to_string());

        output.push_str(&format!(
            "    AppSegmentRule {{\n\
             \x20       name: \"{}\",\n\
             \x20       signature: &[{}],\n\
             \x20       condition_type: {},\n\
             \x20       format_handler: {},\n\
             \x20       notes: {},\n\
             \x20   }},\n",
            rule.name, signature_bytes, condition_type, format_handler, notes
        ));
    }

    /// Generate lookup functions
    fn generate_lookup_functions(&self, output: &mut String) {
        output.push_str(
            "/// Get APP segment rules for a specific segment type\n\
             pub fn get_app_segment_rules(segment: u8) -> Option<&'static [AppSegmentRule]> {\n\
             \x20   if segment <= 15 {\n\
             \x20       Some(APP_SEGMENT_LOOKUP[segment as usize])\n\
             \x20   } else {\n\
             \x20       None\n\
             \x20   }\n\
             }\n\n\
             /// Identify APP segment format from data\n\
             pub fn identify_app_segment(segment: u8, data: &[u8]) -> Option<&'static AppSegmentRule> {\n\
             \x20   let rules = get_app_segment_rules(segment)?;\n\
             \x20   \n\
             \x20   for rule in rules {\n\
             \x20       match &rule.condition_type {\n\
             \x20           ConditionType::StartsWith => {\n\
             \x20               if data.starts_with(rule.signature) {\n\
             \x20                   return Some(rule);\n\
             \x20               }\n\
             \x20           }\n\
             \x20           ConditionType::Contains => {\n\
             \x20               if data.windows(rule.signature.len()).any(|w| w == rule.signature) {\n\
             \x20                   return Some(rule);\n\
             \x20               }\n\
             \x20           }\n\
             \x20           ConditionType::Regex(_) => {\n\
             \x20               // TODO: Implement regex matching for complex patterns\n\
             \x20               if data.starts_with(rule.signature) {\n\
             \x20                   return Some(rule);\n\
             \x20               }\n\
             \x20           }\n\
             \x20           ConditionType::Custom(_) => {\n\
             \x20               // TODO: Implement custom condition logic\n\
             \x20               if !rule.signature.is_empty() && data.starts_with(rule.signature) {\n\
             \x20                   return Some(rule);\n\
             \x20               }\n\
             \x20           }\n\
             \x20       }\n\
             \x20   }\n\
             \x20   \n\
             \x20   None\n\
             }\n\n\
             /// Get trailer segment rules\n\
             pub fn get_trailer_rules() -> &'static [AppSegmentRule] {\n\
             \x20   TRAILER_SEGMENTS\n\
             }\n"
        );
    }
}

impl Extractor for AppSegmentTablesExtractor {
    fn extract(&self, exiftool_path: &Path) -> Result<(), String> {
        let jpeg_pm_path = exiftool_path.join("lib/Image/ExifTool/JPEG.pm");
        let content = fs::read_to_string(&jpeg_pm_path)
            .map_err(|e| format!("Failed to read JPEG.pm: {}", e))?;

        println!("📊 Extracting APP segment definitions from JPEG.pm...");

        let rules = self.parse_jpeg_main_table(&content)?;

        println!("   Found {} APP segment rules", rules.len());

        let generated_code = self.generate_app_segment_tables(&rules);

        // Write to src/tables/app_segments.rs
        let output_path = Path::new("src/tables/app_segments.rs");
        fs::write(output_path, generated_code)
            .map_err(|e| format!("Failed to write app_segments.rs: {}", e))?;

        println!("✅ Generated APP segment tables: {} rules", rules.len());
        println!("   Output: {}", output_path.display());

        Ok(())
    }
}
