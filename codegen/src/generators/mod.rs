//! Code generators for different types of ExifTool data

pub mod tags;
pub mod composite_tags;
pub mod conversion_refs;
pub mod supported_tags;
pub mod simple_tables;
pub mod module;

pub use tags::generate_tag_table;
pub use composite_tags::generate_composite_tag_table;
pub use conversion_refs::generate_conversion_refs;
pub use supported_tags::generate_supported_tags;
pub use simple_tables::generate_simple_tables;
pub use module::generate_mod_file;