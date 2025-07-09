//! File extension mapping generation

use anyhow::Result;
use std::path::Path;

/// Generate file extension mappings
pub fn generate_extension_mappings(_json_dir: &Path, _output_dir: &str) -> Result<()> {
    // Extensions are currently handled as part of simple_tables
    // This will be refactored when we complete the migration
    Ok(())
}