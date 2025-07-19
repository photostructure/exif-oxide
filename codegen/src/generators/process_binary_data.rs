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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub print_conv: Option<Vec<PrintConvEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub print_conv_expr: Option<String>,
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
    pub value: String,
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
    code.push_str("use std::sync::LazyLock;\n\n");
    
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
    code.push_str(&format!("pub struct {} {{\n", struct_name));
    
    // Add table header fields as struct members
    if let Some(format) = &data.table_data.header.format {
        code.push_str(&format!("    pub default_format: &'static str, // \"{}\"\n", escape_string(format)));
    }
    if let Some(first_entry) = data.table_data.header.first_entry {
        code.push_str(&format!("    pub first_entry: i32, // {}\n", first_entry));
    }
    if let Some(groups) = &data.table_data.header.groups {
        if groups.len() >= 2 {
            code.push_str(&format!("    pub groups: (&'static str, &'static str), // (\"{}\", \"{}\")\n", 
                groups.get(0).unwrap_or(&"".to_string()),
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
    code.push_str(&format!("impl {} {{\n", struct_name));
    
    // New function
    code.push_str("    /// Create new table instance\n");
    code.push_str("    pub fn new() -> Self {\n");
    code.push_str("        Self {\n");
    if let Some(format) = &data.table_data.header.format {
        code.push_str(&format!("            default_format: \"{}\",\n", escape_string(format)));
    }
    if let Some(first_entry) = data.table_data.header.first_entry {
        code.push_str(&format!("            first_entry: {},\n", first_entry));
    }
    if let Some(groups) = &data.table_data.header.groups {
        if groups.len() >= 2 {
            code.push_str(&format!("            groups: (\"{}\", \"{}\"),\n", 
                groups.get(0).unwrap_or(&"".to_string()),
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
    
    code.push_str("}\n\n");
    
    // Generate Default implementation
    code.push_str(&format!("impl Default for {} {{\n", struct_name));
    code.push_str("    fn default() -> Self {\n");
    code.push_str("        Self::new()\n");
    code.push_str("    }\n");
    code.push_str("}\n");
    
    Ok(code)
}