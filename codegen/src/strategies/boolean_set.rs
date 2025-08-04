//! BooleanSetStrategy - Processes ExifTool boolean set definitions
//!
//! This strategy recognizes and processes symbols that contain membership sets
//! (hashes where keys map to truthy values for fast lookups like "if ($isDatChunk{$chunk})").

use anyhow::Result;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use tracing::{debug, info};

use super::{ExtractionStrategy, ExtractionContext, GeneratedFile};
use crate::field_extractor::FieldSymbol;
use crate::strategies::output_locations::generate_module_path;

/// Strategy for processing boolean membership sets
pub struct BooleanSetStrategy {
    /// Boolean sets to be generated per module
    boolean_sets: HashMap<String, Vec<BooleanSet>>,
}

#[derive(Debug, Clone)]
struct BooleanSet {
    name: String,
    module_name: String,
    keys: Vec<String>,
}

impl BooleanSetStrategy {
    pub fn new() -> Self {
        Self {
            boolean_sets: HashMap::new(),
        }
    }
    
    /// Check if symbol is a boolean set (keys mapping to truthy values)
    fn is_boolean_set_symbol(symbol: &FieldSymbol) -> bool {
        if let Some(data) = symbol.data.as_object() {
            // Skip ProcessBinaryData indicators
            if data.contains_key("FIRST_ENTRY") || data.contains_key("FORMAT") {
                return false;
            }
            
            // Skip tag table indicators  
            if data.contains_key("WRITABLE") || data.contains_key("GROUPS") {
                return false;
            }
            
            // Look for patterns indicating boolean sets
            let boolean_patterns = ["isDat", "isTxt", "noLeap", "Valid", "Type"];
            for pattern in &boolean_patterns {
                if symbol.name.contains(pattern) {
                    return true;
                }
            }
            
            // Check if most values are 1 or other truthy indicators
            let mut truthy_count = 0;
            let mut total_count = 0;
            
            for (key, value) in data {
                // Skip special ExifTool metadata keys
                if key.starts_with(char::is_uppercase) && key.len() > 3 {
                    continue;
                }
                
                total_count += 1;
                
                match value {
                    JsonValue::Number(n) if n.as_u64() == Some(1) => truthy_count += 1,
                    JsonValue::Bool(true) => truthy_count += 1,
                    JsonValue::String(s) if s == "1" => truthy_count += 1,
                    _ => {}
                }
            }
            
            // If >80% of values are truthy, likely a boolean set
            if total_count > 0 && (truthy_count as f64 / total_count as f64) > 0.8 {
                return true;
            }
        }
        
        false
    }
    
    /// Extract keys that map to truthy values
    fn extract_boolean_keys(data: &JsonValue) -> Vec<String> {
        let mut keys = Vec::new();
        
        if let Some(obj) = data.as_object() {
            for (key, value) in obj {
                // Skip ExifTool metadata
                if key.starts_with(char::is_uppercase) && key.len() > 3 {
                    continue;
                }
                
                // Check if value is truthy
                let is_truthy = match value {
                    JsonValue::Number(n) if n.as_u64() == Some(1) => true,
                    JsonValue::Bool(true) => true,
                    JsonValue::String(s) if s == "1" => true,
                    _ => false,
                };
                
                if is_truthy {
                    keys.push(key.clone());
                }
            }
        }
        
        keys.sort(); // Consistent ordering
        keys
    }
    
    /// Generate Rust code for boolean set
    fn generate_boolean_set_code(boolean_set: &BooleanSet) -> String {
        let mut code = String::new();
        
        code.push_str(&format!("//! Generated boolean set: {}\n", boolean_set.name));
        code.push_str("//!\n");
        code.push_str("//! Fast membership testing for string keys\n");
        code.push_str("\n");
        code.push_str("use std::sync::LazyLock;\n");
        code.push_str("use std::collections::HashSet;\n");
        code.push_str("\n");
        
        // Generate constant name (convert camelCase to SCREAMING_SNAKE_CASE)
        let const_name = boolean_set.name
            .chars()
            .fold(String::new(), |mut acc, c| {
                if c.is_uppercase() && !acc.is_empty() {
                    acc.push('_');
                }
                acc.push(c.to_ascii_uppercase());
                acc
            });
        
        code.push_str(&format!("/// Boolean set for fast membership testing: {}\n", boolean_set.name));
        code.push_str(&format!("pub static {}: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {{\n", const_name));
        code.push_str("    let mut set = HashSet::new();\n");
        
        for key in &boolean_set.keys {
            code.push_str(&format!("    set.insert(\"{}\");\n", key));
        }
        
        code.push_str("    set\n");
        code.push_str("});\n");
        code.push_str("\n");
        
        // Generate lookup function
        let fn_name = boolean_set.name
            .chars()
            .fold(String::new(), |mut acc, c| {
                if c.is_uppercase() && !acc.is_empty() {
                    acc.push('_');
                }
                acc.push(c.to_ascii_lowercase());
                acc
            });
        
        code.push_str(&format!("/// Check if key is in {} set\n", boolean_set.name));
        code.push_str(&format!("pub fn is_{}(key: &str) -> bool {{\n", fn_name));
        code.push_str(&format!("    {}.contains(key)\n", const_name));
        code.push_str("}\n");
        
        code
    }
}

impl ExtractionStrategy for BooleanSetStrategy {
    fn name(&self) -> &'static str {
        "BooleanSetStrategy"
    }
    
    fn can_handle(&self, symbol_data: &JsonValue) -> bool {
        // Convert JsonValue to FieldSymbol for analysis
        if let Ok(symbol) = serde_json::from_value::<FieldSymbol>(symbol_data.clone()) {
            Self::is_boolean_set_symbol(&symbol)
        } else {
            false
        }
    }
    
    fn extract(&mut self, symbol_data: &FieldSymbol, context: &mut ExtractionContext) -> Result<()> {
        let keys = Self::extract_boolean_keys(&symbol_data.data);
        
        if keys.is_empty() {
            return Ok(()); // Skip empty sets
        }
        
        context.log_strategy_selection(symbol_data, self.name(), 
            &format!("Detected boolean set with {} truthy keys", keys.len()));
        
        let boolean_set = BooleanSet {
            name: symbol_data.name.clone(),
            module_name: symbol_data.module.clone(),
            keys,
        };
        
        debug!("Extracted boolean set: {}::{} ({} keys)", 
               symbol_data.module, symbol_data.name, boolean_set.keys.len());
        
        self.boolean_sets
            .entry(symbol_data.module.clone())
            .or_insert_with(Vec::new)
            .push(boolean_set);
        
        Ok(())
    }
    
    fn finish_module(&mut self, _module_name: &str) -> Result<()> {
        // Nothing to do per-module
        Ok(())
    }
    
    fn finish_extraction(&mut self) -> Result<Vec<GeneratedFile>> {
        let mut files = Vec::new();
        
        for (module_name, sets) in &self.boolean_sets {
            for boolean_set in sets {
                let code = Self::generate_boolean_set_code(boolean_set);
                let path = generate_module_path(module_name, &boolean_set.name.to_lowercase());
                
                files.push(GeneratedFile {
                    path,
                    content: code,
                    strategy: self.name().to_string(),
                });
                
                debug!("Generated boolean set: {}::{}", module_name, boolean_set.name);
            }
        }
        
        info!("BooleanSetStrategy generated {} files", files.len());
        Ok(files)
    }
}