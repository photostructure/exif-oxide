//! Inline PrintConv extractor implementation
//!
//! This extractor handles tables one at a time, unlike most others.

use super::{Extractor, run_perl_extractor};
use crate::extraction::ModuleConfig;
use std::path::Path;
use anyhow::Result;

pub struct InlinePrintConvExtractor;

impl Extractor for InlinePrintConvExtractor {
    fn name(&self) -> &'static str {
        "Inline PrintConv"
    }
    
    fn script_name(&self) -> &'static str {
        "inline_printconv.pl"
    }
    
    fn output_subdir(&self) -> &'static str {
        "inline_printconv"
    }
    
    fn handles_config(&self, config_type: &str) -> bool {
        config_type == "inline_printconv"
    }
    
    fn build_args(&self, _config: &ModuleConfig, _module_path: &Path) -> Vec<String> {
        // This is handled specially in extract()
        vec![]
    }
    
    fn output_filename(&self, config: &ModuleConfig, hash_name: Option<&str>) -> String {
        let base = config.source_path
            .replace('/', "_")
            .replace(".pm", "")
            .to_lowercase();
            
        if let Some(table) = hash_name {
            format!("{}_inline_printconv_{}.json", base, table.to_lowercase())
        } else {
            format!("{}_inline_printconv.json", base)
        }
    }
    
    // Override extract to handle one table at a time
    fn extract(&self, config: &ModuleConfig, base_dir: &Path, module_path: &Path) -> Result<()> {
        let output_dir = base_dir.join(self.output_subdir());
        std::fs::create_dir_all(&output_dir)?;
        
        // Process each table separately
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