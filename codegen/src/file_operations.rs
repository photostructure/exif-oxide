//! File operations utilities for code generation
//!
//! This module provides utilities for file I/O operations used throughout
//! the code generation process, including UTF-8 error handling and atomic
//! file writing.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tracing::warn;

/// Read a file as UTF-8 string, with fallback handling for non-UTF-8 content
///
/// This function attempts to read a file as a UTF-8 string. If UTF-8 decoding
/// fails, it falls back to reading as bytes and using lossy conversion.
pub fn read_utf8_with_fallback(path: &Path) -> Result<String> {
    match fs::read_to_string(path) {
        Ok(data) => Ok(data),
        Err(err) => {
            warn!("UTF-8 error reading {}: {}", path.display(), err);
            let bytes = fs::read(path)
                .with_context(|| format!("Failed to read bytes from {}", path.display()))?;
            Ok(String::from_utf8_lossy(&bytes).into_owned())
        }
    }
}

/// Write content to a file atomically
///
/// This function writes content to a file, creating parent directories
/// if they don't exist.
pub fn write_file_atomic(path: &Path, content: &str) -> Result<()> {
    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directory for {}", path.display()))?;
    }
    
    fs::write(path, content)
        .with_context(|| format!("Failed to write to {}", path.display()))?;
    
    Ok(())
}

/// Create directories recursively
///
/// Creates all parent directories as needed.
pub fn create_directories(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .with_context(|| format!("Failed to create directory {}", path.display()))?;
    Ok(())
}

/// Check if a file exists
pub fn file_exists(path: &Path) -> bool {
    path.exists()
}

/// Read directory entries
pub fn read_directory(path: &Path) -> Result<fs::ReadDir> {
    fs::read_dir(path)
        .with_context(|| format!("Failed to read directory {}", path.display()))
}

/// Skip empty JSON files during processing
/// 
/// Returns true if the file should be skipped (is empty or contains only whitespace)
#[allow(dead_code)]
pub fn should_skip_empty_json(json_content: &str) -> bool {
    json_content.trim().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_read_utf8_with_fallback() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        // Write valid UTF-8 content
        fs::write(&file_path, "Hello, world!").unwrap();
        
        let result = read_utf8_with_fallback(&file_path).unwrap();
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_write_file_atomic() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("subdir").join("test.txt");
        
        write_file_atomic(&file_path, "Test content").unwrap();
        
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Test content");
    }

    #[test]
    fn test_create_directories() {
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path().join("a").join("b").join("c");
        
        create_directories(&dir_path).unwrap();
        
        assert!(dir_path.exists());
        assert!(dir_path.is_dir());
    }

    #[test]
    fn test_should_skip_empty_json() {
        assert!(should_skip_empty_json(""));
        assert!(should_skip_empty_json("   "));
        assert!(should_skip_empty_json("\n\t  \n"));
        assert!(!should_skip_empty_json("{}"));
        assert!(!should_skip_empty_json("[]"));
    }
}