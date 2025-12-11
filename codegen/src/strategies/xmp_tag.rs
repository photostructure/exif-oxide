//! XmpTagStrategy - Processes ExifTool XMP namespace tag definitions
//!
//! This strategy recognizes and processes XMP namespace tables (hash symbols
//! that contain a NAMESPACE key) and generates XmpTagInfo lookup tables.

use anyhow::Result;
use serde_json::Value as JsonValue;
use tracing::{debug, info};

use super::{ExtractionContext, ExtractionStrategy, GeneratedFile};
use crate::field_extractor::FieldSymbol;
use crate::strategies::output_locations::{generate_module_path, to_snake_case};

/// Strategy for processing XMP namespace tag definitions
pub struct XmpTagStrategy {
    /// Accumulated XMP namespace tables per module
    processed_tables: Vec<ProcessedXmpTable>,
}

#[derive(Debug, Clone)]
struct ProcessedXmpTable {
    module_name: String,
    table_name: String,
    namespace: String,
    tags: Vec<XmpTagEntry>,
}

#[derive(Debug, Clone)]
struct XmpTagEntry {
    property_name: String,
    display_name: String,
    writable: Option<String>,
    list_type: Option<String>,
    resource: bool,
    print_conv: Option<PrintConvData>,
}

#[derive(Debug, Clone)]
struct PrintConvData {
    /// HashMap entries: (key, value) pairs
    entries: Vec<(String, String)>,
}

impl Default for XmpTagStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl XmpTagStrategy {
    pub fn new() -> Self {
        Self {
            processed_tables: Vec::new(),
        }
    }

    /// Check if this is an XMP namespace table (has NAMESPACE key with string value)
    fn is_xmp_namespace_table(symbol: &FieldSymbol) -> bool {
        if let Some(data) = symbol.data.as_object() {
            // XMP tables have a NAMESPACE key with a string value
            data.get("NAMESPACE").and_then(|v| v.as_str()).is_some()
        } else {
            false
        }
    }

    /// Keys that are metadata, not tag definitions
    const METADATA_KEYS: &'static [&'static str] = &[
        "NAMESPACE",
        "GROUPS",
        "NOTES",
        "TABLE_DESC",
        "WRITABLE",
        "WRITE_PROC",
        "CHECK_PROC",
        "LANG_INFO",
        "WRITE_GROUP",
        "STRUCT_NAME",
        "PREFERRED",
        "AVOID",
    ];

    /// Extract tag entries from the symbol data
    fn extract_tags(
        &self,
        data: &serde_json::Map<String, JsonValue>,
        default_writable: Option<&str>,
    ) -> Vec<XmpTagEntry> {
        let mut tags = Vec::new();

        for (key, value) in data {
            // Skip metadata keys
            if Self::METADATA_KEYS.contains(&key.as_str()) {
                continue;
            }

            // Skip function references and non-tag entries
            if let Some(s) = value.as_str() {
                if s.starts_with("[Function:") || s.starts_with("[TableRef:") {
                    continue;
                }
            }

            // Process tag definition
            if let Some(tag_data) = value.as_object() {
                let entry = self.build_tag_entry(key, tag_data, default_writable);
                tags.push(entry);
            } else if value.is_object() || value.as_object().map(|o| o.is_empty()).unwrap_or(false)
            {
                // Empty object {} means simple tag with defaults
                tags.push(XmpTagEntry {
                    property_name: key.clone(),
                    display_name: self.property_to_display_name(key),
                    writable: default_writable.map(String::from),
                    list_type: None,
                    resource: false,
                    print_conv: None,
                });
            }
        }

        // Sort by property name for deterministic output
        tags.sort_by(|a, b| a.property_name.cmp(&b.property_name));
        tags
    }

    /// Build a single tag entry from its definition
    fn build_tag_entry(
        &self,
        property_name: &str,
        tag_data: &serde_json::Map<String, JsonValue>,
        default_writable: Option<&str>,
    ) -> XmpTagEntry {
        // Get display name (Name field or derive from property name)
        let display_name = tag_data
            .get("Name")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| self.property_to_display_name(property_name));

        // Get writable type
        let writable = tag_data
            .get("Writable")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| default_writable.map(String::from));

        // Get list type (Bag, Seq, Alt)
        let list_type = tag_data
            .get("List")
            .and_then(|v| v.as_str())
            .map(String::from);

        // Check for Resource flag
        let resource = tag_data
            .get("Resource")
            .and_then(|v| v.as_i64())
            .map(|v| v == 1)
            .unwrap_or(false);

        // Extract PrintConv if it's a simple hash map
        let print_conv = self.extract_print_conv(tag_data);

        XmpTagEntry {
            property_name: property_name.to_string(),
            display_name,
            writable,
            list_type,
            resource,
            print_conv,
        }
    }

    /// Extract PrintConv if it's a simple hash map (not an expression)
    fn extract_print_conv(
        &self,
        tag_data: &serde_json::Map<String, JsonValue>,
    ) -> Option<PrintConvData> {
        let print_conv = tag_data.get("PrintConv")?;

        // Only handle hash map PrintConv, skip expressions
        let pc_obj = print_conv.as_object()?;

        // Check if this looks like a hash map (keys are strings, values are strings)
        let mut entries = Vec::new();
        for (k, v) in pc_obj {
            // Skip special keys like OTHER
            if k == "OTHER" {
                continue;
            }
            if let Some(display) = v.as_str() {
                entries.push((k.clone(), display.to_string()));
            }
        }

        if entries.is_empty() {
            return None;
        }

        // Sort for deterministic output
        entries.sort_by(|a, b| a.0.cmp(&b.0));

        Some(PrintConvData { entries })
    }

    /// Convert property name to display name (capitalize first letter)
    fn property_to_display_name(&self, property_name: &str) -> String {
        let mut chars = property_name.chars();
        match chars.next() {
            Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            None => String::new(),
        }
    }

    /// Generate Rust code for an XMP namespace table
    fn generate_table_code(&self, table: &ProcessedXmpTable) -> String {
        let mut code = String::new();

        // Check what imports we actually need
        let uses_list_type = table.tags.iter().any(|t| t.list_type.is_some());
        let uses_print_conv = table.tags.iter().any(|t| t.print_conv.is_some());

        // Header
        code.push_str(&format!(
            "//! Generated XMP tag table for {} namespace ({}::{})\n",
            table.namespace, table.module_name, table.table_name
        ));
        code.push_str("//!\n");
        code.push_str(
            "//! This file is auto-generated by codegen/src/strategies/xmp_tag.rs. Do not edit manually.\n",
        );
        code.push('\n');

        // Imports - only what's actually used
        code.push_str("use crate::core::XmpTagInfo;\n");
        if uses_list_type {
            code.push_str("use crate::core::XmpListType;\n");
        }
        if uses_print_conv {
            code.push_str("use crate::types::PrintConv;\n");
        }
        code.push_str("use std::collections::HashMap;\n");
        code.push_str("use std::sync::LazyLock;\n");
        code.push('\n');

        // Note: PrintConv is inlined in XmpTagInfo, not as separate constants
        // This is required because HashMap::from() with .to_string() can't be in statics

        // Generate constant name using namespace (cleaner than module_table)
        let namespace_snake = to_snake_case(&table.namespace);
        let constant_name = format!("XMP_{}_TAGS", namespace_snake.to_uppercase());

        // Generate HashMap
        code.push_str(&format!(
            "/// XMP tag definitions for {} namespace\n",
            table.namespace
        ));

        if table.tags.is_empty() {
            code.push_str(&format!(
                "pub static {constant_name}: LazyLock<HashMap<&'static str, XmpTagInfo>> = LazyLock::new(HashMap::new);\n"
            ));
        } else {
            code.push_str(&format!(
                "pub static {constant_name}: LazyLock<HashMap<&'static str, XmpTagInfo>> = LazyLock::new(|| {{\n"
            ));
            code.push_str("    HashMap::from([\n");

            for tag in &table.tags {
                let list_type = match tag.list_type.as_deref() {
                    Some("Bag") => "Some(XmpListType::Bag)",
                    Some("Seq") => "Some(XmpListType::Seq)",
                    Some("Alt") => "Some(XmpListType::Alt)",
                    _ => "None",
                };

                let writable = match &tag.writable {
                    Some(w) => format!("Some(\"{w}\")"),
                    None => "None".to_string(),
                };

                // Generate inlined PrintConv if present
                let print_conv = if let Some(pc) = &tag.print_conv {
                    let mut pc_code =
                        String::from("Some(PrintConv::Simple(std::collections::HashMap::from([\n");
                    for (k, v) in &pc.entries {
                        let k_escaped = k.replace('\\', "\\\\").replace('"', "\\\"");
                        let v_escaped = v.replace('\\', "\\\\").replace('"', "\\\"");
                        pc_code.push_str(&format!(
                            "                (\"{k_escaped}\".to_string(), \"{v_escaped}\"),\n"
                        ));
                    }
                    pc_code.push_str("            ])))");
                    pc_code
                } else {
                    "None".to_string()
                };

                let property_name = &tag.property_name;
                let display_name = &tag.display_name;
                let resource = tag.resource;
                code.push_str(&format!("        (\"{property_name}\", XmpTagInfo {{\n"));
                code.push_str(&format!("            name: \"{display_name}\",\n"));
                code.push_str(&format!("            writable: {writable},\n"));
                code.push_str(&format!("            list: {list_type},\n"));
                code.push_str(&format!("            resource: {resource},\n"));
                code.push_str(&format!("            print_conv: {print_conv},\n"));
                code.push_str("        }),\n");
            }

            code.push_str("    ])\n");
            code.push_str("});\n");
        }

        code
    }
}

impl ExtractionStrategy for XmpTagStrategy {
    fn name(&self) -> &'static str {
        "XmpTagStrategy"
    }

    fn can_handle(&self, symbol: &FieldSymbol) -> bool {
        Self::is_xmp_namespace_table(symbol)
    }

    fn extract(&mut self, symbol: &FieldSymbol, context: &mut ExtractionContext) -> Result<()> {
        let data = symbol
            .data
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Expected XMP table to be an object"))?;

        let namespace = data
            .get("NAMESPACE")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing NAMESPACE key"))?;

        let default_writable = data.get("WRITABLE").and_then(|v| v.as_str());

        let tags = self.extract_tags(data, default_writable);

        info!(
            "📦 XmpTagStrategy: Extracted {} tags from {}::{} (namespace: {})",
            tags.len(),
            symbol.module,
            symbol.name,
            namespace
        );

        // Log strategy selection
        context.strategy_log.push(super::StrategySelection {
            symbol_name: symbol.name.clone(),
            module_name: symbol.module.clone(),
            strategy_name: self.name().to_string(),
            reasoning: format!(
                "XMP namespace table detected (NAMESPACE={}), {} tags",
                namespace,
                tags.len()
            ),
        });

        self.processed_tables.push(ProcessedXmpTable {
            module_name: symbol.module.clone(),
            table_name: symbol.name.clone(),
            namespace: namespace.to_string(),
            tags,
        });

        Ok(())
    }

    fn finish_module(&mut self, _module_name: &str) -> Result<()> {
        // Nothing to do per-module
        Ok(())
    }

    fn finish_extraction(&mut self, context: &mut ExtractionContext) -> Result<Vec<GeneratedFile>> {
        let mut files = Vec::new();

        for table in &self.processed_tables {
            let code = self.generate_table_code(table);

            // generate_module_path returns full path including filename
            let path =
                generate_module_path(&table.module_name, &format!("{}_tags", table.table_name));

            debug!(
                "📝 XmpTagStrategy: Generated {} ({} bytes)",
                path,
                code.len()
            );

            files.push(GeneratedFile {
                path: path.clone(),
                content: code,
            });

            // Write directly to disk as well
            super::write_generated_file(
                &context.output_dir,
                &path,
                &files.last().unwrap().content,
            )?;
        }

        info!(
            "✅ XmpTagStrategy: Generated {} XMP namespace tag tables",
            files.len()
        );

        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_is_xmp_namespace_table() {
        let xmp_symbol = FieldSymbol {
            name: "dc".to_string(),
            module: "XMP".to_string(),
            symbol_type: "hash".to_string(),
            data: json!({
                "NAMESPACE": "dc",
                "GROUPS": {"1": "XMP-dc"},
                "creator": {"List": "Seq"}
            }),
            metadata: Default::default(),
        };

        assert!(XmpTagStrategy::is_xmp_namespace_table(&xmp_symbol));

        let non_xmp = FieldSymbol {
            name: "Main".to_string(),
            module: "Canon".to_string(),
            symbol_type: "hash".to_string(),
            data: json!({
                "GROUPS": {"0": "Canon"},
                "0x0001": {"Name": "CanonCameraSettings"}
            }),
            metadata: Default::default(),
        };

        assert!(!XmpTagStrategy::is_xmp_namespace_table(&non_xmp));
    }

    #[test]
    fn test_property_to_display_name() {
        let strategy = XmpTagStrategy::new();
        assert_eq!(strategy.property_to_display_name("creator"), "Creator");
        assert_eq!(
            strategy.property_to_display_name("attributionName"),
            "AttributionName"
        );
    }

    #[test]
    fn test_extract_tags() {
        let strategy = XmpTagStrategy::new();
        let data: serde_json::Map<String, JsonValue> = serde_json::from_value(json!({
            "NAMESPACE": "dc",
            "GROUPS": {"1": "XMP-dc"},
            "WRITABLE": "string",
            "creator": {"List": "Seq", "Groups": {"2": "Author"}},
            "description": {"Writable": "lang-alt"},
            "title": {}
        }))
        .unwrap();

        let tags = strategy.extract_tags(&data, Some("string"));

        assert_eq!(tags.len(), 3);

        let creator = tags.iter().find(|t| t.property_name == "creator").unwrap();
        assert_eq!(creator.display_name, "Creator");
        assert_eq!(creator.list_type, Some("Seq".to_string()));
        assert_eq!(creator.writable, Some("string".to_string()));

        let description = tags
            .iter()
            .find(|t| t.property_name == "description")
            .unwrap();
        assert_eq!(description.writable, Some("lang-alt".to_string()));
    }
}
