//! File type lookup extractor implementation

use super::Extractor;
use crate::extraction::ModuleConfig;
use std::path::Path;

pub struct FileTypeLookupExtractor;

impl Extractor for FileTypeLookupExtractor {
    fn name(&self) -> &'static str {
        "File Type Lookup"
    }
    
    fn script_name(&self) -> &'static str {
        "file_type_lookup.pl"
    }
    
    fn output_subdir(&self) -> &'static str {
        "file_types"
    }
    
    fn handles_config(&self, config_type: &str) -> bool {
        config_type == "file_type_lookup"
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
    
    fn output_filename(&self, config: &ModuleConfig, _hash_name: Option<&str>) -> String {
        let module_name = self.sanitize_module_name(config);
        format!("{module_name}_file_type_lookup.json")
    }
}