//! Codegen-time registry for PrintConv/ValueConv mappings
//!
//! This module provides compile-time lookup of Perl expressions to Rust function paths.
//! The registry is used during code generation to emit direct function calls,
//! eliminating runtime lookup overhead.
//!
//! ## Design: No Expression Normalization
//!
//! The registry uses direct string matching without normalization.
//! See docs/design/NORMALIZATION-DECISION.md for the full rationale.
//! In brief: we add multiple registry entries for formatting variations
//! rather than normalizing expressions, eliminating 80,000+ subprocess calls.

pub mod printconv_registry;
pub mod types;
pub mod valueconv_registry;

#[cfg(test)]
mod tests;

// Re-export key types and functions
pub use printconv_registry::{lookup_printconv, lookup_tag_specific_printconv};
pub use types::ValueConvType;
pub use valueconv_registry::classify_valueconv_expression;
