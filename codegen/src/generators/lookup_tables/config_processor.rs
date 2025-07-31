//! Configuration processing utilities for lookup table generation

use anyhow::{Result, Context};
use std::fs;
use std::path::Path;
use serde_json::Value;
use tracing::warn;

/// Read and parse a JSON configuration file
pub fn read_config_json(config_path: &Path) -> Result<Value> {
    let config_content = fs::read_to_string(config_path)?;
    let config_json: Value = serde_json::from_str(&config_content)
        .with_context(|| format!("Failed to parse JSON file: {}", config_path.display()))?;
    Ok(config_json)
}

/// Check if a configuration file exists and process it
pub fn process_config_if_exists<F>(
    config_dir: &Path,
    config_filename: &str,
    processor: F,
) -> Result<Vec<String>>
where
    F: FnOnce(Value) -> Result<Vec<String>>,
{
    let config_path = config_dir.join(config_filename);
    if config_path.exists() {
        let config = read_config_json(&config_path)?;
        processor(config)
    } else {
        Ok(Vec::new())
    }
}

/// Process a configuration that contains a "tables" array
/// TODO: Rename "tables" field to "targets" and merge with process_array_config for DRY
pub fn process_tables_config<F>(
    config: Value,
    table_processor: F,
) -> Result<Vec<String>>
where
    F: Fn(&Value) -> Result<Option<String>>,
{
    let mut generated_files = Vec::new();
    
    if let Some(tables) = config["tables"].as_array() {
        for table_config in tables {
            match table_processor(table_config) {
                Ok(Some(filename)) => generated_files.push(filename),
                Ok(None) => {}, // Skip silently
                Err(e) => warn!("    ⚠ Error processing table config: {}", e),
            }
        }
    }
    
    Ok(generated_files)
}

/// Process tag table structure configurations (handles multiple files)
#[allow(dead_code)]
pub fn find_tag_structure_configs(config_dir: &Path) -> Result<Vec<String>> {
    let mut config_files = Vec::new();
    
    if let Ok(entries) = fs::read_dir(config_dir) {
        let mut files: Vec<_> = entries
            .filter_map(Result::ok)
            .filter(|entry| {
                let file_name = entry.file_name();
                let file_name_str = file_name.to_string_lossy();
                file_name_str.ends_with("_tag_table_structure.json") || 
                file_name_str == "tag_table_structure.json"
            })
            .collect();
        
        // Sort to prioritize Main table first
        files.sort_by(|a, b| {
            let a_name = a.file_name();
            let b_name = b.file_name(); 
            let a_str = a_name.to_string_lossy();
            let b_str = b_name.to_string_lossy();
            
            // Main table (tag_table_structure.json) comes first
            if a_str == "tag_table_structure.json" {
                std::cmp::Ordering::Less
            } else if b_str == "tag_table_structure.json" {
                std::cmp::Ordering::Greater
            } else {
                a_str.cmp(&b_str)
            }
        });
        
        for entry in files {
            config_files.push(entry.path().to_string_lossy().to_string());
        }
    }
    
    Ok(config_files)
}

/// Load extracted data from JSON file
pub fn load_extracted_json<T>(path: &Path) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let content = fs::read_to_string(path)?;
    let data: T = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse extracted JSON file: {}", path.display()))?;
    Ok(data)
}