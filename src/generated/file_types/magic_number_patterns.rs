//! Magic number regex patterns generated from ExifTool's magicNumber hash
//!
//! Generated at: Wed Jul 9 02:29:26 PM PDT 2025
//! Total patterns: 0
//! Compatibility: Handled by simplified extract.pl

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Magic number regex patterns for file type detection
/// These patterns are validated to be compatible with the Rust regex crate
static MAGIC_NUMBER_PATTERNS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    
    HashMap::new()
});

/// Compatibility status for each magic number pattern
static PATTERN_COMPATIBILITY: Lazy<HashMap<&'static str, bool>> = Lazy::new(|| {
    
    HashMap::new()
});

/// Get magic number pattern for a file type
pub fn get_magic_number_pattern(file_type: &str) -> Option<&'static str> {
    MAGIC_NUMBER_PATTERNS.get(file_type).copied()
}

/// Check if a file type has a Rust-compatible magic number pattern
pub fn is_pattern_compatible(file_type: &str) -> bool {
    PATTERN_COMPATIBILITY
        .get(file_type)
        .copied()
        .unwrap_or(false)
}

/// Get all file types with magic number patterns
pub fn get_magic_file_types() -> Vec<&'static str> {
    MAGIC_NUMBER_PATTERNS.keys().copied().collect()
}

/// Get all file types with Rust-compatible patterns
pub fn get_compatible_file_types() -> Vec<&'static str> {
    PATTERN_COMPATIBILITY
        .iter()
        .filter(|(_, &compatible)| compatible)
        .map(|(&file_type, _)| file_type)
        .collect()
}
