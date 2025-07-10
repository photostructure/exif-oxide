//! ExifTool module patching utilities
//!
//! Handles patching ExifTool modules to expose my-scoped variables as package variables
//! for extraction by our simple table framework.

use anyhow::{Context, Result};
use regex::Regex;
use std::fs;
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
    
    // Read entire file into memory
    let content = fs::read_to_string(module_path)
        .with_context(|| format!("Failed to read {}", module_path.display()))?;
    
    // Process all lines
    let mut patched_lines = Vec::new();
    let mut any_changes = false;
    
    for line in content.lines() {
        let mut patched_line = line.to_string();
        
        // Apply all variable patches to this line
        for (var, pattern) in variables.iter().zip(&patterns) {
            let old_line = patched_line.clone();
            patched_line = pattern.replace(&patched_line, "${1}our${2}").to_string();
            if patched_line != old_line {
                println!("    Converted 'my %{}' to 'our %{}'", var, var);
                any_changes = true;
            }
        }
        
        patched_lines.push(patched_line);
    }
    
    // Only write back if there were changes
    if any_changes {
        let patched_content = patched_lines.join("\n");
        // Ensure file ends with newline
        let final_content = if patched_content.ends_with('\n') {
            patched_content
        } else {
            patched_content + "\n"
        };
        
        fs::write(module_path, final_content)
            .with_context(|| format!("Failed to write {}", module_path.display()))?;
        
        println!("    Patched {}", module_path.display());
    } else {
        println!("    No changes needed for {}", module_path.display());
    }
    
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_patch_module() -> Result<()> {
        // Create a temporary directory and file
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.pm");
        
        // Write test content
        let original_content = r#"#!/usr/bin/perl

# Test file
my %testHash = (
    1 => "One",
    2 => "Two",
);

my %anotherHash = (
    'a' => "Alpha",
    'b' => "Beta",
);

my $scalar = 42;
my %keepThis = ();
"#;
        
        fs::write(&test_file, original_content)?;
        
        // Apply patches
        patch_module(&test_file, &["testHash".to_string(), "anotherHash".to_string()])?;
        
        // Read patched content
        let patched = fs::read_to_string(&test_file)?;
        
        // Verify patches were applied correctly
        assert!(patched.contains("our %testHash ="), "testHash should be converted to our");
        assert!(patched.contains("our %anotherHash ="), "anotherHash should be converted to our");
        assert!(!patched.contains("my %testHash"), "my %testHash should not exist");
        assert!(!patched.contains("my %anotherHash"), "my %anotherHash should not exist");
        assert!(patched.contains("my $scalar"), "scalar should remain unchanged");
        assert!(patched.contains("my %keepThis"), "keepThis should remain unchanged");
        
        Ok(())
    }
    
    #[test]
    fn test_patch_module_empty_variables() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.pm");
        fs::write(&test_file, "# Empty file")?;
        
        // Should succeed with empty variables list
        patch_module(&test_file, &[])?;
        
        Ok(())
    }
    
    #[test]
    fn test_patch_module_idempotent() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.pm");
        
        let content = "my %testHash = (1 => 'One');\n";
        fs::write(&test_file, content)?;
        
        // First patch
        patch_module(&test_file, &["testHash".to_string()])?;
        let first_patched = fs::read_to_string(&test_file)?;
        
        // Second patch (should be idempotent)
        patch_module(&test_file, &["testHash".to_string()])?;
        let second_patched = fs::read_to_string(&test_file)?;
        
        assert_eq!(first_patched, second_patched, "Patching should be idempotent");
        assert!(first_patched.contains("our %testHash"));
        
        Ok(())
    }
}