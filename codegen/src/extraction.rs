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

fn parse_all_module_configs(module_config_dir: &Path) -> Result<Vec<ModuleConfig>> {
    let mut configs = Vec::new();
    
    // Look for all supported config files
    let config_files = [
        "simple_table.json",
        "file_type_lookup.json", 
        "regex_patterns.json"
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

fn try_parse_module_config(module_config_dir: &Path) -> Result<Option<ModuleConfig>> {
    // Check for any of the supported config files
    let simple_table_config = module_config_dir.join("simple_table.json");
    let file_type_lookup_config = module_config_dir.join("file_type_lookup.json");
    let magic_number_config = module_config_dir.join("magic_number.json");
    
    // Determine which config file exists
    let config_path = if simple_table_config.exists() {
        simple_table_config
    } else if file_type_lookup_config.exists() {
        file_type_lookup_config
    } else if magic_number_config.exists() {
        magic_number_config
    } else {
        return Ok(None);
    };
    
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
    
    let hash_names: Vec<String> = tables.iter()
        .filter_map(|table| table["hash_name"].as_str())
        .map(|name| name.trim_start_matches('%').to_string())
        .collect();
    
    let module_name = Path::new(source_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    Ok(Some(ModuleConfig {
        source_path: source_path.to_string(),
        hash_names,
        module_name,
    }))
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
    
    let hash_names: Vec<String> = tables.iter()
        .filter_map(|table| table["hash_name"].as_str())
        .map(|name| name.trim_start_matches('%').to_string())
        .collect();
    
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
    let repo_root = Path::new("..");
    let module_path = repo_root.join(&config.source_path);
    
    patch_module_if_needed(&module_path, &config.hash_names)?;
    
    // Check if this config needs a special extractor based on the config filename
    match needs_special_extractor_by_name(&config.module_name) {
        Some(SpecialExtractor::FileTypeLookup) => {
            run_file_type_lookup_extractor(config, extract_dir)?;
        }
        Some(SpecialExtractor::RegexPatterns) => {
            run_regex_patterns_extractor(config, extract_dir)?;
        }
        None => {
            run_extraction_script(config, extract_dir)?;
        }
    }
    
    Ok(())
}

fn patch_module_if_needed(module_path: &Path, hash_names: &[String]) -> Result<()> {
    patching::patch_module(module_path, hash_names)
}

fn run_extraction_script(config: &ModuleConfig, extract_dir: &Path) -> Result<()> {
    let hash_names_with_percent: Vec<String> = config.hash_names.iter()
        .map(|name| format!("%{}", name))
        .collect();
    
    // From generated/extract directory, we need ../../../ to get to repo root
    let source_path_for_perl = format!("../../../{}", config.source_path);
    
    let mut cmd = Command::new("perl");
    cmd.arg("../../extractors/simple_table.pl")  // From generated/extract, go up to codegen
       .arg(&source_path_for_perl)
       .args(&hash_names_with_percent)
       .current_dir(extract_dir);
    
    setup_perl_environment(&mut cmd);
    
    println!("    Running: perl simple_table.pl {} {}", 
        source_path_for_perl, 
        hash_names_with_percent.join(" ")
    );
    
    execute_extraction_command(cmd, &config.module_name)
}

fn setup_perl_environment(cmd: &mut Command) {
    let perl5lib = format!(
        "{}:{}",
        std::env::var("HOME").unwrap_or_default() + "/perl5/lib/perl5",
        "../lib:../../third-party/exiftool/lib"
    );
    cmd.env("PERL5LIB", perl5lib);
}

fn execute_extraction_command(mut cmd: Command, module_name: &str) -> Result<()> {
    let output = cmd.output()
        .with_context(|| format!("Failed to execute simple_table.pl for {}", module_name))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("simple_table.pl failed for {}: {}", module_name, stderr));
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

fn needs_special_extractor(config_dir: &Path) -> Option<SpecialExtractor> {
    if config_dir.join("file_type_lookup.json").exists() {
        return Some(SpecialExtractor::FileTypeLookup);
    }
    if config_dir.join("regex_patterns.json").exists() {
        return Some(SpecialExtractor::RegexPatterns);
    }
    None
}

fn needs_special_extractor_by_name(config_name: &str) -> Option<SpecialExtractor> {
    match config_name {
        "file_type_lookup" => Some(SpecialExtractor::FileTypeLookup),
        "regex_patterns" => Some(SpecialExtractor::RegexPatterns),
        _ => None,
    }
}

fn run_file_type_lookup_extractor(config: &ModuleConfig, extract_dir: &Path) -> Result<()> {
    // File type lookup always extracts from %fileTypeLookup hash
    let hash_name = "%fileTypeLookup";
    
    // From generated/extract directory, we need ../../../ to get to repo root
    let source_path_for_perl = format!("../../../{}", config.source_path);
    
    let mut cmd = Command::new("perl");
    cmd.arg("../../extractors/file_type_lookup.pl")  // From generated/extract, go up to codegen
       .arg(&source_path_for_perl)
       .arg(hash_name)
       .current_dir(extract_dir);
    
    setup_perl_environment(&mut cmd);
    
    println!("    Running: perl file_type_lookup.pl {} {}", 
        source_path_for_perl, 
        hash_name
    );
    
    // Redirect output to file_type_lookup.json
    let output_path = extract_dir.join("file_type_lookup.json");
    cmd.stdout(fs::File::create(&output_path)?);
    
    let output = cmd.output()
        .with_context(|| format!("Failed to execute file_type_lookup.pl for {}", config.module_name))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("file_type_lookup.pl failed for {}: {}", config.module_name, stderr));
    }
    
    // Print any stderr output from the script
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        print!("{}", stderr);
    }
    
    println!("    Created file_type_lookup.json");
    
    Ok(())
}

fn run_regex_patterns_extractor(config: &ModuleConfig, extract_dir: &Path) -> Result<()> {
    // Get hash names from config, adding % prefix as needed
    let hash_names_with_percent: Vec<String> = config.hash_names.iter()
        .map(|name| format!("%{}", name))
        .collect();
    
    if hash_names_with_percent.is_empty() {
        return Err(anyhow::anyhow!("No hash names specified in regex_patterns config"));
    }
    
    // From generated/extract directory, we need ../../../ to get to repo root
    let source_path_for_perl = format!("../../../{}", config.source_path);
    
    let mut cmd = Command::new("perl");
    cmd.arg("../../extractors/regex_patterns.pl")  // From generated/extract, go up to codegen
       .arg(&source_path_for_perl)
       .arg(&hash_names_with_percent[0])  // regex_patterns.pl expects single hash name
       .current_dir(extract_dir);
    
    setup_perl_environment(&mut cmd);
    
    println!("    Running: perl regex_patterns.pl {} {}", 
        source_path_for_perl, 
        hash_names_with_percent[0]
    );
    
    // Redirect output to regex_patterns.json  
    let output_path = extract_dir.join("regex_patterns.json");
    cmd.stdout(fs::File::create(&output_path)?);
    
    let output = cmd.output()
        .with_context(|| format!("Failed to execute regex_patterns.pl for {}", config.module_name))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("regex_patterns.pl failed for {}: {}", config.module_name, stderr));
    }
    
    // Print any stderr output (status messages)
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        print!("{}", stderr);
    }
    
    println!("    Created regex_patterns.json");
    
    Ok(())
}