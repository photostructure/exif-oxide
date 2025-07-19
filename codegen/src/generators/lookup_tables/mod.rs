//! Pure key-value lookup table generation
//! 
//! This module handles generation of simple HashMap-based lookup tables
//! like Canon white balance values, Nikon lens IDs, etc.
//! 
//! These are straightforward mappings from numeric or string keys to descriptive values.

pub mod standard;
pub mod inline_printconv;

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use crate::schemas::{ExtractedTable, input::TableConfig};
use crate::generators::data_sets;
use crate::common::utils::{module_to_source_path, module_dir_to_source_path};
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
    
    // Check for tag_table_structure.json configuration
    let tag_structure_config = config_dir.join("tag_table_structure.json");
    if tag_structure_config.exists() {
        let config_content = fs::read_to_string(&tag_structure_config)?;
        let config_json: serde_json::Value = serde_json::from_str(&config_content)?;
        
        // Extract table name from config
        if let Some(table_name) = config_json["table"].as_str() {
            // Look for the corresponding extracted tag structure JSON file
            let extract_dir = Path::new("generated/extract");
            let module_base = module_name.trim_end_matches("_pm");
            let structure_file = format!("{}_tag_structure.json", module_base.to_lowercase());
            let structure_path = extract_dir.join(&structure_file);
            
            if structure_path.exists() {
                let structure_content = fs::read_to_string(&structure_path)?;
                let structure_data: crate::generators::tag_structure::TagStructureData = 
                    serde_json::from_str(&structure_content)?;
                
                // Generate file for this tag structure
                let file_name = generate_tag_structure_file(
                    &structure_data,
                    &module_output_dir
                )?;
                generated_files.push(file_name);
                has_content = true;
            }
        }
    }
    
    // Check for process_binary_data.json configuration
    let process_binary_data_config = config_dir.join("process_binary_data.json");
    if process_binary_data_config.exists() {
        let config_content = fs::read_to_string(&process_binary_data_config)?;
        let config_json: serde_json::Value = serde_json::from_str(&config_content)?;
        
        // Extract table name from config
        if let Some(table_name) = config_json["table"].as_str() {
            // Look for the corresponding extracted binary data JSON file
            let extract_dir = Path::new("generated/extract");
            let module_base = module_name.trim_end_matches("_pm");
            let binary_data_file = format!("{}_binary_data.json", module_base.to_lowercase());
            let binary_data_path = extract_dir.join(&binary_data_file);
            
            if binary_data_path.exists() {
                let binary_data_content = fs::read_to_string(&binary_data_path)?;
                let binary_data_data: crate::generators::process_binary_data::ProcessBinaryDataExtraction = 
                    serde_json::from_str(&binary_data_content)?;
                
                // Generate file for this ProcessBinaryData table
                let file_name = generate_process_binary_data_file(
                    &binary_data_data,
                    &module_output_dir
                )?;
                generated_files.push(file_name);
                has_content = true;
            }
        }
    }
    
    // Check for model_detection.json configuration
    let model_detection_config = config_dir.join("model_detection.json");
    if model_detection_config.exists() {
        let config_content = fs::read_to_string(&model_detection_config)?;
        let config_json: serde_json::Value = serde_json::from_str(&config_content)?;
        
        // Extract table name from config
        if let Some(table_name) = config_json["table"].as_str() {
            // Look for the corresponding extracted model detection JSON file
            let extract_dir = Path::new("generated/extract");
            let module_base = module_name.trim_end_matches("_pm");
            let model_detection_file = format!("{}_model_detection.json", module_base.to_lowercase());
            let model_detection_path = extract_dir.join(&model_detection_file);
            
            if model_detection_path.exists() {
                let model_detection_content = fs::read_to_string(&model_detection_path)?;
                let model_detection_data: crate::generators::model_detection::ModelDetectionExtraction = 
                    serde_json::from_str(&model_detection_content)?;
                
                // Generate file for this ModelDetection table
                let file_name = generate_model_detection_file(
                    &model_detection_data,
                    &module_output_dir
                )?;
                generated_files.push(file_name);
                has_content = true;
            }
        }
    }
    
    // Check for conditional_tags.json configuration
    let conditional_tags_config = config_dir.join("conditional_tags.json");
    if conditional_tags_config.exists() {
        let config_content = fs::read_to_string(&conditional_tags_config)?;
        let config_json: serde_json::Value = serde_json::from_str(&config_content)?;
        
        // Extract table name from config
        if let Some(table_name) = config_json["table"].as_str() {
            // Look for the corresponding extracted conditional tags JSON file
            let extract_dir = Path::new("generated/extract");
            let module_base = module_name.trim_end_matches("_pm");
            let conditional_tags_file = format!("{}_conditional_tags.json", module_base.to_lowercase());
            let conditional_tags_path = extract_dir.join(&conditional_tags_file);
            
            if conditional_tags_path.exists() {
                let conditional_tags_content = fs::read_to_string(&conditional_tags_path)?;
                let conditional_tags_data: crate::generators::conditional_tags::ConditionalTagsExtraction = 
                    serde_json::from_str(&conditional_tags_content)?;
                
                // Generate file for this ConditionalTags table
                let file_name = generate_conditional_tags_file(
                    &conditional_tags_data,
                    &module_output_dir
                )?;
                generated_files.push(file_name);
                has_content = true;
            }
        }
    }
    
    // Check for inline_printconv.json configuration - create individual files
    let inline_printconv_config = config_dir.join("inline_printconv.json");
    if inline_printconv_config.exists() {
        let config_content = fs::read_to_string(&inline_printconv_config)?;
        let config_json: serde_json::Value = serde_json::from_str(&config_content)?;
        
        if let Some(tables) = config_json["tables"].as_array() {
            for table_config in tables {
                let table_name = table_config["table_name"].as_str().unwrap_or("");
                
                // Look for the corresponding extracted inline printconv JSON file
                let extract_dir = Path::new("generated/extract");
                let inline_file_name = format!("inline_printconv__{}.json", 
                    convert_table_name_to_snake_case(table_name)
                );
                let inline_file_path = extract_dir.join(&inline_file_name);
                
                if inline_file_path.exists() {
                    let inline_data_content = fs::read_to_string(&inline_file_path)?;
                    let inline_data: inline_printconv::InlinePrintConvData = 
                        serde_json::from_str(&inline_data_content)?;
                    
                    // Generate file for this table's inline PrintConv entries
                    let file_name = generate_inline_printconv_file(
                        &inline_data, 
                        table_name,
                        &module_output_dir
                    )?;
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
    content.push_str(&format!("//! {}\n", extracted_table.metadata.description));
    
    // Convert module filename to relative path for display
    let module_path = module_to_source_path(&extracted_table.source.module);
    
    content.push_str(&format!("//! \n//! Auto-generated from {}\n", module_path));
    content.push_str("//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen\n\n");
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
    content.push_str(&format!("//! {}\n", extracted_table.metadata.description));
    
    // Convert module filename to relative path for display
    let module_path = module_to_source_path(&extracted_table.source.module);
    
    content.push_str(&format!("//! \n//! Auto-generated from {}\n", module_path));
    content.push_str("//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen\n\n");
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
    
    // File header with source path
    let source_path = module_dir_to_source_path(module_name);
    mod_content.push_str(&format!("//! Generated lookup tables from {}\n", module_name.replace("_pm", ".pm")));
    mod_content.push_str("//!\n");
    mod_content.push_str(&format!("//! Auto-generated from {}\n", source_path));
    mod_content.push_str("//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen\n\n");
    
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

/// Generate file for inline PrintConv tables
fn generate_inline_printconv_file(
    inline_data: &inline_printconv::InlinePrintConvData,
    table_name: &str,
    output_dir: &Path,
) -> Result<String> {
    let table_code = inline_printconv::generate_inline_printconv_file(inline_data, table_name)?;
    
    // Create descriptive filename from table name
    let file_name = table_name
        .chars()
        .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
        .collect::<String>() + "_inline";
    
    let file_path = output_dir.join(format!("{}.rs", file_name));
    
    let mut content = String::new();
    content.push_str(&format!("//! Inline PrintConv tables for {} table\n", table_name));
    
    // Convert module filename to relative path for display
    let module_path = module_to_source_path(&inline_data.source.module);
    
    content.push_str(&format!("//! \n//! Auto-generated from {} (table: {})\n", module_path, table_name));
    content.push_str("//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen\n\n");
    content.push_str("use std::collections::HashMap;\n");
    content.push_str("use std::sync::LazyLock;\n\n");
    content.push_str(&table_code);
    
    fs::write(&file_path, content)?;
    println!("  ✓ Generated {}", file_path.display());
    
    Ok(file_name)
}

/// Generate individual file for a tag table structure
fn generate_tag_structure_file(
    structure_data: &crate::generators::tag_structure::TagStructureData,
    output_dir: &Path,
) -> Result<String> {
    let structure_code = crate::generators::tag_structure::generate_tag_structure(structure_data)?;
    
    // Create filename from manufacturer name
    let file_name = "tag_structure";
    let file_path = output_dir.join(format!("{}.rs", file_name));
    
    let mut content = String::new();
    content.push_str("//! Auto-generated from ExifTool source\n");
    content.push_str("//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen\n\n");
    content.push_str(&structure_code);
    
    fs::write(&file_path, content)?;
    println!("  ✓ Generated {}", file_path.display());
    
    Ok(file_name.to_string())
}

fn generate_process_binary_data_file(
    binary_data: &crate::generators::process_binary_data::ProcessBinaryDataExtraction,
    output_dir: &Path,
) -> Result<String> {
    let binary_code = crate::generators::process_binary_data::generate_process_binary_data(binary_data)?;
    
    // Create filename from table name
    let table_name = &binary_data.table_data.table_name;
    let file_name = format!("{}_binary_data", table_name.to_lowercase());
    let file_path = output_dir.join(format!("{}.rs", file_name));
    
    let mut content = String::new();
    content.push_str("//! Auto-generated from ExifTool source\n");
    content.push_str("//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen\n\n");
    content.push_str(&binary_code);
    
    fs::write(&file_path, content)?;
    println!("  ✓ Generated {}", file_path.display());
    
    Ok(file_name.to_string())
}

fn generate_model_detection_file(
    model_detection: &crate::generators::model_detection::ModelDetectionExtraction,
    output_dir: &Path,
) -> Result<String> {
    let model_code = crate::generators::model_detection::generate_model_detection(model_detection)?;
    
    // Create filename from table name
    let table_name = &model_detection.patterns_data.table_name;
    let file_name = format!("{}_model_detection", table_name.to_lowercase());
    let file_path = output_dir.join(format!("{}.rs", file_name));
    
    let mut content = String::new();
    content.push_str("//! Auto-generated from ExifTool source\n");
    content.push_str("//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen\n\n");
    content.push_str(&model_code);
    
    fs::write(&file_path, content)?;
    println!("  ✓ Generated {}", file_path.display());
    
    Ok(file_name.to_string())
}

fn generate_conditional_tags_file(
    conditional_tags: &crate::generators::conditional_tags::ConditionalTagsExtraction,
    output_dir: &Path,
) -> Result<String> {
    let conditional_code = crate::generators::conditional_tags::generate_conditional_tags(conditional_tags)?;
    
    // Create filename from table name
    let table_name = &conditional_tags.conditional_data.table_name;
    let file_name = format!("{}_conditional_tags", table_name.to_lowercase());
    let file_path = output_dir.join(format!("{}.rs", file_name));
    
    let mut content = String::new();
    content.push_str("//! Auto-generated from ExifTool source\n");
    content.push_str("//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen\n\n");
    content.push_str(&conditional_code);
    
    fs::write(&file_path, content)?;
    println!("  ✓ Generated {}", file_path.display());
    
    Ok(file_name.to_string())
}

/// Convert table name to snake_case to match Perl transformation
/// Replicates: s/([A-Z])/_\L$1/g; s/^_//; lc($filename)
fn convert_table_name_to_snake_case(table_name: &str) -> String {
    let mut result = String::new();
    
    for ch in table_name.chars() {
        if ch.is_uppercase() {
            result.push('_');
            result.push(ch.to_ascii_lowercase());
        } else {
            result.push(ch);
        }
    }
    
    // Remove leading underscore if present
    if result.starts_with('_') {
        result.remove(0);
    }
    
    result
}

