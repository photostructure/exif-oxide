//! Module discovery for code generation
//!
//! This module handles automatic discovery of module directories and
//! orchestrates the processing of each module's configuration.

use anyhow::{Context, Result};
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;
use tracing::{debug, info};

use crate::file_operations::read_directory;
use crate::generators::lookup_tables;
use crate::schemas::ExtractedTable;

/// Discover and process all module directories
///
/// This function auto-discovers module directories (ending with "_pm") and
/// processes each one using the lookup_tables system.
pub fn discover_and_process_modules(
    config_dir: &Path,
    all_extracted_tables: &HashMap<String, ExtractedTable>,
    output_dir: &str,
) -> Result<()> {
    let discovery_start = Instant::now();
    let config_entries = read_directory(config_dir)
        .context("Failed to read config directory")?;
    
    let mut module_dirs = Vec::new();
    
    // First pass: collect all module directories
    for entry in config_entries {
        let entry = entry.context("Failed to read directory entry")?;
        let module_config_dir = entry.path();
        
        // Skip files, only process directories
        if !module_config_dir.is_dir() {
            continue;
        }
        
        // Skip hidden directories
        if let Some(dir_name) = module_config_dir.file_name() {
            if dir_name.to_string_lossy().starts_with('.') {
                continue;
            }
            let module_name = dir_name.to_string_lossy().to_string();
            module_dirs.push((module_config_dir, module_name));
        }
    }
    
    info!("🔍 Module discovery completed in {:.3}s - found {} modules to process", 
          discovery_start.elapsed().as_secs_f64(), module_dirs.len());
    
    // Second pass: process each module in parallel for maximum performance
    let total_generation_time = Mutex::new(0.0);
    let generation_stats: Mutex<HashMap<String, f64>> = Mutex::new(HashMap::new());
    
    let parallel_start = Instant::now();
    info!("🚀 Starting parallel module processing with {} threads", rayon::current_num_threads());
    
    module_dirs.par_iter().try_for_each(|(module_config_dir, module_name)| -> Result<()> {
        debug!("  Processing module: {}", module_name);
        
        let start = Instant::now();
        lookup_tables::process_config_directory(
            module_config_dir,
            module_name,
            all_extracted_tables,
            output_dir,
        )?;
        let elapsed = start.elapsed().as_secs_f64();
        
        // Update shared stats (with locking)
        {
            let mut total_time = total_generation_time.lock().unwrap();
            *total_time += elapsed;
        }
        
        {
            let mut stats = generation_stats.lock().unwrap();
            let module_type = if module_name.ends_with("_pm") { &module_name[..module_name.len()-3] } else { module_name };
            *stats.entry(module_type.to_string()).or_insert(0.0) += elapsed;
        }
        
        info!("    🏭 Module {} code generation completed in {:.3}s", module_name, elapsed);
        Ok(())
    })?;
    
    let parallel_elapsed = parallel_start.elapsed().as_secs_f64();
    let total_generation_time = *total_generation_time.lock().unwrap();
    let generation_stats = generation_stats.into_inner().unwrap();
    
    // Summary statistics
    info!("🏭 CODE GENERATION PHASE SUMMARY:");
    info!("  Parallel wall time: {:.3}s (vs {:.3}s sequential)", parallel_elapsed, total_generation_time);
    info!("  Speedup: {:.1}x with {} threads", total_generation_time / parallel_elapsed, rayon::current_num_threads());
    info!("  Total CPU time: {:.3}s", total_generation_time);
    info!("  Average per module: {:.3}s", total_generation_time / generation_stats.len() as f64);
    for (module_type, time) in generation_stats {
        info!("  {}: {:.3}s", module_type, time);
    }
    info!("  Overall generation phase: {:.3}s", discovery_start.elapsed().as_secs_f64());
    
    Ok(())
}

/// Generate module file structure
///
/// This function updates the main generated mod.rs file to include
/// all discovered modules.
pub fn update_generated_mod_file(output_dir: &str) -> Result<()> {
    use crate::file_operations::{file_exists, write_file_atomic};
    use std::fs;
    
    let mod_path = format!("{output_dir}/mod.rs");
    let mut content = if file_exists(Path::new(&mod_path)) {
        fs::read_to_string(&mod_path)?
    } else {
        String::new()
    };

    // Auto-discover module directories (any directory ending in _pm)
    let entries = read_directory(Path::new(output_dir))?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                if dir_name.ends_with("_pm") && path.join("mod.rs").exists() {
                    let mod_declaration = format!("pub mod {dir_name};\n");
                    if !content.contains(&mod_declaration) {
                        content.push_str(&mod_declaration);
                    }
                }
            }
        }
    }

    write_file_atomic(Path::new(&mod_path), &content)?;
    Ok(())
}

/// Discover all module directories ending with "_pm"
///
/// Returns a sorted list of module directory names for consistent processing.
#[allow(dead_code)]
pub fn discover_module_directories(config_dir: &Path) -> Result<Vec<String>> {
    let mut modules = Vec::new();
    
    if !config_dir.exists() {
        return Ok(modules);
    }
    
    let entries = read_directory(config_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                if dir_name.ends_with("_pm") {
                    modules.push(dir_name.to_string());
                }
            }
        }
    }
    
    modules.sort(); // For consistent output
    Ok(modules)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_discover_module_directories() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path();
        
        // Create some module directories
        fs::create_dir_all(config_dir.join("Canon_pm")).unwrap();
        fs::create_dir_all(config_dir.join("Nikon_pm")).unwrap();
        fs::create_dir_all(config_dir.join("not_a_module")).unwrap();
        fs::create_dir_all(config_dir.join("ExifTool_pm")).unwrap();
        
        let modules = discover_module_directories(config_dir).unwrap();
        
        // Should be sorted
        assert_eq!(modules, vec!["Canon_pm", "ExifTool_pm", "Nikon_pm"]);
    }

    #[test]
    fn test_discover_module_directories_empty() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().join("nonexistent");
        
        let modules = discover_module_directories(&config_dir).unwrap();
        assert!(modules.is_empty());
    }

    #[test]
    fn test_discover_module_directories_no_modules() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path();
        
        // Create some non-module directories
        fs::create_dir_all(config_dir.join("not_a_module")).unwrap();
        fs::create_dir_all(config_dir.join("another_dir")).unwrap();
        
        let modules = discover_module_directories(config_dir).unwrap();
        assert!(modules.is_empty());
    }
}