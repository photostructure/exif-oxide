//! File type detection and lookup functionality
//!
//! This module contains generated code for file type detection based on
//! ExifTool's fileTypeLookup infrastructure.

pub mod file_type_lookup;
pub mod magic_numbers;

#[cfg(test)]
mod file_type_lookup_tests;

pub use file_type_lookup::*;
pub use magic_numbers::*;
