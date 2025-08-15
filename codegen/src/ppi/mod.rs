//! PPI JSON Parser for ExifTool Expression Processing
//!
//! This module parses PPI (Perl Parsing Interface) JSON structures output by
//! our Perl field extractor (field_extractor.pl) and converts them
//! into Rust source code during the codegen phase.
//!
//! Architecture:
//! - **Codegen-time**: Parse JSON → Normalize AST → Generate Rust functions  
//! - **Runtime**: Generated functions call `ast::` runtime support library
//!
//! Trust ExifTool: All generated code preserves exact Perl evaluation semantics.

pub mod fn_registry;
pub mod normalizer;
pub mod parser;
pub mod rust_generator;
pub mod types;

pub use fn_registry::*;
pub use parser::*;
pub use rust_generator::*;
pub use types::*;
