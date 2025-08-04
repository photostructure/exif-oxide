//! CompositeTagStrategy - Processes ExifTool composite tag definitions
//!
//! This strategy recognizes and processes symbols that contain composite tag
//! definitions with dependencies and calculation logic.

use anyhow::{Context, Result};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info, warn};

use super::{ExtractionStrategy, ExtractionContext, GeneratedFile};
use crate::field_extractor::FieldSymbol;
use crate::strategies::output_locations::generate_module_path;

/// Strategy for processing composite tag definitions  
pub struct CompositeTagStrategy {
    /// Composite tags to be generated
    pending_extractions: Vec<CompositeExtraction>,
}

#[derive(Debug, Clone)]
struct CompositeExtraction {
    module_name: String,
    module_path: String,
    table_name: String,
    symbol_name: String,
}

impl CompositeTagStrategy {
    pub fn new() -> Self {
        Self {
            pending_extractions: Vec::new(),
        }
    }
    
    /// Check if symbol contains composite tag characteristics
    fn is_composite_symbol(symbol: &FieldSymbol) -> bool {
        // Composite table is always a composite symbol
        if symbol.name == "Composite" {
            return true;
        }
        
        if let Some(data) = symbol.data.as_object() {
            // Look for composite tag indicators
            let has_require = data.values().any(|v| {
                if let Some(obj) = v.as_object() {
                    obj.contains_key("Require") || obj.contains_key("Desire")
                } else {
                    false
                }
            });
            
            if has_require {
                return true;
            }
            
            // Look for composite-style calculations
            let has_calculations = data.values().any(|v| {
                if let Some(obj) = v.as_object() {
                    obj.contains_key("ValueConv") && obj.get("Groups").is_some()
                } else {
                    false
                }
            });
            
            if has_calculations {
                return true;
            }
        }
        
        false
    }
    
    /// Extract composite tags using composite_tags.pl extractor
    fn extract_composite_tags(&self, extraction: &CompositeExtraction) -> Result<String> {
        debug!("Extracting composite tags: {}::{}", extraction.module_name, extraction.table_name);
        
        let output = Command::new("perl")
            .arg("extractors/composite_tags.pl")
            .arg(&extraction.module_path)
            .arg(&extraction.table_name)
            .arg("--frequency-threshold")
            .arg("0.1") // Lower threshold for universal extraction
            .arg("--include-mainstream")
            .current_dir(".")
            .output()
            .with_context(|| format!("Failed to run composite_tags.pl for {}::{}", 
                                   extraction.module_name, extraction.table_name))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("composite_tags.pl failed: {}", stderr));
        }

        let stdout = String::from_utf8(output.stdout)?;
        Ok(stdout)
    }
    
    /// Generate Rust code from composite tags JSON
    fn generate_composite_code(&self, json_data: &str, extraction: &CompositeExtraction) -> Result<String> {
        let composite_data: JsonValue = serde_json::from_str(json_data)?;
        
        let composites = composite_data["composites"].as_array()
            .ok_or_else(|| anyhow::anyhow!("No composites array found"))?;
            
        if composites.is_empty() {
            return Ok(String::new()); // Skip empty tables
        }
        
        let mut code = String::new();
        code.push_str("//! Generated composite tag definitions\n");
        code.push_str("//!\n");
        code.push_str(&format!("//! Extracted from {}::{}\n", 
                              extraction.module_name, extraction.table_name));
        code.push_str("\n");
        code.push_str("use std::sync::LazyLock;\n");
        code.push_str("use std::collections::HashMap;\n");
        code.push_str("use crate::types::CompositeTagInfo;\n");
        code.push_str("\n");
        
        // Generate composite definitions
        code.push_str(&format!("/// Composite tag definitions for {} table\n", extraction.table_name));
        code.push_str(&format!("pub static {}_COMPOSITES: LazyLock<HashMap<&'static str, CompositeTagInfo>> = LazyLock::new(|| {{\n", 
                              extraction.table_name.to_uppercase()));
        code.push_str("    let mut tags = HashMap::new();\n");
        
        for composite in composites {
            let name = composite["name"].as_str().unwrap_or("Unknown");
            let empty_map = serde_json::Map::new();
            let groups = composite["groups"].as_object().unwrap_or(&empty_map);
            let require = composite["require"].as_array();
            let desire = composite["desire"].as_array();
            
            code.push_str(&format!("    tags.insert(\"{}\", CompositeTagInfo {{\n", name));
            code.push_str(&format!("        name: \"{}\",\n", name));
            
            // Generate groups
            code.push_str("        groups: {\n");
            code.push_str("            let mut g = HashMap::new();\n");
            for (group_num, group_name) in groups {
                if let Some(group_str) = group_name.as_str() {
                    code.push_str(&format!("            g.insert({}, \"{}\");\n", group_num, group_str));
                }
            }
            code.push_str("            g\n");
            code.push_str("        },\n");
            
            // Generate dependencies
            code.push_str("        require: vec![\n");
            if let Some(req_array) = require {
                for req in req_array {
                    if let Some(req_str) = req.as_str() {
                        code.push_str(&format!("            \"{}\",\n", req_str));
                    }
                }
            }
            code.push_str("        ],\n");
            
            code.push_str("        desire: vec![\n");
            if let Some(des_array) = desire {
                for des in des_array {
                    if let Some(des_str) = des.as_str() {
                        code.push_str(&format!("            \"{}\",\n", des_str));
                    }
                }
            }
            code.push_str("        ],\n");
            
            code.push_str("        value_conv: None, // TODO: Implement ValueConv processing\n");
            code.push_str("    });\n");
        }
        
        code.push_str("    tags\n");
        code.push_str("});\n");
        code.push_str("\n");
        
        // Generate lookup function
        code.push_str(&format!("/// Look up composite tag information for {} table\n", extraction.table_name));
        code.push_str(&format!("pub fn lookup_{}_composite(name: &str) -> Option<&'static CompositeTagInfo> {{\n", 
                              extraction.table_name.to_lowercase()));
        code.push_str(&format!("    {}_COMPOSITES.get(name)\n", extraction.table_name.to_uppercase()));
        code.push_str("}\n");
        
        // Generate calculation function stub
        code.push_str("\n");
        code.push_str(&format!("/// Calculate {} composite values\n", extraction.table_name));
        code.push_str(&format!("pub fn calculate_{}_composites(tags: &HashMap<String, String>) -> HashMap<String, String> {{\n", 
                              extraction.table_name.to_lowercase()));
        code.push_str("    let mut result = HashMap::new();\n");
        code.push_str("    \n");
        code.push_str(&format!("    for (name, composite) in {}_COMPOSITES.iter() {{\n", 
                              extraction.table_name.to_uppercase()));
        code.push_str("        // Check if all required tags are present\n");
        code.push_str("        let has_required = composite.require.iter().all(|req| tags.contains_key(*req));\n");
        code.push_str("        if has_required {\n");
        code.push_str("            // TODO: Implement actual composite calculation logic\n");
        code.push_str("            result.insert(name.to_string(), \"[Composite]\".to_string());\n");
        code.push_str("        }\n");
        code.push_str("    }\n");
        code.push_str("    \n");
        code.push_str("    result\n");
        code.push_str("}\n");
        
        Ok(code)
    }
}

impl ExtractionStrategy for CompositeTagStrategy {
    fn name(&self) -> &'static str {
        "CompositeTagStrategy"
    }
    
    fn can_handle(&self, symbol_data: &JsonValue) -> bool {
        // Convert JsonValue to FieldSymbol for analysis
        if let Ok(symbol) = serde_json::from_value::<FieldSymbol>(symbol_data.clone()) {
            Self::is_composite_symbol(&symbol)
        } else {
            false
        }
    }
    
    fn extract(&mut self, symbol_data: &FieldSymbol, context: &mut ExtractionContext) -> Result<()> {
        context.log_strategy_selection(symbol_data, self.name(), 
            "Detected Composite table or symbols with Require/Desire dependencies");
        
        // Build path to ExifTool module
        let module_path = format!("../third-party/exiftool/lib/Image/ExifTool/{}.pm", 
                                symbol_data.module);
        
        // Queue extraction for processing
        let extraction = CompositeExtraction {
            module_name: symbol_data.module.clone(),
            module_path,
            table_name: symbol_data.name.clone(),
            symbol_name: symbol_data.name.clone(),
        };
        
        debug!("Queued Composite extraction: {}::{}", extraction.module_name, extraction.table_name);
        self.pending_extractions.push(extraction);
        
        Ok(())
    }
    
    fn finish_module(&mut self, _module_name: &str) -> Result<()> {
        // Nothing to do per-module
        Ok(())
    }
    
    fn finish_extraction(&mut self) -> Result<Vec<GeneratedFile>> {
        let mut files = Vec::new();
        
        info!("Processing {} queued Composite extractions", self.pending_extractions.len());
        
        for extraction in &self.pending_extractions {
            match self.extract_composite_tags(extraction) {
                Ok(json_data) => {
                    match self.generate_composite_code(&json_data, extraction) {
                        Ok(code) => {
                            if !code.trim().is_empty() {
                                let path = generate_module_path(&extraction.module_name, 
                                    &format!("{}_composite", extraction.table_name.to_lowercase()));
                                
                                files.push(GeneratedFile {
                                    path,
                                    content: code,
                                    strategy: self.name().to_string(),
                                });
                                
                                debug!("Generated Composite code for {}::{}", 
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
                    warn!("Failed to extract composite tags {}::{}: {}", 
                         extraction.module_name, extraction.table_name, e);
                }
            }
        }
        
        info!("CompositeTagStrategy generated {} files", files.len());
        Ok(files)
    }
}