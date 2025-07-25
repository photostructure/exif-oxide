//! Macro-generated stub extractors
//! 
//! This file provides a macro that automatically generates extractor implementations for common patterns,
//! eliminating boilerplate code. When an extractor needs custom behavior (like special filename handling),
//! implement it manually instead of using the macro.

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
                self.standardized_filename(config, hash_name)
            }
            
            fn config_type_name(&self) -> &'static str {
                $config_type
            }
        }
    };
}

// Define remaining extractors using the macro
define_extractor!(TagDefinitionsExtractor, "Tag Definitions", "tag_definitions.pl", "tag_definitions", "tag_definitions");
define_extractor!(CompositeTagsExtractor, "Composite Tags", "composite_tags.pl", "composite_tags", "composite_tags");
define_extractor!(ProcessBinaryDataExtractor, "Process Binary Data", "process_binary_data.pl", "binary_data", "process_binary_data");
define_extractor!(ModelDetectionExtractor, "Model Detection", "model_detection.pl", "model_detection", "model_detection");
define_extractor!(ConditionalTagsExtractor, "Conditional Tags", "conditional_tags.pl", "conditional_tags", "conditional_tags");
// RegexPatternsExtractor needs custom filename handling to match generator expectations
pub struct RegexPatternsExtractor;

impl Extractor for RegexPatternsExtractor {
    fn name(&self) -> &'static str {
        "Regex Patterns"
    }
    
    fn script_name(&self) -> &'static str {
        "regex_patterns.pl"
    }
    
    fn output_subdir(&self) -> &'static str {
        "file_types"
    }
    
    fn handles_config(&self, config_type: &str) -> bool {
        config_type == "regex_patterns"
    }
    
    fn build_args(&self, config: &ModuleConfig, module_path: &Path) -> Vec<String> {
        let mut args = vec![
            module_path.to_string_lossy().to_string()
        ];
        
        // Pass table/hash names as additional args
        args.extend(config.hash_names.clone());
        
        args
    }
    
    fn output_filename(&self, _config: &ModuleConfig, _hash_name: Option<&str>) -> String {
        // Use the exact filename the generator expects
        "regex_patterns.json".to_string()
    }
    
    fn config_type_name(&self) -> &'static str {
        "regex_patterns"
    }
}