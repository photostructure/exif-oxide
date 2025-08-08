//! Codegen library for exif-oxide
//!
//! This library provides code generation capabilities for translating
//! ExifTool Perl modules to Rust code.

pub mod common;
pub mod conv_registry;
pub mod expression_compiler;
pub mod field_extractor;
pub mod file_operations;
pub mod strategies;
pub mod types;
pub mod validation;

// Re-export commonly used types
pub use field_extractor::{FieldExtractionStats, FieldExtractor, FieldMetadata, FieldSymbol};
