//! Codegen library for exif-oxide
//!
//! This library provides code generation capabilities for translating
//! ExifTool Perl modules to Rust code.

pub mod common;
pub mod expression_compiler;
pub mod field_extractor;
pub mod file_operations;
pub mod impl_registry;
pub mod ppi; // PPI JSON parsing for codegen-time AST processing
pub mod strategies;
pub mod types;
pub mod validation;

// Re-export commonly used types
pub use field_extractor::{FieldExtractionStats, FieldExtractor, FieldMetadata, FieldSymbol};
