//! Simple table extraction orchestration
//!
//! Handles auto-discovery of configs and orchestration of Perl extraction scripts.

use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::path::Path;
use tracing::{debug, warn};

use crate::patching;
use crate::extractors::find_extractor;
use crate::config::load_extracted_tables_with_config;
use crate::generators::lookup_tables;

// Constants for path navigation

#[derive(Debug)]
pub struct ModuleConfig {
    pub source_path: String,
    pub hash_names: Vec<String>,
    pub module_name: String,
}

/// Extract all simple tables using Rust orchestration (replaces Makefile targets)
pub fn extract_all_simple_tables() -> Result<()> {
    debug!("📊 Extracting all tables and data...");
    
    let extract_base = Path::new("generated/extract");
    fs::create_dir_all(extract_base)?;
    
    let configs = discover_module_configs()?;
    
    for config in configs {
        process_module_config(&config, extract_base)?;
    }
    
    debug!("  ✓ Simple table extraction complete");
    Ok(())
}

/// Extract and generate code for a single config file for debugging purposes
pub fn extract_single_config(config_path: &str) -> Result<()> {
    debug!("📊 Extracting and generating single config: {}", config_path);
    
    let extract_base = Path::new("generated/extract");
    fs::create_dir_all(extract_base)?;
    
    let config_path = Path::new(config_path);
    if !config_path.exists() {
        return Err(anyhow::anyhow!("Config file not found: {}", config_path.display()));
    }
    
    if let Some(config) = try_parse_single_config(config_path)? {
        // Extract the data
        process_module_config(&config, extract_base)?;
        debug!("  ✓ Single config extraction complete");
        
        // Generate code for this specific config
        process_single_config_generation(&config, extract_base)?;
        debug!("  ✓ Single config generation complete");
    } else {
        return Err(anyhow::anyhow!("Failed to parse config file: {}", config_path.display()));
    }
    
    Ok(())
}

/// Process code generation for a single config file
fn process_single_config_generation(config: &ModuleConfig, extract_base: &Path) -> Result<()> {
    debug!("🔧 Generating code for single config...");
    
    let output_dir = "../src/generated";
    let config_dir = Path::new("config");
    
    // Load extracted tables for this specific config
    let all_extracted_tables = load_extracted_tables_with_config(extract_base, config_dir)?;
    
    // Determine the module directory from the source path
    // e.g., "third-party/exiftool/lib/Image/ExifTool/Canon.pm" -> "Canon_pm"
    let module_name = config.source_path
        .split('/')
        .next_back()
        .unwrap_or("unknown")
        .replace(".pm", "_pm");
    
    debug!("  Looking for module directory: {}", module_name);
    
    // Find the module config directory
    let module_config_dir = config_dir.join(&module_name);
    if module_config_dir.exists() {
        debug!("  Processing module config directory: {}", module_config_dir.display());
        
        // Use the lookup_tables processor to generate code for this module
        lookup_tables::process_config_directory(
            &module_config_dir,
            &module_name,
            &all_extracted_tables,
            output_dir,
        )?;
        
        debug!("  ✓ Generated code for module: {}", module_name);
    } else {
        debug!("  ⚠️  Module config directory not found: {}", module_config_dir.display());
        debug!("      Expected: {}", module_config_dir.display());
    }
    
    Ok(())
}

fn discover_module_configs() -> Result<Vec<ModuleConfig>> {
    let config_dir = Path::new("config");
    let mut configs = Vec::new();
    
    for entry in fs::read_dir(config_dir)? {
        let entry = entry?;
        let module_config_dir = entry.path();
        
        if should_skip_directory(&module_config_dir) {
            continue;
        }
        
        // Process all configs in this module directory
        let module_configs = parse_all_module_configs(&module_config_dir)?;
        configs.extend(module_configs);
    }
    
    Ok(configs)
}

fn should_skip_directory(path: &Path) -> bool {
    !path.is_dir() || 
    path.file_name()
        .and_then(|name| name.to_str())
        .is_none_or(|name| name.starts_with('.'))
}

/// Parse all config files in a module directory.
/// 
/// A module can have multiple config types (simple_table.json, file_type_lookup.json, etc.)
/// and this function collects all of them, allowing for multi-config modules like:
/// 
/// ```
/// ExifTool_pm/
/// ├── simple_table.json      # Basic lookup tables
/// ├── file_type_lookup.json  # File type detection
/// └── boolean_set.json       # Boolean set membership
/// ```
fn parse_all_module_configs(module_config_dir: &Path) -> Result<Vec<ModuleConfig>> {
    let mut configs = Vec::new();
    
    // Look for all supported config files using patterns
    let config_patterns = [
        "simple_table.json",
        "file_type_lookup.json", 
        "regex_patterns.json",
        "boolean_set.json",
        "tag_definitions.json",
        "composite_tags.json",
        "tag_table_structure.json",    // Exact match for Main table configs
        "*_tag_table_structure.json",  // Matches equipment_tag_table_structure.json and other variants
        "process_binary_data.json",
        "model_detection.json",
        "conditional_tags.json",
        "runtime_table.json",
        "tag_kit.json"
    ];
    
    // Process each pattern
    for pattern in &config_patterns {
        // For patterns with wildcards, use glob matching
        if pattern.contains('*') {
            let glob_pattern = module_config_dir.join(pattern).to_string_lossy().to_string();
            if let Ok(paths) = glob::glob(&glob_pattern) {
                for entry in paths.flatten() {
                    if let Some(config) = try_parse_single_config(&entry)? {
                        configs.push(config);
                    }
                }
            }
        } else {
            // For exact filenames, use direct path
            let config_path = module_config_dir.join(pattern);
            if config_path.exists() {
                if let Some(config) = try_parse_single_config(&config_path)? {
                    configs.push(config);
                }
            }
        }
    }
    
    Ok(configs)
}

fn try_parse_single_config(config_path: &Path) -> Result<Option<ModuleConfig>> {
    debug!("Reading config file: {}", config_path.display());
    let config_content = match fs::read_to_string(config_path) {
        Ok(data) => data,
        Err(err) => {
            warn!("UTF-8 error reading {}: {}", config_path.display(), err);
            let bytes = fs::read(config_path)
                .with_context(|| format!("Failed to read bytes from {}", config_path.display()))?;
            String::from_utf8_lossy(&bytes).into_owned()
        }
    };
    
    let config: Value = serde_json::from_str(&config_content)
        .with_context(|| format!("Failed to parse {}", config_path.display()))?;
    
    let source_path = config["source"].as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing 'source' field in {}", config_path.display()))?;
    
    let config_type = config_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    
    let hash_names: Vec<String> = match config_type {
        "tag_definitions.json" | "composite_tags.json" | "tag_table_structure.json" |
        "process_binary_data.json" | "model_detection.json" | "conditional_tags.json" => {
            // For tag definitions, composite tags, and tag table structure, we use the table name from config root
            let table = config["table"].as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing 'table' field in {}", config_path.display()))?;
            vec![table.to_string()]
        },
        config_name if config_name.ends_with("_tag_table_structure.json") => {
            // For tag table structure configs (including equipment_tag_table_structure.json)
            let table = config["table"].as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing 'table' field in {}", config_path.display()))?;
            vec![table.to_string()]
        },
        "tag_kit.json" => {
            // For tag kits, we extract table names from the tables array
            let tables = config["tables"].as_array()
                .ok_or_else(|| anyhow::anyhow!("Missing 'tables' field in {}", config_path.display()))?;
            
            tables.iter()
                .filter_map(|table| table["table_name"].as_str())
                .map(|name| name.to_string())
                .collect()
        },
        "runtime_table.json" => {
            // For runtime tables, we extract table names from the tables array
            let tables = config["tables"].as_array()
                .ok_or_else(|| anyhow::anyhow!("Missing 'tables' field in {}", config_path.display()))?;
            
            tables.iter()
                .filter_map(|table| table["table_name"].as_str())
                .map(|name| name.trim_start_matches('%').to_string())
                .collect()
        },
        _ => {
            // For other configs, we need the tables array
            let tables = config["tables"].as_array()
                .ok_or_else(|| anyhow::anyhow!("Missing 'tables' field in {}", config_path.display()))?;
            
            if tables.is_empty() {
                return Ok(None);
            }
            
            // Extract hash names from tables
            tables.iter()
                .filter_map(|table| table["hash_name"].as_str())
                .map(|name| name.trim_start_matches('%').to_string())
                .collect()
        }
    };
    
    if hash_names.is_empty() {
        return Ok(None);
    }
    
    let module_name = config_path.file_stem()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid config filename: {}", config_path.display()))?
        .to_string();
    
    Ok(Some(ModuleConfig {
        source_path: source_path.to_string(),
        hash_names,
        module_name,
    }))
}

fn process_module_config(config: &ModuleConfig, extract_base: &Path) -> Result<()> {
    debug!("  📷 Processing {} tables...", config.module_name);
    
    // Find the appropriate extractor
    let extractor = find_extractor(&config.module_name)
        .ok_or_else(|| anyhow::anyhow!("No extractor found for config type: {}", config.module_name))?;
    
    // Get absolute paths
    let current_dir = std::env::current_dir()?;
    let repo_root = current_dir.parent()
        .ok_or_else(|| anyhow::anyhow!("Could not find repo root"))?;
    let module_path = repo_root.join(&config.source_path).canonicalize()
        .with_context(|| format!("Failed to canonicalize module path: {}", config.source_path))?;
    
    // Only patch if the extractor requires it
    if extractor.requires_patching() {
        patching::patch_module(&module_path, &config.hash_names)?;
    }
    
    // Execute the extraction
    extractor.extract(config, extract_base, &module_path)?;
    
    Ok(())
}