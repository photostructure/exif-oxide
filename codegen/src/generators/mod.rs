//! Code generators for different types of ExifTool data

pub mod tags;
pub mod composite_tags;
pub mod conversion_refs;
pub mod supported_tags;
pub mod module;
pub mod tag_structure;
pub mod process_binary_data;
pub mod model_detection;
pub mod conditional_tags;
pub mod offset_patterns;
pub mod tag_kit;
pub mod tag_kit_split;
pub mod tag_kit_modular;

// Modular architecture
pub mod lookup_tables;
pub mod file_detection;
pub mod data_sets;


pub use tags::generate_tag_table;
pub use composite_tags::generate_composite_tag_table;
pub use supported_tags::generate_supported_tags;
pub use module::generate_mod_file;
