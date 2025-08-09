//! Magic number pattern matching utilities
//!
//! Handles the two-HashMap magic number system with fast literal matching
//! and regex fallback for complex patterns.

use crate::generated::exif_tool::magic_numbers::{LITERAL_MAGIC_NUMBERS, REGEX_MAGIC_NUMBERS};

/// Match magic number patterns using the two-HashMap system
/// First tries literal patterns (fast), then falls back to regex patterns (slower)
pub fn matches_magic_number(file_type: &str, buffer: &[u8]) -> bool {
    // First try literal patterns for fast matching
    if let Some(pattern) = LITERAL_MAGIC_NUMBERS.get(file_type) {
        return buffer.starts_with(pattern);
    }

    // Fall back to regex patterns for complex cases
    if let Some(regex) = REGEX_MAGIC_NUMBERS.get(file_type) {
        return regex.is_match(buffer);
    }

    false
}

/// Last-ditch scan for embedded JPEG/TIFF signatures
/// ExifTool equivalent: ExifTool.pm:2976-2983
pub fn scan_for_embedded_signatures(buffer: &[u8]) -> Option<String> {
    // Look for JPEG signature: \xff\xd8\xff
    if let Some(pos) = buffer.windows(3).position(|w| w == b"\xff\xd8\xff") {
        if pos > 0 {
            eprintln!("Warning: Processing JPEG-like data after unknown {pos}-byte header");
        }
        return Some("JPEG".to_string());
    }

    // Look for TIFF signatures: II*\0 or MM\0*
    if let Some(pos) = buffer
        .windows(4)
        .position(|w| w == b"II*\0" || w == b"MM\0*")
    {
        if pos > 0 {
            eprintln!("Warning: Processing TIFF-like data after unknown {pos}-byte header");
        }
        return Some("TIFF".to_string());
    }

    None
}

/// Validate XMP pattern: \0{0,3}(\xfe\xff|\xff\xfe|\xef\xbb\xbf)?\0{0,3}\s*<
/// ExifTool.pm:1018 - XMP files can start with optional BOM and null bytes, then whitespace, then '<'
#[allow(dead_code)]
pub fn validate_xmp_pattern(buffer: &[u8]) -> bool {
    if buffer.is_empty() {
        return false;
    }

    let mut pos = 0;

    // Skip up to 3 null bytes at the beginning
    while pos < buffer.len() && pos < 3 && buffer[pos] == 0 {
        pos += 1;
    }

    // Check for optional BOM (Byte Order Mark)
    if pos + 3 <= buffer.len() {
        // UTF-8 BOM: EF BB BF
        if buffer[pos..pos + 3] == [0xef, 0xbb, 0xbf] {
            pos += 3;
        }
    }
    if pos + 2 <= buffer.len() {
        // UTF-16 BE BOM: FE FF or UTF-16 LE BOM: FF FE
        if buffer[pos..pos + 2] == [0xfe, 0xff] || buffer[pos..pos + 2] == [0xff, 0xfe] {
            pos += 2;
        }
    }

    // Skip up to 3 more null bytes after BOM
    while pos < buffer.len() && pos < 6 && buffer[pos] == 0 {
        pos += 1;
    }

    // Skip whitespace (space, tab, newline, carriage return)
    while pos < buffer.len()
        && (buffer[pos] == b' '
            || buffer[pos] == b'\t'
            || buffer[pos] == b'\n'
            || buffer[pos] == b'\r')
    {
        pos += 1;
    }

    // Finally, check for '<' character
    pos < buffer.len() && buffer[pos] == b'<'
}
