//! Codegen-time registry for PrintConv/ValueConv mappings
//!
//! This module provides compile-time lookup of Perl expressions to Rust function paths.
//! The registry is used during code generation to emit direct function calls,
//! eliminating runtime lookup overhead.

pub mod normalization;
pub mod printconv_registry;
pub mod types;
pub mod valueconv_registry;

#[cfg(test)]
mod tests;

// Re-export key types and functions
pub use printconv_registry::{lookup_printconv, lookup_tag_specific_printconv};
pub use valueconv_registry::lookup_valueconv;
