//! File type detection and identification

pub mod file_type_lookup;
pub mod magic_numbers;
pub mod mime_types;

pub use file_type_lookup::*;
pub use magic_numbers::*;
pub use mime_types::*;
