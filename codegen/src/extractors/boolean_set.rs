//! Boolean set extractor implementation
//!
//! This extractor handles sets one at a time, similar to inline_printconv.

use super::{Extractor, run_perl_extractor};
use crate::extraction::ModuleConfig;
use std::path::Path;
use anyhow::Result;

pub struct BooleanSetExtractor;

impl Extractor for BooleanSetExtractor {
    fn name(&self) -> &'static str {
        "Boolean Set"
    }
    
    fn script_name(&self) -> &'static str {
        "boolean_set.pl"
    }
    
    fn output_subdir(&self) -> &'static str {
        "boolean_sets"
    }
    
    
    fn handles_config(&self, config_type: &str) -> bool {
        config_type == "boolean_set"
    }
    
    fn build_args(&self, _config: &ModuleConfig, _module_path: &Path) -> Vec<String> {
        // This is handled specially in extract()
        vec![]
    }
    
    fn output_filename(&self, config: &ModuleConfig, hash_name: Option<&str>) -> String {
        self.standardized_filename(config, hash_name)
    }
    
    fn config_type_name(&self) -> &'static str {
        "boolean_set"
    }
    
    // Override extract to handle one set at a time
    fn extract(&self, config: &ModuleConfig, base_dir: &Path, module_path: &Path) -> Result<()> {
        let output_dir = base_dir.join(self.output_subdir());
        std::fs::create_dir_all(&output_dir)?;
        
        // Process each boolean set separately
        for set_name in &config.hash_names {
            let args = vec![
                module_path.to_string_lossy().to_string(),
                set_name.clone(),
            ];
            
            let output_filename = self.output_filename(config, Some(set_name));
            run_perl_extractor(
                self.script_name(),
                &args,
                &output_dir,
                config,
                &format!("{} ({})", self.name(), set_name),
                &output_filename,
            )?;
        }
        
        Ok(())
    }
}