//! Extractors for synchronizing ExifTool algorithms with exif-oxide

use std::path::Path;

/// Trait for extracting data from ExifTool source files
pub trait Extractor {
    /// Extract data from ExifTool source and generate Rust code
    fn extract(&self, exiftool_path: &Path) -> Result<(), String>;
}

mod binary_formats;
mod binary_tags;
mod datetime_patterns;
mod exif_tags;
mod gpmf_format;
mod gpmf_tags;
mod magic_numbers;
mod maker_detection;
mod printconv_analyzer;
mod printconv_generator;
mod printconv_tables;

pub use binary_formats::BinaryFormatsExtractor;
pub use binary_tags::BinaryTagsExtractor;
pub use datetime_patterns::DateTimePatternsExtractor;
pub use exif_tags::ExifTagsExtractor;
pub use gpmf_format::GpmfFormatExtractor;
pub use gpmf_tags::GpmfTagsExtractor;
pub use magic_numbers::MagicNumbersExtractor;
pub use maker_detection::MakerDetectionExtractor;
pub use printconv_analyzer::{PrintConvAnalyzer, PrintConvPattern, PrintConvType};
pub use printconv_generator::PrintConvGenerator;
pub use printconv_tables::PrintConvTablesExtractor;
