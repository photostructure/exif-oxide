//! PrintConv Pattern Analyzer
//!
//! Analyzes ExifTool PrintConv patterns to identify reusable vs manufacturer-specific patterns

use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct PrintConvPattern {
    pub tag_id: String,
    pub tag_name: String,
    pub pattern_type: PrintConvType,
    pub values: Vec<(String, String)>, // key -> value mappings
}

#[derive(Debug, Clone)]
pub enum PrintConvType {
    Universal(String), // References existing PrintConvId (e.g., "OnOff", "Quality")
    Lookup(String),    // Manufacturer-specific lookup table
    Complex(String),   // Complex conversion requiring custom function
}

pub struct PrintConvAnalyzer {
    manufacturer: String,
    patterns: Vec<PrintConvPattern>,
}

impl PrintConvAnalyzer {
    pub fn get_patterns(&self) -> &[PrintConvPattern] {
        &self.patterns
    }

    #[allow(dead_code)]
    pub fn get_manufacturer(&self) -> &str {
        &self.manufacturer
    }

    pub fn new(manufacturer_file: &str) -> Self {
        let manufacturer = manufacturer_file.trim_end_matches(".pm").to_string();

        Self {
            manufacturer,
            patterns: Vec::new(),
        }
    }

    pub fn analyze(&self, manufacturer_path: &Path) -> Result<(), String> {
        println!("Reading {}...", manufacturer_path.display());

        let content = fs::read_to_string(manufacturer_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let mut analyzer = self.clone();
        analyzer.parse_printconv_patterns(&content)?;
        analyzer.classify_patterns();
        analyzer.print_analysis();

        Ok(())
    }

    fn parse_printconv_patterns(&mut self, content: &str) -> Result<(), String> {
        // Canon uses subdirectory tables extensively, so we need to search both:
        // 1. Main table inline PrintConv patterns
        // 2. Subdirectory table PrintConv patterns

        self.parse_main_table_patterns(content)?;
        self.parse_subdirectory_patterns(content)?;

        Ok(())
    }

    fn parse_main_table_patterns(&mut self, content: &str) -> Result<(), String> {
        // Find the main table for this manufacturer
        let table_pattern = format!("%Image::ExifTool::{}::Main = (", self.manufacturer);

        let table_start = content
            .find(&table_pattern)
            .ok_or_else(|| format!("Could not find {} main table", self.manufacturer))?;

        // Extract table content up to next table definition
        let table_content = &content[table_start..];
        let search_end = table_content
            .find("\n%")
            .unwrap_or(table_content.len().min(50000));
        let search_content = &table_content[..search_end];

        // Look for inline PrintConv patterns (like FileNumber)
        let inline_printconv_re =
            Regex::new(r"(?s)(0x[0-9a-fA-F]+)\s*=>\s*\{([^}]*PrintConv[^}]*)\}")
                .map_err(|e| format!("Inline PrintConv regex error: {}", e))?;

        let name_re =
            Regex::new(r"Name\s*=>\s*'([^']+)'").map_err(|e| format!("Name regex error: {}", e))?;

        for cap in inline_printconv_re.captures_iter(search_content) {
            let tag_hex = &cap[1];
            let tag_content = &cap[2];

            // Extract tag name
            let tag_name = if let Some(name_cap) = name_re.captures(tag_content) {
                name_cap[1].to_string()
            } else {
                continue;
            };

            // Look for simple PrintConv assignments
            if let Some(printconv_start) = tag_content.find("PrintConv => ") {
                let printconv_section = &tag_content[printconv_start + 13..];
                let printconv_value = self.extract_complete_printconv_value(printconv_section)?;

                // Skip complex expressions, focus on hash tables
                if printconv_value.starts_with('{') && printconv_value.contains('}') {
                    let values = self.parse_printconv_hash(&printconv_value)?;
                    if !values.is_empty() {
                        let pattern = PrintConvPattern {
                            tag_id: tag_hex.to_string(),
                            tag_name,
                            pattern_type: PrintConvType::Lookup("TBD".to_string()),
                            values,
                        };
                        self.patterns.push(pattern);
                    } else {
                        // Hash parsing failed, treat as complex with debug info
                        let debug_content = if printconv_value.len() > 200 {
                            format!("{}...", &printconv_value[..200])
                        } else {
                            printconv_value.clone()
                        };
                        let pattern = PrintConvPattern {
                            tag_id: tag_hex.to_string(),
                            tag_name,
                            pattern_type: PrintConvType::Complex("TBD".to_string()),
                            values: vec![("parse_failed".to_string(), debug_content)],
                        };
                        self.patterns.push(pattern);
                    }
                } else {
                    // Record complex PrintConv for later analysis
                    let pattern = PrintConvPattern {
                        tag_id: tag_hex.to_string(),
                        tag_name,
                        pattern_type: PrintConvType::Complex("TBD".to_string()),
                        values: vec![("complex".to_string(), printconv_value.to_string())],
                    };
                    self.patterns.push(pattern);
                }
            }
        }

        Ok(())
    }

    fn parse_subdirectory_patterns(&mut self, content: &str) -> Result<(), String> {
        // Find all subdirectory table definitions (Canon's main pattern)
        let table_def_re = Regex::new(&format!(
            r"%Image::ExifTool::{}::([A-Za-z0-9_]+)\s*=\s*\(",
            self.manufacturer
        ))
        .map_err(|e| format!("Table definition regex error: {}", e))?;

        for table_cap in table_def_re.captures_iter(content) {
            let table_name = &table_cap[1];
            let full_table_name =
                format!("%Image::ExifTool::{}::{}", self.manufacturer, table_name);

            if let Some(table_start) = content.find(&full_table_name) {
                // Extract this table's content
                let table_content = &content[table_start..];
                let search_end = table_content
                    .find("\n%")
                    .unwrap_or(table_content.len().min(50000));
                let search_content = &table_content[..search_end];

                // Look for PrintConv patterns in this subdirectory table
                self.parse_table_printconv_patterns(search_content, table_name)?;
            }
        }

        Ok(())
    }

    fn parse_table_printconv_patterns(
        &mut self,
        table_content: &str,
        table_name: &str,
    ) -> Result<(), String> {
        // Look for entries with PrintConv in subdirectory tables
        let entry_with_printconv_re =
            Regex::new(r"(?s)(\d+|0x[0-9a-fA-F]+)\s*=>\s*\{([^}]*PrintConv[^}]*)\}")
                .map_err(|e| format!("Entry PrintConv regex error: {}", e))?;

        let name_re =
            Regex::new(r"Name\s*=>\s*'([^']+)'").map_err(|e| format!("Name regex error: {}", e))?;

        for cap in entry_with_printconv_re.captures_iter(table_content) {
            let tag_index = &cap[1];
            let entry_content = &cap[2];

            // Extract tag name
            let tag_name = if let Some(name_cap) = name_re.captures(entry_content) {
                name_cap[1].to_string()
            } else {
                format!("{}Index{}", table_name, tag_index)
            };

            // Look for PrintConv hash tables
            if let Some(printconv_start) = entry_content.find("PrintConv => ") {
                let printconv_section = &entry_content[printconv_start + 13..];

                // Extract the complete PrintConv value (could be multiline hash)
                let printconv_value = self.extract_complete_printconv_value(printconv_section)?;

                if printconv_value.starts_with('{') && printconv_value.contains('}') {
                    let values = self.parse_printconv_hash(&printconv_value)?;
                    if !values.is_empty() {
                        let pattern = PrintConvPattern {
                            tag_id: format!("{}:{}", table_name, tag_index),
                            tag_name,
                            pattern_type: PrintConvType::Lookup("TBD".to_string()),
                            values,
                        };
                        self.patterns.push(pattern);
                    } else {
                        // Hash parsing failed, treat as complex with debug info
                        let debug_content = if printconv_value.len() > 200 {
                            format!("{}...", &printconv_value[..200])
                        } else {
                            printconv_value.clone()
                        };
                        let pattern = PrintConvPattern {
                            tag_id: format!("{}:{}", table_name, tag_index),
                            tag_name,
                            pattern_type: PrintConvType::Complex("TBD".to_string()),
                            values: vec![("parse_failed".to_string(), debug_content)],
                        };
                        self.patterns.push(pattern);
                    }
                } else if !printconv_value.is_empty() && printconv_value != "undef" {
                    // Complex PrintConv pattern
                    let pattern = PrintConvPattern {
                        tag_id: format!("{}:{}", table_name, tag_index),
                        tag_name,
                        pattern_type: PrintConvType::Complex("TBD".to_string()),
                        values: vec![("complex".to_string(), printconv_value.to_string())],
                    };
                    self.patterns.push(pattern);
                }
            }
        }

        Ok(())
    }

    fn extract_complete_printconv_value(&self, printconv_section: &str) -> Result<String, String> {
        let trimmed = printconv_section.trim();

        if trimmed.starts_with('{') {
            // Find matching closing brace for multiline hash
            let mut brace_count = 0;
            let mut in_string = false;
            let mut escape_next = false;
            let mut end_pos = 0;

            for (i, ch) in trimmed.char_indices() {
                if escape_next {
                    escape_next = false;
                    continue;
                }

                match ch {
                    '\\' => escape_next = true,
                    '\'' | '"' => in_string = !in_string,
                    '{' if !in_string => brace_count += 1,
                    '}' if !in_string => {
                        brace_count -= 1;
                        if brace_count == 0 {
                            end_pos = i + 1;
                            break;
                        }
                    }
                    _ => {}
                }
            }

            if end_pos > 0 {
                Ok(trimmed[..end_pos].to_string())
            } else {
                // Fallback: take everything up to first comma or end of line
                let fallback_end = trimmed.find(',').unwrap_or(trimmed.len());
                Ok(trimmed[..fallback_end].to_string())
            }
        } else {
            // Single line value - take everything up to comma or newline
            let end_pos = trimmed
                .find(',')
                .or_else(|| trimmed.find('\n'))
                .unwrap_or(trimmed.len());
            Ok(trimmed[..end_pos].trim().to_string())
        }
    }

    fn parse_printconv_hash(
        &self,
        printconv_content: &str,
    ) -> Result<Vec<(String, String)>, String> {
        let mut values = Vec::new();

        // Enhanced regex to extract different key => value formats
        let patterns = [
            // 0x12345678 => 'Value'
            r#"(0x[0-9a-fA-F]+)\s*=>\s*'([^']+)'"#,
            // 123 => 'Value'
            r#"(\d+)\s*=>\s*'([^']+)'"#,
            // -1 => 'Value'
            r#"(-?\d+)\s*=>\s*'([^']+)'"#,
            // Handle double quotes too
            r#"(0x[0-9a-fA-F]+)\s*=>\s*"([^"]+)""#,
            r#"(\d+)\s*=>\s*"([^"]+)""#,
        ];

        for pattern in &patterns {
            let pair_re =
                Regex::new(pattern).map_err(|e| format!("Regex error for {}: {}", pattern, e))?;

            for cap in pair_re.captures_iter(printconv_content) {
                let key = cap[1].to_string();
                let value = cap[2].to_string();
                // Avoid duplicates
                if !values.iter().any(|(k, _)| k == &key) {
                    values.push((key, value));
                }
            }
        }

        // If no matches, but we have hash content, show the raw content for debugging
        if values.is_empty() && printconv_content.contains("=>") {
            // Extract first few characters to show what we couldn't parse
            let debug_content = printconv_content
                .lines()
                .take(3)
                .collect::<Vec<_>>()
                .join(" ")
                .chars()
                .take(100)
                .collect::<String>();
            values.push(("debug".to_string(), debug_content));
        }

        Ok(values)
    }

    fn classify_patterns(&mut self) {
        // Known universal patterns that work across all manufacturers
        let universal_patterns = [
            ("OnOff", vec!["0", "Off", "1", "On"]),
            ("YesNo", vec!["0", "No", "1", "Yes"]),
            (
                "Quality",
                vec!["1", "Best", "2", "Better", "3", "Good", "4", "Normal"],
            ),
            ("FlashMode", vec!["0", "Auto", "1", "On", "2", "Off"]),
            (
                "WhiteBalance",
                vec!["0", "Auto", "1", "Daylight", "2", "Cloudy", "3", "Tungsten"],
            ),
            (
                "MeteringMode",
                vec!["0", "Multi", "1", "Center", "2", "Spot"],
            ),
        ];

        // Group patterns by shared lookup tables (e.g., %userDefStyles, %canonLensTypes)
        let shared_lookups = self.identify_shared_lookup_tables();

        // Create classifications first to avoid borrowing issues
        let mut classifications = Vec::new();

        for (i, pattern) in self.patterns.iter().enumerate() {
            let mut is_universal = false;
            let mut pattern_type = PrintConvType::Lookup("TBD".to_string());

            // Check if this pattern matches a universal one
            for (universal_name, universal_values) in &universal_patterns {
                if Self::pattern_matches_universal(&pattern.values, universal_values) {
                    pattern_type = PrintConvType::Universal(universal_name.to_string());
                    is_universal = true;
                    break;
                }
            }

            if !is_universal {
                // Check if this pattern shares a lookup table with others
                if let Some(shared_lookup_name) = shared_lookups.get(&pattern.tag_id) {
                    pattern_type = PrintConvType::Lookup(shared_lookup_name.clone());
                } else if pattern.values.len() <= 20 && Self::is_simple_lookup(&pattern.values) {
                    let lookup_name = format!("{}{}Lookup", self.manufacturer, pattern.tag_name);
                    pattern_type = PrintConvType::Lookup(lookup_name);
                } else {
                    let function_name = format!("{}{}", self.manufacturer, pattern.tag_name);
                    pattern_type = PrintConvType::Complex(function_name);
                }
            }

            classifications.push((i, pattern_type));
        }

        // Apply classifications
        for (i, pattern_type) in classifications {
            self.patterns[i].pattern_type = pattern_type;
        }
    }

    fn identify_shared_lookup_tables(&self) -> HashMap<String, String> {
        let mut shared_lookups = HashMap::new();
        let mut lookup_table_patterns: HashMap<String, Vec<String>> = HashMap::new();

        // Group patterns by their Perl lookup table reference
        for pattern in &self.patterns {
            for (key, value) in &pattern.values {
                if key == "complex" && (value.contains('%') || value.starts_with("\\%")) {
                    // Extract the lookup table name (e.g., '\%userDefStyles' -> 'userDefStyles')
                    let table_name = value.trim_start_matches('\\').trim_start_matches('%');
                    if !table_name.is_empty() && table_name.len() < 50 {
                        lookup_table_patterns
                            .entry(table_name.to_string())
                            .or_default()
                            .push(pattern.tag_id.clone());
                    }
                }
            }
        }

        // Create shared PrintConvId names for tables used by multiple tags
        for (table_name, tag_ids) in lookup_table_patterns {
            if tag_ids.len() > 1 {
                // Multiple tags use this table - create a shared PrintConvId
                let shared_name = format!("Canon{}", Self::to_camel_case(&table_name));
                for tag_id in tag_ids {
                    shared_lookups.insert(tag_id, shared_name.clone());
                }
            }
        }

        shared_lookups
    }

    fn to_camel_case(input: &str) -> String {
        input
            .split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().collect::<String>() + &chars.collect::<String>()
                    }
                }
            })
            .collect()
    }

    fn pattern_matches_universal(
        pattern_values: &[(String, String)],
        universal_values: &[&str],
    ) -> bool {
        // Simple heuristic: if 50% of the pattern values match universal values, consider it universal
        let mut matches = 0;
        let total = pattern_values.len().min(universal_values.len() / 2);

        for (key, value) in pattern_values.iter().take(total) {
            for chunk in universal_values.chunks(2) {
                if chunk.len() == 2 && key == chunk[0] && value == chunk[1] {
                    matches += 1;
                    break;
                }
            }
        }

        matches as f64 / total as f64 > 0.5
    }

    fn is_simple_lookup(values: &[(String, String)]) -> bool {
        // Simple lookup if all values are straightforward strings without complex logic
        values
            .iter()
            .all(|(_, value)| !value.contains('$') && !value.contains('(') && value.len() < 50)
    }

    fn print_analysis(&self) {
        println!("{} PRINTCONV ANALYSIS:", self.manufacturer.to_uppercase());
        println!("=========================");

        let mut universal_count = 0;
        let mut lookup_count = 0;
        let mut complex_count = 0;
        let mut shared_lookup_groups: HashMap<String, Vec<String>> = HashMap::new();

        // Group patterns by shared PrintConvId for optimization analysis
        for pattern in &self.patterns {
            if let PrintConvType::Lookup(lookup_name) = &pattern.pattern_type {
                if lookup_name.starts_with("Canon") && !lookup_name.contains("Lookup") {
                    // This is a shared lookup table
                    shared_lookup_groups
                        .entry(lookup_name.clone())
                        .or_default()
                        .push(format!("{} '{}'", pattern.tag_id, pattern.tag_name));
                }
            }
        }

        println!("Reusable patterns found:");
        for pattern in &self.patterns {
            if let PrintConvType::Universal(name) = &pattern.pattern_type {
                println!(
                    "- {} '{}' → PrintConvId::{} (existing)",
                    pattern.tag_id, pattern.tag_name, name
                );
                universal_count += 1;
            }
        }

        // Show shared lookup table optimizations
        if !shared_lookup_groups.is_empty() {
            println!("\nShared lookup table optimizations:");
            for (shared_name, tag_patterns) in &shared_lookup_groups {
                println!(
                    "- {} → {} tags can share single implementation:",
                    shared_name,
                    tag_patterns.len()
                );
                for tag_pattern in tag_patterns.iter().take(3) {
                    println!("  • {}", tag_pattern);
                }
                if tag_patterns.len() > 3 {
                    println!("  • ... {} more", tag_patterns.len() - 3);
                }
            }
        }

        println!("\nNew patterns needed:");
        for pattern in &self.patterns {
            match &pattern.pattern_type {
                PrintConvType::Lookup(name) => {
                    // Skip shared lookups since they're shown above
                    if !shared_lookup_groups.contains_key(name) {
                        println!(
                            "- {} '{}' → NEW: PrintConvId::{}",
                            pattern.tag_id, pattern.tag_name, name
                        );
                        if pattern.values.len() <= 5 {
                            print!("  Perl: {{ ");
                            for (i, (key, value)) in pattern.values.iter().enumerate() {
                                if i > 0 {
                                    print!(", ");
                                }
                                print!("{} => '{}'", key, value);
                            }
                            println!(" }}");
                        } else {
                            println!("  Perl: {{ {} entries }}", pattern.values.len());
                        }
                    }
                    lookup_count += 1;
                }
                PrintConvType::Complex(name) => {
                    println!(
                        "- {} '{}' → NEW: PrintConvId::{} (complex)",
                        pattern.tag_id, pattern.tag_name, name
                    );
                    complex_count += 1;
                }
                _ => {}
            }
        }

        let total = self.patterns.len();
        let reusable_pct = if total > 0 {
            (universal_count * 100) / total
        } else {
            0
        };
        let shared_count = shared_lookup_groups
            .values()
            .map(|v| v.len())
            .sum::<usize>();
        let optimization_pct = if total > 0 {
            (shared_count * 100) / total
        } else {
            0
        };

        println!("\nOptimization summary:");
        println!(
            "- {}% patterns reuse existing universal functions",
            reusable_pct
        );
        println!(
            "- {}% patterns benefit from shared lookup table optimization",
            optimization_pct
        );
        println!(
            "- {} shared lookup tables eliminate {} duplicate implementations",
            shared_lookup_groups.len(),
            shared_count.saturating_sub(shared_lookup_groups.len())
        );
        println!(
            "\nTotal patterns: {} ({} universal, {} lookup, {} complex)",
            total, universal_count, lookup_count, complex_count
        );
    }
}

impl Clone for PrintConvAnalyzer {
    fn clone(&self) -> Self {
        Self {
            manufacturer: self.manufacturer.clone(),
            patterns: self.patterns.clone(),
        }
    }
}
