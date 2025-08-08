//! MagicNumberStrategy for processing ExifTool's magic number regex patterns
//!
//! This strategy handles the %magicNumber symbol which contains regex patterns
//! for binary file type detection (e.g., "JPEG":"\\xff\\xd8\\xff")
//!
//! The working approach converts simple literal byte patterns to byte arrays
//! and marks complex patterns as "complex" with empty arrays.

use anyhow::Result;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use tracing::{debug, info};

use super::{ExtractionContext, ExtractionStrategy, GeneratedFile};
use crate::field_extractor::FieldSymbol;

/// Strategy for processing ExifTool's magicNumber regex patterns
pub struct MagicNumberStrategy {
    /// Collected magic number data by module
    magic_data: HashMap<String, MagicNumberData>,
}

/// Magic number patterns extracted from ExifTool
#[derive(Debug, Clone)]
struct MagicNumberData {
    /// Symbol name (should be "magicNumber")
    name: String,

    /// Module name (should be "ExifTool")
    module: String,

    /// File type to regex pattern mappings
    patterns: HashMap<String, String>,
}

impl MagicNumberStrategy {
    /// Create new MagicNumberStrategy
    pub fn new() -> Self {
        Self {
            magic_data: HashMap::new(),
        }
    }

    /// Convert simple Perl regex patterns to byte arrays
    /// Following the working implementation's approach - be very conservative
    fn convert_pattern_to_bytes(&self, file_type: &str, pattern: &str) -> Option<Vec<u8>> {
        // Only handle the most common, simple patterns that we're certain about
        // Everything else should be marked as complex

        match file_type {
            // Simple literal byte sequences - direct conversion from working implementation
            "JPEG" if pattern == "\\xff\\xd8\\xff" => Some(vec![0xff, 0xd8, 0xff]),
            "BMP" if pattern == "BM" => Some(vec![0x42, 0x4d]), // BM
            "GIF" if pattern == "GIF8[79]a" => Some(vec![0x47, 0x49, 0x46, 0x38, 0x39, 0x61]), // GIF89a (most common)
            "PNG" if pattern == "(\\x89P|\\x8aM|\\x8bJ)NG\\r\\n\\x1a\\n" => {
                // Use most common variant: \x89PNG\r\n\x1a\n
                Some(vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a])
            }
            "TIFF" if pattern == "(II|MM)" => Some(vec![0x49, 0x49]), // II (little-endian)
            "ZIP" if pattern == "PK\\x03\\x04" => Some(vec![0x50, 0x4b, 0x03, 0x04]),
            "PDF" if pattern == "\\s*%PDF-\\d+\\.\\d+" => Some(vec![0x25, 0x50, 0x44, 0x46]), // %PDF
            "GZIP" if pattern == "\\x1f\\x8b\\x08" => Some(vec![0x1f, 0x8b, 0x08]),

            // Only try parsing hex patterns for very simple cases (no regex metacharacters)
            _ => {
                if self.is_complex_pattern(pattern) {
                    None // Complex pattern - use empty array
                } else {
                    self.parse_simple_hex_pattern(pattern)
                }
            }
        }
    }

    /// Parse patterns that are simple hex literal sequences
    /// Based on the working implementation's conversion logic
    fn parse_simple_hex_pattern(&self, pattern: &str) -> Option<Vec<u8>> {
        // Only handle patterns that are pure literal byte sequences
        // No regex metacharacters, alternations, or quantifiers
        if self.is_complex_pattern(pattern) {
            return None;
        }

        let mut bytes = Vec::new();
        let mut chars = pattern.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '\\' => {
                    if let Some(next_ch) = chars.next() {
                        match next_ch {
                            'x' => {
                                // Parse \xNN hex escape
                                let hex1 = chars.next()?;
                                let hex2 = chars.next()?;
                                let hex_str = format!("{}{}", hex1, hex2);
                                let byte = u8::from_str_radix(&hex_str, 16).ok()?;
                                bytes.push(byte);
                            }
                            'r' => bytes.push(0x0d), // \r
                            'n' => bytes.push(0x0a), // \n
                            't' => bytes.push(0x09), // \t
                            '0' => bytes.push(0x00), // \0
                            _ => {
                                // Literal escaped character
                                bytes.push(next_ch as u8);
                            }
                        }
                    } else {
                        return None; // Incomplete escape
                    }
                }
                _ => {
                    // Literal ASCII character
                    if ch.is_ascii() {
                        bytes.push(ch as u8);
                    } else {
                        // Handle Unicode characters like û in BPG pattern
                        match ch {
                            'û' => bytes.push(0xfb), // U+00FB -> 0xfb
                            'é' => bytes.push(0xe9), // U+00E9 -> 0xe9
                            'ñ' => bytes.push(0xf1), // U+00F1 -> 0xf1
                            _ => return None,        // Unknown Unicode character
                        }
                    }
                }
            }
        }

        Some(bytes)
    }

    /// Check if pattern contains regex metacharacters that make it complex
    fn is_complex_pattern(&self, pattern: &str) -> bool {
        // Be very conservative - patterns with ANY regex metacharacters are complex
        // Only convert the patterns we explicitly handle in convert_pattern_to_bytes
        pattern.chars().any(|c| {
            matches!(
                c,
                '[' | ']'
                    | '('
                    | ')'
                    | '|'
                    | '+'
                    | '*'
                    | '?'
                    | '^'
                    | '$'
                    | '{'
                    | '}'
                    | '.'
                    | '\\'
                    | '\0' // Null bytes also make it complex
            )
        })
    }

    /// Check if pattern is only simple hex escapes and literal characters
    fn is_simple_hex_only_pattern(&self, pattern: &str) -> bool {
        let mut chars = pattern.chars().peekable();
        while let Some(ch) = chars.next() {
            match ch {
                '\\' => {
                    if let Some(next_ch) = chars.next() {
                        match next_ch {
                            'x' => {
                                // Must be followed by exactly 2 hex digits
                                if chars.next().is_some() && chars.next().is_some() {
                                    continue;
                                } else {
                                    return false;
                                }
                            }
                            'r' | 'n' | 't' | '0' => continue, // Simple escape sequences
                            _ => return false,                 // Other escapes make it complex
                        }
                    } else {
                        return false; // Incomplete escape
                    }
                }
                c if c.is_ascii_graphic() || c.is_ascii_whitespace() => continue,
                'û' | 'é' | 'ñ' => continue, // Known Unicode characters we can handle
                _ => return false,           // Other characters make it complex
            }
        }
        true
    }

    /// Generate Rust code for magic number patterns
    /// Following the exact format of the working implementation
    fn generate_magic_number_code(&self, data: &MagicNumberData) -> String {
        let mut code = String::new();

        // File header - matches working implementation format
        code.push_str(
            "//! Magic number regex patterns generated from ExifTool's magicNumber hash\n",
        );
        code.push_str("//!\n");
        code.push_str(&format!("//! Total patterns: {}\n", data.patterns.len()));
        code.push_str("//! Source: ExifTool.pm %magicNumber\n");
        code.push_str("//!\n");
        code.push_str(
            "//! IMPORTANT: These patterns are converted from Perl regex to Rust-compatible\n",
        );
        code.push_str("//! byte patterns for use with the regex crate.\n\n");

        // Imports - exact format from working implementation
        code.push_str("use std::collections::HashMap;\n");
        code.push_str("use std::sync::LazyLock;\n\n");

        // Comment section showing all patterns (for reference)
        code.push_str("/// Raw magic number patterns from ExifTool's %magicNumber hash\n");
        code.push_str("static MAGIC_PATTERN_DATA: &[(&str, &[u8])] = &[\n");

        // Sort patterns for consistent output
        let mut patterns: Vec<_> = data.patterns.iter().collect();
        patterns.sort_by_key(|&(k, _)| k);

        // First, generate comments showing original patterns (with control characters escaped)
        for (file_type, pattern) in &patterns {
            let escaped_pattern = pattern
                .replace('\0', "\\0") // Null bytes
                .replace('\n', "\\n") // Newlines
                .replace('\r', "\\r") // Carriage returns
                .replace('\t', "\\t"); // Tabs
            code.push_str(&format!("    // {}: ^{}\n", file_type, escaped_pattern));
        }
        code.push_str("];\n\n");

        // Generate the main REGEX_PATTERNS HashMap - exact format from working implementation
        code.push_str("/// Magic number patterns as byte slices for quick comparison\n");
        code.push_str("/// This matches file_detection.rs expectation of REGEX_PATTERNS\n");
        code.push_str("pub static REGEX_PATTERNS: LazyLock<HashMap<&'static str, &'static [u8]>> = LazyLock::new(|| {\n");
        code.push_str("    let mut map = HashMap::new();\n\n");

        // Generate byte array entries
        for (file_type, pattern) in &patterns {
            if let Some(byte_pattern) = self.convert_pattern_to_bytes(file_type, pattern) {
                // Generate byte array in the exact format of the working implementation
                let byte_list = byte_pattern
                    .iter()
                    .map(|b| format!("{}", b))
                    .collect::<Vec<_>>()
                    .join(", ");

                code.push_str(&format!(
                    "    map.insert(\"{}\", &[{}][..]);\n",
                    file_type, byte_list
                ));
            } else {
                // Complex patterns get empty arrays with comments - exact format from working implementation
                let escaped_pattern = pattern
                    .replace('\0', "\\0") // Escape null bytes
                    .replace('\n', "\\n") // Escape newlines
                    .replace('\r', "\\r") // Escape carriage returns
                    .replace('\t', "\\t") // Escape tabs
                    .replace('\\', "\\\\") // Escape backslashes
                    .replace('"', "\\\""); // Escape quotes
                code.push_str(&format!(
                    "    // Complex pattern for {}: {}\n",
                    file_type, escaped_pattern
                ));
                code.push_str(&format!("    map.insert(\"{}\", &[][..]);\n", file_type));
            }
        }

        code.push_str("\n    map\n");
        code.push_str("});\n\n");

        // Generate helper functions - same as working implementation
        self.generate_helper_functions(&mut code);

        code
    }

    /// Generate helper functions - exact format from working implementation
    fn generate_helper_functions(&self, code: &mut String) {
        code.push_str("/// Test if a byte buffer matches a magic number pattern\n");
        code.push_str("pub fn matches_magic_number(file_type: &str, buffer: &[u8]) -> bool {\n");
        code.push_str("    if let Some(pattern) = REGEX_PATTERNS.get(file_type) {\n");
        code.push_str("        buffer.starts_with(pattern)\n");
        code.push_str("    } else {\n");
        code.push_str("        false\n");
        code.push_str("    }\n");
        code.push_str("}\n\n");

        code.push_str("/// Get all file types with magic number patterns\n");
        code.push_str("pub fn get_magic_file_types() -> Vec<&'static str> {\n");
        code.push_str("    REGEX_PATTERNS.keys().copied().collect()\n");
        code.push_str("}\n");
    }
}

impl ExtractionStrategy for MagicNumberStrategy {
    fn name(&self) -> &'static str {
        "MagicNumberStrategy"
    }

    fn can_handle(&self, symbol: &FieldSymbol) -> bool {
        // Only handle magicNumber from ExifTool module
        if symbol.name != "magicNumber" || symbol.module != "ExifTool" {
            return false;
        }

        // Must be a hash with string values (regex patterns)
        if let JsonValue::Object(map) = &symbol.data {
            if map.is_empty() {
                return false;
            }

            // All values must be strings (regex patterns)
            let all_strings = map.values().all(|v| v.is_string());

            debug!(
                "magicNumber pattern check: all_strings={}, size={}",
                all_strings,
                map.len()
            );

            all_strings
        } else {
            false
        }
    }

    fn extract(&mut self, symbol: &FieldSymbol, context: &mut ExtractionContext) -> Result<()> {
        info!("🔧 Extracting magicNumber regex patterns");

        if let JsonValue::Object(map) = &symbol.data {
            let mut patterns = HashMap::new();

            for (file_type, value) in map {
                if let Some(pattern_str) = value.as_str() {
                    patterns.insert(file_type.clone(), pattern_str.to_string());
                }
            }

            let data = MagicNumberData {
                name: symbol.name.clone(),
                module: symbol.module.clone(),
                patterns,
            };

            info!("    ✓ Parsed {} magic number patterns", data.patterns.len());

            self.magic_data.insert(symbol.module.clone(), data);

            context.log_strategy_selection(
                symbol,
                self.name(),
                "Hash with regex string values for magic number detection",
            );
        }

        Ok(())
    }

    fn finish_module(&mut self, _module_name: &str) -> Result<()> {
        // No per-module finalization needed
        Ok(())
    }

    fn finish_extraction(&mut self) -> Result<Vec<GeneratedFile>> {
        let mut files = Vec::new();

        for (_module, data) in &self.magic_data {
            let content = self.generate_magic_number_code(data);

            files.push(GeneratedFile {
                path: "exiftool_pm/regex_patterns.rs".to_string(),
                content,
                strategy: self.name().to_string(),
            });

            info!(
                "📁 Generated regex_patterns.rs with {} magic number patterns",
                data.patterns.len()
            );
        }

        Ok(files)
    }
}

impl Default for MagicNumberStrategy {
    fn default() -> Self {
        Self::new()
    }
}
