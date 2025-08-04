//! Codegen library for exif-oxide
//!
//! This library provides code generation capabilities for translating
//! ExifTool Perl modules to Rust code.

pub mod common;
pub mod config;
pub mod conv_registry;
pub mod discovery;
pub mod expression_compiler;
pub mod extraction;
pub mod extractors;
pub mod file_operations;
pub mod generators;
pub mod schemas;
pub mod table_processor;
pub mod field_extractor;
pub mod strategies;
pub mod validation;

// Re-export commonly used types
pub use field_extractor::{FieldExtractor, FieldSymbol, FieldMetadata, FieldExtractionStats};