//! Code generators for different types of ExifTool data

pub mod tags;
pub mod composite_tags;
pub mod conversion_refs;
pub mod supported_tags;
pub mod module;

// Modular architecture
pub mod lookup_tables;
pub mod file_detection;
pub mod data_sets;


pub use tags::generate_tag_table;
pub use composite_tags::generate_composite_tag_table;
pub use conversion_refs::generate_conversion_refs;
pub use supported_tags::generate_supported_tags;
pub use module::generate_mod_file;