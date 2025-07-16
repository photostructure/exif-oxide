//! Boolean set generator for membership testing

use anyhow::Result;
use crate::schemas::ExtractedTable;
use crate::common::escape_string;

/// Generate code for a boolean set table (keys map to boolean values)
pub fn generate_boolean_set(
    hash_name: &str,
    table_data: &ExtractedTable,
) -> Result<String> {
    let metadata = &table_data.metadata;
    let source = &table_data.source;
    let mut code = String::new();
    
    let constant_name = &metadata.constant_name;
    
    // Table header comment (not file header)
    code.push_str(&format!(
        "// Generated {} boolean set\n// Source: ExifTool {} {}\n// Description: {}\n\n",
        constant_name.to_lowercase().replace('_', " "),
        source.module,
        hash_name,
        metadata.description
    ));
    
    // Verify key type
    let key_type = &metadata.key_type;
    
    if key_type != "String" {
        return Err(anyhow::anyhow!(
            "boolean_set extraction only supports String key_type, got: {}",
            key_type
        ));
    }
    
    // Sort entries and filter for truthy values
    let mut sorted_entries = table_data.entries.clone();
    sorted_entries.sort_by(|a, b| {
        a.key.as_ref().unwrap_or(&String::new()).cmp(b.key.as_ref().unwrap_or(&String::new()))
    });
    
    // Collect only keys with truthy values
    let truthy_keys: Vec<String> = sorted_entries
        .iter()
        .filter_map(|entry| {
            entry.key.as_ref().map(|k| k.clone())
        })
        .collect();
    
    // Generate static data array
    let data_name = format!("{}_DATA", constant_name);
    code.push_str(&format!(
        "/// Static data for {} set ({} entries)\n",
        metadata.description.to_lowercase(),
        truthy_keys.len()
    ));
    code.push_str(&format!(
        "static {}: &[&str] = &[\n",
        data_name
    ));
    
    // Add entries to data array
    for key in &truthy_keys {
        code.push_str(&format!("    \"{}\",\n", escape_string(key)));
    }
    
    code.push_str("];\n\n");
    
    // Generate lazy HashSet
    code.push_str(&format!(
        "/// {} boolean set table\n/// Built from static data on first access\n",
        metadata.description
    ));
    code.push_str(&format!(
        "pub static {}: LazyLock<HashSet<&'static str>> =\n    LazyLock::new(|| {}.iter().copied().collect());\n\n",
        constant_name,
        data_name
    ));
    
    // Generate check function
    let fn_name = constant_name.to_lowercase();
    code.push_str(&format!(
        "/// Check if a file type is in the {} set\npub fn is_{}(file_type: &str) -> bool {{\n",
        metadata.description.to_lowercase(),
        fn_name.strip_suffix("_types").unwrap_or(&fn_name)
    ));
    
    code.push_str(&format!("    {}.contains(file_type)\n", constant_name));
    code.push_str("}\n");
    
    Ok(code)
}