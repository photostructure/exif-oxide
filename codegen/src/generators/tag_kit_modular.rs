//! Modular tag kit generator that splits output into smaller files
//!
//! This generator creates tag definitions split by category to keep file sizes manageable

use anyhow::Result;
use std::fs;
use std::collections::HashMap;
use crate::common::escape_string;
use crate::schemas::tag_kit::{TagKitExtraction, TagKit, ExtractedTable};
use super::tag_kit_split::{split_tag_kits, TagCategory};
use crate::conv_registry::{lookup_printconv, lookup_valueconv};
use serde::Deserialize;

/// Shared table data loaded from shared_tables.json
#[derive(Debug, Deserialize)]
struct SharedTablesData {
    #[allow(dead_code)]
    metadata: serde_json::Value,
    #[allow(dead_code)]
    tables: HashMap<String, SharedTable>,
}

#[derive(Debug, Deserialize)]
struct SharedTable {
    #[allow(dead_code)]
    path: String,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    table_type: String,
    #[allow(dead_code)]
    tags: HashMap<String, serde_json::Value>,
}

/// Load shared tables from JSON file
fn load_shared_tables() -> Result<SharedTablesData> {
    let shared_tables_path = "data/shared_tables.json";
    if std::path::Path::new(shared_tables_path).exists() {
        let contents = fs::read_to_string(shared_tables_path)?;
        Ok(serde_json::from_str(&contents)?)
    } else {
        // Return empty data if file doesn't exist
        Ok(SharedTablesData {
            metadata: serde_json::Value::Null,
            tables: HashMap::new(),
        })
    }
}

/// Check if a table name refers to a cross-module reference
fn is_cross_module_reference(table_name: &str, current_module: &str) -> bool {
    if let Some(module_part) = table_name.strip_prefix("Image::ExifTool::") {
        if let Some(referenced_module) = module_part.split("::").next() {
            // Handle _pm suffix in module names
            let normalized_current = current_module.trim_end_matches("_pm");
            return referenced_module != normalized_current;
        }
    }
    false
}


/// Generate the apply_print_conv function with direct function calls
fn generate_print_conv_function(
    code: &mut String,
    extraction: &TagKitExtraction,
    module_name: &str,
    const_name: &str,
) -> Result<()> {
    code.push_str("/// Apply PrintConv for a tag from this module\n");
    code.push_str("pub fn apply_print_conv(\n");
    code.push_str("    tag_id: u32,\n");
    code.push_str("    value: &TagValue,\n");
    code.push_str("    _evaluator: &mut ExpressionEvaluator,\n");
    code.push_str("    _errors: &mut Vec<String>,\n");
    code.push_str("    warnings: &mut Vec<String>,\n");
    code.push_str(") -> TagValue {\n");
    
    // First, collect all tags that need PrintConv
    // Use HashMap to deduplicate by tag_id (keep first occurrence)
    let mut tag_convs_map: std::collections::HashMap<u32, (String, String)> = std::collections::HashMap::new();
    
    for tag_kit in &extraction.tag_kits {
        let tag_id = tag_kit.tag_id.parse::<u32>().unwrap_or(0);
        
        // Skip if we already have a PrintConv for this tag_id
        if tag_convs_map.contains_key(&tag_id) {
            continue;
        }
        
        match tag_kit.print_conv_type.as_str() {
            "Expression" => {
                if let Some(expr_data) = &tag_kit.print_conv_data {
                    if let Some(expr_str) = expr_data.as_str() {
                        if let Some((module_path, func_name)) = lookup_printconv(expr_str, module_name) {
                            tag_convs_map.insert(tag_id, (module_path.to_string(), func_name.to_string()));
                        }
                    }
                }
            }
            "Manual" => {
                if let Some(func_data) = &tag_kit.print_conv_data {
                    if let Some(func_str) = func_data.as_str() {
                        if let Some((module_path, func_name)) = lookup_printconv(func_str, module_name) {
                            tag_convs_map.insert(tag_id, (module_path.to_string(), func_name.to_string()));
                        }
                    }
                }
            }
            _ => {}
        }
    }
    
    // Convert to sorted Vec for deterministic output
    let mut tag_convs: Vec<(u32, String, String)> = tag_convs_map
        .into_iter()
        .map(|(id, (module_path, func_name))| (id, module_path, func_name))
        .collect();
    
    if tag_convs.is_empty() {
        // No PrintConv functions - use shared fallback handling
        code.push_str(&format!("    if let Some(tag_kit) = {const_name}.get(&tag_id) {{\n"));
        code.push_str("        crate::implementations::generic::apply_fallback_print_conv(tag_id, value, crate::to_print_conv_ref!(&tag_kit.print_conv))\n");
        code.push_str("    } else {\n");
        code.push_str("        value.clone()\n");
        code.push_str("    }\n");
    } else {
        // Generate optimized match with direct function calls
        code.push_str("    match tag_id {\n");
        
        // Sort by tag_id for deterministic output
        tag_convs.sort_by_key(|(id, _, _)| *id);
        
        for (tag_id, module_path, func_name) in tag_convs {
            code.push_str(&format!("        {tag_id} => {}::{}(value),\n", module_path, func_name));
        }
        
        code.push_str("        _ => {\n");
        code.push_str(&format!("            // Fall back to shared handling\n"));
        code.push_str(&format!("            if let Some(tag_kit) = {const_name}.get(&tag_id) {{\n"));
        code.push_str("                crate::implementations::generic::apply_fallback_print_conv(tag_id, value, crate::to_print_conv_ref!(&tag_kit.print_conv))\n");
        code.push_str("            } else {\n");
        code.push_str("                value.clone()\n");
        code.push_str("            }\n");
        code.push_str("        }\n");
        code.push_str("    }\n");
    }
    
    code.push_str("}\n\n");
    
    Ok(())
}

/// Generate modular tag kit code from extracted data
pub fn generate_modular_tag_kit(
    extraction: &TagKitExtraction,
    output_dir: &str,
    module_name: &str,
) -> Result<()> {
    // Create output directory for tag kit modules 
    // Create tag_kit subdirectory inside the output directory
    let tag_kit_dir = format!("{output_dir}/tag_kit");
    fs::create_dir_all(&tag_kit_dir)?;
    
    // Load shared tables for cross-module references
    let shared_tables = load_shared_tables()?;
    
    // Split tag kits by category
    let categories = split_tag_kits(&extraction.tag_kits);
    
    // Keep track of all generated modules
    let mut generated_modules = Vec::new();
    let mut total_print_conv_count = 0;
    
    // Collect all subdirectory information
    let subdirectory_info = collect_subdirectory_info(&extraction.tag_kits);
    
    // Generate a module for each category with tags (sorted for deterministic PRINT_CONV numbering)
    let mut sorted_categories: Vec<(&TagCategory, &Vec<&TagKit>)> = categories.iter().collect();
    sorted_categories.sort_by_key(|(category, _)| category.module_name());
    
    for (category, tag_kits) in sorted_categories {
        if tag_kits.is_empty() {
            continue;
        }
        
        let module_name_cat = category.module_name();
        let (module_code, print_conv_count) = generate_category_module(
            module_name_cat,
            tag_kits,
            &extraction.source,
            &mut total_print_conv_count,
            module_name,
            &shared_tables,
        )?;
        
        // Write category module
        let file_path = format!("{tag_kit_dir}/{module_name_cat}.rs");
        fs::write(&file_path, module_code)?;
        
        generated_modules.push(module_name_cat);
        
        tracing::debug!("  ✓ Generated {} with {} tags, {} PrintConv tables",
            module_name_cat,
            tag_kits.len(),
            print_conv_count
        );
    }
    
    // Generate mod.rs that combines all modules and subdirectory processors
    let mod_code = generate_mod_file(&generated_modules, module_name, extraction, &subdirectory_info, &shared_tables)?;
    fs::write(format!("{tag_kit_dir}/mod.rs"), mod_code)?;
    
    // Summary
    tracing::debug!("  ✓ Generated modular tag kit for {} with {} tags split into {} modules", 
        module_name,
        extraction.tag_kits.len(),
        generated_modules.len()
    );
    
    Ok(())
}

/// Generate code for a single category module
fn generate_category_module(
    category_name: &str,
    tag_kits: &[&TagKit],
    source: &crate::schemas::tag_kit::SourceInfo,
    print_conv_counter: &mut usize,
    _current_module: &str,
    _shared_tables: &SharedTablesData,
) -> Result<(String, usize)> {
    let mut code = String::new();
    
    // Header with warning suppression at the top
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
    code.push_str("use super::{TagKitDef, PrintConvType, SubDirectoryType};\n");
    // Import subdirectory processor functions from parent module
    code.push_str("use super::*;\n\n");
    
    // Generate PrintConv lookup tables for this category
    let mut local_print_conv_count = 0;
    for tag_kit in tag_kits {
        tracing::debug!("Processing tag '{}' with print_conv_type='{}'", tag_kit.name, tag_kit.print_conv_type);
        if tag_kit.print_conv_type == "Simple" {
            tracing::debug!("Found Simple PrintConv for tag '{}'", tag_kit.name);
            if let Some(print_conv_data) = &tag_kit.print_conv_data {
                tracing::debug!("PrintConv data exists for tag '{}'", tag_kit.name);
                if let Some(data_obj) = print_conv_data.as_object() {
                    tracing::debug!("PrintConv data is an object with {} entries", data_obj.len());
                    let const_name = format!("PRINT_CONV_{}", *print_conv_counter);
                    *print_conv_counter += 1;
                    local_print_conv_count += 1;
                    
                    code.push_str(&format!("static {const_name}: LazyLock<HashMap<String, &'static str>> = LazyLock::new(|| {{\n"));
                    code.push_str("    let mut map = HashMap::new();\n");
                    
                    // Sort keys for deterministic output
                    let mut sorted_keys: Vec<&String> = data_obj.keys().collect();
                    sorted_keys.sort();
                    
                    for key in sorted_keys {
                        let value = &data_obj[key];
                        tracing::debug!("Processing PrintConv entry: key='{}', value={:?}", key, value);
                        crate::generators::lookup_tables::generate_print_conv_entry(&mut code, key, value);
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
        code.push_str(&format!("            name: \"{}\",\n", escape_string(&tag_kit.name)));
        code.push_str(&format!("            format: \"{}\",\n", escape_string(&tag_kit.format)));
        
        // Groups (currently empty in extraction)
        code.push_str("            groups: HashMap::new(),\n");
        
        // Writable
        code.push_str(&format!("            writable: {},\n", 
            if tag_kit.writable.is_some() { "true" } else { "false" }));
        
        // Notes
        if let Some(notes) = &tag_kit.notes {
            let trimmed_notes = notes.trim();
            code.push_str(&format!("            notes: Some(\"{}\"),\n", escape_string(trimmed_notes)));
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
                            escape_string(expr_str)));
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
                            escape_string(name_str)));
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
            code.push_str(&format!("            value_conv: Some(\"{}\"),\n", escape_string(value_conv)));
        } else {
            code.push_str("            value_conv: None,\n");
        }
        
        // Add subdirectory field
        if tag_kit.subdirectory.is_some() {
            code.push_str(&format!("            subdirectory: Some(SubDirectoryType::Binary {{ processor: process_tag_{tag_id:#x}_subdirectory }}),\n"));
        } else {
            code.push_str("            subdirectory: None,\n");
        }
        
        code.push_str("        }),\n");
    }
    
    code.push_str("    ]\n");
    code.push_str("}\n");
    
    Ok((code, local_print_conv_count))
}

/// Generate the mod.rs file that combines all category modules
fn generate_mod_file(
    modules: &[&str], 
    module_name: &str,
    extraction: &TagKitExtraction,
    subdirectory_info: &HashMap<u32, SubDirectoryCollection>,
    shared_tables: &SharedTablesData,
) -> Result<String> {
    let mut code = String::new();
    
    // Header
    code.push_str(&format!("//! Modular tag kits with embedded PrintConv for {module_name}\n"));
    code.push_str("//!\n");
    code.push_str("//! This file is automatically generated by codegen.\n");
    code.push_str("//! DO NOT EDIT MANUALLY - changes will be overwritten.\n");
    code.push_str("//!\n");
    code.push_str(&format!("//! Generated from: {} table: {}\n", extraction.source.module, extraction.source.table));
    code.push_str("//!\n");
    // NOTE: Do NOT add extraction timestamps here - they create spurious git diffs
    // that make it impossible to track real changes to generated code
    
    code.push_str("#![allow(unused_imports)]\n");
    code.push_str("#![allow(unused_mut)]\n");
    code.push_str("#![allow(dead_code)]\n");
    code.push_str("#![allow(unused_variables)]\n\n");
    
    // Module declarations (sorted for deterministic output)
    let mut sorted_modules: Vec<&str> = modules.iter().map(|s| s.as_ref()).collect();
    sorted_modules.sort();
    
    for module in sorted_modules {
        code.push_str(&format!("pub mod {module};\n"));
    }
    code.push('\n');
    
    // Common imports
    code.push_str("use std::collections::HashMap;\n");
    code.push_str("use std::sync::LazyLock;\n");
    code.push_str("use crate::types::{TagValue, Result};\n");
    code.push_str("use crate::tiff_types::ByteOrder;\n");
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
    code.push_str("    pub subdirectory: Option<SubDirectoryType>,\n");
    code.push_str("}\n\n");
    
    // PrintConv type enum
    code.push_str("#[derive(Debug, Clone)]\n");
    code.push_str("pub enum PrintConvType {\n");
    code.push_str("    None,\n");
    code.push_str("    Simple(&'static HashMap<String, &'static str>),\n");
    code.push_str("    Expression(&'static str),\n");
    code.push_str("    Manual(&'static str),\n");
    code.push_str("}\n\n");
    
    // Type alias to fix clippy::type_complexity warning
    code.push_str("/// Type alias for subdirectory processor function\n");
    code.push_str("pub type SubDirectoryProcessor = fn(&[u8], ByteOrder) -> Result<Vec<(String, TagValue)>>;\n\n");
    
    // SubDirectory type enum
    code.push_str("#[derive(Debug, Clone)]\n");
    code.push_str("pub enum SubDirectoryType {\n");
    code.push_str("    Binary {\n");
    code.push_str("        processor: SubDirectoryProcessor,\n");
    code.push_str("    },\n");
    code.push_str("}\n\n");
    
    // Combined tag map
    let const_name = format!("{}_TAG_KITS", module_name.to_uppercase());
    code.push_str(&format!("/// All tag kits for {module_name}\n"));
    code.push_str(&format!("pub static {const_name}: LazyLock<HashMap<u32, TagKitDef>> = LazyLock::new(|| {{\n"));
    code.push_str("    let mut map = HashMap::new();\n");
    code.push_str("    \n");
    
    // Add tags from each module (sorted for deterministic output)
    let mut sorted_modules: Vec<&str> = modules.iter().map(|s| s.as_ref()).collect();
    sorted_modules.sort();
    
    for module in sorted_modules {
        code.push_str(&format!("    // {module} tags\n"));
        code.push_str(&format!("    for (id, tag_def) in {module}::get_{module}_tags() {{\n"));
        code.push_str("        map.insert(id, tag_def);\n");
        code.push_str("    }\n");
        code.push_str("    \n");
    }
    
    code.push_str("    map\n");
    code.push_str("});\n\n");
    
    // Generate binary data helper functions
    code.push_str("// Helper functions for reading binary data\n");
    
    // Add model matching helper
    code.push_str("fn model_matches(model: &str, pattern: &str) -> bool {\n");
    code.push_str("    // ExifTool regexes are already in a compatible format\n");
    code.push_str("    // Just need to ensure proper escaping was preserved\n");
    code.push_str("    \n");
    code.push_str("    match regex::Regex::new(pattern) {\n");
    code.push_str("        Ok(re) => re.is_match(model),\n");
    code.push_str("        Err(e) => {\n");
    code.push_str("            tracing::warn!(\"Failed to compile model regex '{}': {}\", pattern, e);\n");
    code.push_str("            false\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");
    
    // Add format matching helper
    code.push_str("fn format_matches(format: &str, pattern: &str) -> bool {\n");
    code.push_str("    if let Ok(re) = regex::Regex::new(pattern) {\n");
    code.push_str("        re.is_match(format)\n");
    code.push_str("    } else {\n");
    code.push_str("        tracing::warn!(\"Failed to compile format regex: {}\", pattern);\n");
    code.push_str("        false\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");
    
    code.push_str("fn read_int16s_array(data: &[u8], byte_order: ByteOrder, count: usize) -> Result<Vec<i16>> {\n");
    code.push_str("    if data.len() < count * 2 {\n");
    code.push_str("        return Err(crate::types::ExifError::ParseError(\"Insufficient data for int16s array\".to_string()));\n");
    code.push_str("    }\n");
    code.push_str("    let mut values = Vec::with_capacity(count);\n");
    code.push_str("    for i in 0..count {\n");
    code.push_str("        let offset = i * 2;\n");
    code.push_str("        let value = match byte_order {\n");
    code.push_str("            ByteOrder::LittleEndian => i16::from_le_bytes([data[offset], data[offset + 1]]),\n");
    code.push_str("            ByteOrder::BigEndian => i16::from_be_bytes([data[offset], data[offset + 1]]),\n");
    code.push_str("        };\n");
    code.push_str("        values.push(value);\n");
    code.push_str("    }\n");
    code.push_str("    Ok(values)\n");
    code.push_str("}\n\n");
    
    code.push_str("fn read_int16u_array(data: &[u8], byte_order: ByteOrder, count: usize) -> Result<Vec<u16>> {\n");
    code.push_str("    if data.len() < count * 2 {\n");
    code.push_str("        return Err(crate::types::ExifError::ParseError(\"Insufficient data for int16u array\".to_string()));\n");
    code.push_str("    }\n");
    code.push_str("    let mut values = Vec::with_capacity(count);\n");
    code.push_str("    for i in 0..count {\n");
    code.push_str("        let offset = i * 2;\n");
    code.push_str("        let value = match byte_order {\n");
    code.push_str("            ByteOrder::LittleEndian => u16::from_le_bytes([data[offset], data[offset + 1]]),\n");
    code.push_str("            ByteOrder::BigEndian => u16::from_be_bytes([data[offset], data[offset + 1]]),\n");
    code.push_str("        };\n");
    code.push_str("        values.push(value);\n");
    code.push_str("    }\n");
    code.push_str("    Ok(values)\n");
    code.push_str("}\n\n");
    
    code.push_str("fn read_int16s(data: &[u8], byte_order: ByteOrder) -> Result<i16> {\n");
    code.push_str("    if data.len() < 2 {\n");
    code.push_str("        return Err(crate::types::ExifError::ParseError(\"Insufficient data for int16s\".to_string()));\n");
    code.push_str("    }\n");
    code.push_str("    Ok(match byte_order {\n");
    code.push_str("        ByteOrder::LittleEndian => i16::from_le_bytes([data[0], data[1]]),\n");
    code.push_str("        ByteOrder::BigEndian => i16::from_be_bytes([data[0], data[1]]),\n");
    code.push_str("    })\n");
    code.push_str("}\n\n");
    
    // Generate subdirectory processing functions
    if !subdirectory_info.is_empty() {
        code.push_str("// Subdirectory processing functions\n");
        
        // Generate binary data parsers for each unique subdirectory table
        let mut generated_tables = std::collections::BTreeSet::new();
        let mut referenced_tables = std::collections::BTreeSet::new();
        
        // Sort subdirectory collections by tag_id for deterministic output
        let mut sorted_collections: Vec<(&u32, &SubDirectoryCollection)> = subdirectory_info.iter().collect();
        sorted_collections.sort_by_key(|(tag_id, _)| *tag_id);
        
        for (_, collection) in sorted_collections {
            for variant in &collection.variants {
                // Track all referenced tables
                if !variant.table_name.is_empty() && variant.table_name != "Unknown" && !is_cross_module_reference(&variant.table_name, module_name) {
                    let table_fn_name = variant.table_name
                        .replace("Image::ExifTool::", "")
                        .replace("::", "_")
                        .to_lowercase();
                    referenced_tables.insert(table_fn_name.clone());
                    
                    if variant.is_binary_data {
                        if let Some(extracted_table) = &variant.extracted_table {
                            if generated_tables.insert(table_fn_name.clone()) {
                                generate_binary_parser(&mut code, &table_fn_name, extracted_table)?;
                            }
                        }
                    }
                }
            }
        }
        
        // Generate stub functions for referenced but not generated tables
        let missing_tables: Vec<_> = referenced_tables
            .difference(&generated_tables)
            .cloned()
            .collect();
        
        if !missing_tables.is_empty() {
            code.push_str("\n// Stub functions for tables not extracted by tag kit\n");
            for table_name in missing_tables {
                code.push_str(&format!("fn process_{table_name}(data: &[u8], _byte_order: ByteOrder) -> Result<Vec<(String, TagValue)>> {{\n"));
                code.push_str("    // TODO: Implement when this table is extracted\n");
                code.push_str("    tracing::debug!(\"Stub function called for {}\", data.len());\n");
                code.push_str("    Ok(vec![])\n");
                code.push_str("}\n\n");
            }
        }
        
        // Generate conditional dispatch functions for tags with subdirectories
        // Sort by tag_id for deterministic output
        let mut sorted_subdirs: Vec<(&u32, &SubDirectoryCollection)> = subdirectory_info.iter().collect();
        sorted_subdirs.sort_by_key(|(tag_id, _)| *tag_id);
        
        for (tag_id, collection) in sorted_subdirs {
            generate_subdirectory_dispatcher(&mut code, *tag_id, collection, module_name, shared_tables)?;
        }
    }
    
    // Generate apply_print_conv function with direct function calls
    generate_print_conv_function(&mut code, extraction, module_name, &const_name)?;
    
    // Add subdirectory processing functions
    code.push_str("/// Check if a tag has subdirectory processing\n");
    code.push_str("pub fn has_subdirectory(tag_id: u32) -> bool {\n");
    code.push_str(&format!("    if let Some(tag_kit) = {const_name}.get(&tag_id) {{\n"));
    code.push_str("        tag_kit.subdirectory.is_some()\n");
    code.push_str("    } else {\n");
    code.push_str("        false\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");
    
    code.push_str("/// Process subdirectory tags and return multiple extracted tags\n");
    code.push_str("pub fn process_subdirectory(\n");
    code.push_str("    tag_id: u32,\n");
    code.push_str("    value: &TagValue,\n");
    code.push_str("    byte_order: ByteOrder,\n");
    code.push_str(") -> Result<HashMap<String, TagValue>> {\n");
    code.push_str("    use tracing::debug;\n");
    code.push_str("    let mut result = HashMap::new();\n");
    code.push_str("    \n");
    code.push_str("    debug!(\"process_subdirectory called for tag_id: 0x{:04x}\", tag_id);\n");
    code.push_str("    \n");
    code.push_str(&format!("    if let Some(tag_kit) = {const_name}.get(&tag_id) {{\n"));
    code.push_str("        if let Some(SubDirectoryType::Binary { processor }) = &tag_kit.subdirectory {\n");
    code.push_str("            debug!(\"Found subdirectory processor for tag_id: 0x{:04x}\", tag_id);\n");
    code.push_str("            let bytes = match value {\n");
    code.push_str("                TagValue::U16Array(arr) => {\n");
    code.push_str("                    debug!(\"Converting U16Array with {} elements to bytes\", arr.len());\n");
    code.push_str("                    // Convert U16 array to bytes based on byte order\n");
    code.push_str("                    let mut bytes = Vec::with_capacity(arr.len() * 2);\n");
    code.push_str("                    for val in arr {\n");
    code.push_str("                        match byte_order {\n");
    code.push_str("                            ByteOrder::LittleEndian => bytes.extend_from_slice(&val.to_le_bytes()),\n");
    code.push_str("                            ByteOrder::BigEndian => bytes.extend_from_slice(&val.to_be_bytes()),\n");
    code.push_str("                        }\n");
    code.push_str("                    }\n");
    code.push_str("                    bytes\n");
    code.push_str("                }\n");
    code.push_str("                TagValue::U8Array(arr) => arr.clone(),\n");
    code.push_str("                _ => return Ok(result), // Not array data\n");
    code.push_str("            };\n");
    code.push_str("            \n");
    code.push_str("            debug!(\"Calling processor with {} bytes\", bytes.len());\n");
    code.push_str("            // Process subdirectory and collect all extracted tags\n");
    code.push_str("            match processor(&bytes, byte_order) {\n");
    code.push_str("                Ok(extracted_tags) => {\n");
    code.push_str("                    debug!(\"Processor returned {} tags\", extracted_tags.len());\n");
    code.push_str("                    for (name, value) in extracted_tags {\n");
    code.push_str("                        result.insert(name, value);\n");
    code.push_str("                    }\n");
    code.push_str("                }\n");
    code.push_str("                Err(e) => {\n");
    code.push_str("                    debug!(\"Processor error: {:?}\", e);\n");
    code.push_str("                }\n");
    code.push_str("            }\n");
    code.push_str("        } else {\n");
    code.push_str("            debug!(\"No subdirectory processor found for tag_id: 0x{:04x}\", tag_id);\n");
    code.push_str("        }\n");
    code.push_str("    } else {\n");
    code.push_str("        debug!(\"Tag not found in TAG_KITS: 0x{:04x}\", tag_id);\n");
    code.push_str("    }\n");
    code.push_str("    \n");
    code.push_str("    Ok(result)\n");
    code.push_str("}\n");
    
    Ok(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_count_conditions() {
        // Simple count
        assert_eq!(parse_count_conditions("$count == 582"), vec![582]);
        
        // OR with 'or'
        assert_eq!(parse_count_conditions("$count == 1273 or $count == 1275"), vec![1273, 1275]);
        
        // OR with '||'
        assert_eq!(parse_count_conditions("$count == 1536 || $count == 2048"), vec![1536, 2048]);
        
        // Multi-line
        let multi = "$count == 692  or $count == 674  or $count == 702 or
                $count == 1227 or $count == 1250";
        assert_eq!(parse_count_conditions(multi), vec![692, 674, 702, 1227, 1250]);
    }
    
    #[test]
    fn test_parse_model_patterns() {
        // Simple model match
        let pattern = parse_model_pattern("$$self{Model} =~ /EOS 5D/").unwrap();
        assert_eq!(pattern.regex, "EOS 5D");
        assert!(!pattern.negated);
        
        // Negated match
        let pattern = parse_model_pattern("$$self{Model} !~ /EOS/").unwrap();
        assert_eq!(pattern.regex, "EOS");
        assert!(pattern.negated);
        
        // With word boundaries
        let pattern = parse_model_pattern("$$self{Model} =~ /\\b(450D|REBEL XSi|Kiss X2)\\b/").unwrap();
        assert_eq!(pattern.regex, "\\b(450D|REBEL XSi|Kiss X2)\\b");
        assert!(!pattern.negated);
        
        // With end anchor
        let pattern = parse_model_pattern("$$self{Model} =~ /\\b1Ds? Mark III$/").unwrap();
        assert_eq!(pattern.regex, "\\b1Ds? Mark III$");
        assert!(!pattern.negated);
    }
    
    #[test]
    fn test_parse_format_patterns() {
        // Equality
        let pattern = parse_format_pattern("$format eq \"int32u\"").unwrap();
        assert_eq!(pattern.format, "int32u");
        assert_eq!(pattern.operator, ComparisonOp::Eq);
        assert!(pattern.additional_condition.is_none());
        
        // Inequality
        let pattern = parse_format_pattern("$format ne \"ifd\"").unwrap();
        assert_eq!(pattern.format, "ifd");
        assert_eq!(pattern.operator, ComparisonOp::Ne);
        
        // With additional condition
        let pattern = parse_format_pattern("$format eq \"int32u\" and ($count == 138 or $count == 148)").unwrap();
        assert_eq!(pattern.format, "int32u");
        assert_eq!(pattern.operator, ComparisonOp::Eq);
        assert_eq!(pattern.additional_condition.as_ref().unwrap(), "($count == 138 or $count == 148)");
        
        // Regex pattern
        let pattern = parse_format_pattern("$format =~ /^int16/").unwrap();
        assert_eq!(pattern.format, "^int16");
        assert_eq!(pattern.operator, ComparisonOp::Regex);
    }
    
    #[test]
    fn test_subdirectory_condition_classification() {
        // Count condition
        match parse_subdirectory_condition("$count == 582") {
            SubdirectoryCondition::Count(counts) => assert_eq!(counts, vec![582]),
            _ => panic!("Expected Count condition"),
        }
        
        // Model condition
        match parse_subdirectory_condition("$$self{Model} =~ /EOS 5D/") {
            SubdirectoryCondition::Model(pattern) => {
                assert_eq!(pattern.regex, "EOS 5D");
                assert!(!pattern.negated);
            }
            _ => panic!("Expected Model condition"),
        }
        
        // Format condition
        match parse_subdirectory_condition("$format eq \"int32u\"") {
            SubdirectoryCondition::Format(pattern) => {
                assert_eq!(pattern.format, "int32u");
                assert_eq!(pattern.operator, ComparisonOp::Eq);
            }
            _ => panic!("Expected Format condition"),
        }
        
        // Complex runtime condition
        match parse_subdirectory_condition("$$valPt =~ /^\\0/ and $$valPt !~ /^(\\0\\0\\0\\0|\\x00\\x40\\xdc\\x05)/") {
            SubdirectoryCondition::Runtime(s) => {
                assert!(s.contains("$$valPt"));
            }
            _ => panic!("Expected Runtime condition"),
        }
    }
}

/// Information about a subdirectory and all its variants
#[derive(Debug)]
struct SubDirectoryCollection {
    #[allow(dead_code)]
    tag_id: u32,
    #[allow(dead_code)]
    tag_name: String,
    variants: Vec<SubDirectoryVariant>,
}

#[derive(Debug, Clone)]
struct SubDirectoryVariant {
    #[allow(dead_code)]
    variant_id: String,
    condition: Option<String>,
    table_name: String,
    is_binary_data: bool,
    extracted_table: Option<ExtractedTable>,
}

/// Collect all subdirectory information from tag kits
fn collect_subdirectory_info(tag_kits: &[TagKit]) -> HashMap<u32, SubDirectoryCollection> {
    let mut subdirectory_map: HashMap<u32, SubDirectoryCollection> = HashMap::new();
    
    for tag_kit in tag_kits {
        if let Some(subdirectory) = &tag_kit.subdirectory {
            let tag_id = tag_kit.tag_id.parse::<u32>().unwrap_or(0);
            
            let variant = SubDirectoryVariant {
                variant_id: tag_kit.variant_id.clone().unwrap_or_else(|| format!("{tag_id}_default")),
                condition: tag_kit.condition.clone(),
                table_name: subdirectory.tag_table.clone(),
                is_binary_data: subdirectory.is_binary_data.unwrap_or(false),
                extracted_table: subdirectory.extracted_table.clone(),
            };
            
            subdirectory_map
                .entry(tag_id)
                .and_modify(|collection| collection.variants.push(variant.clone()))
                .or_insert_with(|| SubDirectoryCollection {
                    tag_id,
                    tag_name: tag_kit.name.clone(),
                    variants: vec![variant],
                });
        }
    }
    
    subdirectory_map
}

/// Generate a binary data parser function for a subdirectory table
/// 
/// CRITICAL: ExifTool Binary Data Offset Handling
/// ===============================================
/// ExifTool allows NEGATIVE tag offsets in binary data tables!
/// 
/// Reference: ExifTool.pm lines 9830-9836 (ProcessBinaryData function)
/// ```perl
/// # get relative offset of this entry
/// my $entry = int($index) * $increment + $varSize;
/// # allow negative indices to represent bytes from end
/// if ($entry < 0) {
///     $entry += $size;
///     next if $entry < 0;
/// }
/// ```
/// 
/// This means:
/// - Tag offsets can be LESS than FIRST_ENTRY (e.g., offset 0 with FIRST_ENTRY = 1)
/// - Negative offsets are interpreted as offsets from END of data block
/// - Example: offset -2 means "2 bytes before end of data"
/// 
/// FOOTGUN WARNING: Using unsigned arithmetic here will cause wraparound!
/// A calculation like (0 - 1) * 2 = -2 becomes 18446744073709551614 in usize.
/// This creates absurd comparisons like "if data.len() >= 18446744073709551615"
fn generate_binary_parser(code: &mut String, fn_name: &str, table: &ExtractedTable) -> Result<()> {
    code.push_str(&format!("fn process_{fn_name}(data: &[u8], byte_order: ByteOrder) -> Result<Vec<(String, TagValue)>> {{\n"));
    code.push_str("    let mut tags = Vec::new();\n");
    
    // Get the format size multiplier
    let format_size = match table.format.as_deref() {
        Some("int16s") | Some("int16u") => 2,
        Some("int32s") | Some("int32u") => 4,
        Some("int8s") | Some("int8u") => 1,
        _ => 2, // Default to int16
    };
    
    let first_entry = table.first_entry.unwrap_or(0);
    
    // Generate tag extraction code for each tag in the table
    for tag in &table.tags {
        // CRITICAL: Parse as i32 to handle negative offsets properly!
        let tag_offset = tag.tag_id.parse::<i32>().unwrap_or(0);
        // CRITICAL: Keep as signed to detect negative offsets
        let byte_offset = (tag_offset - first_entry) * format_size;
        
        code.push_str(&format!("    // {} at offset {}\n", tag.name, tag.tag_id));
        
        // Handle negative offsets (from end of data) like ExifTool does
        let offset_var_name = tag.name.to_lowercase().replace([' ', '-'], "_");
        if byte_offset < 0 {
            // For negative offsets, we need to generate runtime calculation
            code.push_str(&format!("    // {} uses negative offset {} (from end of data)\n", tag.name, byte_offset));
            code.push_str(&format!("    if data.len() as i32 + {byte_offset} < 0 {{\n"));
            code.push_str(&format!("        // Skipping {} - negative offset beyond data start\n", tag.name));
            code.push_str("        // (This is normal for some tables)\n");
            code.push_str("    } else {\n");
            code.push_str(&format!("        let {offset_var_name}_offset = (data.len() as i32 + {byte_offset}) as usize;\n"));
            // Set indent for the tag processing code
            code.push_str("        ");
        } else {
            // For positive offsets, use direct value
            let _byte_offset_usize = byte_offset as usize;
            // No extra indent needed
        }
        
        if let Some(format) = &tag.format {
            if format.ends_with(']') {
                // Array format like "int16s[4]"
                if let Some(array_start) = format.find('[') {
                    let base_format = &format[..array_start];
                    let count_str = &format[array_start + 1..format.len() - 1];
                    if let Ok(count) = count_str.parse::<usize>() {
                        let total_size = match base_format {
                            "int16s" | "int16u" => 2 * count,
                            "int32s" | "int32u" => 4 * count,
                            "int8s" | "int8u" => count,
                            _ => 2 * count,
                        };
                        
                        // Generate bounds check based on offset type
                        if byte_offset < 0 {
                            code.push_str(&format!("if {offset_var_name}_offset + {total_size} <= data.len() {{\n"));
                        } else {
                            code.push_str(&format!("    if data.len() >= {} {{\n", byte_offset as usize + total_size));
                        }
                        
                        match base_format {
                            "int16s" => {
                                if byte_offset < 0 {
                                    code.push_str(&format!("            if let Ok(values) = read_int16s_array(&data[{offset_var_name}_offset..{offset_var_name}_offset + {total_size}], byte_order, {count}) {{\n"));
                                } else {
                                    code.push_str(&format!("        if let Ok(values) = read_int16s_array(&data[{}..{}], byte_order, {}) {{\n", 
                                        byte_offset as usize, byte_offset as usize + total_size, count));
                                }
                                code.push_str("            let value_str = values.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(\" \");\n");
                                code.push_str(&format!("            tags.push((\"{}\".to_string(), TagValue::String(value_str)));\n", tag.name));
                                code.push_str("        }\n");
                            }
                            "int16u" => {
                                if byte_offset < 0 {
                                    code.push_str(&format!("            if let Ok(values) = read_int16u_array(&data[{offset_var_name}_offset..{offset_var_name}_offset + {total_size}], byte_order, {count}) {{\n"));
                                } else {
                                    code.push_str(&format!("        if let Ok(values) = read_int16u_array(&data[{}..{}], byte_order, {}) {{\n", 
                                        byte_offset as usize, byte_offset as usize + total_size, count));
                                }
                                code.push_str("            let value_str = values.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(\" \");\n");
                                code.push_str(&format!("            tags.push((\"{}\".to_string(), TagValue::String(value_str)));\n", tag.name));
                                code.push_str("        }\n");
                            }
                            _ => {
                                // For other formats, just store the raw offset for now
                                code.push_str(&format!("        // TODO: Handle format {base_format}\n"));
                            }
                        }
                        
                        code.push_str("    }\n");
                        // Close the negative offset check if needed
                        if byte_offset < 0 {
                            code.push_str("    }\n");
                        }
                    }
                }
            } else {
                // Single value format
                let value_size = match format.as_str() {
                    "int16s" | "int16u" => 2,
                    "int32s" | "int32u" => 4,
                    "int8s" | "int8u" => 1,
                    _ => 2,
                };
                
                // Generate bounds check based on offset type
                if byte_offset < 0 {
                    code.push_str(&format!("if {offset_var_name}_offset + {value_size} <= data.len() {{\n"));
                } else {
                    code.push_str(&format!("    if data.len() >= {} {{\n", byte_offset as usize + value_size));
                }
                
                match format.as_str() {
                    "int16s" => {
                        if byte_offset < 0 {
                            code.push_str(&format!("            if let Ok(value) = read_int16s(&data[{offset_var_name}_offset..{offset_var_name}_offset + {value_size}], byte_order) {{\n"));
                        } else {
                            code.push_str(&format!("        if let Ok(value) = read_int16s(&data[{}..{}], byte_order) {{\n", 
                                byte_offset as usize, byte_offset as usize + value_size));
                        }
                        code.push_str(&format!("            tags.push((\"{}\".to_string(), TagValue::I16(value)));\n", tag.name));
                        code.push_str("        }\n");
                    }
                    _ => {
                        // For other formats, just store the raw offset for now
                        code.push_str(&format!("        // TODO: Handle format {format}\n"));
                    }
                }
                
                code.push_str("    }\n");
                // Close the negative offset check if needed
                if byte_offset < 0 {
                    code.push_str("    }\n");
                }
            }
        } else {
            // No format specified - close negative offset block if needed
            if byte_offset < 0 {
                code.push_str("    }\n");
            }
        }
        
        code.push_str("    \n");
    }
    
    code.push_str("    Ok(tags)\n");
    code.push_str("}\n\n");
    
    Ok(())
}

/// Represents different types of subdirectory conditions
#[derive(Debug, Clone)]
enum SubdirectoryCondition {
    /// Count comparisons: $count == 582, $count == 1273 or $count == 1275
    Count(Vec<usize>),
    /// Model regex patterns: $$self{Model} =~ /EOS 5D/
    Model(ModelPattern),
    /// Format checks: $format eq "int32u"
    Format(FormatPattern),
    /// Complex conditions stored as runtime strings
    Runtime(String),
}

#[derive(Debug, Clone)]
struct ModelPattern {
    #[allow(dead_code)]
    regex: String,
    #[allow(dead_code)]
    negated: bool,
}

#[derive(Debug, Clone)]
struct FormatPattern {
    #[allow(dead_code)]
    format: String,
    #[allow(dead_code)]
    operator: ComparisonOp,
    #[allow(dead_code)]
    additional_condition: Option<String>, // for cases like: $format eq "int32u" and ($count == 138 or $count == 148)
}

#[derive(Debug, Clone, PartialEq)]
enum ComparisonOp {
    Eq,  // eq
    Ne,  // ne
    Regex, // =~
}

/// Parse subdirectory conditions into structured types
fn parse_subdirectory_condition(condition: &str) -> SubdirectoryCondition {
    let trimmed = condition.trim();
    
    // Count patterns
    if trimmed.contains("$count") && !trimmed.contains("$$self") && !trimmed.contains("$format") {
        let counts = parse_count_conditions(trimmed);
        if !counts.is_empty() {
            return SubdirectoryCondition::Count(counts);
        }
    }
    
    // Model patterns
    if let Some(model_pattern) = parse_model_pattern(trimmed) {
        return SubdirectoryCondition::Model(model_pattern);
    }
    
    // Format patterns
    if let Some(format_pattern) = parse_format_pattern(trimmed) {
        return SubdirectoryCondition::Format(format_pattern);
    }
    
    // Everything else goes to runtime
    SubdirectoryCondition::Runtime(condition.to_string())
}

/// Parse model regex patterns like: $$self{Model} =~ /EOS 5D/
fn parse_model_pattern(condition: &str) -> Option<ModelPattern> {
    // Match patterns like: $$self{Model} =~ /regex/ or $$self{Model} !~ /regex/
    let re = regex::Regex::new(r#"\$\$self\{Model\}\s*(=~|!~)\s*/([^/]+)/"#).ok()?;
    
    if let Some(captures) = re.captures(condition) {
        let operator = captures.get(1)?.as_str();
        let regex = captures.get(2)?.as_str().to_string();
        let negated = operator == "!~";
        
        return Some(ModelPattern { regex, negated });
    }
    
    None
}

/// Parse format patterns like: $format eq "int32u"
fn parse_format_pattern(condition: &str) -> Option<FormatPattern> {
    // Simple equality/inequality
    if let Some(captures) = regex::Regex::new(r#"\$format\s+(eq|ne)\s+"([^"]+)""#).ok()?.captures(condition) {
        let operator = match captures.get(1)?.as_str() {
            "eq" => ComparisonOp::Eq,
            "ne" => ComparisonOp::Ne,
            _ => return None,
        };
        let format = captures.get(2)?.as_str().to_string();
        
        // Check for additional conditions after "and"
        let additional = if condition.contains(" and ") {
            condition.split(" and ").nth(1).map(|s| s.trim().to_string())
        } else {
            None
        };
        
        return Some(FormatPattern { format, operator, additional_condition: additional });
    }
    
    // Regex format patterns like: $format =~ /^int16/
    if let Some(captures) = regex::Regex::new(r#"\$format\s+=~\s*/([^/]+)/"#).ok()?.captures(condition) {
        let regex = captures.get(1)?.as_str().to_string();
        return Some(FormatPattern {
            format: regex,
            operator: ComparisonOp::Regex,
            additional_condition: None,
        });
    }
    
    None
}

/// Parse count conditions including OR operators
/// Handles:
/// - Simple: "$count == 582"
/// - OR: "$count == 1273 or $count == 1275"
/// - Perl OR: "$count == 1536 || $count == 2048"
/// - Multi-line: "$count == 692 or $count == 674 or $count == 702"
fn parse_count_conditions(condition: &str) -> Vec<usize> {
    let mut counts = Vec::new();
    
    // Normalize both "or" and "||" to a common separator
    let normalized = condition
        .replace("||", " or ")
        .replace('\n', " ");
    
    // Split on " or " and process each part
    for part in normalized.split(" or ") {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        
        // Look for "== number" pattern
        if let Some(eq_pos) = trimmed.find("==") {
            let count_str = trimmed[eq_pos + 2..].trim();
            if let Ok(count) = count_str.parse::<usize>() {
                counts.push(count);
            }
        }
    }
    
    counts
}

/// Generate a conditional dispatch function for a tag with subdirectory variants
fn generate_subdirectory_dispatcher(
    code: &mut String, 
    tag_id: u32, 
    collection: &SubDirectoryCollection,
    current_module: &str,
    _shared_tables: &SharedTablesData,
) -> Result<()> {
    code.push_str(&format!("pub fn process_tag_{tag_id:#x}_subdirectory(data: &[u8], byte_order: ByteOrder) -> Result<Vec<(String, TagValue)>> {{\n"));
    code.push_str("    use tracing::debug;\n");
    code.push_str("    // TODO: Accept model and format parameters when runtime integration supports it\n");
    
    // Determine the format for count calculation (usually int16s for Canon)
    let format_size = 2; // Default to int16s
    code.push_str(&format!("    let count = data.len() / {format_size};\n"));
    code.push_str(&format!("    debug!(\"process_tag_{tag_id:#x}_subdirectory called with {{}} bytes, count={{}}\", data.len(), count);\n"));
    code.push_str("    \n");
    
    // Analyze all variants and their conditions
    let mut _has_count = false;
    let mut _has_model = false;
    let mut _has_format = false;
    let mut _has_runtime = false;
    
    for variant in &collection.variants {
        if let Some(condition_str) = &variant.condition {
            let condition = parse_subdirectory_condition(condition_str);
            match condition {
                SubdirectoryCondition::Count(_) => _has_count = true,
                SubdirectoryCondition::Model(_) => _has_model = true,
                SubdirectoryCondition::Format(_) => _has_format = true,
                SubdirectoryCondition::Runtime(_) => _has_runtime = true,
            }
        }
    }
    
    // Check if there's only one variant with no condition
    if collection.variants.len() == 1 && collection.variants[0].condition.is_none() {
        // Direct call to the single processor function
        let variant = &collection.variants[0];
        if variant.table_name == "Unknown" || is_cross_module_reference(&variant.table_name, current_module) {
            // Special case for "Unknown" table or cross-module reference
            if variant.table_name == "Unknown" {
                code.push_str("    // Reference to Unknown table\n");
            } else {
                code.push_str(&format!("    // Cross-module reference to {}\n", variant.table_name));
            }
            code.push_str("    // TODO: Implement cross-module subdirectory support\n");
            code.push_str("    Ok(vec![])\n");
        } else {
            let table_fn_name = variant.table_name
                .replace("Image::ExifTool::", "")
                .replace("::", "_")
                .to_lowercase();
            
            code.push_str("    // Single unconditional subdirectory\n");
            code.push_str(&format!("    process_{table_fn_name}(data, byte_order)\n"));
        }
    } else {
        // For now, only generate count-based matching
        // Model and format conditions will be added when runtime supports them
        code.push_str("    match count {\n");
        
        for variant in &collection.variants {
            if let Some(condition_str) = &variant.condition {
                let condition = parse_subdirectory_condition(condition_str);
                
                match condition {
                    SubdirectoryCondition::Count(counts) => {
                        if is_cross_module_reference(&variant.table_name, current_module) {
                            // Cross-module reference - add comment
                            for count_val in counts {
                                code.push_str(&format!("        {count_val} => {{\n"));
                                code.push_str(&format!("            // Cross-module reference to {}\n", variant.table_name));
                                code.push_str("            // TODO: Implement cross-module subdirectory support\n");
                                code.push_str("            Ok(vec![])\n");
                                code.push_str("        }\n");
                            }
                        } else {
                            let table_fn_name = variant.table_name
                                .replace("Image::ExifTool::", "")
                                .replace("::", "_")
                                .to_lowercase();
                                
                            for count_val in counts {
                                code.push_str(&format!("        {count_val} => {{\n"));
                                code.push_str(&format!("            debug!(\"Matched count {count_val} for variant {table_fn_name}\");\n"));
                                code.push_str(&format!("            process_{table_fn_name}(data, byte_order)\n"));
                                code.push_str("        }\n");
                            }
                        }
                    }
                    SubdirectoryCondition::Model(_pattern) => {
                        // Add as comment for now
                        let escaped = escape_string(condition_str);
                        code.push_str(&format!("        // Model condition not yet supported: {escaped}\n"));
                        code.push_str(&format!("        // Would dispatch to: {}\n", variant.table_name));
                    }
                    SubdirectoryCondition::Format(_pattern) => {
                        // Add as comment for now
                        let escaped = escape_string(condition_str);
                        code.push_str(&format!("        // Format condition not yet supported: {escaped}\n"));
                        code.push_str(&format!("        // Would dispatch to: {}\n", variant.table_name));
                    }
                    SubdirectoryCondition::Runtime(runtime_str) => {
                        // Add as comment for now
                        let escaped = escape_string(&runtime_str);
                        code.push_str(&format!("        // Runtime condition not yet supported: {escaped}\n"));
                        code.push_str(&format!("        // Would dispatch to: {}\n", variant.table_name));
                    }
                }
            }
        }
        
        code.push_str("        _ => Ok(vec![]), // No matching variant\n");
        code.push_str("    }\n");
    }
    code.push_str("}\n\n");
    
    Ok(())
}