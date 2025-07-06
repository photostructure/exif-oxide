//! file_types table modules

pub mod file_type_lookup;
pub mod magic_numbers;
pub mod mime_types;

// Re-export commonly used functions for convenience
pub use file_type_lookup::{
    extensions_for_format, get_primary_format, lookup_file_type_lookup, resolve_file_type,
    supports_format,
};
pub use magic_numbers::lookup_magic_number_patterns;
pub use mime_types::lookup_mime_types;
