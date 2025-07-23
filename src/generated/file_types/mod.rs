//! File type detection module
//!
//! This module contains generated code for file type detection.

pub mod file_type_lookup;
// pub mod magic_number_patterns; // TODO: Generate magic number patterns

// Re-export commonly used items
pub use file_type_lookup::{
    extensions_for_format, get_primary_format, resolve_file_type, supports_format,
};
// pub use magic_number_patterns::{detect_file_type_by_magic, MAGIC_NUMBER_PATTERNS}; // TODO: Generate magic number patterns
