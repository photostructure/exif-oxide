//! Extractors for synchronizing ExifTool algorithms with exif-oxide

use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

/// Trait for extracting data from ExifTool source files
pub trait Extractor {
    /// Extract data from ExifTool source and generate Rust code
    fn extract(&self, exiftool_path: &Path) -> Result<(), String>;
}

/// Priority level for sync issues
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Medium,
    High,
}

/// Source location in Perl code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerlSource {
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines: Option<String>,
}

/// A sync issue that needs manual attention
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncIssue {
    pub priority: Priority,
    pub command: String,
    pub perl_source: PerlSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rust_target: Option<String>,
    pub description: String,
    pub suggested_implementation: String,
}

/// Emit a sync issue to the shared JSON Lines file
pub fn emit_sync_issue(issue: SyncIssue) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("sync-todos.jsonl")
        .map_err(|e| format!("Failed to open sync-todos.jsonl: {}", e))?;

    writeln!(
        file,
        "{}",
        serde_json::to_string(&issue).map_err(|e| format!("Failed to serialize issue: {}", e))?
    )
    .map_err(|e| format!("Failed to write issue: {}", e))?;

    Ok(())
}

mod app_segment_tables;
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
mod printconv_sync;
mod printconv_tables;
mod shared_tables_sync;
mod writable_tags;

// Export extractors
pub use app_segment_tables::AppSegmentTablesExtractor;
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
pub use printconv_sync::PrintConvSyncExtractor;
pub use printconv_tables::PrintConvTablesExtractor;
pub use shared_tables_sync::SharedTablesSyncExtractor;
pub use writable_tags::WritableTagsExtractor;
