//! Rust code generation tool for exif-oxide (modularized version)
//!
//! This tool reads JSON output from Perl extractors and generates
//! Rust code following the "runtime references, no stubs" architecture.

use anyhow::{Context, Result};
use clap::{Arg, Command};
use std::fs;
use std::path::Path;

mod common;
mod extraction;
mod generators;
mod patching;
mod schemas;
mod validation;

use common::{normalize_format, parse_hex_id};
use extraction::extract_all_simple_tables;
use generators::{
    generate_composite_tag_table, generate_conversion_refs,
    generate_mod_file, generate_supported_tags, generate_tag_table, macro_based,
};
use schemas::{CompositeData, ExtractedData, ExtractedTable, GeneratedCompositeTag, GeneratedTag, TableEntry, TableSource};
use validation::validate_all_configs;
use serde::Deserialize;

/// Simplified structure for extracted JSON files (temporary)
#[derive(Debug, Deserialize)]
struct SimpleExtractedTable {
    pub source: TableSource,
    pub metadata: SimpleMetadata,
    pub entries: Vec<TableEntry>,
}

#[derive(Debug, Deserialize)]
struct SimpleMetadata {
    pub entry_count: usize,
}

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    let matches = Command::new("exif-oxide-codegen")
        .version("0.1.0")
        .about("Generate Rust code from ExifTool extraction data")
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("DIR")
                .help("Output directory for generated code")
                .default_value("../src/generated"),
        )
        .arg(
            Arg::new("tag-data")
                .long("tag-data")
                .value_name("FILE")
                .help("Path to tag extraction JSON")
                .default_value("generated/tag_tables.json"),
        )
        .get_matches();

    let output_dir = matches.get_one::<String>("output").unwrap();
    let tag_data_path = matches.get_one::<String>("tag-data").unwrap();

    // We're running from the codegen directory
    let current_dir = std::env::current_dir()?;

    // Create output directory
    fs::create_dir_all(output_dir)?;

    println!("🔧 exif-oxide Code Generation");
    println!("=============================");
    
    // Extract simple tables using Rust orchestration (replaces Makefile extract-* targets)
    extract_all_simple_tables()?;

    // Process tag tables
    let tag_data_file = current_dir.join(tag_data_path);
    println!("Looking for tag data at: {}", tag_data_file.display());
    if tag_data_file.exists() {
        println!("\n📋 Processing tag tables...");
        let json_data = match fs::read_to_string(&tag_data_file) {
            Ok(data) => data,
            Err(err) => {
                // Handle UTF-8 errors gracefully by reading as bytes and converting
                eprintln!("Warning: UTF-8 error reading {}: {}", tag_data_file.display(), err);
                let bytes = fs::read(&tag_data_file)
                    .with_context(|| format!("Failed to read bytes from {}", tag_data_file.display()))?;
                String::from_utf8_lossy(&bytes).into_owned()
            }
        };

        let extracted: ExtractedData = serde_json::from_str(&json_data)
            .with_context(|| "Failed to parse tag extraction JSON")?;

        // Convert extracted tags to generated format
        let generated_tags = convert_tags(&extracted)?;

        // Generate code for tag tables
        generate_tag_table(&generated_tags, output_dir)?;
        generate_conversion_refs(&extracted.conversion_refs, output_dir)?;

        // Process composite tags separately
        let composite_file = current_dir.join("generated/composite_tags.json");
        if composite_file.exists() {
            println!("\n🔗 Processing composite tags...");
            let composite_json = match fs::read_to_string(&composite_file) {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("Warning: UTF-8 error reading {}: {}", composite_file.display(), err);
                    let bytes = fs::read(&composite_file)
                        .with_context(|| format!("Failed to read bytes from {}", composite_file.display()))?;
                    String::from_utf8_lossy(&bytes).into_owned()
                }
            };
            let composite_data: CompositeData = serde_json::from_str(&composite_json)
                .with_context(|| "Failed to parse composite tags JSON")?;

            let generated_composites = convert_composite_tags_from_data(&composite_data)?;
            generate_composite_tag_table(&generated_composites, output_dir)?;
            generate_supported_tags(&generated_tags, &generated_composites, output_dir)?;
        } else {
            // Generate without composite tags
            generate_supported_tags(&generated_tags, &[], output_dir)?;
        }
    } else {
        println!("Tag data file not found!");
    }

    // The old extract.json processing has been removed.
    // All extraction is now handled by the new modular configuration system below.

    // Update file_types mod.rs to include generated modules
    let file_types_mod_path = format!("{}/file_types/mod.rs", output_dir);
    if Path::new(&file_types_mod_path).exists() {
        println!("\n📝 Updating file_types mod.rs with generated modules...");
        let mut content = fs::read_to_string(&file_types_mod_path)?;

        // Check if modules are already declared
        let mut updated = false;

        if !content.contains("pub mod file_type_lookup;") {
            // Find where to insert the module declarations (after other module declarations)
            if let Some(pos) = content.find("\n// Re-export") {
                let module_decls = "pub mod file_type_lookup;\npub mod magic_numbers;\n\n";
                content.insert_str(pos, &format!("\n{}", module_decls));
                updated = true;
            }
        }

        // Add re-exports if not present
        if !content.contains("pub use file_type_lookup::") {
            // Find the end of existing re-exports
            if let Some(pos) = content.rfind("pub use") {
                if let Some(end_pos) = content[pos..].find(";\n") {
                    let insert_pos = pos + end_pos + 2;
                    let re_exports = "pub use file_type_lookup::{resolve_file_type, get_primary_format, supports_format, extensions_for_format};\npub use magic_numbers::{get_magic_pattern, get_magic_file_types};\n";
                    content.insert_str(insert_pos, re_exports);
                    updated = true;
                }
            }
        }

        if updated {
            fs::write(&file_types_mod_path, content)?;
            println!(
                "  ✓ Updated file_types mod.rs with file_type_lookup and magic_numbers modules"
            );
        } else {
            println!("  ✓ file_types mod.rs already contains all necessary declarations");
        }
    }

    // NEW: Process using the new macro-based configuration system
    println!("\n🔄 Processing new macro-based configuration...");

    let config_dir = current_dir.join("config");
    let schemas_dir = current_dir.join("schemas");

    // Validate all configurations first
    if config_dir.exists() && schemas_dir.exists() {
        validate_all_configs(&config_dir, &schemas_dir)?;

        // Load all extracted tables into a HashMap for easy lookup
        let mut all_extracted_tables = std::collections::HashMap::new();

        // Read all JSON files from generated/extract directory
        let extract_dir = current_dir.join("generated/extract");
        if extract_dir.exists() {
            for entry in fs::read_dir(&extract_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    let json_data = match fs::read_to_string(&path) {
                        Ok(data) => data,
                        Err(err) => {
                            eprintln!("Warning: UTF-8 error reading {}: {}", path.display(), err);
                            let bytes = fs::read(&path)
                                .with_context(|| format!("Failed to read bytes from {}", path.display()))?;
                            String::from_utf8_lossy(&bytes).into_owned()
                        }
                    };
                    // Skip empty files
                    if json_data.trim().is_empty() {
                        println!("  ⚠️  Skipping empty file: {}", path.display());
                        continue;
                    }
                    // Try to parse as SimpleExtractedTable first
                    match serde_json::from_str::<SimpleExtractedTable>(&json_data) {
                        Ok(simple_table) => {
                            // Get config metadata from the module's simple_table.json
                            let module_name = simple_table.source.module.replace(".pm", "_pm");
                            let config_file = config_dir.join(&module_name).join("simple_table.json");
                            
                            if config_file.exists() {
                                if let Ok(config_content) = fs::read_to_string(&config_file) {
                                    if let Ok(config_json) = serde_json::from_str::<serde_json::Value>(&config_content) {
                                        if let Some(tables) = config_json["tables"].as_array() {
                                            // Find matching table config
                                            let hash_name = &simple_table.source.hash_name;
                                            if let Some(table_config) = tables.iter().find(|t| t["hash_name"] == *hash_name) {
                                                // Create full ExtractedTable with metadata from config
                                                let table = ExtractedTable {
                                                    source: simple_table.source,
                                                    metadata: schemas::input::TableMetadata {
                                                        description: table_config["description"].as_str().unwrap_or("").to_string(),
                                                        constant_name: table_config["constant_name"].as_str().unwrap_or("").to_string(),
                                                        key_type: table_config["key_type"].as_str().unwrap_or("String").to_string(),
                                                        entry_count: simple_table.metadata.entry_count,
                                                    },
                                                    entries: simple_table.entries,
                                                };
                                                all_extracted_tables.insert(table.source.hash_name.clone(), table);
                                                continue;
                                            }
                                        }
                                    }
                                }
                            }
                            
                            eprintln!("Warning: Could not find config for {}: {}", path.display(), simple_table.source.hash_name);
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to parse {}: {}", path.display(), e);
                            eprintln!("  First 200 chars of content: {}", &json_data.chars().take(200).collect::<String>());
                        }
                    }
                }
            }
        }

        println!("  Found {} extracted tables", all_extracted_tables.len());

        // Auto-discover and process each module directory
        let config_entries = fs::read_dir(&config_dir)
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
                println!("  Processing module: {}", module_name);
                macro_based::process_config_directory(
                    &module_config_dir,
                    &module_name,
                    &all_extracted_tables,
                    output_dir,
                )?;
            }
        }

        // Generate macros.rs file
        let macros_path = format!("{}/macros.rs", output_dir);
        if !Path::new(&macros_path).exists() {
            println!("  Note: macros.rs should already exist at src/generated/macros.rs");
        }

        // Update the main mod.rs to include new modules
        update_generated_mod_file(output_dir)?;
    } else {
        println!("  ⚠️  New config directory structure not found, using legacy generation only");
    }

    // Generate module file
    generate_mod_file(output_dir)?;

    println!("\n✅ Code generation complete!");

    Ok(())
}

/// Convert extracted tags to generated format
fn convert_tags(data: &ExtractedData) -> Result<Vec<GeneratedTag>> {
    let mut all_tags = Vec::new();

    // Convert EXIF tags
    for tag in &data.tags.exif {
        all_tags.push(GeneratedTag {
            id: parse_hex_id(&tag.id)?,
            name: tag.name.clone(),
            format: normalize_format(&tag.format),
            groups: tag.groups.clone(),
            writable: tag.writable != 0,
            description: tag.description.clone(),
            print_conv_ref: tag.print_conv_ref.clone(),
            value_conv_ref: tag.value_conv_ref.clone(),
            notes: tag.notes.clone(),
        });
    }

    // Convert GPS tags
    for tag in &data.tags.gps {
        all_tags.push(GeneratedTag {
            id: parse_hex_id(&tag.id)?,
            name: tag.name.clone(),
            format: normalize_format(&tag.format),
            groups: tag.groups.clone(),
            writable: tag.writable != 0,
            description: tag.description.clone(),
            print_conv_ref: tag.print_conv_ref.clone(),
            value_conv_ref: tag.value_conv_ref.clone(),
            notes: tag.notes.clone(),
        });
    }

    Ok(all_tags)
}

/// Convert extracted composite tags to generated format
/// Update the generated mod.rs file to include new module structure
fn update_generated_mod_file(output_dir: &str) -> Result<()> {
    let mod_path = format!("{}/mod.rs", output_dir);
    let mut content = if Path::new(&mod_path).exists() {
        fs::read_to_string(&mod_path)?
    } else {
        String::new()
    };

    // Add macros module if not present
    if !content.contains("pub mod macros;") {
        content.insert_str(0, "#[macro_use]\npub mod macros;\n\n");
    }

    // Add new module directories
    let modules = ["Canon_pm", "Nikon_pm", "ExifTool_pm", "Exif_pm", "XMP_pm"];
    for module in &modules {
        let module_dir = Path::new(output_dir).join(module);
        if module_dir.exists() && module_dir.join("mod.rs").exists() {
            let mod_declaration = format!("pub mod {};\n", module);
            if !content.contains(&mod_declaration) {
                content.push_str(&mod_declaration);
            }
        }
    }

    fs::write(&mod_path, content)?;
    Ok(())
}

fn convert_composite_tags_from_data(data: &CompositeData) -> Result<Vec<GeneratedCompositeTag>> {
    Ok(data
        .composite_tags
        .iter()
        .map(|tag| GeneratedCompositeTag {
            name: tag.name.clone(),
            table: tag.table.clone(),
            require: tag.require.clone().unwrap_or_default(),
            desire: tag.desire.clone().unwrap_or_default(),
            print_conv_ref: tag.print_conv_ref.clone(),
            value_conv_ref: tag.value_conv_ref.clone(),
            description: tag.description.clone(),
            writable: tag.writable != 0,
        })
        .collect())
}
