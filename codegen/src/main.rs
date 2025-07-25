//! Rust code generation tool for exif-oxide (modularized version)
//!
//! This tool reads JSON output from Perl extractors and generates
//! Rust code following the "runtime references, no stubs" architecture.

use anyhow::Result;
use clap::{Arg, Command};
use std::fs;
use std::path::Path;
use tracing::{info, debug};
use tracing_subscriber::EnvFilter;

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
use extraction::{extract_all_simple_tables, extract_single_config};
use table_processor::process_composite_tags_only;
use file_operations::{create_directories, file_exists, write_file_atomic};
use generators::generate_mod_file;
use validation::validate_all_configs;


fn main() -> Result<()> {
    // Initialize tracing with default level of INFO to keep output quiet by default
    // Set RUST_LOG=debug for verbose output
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();
    
    let matches = Command::new("exif-oxide-codegen")
        .version("0.1.0")
        .about("Generate Rust code from ExifTool extraction data")
        .arg(
            Arg::new("config")
                .help("Single config file to process (for debugging)")
                .value_name("CONFIG_FILE")
                .index(1)
                .required(false),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("DIR")
                .help("Output directory for generated code")
                .default_value("../src/generated"),
        )
        .get_matches();

    // Check if single config mode
    if let Some(config_file) = matches.get_one::<String>("config") {
        info!("🔧 Running single config extraction for: {}", config_file);
        return extract_single_config(config_file);
    }

    let output_dir = matches.get_one::<String>("output").unwrap();

    // We're running from the codegen directory
    let current_dir = std::env::current_dir()?;

    // Create output directory
    create_directories(Path::new(output_dir))?;

    info!("🔧 exif-oxide Code Generation");
    debug!("=============================");
    
    // Extract all tables using Rust orchestration (replaces Makefile extract-* targets)
    // This now includes tag definitions and composite tags via the config system
    extract_all_simple_tables()?;

    // Process modular tag tables (only for composite tags now)
    let extract_dir = current_dir.join("generated").join("extract");
    debug!("📋 Processing composite tags...");
    process_composite_tags_only(&extract_dir, output_dir)?;

    // Tag kit processing is now integrated into the module-based system

    // The old extract.json processing has been removed.
    // All extraction is now handled by the new modular configuration system below.

    // Generate file type detection code
    debug!("📁 Generating file type detection code...");
    let extract_dir = current_dir.join("generated").join("extract");
    generators::file_detection::generate_file_detection_code(&extract_dir, output_dir)?;

    // Create or update file_types mod.rs to include generated modules
    let file_types_mod_path = format!("{output_dir}/file_types/mod.rs");
    debug!("📝 Creating/updating file_types mod.rs with generated modules...");
    
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
        updated = true;
    }
    
    if !content.contains("pub mod magic_number_patterns;") {
        // Add magic number patterns module
        content.push_str("pub mod magic_number_patterns;\n\n");
        updated = true;
    }

    // Add re-exports if not present
    if !content.contains("pub use file_type_lookup::") {
        content.push_str("// Re-export commonly used items\n");
        content.push_str("pub use file_type_lookup::{\n");
        content.push_str("    extensions_for_format, get_primary_format, lookup_file_type_by_extension,\n");
        content.push_str("    resolve_file_type, supports_format, FILE_TYPE_EXTENSIONS,\n");
        content.push_str("};\n");
        content.push_str("pub use magic_number_patterns::{\n");
        content.push_str("    get_magic_file_types, get_magic_number_pattern, matches_magic_number,\n");
        content.push_str("};\n");
        content.push('\n');
        content.push_str("// Import regex patterns from their source-based location (ExifTool.pm)\n");
        content.push_str("pub use crate::generated::ExifTool_pm::regex_patterns::{detect_file_type_by_regex, REGEX_PATTERNS};\n");
        updated = true;
    }

    if updated || !file_exists(Path::new(&file_types_mod_path)) {
        write_file_atomic(Path::new(&file_types_mod_path), &content)?;
        debug!("  ✓ Created/updated file_types mod.rs with file_type_lookup and regex_patterns re-exports");
    } else {
        debug!("  ✓ file_types mod.rs already contains all necessary declarations");
    }

    // NEW: Process using the new macro-based configuration system
    debug!("🔄 Processing new macro-based configuration...");

    let config_dir = current_dir.join("config");
    let schemas_dir = current_dir.join("schemas");

    // Validate all configurations first
    if config_dir.exists() && schemas_dir.exists() {
        validate_all_configs(&config_dir, &schemas_dir)?;

        // Load all extracted tables with their configurations
        let extract_dir = current_dir.join("generated/extract");
        let all_extracted_tables = load_extracted_tables_with_config(&extract_dir, &config_dir)?;

        debug!("  Found {} extracted tables", all_extracted_tables.len());

        // Auto-discover and process each module directory
        discover_and_process_modules(&config_dir, &all_extracted_tables, output_dir)?;

        // No macros.rs needed - using direct code generation

        // Update the main mod.rs to include new modules
        update_generated_mod_file(output_dir)?;
    } else {
        debug!("  ⚠️  New config directory structure not found, using legacy generation only");
    }

    // Generate module file
    generate_mod_file(output_dir)?;

    info!("✅ Code generation complete!");

    Ok(())
}


