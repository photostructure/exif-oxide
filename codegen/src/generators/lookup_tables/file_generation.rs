//! Utility for consistent file generation across all lookup table generators

use anyhow::Result;
use std::fs;
use std::path::Path;
use tracing::debug;
use crate::common::utils::module_to_source_path;

/// Constants for file generation
pub const DO_NOT_EDIT_HEADER: &str = "//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen";

pub const STANDARD_IMPORTS: &[&str] = &[
    "use std::collections::HashMap;",
    "use std::sync::LazyLock;",
];

pub const HASHSET_IMPORTS: &[&str] = &[
    "use std::collections::HashSet;", 
    "use std::sync::LazyLock;",
];

/// Builder for generating consistent file headers and content
pub struct FileGenerator {
    description: String,
    source_module: Option<String>,
    source_table: Option<String>,
    imports: Vec<&'static str>,
    content: String,
}

impl FileGenerator {
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            source_module: None,
            source_table: None,
            imports: Vec::new(),
            content: String::new(),
        }
    }

    pub fn with_source_module(mut self, module: &str) -> Self {
        self.source_module = Some(module_to_source_path(module));
        self
    }

    pub fn with_source_table(mut self, table: &str) -> Self {
        self.source_table = Some(table.to_string());
        self
    }

    pub fn with_standard_imports(mut self) -> Self {
        self.imports.extend_from_slice(STANDARD_IMPORTS);
        self
    }

    pub fn with_hashset_imports(mut self) -> Self {
        self.imports.extend_from_slice(HASHSET_IMPORTS);
        self
    }

    pub fn with_custom_imports(mut self, imports: &[&'static str]) -> Self {
        self.imports.extend_from_slice(imports);
        self
    }

    pub fn with_content(mut self, content: String) -> Self {
        self.content = content;
        self
    }

    pub fn write_to_file(self, file_path: &Path) -> Result<String> {
        let mut full_content = String::new();
        
        // Header with description
        full_content.push_str(&format!("//! {}\n", self.description));
        
        // Source information if provided
        if let Some(source) = &self.source_module {
            full_content.push_str("//! \n");
            if let Some(table) = &self.source_table {
                full_content.push_str(&format!("//! Auto-generated from {source} (table: {table})\n"));
            } else {
                full_content.push_str(&format!("//! Auto-generated from {source}\n"));
            }
        }
        
        // DO NOT EDIT warning
        full_content.push_str(&format!("{DO_NOT_EDIT_HEADER}\n\n"));
        
        // Imports
        for import in &self.imports {
            full_content.push_str(&format!("{import}\n"));
        }
        if !self.imports.is_empty() {
            full_content.push('\n');
        }
        
        // Main content
        full_content.push_str(&self.content);
        
        fs::write(file_path, full_content)?;
        debug!("  ✓ Generated {}", file_path.display());
        
        // Return the filename stem for module registration
        Ok(file_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string())
    }
}

/// Generate a module mod.rs file that re-exports all generated files
pub fn generate_module_mod_file(
    generated_files: &[String],
    module_name: &str,
    output_dir: &Path,
) -> Result<()> {
    let mut content = String::new();
    
    // File header  
    content.push_str(&format!("//! Generated lookup tables from {}\n", module_name.replace("_pm", ".pm")));
    content.push_str("//!\n");
    content.push_str(&format!("//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/{}.pm\n", 
        module_name.replace("_pm", "")));
    content.push_str(&format!("{DO_NOT_EDIT_HEADER}\n\n"));
    
    // Module declarations
    for file_name in generated_files {
        content.push_str(&format!("pub mod {file_name};\n"));
    }
    content.push('\n');
    
    // Re-export all lookup functions and constants
    content.push_str("// Re-export all lookup functions and constants\n");
    for file_name in generated_files {
        content.push_str(&format!("pub use {file_name}::*;\n"));
    }
    
    let mod_file_path = output_dir.join("mod.rs");
    fs::write(&mod_file_path, content)?;
    
    debug!("  ✓ Generated {}", mod_file_path.display());
    Ok(())
}