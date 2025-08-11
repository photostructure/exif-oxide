//! Codegen library for exif-oxide
//!
//! This library provides code generation capabilities for translating
//! ExifTool Perl modules to Rust code.

// AST infrastructure moved to shared ast crate for P07/P08
pub mod common;
pub mod expression_compiler;
pub mod field_extractor;
pub mod file_operations;
pub mod impl_registry;
pub mod strategies;
pub mod types;
pub mod validation;

// Re-export commonly used types
pub use field_extractor::{FieldExtractionStats, FieldExtractor, FieldMetadata, FieldSymbol};
