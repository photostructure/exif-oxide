//! Pure key-value lookup table generation
//! 
//! This module handles generation of simple HashMap-based lookup tables
//! like Canon white balance values, Nikon lens IDs, etc.
//! 
//! These are straightforward mappings from numeric or string keys to descriptive values.

pub mod standard;
pub mod inline_printconv;
pub mod runtime;

mod file_generation;
mod path_utils;
mod config_processor;

use anyhow::{Result, Context};
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use crate::schemas::ExtractedTable;
use crate::generators::data_sets;
use tracing::{debug, warn};

/// Generate a single print conv entry for a HashMap
pub fn generate_print_conv_entry(code: &mut String, key: &str, value: &serde_json::Value) {
    match value {
        serde_json::Value::String(val_str) => {
            code.push_str(&format!("    map.insert(\"{}\".to_string(), \"{}\");\n", 
                crate::common::escape_string(key), 
                crate::common::escape_string(val_str)));
        },
        serde_json::Value::Number(num) => {
            // Handle numeric values by converting them to strings
            code.push_str(&format!("    map.insert(\"{}\".to_string(), \"{}\");\n", 
                crate::common::escape_string(key), 
                num.to_string()));
        },
        _ => {
            // Skip other types (arrays, objects, booleans, null)
        }
    }
}

/// Process simple_table.json configuration
fn process_simple_table_config(
    config_dir: &Path,
    _module_name: &str,
    extracted_tables: &HashMap<String, ExtractedTable>,
    output_dir: &Path,
) -> Result<Vec<String>> {
    config_processor::process_config_if_exists(config_dir, "simple_table.json", |config| {
        config_processor::process_tables_config(config, |table_config| {
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
                let file_name = generate_table_file(hash_name, &updated_table, output_dir)?;
                Ok(Some(file_name))
            } else {
                Ok(None)
            }
        })
    })
}

/// Process boolean_set.json configuration
fn process_boolean_set_config(
    config_dir: &Path,
    module_name: &str,
    output_dir: &Path,
) -> Result<Vec<String>> {
    config_processor::process_config_if_exists(config_dir, "boolean_set.json", |config| {
        config_processor::process_tables_config(config, |table_config| {
            let hash_name = table_config["hash_name"].as_str().unwrap_or("");
            
            // Look for extracted boolean set file
            let boolean_set_file = path_utils::construct_extract_filename(
                module_name,
                &format!("boolean_set__{}", hash_name.trim_start_matches('%').to_lowercase())
            );
            let boolean_set_path = path_utils::get_extract_dir("boolean_sets").join(format!("{}.json", &boolean_set_file));
            
            debug!("Looking for boolean set file: {} at path: {:?}", hash_name, boolean_set_path);
            
            if boolean_set_path.exists() {
                // Load and generate
                let boolean_set_data: serde_json::Value = config_processor::load_extracted_json(&boolean_set_path)?;
                let file_name = generate_boolean_set_file_from_json(
                    hash_name,
                    &boolean_set_data,
                    table_config,
                    output_dir
                )?;
                Ok(Some(file_name))
            } else {
                warn!("Boolean set file not found for {}: {:?}", hash_name, boolean_set_path);
                Ok(None)
            }
        })
    })
}

/// Process a single-extract configuration (process_binary_data, model_detection, etc.)
fn process_single_extract_config<T, F>(
    config_dir: &Path,
    module_name: &str,
    config_filename: &str,
    extract_subdir: &str,
    extract_pattern: &str,
    output_dir: &Path,
    generator: F,
) -> Result<Vec<String>>
where
    T: serde::de::DeserializeOwned,
    F: FnOnce(&T, &Path) -> Result<String>,
{
    config_processor::process_config_if_exists(config_dir, config_filename, |config| {
        if let Some(_table_name) = config["table"].as_str() {
            let extract_file = path_utils::construct_extract_filename(module_name, extract_pattern);
            let extract_path = path_utils::get_extract_dir(extract_subdir).join(&extract_file);
            
            if extract_path.exists() {
                let data: T = config_processor::load_extracted_json(&extract_path)?;
                let file_name = generator(&data, output_dir)?;
                Ok(vec![file_name])
            } else {
                Ok(vec![])
            }
        } else {
            Ok(vec![])
        }
    })
}

/// Process configuration files from a directory and generate modular structure
pub fn process_config_directory(
    config_dir: &Path,
    module_name: &str,
    extracted_tables: &HashMap<String, ExtractedTable>,
    output_dir: &str,
) -> Result<()> {
    debug!("    Processing config directory for module: {}", module_name);
    
    let mut generated_files = Vec::new();
    let mut has_content = false;
    
    // Create module directory
    let module_output_dir = Path::new(output_dir).join(module_name);
    fs::create_dir_all(&module_output_dir)?;
    
    // Process simple_table configuration
    let simple_table_files = process_simple_table_config(config_dir, module_name, extracted_tables, &module_output_dir)?;
    generated_files.extend(simple_table_files);
    has_content |= !generated_files.is_empty();
    
    // Process boolean_set configuration
    let boolean_set_files = process_boolean_set_config(config_dir, module_name, &module_output_dir)?;
    has_content |= !boolean_set_files.is_empty();
    generated_files.extend(boolean_set_files);
    
    // Check for all tag table structure configurations  
    // Process Main table first, then subdirectory tables
    if let Ok(entries) = fs::read_dir(config_dir) {
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
            let config_json: serde_json::Value = serde_json::from_str(&config_content)
                .with_context(|| format!("Failed to parse tag structure config: {}", tag_structure_config.display()))?;
            
            // Extract table name from config
            if let Some(table_name) = config_json["table"].as_str() {
                // Look for the corresponding extracted tag structure JSON file
                // Use the new standardized filename pattern: module__tag_structure__table.json
                let structure_filename = path_utils::construct_extract_filename(
                    module_name,
                    &format!("tag_structure__{}", table_name.to_lowercase())
                );
                
                // Tag structure files are stored separately, not in extracted_tables
                let extract_dir = std::env::current_dir()?.join(path_utils::get_extract_dir("tag_structures"));
                let structure_path = extract_dir.join(&structure_filename);
                
                if structure_path.exists() {
                    let structure_content = fs::read_to_string(&structure_path)?;
                    let mut structure_data: crate::generators::tag_structure::TagStructureData = 
                        serde_json::from_str(&structure_content)
                        .with_context(|| format!("Failed to parse tag structure data: {}", structure_path.display()))?;
                    
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
                    warn!("    ⚠ Tag structure file not found: {}", structure_path.display());
                }
            }
        }
    }
    
    // Process process_binary_data configuration (handles multiple tables)
    let binary_data_files = process_process_binary_data_config(
        config_dir,
        module_name,
        &module_output_dir,
    )?;
    has_content |= !binary_data_files.is_empty();
    generated_files.extend(binary_data_files);
    
    // Process model_detection configuration
    let model_detection_files = process_single_extract_config::<
        crate::generators::model_detection::ModelDetectionExtraction,
        _
    >(
        config_dir,
        module_name,
        "model_detection.json",
        "model_detection",
        "model_detection.json",
        &module_output_dir,
        generate_model_detection_file,
    )?;
    has_content |= !model_detection_files.is_empty();
    generated_files.extend(model_detection_files);
    
    // Process conditional_tags configuration
    let conditional_tags_files = process_single_extract_config::<
        crate::generators::conditional_tags::ConditionalTagsExtraction,
        _
    >(
        config_dir,
        module_name,
        "conditional_tags.json",
        "conditional_tags",
        "conditional_tags.json",
        &module_output_dir,
        generate_conditional_tags_file,
    )?;
    has_content |= !conditional_tags_files.is_empty();
    generated_files.extend(conditional_tags_files);
    
    // Process runtime_table configuration
    let runtime_table_files = process_runtime_table_config(config_dir, module_name, &module_output_dir)?;
    has_content |= !runtime_table_files.is_empty();
    generated_files.extend(runtime_table_files);
    
    // Process tag_kit configuration
    let tag_kit_files = process_tag_kit_config(config_dir, module_name, &module_output_dir)?;
    has_content |= !tag_kit_files.is_empty();
    generated_files.extend(tag_kit_files);
    
    // Process inline_printconv configuration
    let inline_printconv_files = process_inline_printconv_config(config_dir, module_name, &module_output_dir)?;
    has_content |= !inline_printconv_files.is_empty();
    generated_files.extend(inline_printconv_files);
    
    // Process regex_patterns configuration
    let regex_patterns_files = process_regex_patterns_config(config_dir, module_name, &module_output_dir)?;
    has_content |= !regex_patterns_files.is_empty();
    generated_files.extend(regex_patterns_files);
    
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
        file_generation::generate_module_mod_file(&generated_files, module_name, &module_output_dir)?;
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
    let file_name = path_utils::hash_name_to_filename(hash_name);
    let file_path = output_dir.join(format!("{file_name}.rs"));
    
    file_generation::FileGenerator::new(&extracted_table.metadata.description)
        .with_source_module(&extracted_table.source.module)
        .with_standard_imports()
        .with_content(table_code)
        .write_to_file(&file_path)
}

/// Generate individual file for a boolean set
#[allow(dead_code)]
fn generate_boolean_set_file(
    hash_name: &str,
    extracted_table: &ExtractedTable,
    output_dir: &Path,
) -> Result<String> {
    let table_code = data_sets::boolean::generate_boolean_set(hash_name, extracted_table)?;
    let file_name = path_utils::hash_name_to_filename(hash_name);
    let file_path = output_dir.join(format!("{file_name}.rs"));
    
    file_generation::FileGenerator::new(&extracted_table.metadata.description)
        .with_source_module(&extracted_table.source.module)
        .with_hashset_imports()
        .with_content(table_code)
        .write_to_file(&file_path)
}



/// Generate file for inline PrintConv tables
fn generate_inline_printconv_file(
    inline_data: &inline_printconv::InlinePrintConvData,
    table_name: &str,
    output_dir: &Path,
) -> Result<String> {
    let table_code = inline_printconv::generate_inline_printconv_file(inline_data, table_name)?;
    let file_name = table_name
        .chars()
        .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
        .collect::<String>() + "_inline";
    let file_path = output_dir.join(format!("{file_name}.rs"));
    
    let description = format!("Inline PrintConv tables for {table_name} table");
    let module = inline_data.source.as_ref().map(|s| s.module.as_str()).unwrap_or("unknown module");
    
    file_generation::FileGenerator::new(description)
        .with_source_module(module)
        .with_source_table(table_name)
        .with_standard_imports()
        .with_content(table_code)
        .write_to_file(&file_path)
}

/// Generate individual file for a tag table structure
fn generate_tag_structure_file(
    structure_data: &crate::generators::tag_structure::TagStructureData,
    output_dir: &Path,
) -> Result<String> {
    let structure_code = crate::generators::tag_structure::generate_tag_structure(structure_data)?;
    let file_name = if structure_data.source.table == "Main" {
        "tag_structure".to_string()
    } else {
        format!("{}_tag_structure", structure_data.source.table.to_lowercase())
    };
    let file_path = output_dir.join(format!("{file_name}.rs"));
    
    file_generation::FileGenerator::new("Auto-generated from ExifTool source")
        .with_content(structure_code)
        .write_to_file(&file_path)
}

/// Process process_binary_data configuration with support for multiple tables
fn process_process_binary_data_config(
    config_dir: &Path,
    module_name: &str,
    output_dir: &Path,
) -> Result<Vec<String>> {
    config_processor::process_config_if_exists(config_dir, "process_binary_data.json", |config| {
        let mut generated_files = Vec::new();
        
        // Handle both single table and multiple tables format
        let table_names: Vec<String> = if let Some(table) = config["table"].as_str() {
            // Legacy single table format
            vec![table.to_string()]
        } else if let Some(tables_array) = config["tables"].as_array() {
            // New multi-table format
            tables_array.iter()
                .filter_map(|table| table.as_str())
                .map(|name| name.to_string())
                .collect()
        } else {
            return Ok(vec![]); // No tables to process
        };
        
        // Process each table
        for table_name in table_names {
            // Construct extract filename for this specific table
            let extract_file = format!("{}__process_binary_data__{}.json", 
                module_name.trim_end_matches("_pm").to_lowercase(),
                table_name.to_lowercase()
            );
            let extract_path = path_utils::get_extract_dir("binary_data").join(&extract_file);
            
            if extract_path.exists() {
                let data: crate::generators::process_binary_data::ProcessBinaryDataExtraction = 
                    config_processor::load_extracted_json(&extract_path)?;
                let file_name = generate_process_binary_data_file(&data, output_dir)?;
                generated_files.push(file_name);
            }
        }
        
        Ok(generated_files)
    })
}

fn generate_process_binary_data_file(
    binary_data: &crate::generators::process_binary_data::ProcessBinaryDataExtraction,
    output_dir: &Path,
) -> Result<String> {
    let binary_code = crate::generators::process_binary_data::generate_process_binary_data(binary_data)?;
    let table_name = &binary_data.table_data.table_name;
    let file_name = format!("{}_binary_data", table_name.to_lowercase());
    let file_path = output_dir.join(format!("{file_name}.rs"));
    
    file_generation::FileGenerator::new("Auto-generated from ExifTool source")
        .with_content(binary_code)
        .write_to_file(&file_path)
}

fn generate_model_detection_file(
    model_detection: &crate::generators::model_detection::ModelDetectionExtraction,
    output_dir: &Path,
) -> Result<String> {
    let model_code = crate::generators::model_detection::generate_model_detection(model_detection)?;
    let table_name = &model_detection.patterns_data.table_name;
    let file_name = format!("{}_model_detection", table_name.to_lowercase());
    let file_path = output_dir.join(format!("{file_name}.rs"));
    
    file_generation::FileGenerator::new("Auto-generated from ExifTool source")
        .with_content(model_code)
        .write_to_file(&file_path)
}

fn generate_conditional_tags_file(
    conditional_tags: &crate::generators::conditional_tags::ConditionalTagsExtraction,
    output_dir: &Path,
) -> Result<String> {
    let conditional_code = crate::generators::conditional_tags::generate_conditional_tags(conditional_tags)?;
    let table_name = &conditional_tags.conditional_data.table_name;
    let file_name = format!("{}_conditional_tags", table_name.to_lowercase());
    let file_path = output_dir.join(format!("{file_name}.rs"));
    
    file_generation::FileGenerator::new("Auto-generated from ExifTool source")
        .with_content(conditional_code)
        .write_to_file(&file_path)
}

/// Integrate Main table tags that have subdirectories into the tag kit
fn integrate_main_table_tags(
    tag_kit_data: &mut crate::schemas::tag_kit::TagKitExtraction,
    tag_structure_data: &crate::generators::tag_structure::TagStructureData,
) {
    use crate::schemas::tag_kit::{TagKit, SubDirectoryInfo};
    
    // Convert Main table tags with subdirectories into tag kit entries
    for tag in &tag_structure_data.tags {
        if tag.has_subdirectory && tag.subdirectory_table.is_some() {
            let subdirectory_table = tag.subdirectory_table.as_ref().unwrap();
            
            // Create a tag kit entry for this Main table tag
            let tag_kit = TagKit {
                tag_id: tag.tag_id_decimal.to_string(),
                name: tag.name.clone(),
                format: "unknown".to_string(), // Main table tags don't have specific formats in tag structure
                groups: HashMap::new(),
                writable: if tag.writable { Some(serde_json::Value::Bool(true)) } else { None },
                notes: tag.description.clone(),
                print_conv_type: "None".to_string(),
                print_conv_data: None,
                value_conv: None,
                variant_id: None,
                condition: None,
                subdirectory: Some(SubDirectoryInfo {
                    tag_table: subdirectory_table.clone(),
                    validate: None,
                    has_validate_code: Some(false),
                    process_proc: None,
                    has_process_proc_code: Some(false),
                    base: None,
                    byte_order: None,
                    is_binary_data: Some(true), // Assume binary data for Canon subdirectories
                    extracted_table: None,
                }),
            };
            
            // Add to tag kits if not already present
            let tag_id_str = tag.tag_id_decimal.to_string();
            if !tag_kit_data.tag_kits.iter().any(|tk| tk.tag_id == tag_id_str && tk.subdirectory.is_some()) {
                tag_kit_data.tag_kits.push(tag_kit);
            }
        }
    }
}

/// Process tag_kit configuration
fn process_tag_kit_config(
    config_dir: &Path,
    module_name: &str,
    module_output_dir: &Path,
) -> Result<Vec<String>> {
    config_processor::process_config_if_exists(
        config_dir,
        "tag_kit.json",
        |_config_json| {
            // Look for extracted tag kit JSON file
            let tag_kit_file = path_utils::construct_extract_filename(module_name, "tag_kit.json");
            let tag_kit_path = path_utils::get_extract_dir("tag_kits").join(&tag_kit_file);
            
            if tag_kit_path.exists() {
                let mut tag_kit_data: crate::schemas::tag_kit::TagKitExtraction = 
                    config_processor::load_extracted_json(&tag_kit_path)?;
                
                // Also load Main table tag structure to get tags with subdirectories
                let tag_structure_file = path_utils::construct_extract_filename(module_name, "tag_structure__main.json");
                let tag_structure_path = path_utils::get_extract_dir("tag_structures").join(&tag_structure_file);
                
                if tag_structure_path.exists() {
                    let tag_structure_data: crate::generators::tag_structure::TagStructureData = 
                        config_processor::load_extracted_json(&tag_structure_path)?;
                    
                    // Add Main table tags that have subdirectories to the tag kit
                    integrate_main_table_tags(&mut tag_kit_data, &tag_structure_data);
                }
                
                // Generate modular tag kit files in module directory
                Ok(generate_tag_kit_module(&tag_kit_data, module_output_dir)?)
            } else {
                Ok(Vec::new())
            }
        }
    )
}

/// Process inline_printconv configuration
fn process_inline_printconv_config(
    config_dir: &Path,
    _module_name: &str,
    module_output_dir: &Path,
) -> Result<Vec<String>> {
    config_processor::process_config_if_exists(
        config_dir,
        "inline_printconv.json",
        |config_json| {
            config_processor::process_tables_config(config_json, |table_config| {
                let table_name = table_config["table_name"].as_str().unwrap_or("");
                
                // Look for the corresponding extracted inline printconv JSON file
                let inline_file_name = format!("inline_printconv__{}.json", 
                    path_utils::convert_table_name_to_snake_case(table_name)
                );
                let inline_file_path = path_utils::get_extract_dir("inline_printconv").join(&inline_file_name);
                
                if inline_file_path.exists() {
                    let inline_data: inline_printconv::InlinePrintConvData = 
                        config_processor::load_extracted_json(&inline_file_path)?;
                    
                    // Generate file for this table's inline PrintConv entries
                    Ok(Some(generate_inline_printconv_file(
                        &inline_data, 
                        table_name,
                        module_output_dir
                    )?))
                } else {
                    Ok(None)
                }
            })
        }
    )
}

/// Process regex_patterns configuration
fn process_regex_patterns_config(
    config_dir: &Path,
    module_name: &str,
    module_output_dir: &Path,
) -> Result<Vec<String>> {
    config_processor::process_config_if_exists(
        config_dir,
        "regex_patterns.json",
        |config_json| {
            config_processor::process_tables_config(config_json, |table_config| {
                // Look for extracted regex patterns file using standardized naming
                let module_base = path_utils::get_module_base(module_name);
                let regex_file = format!("{module_base}__regex_patterns.json").to_lowercase();
                let regex_path = path_utils::get_extract_dir("file_types").join(&regex_file);
                
                if regex_path.exists() {
                    // Generate regex_patterns.rs file
                    Ok(Some(generate_regex_patterns_file(
                        &regex_path,
                        table_config,
                        module_output_dir
                    )?))
                } else {
                    Ok(None)
                }
            })
        }
    )
}

/// Process runtime_table configuration
fn process_runtime_table_config(
    config_dir: &Path,
    module_name: &str,
    module_output_dir: &Path,
) -> Result<Vec<String>> {
    config_processor::process_config_if_exists(
        config_dir,
        "runtime_table.json",
        |config_json| {
            config_processor::process_tables_config(config_json, |table_config| {
                if let Some(table_name) = table_config["table_name"].as_str() {
                    let clean_table_name = table_name.trim_start_matches('%');
                    
                    // Look for the corresponding extracted runtime table JSON file
                    let module_base = path_utils::get_module_base(module_name);
                    let runtime_table_file = format!("{}__runtime_table__{}.json", 
                                                   module_base.to_lowercase(), 
                                                   clean_table_name.to_lowercase());
                    let runtime_table_path = path_utils::get_extract_dir("runtime_tables").join(&runtime_table_file);
                    
                    if runtime_table_path.exists() {
                        let runtime_table_data: crate::schemas::input::RuntimeTablesData = 
                            config_processor::load_extracted_json(&runtime_table_path)?;
                        
                        // Generate file for this RuntimeTable
                        Ok(Some(generate_runtime_table_file(
                            &runtime_table_data,
                            module_output_dir,
                            table_config
                        )?))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            })
        }
    )
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
            if path.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
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
            if file_stem == pattern || file_stem.ends_with(&format!("_{pattern}")) {
                // Special case: detect Olympus naming conflicts
                if file_stem == "tag_structure" && has_conflicting_olympus_structs {
                    warn!("  ⚠ Including tag_structure.rs but detected naming conflict with equipment_tag_structure.rs");
                    warn!("      Both define OlympusDataType enum - this may cause compilation errors");
                }
                
                additional_files.push(file_stem.clone());
                debug!("  ✓ Detected standalone generated file: {}.rs", file_stem);
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
                        debug!("  ✓ Detected generated subdirectory: {}/", name);
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
    
    let filename = format!("{function_name}_runtime.rs");
    let output_path = output_dir.join(&filename);
    
    fs::write(&output_path, runtime_code)?;
    debug!("  📋 Generated runtime table file: {}", filename);
    
    Ok(filename.replace(".rs", ""))
}

/// Generate modular tag kit files within the module directory
/// This creates a tag_kit subdirectory with modular category files
fn generate_tag_kit_module(
    tag_kit_data: &crate::schemas::tag_kit::TagKitExtraction,
    module_output_dir: &Path,
) -> Result<Vec<String>> {
    // Get the module name from the output directory
    let module_name = module_output_dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    
    // Use the enhanced tag_kit_modular generator with subdirectory support
    // Pass the module_output_dir directly so tag_kit is created inside it
    crate::generators::tag_kit_modular::generate_modular_tag_kit(
        tag_kit_data,
        module_output_dir.to_str().unwrap(),
        module_name,
    )?;
    
    // Return "tag_kit" as the generated module name (for parent mod.rs)
    Ok(vec!["tag_kit".to_string()])
}

/// Generate code for a single tag kit category module
/// This is adapted from the existing tag_kit_modular.rs code
#[allow(dead_code)]
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
    code.push_str("#![allow(unused_mut)]\n");
    code.push_str("#![allow(dead_code)]\n");
    code.push_str("#![allow(unused_variables)]\n\n");
    
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
                    
                    code.push_str(&format!("static {const_name}: LazyLock<HashMap<String, &'static str>> = LazyLock::new(|| {{\n"));
                    code.push_str("    let mut map = HashMap::new();\n");
                    
                    for (key, value) in data_obj {
                        generate_print_conv_entry(&mut code, key, value);
                    }
                    
                    code.push_str("    map\n");
                    code.push_str("});\n\n");
                }
            }
        }
    }
    
    // Generate tag definitions function
    code.push_str(&format!("/// Get tag definitions for {category_name} category\n"));
    code.push_str(&format!("pub fn get_{category_name}_tags() -> Vec<(u32, TagKitDef)> {{\n"));
    code.push_str("    vec![\n");
    
    // Reset print conv counter for this category
    let mut category_print_conv_index = *print_conv_counter - local_print_conv_count;
    
    for tag_kit in tag_kits {
        let tag_id = tag_kit.tag_id.parse::<u32>().unwrap_or(0);
        
        code.push_str(&format!("        ({tag_id}, TagKitDef {{\n"));
        code.push_str(&format!("            id: {tag_id},\n"));
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
                    code.push_str(&format!("            print_conv: PrintConvType::Simple(&PRINT_CONV_{category_print_conv_index}),\n"));
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
#[allow(dead_code)]
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
        code.push_str(&format!("pub mod {module};\n"));
    }
    code.push('\n');
    
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
        code.push_str(&format!("    // {module} tags\n"));
        code.push_str(&format!("    for (id, tag_def) in {module}::get_{module}_tags() {{\n"));
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
    
    code.push_str(&format!("/// {description}\n"));
    code.push_str(&format!("pub static {constant_name}: LazyLock<HashMap<{key_type}, bool>> = LazyLock::new(|| {{\n"));
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
    code.push_str(&format!("/// Check if key exists in {description}\n"));
    code.push_str(&format!("pub fn {function_name}(key: &{key_type}) -> bool {{\n"));
    code.push_str(&format!("    {constant_name}.contains_key(key)\n"));
    code.push_str("}\n");
    
    // Create descriptive filename from hash name
    let file_name = path_utils::hash_name_to_filename(hash_name);
    let file_path = output_dir.join(format!("{file_name}.rs"));
    
    let mut content = String::new();
    content.push_str(&format!("//! {description}\n"));
    content.push_str("//! \n//! Auto-generated from ExifTool source\n");
    content.push_str("//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen\n\n");
    content.push_str(&code);
    
    fs::write(&file_path, content)?;
    debug!("  ✓ Generated {}", file_path.display());
    
    Ok(file_name)
}

/// Generate regex_patterns.rs file from extracted regex patterns JSON
fn generate_regex_patterns_file(
    regex_file_path: &Path,
    table_config: &serde_json::Value,
    output_dir: &Path,
) -> Result<String> {
    let regex_content = fs::read_to_string(regex_file_path)?;
    let regex_data: serde_json::Value = serde_json::from_str(&regex_content)
        .with_context(|| format!("Failed to parse regex file: {}", regex_file_path.display()))?;
    
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
    code.push_str(&format!("/// {description}\n"));
    code.push_str(&format!("pub static {constant_name}: LazyLock<HashMap<&'static str, &'static [u8]>> = LazyLock::new(|| {{\n"));
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
                code.push_str(&format!("    map.insert(\"{file_type}\", &{escaped_pattern} as &[u8]);\n"
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
    
    debug!("  ✓ Generated {}", file_path.display());
    
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
                    let hex_str = format!("{hex1}{hex2}");
                    if let Ok(byte_val) = u8::from_str_radix(&hex_str, 16) {
                        result.push_str(&format!("0x{byte_val:02x}u8"));
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