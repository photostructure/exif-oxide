//! Tag table structure extractor implementation
//!
//! This extractor handles tables one at a time to support multiple table structures per module.

use super::{Extractor, run_perl_extractor};
use crate::extraction::ModuleConfig;
use std::path::Path;
use anyhow::Result;

pub struct TagTableStructureExtractor;

impl Extractor for TagTableStructureExtractor {
    fn name(&self) -> &'static str {
        "Tag Table Structure"
    }
    
    fn script_name(&self) -> &'static str {
        "tag_table_structure.pl"
    }
    
    fn output_subdir(&self) -> &'static str {
        "tag_structures"
    }
    
    fn handles_config(&self, config_type: &str) -> bool {
        config_type == "tag_table_structure" || config_type.ends_with("_tag_table_structure")
    }
    
    fn build_args(&self, _config: &ModuleConfig, _module_path: &Path) -> Vec<String> {
        // This is handled specially in extract()
        vec![]
    }
    
    fn output_filename(&self, config: &ModuleConfig, hash_name: Option<&str>) -> String {
        self.standardized_filename(config, hash_name)
    }
    
    fn config_type_name(&self) -> &'static str {
        "tag_structure"
    }
    
    // Override extract to handle one table at a time
    fn extract(&self, config: &ModuleConfig, base_dir: &Path, module_path: &Path) -> Result<()> {
        let output_dir = base_dir.join(self.output_subdir());
        std::fs::create_dir_all(&output_dir)?;
        
        // Process each table separately (config.hash_names contains table names like ["Main"] or ["Equipment"])
        for table_name in &config.hash_names {
            let args = vec![
                module_path.to_string_lossy().to_string(),
                table_name.clone(),
            ];
            
            let output_filename = self.output_filename(config, Some(table_name));
            run_perl_extractor(
                self.script_name(),
                &args,
                &output_dir,
                config,
                &format!("{} ({})", self.name(), table_name),
                &output_filename,
            )?;
        }
        
        Ok(())
    }
}