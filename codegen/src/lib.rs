//! Codegen library for exif-oxide
//!
//! This library provides code generation capabilities for translating
//! ExifTool Perl modules to Rust code.

pub mod common;
// pub mod expression_compiler; // DELETED: PPI AST handles all Perl interpretation at build time
pub mod field_extractor;
pub mod file_operations;
// pub mod generate_unsupported_tests; // Module file doesn't exist yet
pub mod impl_registry;
pub mod ppi; // PPI JSON parsing for codegen-time AST processing
pub mod strategies;
pub mod types;
pub mod validation;

#[cfg(test)]
mod test_unary_negation;

// Re-export commonly used types
pub use field_extractor::{FieldExtractor, FieldMetadata, FieldSymbol};
