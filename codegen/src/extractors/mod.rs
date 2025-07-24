//! Trait-based extractor system for code generation
//!
//! This module provides a unified interface for all extraction types,
//! eliminating repetitive code and making it easy to add new extractors.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;
use tracing::{debug, warn};

use crate::extraction::ModuleConfig;

/// Core trait that all extractors implement
pub trait Extractor: Send + Sync {
    /// Name of the extractor for logging
    fn name(&self) -> &'static str;
    
    /// Perl script name (e.g., "simple_table.pl")
    fn script_name(&self) -> &'static str;
    
    /// Output subdirectory under generated/extract/
    fn output_subdir(&self) -> &'static str;
    
    /// Whether this extractor requires patching ExifTool modules
    fn requires_patching(&self) -> bool {
        false
    }
    
    /// Check if this extractor handles the given config type
    fn handles_config(&self, config_type: &str) -> bool;
    
    /// Build command-line arguments for the Perl script
    fn build_args(&self, config: &ModuleConfig, module_path: &Path) -> Vec<String>;
    
    /// Generate output filename for extracted data
    fn output_filename(&self, config: &ModuleConfig, hash_name: Option<&str>) -> String;
    
    /// Extract clean module name from source path for consistent naming
    fn sanitize_module_name(&self, config: &ModuleConfig) -> String {
        // Extract module name from path like "third-party/exiftool/lib/Image/ExifTool/Canon.pm" -> "canon"
        if let Some(filename) = config.source_path.split('/').next_back() {
            filename.replace(".pm", "").to_lowercase().replace('-', "_")
        } else {
            "unknown".to_string()
        }
    }
    
    /// Generate a standardized filename using the pattern: module__type__name.json
    /// Example: olympus__tag_structure__equipment.json
    fn standardized_filename(&self, config: &ModuleConfig, table_or_hash_name: Option<&str>) -> String {
        let module = self.sanitize_module_name(config);
        let config_type = self.config_type_name();
        
        if let Some(name) = table_or_hash_name {
            format!("{}__{}__{}.json", module, config_type, name.to_lowercase())
        } else {
            format!("{module}__{config_type}.json")
        }
    }
    
    /// Get the config type name for filename generation (e.g., "tag_structure", "simple_table")
    fn config_type_name(&self) -> &'static str {
        // Default implementation - extractors should override if needed
        self.output_subdir()
    }
    
    /// Execute the extraction
    fn extract(&self, config: &ModuleConfig, base_dir: &Path, module_path: &Path) -> Result<()> {
        let output_dir = base_dir.join(self.output_subdir());
        fs::create_dir_all(&output_dir)?;
        
        let args = self.build_args(config, module_path);
        let output_filename = self.output_filename(config, None);
        run_perl_extractor(
            self.script_name(),
            &args,
            &output_dir,
            config,
            self.name(),
            &output_filename,
        )
    }
}

/// Run a Perl extraction script with the given arguments
pub(super) fn run_perl_extractor(
    script_name: &str,
    args: &[String],
    output_dir: &Path,
    config: &ModuleConfig,
    extractor_name: &str,
    output_filename: &str,
) -> Result<()> {
    // Get absolute paths
    let codegen_dir = std::env::current_dir()?.canonicalize()?;
    let repo_root = codegen_dir.parent()
        .ok_or_else(|| anyhow::anyhow!("Could not find repo root"))?;
    let script_path = codegen_dir.join("extractors").join(script_name);
    
    let mut cmd = Command::new("perl");
    cmd.arg(&script_path)
       .args(args)
       .current_dir(output_dir);
    
    // Pass paths as environment variables
    cmd.env("CODEGEN_DIR", &codegen_dir);
    cmd.env("REPO_ROOT", repo_root);
    
    setup_perl_environment(&mut cmd);
    
    debug!("    Running: perl {} {}", script_name, args.join(" "));
    
    let output = cmd.output()
        .with_context(|| format!("Failed to execute {} for {}", extractor_name, config.module_name))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("{} failed for {}: {}", extractor_name, config.module_name, stderr));
    }
    
    // Handle script output 
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Filter stderr - show only actual errors, not verbose progress messages
    if !stderr.is_empty() {
        // Split stderr into lines and filter out verbose progress messages
        for line in stderr.lines() {
            if line.contains("Error:") || line.contains("error:") || 
               line.contains("Failed") || line.contains("Warning:") ||
               line.contains("Execution of") || line.contains("aborted") {
                warn!("Perl extractor: {}", line);
            } else {
                debug!("Perl extractor: {}", line);
            }
        }
    }
    
    // The Perl extractors write their JSON output to stdout
    // We need to capture it and write it to the appropriate file
    if !stdout.is_empty() {
        let output_file = output_dir.join(output_filename);
        fs::write(&output_file, stdout.as_bytes())
            .with_context(|| format!("Failed to write output to {}", output_file.display()))?;
        
        debug!("Created {}", output_filename);
    }
    
    Ok(())
}

fn setup_perl_environment(cmd: &mut Command) {
    let perl5lib = format!(
        "{}:{}",
        std::env::var("HOME").unwrap_or_default() + "/perl5/lib/perl5",
        "../lib:../../third-party/exiftool/lib"
    );
    cmd.env("PERL5LIB", perl5lib);
}

// Re-export all extractor implementations
mod simple_table;
mod tag_kit;
mod runtime_table;
mod file_type;
mod inline_printconv;
mod boolean_set;
mod tag_table_structure;
mod stubs;

pub use simple_table::SimpleTableExtractor;
pub use tag_kit::TagKitExtractor;
pub use runtime_table::RuntimeTableExtractor;
pub use file_type::FileTypeLookupExtractor;
pub use inline_printconv::InlinePrintConvExtractor;
pub use boolean_set::BooleanSetExtractor;
pub use tag_table_structure::TagTableStructureExtractor;
pub use stubs::{
    TagDefinitionsExtractor,
    CompositeTagsExtractor, ProcessBinaryDataExtractor,
    ModelDetectionExtractor, ConditionalTagsExtractor, RegexPatternsExtractor,
};

/// Registry of all available extractors
pub fn all_extractors() -> Vec<Box<dyn Extractor>> {
    vec![
        Box::new(SimpleTableExtractor),
        Box::new(TagKitExtractor),
        Box::new(RuntimeTableExtractor),
        Box::new(FileTypeLookupExtractor),
        Box::new(BooleanSetExtractor),
        Box::new(InlinePrintConvExtractor),
        Box::new(TagDefinitionsExtractor),
        Box::new(CompositeTagsExtractor),
        Box::new(TagTableStructureExtractor),
        Box::new(ProcessBinaryDataExtractor),
        Box::new(ModelDetectionExtractor),
        Box::new(ConditionalTagsExtractor),
        Box::new(RegexPatternsExtractor),
    ]
}

/// Find the appropriate extractor for a config type
pub fn find_extractor(config_type: &str) -> Option<Box<dyn Extractor>> {
    all_extractors()
        .into_iter()
        .find(|e| e.handles_config(config_type))
}