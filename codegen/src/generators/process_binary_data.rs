//! ProcessBinaryData table generator for manufacturer binary parsing
//!
//! This generator creates Rust code from ExifTool ProcessBinaryData tables,
//! providing binary data parsing structures for manufacturer-specific formats.

use crate::common::escape_string;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ProcessBinaryDataExtraction {
    pub manufacturer: String,
    pub source: SourceInfo,
    pub table_data: TableData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SourceInfo {
    pub module: String,
    pub table: String,
    pub extracted_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TableData {
    pub table_name: String,
    pub header: TableHeader,
    pub tags: Vec<BinaryDataTag>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TableHeader {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_entry: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<Vec<String>>,
    #[serde(default)]
    pub writable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BinaryDataTag {
    pub offset: String,
    pub offset_decimal: u16,
    pub name: String,
    #[serde(default)]
    pub simple: bool,
    #[serde(default)]
    pub conditional: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variants: Option<Vec<TagVariant>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub print_conv: Option<Vec<PrintConvEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub print_conv_expr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_conv: Option<String>,
    #[serde(default)]
    pub writable: bool,
    #[serde(default)]
    pub unknown: bool,
    #[serde(default)]
    pub binary: bool,
    #[serde(default)]
    pub hidden: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TagVariant {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub print_conv: Option<Vec<PrintConvEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub print_conv_expr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_conv: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    #[serde(default)]
    pub writable: bool,
    #[serde(default)]
    pub unknown: bool,
    #[serde(default)]
    pub binary: bool,
    #[serde(default)]
    pub hidden: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PrintConvEntry {
    pub key: String,
    // TODO: Proper BITMASK support - see docs/todo/P15c-bitmask-printconv-implementation.md
    // For now, we handle both string values and BITMASK objects with a custom deserializer
    #[serde(deserialize_with = "deserialize_print_conv_value")]
    pub value: String,
}

// Custom deserializer to handle both string values and BITMASK objects
fn deserialize_print_conv_value<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    
    struct PrintConvValueVisitor;
    
    impl<'de> Visitor<'de> for PrintConvValueVisitor {
        type Value = String;
        
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or BITMASK object")
        }
        
        fn visit_str<E>(self, value: &str) -> Result<String, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }
        
        fn visit_map<A>(self, mut map: A) -> Result<String, A::Error>
        where
            A: de::MapAccess<'de>,
        {
            // TODO P15c: Extract and properly process BITMASK mapping data
            // For now, consume all map entries and return a placeholder
            while let Some((_key, _value)) = map.next_entry::<String, serde_json::Value>()? {
                // Consume all entries to avoid "trailing characters" error
            }
            Ok("TODO_BITMASK_P15c".to_string())
        }
    }
    
    deserializer.deserialize_any(PrintConvValueVisitor)
}

/// Generate Rust code for a ProcessBinaryData table
pub fn generate_process_binary_data(data: &ProcessBinaryDataExtraction) -> Result<String> {
    let mut code = String::new();
    
    // Add header comment
    code.push_str(&format!(
        "//! {} ProcessBinaryData table {} generated from {}\n",
        data.manufacturer, data.table_data.table_name, data.source.module
    ));
    code.push_str(&format!(
        "//! ExifTool: {} %{}::{}\n\n",
        data.source.module, data.manufacturer, data.table_data.table_name
    ));
    
    // Add imports
    code.push_str("use std::collections::HashMap;\n");
    code.push_str("use std::sync::LazyLock;\n");
    
    // Add conditional imports if needed
    let has_conditional_tags = data.table_data.tags.iter()
        .any(|tag| tag.conditional && tag.variants.is_some());
    
    if has_conditional_tags {
        code.push_str("use crate::types::{BinaryDataTag, BinaryDataTagVariant};\n");
    }
    
    code.push_str("\n");
    
    // Generate table structure
    let struct_name = format!("{}{}Table", data.manufacturer, data.table_data.table_name);
    
    code.push_str(&format!(
        "/// {} ProcessBinaryData table for {}\n",
        data.manufacturer, data.table_data.table_name
    ));
    code.push_str(&format!(
        "/// Total tags: {}\n",
        data.table_data.tags.len()
    ));
    code.push_str("#[derive(Debug, Clone)]\n");
    code.push_str(&format!("pub struct {struct_name} {{\n"));
    
    // Add table header fields as struct members
    if let Some(format) = &data.table_data.header.format {
        code.push_str(&format!("    pub default_format: &'static str, // \"{}\"\n", escape_string(format)));
    }
    if let Some(first_entry) = data.table_data.header.first_entry {
        code.push_str(&format!("    pub first_entry: i32, // {first_entry}\n"));
    }
    if let Some(groups) = &data.table_data.header.groups {
        if groups.len() >= 2 {
            code.push_str(&format!("    pub groups: (&'static str, &'static str), // (\"{}\", \"{}\")\n", 
                groups.first().unwrap_or(&"".to_string()),
                groups.get(1).unwrap_or(&"".to_string())
            ));
        }
    }
    
    code.push_str("}\n\n");
    
    // Generate tag offset mapping
    code.push_str(&format!(
        "/// Tag definitions for {} {}\n",
        data.manufacturer, data.table_data.table_name
    ));
    code.push_str(&format!(
        "pub static {}_TAGS: LazyLock<HashMap<u16, &'static str>> = LazyLock::new(|| {{\n",
        data.table_data.table_name.to_uppercase()
    ));
    code.push_str("    let mut map = HashMap::new();\n");
    
    // Add all tag mappings
    for tag in &data.table_data.tags {
        code.push_str(&format!(
            "    map.insert({}, \"{}\"); // {}: {}\n",
            tag.offset_decimal, tag.name, tag.offset, tag.name
        ));
    }
    
    code.push_str("    map\n");
    code.push_str("});\n\n");
    
    // Generate conditional variants for tags that have them
    let conditional_tags: Vec<&BinaryDataTag> = data.table_data.tags.iter()
        .filter(|tag| tag.conditional && tag.variants.is_some())
        .collect();
    
    if !conditional_tags.is_empty() {
        
        code.push_str(&format!(
            "/// Conditional variants for {} {}\n",
            data.manufacturer, data.table_data.table_name
        ));
        code.push_str(&format!(
            "pub static {}_VARIANTS: LazyLock<HashMap<u16, BinaryDataTag>> = LazyLock::new(|| {{\n",
            data.table_data.table_name.to_uppercase()
        ));
        code.push_str("    let mut map = HashMap::new();\n");
        
        for tag in &conditional_tags {
            if let Some(variants) = &tag.variants {
                code.push_str(&format!("    // Tag {}: {} (conditional)\n", tag.offset_decimal, tag.name));
                code.push_str(&format!("    map.insert({}, BinaryDataTag {{\n", tag.offset_decimal));
                code.push_str(&format!("        name: \"{}\".to_string(),\n", tag.name));
                code.push_str("        variants: vec![\n");
                
                for variant in variants {
                    code.push_str("            BinaryDataTagVariant {\n");
                    code.push_str(&format!("                name: \"{}\".to_string(),\n", variant.name));
                    
                    if let Some(condition) = &variant.condition {
                        code.push_str(&format!("                condition: Some(\"{}\".to_string()),\n", escape_string(condition)));
                    } else {
                        code.push_str("                condition: None,\n");
                    }
                    
                    code.push_str("                format_spec: None,\n");
                    code.push_str("                format: None,\n");
                    code.push_str("                mask: None,\n");
                    code.push_str("                print_conv: None,\n");
                    
                    if let Some(value_conv) = &variant.value_conv {
                        code.push_str(&format!("                value_conv: Some(\"{}\".to_string()),\n", escape_string(value_conv)));
                    } else {
                        code.push_str("                value_conv: None,\n");
                    }
                    
                    if let Some(print_conv_expr) = &variant.print_conv_expr {
                        code.push_str(&format!("                print_conv_expr: Some(\"{}\".to_string()),\n", escape_string(print_conv_expr)));
                    } else {
                        code.push_str("                print_conv_expr: None,\n");
                    }
                    
                    code.push_str("                data_member: None,\n");
                    code.push_str("                group: None,\n");
                    
                    if let Some(priority) = variant.priority {
                        code.push_str(&format!("                priority: Some({}),\n", priority));
                    } else {
                        code.push_str("                priority: None,\n");
                    }
                    
                    code.push_str("            },\n");
                }
                
                code.push_str("        ],\n");
                code.push_str("        format_spec: None,\n");
                code.push_str("        format: None,\n");
                code.push_str("        mask: None,\n");
                code.push_str("        print_conv: None,\n");
                code.push_str("        data_member: None,\n");
                code.push_str("        group: None,\n");
                code.push_str("    });\n\n");
            }
        }
        
        code.push_str("    map\n");
        code.push_str("});\n\n");
    }
    
    // Generate format mapping for complex tags
    let complex_tags: Vec<&BinaryDataTag> = data.table_data.tags.iter()
        .filter(|tag| tag.format.is_some())
        .collect();
    
    if !complex_tags.is_empty() {
        code.push_str(&format!(
            "/// Format specifications for {} {}\n",
            data.manufacturer, data.table_data.table_name
        ));
        code.push_str(&format!(
            "pub static {}_FORMATS: LazyLock<HashMap<u16, &'static str>> = LazyLock::new(|| {{\n",
            data.table_data.table_name.to_uppercase()
        ));
        code.push_str("    let mut map = HashMap::new();\n");
        
        for tag in &complex_tags {
            if let Some(format) = &tag.format {
                code.push_str(&format!(
                    "    map.insert({}, \"{}\"); // {}: {}\n",
                    tag.offset_decimal, escape_string(format), tag.offset, tag.name
                ));
            }
        }
        
        code.push_str("    map\n");
        code.push_str("});\n\n");
    }
    
    // Generate lookup functions
    code.push_str(&format!("impl {struct_name} {{\n"));
    
    // New function
    code.push_str("    /// Create new table instance\n");
    code.push_str("    pub fn new() -> Self {\n");
    code.push_str("        Self {\n");
    if let Some(format) = &data.table_data.header.format {
        code.push_str(&format!("            default_format: \"{}\",\n", escape_string(format)));
    }
    if let Some(first_entry) = data.table_data.header.first_entry {
        code.push_str(&format!("            first_entry: {first_entry},\n"));
    }
    if let Some(groups) = &data.table_data.header.groups {
        if groups.len() >= 2 {
            code.push_str(&format!("            groups: (\"{}\", \"{}\"),\n", 
                groups.first().unwrap_or(&"".to_string()),
                groups.get(1).unwrap_or(&"".to_string())
            ));
        }
    }
    code.push_str("        }\n");
    code.push_str("    }\n\n");
    
    // Tag name lookup
    code.push_str("    /// Get tag name for offset\n");
    code.push_str("    pub fn get_tag_name(&self, offset: u16) -> Option<&'static str> {\n");
    code.push_str(&format!("        {}_TAGS.get(&offset).copied()\n", data.table_data.table_name.to_uppercase()));
    code.push_str("    }\n\n");
    
    // Format lookup
    if !complex_tags.is_empty() {
        code.push_str("    /// Get format specification for offset\n");
        code.push_str("    pub fn get_format(&self, offset: u16) -> Option<&'static str> {\n");
        code.push_str(&format!("        {}_FORMATS.get(&offset).copied()\n", data.table_data.table_name.to_uppercase()));
        code.push_str("    }\n\n");
    }
    
    // Get all offsets
    code.push_str("    /// Get all valid offsets for this table\n");
    code.push_str("    pub fn get_offsets(&self) -> Vec<u16> {\n");
    code.push_str(&format!("        {}_TAGS.keys().copied().collect()\n", data.table_data.table_name.to_uppercase()));
    code.push_str("    }\n");
    
    // Add conditional variant lookup if needed
    if !conditional_tags.is_empty() {
        code.push_str("\n    /// Get conditional variants for a tag if available\n");
        code.push_str("    pub fn get_conditional_tag(&self, offset: u16) -> Option<&BinaryDataTag> {\n");
        code.push_str(&format!("        {}_VARIANTS.get(&offset)\n", data.table_data.table_name.to_uppercase()));
        code.push_str("    }\n");
        
        code.push_str("\n    /// Check if a tag has conditional variants\n");
        code.push_str("    pub fn is_conditional(&self, offset: u16) -> bool {\n");
        code.push_str(&format!("        {}_VARIANTS.contains_key(&offset)\n", data.table_data.table_name.to_uppercase()));
        code.push_str("    }\n");
    }
    
    code.push_str("}\n\n");
    
    // Generate Default implementation
    code.push_str(&format!("impl Default for {struct_name} {{\n"));
    code.push_str("    fn default() -> Self {\n");
    code.push_str("        Self::new()\n");
    code.push_str("    }\n");
    code.push_str("}\n");
    
    Ok(code)
}