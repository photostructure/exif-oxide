//! Rust code generation tool for exif-oxide (modularized version)
//!
//! This tool reads JSON output from Perl extractors and generates
//! Rust code following the "runtime references, no stubs" architecture.

use anyhow::{Context, Result};
use clap::{Arg, Command};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Instant;
use tracing::{debug, info, warn};
use tracing_subscriber::EnvFilter;

mod common;
mod conv_registry;
mod expression_compiler;
mod field_extractor;
mod file_operations;
mod schemas;
mod strategies;

use field_extractor::FieldExtractor;
use file_operations::create_directories;
use strategies::{GeneratedFile, StrategyDispatcher};

#[derive(Debug, Deserialize, Serialize)]
struct ExifToolModulesConfig {
    description: String,
    modules: ModuleGroups,
}

#[derive(Debug, Deserialize, Serialize)]
struct ModuleGroups {
    core: Vec<String>,
    manufacturer: Vec<String>,
    format: Vec<String>,
}

fn main() -> Result<()> {
    // Initialize tracing with default level of INFO to keep output quiet by default
    // Set RUST_LOG=debug for verbose output
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

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
            Arg::new("modules")
                .long("modules")
                .short('m')
                .help("Specific ExifTool modules to process (e.g., GPS.pm Canon.pm)")
                .value_name("MODULES")
                .action(clap::ArgAction::Append)
                .required(false),
        )
        .get_matches();

    let output_dir = matches.get_one::<String>("output").unwrap();

    // Extract specific modules if provided
    let selected_modules: Option<Vec<String>> = matches
        .get_many::<String>("modules")
        .map(|values| values.map(|s| s.clone()).collect());

    // We're running from the codegen directory
    let current_dir = std::env::current_dir()?;

    // Create output directory
    create_directories(Path::new(output_dir))?;

    info!("🔧 exif-oxide Code Generation");
    debug!("=============================");

    // Universal symbol table extraction is now the default approach
    info!("🔄 Using universal symbol table extraction");
    run_universal_extraction(&current_dir, output_dir, selected_modules.as_ref())?;

    // DISABLED: Process modular tag tables (now handled by CompositeTagStrategy)
    // The strategy-based system with CompositeTagStrategy handles composite tag processing
    //
    // let extract_dir = current_dir.join("generated").join("extract");
    // debug!("📋 Processing composite tags...");
    // let start = Instant::now();
    // process_composite_tags_only(&extract_dir, output_dir)?;
    // info!("📋 Composite tags phase completed in {:.2}s", start.elapsed().as_secs_f64());

    // Tag kit processing is now integrated into the module-based system

    // The old extract.json processing has been removed.
    // All extraction is now handled by the new modular configuration system below.

    // DISABLED: Config-based generation system (replaced by strategy-based system)
    // This legacy system was creating duplicate Canon_pm/ directories alongside the new canon/ directories
    // The strategy-based system in run_universal_extraction() now handles all code generation
    //
    // if selected_modules.is_none() {
    //     // NEW: Process using the new macro-based configuration system
    //     debug!("🔄 Processing new macro-based configuration...");

    //     let config_dir = current_dir.join("config");
    //     let schemas_dir = current_dir.join("schemas");

    //     // Validate all configurations first
    //     if config_dir.exists() && schemas_dir.exists() {
    //         let start = Instant::now();
    //         validate_all_configs(&config_dir, &schemas_dir)?;
    //         debug!("  ✓ Config validation completed in {:.2}s", start.elapsed().as_secs_f64());

    //         // Load all extracted tables with their configurations
    //         let extract_dir = current_dir.join("generated/extract");
    //         let start = Instant::now();
    //         let all_extracted_tables = load_extracted_tables_with_config(&extract_dir, &config_dir)?;
    //         debug!("  ✓ Loaded {} extracted tables in {:.2}s", all_extracted_tables.len(), start.elapsed().as_secs_f64());

    //         // Auto-discover and process each module directory
    //         let start = Instant::now();
    //         discover_and_process_modules(&config_dir, &all_extracted_tables, output_dir)?;
    //         info!("🔄 Module processing phase completed in {:.2}s", start.elapsed().as_secs_f64());

    //         // No macros.rs needed - using direct code generation

    //         // Update the main mod.rs to include new modules
    //         let start = Instant::now();
    //         update_generated_mod_file(output_dir)?;
    //         debug!("  ✓ Updated generated mod.rs in {:.2}s", start.elapsed().as_secs_f64());
    //     } else {
    //         debug!("  ⚠️  New config directory structure not found, using legacy generation only");
    //     }
    // } else {
    //     debug!("🎯 Skipping config-based generation - processing specific modules only");
    // }
    debug!("🎯 Config-based generation disabled - using strategy-based system only");

    // DISABLED: Generate module file (now handled by strategy system)
    // The strategy-based system in StrategyDispatcher::update_main_mod_file() handles this
    //
    // let start = Instant::now();
    // generate_mod_file(output_dir)?;
    // debug!("  ✓ Generated module file in {:.2}s", start.elapsed().as_secs_f64());

    info!("✅ Code generation complete!");

    Ok(())
}

/// Load default modules from exiftool_modules.json config
fn load_default_modules(current_dir: &Path) -> Result<Vec<String>> {
    let config_path = current_dir.join("../config/exiftool_modules.json");

    if !config_path.exists() {
        warn!(
            "⚠️  exiftool_modules.json not found at {}, using fallback modules",
            config_path.display()
        );
        return Ok(vec![
            "GPS.pm".to_string(),
            "DNG.pm".to_string(),
            "Canon.pm".to_string(),
        ]);
    }

    let config_content = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

    let config: ExifToolModulesConfig = serde_json::from_str(&config_content)
        .with_context(|| format!("Failed to parse config file: {}", config_path.display()))?;

    // Combine all module groups
    let mut all_modules = Vec::new();
    all_modules.extend(config.modules.core);
    all_modules.extend(config.modules.manufacturer);
    all_modules.extend(config.modules.format);

    info!(
        "📋 Loaded {} modules from exiftool_modules.json",
        all_modules.len()
    );
    debug!("  Modules: {:?}", all_modules);

    Ok(all_modules)
}

/// Run field extraction with strategy-based processing
fn run_universal_extraction(
    current_dir: &Path,
    output_dir: &str,
    selected_modules: Option<&Vec<String>>,
) -> Result<()> {
    let extractor = FieldExtractor::new();
    let mut dispatcher = StrategyDispatcher::new();
    let exiftool_base_dir = current_dir.join("../third-party/exiftool");

    info!("🔍 Building ExifTool module paths from configuration");

    // Load module paths from configuration
    let default_modules = load_default_modules(current_dir)?;
    let mut module_paths = Vec::new();

    // Convert relative paths from JSON config to absolute paths
    for module_path_str in &default_modules {
        let full_path = exiftool_base_dir.join(module_path_str);
        if full_path.exists() {
            module_paths.push(full_path);
        } else {
            warn!(
                "⚠️  Module path not found: {} (resolved to {})",
                module_path_str,
                full_path.display()
            );
        }
    }

    info!(
        "📦 Found {} ExifTool modules to process",
        module_paths.len()
    );

    // Select modules to process
    let selected_paths: Vec<&std::path::PathBuf> = if let Some(modules) = selected_modules {
        // User specified specific modules - resolve and validate them
        let mut resolved_paths = Vec::new();

        for module_name in modules {
            // Find the module path by filename match
            if let Some(module_path) = module_paths.iter().find(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name == module_name)
                    .unwrap_or(false)
            }) {
                resolved_paths.push(module_path);
            } else {
                return Err(anyhow::anyhow!(
                    "Module not found: {}. Available modules: {:?}",
                    module_name,
                    module_paths
                        .iter()
                        .filter_map(|p| p.file_name()?.to_str())
                        .collect::<Vec<_>>()
                ));
            }
        }

        info!(
            "🎯 Processing {} user-selected modules: {:?}",
            resolved_paths.len(),
            resolved_paths
                .iter()
                .map(|p| p.file_name().unwrap().to_string_lossy())
                .collect::<Vec<_>>()
        );
        resolved_paths
    } else {
        // Process all configured modules (paths already loaded and validated above)
        let all_paths: Vec<&std::path::PathBuf> = module_paths.iter().collect();

        info!(
            "🏭 Processing {} configured modules: {:?}",
            all_paths.len(),
            all_paths
                .iter()
                .map(|p| p.file_name().unwrap().to_string_lossy())
                .collect::<Vec<_>>()
        );
        all_paths
    };

    let start = Instant::now();
    let mut all_symbols = Vec::new();

    // Extract symbols from selected modules
    for module_path in selected_paths {
        match extractor.extract_module(module_path) {
            Ok((symbols, stats)) => {
                info!(
                    "✅ {}: {} symbols extracted from {} total",
                    stats.module_name, stats.extracted_symbols, stats.total_symbols
                );

                debug!(
                    "  📋 {} symbols ready for strategy processing",
                    symbols.len()
                );
                all_symbols.extend(symbols);
            }
            Err(e) => {
                warn!("❌ Failed to extract from {}: {}", module_path.display(), e);
            }
        }
    }

    let extraction_time = start.elapsed();
    info!(
        "🔄 Field extraction completed in {:.2}s",
        extraction_time.as_secs_f64()
    );

    // Process extracted symbols through strategy system
    if !all_symbols.is_empty() {
        let strategy_start = Instant::now();
        info!(
            "🎯 Processing {} symbols through strategy system",
            all_symbols.len()
        );

        match dispatcher.process_symbols(all_symbols, output_dir) {
            Ok(generated_files) => {
                let strategy_time = strategy_start.elapsed();
                info!(
                    "✅ Strategy processing completed in {:.2}s",
                    strategy_time.as_secs_f64()
                );

                // Write generated files to disk
                let write_start = Instant::now();
                write_generated_files(&generated_files, output_dir)?;
                let write_time = write_start.elapsed();

                info!(
                    "📁 {} files written in {:.2}s",
                    generated_files.len(),
                    write_time.as_secs_f64()
                );
                info!(
                    "🏁 Total field extraction time: {:.2}s",
                    (extraction_time + strategy_time + write_time).as_secs_f64()
                );
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
