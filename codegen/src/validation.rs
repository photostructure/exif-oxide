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
    let schema_content = match fs::read_to_string(schema_path) {
        Ok(data) => data,
        Err(err) => {
            eprintln!(
                "Warning: UTF-8 error reading {}: {}",
                schema_path.display(),
                err
            );
            let bytes = fs::read(schema_path)
                .with_context(|| format!("Failed to read bytes from {}", schema_path.display()))?;
            String::from_utf8_lossy(&bytes).into_owned()
        }
    };
    let schema: Value = serde_json::from_str(&schema_content)
        .with_context(|| format!("Failed to parse schema: {}", schema_path.display()))?;

    // Compile the schema
    let compiled = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema)
        .map_err(|e| anyhow::anyhow!("Failed to compile schema: {}", e))?;

    // Read the instance
    let instance_content = match fs::read_to_string(config_path) {
        Ok(data) => data,
        Err(err) => {
            eprintln!(
                "Warning: UTF-8 error reading {}: {}",
                config_path.display(),
                err
            );
            let bytes = fs::read(config_path)
                .with_context(|| format!("Failed to read bytes from {}", config_path.display()))?;
            String::from_utf8_lossy(&bytes).into_owned()
        }
    };
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
        "boolean_set.json",
        "file_type_lookup.json",
        "print_conv.json",
        "regex_patterns.json",
        "simple_table.json",
    ];

    for config_file in &config_files {
        let config_path = config_dir.join(config_file);
        if config_path.exists() {
            let schema_path = schemas_dir.join(config_file);
            validate_config(&config_path, &schema_path)?;
        }
    }

    Ok(())
}

/// Validate all module configurations
pub fn validate_all_configs(config_root: &Path, schemas_dir: &Path) -> Result<()> {
    println!("🔍 Validating configuration files...");

    // Auto-discover all module directories ending in _pm
    let mut modules = Vec::new();
    let entries = fs::read_dir(config_root)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                if dir_name.ends_with("_pm") {
                    modules.push(dir_name.to_string());
                }
            }
        }
    }

    modules.sort(); // For consistent output

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
