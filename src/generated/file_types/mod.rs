//! file_types table modules

pub mod file_extensions;
pub mod file_type_lookup;
pub mod magic_number_patterns;
pub mod magic_numbers;
pub mod mime_types;
pub mod weak_magic_types;

// Re-export commonly used functions
pub use file_type_lookup::{
    extensions_for_format, get_primary_format, resolve_file_type, supports_format,
};
pub use magic_number_patterns::{get_magic_file_types, get_magic_number_pattern};
pub use magic_numbers::get_magic_pattern;
pub use mime_types::lookup_mime_types;
