//! Rust code generation tool for exif-oxide (modularized version)
//!
//! This tool reads JSON output from Perl extractors and generates
//! Rust code following the "runtime references, no stubs" architecture.

use anyhow::{Context, Result};
use clap::{Arg, Command};
use std::fs;
use std::path::Path;

mod common;
mod config;
mod discovery;
mod extraction;
mod extractors;
mod file_operations;
mod generated;
mod generators;
mod patching;
mod schemas;
mod table_processor;
mod validation;

use config::load_extracted_tables_with_config;
use discovery::{discover_and_process_modules, update_generated_mod_file};
use extraction::extract_all_simple_tables;
use table_processor::process_tag_tables_modular;
use file_operations::{create_directories, file_exists, read_utf8_with_fallback, write_file_atomic};
use generators::{generate_conversion_refs, generate_mod_file};
use validation::validate_all_configs;


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
    create_directories(Path::new(output_dir))?;

    println!("🔧 exif-oxide Code Generation");
    println!("=============================");
    
    // Extract all tables using Rust orchestration (replaces Makefile extract-* targets)
    // This now includes tag definitions and composite tags via the config system
    extract_all_simple_tables()?;

    // Process modular tag tables
    let extract_dir = current_dir.join("generated").join("extract");
    println!("\n📋 Processing modular tag tables...");
    process_tag_tables_modular(&extract_dir, output_dir)?;

    // Tag kit processing is now integrated into the module-based system

    // The old extract.json processing has been removed.
    // All extraction is now handled by the new modular configuration system below.

    // Generate file type detection code
    println!("\n📁 Generating file type detection code...");
    let extract_dir = current_dir.join("generated").join("extract");
    generators::file_detection::generate_file_detection_code(&extract_dir, &output_dir)?;

    // Create or update file_types mod.rs to include generated modules
    let file_types_mod_path = format!("{}/file_types/mod.rs", output_dir);
    println!("\n📝 Creating/updating file_types mod.rs with generated modules...");
    
    // Create default content if file doesn't exist
    let mut content = if file_exists(Path::new(&file_types_mod_path)) {
        fs::read_to_string(&file_types_mod_path)?
    } else {
        // Create default mod.rs content
        String::from(
            "//! File type detection module\n\
             //!\n\
             //! This module contains generated code for file type detection.\n\n"
        )
    };

    // Check if modules are already declared
    let mut updated = false;

    if !content.contains("pub mod file_type_lookup;") {
        // Add module declarations
        content.push_str("pub mod file_type_lookup;\n");
        content.push_str("pub mod magic_number_patterns;\n\n");
        updated = true;
    }

    // Add re-exports if not present
    if !content.contains("pub use file_type_lookup::") {
        content.push_str("// Re-export commonly used items\n");
        content.push_str("pub use file_type_lookup::{lookup_file_type_by_extension, FILE_TYPE_EXTENSIONS};\n");
        content.push_str("pub use magic_number_patterns::{detect_file_type_by_magic, MAGIC_NUMBER_PATTERNS};\n");
        updated = true;
    }

    if updated || !file_exists(Path::new(&file_types_mod_path)) {
        write_file_atomic(Path::new(&file_types_mod_path), &content)?;
        println!("  ✓ Created/updated file_types mod.rs with file_type_lookup and magic_number_patterns modules");
    } else {
        println!("  ✓ file_types mod.rs already contains all necessary declarations");
    }

    // NEW: Process using the new macro-based configuration system
    println!("\n🔄 Processing new macro-based configuration...");

    let config_dir = current_dir.join("config");
    let schemas_dir = current_dir.join("schemas");

    // Validate all configurations first
    if config_dir.exists() && schemas_dir.exists() {
        validate_all_configs(&config_dir, &schemas_dir)?;

        // Load all extracted tables with their configurations
        let extract_dir = current_dir.join("generated/extract");
        let all_extracted_tables = load_extracted_tables_with_config(&extract_dir, &config_dir)?;

        println!("  Found {} extracted tables", all_extracted_tables.len());

        // Auto-discover and process each module directory
        discover_and_process_modules(&config_dir, &all_extracted_tables, output_dir)?;

        // No macros.rs needed - using direct code generation

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


