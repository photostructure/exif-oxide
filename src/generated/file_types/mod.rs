//! File type detection module
//!
//! This module contains generated code for file type detection.

pub mod file_type_lookup;
pub mod magic_number_patterns;

// Re-export commonly used items
pub use file_type_lookup::{
    extensions_for_format, get_primary_format, lookup_file_type_by_extension, resolve_file_type,
    supports_format, FILE_TYPE_EXTENSIONS,
};
pub use magic_number_patterns::{
    get_magic_file_types, get_magic_number_pattern, matches_magic_number,
};

// Import regex patterns from their source-based location (ExifTool.pm)
pub use crate::generated::ExifTool_pm::regex_patterns::{
    detect_file_type_by_regex, REGEX_PATTERNS,
};
