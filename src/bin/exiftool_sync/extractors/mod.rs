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
mod magic_numbers;
mod maker_detection;

pub use binary_formats::BinaryFormatsExtractor;
pub use binary_tags::BinaryTagsExtractor;
pub use datetime_patterns::DateTimePatternsExtractor;
pub use magic_numbers::MagicNumbersExtractor;
pub use maker_detection::MakerDetectionExtractor;
