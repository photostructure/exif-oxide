//! Tag table structure generator for manufacturer enums
//!
//! This generator creates Rust enums from ExifTool manufacturer Main tables,
//! providing type-safe tag identification and metadata access.
use crate::common::escape_string;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Deserialize, Serialize)]
pub struct TagStructureData {
    pub manufacturer: String,
    pub source: SourceInfo,
    pub metadata: TableMetadata,
    pub tags: Vec<TagDefinition>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SourceInfo {
    pub module: String,
    pub table: String,
    pub extracted_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TableMetadata {
    pub total_tags: usize,
    pub has_process_binary_data: bool,
    pub has_conditional_tags: bool,
    #[serde(default = "default_enum_name")]
    pub enum_name: String,
}

fn default_enum_name() -> String {
    "UnknownDataType".to_string()
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TagDefinition {
    pub tag_id: String,
    pub tag_id_decimal: u16,
    pub name: String,
    #[serde(default)]
    pub has_subdirectory: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subdirectory_table: Option<String>,
    #[serde(default)]
    pub process_binary_data: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<Vec<String>>,
    #[serde(default)]
    pub writable: bool,
    #[serde(default)]
    pub unknown: bool,
    #[serde(default)]
    pub binary: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub conditional: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variants: Option<Vec<TagVariant>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TagVariant {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    // Other variant-specific fields can be added here
}

/// Generate Rust code for a tag table structure
pub fn generate_tag_structure(data: &TagStructureData) -> Result<String> {
    let mut code = String::new();
    
    // Add header comment
    code.push_str(&format!(
        "//! {} tag table structure generated from {}\n",
        data.manufacturer, data.source.module
    ));
    code.push_str(&format!(
        "//! ExifTool: {} %{}::{}\n\n",
        data.source.module, data.manufacturer, data.source.table
    ));
    
    // Generate enum name - prefer config override, fallback to manufacturer-based name
    let enum_name = if !data.metadata.enum_name.is_empty() && data.metadata.enum_name != "UnknownDataType" {
        data.metadata.enum_name.clone()
    } else {
        format!("{}DataType", data.manufacturer)
    };
    
    // No imports needed for tag structure enums
    
    // Generate the enum
    code.push_str(&format!(
        "/// {} data types from %{}::{} table\n",
        data.manufacturer, data.manufacturer, data.source.table
    ));
    code.push_str(&format!(
        "/// Total tags: {} (conditional: {}, with subdirectories: {})\n",
        data.metadata.total_tags,
        data.tags.iter().filter(|t| t.conditional).count(),
        data.tags.iter().filter(|t| t.has_subdirectory).count()
    ));
    code.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n");
    code.push_str(&format!("pub enum {enum_name} {{\n"));
    
    // Generate enum variants and store mappings
    let mut seen_names = HashSet::new();
    let mut variant_mappings = Vec::new();
    
    for tag in &data.tags {
        // Create a valid Rust variant name from the tag name
        let variant_name = create_variant_name(&tag.name, &mut seen_names);
        
        // Store the mapping for use in method generation
        variant_mappings.push((tag, variant_name.clone()));
        
        // Add documentation comment
        code.push_str(&format!("    /// {}: {}\n", tag.tag_id, tag.name));
        if let Some(desc) = &tag.description {
            code.push_str(&format!("    /// {}\n", escape_string(desc)));
        }
        if tag.has_subdirectory {
            if let Some(table) = &tag.subdirectory_table {
                code.push_str(&format!("    /// ExifTool: SubDirectory -> {table}\n"));
            }
            if tag.process_binary_data {
                code.push_str("    /// ExifTool: ProcessBinaryData\n");
            }
        }
        
        code.push_str(&format!("    {variant_name},\n"));
        
        if tag.conditional {
            code.push('\n');
        }
    }
    
    code.push_str("}\n\n");
    
    // Generate impl block
    code.push_str(&format!("impl {enum_name} {{\n"));
    
    // Generate tag_id method
    code.push_str("    /// Get tag ID for this data type\n");
    code.push_str("    pub fn tag_id(&self) -> u16 {\n");
    code.push_str("        match self {\n");
    
    for (tag, variant_name) in &variant_mappings {
        code.push_str(&format!(
            "            {}::{} => {},\n",
            enum_name, variant_name, tag.tag_id
        ));
    }
    
    code.push_str("        }\n");
    code.push_str("    }\n\n");
    
    // Generate from_tag_id method
    code.push_str("    /// Get data type from tag ID\n");
    code.push_str(&format!(
        "    pub fn from_tag_id(tag_id: u16) -> Option<{enum_name}> {{\n"
    ));
    code.push_str("        match tag_id {\n");
    
    for (tag, variant_name) in &variant_mappings {
        code.push_str(&format!(
            "            {} => Some({}::{}),\n",
            tag.tag_id, enum_name, variant_name
        ));
    }
    
    code.push_str("            _ => None,\n");
    code.push_str("        }\n");
    code.push_str("    }\n\n");
    
    // Generate name method
    code.push_str("    /// Get the ExifTool tag name\n");
    code.push_str("    pub fn name(&self) -> &'static str {\n");
    code.push_str("        match self {\n");
    
    for (tag, variant_name) in &variant_mappings {
        code.push_str(&format!(
            "            {}::{} => \"{}\",\n",
            enum_name, variant_name, tag.name
        ));
    }
    
    code.push_str("        }\n");
    code.push_str("    }\n\n");
    
    // Generate has_subdirectory method
    code.push_str("    /// Check if this tag has a subdirectory\n");
    code.push_str("    pub fn has_subdirectory(&self) -> bool {\n");
    
    let subdirectory_tags: Vec<&(&TagDefinition, String)> = variant_mappings.iter()
        .filter(|(tag, _)| tag.has_subdirectory)
        .collect();
    
    if !subdirectory_tags.is_empty() {
        code.push_str("        matches!(self,\n");
        for (i, (_, variant_name)) in subdirectory_tags.iter().enumerate() {
            if i == 0 {
                code.push_str(&format!("            {enum_name}::{variant_name}"));
            } else {
                code.push_str(&format!(" |\n            {enum_name}::{variant_name}"));
            }
        }
        code.push_str("\n        )\n");
    } else {
        code.push_str("        false\n");
    }
    
    code.push_str("    }\n\n");
    
    // Generate groups method
    code.push_str("    /// Get the group hierarchy for this tag\n");
    code.push_str("    pub fn groups(&self) -> (&'static str, &'static str) {\n");
    code.push_str("        match self {\n");
    
    for (tag, variant_name) in &variant_mappings {
        let groups = if let Some(g) = &tag.groups {
            let group0 = g.first().map(|s| s.as_str()).unwrap_or("MakerNotes");
            let group2 = g.get(2).map(|s| s.as_str()).unwrap_or("Camera");
            (group0, group2)
        } else {
            ("MakerNotes", "Camera")
        };
        
        code.push_str(&format!(
            "            {}::{} => (\"{}\", \"{}\"),\n",
            enum_name, variant_name, groups.0, groups.1
        ));
    }
    
    code.push_str("        }\n");
    code.push_str("    }\n");
    
    code.push_str("}\n");
    
    // Generate subdirectory tag lookup function if this is not a Main table
    if data.source.table != "Main" {
        code.push('\n');
        generate_subdirectory_lookup_function(&mut code, data)?;
    }
    
    Ok(code)
}

/// Generate lookup function for subdirectory table tag name resolution
/// This generates functions like get_equipment_tag_name(tag_id: u16) -> Option<&'static str>
fn generate_subdirectory_lookup_function(code: &mut String, data: &TagStructureData) -> Result<()> {
    let table_name_lower = data.source.table.to_lowercase();
    let lookup_fn_name = format!("get_{table_name_lower}_tag_name");
    
    // Add function documentation
    code.push_str(&format!(
        "/// Get tag name for {} subdirectory\n",
        data.source.table
    ));
    code.push_str(&format!(
        "/// ExifTool: {} %{}::{} table\n",
        data.source.module, data.manufacturer, data.source.table
    ));
    code.push_str(&format!(
        "pub fn {lookup_fn_name}(tag_id: u16) -> Option<&'static str> {{\n"
    ));
    code.push_str("    match tag_id {\n");
    
    // Generate match arms for each tag
    for tag in &data.tags {
        code.push_str(&format!(
            "        {} => Some(\"{}\"),\n",
            format_tag_id_as_hex(tag.tag_id_decimal),
            escape_string(&tag.name)
        ));
    }
    
    code.push_str("        _ => None,\n");
    code.push_str("    }\n");
    code.push_str("}\n");
    
    Ok(())
}

/// Format tag ID as hexadecimal for match patterns
fn format_tag_id_as_hex(tag_id_decimal: u16) -> String {
    format!("0x{tag_id_decimal:04x}")
}

/// Create a valid Rust variant name from a tag name
fn create_variant_name(name: &str, seen_names: &mut HashSet<String>) -> String {
    // Remove common prefixes
    let name = name.strip_prefix("Canon").unwrap_or(name);
    let name = name.strip_prefix("Nikon").unwrap_or(name);
    let name = name.strip_prefix("Olympus").unwrap_or(name);
    
    // Convert to valid Rust identifier
    let mut variant_name = String::new();
    let mut chars = name.chars().peekable();
    
    // Ensure first character is uppercase
    if let Some(ch) = chars.next() {
        if ch.is_alphabetic() {
            variant_name.push(ch.to_uppercase().next().unwrap());
        } else if ch.is_numeric() {
            variant_name.push('_');
            variant_name.push(ch);
        }
    }
    
    // Process remaining characters
    while let Some(ch) = chars.next() {
        if ch.is_alphanumeric() {
            variant_name.push(ch);
        } else if ch == '-' || ch == '_' || ch == ' ' {
            // Convert next character to uppercase for CamelCase
            if let Some(next_ch) = chars.peek().copied() {
                if next_ch.is_alphanumeric() {
                    chars.next();
                    variant_name.extend(next_ch.to_uppercase());
                }
            }
        }
    }
    
    // Handle duplicates by appending a number
    let mut final_name = variant_name.clone();
    let mut counter = 2;
    while seen_names.contains(&final_name) {
        final_name = format!("{variant_name}{counter}");
        counter += 1;
    }
    
    seen_names.insert(final_name.clone());
    final_name
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_variant_name() {
        let mut seen = HashSet::new();
        
        assert_eq!(create_variant_name("CameraSettings", &mut seen), "CameraSettings");
        assert_eq!(create_variant_name("CanonImageType", &mut seen), "ImageType");
        assert_eq!(create_variant_name("AF-Info", &mut seen), "AFInfo");
        assert_eq!(create_variant_name("WB_Info", &mut seen), "WBInfo");
        assert_eq!(create_variant_name("0x1234Data", &mut seen), "_0x1234Data");
        
        // Test duplicate handling
        seen.clear();
        assert_eq!(create_variant_name("AFInfo", &mut seen), "AFInfo");
        assert_eq!(create_variant_name("AFInfo", &mut seen), "AFInfo2");
        assert_eq!(create_variant_name("AFInfo", &mut seen), "AFInfo3");
    }
}