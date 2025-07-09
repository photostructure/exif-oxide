//! MIME type mapping generation

use anyhow::Result;
use std::path::Path;

/// Generate MIME type mappings
pub fn generate_mime_mappings(_json_dir: &Path, _output_dir: &str) -> Result<()> {
    // MIME types are currently handled as part of extract system
    // This will be refactored when we complete the migration
    Ok(())
}