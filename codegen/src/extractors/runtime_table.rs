//! Runtime table extractor implementation

use super::Extractor;
use crate::extraction::ModuleConfig;
use std::path::Path;

pub struct RuntimeTableExtractor;

impl Extractor for RuntimeTableExtractor {
    fn name(&self) -> &'static str {
        "Runtime Table"
    }
    
    fn script_name(&self) -> &'static str {
        "runtime_table.pl"
    }
    
    fn output_subdir(&self) -> &'static str {
        "runtime_tables"
    }
    
    fn requires_patching(&self) -> bool {
        false // Runtime tables don't need patching
    }
    
    fn handles_config(&self, config_type: &str) -> bool {
        config_type == "runtime_table"
    }
    
    fn build_args(&self, config: &ModuleConfig, module_path: &Path) -> Vec<String> {
        let mut args = vec![
            module_path.to_string_lossy().to_string()
        ];
        
        // Add table names (already without % prefix for runtime tables)
        args.extend(config.hash_names.clone());
        
        args
    }
    
    fn output_filename(&self, config: &ModuleConfig, hash_name: Option<&str>) -> String {
        let base_name = config.source_path
            .replace('/', "_")
            .replace(".pm", "")
            .to_lowercase();
            
        if let Some(table) = hash_name {
            format!("{}_runtime_table_{}.json", base_name, table.to_lowercase())
        } else {
            format!("{}_runtime_tables.json", base_name)
        }
    }
}