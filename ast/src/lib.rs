//! Shared AST Infrastructure for exif-oxide
//!
//! This crate provides Rust representations of Perl PPI AST nodes and conversion logic
//! for ExifTool expressions. Used by both P08 (codegen AST-to-Rust) and P07 (unified
//! expression evaluation system).
//!
//! See docs/todo/P08-ppi-ast-foundation.md and P07-unified-expression-system.md

pub mod exif_context;
pub mod ppi_converter;
pub mod ppi_types;

pub use exif_context::*;
pub use ppi_converter::*;
pub use ppi_types::*;
