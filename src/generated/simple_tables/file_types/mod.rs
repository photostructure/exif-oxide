//! file_types table modules

pub mod file_extensions;
pub mod mime_types;
pub mod weak_magic_types;

pub mod file_type_lookup;
pub mod magic_numbers;

// Re-export commonly used functions for convenience
pub use file_extensions::lookup_file_type_extensions;
pub use file_type_lookup::{
    extensions_for_format, get_primary_format, resolve_file_type, supports_format,
};
pub use magic_numbers::{get_magic_file_types, get_magic_pattern};
pub use mime_types::lookup_mime_types;
pub use weak_magic_types::is_weak_magic;
