//! Rust code generation tool for exif-oxide (modularized version)
//!
//! This tool reads JSON output from Perl extractors and generates
//! Rust code following the "runtime references, no stubs" architecture.

use anyhow::{anyhow, Context, Result};
use clap::{Arg, Command};
use std::fs;
use std::path::Path;

mod common;
mod schemas;
mod generators;

use common::{parse_hex_id, normalize_format};
use schemas::{ExtractedData, ExtractedTable, GeneratedTag, GeneratedCompositeTag, CompositeData};
use generators::{
    generate_tag_table,
    generate_composite_tag_table,
    generate_conversion_refs,
    generate_supported_tags,
    generate_mod_file,
    lookup_tables,
    file_detection,
    data_sets,
};

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

    // Process extracted data from individual files using modular architecture directly
    let extract_dir = current_dir.join("generated/extract");
    println!("Looking for extract directory at: {}", extract_dir.display());
    if extract_dir.exists() && extract_dir.is_dir() {
        println!("\n📊 Processing extracted data from individual files...");
        
        // Read the extract.json configuration
        let config_path = current_dir.join("extract.json");
        let config_data = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config at {}", config_path.display()))?;
        let config: serde_json::Value = serde_json::from_str(&config_data)
            .with_context(|| "Failed to parse extract.json config")?;
        
        let tables = config["tables"].as_array()
            .ok_or_else(|| anyhow::anyhow!("No tables array in config"))?;
        
        println!("  Processing {} extracted data tables...", tables.len());
        
        // Process each table based on its extraction type
        let mut processed_count = 0;
        for table_config in tables {
            let hash_name = table_config["hash_name"].as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing hash_name"))?;
            let output_file = table_config["output_file"].as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing output_file"))?;
            let extraction_type = table_config["extraction_type"].as_str();
            let constant_name = table_config["constant_name"].as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing constant_name"))?;
            
            // Load the extracted JSON data
            // The extract.pl script generates filenames from constant_name in lowercase
            let json_filename = constant_name.to_lowercase().replace("_table", "") + ".json";
            let json_file = extract_dir.join(&json_filename);
            
            if !json_file.exists() {
                println!("    ⚠️  Skipping {} - no extracted data found", hash_name);
                continue;
            }
            
            let json_data = fs::read_to_string(&json_file)?;
            let extracted_table: schemas::ExtractedTable = serde_json::from_str(&json_data)?;
            
            // Generate code based on extraction type
            let code = match extraction_type {
                Some("boolean_set") => {
                    data_sets::generate_boolean_set(hash_name, &extracted_table)?
                }
                Some("regex_strings") | Some("file_type_lookup") => {
                    // These are handled by file_detection module below
                    continue;
                }
                _ => {
                    // Default to standard lookup table
                    lookup_tables::generate_lookup_table(hash_name, &extracted_table)?
                }
            };
            
            // Write to file (directly to output_dir for flattened structure)
            let output_path = format!("{}/{}", output_dir, output_file);
            
            // Create subdirectory if needed
            if let Some(parent) = std::path::Path::new(&output_path).parent() {
                fs::create_dir_all(parent)?;
            }
            
            println!("    Writing to: {}", output_path);
            fs::write(&output_path, code)?;
            
            processed_count += 1;
        }
        
        println!("  ✓ Generated {} tables using modular architecture", processed_count);
        
        // Generate file detection code (file_type_lookup and regex patterns)
        println!("\n  🔍 Generating file detection code...");
        file_detection::generate_file_detection_code(&extract_dir, output_dir)?;
    } else {
        println!("Extract directory not found!");
    }

    // Note: file_type_lookup and regex_patterns are now handled by the modular architecture
    // through the file_detection module
    
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
            println!("  ✓ Updated file_types mod.rs with file_type_lookup and magic_numbers modules");
        } else {
            println!("  ✓ file_types mod.rs already contains all necessary declarations");
        }
    }

    // Generate module file
    generate_mod_file(output_dir)?;

    println!("\n✅ Code generation complete!");
    println!("\nNext steps:");
    println!("1. Add 'mod generated;' to src/lib.rs");
    println!("2. Use --show-missing on real images to see what implementations are needed");
    println!("3. Implement missing PrintConv/ValueConv and composite functions in implementations/");

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