//! FileTypeLookupStrategy for processing ExifTool's discriminated union file type patterns
//!
//! This strategy handles the %fileTypeLookup symbol which uses a discriminated union pattern:
//! - String values are aliases: "3GP2" → "3G2" 
//! - Array values are definitions: "JPEG" → ["JPEG", "Joint Photographic Experts Group"]
//! - Array values can have multiple formats: "AI" → [["PDF","PS"], "Adobe Illustrator"]

use anyhow::Result;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use tracing::{debug, info};

use super::{ExtractionContext, ExtractionStrategy, GeneratedFile};
use crate::field_extractor::FieldSymbol;

/// Strategy for processing ExifTool's fileTypeLookup discriminated union
pub struct FileTypeLookupStrategy {
    /// Collected file type data by module
    file_type_data: HashMap<String, FileTypeLookupData>,
}

/// File type lookup data extracted from ExifTool
#[derive(Debug, Clone)]
struct FileTypeLookupData {
    /// Symbol name (should be "fileTypeLookup")
    name: String,
    
    /// Module name (should be "ExifTool") 
    module: String,
    
    /// Extension aliases (string → string mappings)
    aliases: HashMap<String, String>,
    
    /// File type definitions (extension → (formats, description) mappings)
    definitions: HashMap<String, (Vec<String>, String)>,
}

impl FileTypeLookupStrategy {
    /// Create new FileTypeLookupStrategy
    pub fn new() -> Self {
        Self {
            file_type_data: HashMap::new(),
        }
    }
    
    /// Extract discriminated union data from symbol
    fn parse_file_type_data(&self, data: &JsonValue) -> Option<(HashMap<String, String>, HashMap<String, (Vec<String>, String)>)> {
        let map = data.as_object()?;
        let mut aliases = HashMap::new();
        let mut definitions = HashMap::new();
        
        for (ext, value) in map {
            match value {
                JsonValue::String(alias) => {
                    // Simple alias: "3GP2" → "3G2"
                    aliases.insert(ext.clone(), alias.clone());
                }
                JsonValue::Array(array) => {
                    // Definition array: ["format", "description"] or [["fmt1", "fmt2"], "description"]
                    if array.len() >= 2 {
                        let formats = match &array[0] {
                            JsonValue::String(single_format) => {
                                vec![single_format.clone()]
                            }
                            JsonValue::Array(format_array) => {
                                format_array.iter()
                                    .filter_map(|v| v.as_str())
                                    .map(|s| s.to_string())
                                    .collect()
                            }
                            _ => continue, // Skip malformed entries
                        };
                        
                        if let Some(description) = array[1].as_str() {
                            definitions.insert(ext.clone(), (formats, description.to_string()));
                        }
                    }
                }
                _ => {
                    // Skip other value types
                    continue;
                }
            }
        }
        
        Some((aliases, definitions))
    }
    
    /// Generate Rust code for file type lookup
    fn generate_file_type_code(&self, data: &FileTypeLookupData) -> String {
        let mut code = String::new();
        
        // File header
        code.push_str("//! File type lookup tables generated from ExifTool's fileTypeLookup hash\n");
        code.push_str("//!\n");
        code.push_str(&format!("//! Total aliases: {}\n", data.aliases.len()));
        code.push_str(&format!("//! Total definitions: {}\n", data.definitions.len()));
        code.push_str("//! Source: ExifTool.pm %fileTypeLookup\n\n");
        
        // Imports
        code.push_str("use std::collections::HashMap;\n");
        code.push_str("use std::sync::LazyLock;\n\n");
        
        // Extension aliases static table
        code.push_str("/// Extension aliases - maps extensions to their canonical forms\n");
        code.push_str("static EXTENSION_ALIASES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {\n");
        code.push_str("    let mut map = HashMap::new();\n");
        
        // Sort aliases for consistent output
        let mut aliases: Vec<_> = data.aliases.iter().collect();
        aliases.sort_by_key(|&(k, _)| k);
        
        for (ext, alias) in aliases {
            code.push_str(&format!("    map.insert(\"{}\", \"{}\");\n", ext, alias));
        }
        
        code.push_str("    map\n");
        code.push_str("});\n\n");
        
        // File type definitions static table
        code.push_str("/// File type definitions - maps file types to their format descriptions\n");
        code.push_str("static FILE_TYPE_FORMATS: LazyLock<HashMap<&'static str, (Vec<&'static str>, &'static str)>> = LazyLock::new(|| {\n");
        code.push_str("    let mut map = HashMap::new();\n");
        
        // Sort definitions for consistent output
        let mut definitions: Vec<_> = data.definitions.iter().collect();
        definitions.sort_by_key(|&(k, _)| k);
        
        for (ext, (formats, desc)) in definitions {
            let format_list: Vec<String> = formats.iter().map(|f| format!("\"{}\"", f)).collect();
            let escaped_desc = desc.replace('\\', "\\\\").replace('"', "\\\"");
            code.push_str(&format!(
                "    map.insert(\"{}\", (vec![{}], \"{}\"));\n",
                ext, format_list.join(", "), escaped_desc
            ));
        }
        
        code.push_str("    map\n");
        code.push_str("});\n\n");
        
        // Public API functions
        self.generate_api_functions(&mut code);
        
        code
    }
    
    /// Generate public API functions
    fn generate_api_functions(&self, code: &mut String) {
        // resolve_file_type - main resolution function with alias following
        code.push_str("/// Resolve file type from extension, following aliases\n");
        code.push_str("/// Returns (formats, description) tuple if found\n");
        code.push_str("pub fn resolve_file_type(extension: &str) -> Option<(Vec<&'static str>, &'static str)> {\n");
        code.push_str("    const MAX_ALIAS_DEPTH: u8 = 10; // Prevent infinite loops\n\n");
        code.push_str("    let mut current_ext = extension.to_uppercase();\n");
        code.push_str("    let mut depth = 0;\n\n");
        code.push_str("    while depth < MAX_ALIAS_DEPTH {\n");
        code.push_str("        // Check for direct format lookup\n");
        code.push_str("        if let Some((formats, desc)) = FILE_TYPE_FORMATS.get(current_ext.as_str()) {\n");
        code.push_str("            return Some((formats.clone(), *desc));\n");
        code.push_str("        }\n\n");
        code.push_str("        // Check for alias resolution\n");
        code.push_str("        if let Some(alias) = EXTENSION_ALIASES.get(current_ext.as_str()) {\n");
        code.push_str("            current_ext = alias.to_uppercase();\n");
        code.push_str("            depth += 1;\n");
        code.push_str("        } else {\n");
        code.push_str("            break;\n");
        code.push_str("        }\n");
        code.push_str("    }\n\n");
        code.push_str("    None // Not found or circular alias chain\n");
        code.push_str("}\n\n");
        
        // get_primary_format - convenience function
        code.push_str("/// Get primary format for a file type\n");
        code.push_str("pub fn get_primary_format(file_type: &str) -> Option<String> {\n");
        code.push_str("    resolve_file_type(file_type)\n");
        code.push_str("        .map(|(formats, _)| formats[0].to_string())\n");
        code.push_str("}\n\n");
        
        // supports_format - format compatibility check
        code.push_str("/// Check if a file type supports a specific format\n");
        code.push_str("pub fn supports_format(file_type: &str, format: &str) -> bool {\n");
        code.push_str("    resolve_file_type(file_type)\n");
        code.push_str("        .map(|(formats, _)| formats.contains(&format))\n");
        code.push_str("        .unwrap_or(false)\n");
        code.push_str("}\n\n");
        
        // extensions_for_format - reverse lookup
        code.push_str("/// Get all extensions that support a specific format\n");
        code.push_str("pub fn extensions_for_format(target_format: &str) -> Vec<String> {\n");
        code.push_str("    FILE_TYPE_FORMATS\n");
        code.push_str("        .iter()\n");
        code.push_str("        .filter_map(|(ext, (formats, _))| {\n");
        code.push_str("            if formats.contains(&target_format) {\n");
        code.push_str("                Some(ext.to_string())\n");
        code.push_str("            } else {\n");
        code.push_str("                None\n");
        code.push_str("            }\n");
        code.push_str("        })\n");
        code.push_str("        .collect()\n");
        code.push_str("}\n\n");
        
        // FILE_TYPE_EXTENSIONS - expected by src/file_detection.rs
        code.push_str("/// All known file type extensions\n");
        code.push_str("pub static FILE_TYPE_EXTENSIONS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {\n");
        code.push_str("    let mut extensions = Vec::new();\n\n");
        code.push_str("    // Add all extensions from format definitions\n");
        code.push_str("    for ext in FILE_TYPE_FORMATS.keys() {\n");
        code.push_str("        extensions.push(*ext);\n");
        code.push_str("    }\n\n");
        code.push_str("    // Add all extension aliases\n");
        code.push_str("    for ext in EXTENSION_ALIASES.keys() {\n");
        code.push_str("        extensions.push(*ext);\n");
        code.push_str("    }\n\n");
        code.push_str("    extensions.sort();\n");
        code.push_str("    extensions.dedup();\n");
        code.push_str("    extensions\n");
        code.push_str("});\n\n");
        
        // lookup_file_type_by_extension - compatibility wrapper
        code.push_str("/// Lookup file type by extension (wrapper around resolve_file_type)\n");
        code.push_str("/// Returns the first format for compatibility with existing code\n");
        code.push_str("pub fn lookup_file_type_by_extension(extension: &str) -> Option<String> {\n");
        code.push_str("    resolve_file_type(extension)\n");
        code.push_str("        .map(|(formats, _)| formats[0].to_string())\n");
        code.push_str("}\n");
    }
}

impl ExtractionStrategy for FileTypeLookupStrategy {
    fn name(&self) -> &'static str {
        "FileTypeLookupStrategy"
    }
    
    fn can_handle(&self, symbol: &FieldSymbol) -> bool {
        // Only handle fileTypeLookup from ExifTool module
        if symbol.name != "fileTypeLookup" || symbol.module != "ExifTool" {
            return false;
        }
        
        // Must be a hash with discriminated union pattern
        if let JsonValue::Object(map) = &symbol.data {
            if map.is_empty() {
                return false;
            }
            
            // Check for discriminated union pattern: some values are strings (aliases), 
            // others are arrays (definitions)
            let has_string_values = map.values().any(|v| v.is_string());
            let has_array_values = map.values().any(|v| v.is_array());
            
            debug!("fileTypeLookup pattern check: strings={}, arrays={}", has_string_values, has_array_values);
            
            // Must have both strings (aliases) and arrays (definitions) to be discriminated union
            has_string_values && has_array_values
        } else {
            false
        }
    }
    
    fn extract(&mut self, symbol: &FieldSymbol, context: &mut ExtractionContext) -> Result<()> {
        info!("🔧 Extracting fileTypeLookup with discriminated union pattern");
        
        // Parse the discriminated union data
        if let Some((aliases, definitions)) = self.parse_file_type_data(&symbol.data) {
            let data = FileTypeLookupData {
                name: symbol.name.clone(),
                module: symbol.module.clone(),
                aliases,
                definitions,
            };
            
            info!("    ✓ Parsed {} aliases and {} definitions", 
                  data.aliases.len(), data.definitions.len());
            
            self.file_type_data.insert(symbol.module.clone(), data);
            
            context.log_strategy_selection(symbol, self.name(), 
                "Discriminated union pattern: string aliases + array definitions");
        }
        
        Ok(())
    }
    
    fn finish_module(&mut self, _module_name: &str) -> Result<()> {
        // No per-module finalization needed
        Ok(())
    }
    
    fn finish_extraction(&mut self) -> Result<Vec<GeneratedFile>> {
        let mut files = Vec::new();
        
        for (_module, data) in &self.file_type_data {
            let content = self.generate_file_type_code(data);
            
            files.push(GeneratedFile {
                path: "file_types/file_type_lookup.rs".to_string(),
                content,
                strategy: self.name().to_string(),
            });
            
            info!("📁 Generated file_type_lookup.rs with {} aliases and {} definitions", 
                  data.aliases.len(), data.definitions.len());
        }
        
        Ok(files)
    }
}

impl Default for FileTypeLookupStrategy {
    fn default() -> Self {
        Self::new()
    }
}