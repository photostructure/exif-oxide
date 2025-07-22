//! File type detection module
//!
//! This module contains generated code for file type detection.

pub mod file_type_lookup;
pub mod magic_number_patterns;

// Re-export commonly used items
pub use file_type_lookup::{lookup_file_type_by_extension, FILE_TYPE_EXTENSIONS};
pub use magic_number_patterns::{detect_file_type_by_magic, MAGIC_NUMBER_PATTERNS};
