//! File type detection module
//!
//! This module contains generated code for file type detection.

pub mod file_type_lookup;

// Re-export commonly used items
pub use file_type_lookup::{
    extensions_for_format, get_primary_format, resolve_file_type, supports_format,
};

// Import file type extensions and regex patterns from their source-based location (ExifTool.pm)
pub use crate::generated::ExifTool_pm::filetypeext::{
    lookup_file_type_extensions, FILE_TYPE_EXTENSIONS,
};
pub use crate::generated::ExifTool_pm::regex_patterns::{
    detect_file_type_by_regex, REGEX_PATTERNS,
};
