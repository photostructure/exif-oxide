//! Rust code generation tool for exif-oxide (modularized version)
//!
//! This tool reads JSON output from Perl extractors and generates
//! Rust code following the "runtime references, no stubs" architecture.

use anyhow::{Context, Result};
use clap::{Arg, Command};
use indoc::formatdoc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Instant;
use tracing::{debug, info, warn};
use tracing_subscriber::EnvFilter;

mod common;
mod field_extractor;
mod file_operations;
mod impl_registry;
mod ppi; // PPI JSON parsing for codegen-time AST processing
mod schemas;
mod strategies;

use field_extractor::FieldExtractor;
use file_operations::create_directories;
use strategies::StrategyDispatcher;

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

    let output_dir_raw = matches.get_one::<String>("output").unwrap();

    // Resolve output directory to absolute path to avoid nested directory issues
    let output_dir = std::fs::canonicalize(Path::new(output_dir_raw))
        .or_else(|_| {
            // If canonicalize fails (e.g., directory doesn't exist), create it first
            create_directories(Path::new(output_dir_raw))
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            std::fs::canonicalize(Path::new(output_dir_raw))
        })
        .with_context(|| format!("Failed to resolve output directory: {}", output_dir_raw))?
        .to_string_lossy()
        .to_string();

    // Extract specific modules if provided
    let selected_modules: Option<Vec<String>> = matches
        .get_many::<String>("modules")
        .map(|values| values.map(|s| s.clone()).collect());

    // We're running from the codegen directory
    let current_dir = std::env::current_dir()?;

    // Output directory should already exist from canonicalize above
    create_directories(Path::new(&output_dir))?;

    info!("🔧 exif-oxide Code Generation");
    debug!("=============================");

    // Universal symbol table extraction is now the default approach
    info!("🔄 Using universal symbol table extraction");
    run_universal_extraction(&current_dir, &output_dir, selected_modules.as_ref())?;

    info!("✅ Code generation complete!");

    Ok(())
}

/// Load default modules from exiftool_modules.json config
fn load_default_modules(_current_dir: &Path) -> Result<Vec<String>> {
    let config_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../config/exiftool_modules.json");
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
    let exiftool_base_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../third-party/exiftool");

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
            Ok(symbols) => {
                let module_name = module_path
                    .file_stem()
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown")
                    .strip_suffix(".pm")
                    .unwrap_or_else(|| {
                        module_path
                            .file_stem()
                            .and_then(|name| name.to_str())
                            .unwrap_or("unknown")
                    });

                info!("✅ {}: {} symbols extracted", module_name, symbols.len());

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
                    "✅ Strategy processing completed in {:.2}s ({} files generated)",
                    strategy_time.as_secs_f64(),
                    generated_files.len()
                );

                // Generate mod.rs files after files are written to disk
                let mod_update_start = Instant::now();
                info!("📄 Updating mod.rs files after file generation");
                update_mod_files(output_dir)?;
                let mod_update_time = mod_update_start.elapsed();

                info!(
                    "📝 mod.rs files updated in {:.2}s",
                    mod_update_time.as_secs_f64()
                );
                info!(
                    "🏁 Total field extraction time: {:.2}s",
                    (extraction_time + strategy_time + mod_update_time).as_secs_f64()
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

/// Update all mod.rs files by scanning the filesystem after files are written
fn update_mod_files(output_dir: &str) -> Result<()> {
    use std::collections::{BTreeSet, HashMap};
    use std::fs;
    use std::path::Path;

    // Scan filesystem directly to find all module directories and their .rs files
    let mut modules_with_files = HashMap::new();

    // Read all entries in the output directory
    for entry in fs::read_dir(output_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Only process directories (skip files like composite_tags.rs)
        if path.is_dir() {
            let module_name = path.file_name().unwrap().to_string_lossy().to_string();

            // Skip non-module directories, but allow file_types even without mod.rs
            // (we'll create it below)

            // Find all .rs files in this module directory
            let mut file_set = BTreeSet::new();
            for module_file in fs::read_dir(&path)? {
                let module_file = module_file?;
                let file_path = module_file.path();

                // Only include .rs files, excluding mod.rs itself
                if file_path.extension().map_or(false, |ext| ext == "rs") {
                    if let Some(filename) = file_path.file_stem() {
                        let filename_str = filename.to_string_lossy().to_string();
                        if filename_str != "mod" {
                            file_set.insert(filename_str);
                        }
                    }
                }
            }

            if !file_set.is_empty() {
                modules_with_files.insert(module_name, file_set);
            }
        }
    }

    info!(
        "📝 Creating mod.rs files for {} modules (filesystem scan)",
        modules_with_files.len()
    );

    // Create mod.rs files for each module directory
    for (module_dir, file_set) in &modules_with_files {
        let module_dir_path = Path::new(output_dir).join(module_dir);
        let module_mod_path = module_dir_path.join("mod.rs");

        // Ensure the module directory exists
        if let Err(e) = fs::create_dir_all(&module_dir_path) {
            warn!(
                "Failed to create directory {}: {}",
                module_dir_path.display(),
                e
            );
            continue;
        }

        let mut content = formatdoc! {"
            //! Generated module for {}
            //!
            //! This file is auto-generated by codegen/src/main.rs. Do not edit manually.

            ",
            module_dir.strip_suffix("_pm").unwrap_or(module_dir)
        };

        // Files are already deduplicated and sorted from BTreeSet
        // Generate pub mod declarations
        for filename in file_set {
            content.push_str(&format!("pub mod {};\n", filename));
        }

        content.push_str("\n// Re-export commonly used items\n");

        // Always re-export main_tags if it exists
        if file_set.contains("main_tags") {
            // Read the actual constant name from the generated main_tags.rs file
            let main_tags_path = module_dir_path.join("main_tags.rs");
            if let Ok(main_tags_content) = fs::read_to_string(&main_tags_path) {
                // Look for the pattern "pub static CONSTANT_NAME: LazyLock"
                if let Some(start) = main_tags_content.find("pub static ") {
                    // Find the colon after the constant name (more robust pattern matching)
                    if let Some(colon_pos) = main_tags_content[start..].find(':') {
                        let constant_def = &main_tags_content[start..start + colon_pos];
                        if let Some(const_name) = constant_def.strip_prefix("pub static ") {
                            // Trim any whitespace from the constant name
                            let const_name = const_name.trim();
                            debug!(
                                "Found MAIN_TAGS constant: '{}' in module {}",
                                const_name, module_dir
                            );
                            content.push_str(&format!("pub use main_tags::{};\n", const_name));
                        } else {
                            debug!(
                                "Failed to parse constant name from: '{}' in module {}",
                                constant_def, module_dir
                            );
                            // Fallback: Use module-prefixed name based on module directory
                            let module_upper = module_dir.to_uppercase().replace('-', "_");
                            let expected_const = format!("{}_MAIN_TAGS", module_upper);
                            debug!("Using fallback constant name: {}", expected_const);
                            content.push_str(&format!("pub use main_tags::{};\n", expected_const));
                        }
                    } else {
                        debug!("No colon found after 'pub static' in module {}", module_dir);
                        // Fallback: Use module-prefixed name
                        let module_upper = module_dir.to_uppercase().replace('-', "_");
                        let expected_const = format!("{}_MAIN_TAGS", module_upper);
                        content.push_str(&format!("pub use main_tags::{};\n", expected_const));
                    }
                } else {
                    debug!(
                        "No 'pub static' found in main_tags.rs for module {}",
                        module_dir
                    );
                    // Fallback: Use module-prefixed name
                    let module_upper = module_dir.to_uppercase().replace('-', "_");
                    let expected_const = format!("{}_MAIN_TAGS", module_upper);
                    content.push_str(&format!("pub use main_tags::{};\n", expected_const));
                }
            } else {
                debug!("Failed to read main_tags.rs for module {}", module_dir);
                // Fallback: Use module-prefixed name
                let module_upper = module_dir.to_uppercase().replace('-', "_");
                let expected_const = format!("{}_MAIN_TAGS", module_upper);
                content.push_str(&format!("pub use main_tags::{};\n", expected_const));
            }
        }

        if let Err(e) = fs::write(&module_mod_path, content) {
            return Err(anyhow::anyhow!(
                "Failed to write mod.rs file for module '{}' at path '{}': {}",
                module_dir,
                module_mod_path.display(),
                e
            ));
        }
        debug!(
            "📄 Created mod.rs for {} with {} files",
            module_dir,
            file_set.len()
        );
    }

    // Completely regenerate main src/generated/mod.rs based purely on generated modules
    let main_mod_path = Path::new(output_dir).join("mod.rs");
    let mut main_content = formatdoc! {"
        //! Generated code module
        //!
        //! This file is auto-generated by codegen/src/main.rs. Do not edit manually.
        //!
        //! This module re-exports all generated code for easy access.

        "};

    // Collect all modules in a BTreeSet for deterministic, sorted output
    let mut all_modules = BTreeSet::new();

    // Add all generated modules
    for module_dir in modules_with_files.keys() {
        all_modules.insert(module_dir.clone());
    }
    if Path::new(output_dir).join("composite_tags.rs").exists() {
        all_modules.insert("composite_tags".to_string());
    }
    if Path::new(output_dir).join("supported_tags.rs").exists() {
        all_modules.insert("supported_tags".to_string());
    }
    if Path::new(output_dir).join("functions").is_dir() {
        all_modules.insert("functions".to_string());
    }

    // Generate module declarations in sorted order
    for module_dir in &all_modules {
        // Add special attribute for non-snake-case modules
        main_content.push_str(&format!("pub mod {};\n", module_dir));
    }

    main_content.push('\n');

    // Add standard re-exports (only if modules exist)
    main_content.push_str("// Re-export commonly used types and functions\n");
    if Path::new(output_dir).join("supported_tags.rs").exists() {
        main_content.push_str(&formatdoc! {"
            pub use supported_tags::{{
                SUPPORTED_TAG_COUNT, SUPPORTED_COMPOSITE_TAG_COUNT, TOTAL_SUPPORTED_TAG_COUNT,
                SUPPORTED_TAG_NAMES, SUPPORTED_COMPOSITE_TAG_NAMES,
                tag_counts_by_group, supported_tag_summary
            }};
            "});
    }
    if Path::new(output_dir).join("composite_tags.rs").exists() {
        main_content.push_str("pub use composite_tags::{CompositeTagDef, COMPOSITE_TAGS, lookup_composite_tag, all_composite_tag_names, composite_tag_count};\n");
    }

    main_content.push_str(&formatdoc! {"

        /// Initialize all lazy static data structures
        /// This can be called during startup to avoid lazy initialization costs later
        pub fn initialize_all() {{
        }}
        "});

    let modules_added = all_modules.len();

    if let Err(e) = fs::write(&main_mod_path, main_content) {
        return Err(anyhow::anyhow!(
            "Failed to write main mod.rs file at path '{}': {}",
            main_mod_path.display(),
            e
        ));
    }
    info!("📋 Updated main mod.rs with {} new modules", modules_added);

    Ok(())
}
