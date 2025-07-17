//! Simple table extraction orchestration
//!
//! Handles auto-discovery of configs and orchestration of Perl extraction scripts.

use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;
use tracing::debug;

use crate::patching;

// Constants for path navigation
const REPO_ROOT_FROM_CODEGEN: &str = "..";
const CODEGEN_FROM_EXTRACT: &str = "../..";
const REPO_ROOT_FROM_EXTRACT: &str = "../../..";

#[derive(Debug)]
pub struct ModuleConfig {
    pub source_path: String,
    pub hash_names: Vec<String>,
    pub module_name: String,
}

#[derive(Debug)]
enum SpecialExtractor {
    FileTypeLookup,
    RegexPatterns,
    BooleanSet,
    InlinePrintConv,
}

#[derive(Debug)]
struct ExtractorConfig<'a> {
    script_name: &'a str,
    output_file: Option<&'a str>,
    hash_args: Vec<String>,
}

/// Extract all simple tables using Rust orchestration (replaces Makefile targets)
pub fn extract_all_simple_tables() -> Result<()> {
    println!("\n📊 Extracting simple lookup tables...");
    
    let extract_dir = Path::new("generated/extract");
    fs::create_dir_all(extract_dir)?;
    
    let configs = discover_module_configs()?;
    
    for config in configs {
        process_module_config(&config, extract_dir)?;
    }
    
    println!("  ✓ Simple table extraction complete");
    Ok(())
}

fn discover_module_configs() -> Result<Vec<ModuleConfig>> {
    let config_dir = Path::new("config");
    let mut configs = Vec::new();
    
    for entry in fs::read_dir(config_dir)? {
        let entry = entry?;
        let module_config_dir = entry.path();
        
        if should_skip_directory(&module_config_dir) {
            continue;
        }
        
        // Process all configs in this module directory
        let module_configs = parse_all_module_configs(&module_config_dir)?;
        configs.extend(module_configs);
    }
    
    Ok(configs)
}

fn should_skip_directory(path: &Path) -> bool {
    !path.is_dir() || 
    path.file_name()
        .and_then(|name| name.to_str())
        .map_or(true, |name| name.starts_with('.'))
}

/// Parse all config files in a module directory.
/// 
/// A module can have multiple config types (simple_table.json, file_type_lookup.json, etc.)
/// and this function collects all of them, allowing for multi-config modules like:
/// 
/// ```
/// ExifTool_pm/
/// ├── simple_table.json      # Basic lookup tables
/// ├── file_type_lookup.json  # File type detection
/// └── boolean_set.json       # Boolean set membership
/// ```
fn parse_all_module_configs(module_config_dir: &Path) -> Result<Vec<ModuleConfig>> {
    let mut configs = Vec::new();
    
    // Look for all supported config files
    let config_files = [
        "simple_table.json",
        "file_type_lookup.json", 
        "regex_patterns.json",
        "boolean_set.json",
        "inline_printconv.json"
    ];
    
    for config_file in &config_files {
        let config_path = module_config_dir.join(config_file);
        if config_path.exists() {
            if let Some(config) = try_parse_single_config(&config_path)? {
                configs.push(config);
            }
        }
    }
    
    Ok(configs)
}


fn try_parse_single_config(config_path: &Path) -> Result<Option<ModuleConfig>> {
    debug!("Reading config file: {}", config_path.display());
    let config_content = match fs::read_to_string(&config_path) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Warning: UTF-8 error reading {}: {}", config_path.display(), err);
            let bytes = fs::read(&config_path)
                .with_context(|| format!("Failed to read bytes from {}", config_path.display()))?;
            String::from_utf8_lossy(&bytes).into_owned()
        }
    };
    
    let config: Value = serde_json::from_str(&config_content)
        .with_context(|| format!("Failed to parse {}", config_path.display()))?;
    
    let source_path = config["source"].as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing 'source' field in {}", config_path.display()))?;
    
    let tables = config["tables"].as_array()
        .ok_or_else(|| anyhow::anyhow!("Missing 'tables' field in {}", config_path.display()))?;
    
    if tables.is_empty() {
        return Ok(None);
    }
    
    // For inline_printconv, we look for table_name instead of hash_name
    let is_inline_printconv = config_path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n == "inline_printconv.json")
        .unwrap_or(false);
    
    let hash_names: Vec<String> = if is_inline_printconv {
        // For inline PrintConv, we extract table names
        tables.iter()
            .filter_map(|table| table["table_name"].as_str())
            .map(|name| name.to_string())
            .collect()
    } else {
        // For other configs, we extract hash names
        tables.iter()
            .filter_map(|table| table["hash_name"].as_str())
            .map(|name| name.trim_start_matches('%').to_string())
            .collect()
    };
    
    if hash_names.is_empty() {
        return Ok(None);
    }
    
    let module_name = config_path.file_stem()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid config filename: {}", config_path.display()))?
        .to_string();
    
    Ok(Some(ModuleConfig {
        source_path: source_path.to_string(),
        hash_names,
        module_name,
    }))
}

fn process_module_config(config: &ModuleConfig, extract_dir: &Path) -> Result<()> {
    println!("  📷 Processing {} tables...", config.module_name);
    
    // Resolve source path relative to repo root (one level up from codegen/)
    let repo_root = Path::new(REPO_ROOT_FROM_CODEGEN);
    let module_path = repo_root.join(&config.source_path);
    
    // Only patch if we're extracting hashes (not for inline_printconv)
    if config.module_name != "inline_printconv" {
        patching::patch_module(&module_path, &config.hash_names)?;
    }
    
    // Check if this config needs a special extractor based on the config filename
    match needs_special_extractor_by_name(&config.module_name) {
        Some(SpecialExtractor::FileTypeLookup) => {
            run_file_type_lookup_extractor(config, extract_dir)?;
        }
        Some(SpecialExtractor::RegexPatterns) => {
            run_regex_patterns_extractor(config, extract_dir)?;
        }
        Some(SpecialExtractor::BooleanSet) => {
            run_boolean_set_extractor(config, extract_dir)?;
        }
        Some(SpecialExtractor::InlinePrintConv) => {
            run_inline_printconv_extractor(config, extract_dir)?;
        }
        None => {
            run_extraction_script(config, extract_dir)?;
        }
    }
    
    Ok(())
}


fn run_extraction_script(config: &ModuleConfig, extract_dir: &Path) -> Result<()> {
    let hash_names_with_percent: Vec<String> = config.hash_names.iter()
        .map(|name| format!("%{}", name))
        .collect();
    
    let extractor_config = ExtractorConfig {
        script_name: "simple_table.pl",
        output_file: None,
        hash_args: hash_names_with_percent,
    };
    
    run_extractor(config, extract_dir, extractor_config)
}

fn setup_perl_environment(cmd: &mut Command) {
    let perl5lib = format!(
        "{}:{}",
        std::env::var("HOME").unwrap_or_default() + "/perl5/lib/perl5",
        "../lib:../../third-party/exiftool/lib"
    );
    cmd.env("PERL5LIB", perl5lib);
}

/// Common extraction function that handles all extractor types
fn run_extractor(config: &ModuleConfig, extract_dir: &Path, extractor_config: ExtractorConfig) -> Result<()> {
    let source_path_for_perl = format!("{}/{}", REPO_ROOT_FROM_EXTRACT, config.source_path);
    
    let mut cmd = Command::new("perl");
    cmd.arg(format!("{}/extractors/{}", CODEGEN_FROM_EXTRACT, extractor_config.script_name))
       .arg(&source_path_for_perl)
       .args(&extractor_config.hash_args)
       .current_dir(extract_dir);
    
    setup_perl_environment(&mut cmd);
    
    println!("    Running: perl {} {} {}", 
        extractor_config.script_name,
        source_path_for_perl, 
        extractor_config.hash_args.join(" ")
    );
    
    // Redirect output to file if specified
    if let Some(output_file) = extractor_config.output_file {
        let output_path = extract_dir.join(output_file);
        cmd.stdout(fs::File::create(&output_path)?);
    }
    
    execute_extraction_command(cmd, &config.module_name, extractor_config.script_name)?;
    
    if let Some(output_file) = extractor_config.output_file {
        println!("    Created {}", output_file);
    }
    
    Ok(())
}

fn execute_extraction_command(mut cmd: Command, module_name: &str, script_name: &str) -> Result<()> {
    let output = cmd.output()
        .with_context(|| format!("Failed to execute {} for {}", script_name, module_name))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("{} failed for {}: {}", script_name, module_name, stderr));
    }
    
    // Print any output from the script
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    if !stderr.is_empty() {
        print!("{}", stderr);
    }
    if !stdout.is_empty() {
        print!("{}", stdout);
    }
    
    Ok(())
}


fn needs_special_extractor_by_name(config_name: &str) -> Option<SpecialExtractor> {
    match config_name {
        "file_type_lookup" => Some(SpecialExtractor::FileTypeLookup),
        "regex_patterns" => Some(SpecialExtractor::RegexPatterns),
        "boolean_set" => Some(SpecialExtractor::BooleanSet),
        "inline_printconv" => Some(SpecialExtractor::InlinePrintConv),
        _ => None,
    }
}

fn run_file_type_lookup_extractor(config: &ModuleConfig, extract_dir: &Path) -> Result<()> {
    let extractor_config = ExtractorConfig {
        script_name: "file_type_lookup.pl",
        output_file: Some("file_type_lookup.json"),
        hash_args: vec!["%fileTypeLookup".to_string()],
    };
    
    run_extractor(config, extract_dir, extractor_config)
}

fn run_regex_patterns_extractor(config: &ModuleConfig, extract_dir: &Path) -> Result<()> {
    // Get hash names from config, adding % prefix as needed
    let hash_names_with_percent: Vec<String> = config.hash_names.iter()
        .map(|name| format!("%{}", name))
        .collect();
    
    if hash_names_with_percent.is_empty() {
        return Err(anyhow::anyhow!("No hash names specified in regex_patterns config"));
    }
    
    let extractor_config = ExtractorConfig {
        script_name: "regex_patterns.pl",
        output_file: Some("regex_patterns.json"),
        hash_args: vec![hash_names_with_percent[0].clone()], // regex_patterns.pl expects single hash name
    };
    
    run_extractor(config, extract_dir, extractor_config)
}

fn run_boolean_set_extractor(config: &ModuleConfig, extract_dir: &Path) -> Result<()> {
    // Extract each boolean set separately since they need individual output files
    for hash_name in &config.hash_names {
        let hash_name_with_percent = format!("%{}", hash_name);
        let output_file = format!("boolean_set_{}.json", hash_name);
        
        let extractor_config = ExtractorConfig {
            script_name: "boolean_set.pl",
            output_file: Some(&output_file),
            hash_args: vec![hash_name_with_percent],
        };
        
        run_extractor(config, extract_dir, extractor_config)?;
    }
    
    Ok(())
}

fn run_inline_printconv_extractor(config: &ModuleConfig, extract_dir: &Path) -> Result<()> {
    // For inline PrintConv, we need to pass table names from the config
    // The config should specify which tables to extract from
    let tables = config.hash_names.clone(); // These are actually table names for inline_printconv
    
    if tables.is_empty() {
        return Err(anyhow::anyhow!("No tables specified in inline_printconv config"));
    }
    
    // Extract each table's inline PrintConv definitions
    for table_name in &tables {
        let extractor_config = ExtractorConfig {
            script_name: "inline_printconv.pl",
            output_file: None, // Output file will be created by the script
            hash_args: vec![table_name.clone()],
        };
        
        run_extractor(config, extract_dir, extractor_config)?;
    }
    
    Ok(())
}

/// Extract tag definitions (tag tables and composite tags)
/// These are global extractions that don't use the config system
pub fn extract_tag_definitions() -> Result<()> {
    println!("\n📏 Extracting tag definitions...");
    
    let generated_dir = Path::new("generated");
    fs::create_dir_all(generated_dir)?;
    
    // Extract tag tables
    println!("  📋 Extracting tag tables...");
    run_tag_tables_extractor(generated_dir)?;
    
    // Extract composite tags
    println!("  🔗 Extracting composite tags...");
    run_composite_tags_extractor(generated_dir)?;
    
    println!("  ✓ Tag definition extraction complete");
    Ok(())
}

fn run_tag_tables_extractor(output_dir: &Path) -> Result<()> {
    let output_file = output_dir.join("tag_tables.json");
    
    let mut cmd = Command::new("perl");
    cmd.arg("extractors/tag_tables.pl")
       .stdout(fs::File::create(&output_file)?);
    
    setup_perl_environment(&mut cmd);
    
    println!("    Running: perl tag_tables.pl > tag_tables.json");
    
    let output = cmd.output()
        .with_context(|| "Failed to execute tag_tables.pl")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("tag_tables.pl failed: {}", stderr));
    }
    
    // Print any stderr output (progress messages)
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        print!("{}", stderr);
    }
    
    println!("    Created tag_tables.json");
    Ok(())
}

fn run_composite_tags_extractor(output_dir: &Path) -> Result<()> {
    let output_file = output_dir.join("composite_tags.json");
    
    let mut cmd = Command::new("perl");
    cmd.arg("extractors/composite_tags.pl")
       .stdout(fs::File::create(&output_file)?);
    
    setup_perl_environment(&mut cmd);
    
    println!("    Running: perl composite_tags.pl > composite_tags.json");
    
    let output = cmd.output()
        .with_context(|| "Failed to execute composite_tags.pl")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("composite_tags.pl failed: {}", stderr));
    }
    
    // Print any stderr output (progress messages)
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        print!("{}", stderr);
    }
    
    println!("    Created composite_tags.json");
    Ok(())
}