//! Pure key-value lookup table generation
//! 
//! This module handles generation of simple HashMap-based lookup tables
//! like Canon white balance values, Nikon lens IDs, etc.
//! 
//! These are straightforward mappings from numeric or string keys to descriptive values.

pub mod standard;

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use crate::schemas::{ExtractedTable, input::TableConfig};
use crate::generators::data_sets;
use tracing::{debug, info};

/// Process configuration files from a directory and generate modular structure
pub fn process_config_directory(
    config_dir: &Path,
    module_name: &str,
    extracted_tables: &HashMap<String, ExtractedTable>,
    output_dir: &str,
) -> Result<()> {
    println!("    Processing config directory for module: {}", module_name);
    
    let mut generated_files = Vec::new();
    let mut has_content = false;
    
    // Create module directory
    let module_output_dir = Path::new(output_dir).join(module_name);
    fs::create_dir_all(&module_output_dir)?;
    
    // Check for simple_table.json configuration - create individual files
    let simple_table_config = config_dir.join("simple_table.json");
    if simple_table_config.exists() {
        let config_content = fs::read_to_string(&simple_table_config)?;
        let config_json: serde_json::Value = serde_json::from_str(&config_content)?;
        
        if let Some(tables) = config_json["tables"].as_array() {
            for table_config in tables {
                let hash_name = table_config["hash_name"].as_str().unwrap_or("");
                if let Some(extracted_table) = extracted_tables.get(hash_name) {
                    // Update the extracted table's metadata with config values
                    let mut updated_table = extracted_table.clone();
                    if let Some(constant_name) = table_config["constant_name"].as_str() {
                        updated_table.metadata.constant_name = constant_name.to_string();
                    }
                    if let Some(key_type) = table_config["key_type"].as_str() {
                        updated_table.metadata.key_type = key_type.to_string();
                    }
                    if let Some(description) = table_config["description"].as_str() {
                        updated_table.metadata.description = description.to_string();
                    }
                    
                    // Generate individual file for this table
                    let file_name = generate_table_file(hash_name, &updated_table, &module_output_dir)?;
                    generated_files.push(file_name);
                    has_content = true;
                }
            }
        }
    }
    
    // Check for boolean_set.json configuration - create individual files
    let boolean_set_config = config_dir.join("boolean_set.json");
    if boolean_set_config.exists() {
        let config_content = fs::read_to_string(&boolean_set_config)?;
        let config_json: serde_json::Value = serde_json::from_str(&config_content)?;
        
        if let Some(tables) = config_json["tables"].as_array() {
            for table_config in tables {
                let hash_name = table_config["hash_name"].as_str().unwrap_or("");
                if let Some(extracted_table) = extracted_tables.get(hash_name) {
                    // Update the extracted table's metadata with config values
                    let mut updated_table = extracted_table.clone();
                    if let Some(constant_name) = table_config["constant_name"].as_str() {
                        updated_table.metadata.constant_name = constant_name.to_string();
                    }
                    if let Some(description) = table_config["description"].as_str() {
                        updated_table.metadata.description = description.to_string();
                    }
                    
                    // Generate individual file for this boolean set
                    let file_name = generate_boolean_set_file(hash_name, &updated_table, &module_output_dir)?;
                    generated_files.push(file_name);
                    has_content = true;
                }
            }
        }
    }
    
    // Only generate mod.rs if we have content
    if has_content {
        generate_module_mod_file(&generated_files, module_name, &module_output_dir)?;
    }
    
    Ok(())
}

/// Generate individual file for a lookup table
fn generate_table_file(
    hash_name: &str,
    extracted_table: &ExtractedTable,
    output_dir: &Path,
) -> Result<String> {
    let table_code = standard::generate_lookup_table(hash_name, extracted_table)?;
    
    // Create descriptive filename from hash name
    let file_name = hash_name_to_filename(hash_name);
    let file_path = output_dir.join(format!("{}.rs", file_name));
    
    let mut content = String::new();
    content.push_str(&format!("//! {} lookup table\n", extracted_table.metadata.description));
    content.push_str("//!\n");
    content.push_str("//! This file is automatically generated by codegen.\n");
    content.push_str("//! DO NOT EDIT MANUALLY - changes will be overwritten.\n\n");
    content.push_str("use std::collections::HashMap;\n");
    content.push_str("use std::sync::LazyLock;\n\n");
    content.push_str(&table_code);
    
    fs::write(&file_path, content)?;
    println!("  ✓ Generated {}", file_path.display());
    
    Ok(file_name)
}

/// Generate individual file for a boolean set
fn generate_boolean_set_file(
    hash_name: &str,
    extracted_table: &ExtractedTable,
    output_dir: &Path,
) -> Result<String> {
    let table_code = data_sets::boolean::generate_boolean_set(hash_name, extracted_table)?;
    
    // Create descriptive filename from hash name
    let file_name = hash_name_to_filename(hash_name);
    let file_path = output_dir.join(format!("{}.rs", file_name));
    
    let mut content = String::new();
    content.push_str(&format!("//! {} boolean set\n", extracted_table.metadata.description));
    content.push_str("//!\n");
    content.push_str("//! This file is automatically generated by codegen.\n");
    content.push_str("//! DO NOT EDIT MANUALLY - changes will be overwritten.\n\n");
    content.push_str("use std::collections::HashSet;\n");
    content.push_str("use std::sync::LazyLock;\n\n");
    content.push_str(&table_code);
    
    fs::write(&file_path, content)?;
    println!("  ✓ Generated {}", file_path.display());
    
    Ok(file_name)
}

/// Generate mod.rs file that re-exports all generated files
fn generate_module_mod_file(
    generated_files: &[String],
    module_name: &str,
    output_dir: &Path,
) -> Result<()> {
    let mut mod_content = String::new();
    
    // File header
    mod_content.push_str(&format!("//! Generated lookup tables from ExifTool {}\n", module_name.replace("_pm", ".pm")));
    mod_content.push_str("//!\n");
    mod_content.push_str("//! This file is automatically generated by codegen.\n");
    mod_content.push_str("//! DO NOT EDIT MANUALLY - changes will be overwritten.\n\n");
    
    // Module declarations
    for file_name in generated_files {
        mod_content.push_str(&format!("pub mod {};\n", file_name));
    }
    mod_content.push_str("\n");
    
    // Re-export all lookup functions and constants
    mod_content.push_str("// Re-export all lookup functions and constants\n");
    for file_name in generated_files {
        mod_content.push_str(&format!("pub use {}::*;\n", file_name));
    }
    
    let mod_file_path = output_dir.join("mod.rs");
    fs::write(&mod_file_path, mod_content)?;
    
    println!("  ✓ Generated {}", mod_file_path.display());
    Ok(())
}

/// Convert hash name to a valid Rust filename
fn hash_name_to_filename(hash_name: &str) -> String {
    hash_name
        .trim_start_matches('%')
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

