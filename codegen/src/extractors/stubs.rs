//! Macro-generated stub extractors
//! 
//! These extractors follow standard patterns and can be generated with a macro.
//! When implementing specific behavior, move the extractor to its own file.

use super::Extractor;
use crate::extraction::ModuleConfig;
use std::path::Path;

macro_rules! define_extractor {
    ($name:ident, $display_name:expr, $script:expr, $subdir:expr, $config_type:expr) => {
        pub struct $name;
        
        impl Extractor for $name {
            fn name(&self) -> &'static str {
                $display_name
            }
            
            fn script_name(&self) -> &'static str {
                $script
            }
            
            fn output_subdir(&self) -> &'static str {
                $subdir
            }
            
            fn handles_config(&self, config_type: &str) -> bool {
                config_type == $config_type
            }
            
            fn build_args(&self, config: &ModuleConfig, module_path: &Path) -> Vec<String> {
                let mut args = vec![
                    module_path.to_string_lossy().to_string()
                ];
                
                // Most extractors pass table/hash names as additional args
                args.extend(config.hash_names.clone());
                
                args
            }
            
            fn output_filename(&self, config: &ModuleConfig, hash_name: Option<&str>) -> String {
                let base = config.source_path
                    .replace('/', "_")
                    .replace(".pm", "")
                    .to_lowercase();
                    
                if let Some(name) = hash_name {
                    format!("{}_{}.json", base, name.to_lowercase())
                } else {
                    format!("{}_{}.json", base, $config_type)
                }
            }
        }
    };
}

// Define remaining extractors using the macro
define_extractor!(TagDefinitionsExtractor, "Tag Definitions", "tag_definitions.pl", "tag_definitions", "tag_definitions");
define_extractor!(CompositeTagsExtractor, "Composite Tags", "composite_tags.pl", "composite_tags", "composite_tags");
define_extractor!(TagTableStructureExtractor, "Tag Table Structure", "tag_table_structure.pl", "tag_structures", "tag_table_structure");
define_extractor!(ProcessBinaryDataExtractor, "Process Binary Data", "process_binary_data.pl", "binary_data", "process_binary_data");
define_extractor!(ModelDetectionExtractor, "Model Detection", "model_detection.pl", "model_detection", "model_detection");
define_extractor!(ConditionalTagsExtractor, "Conditional Tags", "conditional_tags.pl", "conditional_tags", "conditional_tags");
define_extractor!(RegexPatternsExtractor, "Regex Patterns", "regex_patterns.pl", "file_types", "regex_patterns");