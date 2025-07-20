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

// Modular architecture
pub mod lookup_tables;
pub mod file_detection;
pub mod data_sets;


pub use tags::generate_tag_table;
pub use composite_tags::generate_composite_tag_table;
pub use conversion_refs::generate_conversion_refs;
pub use supported_tags::generate_supported_tags;
pub use module::generate_mod_file;
pub use tag_structure::generate_tag_structure;
pub use process_binary_data::generate_process_binary_data;
pub use model_detection::generate_model_detection;
pub use conditional_tags::generate_conditional_tags;
pub use offset_patterns::generate_offset_patterns;