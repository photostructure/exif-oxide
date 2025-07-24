//! Schema definitions for tag kit extraction data

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tag kit extraction data
#[derive(Debug, Deserialize, Serialize)]
pub struct TagKitExtraction {
    pub source: SourceInfo,
    pub metadata: MetadataInfo,
    pub tag_kits: Vec<TagKit>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SourceInfo {
    pub module: String,
    pub table: String,
    pub extracted_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MetadataInfo {
    pub total_tags_scanned: usize,
    pub tag_kits_extracted: usize,
    pub skipped_complex: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TagKit {
    pub tag_id: String,
    pub name: String,
    pub format: String,
    pub groups: HashMap<String, String>,
    #[serde(default)]
    pub writable: Option<serde_json::Value>,
    #[serde(default)]
    pub notes: Option<String>,
    pub print_conv_type: String,
    #[serde(default)]
    pub print_conv_data: Option<serde_json::Value>,
    #[serde(default)]
    pub value_conv: Option<String>,
    #[serde(default)]
    pub variant_id: Option<String>,
    #[serde(default)]
    pub condition: Option<String>,
    #[serde(default)]
    pub subdirectory: Option<SubDirectoryInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SubDirectoryInfo {
    pub tag_table: String,
    #[serde(default)]
    pub validate: Option<String>,
    #[serde(default)]
    pub process_proc: Option<String>,
    #[serde(default)]
    pub base: Option<serde_json::Value>,
    #[serde(default)]
    pub byte_order: Option<String>,
    #[serde(default)]
    pub has_validate_code: Option<bool>,
    #[serde(default)]
    pub has_process_proc_code: Option<bool>,
    #[serde(default)]
    pub is_binary_data: Option<bool>,
    #[serde(default)]
    pub extracted_table: Option<ExtractedTable>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ExtractedTable {
    pub table_name: String,
    pub is_binary_data: bool,
    #[serde(default)]
    pub has_process_proc: Option<bool>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub first_entry: Option<i32>,
    #[serde(default)]
    pub groups: Option<HashMap<String, String>>,
    pub tags: Vec<ExtractedTag>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ExtractedTag {
    pub tag_id: String,
    pub name: String,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub count: Option<String>,
    #[serde(default)]
    pub has_subdirectory: Option<bool>,
}