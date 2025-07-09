//! Rust code generation tool for exif-oxide (modularized version)
//!
//! This tool reads JSON output from Perl extractors and generates
//! Rust code following the "runtime references, no stubs" architecture.

use anyhow::{Context, Result};
use clap::{Arg, Command};
use std::fs;
use std::path::Path;

mod common;
mod generators;
mod schemas;
mod validation;

use common::{normalize_format, parse_hex_id};
use generators::{
    generate_composite_tag_table, generate_conversion_refs,
    generate_mod_file, generate_supported_tags, generate_tag_table, macro_based,
};
use schemas::{CompositeData, ExtractedData, ExtractedTable, GeneratedCompositeTag, GeneratedTag};
use validation::validate_all_configs;

fn main() -> Result<()> {
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

    // Process tag tables
    let tag_data_file = current_dir.join(tag_data_path);
    println!("Looking for tag data at: {}", tag_data_file.display());
    if tag_data_file.exists() {
        println!("\n📋 Processing tag tables...");
        let json_data = fs::read_to_string(&tag_data_file)
            .with_context(|| format!("Failed to read {}", tag_data_file.display()))?;

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
            let composite_json = fs::read_to_string(&composite_file)
                .with_context(|| format!("Failed to read {}", composite_file.display()))?;
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
                    let json_data = fs::read_to_string(&path)?;
                    if let Ok(table) = serde_json::from_str::<ExtractedTable>(&json_data) {
                        // Use the hash name as the key
                        all_extracted_tables.insert(table.source.hash_name.clone(), table);
                    }
                }
            }
        }

        println!("  Found {} extracted tables", all_extracted_tables.len());

        // Process each module directory
        let modules = ["Canon_pm", "Nikon_pm", "ExifTool_pm", "Exif_pm", "XMP_pm"];
        for module in &modules {
            let module_config_dir = config_dir.join(module);
            if module_config_dir.exists() {
                println!("  Processing module: {}", module);
                macro_based::process_config_directory(
                    &module_config_dir,
                    module,
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
