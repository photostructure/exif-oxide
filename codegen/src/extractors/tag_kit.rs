//! Tag kit extractor implementation

use super::Extractor;
use crate::extraction::ModuleConfig;
use std::path::Path;

pub struct TagKitExtractor;

impl Extractor for TagKitExtractor {
    fn name(&self) -> &'static str {
        "Tag Kit"
    }
    
    fn script_name(&self) -> &'static str {
        "tag_kit.pl"
    }
    
    fn output_subdir(&self) -> &'static str {
        "tag_kits"
    }
    
    fn requires_patching(&self) -> bool {
        false // Tag kit doesn't need patching
    }
    
    fn handles_config(&self, config_type: &str) -> bool {
        config_type == "tag_kit"
    }
    
    fn build_args(&self, config: &ModuleConfig, module_path: &Path) -> Vec<String> {
        let mut args = vec![
            module_path.to_string_lossy().to_string()
        ];
        
        // Add table names from config
        args.extend(config.hash_names.clone());
        
        args
    }
    
    fn output_filename(&self, config: &ModuleConfig, hash_name: Option<&str>) -> String {
        self.standardized_filename(config, hash_name)
    }
    
    fn config_type_name(&self) -> &'static str {
        "tag_kit"
    }
}