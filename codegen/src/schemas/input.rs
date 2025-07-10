//! Input schemas for deserializing JSON from Perl extractors

use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

/// JSON structure from extract_tables.pl
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ExtractedData {
    pub extracted_at: String,
    pub exiftool_version: String,
    pub filter_criteria: String,
    pub stats: TagStats,
    pub tags: TagGroups,
    pub conversion_refs: ConversionRefs,
}

/// Tag statistics
#[derive(Debug, Deserialize)]
pub struct TagStats {
    pub exif_count: usize,
    pub gps_count: usize,
    pub total_tags: usize,
}

/// Tag groups (EXIF, GPS, etc)
#[derive(Debug, Deserialize)]
pub struct TagGroups {
    #[serde(default)]
    pub exif: Vec<ExtractedTag>,
    #[serde(default)]
    pub gps: Vec<ExtractedTag>,
}

/// Conversion references extracted from tag definitions
#[derive(Debug, Deserialize)]
pub struct ConversionRefs {
    pub print_conv: Vec<String>,
    pub value_conv: Vec<String>,
}

/// Individual extracted tag from ExifTool
#[derive(Debug, Deserialize)]
pub struct ExtractedTag {
    pub id: String,
    pub name: String,
    pub format: String,
    #[serde(default)]
    pub groups: Vec<String>,
    pub writable: u8,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub print_conv_ref: Option<String>,
    #[serde(default)]
    pub value_conv_ref: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub frequency: Option<f64>,
    #[serde(default)]
    pub mainstream: Option<u8>,
}

/// Composite tag definition
#[derive(Debug, Deserialize)]
pub struct ExtractedCompositeTag {
    pub name: String,
    pub table: String,
    pub full_name: String,
    #[serde(default)]
    pub require: Option<Vec<String>>,
    #[serde(default)]
    pub desire: Option<Vec<String>>,
    #[serde(default)]
    pub print_conv_ref: Option<String>,
    #[serde(default)]
    pub value_conv_ref: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub writable: u8,
    #[serde(default)]
    pub frequency: Option<f64>,
    #[serde(default)]
    pub mainstream: Option<u8>,
}

/// JSON structure from simple_table.pl
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct SimpleTablesData {
    pub extracted_at: String,
    pub extraction_config: String,
    pub total_tables: usize,
    pub tables: HashMap<String, ExtractedTable>,
}

/// Extracted table data from simple_table.pl
#[derive(Debug, Clone, Deserialize)]
pub struct ExtractedTable {
    pub source: TableSource,
    pub metadata: TableMetadata,
    pub entries: Vec<TableEntry>,
}

/// Table source information
#[derive(Debug, Clone, Deserialize)]
pub struct TableSource {
    pub module: String,
    pub hash_name: String,
    pub extracted_at: String,
}

/// Table metadata from extraction
#[derive(Debug, Clone, Deserialize)]
pub struct TableMetadata {
    pub description: String,
    pub constant_name: String,
    pub key_type: String,
    pub entry_count: usize,
}

/// Table configuration from module config files
#[derive(Debug, Deserialize)]
pub struct TableConfig {
    pub module: String,
    pub output_file: String,
    #[serde(default)]
    pub constant_name: Option<String>,
    #[serde(default)]
    pub key_type: Option<String>,
    #[serde(default)]
    pub extraction_type: Option<String>,
    #[serde(default)]
    pub value_type: Option<String>,
    pub description: String,
}

/// Composite tags JSON structure
#[derive(Debug, Deserialize)]
pub struct CompositeData {
    pub extracted_at: String,
    pub exiftool_version: String,
    pub filter_criteria: String,
    pub stats: CompositeStats,
    pub composite_tags: Vec<ExtractedCompositeTag>,
    pub conversion_refs: ConversionRefs,
}

/// Composite tag statistics
#[derive(Debug, Deserialize)]
pub struct CompositeStats {
    pub total_composite_tags: usize,
    pub exif_table: usize,
    pub gps_table: usize,
    pub main_table: usize,
}

/// Individual table entry (polymorphic for different extraction types)
#[derive(Debug, Deserialize, Clone)]
pub struct TableEntry {
    // Standard simple table fields
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub value: Option<Value>,
    
    // Regex pattern fields
    #[serde(default)]
    pub rust_compatible: Option<bool>,
    #[serde(default)]
    pub compatibility_notes: Option<String>,
    
    // File type lookup specific fields
    #[serde(default)]
    pub extension: Option<String>,
    #[serde(default)]
    pub entry_type: Option<String>,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub formats: Option<Vec<String>>,
    #[serde(default)]
    pub description: Option<String>,
}