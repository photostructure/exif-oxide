//! Rust code generation tool for exif-oxide (modularized version)
//!
//! This tool reads JSON output from Perl extractors and generates
//! Rust code following the "runtime references, no stubs" architecture.

use anyhow::{Context, Result};
use clap::{Arg, Command};
use std::fs;
use std::path::Path;
use std::time::Instant;
use tracing::{info, debug, warn};
use tracing_subscriber::EnvFilter;

mod common;
mod config;
mod conv_registry;
mod discovery;
mod expression_compiler;
mod extraction;
mod extractors;
mod field_extractor;
mod file_operations;
mod generators;
mod schemas;
mod strategies;
mod table_processor;
mod validation;

use config::load_extracted_tables_with_config;
use discovery::{discover_and_process_modules, update_generated_mod_file};
use extraction::{extract_all_simple_tables, extract_single_config};
use table_processor::process_composite_tags_only;
use file_operations::{create_directories, file_exists, write_file_atomic};
use generators::generate_mod_file;
use validation::validate_all_configs;
use field_extractor::FieldExtractor;
use strategies::{StrategyDispatcher, GeneratedFile};


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
        .arg(
            Arg::new("universal")
                .long("universal")
                .help("Use universal symbol table extraction instead of config-based extraction")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    // Check if single config mode
    if let Some(config_file) = matches.get_one::<String>("config") {
        info!("🔧 Running single config extraction for: {}", config_file);
        return extract_single_config(config_file);
    }

    let output_dir = matches.get_one::<String>("output").unwrap();
    let use_universal = matches.get_flag("universal");

    // We're running from the codegen directory
    let current_dir = std::env::current_dir()?;

    // Create output directory
    create_directories(Path::new(output_dir))?;

    info!("🔧 exif-oxide Code Generation");
    debug!("=============================");
    
    if use_universal {
        info!("🔄 Using universal symbol table extraction");
        run_universal_extraction(&current_dir, output_dir)?;
    } else {
        // Extract all tables using Rust orchestration (replaces Makefile extract-* targets)
        // This now includes tag definitions and composite tags via the config system
        let start = Instant::now();
        extract_all_simple_tables()?;
        info!("📊 Extract phase completed in {:.2}s", start.elapsed().as_secs_f64());
    }

    // Process modular tag tables (only for composite tags now)
    let extract_dir = current_dir.join("generated").join("extract");
    debug!("📋 Processing composite tags...");
    let start = Instant::now();
    process_composite_tags_only(&extract_dir, output_dir)?;
    info!("📋 Composite tags phase completed in {:.2}s", start.elapsed().as_secs_f64());

    // Tag kit processing is now integrated into the module-based system

    // The old extract.json processing has been removed.
    // All extraction is now handled by the new modular configuration system below.

    // Generate file type detection code
    debug!("📁 Generating file type detection code...");
    let extract_dir = current_dir.join("generated").join("extract");
    let start = Instant::now();
    generators::file_detection::generate_file_detection_code(&extract_dir, output_dir)?;
    info!("📁 File detection generation completed in {:.2}s", start.elapsed().as_secs_f64());

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

    // Add re-exports if not present
    if !content.contains("pub use file_type_lookup::") {
        content.push_str("// Re-export commonly used items\n");
        content.push_str("pub use file_type_lookup::{\n");
        content.push_str("    extensions_for_format, get_primary_format, lookup_file_type_by_extension,\n");
        content.push_str("    resolve_file_type, supports_format, FILE_TYPE_EXTENSIONS,\n");
        content.push_str("};\n");
        content.push('\n');
        // Note: Regex patterns are used directly from ExifTool_pm::regex_patterns
        // No re-export needed since all code uses the direct import path
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
        let start = Instant::now();
        validate_all_configs(&config_dir, &schemas_dir)?;
        debug!("  ✓ Config validation completed in {:.2}s", start.elapsed().as_secs_f64());

        // Load all extracted tables with their configurations
        let extract_dir = current_dir.join("generated/extract");
        let start = Instant::now();
        let all_extracted_tables = load_extracted_tables_with_config(&extract_dir, &config_dir)?;
        debug!("  ✓ Loaded {} extracted tables in {:.2}s", all_extracted_tables.len(), start.elapsed().as_secs_f64());

        // Auto-discover and process each module directory
        let start = Instant::now();
        discover_and_process_modules(&config_dir, &all_extracted_tables, output_dir)?;
        info!("🔄 Module processing phase completed in {:.2}s", start.elapsed().as_secs_f64());

        // No macros.rs needed - using direct code generation

        // Update the main mod.rs to include new modules
        let start = Instant::now();
        update_generated_mod_file(output_dir)?;
        debug!("  ✓ Updated generated mod.rs in {:.2}s", start.elapsed().as_secs_f64());
    } else {
        debug!("  ⚠️  New config directory structure not found, using legacy generation only");
    }

    // Generate module file
    let start = Instant::now();
    generate_mod_file(output_dir)?;
    debug!("  ✓ Generated module file in {:.2}s", start.elapsed().as_secs_f64());

    info!("✅ Code generation complete!");

    Ok(())
}

/// Run field extraction with strategy-based processing
fn run_universal_extraction(current_dir: &Path, output_dir: &str) -> Result<()> {
    let extractor = FieldExtractor::new();
    let mut dispatcher = StrategyDispatcher::new();
    let exiftool_lib_dir = current_dir.join("../third-party/exiftool/lib/Image/ExifTool");
    
    info!("🔍 Scanning ExifTool modules in: {}", exiftool_lib_dir.display());
    
    // Find all .pm files in the ExifTool directory
    let mut module_paths = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(&exiftool_lib_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("pm") {
                module_paths.push(path);
            }
        }
    }
    
    info!("📦 Found {} ExifTool modules to process", module_paths.len());
    
    // Process a subset for initial testing (GPS, DNG, Canon for comprehensive validation)
    let test_modules = ["GPS.pm", "DNG.pm", "Canon.pm"];
    let test_paths: Vec<_> = module_paths
        .iter()
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| test_modules.contains(&name))
                .unwrap_or(false)
        })
        .collect();
    
    info!("🧪 Processing {} test modules: {:?}", test_paths.len(), 
          test_paths.iter().map(|p| p.file_name().unwrap().to_string_lossy()).collect::<Vec<_>>());
    
    let start = Instant::now();
    let mut all_symbols = Vec::new();
    
    // Extract symbols from all modules
    for module_path in test_paths {
        match extractor.extract_module(module_path) {
            Ok((symbols, stats)) => {
                info!("✅ {}: {} symbols extracted from {} total", 
                      stats.module_name, stats.extracted_symbols, stats.total_symbols);
                
                debug!("  📋 {} symbols ready for strategy processing", symbols.len());
                all_symbols.extend(symbols);
            }
            Err(e) => {
                warn!("❌ Failed to extract from {}: {}", module_path.display(), e);
            }
        }
    }
    
    let extraction_time = start.elapsed();
    info!("🔄 Field extraction completed in {:.2}s", extraction_time.as_secs_f64());
    
    // Process extracted symbols through strategy system
    if !all_symbols.is_empty() {
        let strategy_start = Instant::now();
        info!("🎯 Processing {} symbols through strategy system", all_symbols.len());
        
        match dispatcher.process_symbols(all_symbols, output_dir) {
            Ok(generated_files) => {
                let strategy_time = strategy_start.elapsed();
                info!("✅ Strategy processing completed in {:.2}s", strategy_time.as_secs_f64());
                
                // Write generated files to disk
                let write_start = Instant::now();
                write_generated_files(&generated_files, output_dir)?;
                let write_time = write_start.elapsed();
                
                info!("📁 {} files written in {:.2}s", generated_files.len(), write_time.as_secs_f64());
                info!("🏁 Total field extraction time: {:.2}s", 
                      (extraction_time + strategy_time + write_time).as_secs_f64());
            }
            Err(e) => {
                warn!("❌ Strategy processing failed: {}", e);
                return Err(e);
            }
        }
    } else {
        warn!("⚠️  No symbols extracted for processing");
    }
    
    Ok(())
}

/// Write generated files to disk with appropriate directory structure
fn write_generated_files(files: &[GeneratedFile], base_output_dir: &str) -> Result<()> {
    use std::fs;
    
    for file in files {
        let full_path = Path::new(base_output_dir).join(&file.path);
        
        // Create parent directories if they don't exist
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Write the file
        fs::write(&full_path, &file.content)
            .with_context(|| format!("Failed to write generated file: {}", full_path.display()))?;
        
        debug!("📝 Written: {} ({} bytes)", file.path, file.content.len());
    }
    
    Ok(())
}


