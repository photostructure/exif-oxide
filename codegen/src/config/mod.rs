//! Configuration management for code generation
//!
//! This module handles configuration discovery, parsing, and validation
//! for the modular code generation system.

use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, warn};

use crate::file_operations::{file_exists, read_utf8_with_fallback};
use crate::schemas::{input::TableMetadata, ExtractedTable, TableEntry, TableSource};

/// Simplified metadata structure for extracted JSON files
///
/// IMPORTANT: DO NOT ADD `entry_count` field here!
/// The Rust code calculates entry count from entries.len() to avoid inconsistencies.
/// Adding entry_count back will cause parsing failures and duplicate logic.
#[derive(Debug, serde::Deserialize)]
pub struct SimpleMetadata {
    // entry_count field deliberately omitted - calculate from entries.len() instead
}

/// Simplified structure for extracted JSON files
#[derive(Debug, serde::Deserialize)]
pub struct SimpleExtractedTable {
    pub source: TableSource,
    #[allow(dead_code)]
    pub metadata: SimpleMetadata,
    pub entries: Vec<TableEntry>,
}

/// Configuration metadata for a table
#[derive(Debug, Clone)]
pub struct TableConfig {
    pub description: String,
    pub constant_name: String,
    pub key_type: String,
}

/// Load configuration metadata for a specific table hash
///
/// This function searches for table configuration in both simple_table.json
/// and boolean_set.json files within the module's config directory.
pub fn load_table_config(config_dir: &Path, hash_name: &str) -> Result<Option<TableConfig>> {
    // Try simple_table.json first
    let simple_config_file = config_dir.join("simple_table.json");
    if let Some(config) = load_config_from_file(&simple_config_file, hash_name)? {
        return Ok(Some(config));
    }

    // Try boolean_set.json if not found in simple_table.json
    let boolean_config_file = config_dir.join("boolean_set.json");
    if let Some(config) = load_config_from_file(&boolean_config_file, hash_name)? {
        return Ok(Some(config));
    }

    Ok(None)
}

/// Load configuration from a specific JSON file
fn load_config_from_file(config_file: &Path, hash_name: &str) -> Result<Option<TableConfig>> {
    if !file_exists(config_file) {
        return Ok(None);
    }

    let config_content = read_utf8_with_fallback(config_file)?;
    let config_json: Value = serde_json::from_str(&config_content)
        .with_context(|| format!("Failed to parse config file: {}", config_file.display()))?;

    if let Some(tables) = config_json["tables"].as_array() {
        if let Some(table_config) = tables.iter().find(|t| t["hash_name"] == *hash_name) {
            return Ok(Some(TableConfig {
                description: table_config["description"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
                constant_name: table_config["constant_name"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
                key_type: table_config["key_type"]
                    .as_str()
                    .unwrap_or("String")
                    .to_string(),
            }));
        }
    }

    Ok(None)
}

/// Create an ExtractedTable from simple table data and configuration
///
/// This function combines the extracted table data with configuration metadata
/// to create a complete ExtractedTable object.
pub fn create_extracted_table(
    source: TableSource,
    entries: Vec<TableEntry>,
    config: &TableConfig,
    entry_count: usize,
) -> ExtractedTable {
    ExtractedTable {
        source,
        metadata: TableMetadata {
            description: config.description.clone(),
            constant_name: config.constant_name.clone(),
            key_type: config.key_type.clone(),
            entry_count,
        },
        entries,
    }
}

/// Load all extracted tables with their configurations
///
/// This function processes all extracted tables and matches them with their
/// configuration metadata from the module config directories.
pub fn load_extracted_tables_with_config(
    extract_dir: &Path,
    config_dir: &Path,
) -> Result<HashMap<String, ExtractedTable>> {
    use crate::file_operations::read_directory;

    let mut all_extracted_tables = HashMap::new();

    // Only load from the simple_tables subdirectory
    let simple_tables_dir = extract_dir.join("simple_tables");
    if !simple_tables_dir.exists() {
        return Ok(all_extracted_tables);
    }

    for entry in read_directory(&simple_tables_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let json_data = read_utf8_with_fallback(&path)?;

            // Skip empty files
            if json_data.trim().is_empty() {
                debug!("  ⚠️  Skipping empty file: {}", path.display());
                continue;
            }

            // Try to parse as SimpleExtractedTable first
            match serde_json::from_str::<SimpleExtractedTable>(&json_data) {
                Ok(simple_table) => {
                    // Get module name from the table source
                    let module_name = normalize_module_name(&simple_table.source.module);
                    let hash_name = &simple_table.source.hash_name;

                    // Load configuration for this table
                    let module_config_dir = config_dir.join(&module_name);
                    if let Some(config) = load_table_config(&module_config_dir, hash_name)? {
                        // Create full ExtractedTable with metadata from config
                        // NOTE: entry_count is calculated from entries.len() - DO NOT add to JSON!
                        let table = create_extracted_table(
                            simple_table.source.clone(),
                            simple_table.entries.clone(),
                            &config,
                            simple_table.entries.len(), // Calculate from actual entries
                        );
                        all_extracted_tables.insert(table.source.hash_name.clone(), table);
                    } else {
                        warn!(
                            "Could not find config for {}: {}",
                            path.display(),
                            hash_name
                        );
                    }
                }
                Err(e) => {
                    warn!("Failed to parse {}: {}", path.display(), e);
                    debug!(
                        "  First 200 chars of content: {}",
                        &json_data.chars().take(200).collect::<String>()
                    );
                }
            }
        }
    }

    Ok(all_extracted_tables)
}

/// Normalize module name from ExifTool format to config directory format
///
/// Converts formats like:
/// - "Image::ExifTool::Canon" -> "Canon_pm"
/// - "Image::ExifTool" -> "ExifTool_pm"
/// - "Canon.pm" -> "Canon_pm"
fn normalize_module_name(module: &str) -> String {
    if module.starts_with("Image::ExifTool::") {
        // Old Perl module format: Image::ExifTool::Canon -> Canon_pm
        module
            .strip_prefix("Image::ExifTool::")
            .unwrap()
            .to_string()
            + "_pm"
    } else if module == "Image::ExifTool" {
        // Old main module format: Image::ExifTool -> ExifTool_pm
        "ExifTool_pm".to_string()
    } else if module.contains("/") {
        // New full path format: third-party/exiftool/lib/Image/ExifTool/Canon.pm -> Canon_pm
        if let Some(filename) = module.split('/').next_back() {
            filename.replace(".pm", "_pm")
        } else {
            module.replace(".pm", "_pm")
        }
    } else {
        // Old filename format: Canon.pm -> Canon_pm
        module.replace(".pm", "_pm")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_module_name() {
        assert_eq!(normalize_module_name("Image::ExifTool::Canon"), "Canon_pm");
        assert_eq!(normalize_module_name("Image::ExifTool"), "ExifTool_pm");
        assert_eq!(normalize_module_name("Canon.pm"), "Canon_pm");
        assert_eq!(normalize_module_name("Nikon.pm"), "Nikon_pm");
    }

    #[test]
    fn test_load_table_config_nonexistent() {
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let result = load_table_config(temp_dir.path(), "nonexistent_hash").unwrap();
        assert!(result.is_none());
    }
}
