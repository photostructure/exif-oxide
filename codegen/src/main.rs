//! Rust code generation tool for exif-oxide (modularized version)
//!
//! This tool reads JSON output from Perl extractors and generates
//! Rust code following the "runtime references, no stubs" architecture.

use anyhow::{Context, Result};
use clap::{Arg, Command};
use std::fs;

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
        .arg(
            Arg::new("simple-tables")
                .long("simple-tables")
                .value_name("FILE")
                .help("Path to simple tables JSON")
                .default_value("generated/simple_tables.json"),
        )
        .get_matches();

    let output_dir = matches.get_one::<String>("output").unwrap();
    let tag_data_path = matches.get_one::<String>("tag-data").unwrap();
    let simple_tables_path = matches.get_one::<String>("simple-tables").unwrap();

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

    // Process simple tables
    let simple_tables_file = current_dir.join(simple_tables_path);
    println!("Looking for simple tables at: {}", simple_tables_file.display());
    if simple_tables_file.exists() {
        println!("\n📊 Processing simple tables...");
        generate_simple_tables(&simple_tables_file, output_dir)?;
    } else {
        println!("Simple tables file not found!");
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