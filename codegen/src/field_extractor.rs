//! Field extractor parsing and integration module
//!
//! This module handles parsing JSON Lines output from the field_extractor.pl
//! script and provides structures for strategy pattern dispatch.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Stdio};
use tracing::{debug, info, warn};

/// Symbol extracted from ExifTool module via field symbol table introspection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSymbol {
    /// Symbol type: "hash", "array", or "scalar"
    #[serde(rename = "type")]
    pub symbol_type: String,
    
    /// Symbol name as it appears in the Perl module
    pub name: String,
    
    /// Raw data extracted from the symbol table
    pub data: JsonValue,
    
    /// Module name (e.g., "Canon", "DNG", "ExifTool")
    pub module: String,
    
    /// Metadata about the symbol for strategy pattern matching
    pub metadata: FieldMetadata,
}

/// Metadata about extracted symbols for strategy pattern matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMetadata {
    /// Estimated size (hash key count, array length, or string length)
    pub size: u32,
    
    /// Complexity assessment: "simple", "composite", or "scalar"
    pub complexity: String,
}

/// Statistics from field extraction process
#[derive(Debug, Clone)]
pub struct FieldExtractionStats {
    pub total_symbols: u32,
    pub extracted_symbols: u32,
    pub skipped_symbols: u32,
    pub module_name: String,
}

/// Field extractor runner and parser
pub struct FieldExtractor {
    /// Path to the field_extractor.pl script
    script_path: String,
}

impl FieldExtractor {
    /// Create new field extractor instance
    pub fn new() -> Self {
        Self {
            script_path: "scripts/field_extractor.pl".to_string(),
        }
    }
    
    /// Extract all symbols from a module and return parsed results
    pub fn extract_module(&self, module_path: &Path) -> Result<(Vec<FieldSymbol>, FieldExtractionStats)> {
        info!("Extracting symbols from module: {}", module_path.display());
        
        // Run the field extractor and capture all output at once
        let output = Command::new("perl")
            .arg(&self.script_path)
            .arg(module_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .with_context(|| format!("Failed to execute field extractor for {}", module_path.display()))?;
        
        // Check if the process succeeded
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Field extractor failed: {}", stderr));
        }
        
        // Parse JSON Lines output from stdout
        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let mut symbols = Vec::new();
        
        for line in stdout_str.lines() {
            let line = line.trim();
            
            if line.is_empty() {
                continue;
            }
            
            // Parse JSON line
            match serde_json::from_str::<FieldSymbol>(line) {
                Ok(symbol) => {
                    debug!("Extracted symbol: {} ({})", symbol.name, symbol.symbol_type);
                    symbols.push(symbol);
                }
                Err(e) => {
                    warn!("Failed to parse JSON line: {} - Error: {}", line, e);
                    // Continue processing other lines rather than failing
                }
            }
        }
        
        // Parse extraction statistics from stderr
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stats = parse_extraction_stats(&stderr)?;
        
        info!("Field extraction completed: {} symbols extracted from {} total", 
              stats.extracted_symbols, stats.total_symbols);
        
        Ok((symbols, stats))
    }
    
    /// Extract symbols from multiple modules in parallel
    pub fn extract_modules(&self, module_paths: &[&Path]) -> Result<HashMap<String, (Vec<FieldSymbol>, FieldExtractionStats)>> {
        use rayon::prelude::*;
        
        let results: Result<Vec<_>> = module_paths
            .par_iter()
            .map(|path| {
                let (symbols, stats) = self.extract_module(path)?;
                Ok((stats.module_name.clone(), (symbols, stats)))
            })
            .collect();
        
        results.map(|pairs| pairs.into_iter().collect())
    }
}

impl Default for FieldExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse extraction statistics from stderr output
fn parse_extraction_stats(stderr: &str) -> Result<FieldExtractionStats> {
    let mut total_symbols = 0;
    let mut extracted_symbols = 0;
    let mut skipped_symbols = 0;
    let mut module_name = "unknown".to_string();
    
    for line in stderr.lines() {
        if line.starts_with("Field extraction complete for ") {
            module_name = line
                .strip_prefix("Field extraction complete for ")
                .unwrap_or("unknown")
                .trim_end_matches(':')
                .to_string();
        } else if line.contains("Total symbols examined:") {
            if let Some(num_str) = line.split(':').nth(1) {
                total_symbols = num_str.trim().parse()?;
            }
        } else if line.contains("Successfully extracted:") {
            if let Some(num_str) = line.split(':').nth(1) {
                extracted_symbols = num_str.trim().parse()?;
            }
        } else if line.contains("Skipped (non-serializable):") {
            if let Some(num_str) = line.split(':').nth(1) {
                skipped_symbols = num_str.trim().parse()?;
            }
        }
    }
    
    Ok(FieldExtractionStats {
        total_symbols,
        extracted_symbols,
        skipped_symbols,
        module_name,
    })
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[test]
    fn test_parse_field_symbol_json() {
        let json = r#"{"type":"hash","name":"canonWhiteBalance","data":{"0":"Auto","1":"Daylight","2":"Cloudy"},"module":"Canon","metadata":{"size":3,"complexity":"simple"}}"#;
        
        let symbol: FieldSymbol = serde_json::from_str(json).unwrap();
        
        assert_eq!(symbol.symbol_type, "hash");
        assert_eq!(symbol.name, "canonWhiteBalance");
        assert_eq!(symbol.module, "Canon");
        assert_eq!(symbol.metadata.size, 3);
        assert_eq!(symbol.metadata.complexity, "simple");
        
        // Check data content
        if let JsonValue::Object(data) = symbol.data {
            assert_eq!(data.get("0").unwrap().as_str().unwrap(), "Auto");
            assert_eq!(data.get("1").unwrap().as_str().unwrap(), "Daylight");
        } else {
            panic!("Expected object data");
        }
    }
    
    #[test]
    fn test_parse_extraction_stats() {
        let stderr = r#"Field extraction complete for Canon:
  Total symbols examined: 1500
  Successfully extracted: 873
  Skipped (non-serializable): 627
  Non-serializable entries logged to: generated/extract/non_serializable.log"#;
        
        let stats = parse_extraction_stats(stderr).unwrap();
        
        assert_eq!(stats.module_name, "Canon");
        assert_eq!(stats.total_symbols, 1500);
        assert_eq!(stats.extracted_symbols, 873);
        assert_eq!(stats.skipped_symbols, 627);
    }
    
    #[test]
    fn test_parse_array_symbol() {
        let json = r#"{"type":"array","name":"xlat_array","data":[193,191,109,158],"module":"Nikon","metadata":{"size":4,"complexity":"simple"}}"#;
        
        let symbol: FieldSymbol = serde_json::from_str(json).unwrap();
        
        assert_eq!(symbol.symbol_type, "array");
        assert_eq!(symbol.name, "xlat_array");
        assert_eq!(symbol.module, "Nikon");
        
        // Check array data
        if let JsonValue::Array(data) = symbol.data {
            assert_eq!(data.len(), 4);
            assert_eq!(data[0].as_u64().unwrap(), 193);
            assert_eq!(data[1].as_u64().unwrap(), 191);
        } else {
            panic!("Expected array data");
        }
    }
    
    #[test]
    fn test_parse_scalar_symbol() {
        let json = r#"{"type":"scalar","name":"VERSION","data":"1.25","module":"DNG","metadata":{"size":4,"complexity":"scalar"}}"#;
        
        let symbol: FieldSymbol = serde_json::from_str(json).unwrap();
        
        assert_eq!(symbol.symbol_type, "scalar");
        assert_eq!(symbol.name, "VERSION");
        assert_eq!(symbol.module, "DNG");
        assert_eq!(symbol.data.as_str().unwrap(), "1.25");
    }
}