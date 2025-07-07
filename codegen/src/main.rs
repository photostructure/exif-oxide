//! Rust code generation tool for exif-oxide (modularized version)
//!
//! This tool reads JSON output from Perl extractors and generates
//! Rust code following the "runtime references, no stubs" architecture.

use anyhow::{Context, Result};
use clap::{Arg, Command};
use std::fs;
use std::path::Path;

mod common;
mod schemas;
mod generators;

use common::{parse_hex_id, normalize_format};
use schemas::{ExtractedData, GeneratedTag, GeneratedCompositeTag, CompositeData};
use generators::{
    generate_tag_table,
    generate_composite_tag_table,
    generate_conversion_refs,
    generate_supported_tags,
    generate_simple_tables,
    generate_file_type_lookup,
    generate_regex_patterns,
    generate_mod_file,
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

    // Process simple tables from individual files
    let simple_tables_dir = current_dir.join("generated/simple_tables");
    println!("Looking for simple tables directory at: {}", simple_tables_dir.display());
    if simple_tables_dir.exists() && simple_tables_dir.is_dir() {
        println!("\n📊 Processing simple tables from individual files...");
        generate_simple_tables(&simple_tables_dir, output_dir)?;
    } else {
        println!("Simple tables directory not found!");
    }

    // Process file type lookup data
    let file_type_lookup_file = current_dir.join("generated/file_type_lookup.json");
    println!("Looking for file type lookup at: {}", file_type_lookup_file.display());
    if file_type_lookup_file.exists() {
        println!("\n🔍 Processing file type lookup data...");
        generate_file_type_lookup(&file_type_lookup_file, output_dir)?;
    } else {
        println!("File type lookup data not found!");
    }

    // Process regex patterns data
    let regex_patterns_file = current_dir.join("generated/regex_patterns.json");
    println!("Looking for regex patterns at: {}", regex_patterns_file.display());
    if regex_patterns_file.exists() {
        println!("\n🎯 Processing regex patterns data...");
        // TODO: Temporarily disabled due to UTF-8 error
        // generate_regex_patterns(&regex_patterns_file, output_dir)?;
        println!("⚠️  Skipping regex patterns due to UTF-8 encoding issue");
    } else {
        println!("Regex patterns data not found!");
    }
    
    // Update file_types mod.rs to include generated modules
    let file_types_mod_path = format!("{}/simple_tables/file_types/mod.rs", output_dir);
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