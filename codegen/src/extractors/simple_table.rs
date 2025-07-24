//! Simple table extractor implementation

use super::Extractor;
use crate::extraction::ModuleConfig;
use std::path::Path;

pub struct SimpleTableExtractor;

impl Extractor for SimpleTableExtractor {
    fn name(&self) -> &'static str {
        "Simple Table"
    }
    
    fn script_name(&self) -> &'static str {
        "simple_table.pl"
    }
    
    fn output_subdir(&self) -> &'static str {
        "simple_tables"
    }
    
    fn requires_patching(&self) -> bool {
        true // Simple tables need patching to expose my-scoped variables
    }
    
    fn handles_config(&self, config_type: &str) -> bool {
        config_type == "simple_table"
    }
    
    fn build_args(&self, config: &ModuleConfig, module_path: &Path) -> Vec<String> {
        let mut args = vec![
            module_path.to_string_lossy().to_string()
        ];
        
        // Add hash names with % prefix
        for hash_name in &config.hash_names {
            args.push(format!("%{hash_name}"));
        }
        
        args
    }
    
    fn output_filename(&self, config: &ModuleConfig, hash_name: Option<&str>) -> String {
        // For simple tables, we'll use just the hash name if it's a standalone table
        // This maintains compatibility with existing code that expects "canon_white_balance.json"
        if let Some(hash) = hash_name {
            // Check if this is a module-specific table or a general one
            let module_name = self.sanitize_module_name(config);
            if module_name == "exiftool" {
                // General ExifTool tables don't need module prefix
                format!("{}.json", hash.to_lowercase())
            } else {
                // Module-specific tables use standardized format
                self.standardized_filename(config, Some(hash))
            }
        } else {
            self.standardized_filename(config, None)
        }
    }
    
    fn config_type_name(&self) -> &'static str {
        "simple_table"
    }
}