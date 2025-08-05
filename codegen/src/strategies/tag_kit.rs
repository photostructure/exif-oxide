//! TagKitStrategy - Processes ExifTool tag table definitions
//!
//! This strategy recognizes and processes hash symbols that contain tag definitions
//! with Names, PrintConv, ValueConv, etc. - the complex structures that tag_kit.pl
//! extracts from ExifTool modules.

use anyhow::{Context, Result};
use serde_json::Value as JsonValue;
use std::process::Command;
use tracing::{debug, info, warn};

use super::{ExtractionStrategy, ExtractionContext, GeneratedFile};
use crate::field_extractor::FieldSymbol;
use crate::strategies::output_locations::generate_module_path;

/// Strategy for processing tag table definitions (Main, Composite, etc.)
pub struct TagKitStrategy {
    /// Tag kits to be generated per module
    pending_extractions: Vec<TagKitExtraction>,
}

#[derive(Debug, Clone)]
struct TagKitExtraction {
    module_name: String,
    module_path: String,
    table_name: String,
    symbol_name: String,
}

impl TagKitStrategy {
    pub fn new() -> Self {
        Self {
            pending_extractions: Vec::new(),
        }
    }
    
    /// Check if symbol contains tag definition patterns
    fn is_tag_table_symbol(symbol: &FieldSymbol) -> bool {
        // Look for common tag table indicators in the data
        if let Some(data) = symbol.data.as_object() {
            // Check for ExifTool tag table characteristics
            let has_writable = data.contains_key("WRITABLE");
            let has_groups = data.contains_key("GROUPS");
            let has_notes = data.contains_key("NOTES");
            let has_write_group = data.contains_key("WRITE_GROUP");
            
            // Tag tables often have these metadata fields
            if has_writable || has_groups || has_write_group {
                return true;
            }
            
            // Common tag table names
            let common_tables = ["Main", "Composite", "Extra", "Image"];
            if common_tables.contains(&symbol.name.as_str()) {
                return true;
            }
            
            // Large hash with potential tag definitions - check complexity
            if symbol.metadata.complexity == "composite" || symbol.metadata.size > 50 {
                // Complex or large structures often indicate tag definitions
                return true;
            }
        }
        
        false
    }
    
    /// Extract tag kits using tag_kit.pl extractor
    fn extract_tag_kit(&self, extraction: &TagKitExtraction) -> Result<String> {
        debug!("Extracting tag kit: {}::{}", extraction.module_name, extraction.table_name);
        
        let output = Command::new("perl")
            .arg("extractors/tag_kit.pl")
            .arg(&extraction.module_path)
            .arg(&extraction.table_name)
            .current_dir(".")
            .output()
            .with_context(|| format!("Failed to run tag_kit.pl for {}::{}", 
                                   extraction.module_name, extraction.table_name))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("tag_kit.pl failed: {}", stderr));
        }

        let stdout = String::from_utf8(output.stdout)?;
        Ok(stdout)
    }
    
    /// Generate Rust code from tag kit JSON
    fn generate_tag_kit_code(&self, json_data: &str, extraction: &TagKitExtraction) -> Result<String> {
        let tag_kit_data: JsonValue = serde_json::from_str(json_data)?;
        
        let tag_kits = tag_kit_data["tag_kits"].as_array()
            .ok_or_else(|| anyhow::anyhow!("No tag_kits array found"))?;
            
        if tag_kits.is_empty() {
            return Ok(String::new()); // Skip empty tables
        }
        
        let mut code = String::new();
        code.push_str("//! Generated tag kit definitions\n");
        code.push_str("//!\n");
        code.push_str(&format!("//! Extracted from {}::{}\n", 
                              extraction.module_name, extraction.table_name));
        code.push_str("\n");
        code.push_str("use std::sync::LazyLock;\n");
        code.push_str("use std::collections::HashMap;\n");
        code.push_str("use crate::types::TagInfo;\n");
        code.push_str("\n");
        
        // Generate tag definitions
        code.push_str(&format!("/// Tag definitions for {} table\n", extraction.table_name));
        code.push_str(&format!("pub static {}_TAGS: LazyLock<HashMap<u16, TagInfo>> = LazyLock::new(|| {{\n", 
                              extraction.table_name.to_uppercase()));
        code.push_str("    let mut tags = HashMap::new();\n");
        
        for tag_kit in tag_kits {
            let tag_id = tag_kit["tag_id"].as_str().unwrap_or("0");
            let name = tag_kit["name"].as_str().unwrap_or("Unknown");
            let format = tag_kit["format"].as_str().unwrap_or("unknown");
            
            // Parse tag_id (could be hex string)
            let id_value = if tag_id.starts_with("0x") || tag_id.starts_with("0X") {
                u16::from_str_radix(&tag_id[2..], 16).unwrap_or(0)
            } else {
                tag_id.parse::<u16>().unwrap_or(0)
            };
            
            code.push_str(&format!("    tags.insert({}, TagInfo {{\n", id_value));
            code.push_str(&format!("        name: \"{}\",\n", name));
            code.push_str(&format!("        format: \"{}\",\n", format));
            code.push_str("        print_conv: None, // TODO: Implement PrintConv processing\n");
            code.push_str("    });\n");
        }
        
        code.push_str("    tags\n");
        code.push_str("});\n");
        code.push_str("\n");
        
        // Generate lookup function
        code.push_str(&format!("/// Look up tag information for {} table\n", extraction.table_name));
        code.push_str(&format!("pub fn lookup_{}_tag(tag_id: u16) -> Option<&'static TagInfo> {{\n", 
                              extraction.table_name.to_lowercase()));
        code.push_str(&format!("    {}_TAGS.get(&tag_id)\n", extraction.table_name.to_uppercase()));
        code.push_str("}\n");
        
        Ok(code)
    }
}

impl ExtractionStrategy for TagKitStrategy {
    fn name(&self) -> &'static str {
        "TagKitStrategy"
    }
    
    fn can_handle(&self, symbol: &FieldSymbol) -> bool {
        let result = Self::is_tag_table_symbol(symbol);
        debug!("TagKitStrategy::can_handle({}) -> {}", symbol.name, result);
        result
    }
    
    fn extract(&mut self, symbol_data: &FieldSymbol, context: &mut ExtractionContext) -> Result<()> {
        context.log_strategy_selection(symbol_data, self.name(), 
            "Detected tag table with WRITABLE/GROUPS or common table name");
        
        // Build path to ExifTool module
        let module_path = format!("../third-party/exiftool/lib/Image/ExifTool/{}.pm", 
                                symbol_data.module);
        
        // Queue extraction for processing
        let extraction = TagKitExtraction {
            module_name: symbol_data.module.clone(),
            module_path,
            table_name: symbol_data.name.clone(),
            symbol_name: symbol_data.name.clone(),
        };
        
        debug!("Queued TagKit extraction: {}::{}", extraction.module_name, extraction.table_name);
        self.pending_extractions.push(extraction);
        
        Ok(())
    }
    
    fn finish_module(&mut self, _module_name: &str) -> Result<()> {
        // Nothing to do per-module
        Ok(())
    }
    
    fn finish_extraction(&mut self) -> Result<Vec<GeneratedFile>> {
        let mut files = Vec::new();
        
        info!("Processing {} queued TagKit extractions", self.pending_extractions.len());
        
        for extraction in &self.pending_extractions {
            match self.extract_tag_kit(extraction) {
                Ok(json_data) => {
                    match self.generate_tag_kit_code(&json_data, extraction) {
                        Ok(code) => {
                            if !code.trim().is_empty() {
                                let path = generate_module_path(&extraction.module_name, 
                                    &format!("{}_tags", extraction.table_name.to_lowercase()));
                                
                                files.push(GeneratedFile {
                                    path,
                                    content: code,
                                    strategy: self.name().to_string(),
                                });
                                
                                debug!("Generated TagKit code for {}::{}", 
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
                    warn!("Failed to extract tag kit {}::{}: {}", 
                         extraction.module_name, extraction.table_name, e);
                }
            }
        }
        
        info!("TagKitStrategy generated {} files", files.len());
        Ok(files)
    }
}