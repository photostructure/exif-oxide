//! File type detection system code generation
//! 
//! This module handles all file format detection related code generation:
//! - Magic number patterns (regex)
//! - File type discriminated unions with aliases
//! - MIME type mappings
//! - Extension mappings

pub mod patterns;
pub mod types;
pub mod mime;
pub mod extensions;

use anyhow::Result;
use std::path::Path;

/// Generate all file detection related code
pub fn generate_file_detection_code(
    json_dir: &Path,
    output_dir: &str,
) -> Result<()> {
    // Generate components directly in output_dir
    // They will handle their own subdirectory creation if needed
    patterns::generate_magic_patterns(json_dir, output_dir)?;
    types::generate_file_type_lookup(json_dir, output_dir)?;
    
    // Note: mime and extensions modules are placeholders for future expansion
    // Currently their functionality is included in the file_type_lookup generation
    
    Ok(())
}