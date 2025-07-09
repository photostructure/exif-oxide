//! JSON schema validation for configuration files
//!
//! This module provides validation for the various configuration file types
//! used in the new modular codegen system.

use anyhow::{Context, Result};
use jsonschema::{Draft, JSONSchema};
use serde_json::Value;
use std::fs;
use std::path::Path;

/// Validate a configuration file against its schema
pub fn validate_config(config_path: &Path, schema_path: &Path) -> Result<()> {
    // Read the schema
    let schema_content = fs::read_to_string(schema_path)
        .with_context(|| format!("Failed to read schema: {}", schema_path.display()))?;
    let schema: Value = serde_json::from_str(&schema_content)
        .with_context(|| format!("Failed to parse schema: {}", schema_path.display()))?;
    
    // Compile the schema
    let compiled = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema)
        .map_err(|e| anyhow::anyhow!("Failed to compile schema: {}", e))?;
    
    // Read the instance
    let instance_content = fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read config: {}", config_path.display()))?;
    let instance: Value = serde_json::from_str(&instance_content)
        .with_context(|| format!("Failed to parse config: {}", config_path.display()))?;
    
    // Validate
    let result = compiled.validate(&instance);
    
    if let Err(errors) = result {
        let error_messages: Vec<String> = errors
            .map(|error| format!("  - {}: {}", error.instance_path, error))
            .collect();
        
        return Err(anyhow::anyhow!(
            "Validation failed for {}:\n{}",
            config_path.display(),
            error_messages.join("\n")
        ));
    }
    
    Ok(())
}

/// Validate all configuration files in a directory
pub fn validate_config_directory(config_dir: &Path, schemas_dir: &Path) -> Result<()> {
    if !config_dir.exists() {
        return Ok(()); // No config directory is valid
    }
    
    let config_files = [
        ("simple_table.json", "simple_table.json"),
        ("print_conv.json", "print_conv.json"),
        ("boolean_set.json", "boolean_set.json"),
        ("regex_patterns.json", "regex_strings.json"), // Note: schema name differs
        ("file_type_lookup.json", "file_type_lookup.json"),
    ];
    
    for (config_file, schema_file) in &config_files {
        let config_path = config_dir.join(config_file);
        if config_path.exists() {
            let schema_path = schemas_dir.join(schema_file);
            validate_config(&config_path, &schema_path)?;
        }
    }
    
    Ok(())
}

/// Validate all module configurations
pub fn validate_all_configs(config_root: &Path, schemas_dir: &Path) -> Result<()> {
    println!("🔍 Validating configuration files...");
    
    let modules = ["Canon_pm", "Nikon_pm", "ExifTool_pm", "Exif_pm", "XMP_pm"];
    
    for module in &modules {
        let module_config_dir = config_root.join(module);
        if module_config_dir.exists() {
            println!("  Validating {}/", module);
            validate_config_directory(&module_config_dir, schemas_dir)
                .with_context(|| format!("Validation failed for module {}", module))?;
        }
    }
    
    println!("✅ All configurations valid!");
    Ok(())
}