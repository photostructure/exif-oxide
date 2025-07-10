//! File type detection generated code
//!
//! This module contains generated code for file type detection,
//! including file type lookups and magic number patterns.

pub mod file_type_lookup;
pub mod magic_number_patterns;

// Re-export commonly used functions
pub use file_type_lookup::{
    extensions_for_format, get_primary_format, resolve_file_type, supports_format,
};
pub use magic_number_patterns::{
    get_magic_file_types, get_magic_number_pattern as get_magic_pattern,
};
