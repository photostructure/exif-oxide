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
}