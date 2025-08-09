//! Common utility functions for code generation

/// Classification of magic number patterns
#[derive(Debug, Clone, PartialEq)]
pub enum PatternClassification {
    /// Simple literal byte sequence that can be directly matched
    Literal(Vec<u8>),
    /// Regex pattern that needs regex compilation
    Regex,
}

/// Escape a string for use in Rust code
pub fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\") // Must be first to avoid double-escaping
        .replace('\"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Check if a string contains characters that require raw string literal in Rust
pub fn needs_raw_string(s: &str) -> bool {
    // Check for backslash followed by common escape sequences that aren't valid in Rust
    s.contains("\\s") || s.contains("\\S") || s.contains("\\d") || s.contains("\\D") ||
    s.contains("\\w") || s.contains("\\W") || s.contains("\\b") || s.contains("\\B") ||
    s.contains("\\A") || s.contains("\\z") || s.contains("\\Z") ||
    // Also check for regex patterns
    s.contains("=~") || s.contains("!~")
}

/// Format a string for use in Rust code, using raw string if needed
pub fn format_rust_string(s: &str) -> String {
    if needs_raw_string(s) {
        // Use raw string literal
        if s.contains('"') && !s.contains('#') {
            // Use r#"..."# format
            format!("r#\"{s}\"#")
        } else if s.contains('"') && s.contains('#') {
            // Need to use more # symbols if the string contains #
            let mut hashes = "#".to_string();
            while s.contains(&format!("\"{hashes}")) {
                hashes.push('#');
            }
            format!("r{hashes}\"{s}\"{hashes}")
        } else {
            // Simple raw string
            format!("r\"{s}\"")
        }
    } else {
        // Use regular escaped string
        format!("\"{}\"", escape_string(s))
    }
}

/// Classify ExifTool magic number patterns from raw bytes
/// Raw bytes are the literal escape sequence characters from Perl, not actual bytes
pub fn classify_magic_pattern(raw_bytes: &[u8]) -> PatternClassification {
    // Simple regex detection on raw bytes: does it contain [, {, or (? Then it's regex!
    // Check for ASCII bytes directly to avoid UTF-8 conversion issues
    if raw_bytes.contains(&b'[') || raw_bytes.contains(&b'{') || raw_bytes.contains(&b'(') {
        return PatternClassification::Regex;
    }

    // Try to parse escape sequences to actual bytes
    // Handle raw bytes directly without UTF-8 conversion to preserve all byte values
    if let Some(actual_bytes) = parse_perl_escape_sequences_from_raw_bytes(raw_bytes) {
        PatternClassification::Literal(actual_bytes)
    } else {
        // Failed to parse - treat as regex
        PatternClassification::Regex
    }
}

/// Parse Perl escape sequences directly from raw bytes to avoid UTF-8 conversion issues
/// This handles cases like BPG pattern where byte 251 is not valid UTF-8
pub fn parse_perl_escape_sequences_from_raw_bytes(raw_bytes: &[u8]) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut i = 0;

    while i < raw_bytes.len() {
        match raw_bytes[i] {
            b'\\' if i + 1 < raw_bytes.len() => {
                match raw_bytes[i + 1] {
                    b'x' if i + 3 < raw_bytes.len() => {
                        // Parse \xNN hex escape
                        let hex1 = raw_bytes[i + 2] as char;
                        let hex2 = raw_bytes[i + 3] as char;
                        let hex_str = format!("{hex1}{hex2}");
                        if let Ok(byte_val) = u8::from_str_radix(&hex_str, 16) {
                            bytes.push(byte_val);
                            i += 4; // Skip \xNN
                        } else {
                            return None; // Invalid hex
                        }
                    }
                    b'0' => {
                        bytes.push(0x00); // \0
                        i += 2;
                    }
                    b'r' => {
                        bytes.push(0x0d); // \r
                        i += 2;
                    }
                    b'n' => {
                        bytes.push(0x0a); // \n
                        i += 2;
                    }
                    b't' => {
                        bytes.push(0x09); // \t
                        i += 2;
                    }
                    other => {
                        // Literal escaped character (like \+ -> +, \\ -> \)
                        bytes.push(other);
                        i += 2;
                    }
                }
            }
            other => {
                // Regular character (including non-ASCII bytes like 251)
                bytes.push(other);
                i += 1;
            }
        }
    }

    Some(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_magic_pattern_simple_ascii() {
        // BMP pattern: "BM" -> [66, 77]
        let raw_bytes = vec![66, 77];
        let result = classify_magic_pattern(&raw_bytes);
        assert_eq!(result, PatternClassification::Literal(vec![66, 77]));
    }

    #[test]
    fn test_classify_magic_pattern_simple_hex_escapes() {
        // ASF pattern starts with \x30\x26... -> [92,120,51,48,92,120,50,54,...]
        let raw_bytes = vec![92, 120, 51, 48, 92, 120, 50, 54]; // "\x30\x26"
        let result = classify_magic_pattern(&raw_bytes);
        assert_eq!(result, PatternClassification::Literal(vec![0x30, 0x26]));
    }

    #[test]
    fn test_classify_magic_pattern_with_unicode() {
        // BPG pattern: "BPG" + 0xFB byte -> [66, 80, 71, 251]
        let raw_bytes = vec![66, 80, 71, 251];
        let result = classify_magic_pattern(&raw_bytes);
        assert_eq!(
            result,
            PatternClassification::Literal(vec![66, 80, 71, 251])
        );
    }

    #[test]
    fn test_classify_magic_pattern_regex_brackets() {
        // AAC pattern: "\xff[\xf0\xf1]" -> [92,120,102,102,91,92,120,102,48,92,120,102,49,93]
        let raw_bytes = vec![
            92, 120, 102, 102, 91, 92, 120, 102, 48, 92, 120, 102, 49, 93,
        ];
        let result = classify_magic_pattern(&raw_bytes);
        assert_eq!(result, PatternClassification::Regex);
    }

    #[test]
    fn test_classify_magic_pattern_regex_parens() {
        // AIFF pattern starts with "(FORM" -> [40, 70, 79, 82, 77, ...]
        let raw_bytes = vec![40, 70, 79, 82, 77]; // "(FORM"
        let result = classify_magic_pattern(&raw_bytes);
        assert_eq!(result, PatternClassification::Regex);
    }

    #[test]
    fn test_classify_magic_pattern_regex_quantifiers() {
        // AA pattern starts with ".{4}" -> [46, 123, 52, 125, ...]
        let raw_bytes = vec![46, 123, 52, 125]; // ".{4}"
        let result = classify_magic_pattern(&raw_bytes);
        assert_eq!(result, PatternClassification::Regex);
    }

    #[test]
    fn test_classify_magic_pattern_gzip() {
        // GZIP pattern: '\x1f\x8b\x08' -> [92,120,49,102,92,120,56,98,92,120,48,56]
        let raw_bytes = vec![92, 120, 49, 102, 92, 120, 56, 98, 92, 120, 48, 56]; // "\x1f\x8b\x08"
        let result = classify_magic_pattern(&raw_bytes);
        assert_eq!(result, PatternClassification::Literal(vec![31, 139, 8])); // 0x1f, 0x8b, 0x08
    }

    #[test]
    fn test_classify_magic_pattern_aa() {
        // AA pattern: '.{4}\x57\x90\x75\x36' -> [46,123,52,125,92,120,53,55,92,120,57,48,92,120,55,53,92,120,51,54]
        let raw_bytes = vec![
            46, 123, 52, 125, 92, 120, 53, 55, 92, 120, 57, 48, 92, 120, 55, 53, 92, 120, 51, 54,
        ]; // ".{4}\x57\x90\x75\x36"
        let result = classify_magic_pattern(&raw_bytes);
        assert_eq!(result, PatternClassification::Regex); // Contains {}, so it's a regex pattern
    }

    #[test]
    fn test_classify_magic_pattern_itc() {
        // ITC pattern: '.{4}itch' -> [46,123,52,125,105,116,99,104]
        let raw_bytes = vec![46, 123, 52, 125, 105, 116, 99, 104]; // ".{4}itch"
        let result = classify_magic_pattern(&raw_bytes);
        assert_eq!(result, PatternClassification::Regex); // Contains {}, so it's a regex pattern
    }

    #[test]
    fn test_classify_magic_pattern_bmp() {
        // BMP pattern: 'BM' -> [66, 77]
        let raw_bytes = vec![66, 77]; // "BM"
        let result = classify_magic_pattern(&raw_bytes);
        assert_eq!(result, PatternClassification::Literal(vec![66, 77]));
    }

    #[test]
    fn test_classify_magic_pattern_jpeg() {
        // JPEG pattern: '\xff\xd8\xff' -> [92,120,102,102,92,120,100,56,92,120,102,102]
        let raw_bytes = vec![92, 120, 102, 102, 92, 120, 100, 56, 92, 120, 102, 102]; // "\xff\xd8\xff"
        let result = classify_magic_pattern(&raw_bytes);
        assert_eq!(result, PatternClassification::Literal(vec![255, 216, 255]));
        // 0xff, 0xd8, 0xff
    }

    #[test]
    fn test_classify_magic_pattern_aac_with_brackets() {
        // AAC pattern: '\xff[\xf0\xf1]' -> [92,120,102,102,91,92,120,102,48,92,120,102,49,93]
        let raw_bytes = vec![
            92, 120, 102, 102, 91, 92, 120, 102, 48, 92, 120, 102, 49, 93,
        ]; // "\xff[\xf0\xf1]"
        let result = classify_magic_pattern(&raw_bytes);
        assert_eq!(result, PatternClassification::Regex); // Contains [], so it's a regex pattern
    }

    #[test]
    fn test_classify_magic_pattern_mov_with_parens() {
        // MOV pattern (partial): '(free|skip' -> [40,102,114,101,101,124,115,107,105,112]
        let raw_bytes = vec![40, 102, 114, 101, 101, 124, 115, 107, 105, 112]; // "(free|skip"
        let result = classify_magic_pattern(&raw_bytes);
        assert_eq!(result, PatternClassification::Regex); // Contains (), so it's a regex pattern
    }

    #[test]
    fn test_classify_magic_pattern_xcf() {
        // XCF pattern: 'gimp xcf ' -> [103,105,109,112,32,120,99,102,32]
        let raw_bytes = vec![103, 105, 109, 112, 32, 120, 99, 102, 32]; // "gimp xcf "
        let result = classify_magic_pattern(&raw_bytes);
        assert_eq!(
            result,
            PatternClassification::Literal(vec![103, 105, 109, 112, 32, 120, 99, 102, 32])
        );
    }

    // Removed test for obsolete old-architecture function is_simple_literal_pattern
}
