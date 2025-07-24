//! Tag kit extractor implementation

use super::Extractor;
use crate::extraction::ModuleConfig;
use std::path::Path;
use anyhow::Result;

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
    
    fn build_args(&self, _config: &ModuleConfig, module_path: &Path) -> Vec<String> {
        // This method is not used for tag_kit because we override extract()
        // to call the script once per table
        vec![module_path.to_string_lossy().to_string()]
    }
    
    fn output_filename(&self, config: &ModuleConfig, hash_name: Option<&str>) -> String {
        self.standardized_filename(config, hash_name)
    }
    
    fn config_type_name(&self) -> &'static str {
        "tag_kit"
    }
    
    /// Custom extract implementation for tag_kit that calls the script once per table
    /// and then consolidates all tables into a single module file
    fn extract(&self, config: &ModuleConfig, base_dir: &Path, module_path: &Path) -> Result<()> {
        println!("🔧 TagKitExtractor::extract called for module: {} with {} tables", 
            config.module_name, config.hash_names.len());
        use super::run_perl_extractor;
        use std::fs;
        
        let output_dir = base_dir.join(self.output_subdir());
        fs::create_dir_all(&output_dir)?;
        
        let mut all_tag_kits = Vec::new();
        let mut all_source_info = None;
        let mut total_scanned = 0;
        let mut total_skipped = 0;
        
        // Call the script once per table (hash_name) and collect results
        for table_name in &config.hash_names {
            println!("    Running: perl {} {} {}", 
                self.script_name(), 
                module_path.display(), 
                table_name
            );
            
            let args = vec![
                module_path.to_string_lossy().to_string(),
                table_name.clone()
            ];
            
            let individual_filename = format!("{}__temp__{}.json", 
                self.sanitize_module_name(config), table_name.to_lowercase());
            
            run_perl_extractor(
                self.script_name(),
                &args,
                &output_dir,
                config,
                self.name(),
                &individual_filename,
            )?;
            
            // Read the individual file and parse it
            let individual_path = output_dir.join(&individual_filename);
            if individual_path.exists() {
                let content = fs::read_to_string(&individual_path)?;
                let extraction: crate::schemas::tag_kit::TagKitExtraction = 
                    serde_json::from_str(&content)?;
                
                // Collect tag kits
                all_tag_kits.extend(extraction.tag_kits);
                
                // Accumulate metadata
                total_scanned += extraction.metadata.total_tags_scanned;
                total_skipped += extraction.metadata.skipped_complex;
                
                // Store source info (should be the same for all tables from same module)
                if all_source_info.is_none() {
                    all_source_info = Some(extraction.source);
                }
                
                // Remove the temporary individual file
                fs::remove_file(&individual_path)?;
            }
        }
        
        // Create consolidated tag kit file
        if !all_tag_kits.is_empty() {
            let total_tag_kits = all_tag_kits.len();
            
            let consolidated = crate::schemas::tag_kit::TagKitExtraction {
                source: all_source_info.unwrap_or_else(|| crate::schemas::tag_kit::SourceInfo {
                    module: config.source_path.clone(),
                    table: "Multiple".to_string(),
                    extracted_at: "consolidated".to_string(),
                }),
                metadata: crate::schemas::tag_kit::MetadataInfo {
                    total_tags_scanned: total_scanned,
                    tag_kits_extracted: total_tag_kits,
                    skipped_complex: total_skipped,
                },
                tag_kits: all_tag_kits,
            };
            
            let consolidated_filename = self.standardized_filename(config, None);
            let consolidated_path = output_dir.join(&consolidated_filename);
            let consolidated_content = serde_json::to_string_pretty(&consolidated)?;
            fs::write(&consolidated_path, consolidated_content)?;
            
            println!("  ✓ Consolidated {} tables into {}", config.hash_names.len(), consolidated_filename);
        }
        
        Ok(())
    }
}