//! ExifTool module patching utilities
//!
//! Handles patching ExifTool modules to expose my-scoped variables as package variables
//! for extraction by our simple table framework.

use anyhow::{Context, Result};
use regex::Regex;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::Command;

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
    
    let temp_path = module_path.with_extension("tmp");
    let mut output = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&temp_path)
        .with_context(|| format!("Failed to create temp file {}", temp_path.display()))?;
    
    for line in reader.lines() {
        let mut line = line.with_context(|| format!("Failed to read line from {}", module_path.display()))?;
        
        // Apply all variable patches to this line
        for (var, pattern) in variables.iter().zip(&patterns) {
            let old_line = line.clone();
            line = pattern.replace(&line, "${1}our${2}").to_string();
            if line != old_line {
                println!("    Converted 'my %{}' to 'our %{}'", var, var);
            }
        }
        
        writeln!(output, "{}", line)?;
    }
    
    drop(output); // Ensure file is closed before rename
    
    // Atomically replace the original file
    std::fs::rename(&temp_path, module_path)
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