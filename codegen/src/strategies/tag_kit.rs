//! TagKitStrategy - Processes ExifTool tag table definitions
//!
//! This strategy recognizes and processes hash symbols that contain tag definitions
//! with Names, PrintConv, ValueConv, etc. from field_extractor.pl FieldSymbol data.

use anyhow::Result;
use serde_json::Value as JsonValue;
use tracing::{debug, info, warn};

use super::{ExtractionContext, ExtractionStrategy, GeneratedFile};
use crate::common::utils::escape_string;
use crate::conv_registry::{lookup_printconv, lookup_tag_specific_printconv};
use crate::field_extractor::FieldSymbol;
use crate::strategies::output_locations::generate_module_path;

/// Strategy for processing tag table definitions (Main, Composite, etc.)
pub struct TagKitStrategy {
    /// Tag table symbols processed per module
    processed_symbols: Vec<ProcessedTagTable>,
}

#[derive(Debug, Clone)]
struct ProcessedTagTable {
    module_name: String,
    table_name: String,
    symbol_data: JsonValue,
}

impl TagKitStrategy {
    pub fn new() -> Self {
        Self {
            processed_symbols: Vec::new(),
        }
    }

    /// Check if symbol contains tag definition patterns
    fn is_tag_table_symbol(symbol: &FieldSymbol) -> bool {
        // Don't claim Composite tables - let CompositeTagStrategy handle those
        // Check if this is marked as a composite table (has AddCompositeTags call)
        if symbol.metadata.is_composite_table == 1 {
            return false;
        }

        // Look for common tag table indicators in the data
        if let Some(data) = symbol.data.as_object() {
            // Check for ExifTool tag table characteristics
            let has_writable = data.contains_key("WRITABLE");
            let has_groups = data.contains_key("GROUPS");
            let _has_notes = data.contains_key("NOTES");
            let has_write_group = data.contains_key("WRITE_GROUP");

            // Tag tables often have these metadata fields
            if has_writable || has_groups || has_write_group {
                return true;
            }

            // Common tag table names (excluding Composite which we handle above)
            let common_tables = ["Main", "Extra", "Image"];
            if common_tables.contains(&symbol.name.as_str()) {
                return true;
            }

            // Check for actual tag definition structure in the data
            // Real tag tables have entries with Name, Format, PrintConv, etc.
            for (_key, value) in data {
                if let Some(entry) = value.as_object() {
                    // Look for tag definition fields
                    if entry.contains_key("Name")
                        || entry.contains_key("Format")
                        || entry.contains_key("Writable")
                        || entry.contains_key("PrintConv")
                        || entry.contains_key("ValueConv")
                    {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Generate Rust code from FieldSymbol tag table data  
    fn generate_tag_table_code(&self, symbol: &ProcessedTagTable) -> Result<String> {
        let table_data = symbol
            .symbol_data
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Expected tag table to be an object"))?;

        if table_data.is_empty() {
            return Ok(String::new());
        }

        let mut code = String::new();
        code.push_str("//! Generated tag table definitions\n");
        code.push_str("//!\n");
        code.push_str(&format!(
            "//! Extracted from {}::{} via field_extractor.pl\n",
            symbol.module_name, symbol.table_name
        ));
        code.push_str("\n");
        code.push_str("use std::sync::LazyLock;\n");
        code.push_str("use std::collections::HashMap;\n");
        code.push_str("use crate::types::{TagInfo, PrintConv};\n");
        code.push_str("\n");

        // Generate constant name
        let module_snake_case = symbol
            .module_name
            .to_lowercase()
            .replace("exiftool", "exif_tool");
        let constant_name = format!(
            "{}_{}_TAGS",
            module_snake_case.to_uppercase(),
            symbol.table_name.to_uppercase()
        );

        code.push_str(&format!(
            "/// Tag definitions for {}::{} table\n",
            symbol.module_name, symbol.table_name
        ));
        code.push_str(&format!(
            "pub static {}: LazyLock<HashMap<u16, TagInfo>> = LazyLock::new(|| {{\n",
            constant_name
        ));
        code.push_str("    let mut tags = HashMap::new();\n");

        // Process each tag in the table
        for (tag_key, tag_data) in table_data {
            if let Some(tag_obj) = tag_data.as_object() {
                self.process_tag_entry(&mut code, tag_key, tag_obj, &symbol.module_name)?;
            }
        }

        code.push_str("    tags\n");
        code.push_str("});\n");

        Ok(code)
    }

    /// Process a single tag entry from the field_extractor data
    fn process_tag_entry(
        &self,
        code: &mut String,
        tag_key: &str,
        tag_data: &serde_json::Map<String, JsonValue>,
        module: &str,
    ) -> Result<()> {
        // Extract basic tag information
        let name = tag_data
            .get("Name")
            .and_then(|v| v.as_str())
            .unwrap_or(tag_key);
        let format = tag_data
            .get("Format")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        // Parse tag ID (could be string or number)
        let tag_id = if let Ok(id) = tag_key.parse::<u16>() {
            id
        } else if tag_key.starts_with("0x") || tag_key.starts_with("0X") {
            u16::from_str_radix(&tag_key[2..], 16).unwrap_or(0)
        } else {
            debug!("Skipping non-numeric tag key: {}", tag_key);
            return Ok(());
        };

        code.push_str(&format!("    tags.insert({}, TagInfo {{\n", tag_id));
        code.push_str(&format!("        name: \"{}\",\n", name));
        code.push_str(&format!("        format: \"{}\",\n", format));

        // Process PrintConv if present
        let print_conv = self.process_print_conv(tag_data, module, name)?;
        code.push_str(&format!("        print_conv: {},\n", print_conv));

        // Process ValueConv if present (when we add it to TagInfo)
        // let value_conv = self.process_value_conv(tag_data, module, name)?;
        // code.push_str(&format!("        value_conv: {},\n", value_conv));

        code.push_str("    });\n");

        Ok(())
    }

    /// Process PrintConv field using the existing conv_registry system
    fn process_print_conv(
        &self,
        tag_data: &serde_json::Map<String, JsonValue>,
        module: &str,
        tag_name: &str,
    ) -> Result<String> {
        // Check for PrintConv field
        if let Some(print_conv_value) = tag_data.get("PrintConv") {
            if let Some(print_conv_str) = print_conv_value.as_str() {
                // First try tag-specific lookup
                if let Some((module_path, func_name)) =
                    lookup_tag_specific_printconv(module, tag_name)
                {
                    return Ok(format!(
                        "Some(PrintConv::Manual(\"{}\", \"{}\"))",
                        module_path, func_name
                    ));
                }

                // Then try expression lookup
                if let Some((module_path, func_name)) = lookup_printconv(print_conv_str, module) {
                    return Ok(format!(
                        "Some(PrintConv::Manual(\"{}\", \"{}\"))",
                        module_path, func_name
                    ));
                }

                // For hash references (lookup tables), generate Simple variant
                if print_conv_str.starts_with("{") || print_conv_str.contains("=>") {
                    // This indicates a hash lookup - we'd need to parse it
                    // For now, leave as raw expression
                    return Ok(format!(
                        "Some(PrintConv::Expression(\"{}\".to_string()))",
                        escape_string(print_conv_str)
                    ));
                }

                // Default to Expression type for any other string
                return Ok(format!(
                    "Some(PrintConv::Expression(\"{}\".to_string()))",
                    escape_string(print_conv_str)
                ));
            } else if let Some(_print_conv_obj) = print_conv_value.as_object() {
                // This is likely a hash reference lookup table
                return Ok("Some(PrintConv::Complex)".to_string());
            }
        }

        // No PrintConv found
        Ok("None".to_string())
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

    fn extract(
        &mut self,
        symbol_data: &FieldSymbol,
        context: &mut ExtractionContext,
    ) -> Result<()> {
        context.log_strategy_selection(
            symbol_data,
            self.name(),
            "Detected tag table with PrintConv/ValueConv definitions",
        );

        // Store the symbol data directly for processing
        let processed_table = ProcessedTagTable {
            module_name: symbol_data.module.clone(),
            table_name: symbol_data.name.clone(),
            symbol_data: symbol_data.data.clone(),
        };

        debug!(
            "Stored TagTable symbol: {}::{}",
            processed_table.module_name, processed_table.table_name
        );
        self.processed_symbols.push(processed_table);

        Ok(())
    }

    fn finish_module(&mut self, _module_name: &str) -> Result<()> {
        // Nothing to do per-module
        Ok(())
    }

    fn finish_extraction(&mut self) -> Result<Vec<GeneratedFile>> {
        let mut files = Vec::new();

        if self.processed_symbols.is_empty() {
            return Ok(files);
        }

        info!(
            "Processing {} TagTable symbols with global batch normalization",
            self.processed_symbols.len()
        );

        // STEP 1: Collect all PrintConv expressions for global batch normalization
        let mut all_expressions = std::collections::HashSet::new();
        for symbol in &self.processed_symbols {
            if let Some(data) = symbol.symbol_data.as_object() {
                for (_tag_key, tag_data) in data {
                    if let Some(tag_obj) = tag_data.as_object() {
                        if let Some(print_conv_expr) =
                            tag_obj.get("PrintConv").and_then(|v| v.as_str())
                        {
                            all_expressions.insert(print_conv_expr.to_string());
                        }
                    }
                }
            }
        }

        // STEP 2: Single batch normalization call for ALL expressions
        let expressions_vec: Vec<String> = all_expressions.into_iter().collect();
        let _global_cache = if expressions_vec.is_empty() {
            std::collections::HashMap::new()
        } else {
            crate::conv_registry::normalization::batch_normalize_expressions(&expressions_vec)
                .unwrap_or_default()
        };

        for symbol in &self.processed_symbols {
            match self.generate_tag_table_code(symbol) {
                Ok(code) => {
                    if !code.trim().is_empty() {
                        let path = generate_module_path(
                            &symbol.module_name,
                            &format!("{}_tags", symbol.table_name.to_lowercase()),
                        );

                        files.push(GeneratedFile {
                            path,
                            content: code,
                            strategy: self.name().to_string(),
                        });

                        debug!(
                            "Generated TagTable code for {}::{}",
                            symbol.module_name, symbol.table_name
                        );
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to generate code for {}::{}: {}",
                        symbol.module_name, symbol.table_name, e
                    );
                }
            }
        }

        // Clear processed symbols
        self.processed_symbols.clear();

        info!("TagKitStrategy generated {} files", files.len());
        Ok(files)
    }
}
