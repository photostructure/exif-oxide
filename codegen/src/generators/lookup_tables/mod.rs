//! Pure key-value lookup table generation
//! 
//! This module handles generation of simple HashMap-based lookup tables
//! like Canon white balance values, Nikon lens IDs, etc.
//! 
//! These are straightforward mappings from numeric or string keys to descriptive values.

pub mod standard;
pub mod inline_printconv;
pub mod runtime;

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use crate::schemas::ExtractedTable;
use crate::generators::data_sets;
use crate::common::utils::{module_to_source_path, module_dir_to_source_path};

/// Get the extract subdirectory for a given config type
fn get_extract_subdir(config_type: &str) -> &'static str {
    match config_type {
        "tag_table_structure" => "tag_structures",
        "process_binary_data" => "binary_data",
        "model_detection" => "model_detection",
        "conditional_tags" => "conditional_tags",
        "runtime_table" => "runtime_tables",
        "inline_printconv" => "inline_printconv",
        _ => "simple_tables",
    }
}

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
                
                // Look for extracted boolean set file using standardized naming
                let extract_dir = Path::new("generated/extract").join("boolean_sets");
                let module_base = module_name.trim_end_matches("_pm").to_lowercase();
                let boolean_set_file = format!("{}__boolean_set__{}.json", 
                    module_base, 
                    hash_name.trim_start_matches('%').to_lowercase()
                );
                let boolean_set_path = extract_dir.join(&boolean_set_file);
                
                if boolean_set_path.exists() {
                    // Load the extracted boolean set data
                    let boolean_set_content = fs::read_to_string(&boolean_set_path)?;
                    let boolean_set_data: serde_json::Value = serde_json::from_str(&boolean_set_content)?;
                    
                    // Generate individual file for this boolean set directly from JSON
                    let file_name = generate_boolean_set_file_from_json(
                        hash_name, 
                        &boolean_set_data,
                        table_config,
                        &module_output_dir
                    )?;
                    generated_files.push(file_name);
                    has_content = true;
                }
            }
        }
    }
    
    // Check for all tag table structure configurations  
    // Process Main table first, then subdirectory tables
    if let Ok(entries) = fs::read_dir(&config_dir) {
        let mut config_files: Vec<_> = entries
            .filter_map(Result::ok)
            .filter(|entry| {
                let file_name = entry.file_name();
                let file_name_str = file_name.to_string_lossy();
                file_name_str.ends_with("_tag_table_structure.json") || file_name_str == "tag_table_structure.json"
            })
            .collect();
        
        // Sort to prioritize Main table first
        config_files.sort_by(|a, b| {
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

        for entry in config_files {
            let tag_structure_config = entry.path();
            let config_content = fs::read_to_string(&tag_structure_config)?;
            let config_json: serde_json::Value = serde_json::from_str(&config_content)?;
            
            // Extract table name from config
            if let Some(table_name) = config_json["table"].as_str() {
                // Look for the corresponding extracted tag structure JSON file
                // Use the new standardized filename pattern: module__tag_structure__table.json
                let module_base = module_name.trim_end_matches("_pm");
                let structure_filename = format!("{}__tag_structure__{}.json", 
                                               module_base.to_lowercase(), 
                                               table_name.to_lowercase());
                
                // Tag structure files are stored separately, not in extracted_tables
                let extract_dir = std::env::current_dir()?.join("generated/extract/tag_structures");
                let structure_path = extract_dir.join(&structure_filename);
                
                if structure_path.exists() {
                    let structure_content = fs::read_to_string(&structure_path)?;
                    let mut structure_data: crate::generators::tag_structure::TagStructureData = 
                        serde_json::from_str(&structure_content)?;
                    
                    // Apply output configuration if present
                    if let Some(output_config) = config_json["output"].as_object() {
                        if let Some(enum_name) = output_config["enum_name"].as_str() {
                            structure_data.metadata.enum_name = enum_name.to_string();
                        }
                    }
                    
                    // Generate file for this tag structure
                    let file_name = generate_tag_structure_file(
                        &structure_data,
                        &module_output_dir
                    )?;
                    generated_files.push(file_name);
                    has_content = true;
                } else {
                    println!("    ⚠ Tag structure file not found: {}", structure_path.display());
                }
            }
        }
    }
    
    // Check for process_binary_data.json configuration
    let process_binary_data_config = config_dir.join("process_binary_data.json");
    if process_binary_data_config.exists() {
        let config_content = fs::read_to_string(&process_binary_data_config)?;
        let config_json: serde_json::Value = serde_json::from_str(&config_content)?;
        
        // Extract table name from config
        if let Some(_table_name) = config_json["table"].as_str() {
            // Look for the corresponding extracted binary data JSON file
            let extract_dir = Path::new("generated/extract").join("binary_data");
            let module_base = module_name.trim_end_matches("_pm");
            let binary_data_file = format!("{}__process_binary_data.json", module_base.to_lowercase());
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
        if let Some(_table_name) = config_json["table"].as_str() {
            // Look for the corresponding extracted model detection JSON file
            let extract_dir = Path::new("generated/extract").join("model_detection");
            let module_base = module_name.trim_end_matches("_pm");
            let model_detection_file = format!("{}__model_detection.json", module_base.to_lowercase());
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
        if let Some(_table_name) = config_json["table"].as_str() {
            // Look for the corresponding extracted conditional tags JSON file
            let extract_dir = Path::new("generated/extract").join("conditional_tags");
            let module_base = module_name.trim_end_matches("_pm");
            let conditional_tags_file = format!("{}__conditional_tags.json", module_base.to_lowercase());
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
    
    // Check for runtime_table.json configuration
    let runtime_table_config = config_dir.join("runtime_table.json");
    if runtime_table_config.exists() {
        let config_content = fs::read_to_string(&runtime_table_config)?;
        let config_json: serde_json::Value = serde_json::from_str(&config_content)?;
        
        // Extract each table from the tables array
        if let Some(tables) = config_json["tables"].as_array() {
            for table_config in tables {
                if let Some(table_name) = table_config["table_name"].as_str() {
                    let clean_table_name = table_name.trim_start_matches('%');
                    
                    // Look for the corresponding extracted runtime table JSON file
                    let extract_dir = Path::new("generated/extract").join("runtime_tables");
                    let module_base = module_name.trim_end_matches("_pm");
                    let runtime_table_file = format!("{}__runtime_table__{}.json", 
                                                   module_base.to_lowercase(), 
                                                   clean_table_name.to_lowercase());
                    let runtime_table_path = extract_dir.join(&runtime_table_file);
                    
                    if runtime_table_path.exists() {
                        let runtime_table_content = fs::read_to_string(&runtime_table_path)?;
                        let runtime_table_data: crate::schemas::input::RuntimeTablesData = 
                            serde_json::from_str(&runtime_table_content)?;
                        
                        // Generate file for this RuntimeTable
                        let file_name = generate_runtime_table_file(
                            &runtime_table_data,
                            &module_output_dir,
                            &table_config
                        )?;
                        generated_files.push(file_name);
                        has_content = true;
                    }
                }
            }
        }
    }
    
    // Check for tag_kit.json configuration - create modular tag kit files
    let tag_kit_config = config_dir.join("tag_kit.json");
    if tag_kit_config.exists() {
        let config_content = fs::read_to_string(&tag_kit_config)?;
        let _config_json: serde_json::Value = serde_json::from_str(&config_content)?;
        
        // Look for extracted tag kit JSON file
        let extract_dir = Path::new("generated/extract").join("tag_kits");
        let module_base = module_name.trim_end_matches("_pm");
        let tag_kit_file = format!("{}__tag_kit.json", module_base.to_lowercase());
        let tag_kit_path = extract_dir.join(&tag_kit_file);
        
        if tag_kit_path.exists() {
            let tag_kit_content = fs::read_to_string(&tag_kit_path)?;
            let tag_kit_data: crate::schemas::tag_kit::TagKitExtraction = 
                serde_json::from_str(&tag_kit_content)?;
            
            // Generate modular tag kit files in module directory
            let generated_tag_kit_files = generate_tag_kit_module(&tag_kit_data, &module_output_dir)?;
            for file_name in generated_tag_kit_files {
                generated_files.push(file_name);
            }
            has_content = true;
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
                let extract_dir = Path::new("generated/extract").join("inline_printconv");
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
    
    // Check for regex_patterns.json configuration - create magic number patterns file
    let regex_patterns_config = config_dir.join("regex_patterns.json");
    if regex_patterns_config.exists() {
        let config_content = fs::read_to_string(&regex_patterns_config)?;
        let config_json: serde_json::Value = serde_json::from_str(&config_content)?;
        
        if let Some(tables) = config_json["tables"].as_array() {
            for table_config in tables {
                let _hash_name = table_config["hash_name"].as_str().unwrap_or("");
                
                // Look for extracted regex patterns file using standardized naming
                let extract_dir = Path::new("generated/extract").join("file_types");
                let module_base = module_name.trim_end_matches("_pm").to_lowercase();
                let regex_file = format!("{}__regex_patterns.json", module_base);
                let regex_path = extract_dir.join(&regex_file);
                
                if regex_path.exists() {
                    // Generate regex_patterns.rs file
                    let file_name = generate_regex_patterns_file(
                        &regex_path,
                        table_config,
                        &module_output_dir
                    )?;
                    generated_files.push(file_name);
                    has_content = true;
                }
            }
        }
    }
    
    // Check for additional standalone generated files that might exist in the output directory
    // These are files generated by standalone generators outside the config-based system
    let additional_files = detect_additional_generated_files(&module_output_dir, module_name)?;
    for file_name in additional_files {
        if !generated_files.contains(&file_name) {
            generated_files.push(file_name);
            has_content = true;
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
    let module_path = if let Some(ref source) = inline_data.source {
        module_to_source_path(&source.module)
    } else {
        "unknown module".to_string()
    };
    
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
    
    // Create filename based on table name
    let file_name = if structure_data.source.table == "Main" {
        "tag_structure".to_string()
    } else {
        // For subdirectory tables, use a different filename
        format!("{}_tag_structure", structure_data.source.table.to_lowercase())
    };
    let file_path = output_dir.join(format!("{}.rs", file_name));
    
    let mut content = String::new();
    content.push_str("//! Auto-generated from ExifTool source\n");
    content.push_str("//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen\n\n");
    content.push_str(&structure_code);
    
    fs::write(&file_path, content)?;
    println!("  ✓ Generated {}", file_path.display());
    
    Ok(file_name)
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

/// Detect additional generated files that exist in the output directory
/// These are files created by standalone generators outside the config-based system
fn detect_additional_generated_files(output_dir: &Path, _module_name: &str) -> Result<Vec<String>> {
    let mut additional_files = Vec::new();
    
    // List of known standalone generated file patterns
    let standalone_patterns = [
        "offset_patterns",     // Sony offset patterns
        "tag_structure",       // Canon/Olympus tag structures
    ];
    
    // Check if the output directory exists
    if !output_dir.exists() {
        return Ok(additional_files);
    }
    
    // Collect all .rs files first
    let entries: Result<Vec<_>, _> = std::fs::read_dir(output_dir)?
        .collect();
    let entries = entries?;
    
    let rs_files: Vec<String> = entries
        .iter()
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
                path.file_stem().and_then(|s| s.to_str()).map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();
    
    // Check for potential conflicts in tag_structure files for Olympus
    let has_conflicting_olympus_structs = rs_files.contains(&"tag_structure".to_string()) && 
                                         rs_files.contains(&"equipment_tag_structure".to_string());
    
    // Now check for standalone patterns
    for file_stem in &rs_files {
        for pattern in &standalone_patterns {
            if file_stem == pattern || file_stem.ends_with(&format!("_{}", pattern)) {
                // Special case: detect Olympus naming conflicts
                if file_stem == "tag_structure" && has_conflicting_olympus_structs {
                    println!("  ⚠ Including tag_structure.rs but detected naming conflict with equipment_tag_structure.rs");
                    println!("      Both define OlympusDataType enum - this may cause compilation errors");
                }
                
                additional_files.push(file_stem.clone());
                println!("  ✓ Detected standalone generated file: {}.rs", file_stem);
                break;
            }
        }
    }
    
    // Check for subdirectories that contain mod.rs files
    for entry in entries.iter() {
        let path = entry.path();
        if path.is_dir() {
            // Check if this directory contains a mod.rs file
            let mod_file = path.join("mod.rs");
            if mod_file.exists() {
                if let Some(dir_name) = path.file_name() {
                    let name = dir_name.to_string_lossy().to_string();
                    if !additional_files.contains(&name) {
                        additional_files.push(name.clone());
                        println!("  ✓ Detected generated subdirectory: {}/", name);
                    }
                }
            }
        }
    }
    
    // Sort for deterministic output
    additional_files.sort();
    Ok(additional_files)
}

fn generate_runtime_table_file(
    runtime_data: &crate::schemas::input::RuntimeTablesData,
    output_dir: &Path,
    table_config: &serde_json::Value,
) -> Result<String> {
    let runtime_code = runtime::generate_runtime_table(runtime_data, table_config)?;
    
    // Extract table name for filename
    let _table_name = table_config["table_name"].as_str()
        .unwrap_or("unknown")
        .trim_start_matches('%')
        .to_lowercase();
    let function_name = table_config["function_name"].as_str()
        .unwrap_or("create_table")
        .to_lowercase();
    
    let filename = format!("{}_runtime.rs", function_name);
    let output_path = output_dir.join(&filename);
    
    fs::write(&output_path, runtime_code)?;
    println!("  📋 Generated runtime table file: {}", filename);
    
    Ok(filename.replace(".rs", ""))
}

/// Generate modular tag kit files within the module directory
/// This creates a tag_kit subdirectory with modular category files
fn generate_tag_kit_module(
    tag_kit_data: &crate::schemas::tag_kit::TagKitExtraction,
    module_output_dir: &Path,
) -> Result<Vec<String>> {
    let mut generated_files = Vec::new();
    
    // Create tag_kit subdirectory within the module 
    let tag_kit_dir = module_output_dir.join("tag_kit");
    fs::create_dir_all(&tag_kit_dir)?;
    
    // Use existing modular generation logic but output to module subdirectory
    let categories = crate::generators::tag_kit_split::split_tag_kits(&tag_kit_data.tag_kits);
    let mut total_print_conv_count = 0;
    let mut category_modules = Vec::new();
    
    // Generate a module for each category with tags (sorted for deterministic order)
    let mut sorted_categories: Vec<_> = categories.iter().collect();
    sorted_categories.sort_by_key(|(category, _)| *category);
    
    for (category, tag_kits) in sorted_categories {
        if tag_kits.is_empty() {
            continue;
        }
        
        let module_name_cat = category.module_name();
        let (module_code, print_conv_count) = generate_tag_kit_category_module(
            module_name_cat,
            tag_kits,
            &tag_kit_data.source,
            &mut total_print_conv_count,
        )?;
        
        // Write category module to tag_kit subdirectory
        let category_file = format!("{}.rs", module_name_cat);
        let category_path = tag_kit_dir.join(&category_file);
        fs::write(&category_path, module_code)?;
        
        category_modules.push(module_name_cat.to_string());
        
        println!("  🏷️  Generated tag_kit/{} with {} tags, {} PrintConv tables",
            module_name_cat,
            tag_kits.len(),
            print_conv_count
        );
    }
    
    // Generate mod.rs for tag_kit subdirectory
    let tag_kit_mod_code = generate_tag_kit_mod_file(&category_modules, tag_kit_data)?;
    let mod_path = tag_kit_dir.join("mod.rs");
    fs::write(&mod_path, tag_kit_mod_code)?;
    
    // Return "tag_kit" as the generated module name (for parent mod.rs)
    generated_files.push("tag_kit".to_string());
    
    println!("  🏷️  Generated modular tag_kit with {} tags split into {} categories", 
        tag_kit_data.tag_kits.len(),
        category_modules.len()
    );
    
    Ok(generated_files)
}

/// Generate code for a single tag kit category module
/// This is adapted from the existing tag_kit_modular.rs code
fn generate_tag_kit_category_module(
    category_name: &str,
    tag_kits: &[&crate::schemas::tag_kit::TagKit],
    source: &crate::schemas::tag_kit::SourceInfo,
    print_conv_counter: &mut usize,
) -> Result<(String, usize)> {
    let mut code = String::new();
    
    // Header with warning suppression at the very top
    code.push_str(&format!("//! Tag kits for {} category from {}\n", category_name, source.module));
    code.push_str("//!\n");
    code.push_str("//! This file is automatically generated by codegen.\n");
    code.push_str("//! DO NOT EDIT MANUALLY - changes will be overwritten.\n\n");
    code.push_str("#![allow(unused_imports)]\n");
    code.push_str("#![allow(unused_mut)]\n\n");
    
    code.push_str("use std::collections::HashMap;\n");
    code.push_str("use std::sync::LazyLock;\n");
    code.push_str("use crate::types::TagValue;\n");
    code.push_str("use super::{TagKitDef, PrintConvType};\n\n");
    
    // Generate PrintConv lookup tables for this category
    let mut local_print_conv_count = 0;
    for tag_kit in tag_kits {
        if tag_kit.print_conv_type == "Simple" {
            if let Some(print_conv_data) = &tag_kit.print_conv_data {
                if let Some(data_obj) = print_conv_data.as_object() {
                    let const_name = format!("PRINT_CONV_{}", *print_conv_counter);
                    *print_conv_counter += 1;
                    local_print_conv_count += 1;
                    
                    code.push_str(&format!("static {}: LazyLock<HashMap<String, &'static str>> = LazyLock::new(|| {{\n", const_name));
                    code.push_str("    let mut map = HashMap::new();\n");
                    
                    for (key, value) in data_obj {
                        if let Some(val_str) = value.as_str() {
                            code.push_str(&format!("    map.insert(\"{}\".to_string(), \"{}\");\n", 
                                crate::common::escape_string(key), 
                                crate::common::escape_string(val_str)));
                        }
                    }
                    
                    code.push_str("    map\n");
                    code.push_str("});\n\n");
                }
            }
        }
    }
    
    // Generate tag definitions function
    code.push_str(&format!("/// Get tag definitions for {} category\n", category_name));
    code.push_str(&format!("pub fn get_{}_tags() -> Vec<(u32, TagKitDef)> {{\n", category_name));
    code.push_str("    vec![\n");
    
    // Reset print conv counter for this category
    let mut category_print_conv_index = *print_conv_counter - local_print_conv_count;
    
    for tag_kit in tag_kits {
        let tag_id = tag_kit.tag_id.parse::<u32>().unwrap_or(0);
        
        code.push_str(&format!("        ({}, TagKitDef {{\n", tag_id));
        code.push_str(&format!("            id: {},\n", tag_id));
        code.push_str(&format!("            name: \"{}\",\n", crate::common::escape_string(&tag_kit.name)));
        code.push_str(&format!("            format: \"{}\",\n", crate::common::escape_string(&tag_kit.format)));
        
        // Groups (currently empty in extraction)
        code.push_str("            groups: HashMap::new(),\n");
        
        // Writable
        code.push_str(&format!("            writable: {},\n", 
            if tag_kit.writable.is_some() { "true" } else { "false" }));
        
        // Notes
        if let Some(notes) = &tag_kit.notes {
            let trimmed_notes = notes.trim();
            code.push_str(&format!("            notes: Some(\"{}\"),\n", crate::common::escape_string(trimmed_notes)));
        } else {
            code.push_str("            notes: None,\n");
        }
        
        // PrintConv
        match tag_kit.print_conv_type.as_str() {
            "Simple" => {
                if tag_kit.print_conv_data.is_some() {
                    code.push_str(&format!("            print_conv: PrintConvType::Simple(&PRINT_CONV_{}),\n", 
                        category_print_conv_index));
                    category_print_conv_index += 1;
                } else {
                    code.push_str("            print_conv: PrintConvType::None,\n");
                }
            }
            "Expression" => {
                if let Some(expr_data) = &tag_kit.print_conv_data {
                    if let Some(expr_str) = expr_data.as_str() {
                        code.push_str(&format!("            print_conv: PrintConvType::Expression(\"{}\"),\n", 
                            crate::common::escape_string(expr_str)));
                    } else {
                        code.push_str("            print_conv: PrintConvType::Expression(\"unknown\"),\n");
                    }
                } else {
                    code.push_str("            print_conv: PrintConvType::Expression(\"unknown\"),\n");
                }
            }
            "Manual" => {
                if let Some(func_name) = &tag_kit.print_conv_data {
                    if let Some(name_str) = func_name.as_str() {
                        code.push_str(&format!("            print_conv: PrintConvType::Manual(\"{}\"),\n", 
                            crate::common::escape_string(name_str)));
                    } else {
                        code.push_str("            print_conv: PrintConvType::Manual(\"unknown\"),\n");
                    }
                } else {
                    code.push_str("            print_conv: PrintConvType::Manual(\"unknown\"),\n");
                }
            }
            _ => {
                code.push_str("            print_conv: PrintConvType::None,\n");
            }
        }
        
        // ValueConv
        if let Some(value_conv) = &tag_kit.value_conv {
            code.push_str(&format!("            value_conv: Some(\"{}\"),\n", crate::common::escape_string(value_conv)));
        } else {
            code.push_str("            value_conv: None,\n");
        }
        
        code.push_str("        }),\n");
    }
    
    code.push_str("    ]\n");
    code.push_str("}\n");
    
    Ok((code, local_print_conv_count))
}

/// Generate the tag_kit/mod.rs file that combines all category modules
fn generate_tag_kit_mod_file(
    category_modules: &[String], 
    tag_kit_data: &crate::schemas::tag_kit::TagKitExtraction,
) -> Result<String> {
    let mut code = String::new();
    
    // Header
    code.push_str("//! Modular tag kits with embedded PrintConv\n");
    code.push_str("//!\n");
    code.push_str("//! This file is automatically generated by codegen.\n");
    code.push_str("//! DO NOT EDIT MANUALLY - changes will be overwritten.\n");
    code.push_str("//!\n");
    code.push_str(&format!("//! Generated from: {} table: {}\n", tag_kit_data.source.module, tag_kit_data.source.table));
    code.push_str("//!\n\n");
    // NOTE: Do NOT add extraction timestamps here - they create spurious git diffs
    // that make it impossible to track real changes to generated code
    
    // Module declarations
    for module in category_modules {
        code.push_str(&format!("pub mod {};\n", module));
    }
    code.push_str("\n");
    
    // Common imports
    code.push_str("use std::collections::HashMap;\n");
    code.push_str("use std::sync::LazyLock;\n");
    code.push_str("use crate::types::TagValue;\n");
    code.push_str("use crate::expressions::ExpressionEvaluator;\n\n");
    
    // Tag kit definition struct
    code.push_str("#[derive(Debug, Clone)]\n");
    code.push_str("pub struct TagKitDef {\n");
    code.push_str("    pub id: u32,\n");
    code.push_str("    pub name: &'static str,\n");
    code.push_str("    pub format: &'static str,\n");
    code.push_str("    pub groups: HashMap<&'static str, &'static str>,\n");
    code.push_str("    pub writable: bool,\n");
    code.push_str("    pub notes: Option<&'static str>,\n");
    code.push_str("    pub print_conv: PrintConvType,\n");
    code.push_str("    pub value_conv: Option<&'static str>,\n");
    code.push_str("}\n\n");
    
    // PrintConv type enum
    code.push_str("#[derive(Debug, Clone)]\n");
    code.push_str("pub enum PrintConvType {\n");
    code.push_str("    None,\n");
    code.push_str("    Simple(&'static HashMap<String, &'static str>),\n");
    code.push_str("    Expression(&'static str),\n");
    code.push_str("    Manual(&'static str),\n");
    code.push_str("}\n\n");
    
    // Combined tag map
    code.push_str("/// All tag kits for this module\n");
    code.push_str("pub static TAG_KITS: LazyLock<HashMap<u32, TagKitDef>> = LazyLock::new(|| {\n");
    code.push_str("    let mut map = HashMap::new();\n");
    code.push_str("    \n");
    
    // Add tags from each module
    for module in category_modules {
        code.push_str(&format!("    // {} tags\n", module));
        code.push_str(&format!("    for (id, tag_def) in {}::get_{}_tags() {{\n", module, module));
        code.push_str("        map.insert(id, tag_def);\n");
        code.push_str("    }\n");
        code.push_str("    \n");
    }
    
    code.push_str("    map\n");
    code.push_str("});\n\n");
    
    // Apply PrintConv function
    code.push_str("/// Apply PrintConv for a tag from this module\n");
    code.push_str("#[allow(clippy::ptr_arg)]\n");
    code.push_str("pub fn apply_print_conv(\n");
    code.push_str("    tag_id: u32,\n");
    code.push_str("    value: &TagValue,\n");
    code.push_str("    _evaluator: &mut ExpressionEvaluator,\n");
    code.push_str("    _errors: &mut Vec<String>,\n");
    code.push_str("    warnings: &mut Vec<String>,\n");
    code.push_str(") -> TagValue {\n");
    code.push_str("    if let Some(tag_kit) = TAG_KITS.get(&tag_id) {\n");
    code.push_str("        match &tag_kit.print_conv {\n");
    code.push_str("            PrintConvType::None => value.clone(),\n");
    code.push_str("            PrintConvType::Simple(lookup) => {\n");
    code.push_str("                // Convert value to string key for lookup\n");
    code.push_str("                let key = match value {\n");
    code.push_str("                    TagValue::U8(v) => v.to_string(),\n");
    code.push_str("                    TagValue::U16(v) => v.to_string(),\n");
    code.push_str("                    TagValue::U32(v) => v.to_string(),\n");
    code.push_str("                    TagValue::I16(v) => v.to_string(),\n");
    code.push_str("                    TagValue::I32(v) => v.to_string(),\n");
    code.push_str("                    TagValue::String(s) => s.clone(),\n");
    code.push_str("                    _ => return value.clone(),\n");
    code.push_str("                };\n");
    code.push_str("                \n");
    code.push_str("                if let Some(result) = lookup.get(&key) {\n");
    code.push_str("                    TagValue::String(result.to_string())\n");
    code.push_str("                } else {\n");
    code.push_str("                    TagValue::String(format!(\"Unknown ({})\", value))\n");
    code.push_str("                }\n");
    code.push_str("            }\n");
    code.push_str("            PrintConvType::Expression(expr) => {\n");
    code.push_str("                // TODO: Implement expression evaluation\n");
    code.push_str("                warnings.push(format!(\"Expression PrintConv not yet implemented for tag {}: {}\", \n");
    code.push_str("                    tag_kit.name, expr));\n");
    code.push_str("                value.clone()\n");
    code.push_str("            }\n");
    code.push_str("            PrintConvType::Manual(func_name) => {\n");
    code.push_str("                // TODO: Look up in manual registry\n");
    code.push_str("                warnings.push(format!(\"Manual PrintConv '{}' not found for tag {}\", \n");
    code.push_str("                    func_name, tag_kit.name));\n");
    code.push_str("                value.clone()\n");
    code.push_str("            }\n");
    code.push_str("        }\n");
    code.push_str("    } else {\n");
    code.push_str("        // Tag not found in kit\n");
    code.push_str("        value.clone()\n");
    code.push_str("    }\n");
    code.push_str("}\n");
    
    Ok(code)
}

/// Generate individual file for a boolean set from JSON data
fn generate_boolean_set_file_from_json(
    hash_name: &str,
    boolean_set_data: &serde_json::Value,
    table_config: &serde_json::Value,
    output_dir: &Path,
) -> Result<String> {
    let mut code = String::new();
    
    // Get config values
    let constant_name = table_config["constant_name"].as_str().unwrap_or("BOOLEAN_SET");
    let description = table_config["description"].as_str().unwrap_or("Boolean set");
    let key_type = table_config["key_type"].as_str().unwrap_or("String");
    
    // Generate HashMap
    code.push_str("use std::collections::HashMap;\n");
    code.push_str("use std::sync::LazyLock;\n\n");
    
    code.push_str(&format!("/// {}\n", description));
    code.push_str(&format!("pub static {}: LazyLock<HashMap<{}, bool>> = LazyLock::new(|| {{\n", constant_name, key_type));
    code.push_str("    let mut map = HashMap::new();\n");
    
    // Add entries
    if let Some(entries) = boolean_set_data["entries"].as_array() {
        for entry in entries {
            if let Some(key) = entry["key"].as_str() {
                if key_type == "String" {
                    code.push_str(&format!("    map.insert(\"{}\".to_string(), true);\n", 
                        crate::common::escape_string(key)));
                } else {
                    code.push_str(&format!("    map.insert(\"{}\", true);\n", 
                        crate::common::escape_string(key)));
                }
            }
        }
    }
    
    code.push_str("    map\n");
    code.push_str("});\n\n");
    
    // Generate lookup function
    let function_name = format!("lookup_{}", hash_name.trim_start_matches('%').to_lowercase());
    code.push_str(&format!("/// Check if key exists in {}\n", description));
    code.push_str(&format!("pub fn {}(key: &{}) -> bool {{\n", function_name, key_type));
    code.push_str(&format!("    {}.contains_key(key)\n", constant_name));
    code.push_str("}\n");
    
    // Create descriptive filename from hash name
    let file_name = hash_name_to_filename(hash_name);
    let file_path = output_dir.join(format!("{}.rs", file_name));
    
    let mut content = String::new();
    content.push_str(&format!("//! {}\n", description));
    content.push_str("//! \n//! Auto-generated from ExifTool source\n");
    content.push_str("//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen\n\n");
    content.push_str(&code);
    
    fs::write(&file_path, content)?;
    println!("  ✓ Generated {}", file_path.display());
    
    Ok(file_name)
}

/// Generate regex_patterns.rs file from extracted regex patterns JSON
fn generate_regex_patterns_file(
    regex_file_path: &Path,
    table_config: &serde_json::Value,
    output_dir: &Path,
) -> Result<String> {
    let regex_content = fs::read_to_string(regex_file_path)?;
    let regex_data: serde_json::Value = serde_json::from_str(&regex_content)?;
    
    let mut code = String::new();
    
    // Get config values
    let constant_name = table_config["constant_name"].as_str().unwrap_or("REGEX_PATTERNS");
    let description = table_config["description"].as_str().unwrap_or("Regex patterns");
    
    // Header
    code.push_str("//! Regex patterns for file type detection\n");
    code.push_str("//!\n");
    code.push_str("//! This file is automatically generated by codegen.\n");
    code.push_str("//! DO NOT EDIT MANUALLY - changes will be overwritten.\n\n");
    
    code.push_str("use std::collections::HashMap;\n");
    code.push_str("use std::sync::LazyLock;\n\n");
    
    // Generate the regex patterns map
    code.push_str(&format!("/// {}\n", description));
    code.push_str(&format!("pub static {}: LazyLock<HashMap<&'static str, &'static [u8]>> = LazyLock::new(|| {{\n", constant_name));
    code.push_str("    let mut map = HashMap::new();\n");
    
    // Add entries from magic_patterns array (keeping original field name from extraction)
    if let Some(patterns) = regex_data["magic_patterns"].as_array() {
        for pattern in patterns {
            if let (Some(file_type), Some(pattern_str)) = (
                pattern["file_type"].as_str(),
                pattern["pattern"].as_str()
            ) {
                // Convert pattern string to bytes - escape special characters for Rust byte literal
                let escaped_pattern = escape_pattern_to_bytes(pattern_str);
                code.push_str(&format!("    map.insert(\"{}\", &{} as &[u8]);\n", 
                    file_type,
                    escaped_pattern
                ));
            }
        }
    }
    
    code.push_str("    map\n");
    code.push_str("});\n\n");
    
    // Generate lookup function
    code.push_str("/// Detect file type by regex pattern\n");
    code.push_str("pub fn detect_file_type_by_regex(data: &[u8]) -> Option<&'static str> {\n");
    code.push_str("    for (file_type, pattern) in REGEX_PATTERNS.iter() {\n");
    code.push_str("        if data.starts_with(pattern) {\n");
    code.push_str("            return Some(file_type);\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    code.push_str("    None\n");
    code.push_str("}\n");
    
    // Write to file
    let file_name = "regex_patterns.rs";
    let file_path = output_dir.join(file_name);
    fs::write(&file_path, code)?;
    
    println!("  ✓ Generated {}", file_path.display());
    
    Ok(file_name.replace(".rs", ""))
}

/// Convert pattern string with escape sequences to byte array literal
fn escape_pattern_to_bytes(pattern: &str) -> String {
    let mut result = String::from("[");
    let mut chars = pattern.chars().peekable();
    let mut first = true;
    
    while let Some(ch) = chars.next() {
        if !first {
            result.push_str(", ");
        }
        first = false;
        
        if ch == '\\' {
            if let Some(&next_ch) = chars.peek() {
                if next_ch == 'x' {
                    // Hex escape sequence like \xff
                    chars.next(); // consume 'x'
                    let hex1 = chars.next().unwrap_or('0');
                    let hex2 = chars.next().unwrap_or('0');
                    let hex_str = format!("{}{}", hex1, hex2);
                    if let Ok(byte_val) = u8::from_str_radix(&hex_str, 16) {
                        result.push_str(&format!("0x{:02x}u8", byte_val));
                    } else {
                        result.push_str("0x00u8");
                    }
                } else {
                    // Other escape sequences, treat as literal
                    result.push_str(&format!("0x{:02x}u8", ch as u8));
                }
            } else {
                result.push_str(&format!("0x{:02x}u8", ch as u8));
            }
        } else {
            result.push_str(&format!("0x{:02x}u8", ch as u8));
        }
    }
    
    result.push(']');
    result
}