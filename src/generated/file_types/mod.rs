//! File type detection module
//!
//! This module contains generated code for file type detection.

pub mod file_type_lookup;
// Re-export commonly used items
pub use file_type_lookup::{
    extensions_for_format, get_primary_format, lookup_file_type_by_extension, resolve_file_type,
    supports_format, FILE_TYPE_EXTENSIONS,
};
