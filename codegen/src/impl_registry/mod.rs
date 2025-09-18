//! Codegen-time implementation registry for ExifTool patterns
//!
//! This module provides compile-time lookup of ExifTool patterns (PrintConv/ValueConv
//! expressions, function calls, complex scripts) to Rust implementation paths.
//! The registry is used during code generation to emit direct function calls,
//! eliminating runtime lookup overhead.
//!
//! ## Registries
//!
//! - **PrintConv/ValueConv**: Conversion expressions and lookup tables
//! - **Functions**: Complex ExifTool function calls and Perl builtins
//! - **Custom Scripts**: Multi-line conditional logic and complex patterns
//!
//! ## Design: No Expression Normalization
//!
//! The registry uses direct string matching without normalization.
//! See docs/design/NORMALIZATION-DECISION.md for the full rationale.
//! In brief: we add multiple registry entries for formatting variations
//! rather than normalizing expressions, eliminating 80,000+ subprocess calls.

pub mod fallback_helper;
pub mod function_registry;
pub mod printconv_registry;
pub mod types;
pub mod valueconv_registry;

#[cfg(test)]
mod tests;

// Re-export key types and functions for external use
pub use fallback_helper::try_registry_lookup;
pub use function_registry::{lookup_function, FunctionImplementation, ModuleFunction};
pub use printconv_registry::{lookup_printconv, lookup_tag_specific_printconv};
pub use types::ValueConvType;
pub use valueconv_registry::classify_valueconv_expression;
