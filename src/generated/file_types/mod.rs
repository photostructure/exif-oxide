//! File type detection module
//!
//! This module contains generated code for file type detection.

pub mod file_type_lookup;
pub mod magic_number_patterns;

// Re-export commonly used items
pub use file_type_lookup::{
    extensions_for_format, get_primary_format, resolve_file_type, supports_format,
};
pub use magic_number_patterns::{get_magic_file_types, matches_magic_number};
