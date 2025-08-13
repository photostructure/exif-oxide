//! Field extractor parsing and integration module
//!
//! This module handles parsing JSON Lines output from the field_extractor.pl
//! script and provides structures for strategy pattern dispatch.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::path::Path;
use std::process::{Command, Stdio};
use tracing::{info, trace, warn};

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

    /// Flag indicating if this is a composite table (set when AddCompositeTags was called)
    #[serde(default)]
    pub is_composite_table: u8,
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
            script_path: Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("scripts/field_extractor.pl")
                .to_string_lossy()
                .into_owned(),
        }
    }

    /// Extract all symbols from a module and return parsed results
    pub fn extract_module(&self, module_path: &Path) -> Result<Vec<FieldSymbol>> {
        let extract_start = std::time::Instant::now();
        info!("Extracting symbols from module: {}", module_path.display());
        trace!(
            "🚀 Starting field extraction for: {}",
            module_path.display()
        );

        // Run the field extractor and capture all output at once
        let perl_start = std::time::Instant::now();
        trace!("🔧 Starting perl field extractor script");
        let output = Command::new("perl")
            .arg(&self.script_path)
            .arg(module_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .with_context(|| {
                format!(
                    "Failed to execute field extractor for {}",
                    module_path.display()
                )
            })?;

        let perl_time = perl_start.elapsed();
        trace!("⚡ Perl script completed in {:.2}ms", perl_time.as_millis());

        // Check if the process succeeded
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "Field extractor failed after {:.2}ms: {}",
                perl_time.as_millis(),
                stderr
            ));
        }

        // Parse JSON Lines output from stdout
        let parse_start = std::time::Instant::now();
        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let mut symbols = Vec::new();
        let line_count = stdout_str.lines().count();
        trace!("📋 Parsing {} lines of JSON output", line_count);

        for line in stdout_str.lines() {
            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            // Parse JSON line
            match serde_json::from_str::<FieldSymbol>(line) {
                Ok(symbol) => {
                    trace!(
                        "✓ Parsed symbol: {} ({}, size: {})",
                        symbol.name,
                        symbol.symbol_type,
                        symbol.metadata.size
                    );
                    symbols.push(symbol);
                }
                Err(e) => {
                    warn!("Failed to parse JSON line: {} - Error: {}", line, e);
                    // Continue processing other lines rather than failing
                }
            }
        }

        let parse_time = parse_start.elapsed();
        trace!(
            "📊 JSON parsing completed in {:.2}ms",
            parse_time.as_millis()
        );

        let total_time = extract_start.elapsed();
        let symbol_count = symbols.len();

        info!(
            "Field extraction completed: {} symbols extracted in {:.2}ms (perl: {:.1}ms, parse: {:.1}ms)",
            symbol_count,
            total_time.as_millis(), perl_time.as_millis(), parse_time.as_millis()
        );
        trace!(
            "🏁 Total extraction time breakdown: perl={:.1}%, parse={:.1}%",
            (perl_time.as_millis() as f64 / total_time.as_millis() as f64) * 100.0,
            (parse_time.as_millis() as f64 / total_time.as_millis() as f64) * 100.0
        );

        Ok(symbols)
    }
}

impl Default for FieldExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Removed unused imports

    #[test]
    fn test_parse_field_symbol_json() {
        let json = r#"{"type":"hash","name":"canonWhiteBalance","data":{"0":"Auto","1":"Daylight","2":"Cloudy"},"module":"Canon","metadata":{"size":3,"complexity":"simple"}}"#;

        let symbol: FieldSymbol = serde_json::from_str(json).unwrap();

        assert_eq!(symbol.symbol_type, "hash");
        assert_eq!(symbol.name, "canonWhiteBalance");
        assert_eq!(symbol.module, "Canon");
        assert_eq!(symbol.metadata.size, 3);
        assert_eq!(symbol.metadata.is_composite_table, 0);

        // Check data content
        if let JsonValue::Object(data) = symbol.data {
            assert_eq!(data.get("0").unwrap().as_str().unwrap(), "Auto");
            assert_eq!(data.get("1").unwrap().as_str().unwrap(), "Daylight");
        } else {
            panic!("Expected object data");
        }
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
