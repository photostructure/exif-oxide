//! File operations utilities for code generation
//!
//! This module provides utilities for file I/O operations used throughout
//! the code generation process, including UTF-8 error handling and atomic
//! file writing.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Create directories recursively
///
/// Creates all parent directories as needed.
pub fn create_directories(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .with_context(|| format!("Failed to create directory {}", path.display()))?;
    Ok(())
}

/// Skip empty JSON files during processing
///
/// Returns true if the file should be skipped (is empty or contains only whitespace)
#[allow(dead_code)]
pub fn should_skip_empty_json(json_content: &str) -> bool {
    json_content.trim().is_empty()
}

// Removed obsolete tests for old-architecture functions:
// - test_read_utf8_with_fallback (read_utf8_with_fallback function removed)
// - test_write_file_atomic (write_file_atomic function removed)
