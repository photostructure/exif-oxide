//! Module discovery for code generation
//!
//! This module handles automatic discovery of module directories and
//! orchestrates the processing of each module's configuration.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use tracing::debug;

use crate::file_operations::read_directory;
use crate::generators::lookup_tables;
use crate::schemas::ExtractedTable;

/// Discover and process all module directories
///
/// This function auto-discovers module directories (ending with "_pm") and
/// processes each one using the lookup_tables system.
pub fn discover_and_process_modules(
    config_dir: &Path,
    all_extracted_tables: &HashMap<String, ExtractedTable>,
    output_dir: &str,
) -> Result<()> {
    let config_entries = read_directory(config_dir)
        .context("Failed to read config directory")?;
    
    for entry in config_entries {
        let entry = entry.context("Failed to read directory entry")?;
        let module_config_dir = entry.path();
        
        // Skip files, only process directories
        if !module_config_dir.is_dir() {
            continue;
        }
        
        // Skip hidden directories
        if let Some(dir_name) = module_config_dir.file_name() {
            if dir_name.to_string_lossy().starts_with('.') {
                continue;
            }
            
            let module_name = dir_name.to_string_lossy();
            debug!("  Processing module: {}", module_name);
            
            lookup_tables::process_config_directory(
                &module_config_dir,
                &module_name,
                all_extracted_tables,
                output_dir,
            )?;
        }
    }
    
    Ok(())
}

/// Generate module file structure
///
/// This function updates the main generated mod.rs file to include
/// all discovered modules.
pub fn update_generated_mod_file(output_dir: &str) -> Result<()> {
    use crate::file_operations::{file_exists, write_file_atomic};
    use std::fs;
    
    let mod_path = format!("{output_dir}/mod.rs");
    let mut content = if file_exists(Path::new(&mod_path)) {
        fs::read_to_string(&mod_path)?
    } else {
        String::new()
    };

    // Auto-discover module directories (any directory ending in _pm)
    let entries = read_directory(Path::new(output_dir))?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                if dir_name.ends_with("_pm") && path.join("mod.rs").exists() {
                    let mod_declaration = format!("pub mod {dir_name};\n");
                    if !content.contains(&mod_declaration) {
                        content.push_str(&mod_declaration);
                    }
                }
            }
        }
    }

    write_file_atomic(Path::new(&mod_path), &content)?;
    Ok(())
}

/// Discover all module directories ending with "_pm"
///
/// Returns a sorted list of module directory names for consistent processing.
#[allow(dead_code)]
pub fn discover_module_directories(config_dir: &Path) -> Result<Vec<String>> {
    let mut modules = Vec::new();
    
    if !config_dir.exists() {
        return Ok(modules);
    }
    
    let entries = read_directory(config_dir)?;
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
    Ok(modules)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_discover_module_directories() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path();
        
        // Create some module directories
        fs::create_dir_all(config_dir.join("Canon_pm")).unwrap();
        fs::create_dir_all(config_dir.join("Nikon_pm")).unwrap();
        fs::create_dir_all(config_dir.join("not_a_module")).unwrap();
        fs::create_dir_all(config_dir.join("ExifTool_pm")).unwrap();
        
        let modules = discover_module_directories(config_dir).unwrap();
        
        // Should be sorted
        assert_eq!(modules, vec!["Canon_pm", "ExifTool_pm", "Nikon_pm"]);
    }

    #[test]
    fn test_discover_module_directories_empty() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().join("nonexistent");
        
        let modules = discover_module_directories(&config_dir).unwrap();
        assert!(modules.is_empty());
    }

    #[test]
    fn test_discover_module_directories_no_modules() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path();
        
        // Create some non-module directories
        fs::create_dir_all(config_dir.join("not_a_module")).unwrap();
        fs::create_dir_all(config_dir.join("another_dir")).unwrap();
        
        let modules = discover_module_directories(config_dir).unwrap();
        assert!(modules.is_empty());
    }
}