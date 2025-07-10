//! File type discriminated union generation with alias support

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct FileTypeLookupData {
    pub extracted_at: String,
    pub file_type_lookups: FileTypeLookups,
    pub stats: FileTypeStats,
}

#[derive(Debug, Deserialize)]
pub struct FileTypeLookups {
    pub extensions: Vec<FileTypeLookupEntry>,
    pub mime_types: Vec<FileTypeLookupEntry>,
    pub descriptions: Vec<FileTypeLookupEntry>,
    pub magic_lookups: Vec<FileTypeLookupEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FileTypeLookupEntry {
    pub key: String,
    pub value: serde_json::Value,
    pub lookup_type: String,
    pub source: FileTypeLookupSource,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FileTypeLookupSource {
    pub module: String,
    pub hash: String,
}

#[derive(Debug, Deserialize)]
pub struct FileTypeStats {
    pub total_lookups: usize,
    pub by_type: HashMap<String, usize>,
}

/// Generate file type lookup table from file_type_lookup.json
pub fn generate_file_type_lookup(json_dir: &Path, output_dir: &str) -> Result<()> {
    // Look for file_type_lookup.json
    let file_type_lookup_path = json_dir.join("file_type_lookup.json");
    
    if !file_type_lookup_path.exists() {
        println!("    ⚠️  file_type_lookup.json not found, skipping file type lookups");
        return Ok(());
    }
    
    let json_data = fs::read_to_string(&file_type_lookup_path)?;
    let data: FileTypeLookupData = serde_json::from_str(&json_data)?;
    
    // Generate file_type_lookup.rs directly in output_dir (not in subdirectory)
    generate_file_type_lookup_module(&data, Path::new(output_dir))?;
    
    println!("    ✓ Generated file type lookup tables with {} total lookups", data.stats.total_lookups);
    
    Ok(())
}

fn generate_file_type_lookup_module(data: &FileTypeLookupData, output_dir: &Path) -> Result<()> {
    let mut code = String::new();
    
    // File header
    code.push_str("//! File type lookup tables generated from ExifTool's fileTypeLookup hash\n");
    code.push_str("//!\n");
    code.push_str(&format!("//! Generated at: {}\n", data.extracted_at));
    code.push_str(&format!("//! Total lookups: {}\n", data.stats.total_lookups));
    code.push_str("\n");
    code.push_str("use std::collections::HashMap;\n");
    code.push_str("use once_cell::sync::Lazy;\n");
    code.push_str("\n");
    
    // Generate extension aliases (simple string mappings)
    code.push_str("/// Extension aliases - maps extensions to their canonical forms\n");
    code.push_str("static EXTENSION_ALIASES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {\n");
    code.push_str("    let mut map = HashMap::new();\n");
    
    for entry in &data.file_type_lookups.descriptions {
        if let Some(alias) = entry.value.as_str() {
            // Simple string alias (e.g., "3GP2" -> "3G2")
            code.push_str(&format!("    map.insert(\"{}\", \"{}\");\n", entry.key, alias));
        }
    }
    
    code.push_str("    map\n");
    code.push_str("});\n");
    code.push_str("\n");
    
    // Generate file type formats (complex mappings)
    code.push_str("/// File type formats - maps file types to their format descriptions\n");
    code.push_str("static FILE_TYPE_FORMATS: Lazy<HashMap<&'static str, (Vec<&'static str>, &'static str)>> = Lazy::new(|| {\n");
    code.push_str("    let mut map = HashMap::new();\n");
    
    // Process entries from both extensions and mime_types that have array values
    let all_entries = data.file_type_lookups.extensions.iter()
        .chain(data.file_type_lookups.mime_types.iter());
    
    for entry in all_entries {
        if let Some(arr) = entry.value.as_array() {
            if arr.len() >= 2 {
                let formats = &arr[0];
                let description = arr[1].as_str().unwrap_or("Unknown");
                
                if let Some(format_arr) = formats.as_array() {
                    // Multiple formats: ["PDF", "PS"]
                    let format_list: Vec<String> = format_arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| format!("\"{}\"", s))
                        .collect();
                    code.push_str(&format!("    map.insert(\"{}\", (vec![{}], \"{}\"));\n", 
                                         entry.key, format_list.join(", "), description));
                } else if let Some(format_str) = formats.as_str() {
                    // Single format: "MOV"
                    code.push_str(&format!("    map.insert(\"{}\", (vec![\"{}\"], \"{}\"));\n", 
                                         entry.key, format_str, description));
                }
            }
        }
    }
    
    code.push_str("    map\n");
    code.push_str("});\n");
    code.push_str("\n");
    
    // Generate public API functions
    code.push_str("/// Resolve file type from extension, following aliases\n");
    code.push_str("pub fn resolve_file_type(extension: &str) -> Option<(Vec<&'static str>, &'static str)> {\n");
    code.push_str("    // Convert to uppercase for case-insensitive lookup\n");
    code.push_str("    let ext_upper = extension.to_uppercase();\n");
    code.push_str("    \n");
    code.push_str("    // First check for direct format lookup\n");
    code.push_str("    if let Some((formats, desc)) = FILE_TYPE_FORMATS.get(ext_upper.as_str()) {\n");
    code.push_str("        return Some((formats.clone(), *desc));\n");
    code.push_str("    }\n");
    code.push_str("    \n");
    code.push_str("    // Check for alias resolution\n");
    code.push_str("    if let Some(alias) = EXTENSION_ALIASES.get(ext_upper.as_str()) {\n");
    code.push_str("        return resolve_file_type(alias);\n");
    code.push_str("    }\n");
    code.push_str("    \n");
    code.push_str("    None\n");
    code.push_str("}\n");
    code.push_str("\n");
    
    code.push_str("/// Get primary format for a file type\n");
    code.push_str("pub fn get_primary_format(file_type: &str) -> Option<String> {\n");
    code.push_str("    resolve_file_type(file_type)\n");
    code.push_str("        .map(|(formats, _)| formats[0].to_string())\n");
    code.push_str("}\n");
    code.push_str("\n");
    
    code.push_str("/// Check if a file type supports a specific format\n");
    code.push_str("pub fn supports_format(file_type: &str, format: &str) -> bool {\n");
    code.push_str("    resolve_file_type(file_type)\n");
    code.push_str("        .map(|(formats, _)| formats.contains(&format))\n");
    code.push_str("        .unwrap_or(false)\n");
    code.push_str("}\n");
    code.push_str("\n");
    
    code.push_str("/// Get all extensions that support a specific format\n");
    code.push_str("pub fn extensions_for_format(target_format: &str) -> Vec<String> {\n");
    code.push_str("    let mut extensions = Vec::new();\n");
    code.push_str("    \n");
    code.push_str("    for (ext, (formats, _)) in FILE_TYPE_FORMATS.iter() {\n");
    code.push_str("        if formats.contains(&target_format) {\n");
    code.push_str("            extensions.push(ext.to_string());\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    code.push_str("    \n");
    code.push_str("    extensions\n");
    code.push_str("}\n");
    
    // Write the file to file_types subdirectory
    let file_types_dir = output_dir.join("file_types");
    fs::create_dir_all(&file_types_dir)?;
    let output_path = file_types_dir.join("file_type_lookup.rs");
    fs::write(&output_path, code)?;
    
    Ok(())
}