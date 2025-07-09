//! Magic number patterns for file type detection
//!
//! Generated at: Mon Jul  7 03:35:20 2025 GMT

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Magic number patterns for file type detection
static MAGIC_PATTERNS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(HashMap::new);

/// Get magic number pattern for a file type
pub fn get_magic_pattern(file_type: &str) -> Option<&'static str> {
    MAGIC_PATTERNS.get(file_type).copied()
}

/// Get all file types that have magic number patterns
pub fn get_magic_file_types() -> Vec<&'static str> {
    MAGIC_PATTERNS.keys().copied().collect()
}
