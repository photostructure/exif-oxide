//! SimpleTableStrategy for processing hash symbols with string mappings
//!
//! This strategy handles symbols that represent simple key-value lookups,
//! similar to the existing simple_table.pl extractor output.

use anyhow::Result;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use tracing::{debug, info};

use super::{ExtractionContext, ExtractionStrategy, GeneratedFile, output_locations};
use crate::field_extractor::FieldSymbol;

/// Strategy for processing simple hash tables with string values
/// 
/// Recognizes patterns like:
/// ```json
/// {
///   "type": "hash",
///   "data": {"0": "Auto", "1": "Daylight", "2": "Cloudy"},
///   "metadata": {"size": 3}
/// }
/// ```
pub struct SimpleTableStrategy {
    /// Collected tables by module
    tables: HashMap<String, Vec<SimpleTable>>,
}

/// A simple lookup table extracted from ExifTool symbol
#[derive(Debug, Clone)]
struct SimpleTable {
    /// Symbol name from ExifTool (e.g., "canonWhiteBalance")
    name: String,
    
    /// Module name (e.g., "Canon")
    module: String,
    
    /// Key-value mappings
    data: HashMap<String, String>,
}

impl SimpleTableStrategy {
    /// Create new SimpleTableStrategy
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }
    
    /// Generate standardized output filename for a table using output_locations
    fn output_filename(&self, module: &str, table_name: &str) -> String {
        output_locations::generate_module_path(module, table_name)
    }
    
    
    /// Generate Rust code for a simple table
    fn generate_table_code(&self, table: &SimpleTable) -> String {
        let _struct_name = self.pascal_case(&table.name);
        let const_name = self.constant_case(&table.name);
        let function_name = format!("lookup_{}", output_locations::to_snake_case(&table.name));
        
        // Determine key type from the data
        let key_type = self.infer_key_type(&table.data);
        
        let mut code = String::new();
        
        // File header
        code.push_str(&format!(
            "//! Generated lookup table for {} from ExifTool's {} module\n",
            table.name, table.module
        ));
        code.push_str("//!\n");
        code.push_str("//! This file is auto-generated. Do not edit manually.\n\n");
        
        // Imports
        code.push_str("use std::collections::HashMap;\n");
        code.push_str("use std::sync::LazyLock;\n\n");
        
        // Static data array
        code.push_str(&format!("/// Raw data for {} lookup table\n", table.name));
        code.push_str(&format!("static {}_DATA: &[({}, &'static str)] = &[\n", const_name, key_type));
        
        // Sort entries for consistent output
        let mut entries: Vec<_> = table.data.iter().collect();
        entries.sort_by_key(|&(k, _)| k);
        
        for (key, value) in entries {
            let formatted_key = self.format_key(key, &key_type);
            let escaped_value = value.replace('\\', "\\\\").replace('"', "\\\"");
            code.push_str(&format!("    ({}, \"{}\"),\n", formatted_key, escaped_value));
        }
        
        code.push_str("];\n\n");
        
        // LazyLock HashMap
        code.push_str(&format!("/// {} lookup table\n", table.name));
        code.push_str(&format!(
            "pub static {}: LazyLock<HashMap<{}, &'static str>> = LazyLock::new(|| {{\n",
            const_name, key_type
        ));
        code.push_str(&format!("    {}_DATA.iter().copied().collect()\n", const_name));
        code.push_str("});\n\n");
        
        // Lookup function
        code.push_str(&format!("/// Look up {} value by key\n", table.name));
        code.push_str(&format!(
            "pub fn {}(key: {}) -> Option<&'static str> {{\n",
            function_name, key_type
        ));
        code.push_str(&format!("    {}.get(&key).copied()\n", const_name));
        code.push_str("}\n");
        
        code
    }
    
    /// Convert snake_case to PascalCase
    fn pascal_case(&self, name: &str) -> String {
        name.split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                }
            })
            .collect()
    }
    
    /// Convert to CONSTANT_CASE
    fn constant_case(&self, name: &str) -> String {
        output_locations::to_snake_case(name).to_uppercase()
    }
    
    /// Infer Rust key type from the HashMap keys
    fn infer_key_type(&self, data: &HashMap<String, String>) -> &'static str {
        // Check if all keys are numeric
        let all_numeric = data.keys().all(|k| k.parse::<i64>().is_ok());
        
        if all_numeric {
            // Check range to pick appropriate integer type
            let max_value = data.keys()
                .filter_map(|k| k.parse::<i64>().ok())
                .max()
                .unwrap_or(0);
            
            if max_value <= 255 {
                "u8"
            } else if max_value <= 65535 {
                "u16"
            } else {
                "u32"
            }
        } else {
            "&str"
        }
    }
    
    /// Format key value for Rust code
    fn format_key(&self, key: &str, key_type: &str) -> String {
        match key_type {
            "&str" => format!("\"{}\"", key.replace('\\', "\\\\").replace('"', "\\\"")),
            _ => key.to_string(), // Numeric types
        }
    }
}

impl ExtractionStrategy for SimpleTableStrategy {
    fn name(&self) -> &'static str {
        "SimpleTableStrategy"
    }
    
    fn can_handle(&self, symbol: &FieldSymbol) -> bool {
        // Don't claim composite tables - let CompositeTagStrategy handle those
        if symbol.metadata.is_composite_table == 1 {
            return false;
        }
        
        // Check if this is a hash with simple string values
        if let JsonValue::Object(map) = &symbol.data {
            // Must have at least one entry
            if map.is_empty() {
                return false;
            }
            
            // All values must be strings for simple table
            let all_strings = map.values().all(|v| v.is_string());
            
            // Check that this doesn't look like a tag definition
            let has_tag_markers = map.contains_key("PrintConv") || 
                                 map.contains_key("ValueConv") || 
                                 map.contains_key("Name") ||
                                 map.contains_key("WRITABLE") ||
                                 map.contains_key("GROUPS") ||
                                 map.contains_key("WRITE_GROUP");
            
            // Specifically claim known lens/camera type lookup tables
            let known_lookup_tables = [
                "canonLensTypes", "canonModelID", "canonImageSize", 
                "olympusLensTypes", "olympusCameraTypes",
                "nikonLensIDs", "sonyLensTypes", "pentaxLensTypes"
            ];
            let is_known_lookup = known_lookup_tables.contains(&symbol.name.as_str());
            
            // Detect mixed-key patterns (both numeric and decimal keys like "2.1")
            let has_mixed_keys = map.keys().any(|k| {
                k.contains('.') && k.parse::<f64>().is_ok() // Decimal keys like "2.1"
            });
            
            (all_strings && !has_tag_markers) || is_known_lookup || has_mixed_keys
        } else {
            false
        }
    }
    
    fn extract(&mut self, symbol: &FieldSymbol, _context: &mut ExtractionContext) -> Result<()> {
        // Verify this is a hash symbol
        if symbol.symbol_type != "hash" {
            return Ok(()); // Skip non-hash symbols
        }
        
        // Extract the hash data
        if let JsonValue::Object(data_map) = &symbol.data {
            let mut table_data = HashMap::new();
            
            for (key, value) in data_map {
                if let JsonValue::String(str_value) = value {
                    table_data.insert(key.clone(), str_value.clone());
                }
            }
            
            if !table_data.is_empty() {
                let table = SimpleTable {
                    name: symbol.name.clone(),
                    module: symbol.module.clone(),
                    data: table_data,
                };
                
                debug!("📊 Extracted simple table: {} ({} entries)", table.name, table.data.len());
                
                // Group by module
                self.tables
                    .entry(symbol.module.clone())
                    .or_insert_with(Vec::new)
                    .push(table);
            }
        }
        
        Ok(())
    }
    
    fn finish_module(&mut self, _module_name: &str) -> Result<()> {
        // No per-module finalization needed
        Ok(())
    }
    
    fn finish_extraction(&mut self) -> Result<Vec<GeneratedFile>> {
        let mut generated_files = Vec::new();
        
        info!("🔧 Generating simple table code for {} modules", self.tables.len());
        
        for (module_name, tables) in &self.tables {
            for table in tables {
                let filename = self.output_filename(module_name, &table.name);
                let content = self.generate_table_code(table);
                
                let generated_file = GeneratedFile {
                    path: filename,
                    content,
                    strategy: self.name().to_string(),
                };
                
                debug!("📄 Generated: {} ({} bytes)", generated_file.path, generated_file.content.len());
                generated_files.push(generated_file);
            }
        }
        
        info!("✅ SimpleTableStrategy generated {} files", generated_files.len());
        Ok(generated_files)
    }
}

impl Default for SimpleTableStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::field_extractor::{FieldMetadata, FieldSymbol};
    
    #[test]
    fn test_can_handle_simple_hash() {
        let strategy = SimpleTableStrategy::new();
        
        // Should handle simple string map
        let simple_hash = FieldSymbol {
            symbol_type: "hash".to_string(),
            name: "whiteBalance".to_string(),
            data: json!({"0": "Auto", "1": "Daylight", "2": "Cloudy"}),
            module: "Canon".to_string(),
            metadata: FieldMetadata {
                size: 3,
                is_composite_table: 0,
            },
        };
        assert!(strategy.can_handle(&simple_hash));
        
        // Should reject empty hash
        let empty_hash = FieldSymbol {
            symbol_type: "hash".to_string(),
            name: "emptyHash".to_string(),
            data: json!({}),
            module: "Canon".to_string(),
            metadata: FieldMetadata {
                size: 0,
                is_composite_table: 0,
            },
        };
        assert!(!strategy.can_handle(&empty_hash));
        
        // Should reject tag definition
        let tag_def = FieldSymbol {
            symbol_type: "hash".to_string(),
            name: "tagTable".to_string(),
            data: json!({"Name": "WhiteBalance", "PrintConv": "..."}),
            module: "Canon".to_string(),
            metadata: FieldMetadata {
                size: 2,
                is_composite_table: 0,
            },
        };
        assert!(!strategy.can_handle(&tag_def));
        
        // Should reject non-string values
        let mixed_values = FieldSymbol {
            symbol_type: "hash".to_string(),
            name: "mixedHash".to_string(),
            data: json!({"0": "Auto", "1": 123}),
            module: "Canon".to_string(),
            metadata: FieldMetadata {
                size: 2,
                is_composite_table: 0,
            },
        };
        assert!(!strategy.can_handle(&mixed_values));
    }
    
    #[test]
    fn test_extract_simple_table() {
        let mut strategy = SimpleTableStrategy::new();
        let mut context = ExtractionContext::new("output".to_string());
        
        let symbol = FieldSymbol {
            symbol_type: "hash".to_string(),
            name: "whiteBalance".to_string(),
            data: json!({"0": "Auto", "1": "Daylight", "2": "Cloudy"}),
            module: "Canon".to_string(),
            metadata: FieldMetadata {
                size: 3,
                is_composite_table: 0,
            },
        };
        
        strategy.extract(&symbol, &mut context).unwrap();
        
        // Check that table was stored
        assert_eq!(strategy.tables.len(), 1);
        assert!(strategy.tables.contains_key("Canon"));
        
        let canon_tables = &strategy.tables["Canon"];
        assert_eq!(canon_tables.len(), 1);
        assert_eq!(canon_tables[0].name, "whiteBalance");
        assert_eq!(canon_tables[0].data.len(), 3);
    }
    
    #[test]
    fn test_generate_table_code() {
        let strategy = SimpleTableStrategy::new();
        
        let table = SimpleTable {
            name: "whiteBalance".to_string(),
            module: "Canon".to_string(),
            data: [
                ("0".to_string(), "Auto".to_string()),
                ("1".to_string(), "Daylight".to_string()),
            ].iter().cloned().collect(),
        };
        
        let code = strategy.generate_table_code(&table);
        
        // Check key components are present
        assert!(code.contains("static WHITE_BALANCE_DATA"));
        assert!(code.contains("pub static WHITE_BALANCE"));
        assert!(code.contains("pub fn lookup_white_balance"));
        assert!(code.contains("LazyLock<HashMap<u8"));
        assert!(code.contains("(0, \"Auto\")"));
        assert!(code.contains("(1, \"Daylight\")"));
    }
    
    #[test]
    fn test_key_type_inference() {
        let strategy = SimpleTableStrategy::new();
        
        // Numeric keys should infer appropriate integer type
        let numeric_data: HashMap<String, String> = [
            ("0".to_string(), "Auto".to_string()),
            ("255".to_string(), "Max".to_string()),
        ].iter().cloned().collect();
        assert_eq!(strategy.infer_key_type(&numeric_data), "u8");
        
        let large_numeric: HashMap<String, String> = [
            ("0".to_string(), "Auto".to_string()),
            ("65536".to_string(), "Large".to_string()),
        ].iter().cloned().collect();
        assert_eq!(strategy.infer_key_type(&large_numeric), "u32");
        
        // String keys should use &str
        let string_data: HashMap<String, String> = [
            ("auto".to_string(), "Automatic".to_string()),
            ("manual".to_string(), "Manual".to_string()),
        ].iter().cloned().collect();
        assert_eq!(strategy.infer_key_type(&string_data), "&str");
    }
    
    #[test]
    fn test_naming_conventions() {
        let strategy = SimpleTableStrategy::new();
        
        assert_eq!(output_locations::to_snake_case("WhiteBalance"), "white_balance");
        assert_eq!(output_locations::to_snake_case("canonWhiteBalance"), "canon_white_balance");
        assert_eq!(strategy.constant_case("whiteBalance"), "WHITE_BALANCE");
        assert_eq!(strategy.pascal_case("white_balance"), "WhiteBalance");
    }
}