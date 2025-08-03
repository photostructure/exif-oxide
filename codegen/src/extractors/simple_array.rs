//! Simple array extractor implementation

use super::Extractor;
use crate::extraction::ModuleConfig;
use std::path::Path;

pub struct SimpleArrayExtractor;

impl Extractor for SimpleArrayExtractor {
    fn name(&self) -> &'static str {
        "Simple Array"
    }
    
    fn script_name(&self) -> &'static str {
        "simple_array.pl"
    }
    
    fn output_subdir(&self) -> &'static str {
        "simple_arrays"
    }
    
    
    fn handles_config(&self, config_type: &str) -> bool {
        config_type == "simple_array"
    }
    
    fn build_args(&self, config: &ModuleConfig, module_path: &Path) -> Vec<String> {
        let mut args = vec![
            module_path.to_string_lossy().to_string()
        ];
        
        // Add array expressions directly (the perl script handles @ prefix logic)
        // Note: config.hash_names contains array expressions for simple_array configs
        for array_expr in &config.hash_names {
            args.push(array_expr.clone());
        }
        
        args
    }
    
    fn output_filename(&self, config: &ModuleConfig, array_expr: Option<&str>) -> String {
        // For simple arrays, we'll use the array expression to generate filename
        if let Some(expr) = array_expr {
            // Convert array expression to safe filename
            // xlat[0] -> xlat_0, afPoints231 -> af_points231
            let mut filename = expr.to_string();
            filename = filename.trim_start_matches('@').to_string(); // Remove @ prefix
            filename = filename.replace('[', "_").replace(']', "");   // xlat[0] -> xlat_0
            
            // Convert camelCase to snake_case
            let mut result = String::new();
            for (i, ch) in filename.chars().enumerate() {
                if ch.is_uppercase() && i > 0 {
                    result.push('_');
                }
                result.push(ch.to_lowercase().next().unwrap_or(ch));
            }
            
            // Replace any remaining non-safe characters with underscore
            result = result.chars()
                .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
                .collect::<String>()
                .trim_start_matches('_')
                .to_string();
            
            format!("{}.json", result)
        } else {
            // Fallback - use standardized filename
            self.standardized_filename(config, None)
        }
    }
    
    fn config_type_name(&self) -> &'static str {
        "simple_array"
    }
}