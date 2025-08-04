//! BinaryDataStrategy - Processes ExifTool ProcessBinaryData table definitions
//!
//! This strategy recognizes and processes symbols that contain ProcessBinaryData
//! table characteristics (FIRST_ENTRY, FORMAT, etc.) and generates appropriate
//! Rust binary data parsing code.

use anyhow::{Context, Result};  
use serde_json::Value as JsonValue;
use std::process::Command;
use tracing::{debug, info, warn};

use super::{ExtractionStrategy, ExtractionContext, GeneratedFile};
use crate::field_extractor::FieldSymbol;
use crate::strategies::output_locations::generate_module_path;

/// Strategy for processing ProcessBinaryData table definitions
pub struct BinaryDataStrategy {
    /// Binary data tables to be generated
    pending_extractions: Vec<BinaryDataExtraction>,
}

#[derive(Debug, Clone)]
struct BinaryDataExtraction {
    module_name: String,
    module_path: String,
    table_name: String,
    symbol_name: String,
}

impl BinaryDataStrategy {
    pub fn new() -> Self {
        Self {
            pending_extractions: Vec::new(),
        }
    }
    
    /// Check if symbol contains ProcessBinaryData characteristics
    fn is_binary_data_symbol(symbol: &FieldSymbol) -> bool {
        if let Some(data) = symbol.data.as_object() {
            // ProcessBinaryData tables have these characteristic fields
            let has_first_entry = data.contains_key("FIRST_ENTRY");
            let has_format = data.contains_key("FORMAT");
            let has_priority = data.contains_key("PRIORITY");  
            let has_writable = data.contains_key("WRITABLE");
            
            // ProcessBinaryData indicator pattern
            if has_first_entry && (has_format || has_priority) {
                return true;
            }
            
            // Common ProcessBinaryData table name patterns
            let binary_patterns = [
                "CameraInfo", "ShotInfo", "ProcessingInfo", "ColorData", 
                "AFConfig", "AFInfo", "CameraSettings", "FFMV", "ColorBalance"
            ];
            
            for pattern in &binary_patterns {
                if symbol.name.contains(pattern) {
                    return true;
                }
            }
            
            // Tables with numeric tag definitions (offset -> name mappings)
            let has_numeric_tags = data.keys().any(|k| k.parse::<u32>().is_ok());
            if has_numeric_tags && (has_format || has_writable) {
                return true;
            }
        }
        
        false
    }
    
    /// Extract binary data using process_binary_data.pl extractor
    fn extract_binary_data(&self, extraction: &BinaryDataExtraction) -> Result<String> {
        debug!("Extracting binary data: {}::{}", extraction.module_name, extraction.table_name);
        
        let output = Command::new("perl")
            .arg("extractors/process_binary_data.pl")
            .arg(&extraction.module_path)
            .arg(&extraction.table_name)
            .current_dir(".")
            .output()
            .with_context(|| format!("Failed to run process_binary_data.pl for {}::{}", 
                                   extraction.module_name, extraction.table_name))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("process_binary_data.pl failed: {}", stderr));
        }

        let stdout = String::from_utf8(output.stdout)?;
        Ok(stdout)
    }
    
    /// Generate Rust code from binary data JSON
    fn generate_binary_data_code(&self, json_data: &str, extraction: &BinaryDataExtraction) -> Result<String> {
        let binary_data: JsonValue = serde_json::from_str(json_data)?;
        
        let entries = binary_data["entries"].as_array()
            .ok_or_else(|| anyhow::anyhow!("No entries array found"))?;
            
        if entries.is_empty() {
            return Ok(String::new()); // Skip empty tables
        }
        
        let format = binary_data["format"].as_str().unwrap_or("int8u");
        let first_entry = binary_data["first_entry"].as_u64().unwrap_or(0);
        
        let mut code = String::new();
        code.push_str("//! Generated binary data parsing definitions\n");
        code.push_str("//!\n");
        code.push_str(&format!("//! Extracted from {}::{}\n", 
                              extraction.module_name, extraction.table_name));
        code.push_str("\n");
        code.push_str("use std::sync::LazyLock;\n");
        code.push_str("use std::collections::HashMap;\n");
        code.push_str("use crate::types::BinaryDataEntry;\n");
        code.push_str("\n");
        
        // Generate binary data table metadata
        code.push_str(&format!("/// Binary data table: {} (format: {}, first_entry: {})\n", 
                              extraction.table_name, format, first_entry));
        code.push_str(&format!("pub static {}_BINARY_DATA: LazyLock<HashMap<u16, BinaryDataEntry>> = LazyLock::new(|| {{\n", 
                              extraction.table_name.to_uppercase()));
        code.push_str("    let mut entries = HashMap::new();\n");
        
        for entry in entries {
            let offset = entry["offset"].as_u64().unwrap_or(0) as u16;
            let name = entry["name"].as_str().unwrap_or("Unknown");
            let format = entry["format"].as_str().unwrap_or("int8u");
            let count = entry["count"].as_u64().unwrap_or(1);
            
            code.push_str(&format!("    entries.insert({}, BinaryDataEntry {{\n", offset));
            code.push_str(&format!("        name: \"{}\",\n", name));
            code.push_str(&format!("        format: \"{}\",\n", format));
            code.push_str(&format!("        count: {},\n", count));
            code.push_str("        condition: None, // TODO: Implement conditions\n");
            code.push_str("    });\n");
        }
        
        code.push_str("    entries\n");
        code.push_str("});\n");
        code.push_str("\n");
        
        // Generate lookup function
        code.push_str(&format!("/// Look up binary data entry for {} table\n", extraction.table_name));
        code.push_str(&format!("pub fn lookup_{}_entry(offset: u16) -> Option<&'static BinaryDataEntry> {{\n", 
                              extraction.table_name.to_lowercase()));
        code.push_str(&format!("    {}_BINARY_DATA.get(&offset)\n", extraction.table_name.to_uppercase()));
        code.push_str("}\n");
        
        // Generate parsing function
        code.push_str("\n");
        code.push_str(&format!("/// Parse {} binary data from buffer\n", extraction.table_name));
        code.push_str(&format!("pub fn parse_{}_data(data: &[u8]) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {{\n", 
                              extraction.table_name.to_lowercase()));
        code.push_str("    let mut result = HashMap::new();\n");
        code.push_str("    \n");
        code.push_str(&format!("    for (offset, entry) in {}_BINARY_DATA.iter() {{\n", 
                              extraction.table_name.to_uppercase()));
        code.push_str("        let pos = *offset as usize;\n");
        code.push_str("        if pos < data.len() {\n");
        code.push_str("            // TODO: Implement format-specific parsing\n");
        code.push_str("            let value = format!(\"0x{:02X}\", data[pos]);\n");
        code.push_str("            result.insert(entry.name.to_string(), value);\n");
        code.push_str("        }\n");
        code.push_str("    }\n");
        code.push_str("    \n");
        code.push_str("    Ok(result)\n");
        code.push_str("}\n");
        
        Ok(code)
    }
}

impl ExtractionStrategy for BinaryDataStrategy {
    fn name(&self) -> &'static str {
        "BinaryDataStrategy"
    }
    
    fn can_handle(&self, symbol: &FieldSymbol) -> bool {
        Self::is_binary_data_symbol(symbol)
    }
    
    fn extract(&mut self, symbol_data: &FieldSymbol, context: &mut ExtractionContext) -> Result<()> {
        context.log_strategy_selection(symbol_data, self.name(), 
            "Detected ProcessBinaryData table with FIRST_ENTRY/FORMAT or binary table name pattern");
        
        // Build path to ExifTool module
        let module_path = format!("../third-party/exiftool/lib/Image/ExifTool/{}.pm", 
                                symbol_data.module);
        
        // Queue extraction for processing
        let extraction = BinaryDataExtraction {
            module_name: symbol_data.module.clone(),
            module_path,
            table_name: symbol_data.name.clone(),
            symbol_name: symbol_data.name.clone(),
        };
        
        debug!("Queued BinaryData extraction: {}::{}", extraction.module_name, extraction.table_name);
        self.pending_extractions.push(extraction);
        
        Ok(())
    }
    
    fn finish_module(&mut self, _module_name: &str) -> Result<()> {
        // Nothing to do per-module
        Ok(())
    }
    
    fn finish_extraction(&mut self) -> Result<Vec<GeneratedFile>> {
        let mut files = Vec::new();
        
        info!("Processing {} queued BinaryData extractions", self.pending_extractions.len());
        
        for extraction in &self.pending_extractions {
            match self.extract_binary_data(extraction) {
                Ok(json_data) => {
                    match self.generate_binary_data_code(&json_data, extraction) {
                        Ok(code) => {
                            if !code.trim().is_empty() {
                                let path = generate_module_path(&extraction.module_name, 
                                    &format!("{}_binary_data", extraction.table_name.to_lowercase()));
                                
                                files.push(GeneratedFile {
                                    path,
                                    content: code,
                                    strategy: self.name().to_string(),
                                });
                                
                                debug!("Generated BinaryData code for {}::{}", 
                                      extraction.module_name, extraction.table_name);
                            }
                        }
                        Err(e) => {
                            warn!("Failed to generate code for {}::{}: {}", 
                                 extraction.module_name, extraction.table_name, e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to extract binary data {}::{}: {}", 
                         extraction.module_name, extraction.table_name, e);
                }
            }
        }
        
        info!("BinaryDataStrategy generated {} files", files.len());
        Ok(files)
    }
}