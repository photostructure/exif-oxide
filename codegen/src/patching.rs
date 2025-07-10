//! ExifTool module patching utilities
//!
//! Handles patching ExifTool modules to expose my-scoped variables as package variables
//! for extraction by our simple table framework.

use anyhow::{Context, Result};
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::Command;
use tempfile::NamedTempFile;

/// Patch ExifTool module to convert my-scoped variables to package variables
/// This operation is idempotent - safe to run multiple times
pub fn patch_module(module_path: &Path, variables: &[String]) -> Result<()> {
    if variables.is_empty() {
        return Ok(());
    }
    
    println!("  Patching {} for variables: {}", 
        module_path.display(), 
        variables.join(", ")
    );
    
    // Compile regex patterns once
    let patterns: Vec<_> = variables.iter()
        .map(|var| {
            let pattern = format!(r"^(\s*)my(\s+%{}\s*=)", regex::escape(var));
            Regex::new(&pattern)
        })
        .collect::<Result<Vec<_>, _>>()?;
    
    // Stream process the file
    let input = File::open(module_path)
        .with_context(|| format!("Failed to open {}", module_path.display()))?;
    let reader = BufReader::new(input);
    
    // Create temp file in the same directory as the target to ensure same filesystem
    let parent_dir = module_path.parent()
        .ok_or_else(|| anyhow::anyhow!("Module path has no parent directory"))?;
    let mut temp_file = NamedTempFile::new_in(parent_dir)
        .with_context(|| format!("Failed to create temp file in {}", parent_dir.display()))?;
    
    for line_result in reader.lines() {
        let mut line = match line_result {
            Ok(line) => line,
            Err(err) => {
                // Skip lines with invalid UTF-8 (e.g., camera names in various encodings)
                eprintln!("    Warning: Skipping line with invalid UTF-8 in {}: {}", 
                    module_path.display(), err);
                continue;
            }
        };
        
        // Apply all variable patches to this line
        for (var, pattern) in variables.iter().zip(&patterns) {
            let old_line = line.clone();
            line = pattern.replace(&line, "${1}our${2}").to_string();
            if line != old_line {
                println!("    Converted 'my %{}' to 'our %{}'", var, var);
            }
        }
        
        writeln!(temp_file, "{}", line)?;
    }
    
    // Atomically replace the original file using tempfile's persist method
    temp_file.persist(module_path)
        .with_context(|| format!("Failed to replace {}", module_path.display()))?;
    
    println!("    Patched {}", module_path.display());
    Ok(())
}

/// Revert all ExifTool module patches to keep submodule clean
pub fn revert_patches() -> Result<()> {
    println!("🔄 Reverting ExifTool module patches...");
    
    let output = Command::new("git")
        .args(&["-C", "../third-party/exiftool", "checkout", "--", 
               "lib/Image/*.pm", "lib/Image/ExifTool/*.pm"])
        .output()
        .context("Failed to execute git checkout")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Git checkout failed: {}", stderr));
    }
    
    println!("  ✓ ExifTool modules reverted to original state");
    Ok(())
}