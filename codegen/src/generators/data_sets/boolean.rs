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
            "boolean_set extraction only supports String key_type, got: {key_type}"
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
            entry.key.clone()
        })
        .collect();
    
    // Generate static data array
    let data_name = format!("{constant_name}_DATA");
    let description_lower = metadata.description.to_lowercase();
    let num_entries = truthy_keys.len();
    code.push_str(&format!(
        "/// Static data for {description_lower} set ({num_entries} entries)\n"
    ));
    code.push_str(&format!(
        "static {data_name}: &[&str] = &[\n"
    ));
    
    // Add entries to data array
    for key in &truthy_keys {
        let escaped_key = escape_string(key);
        code.push_str(&format!("    \"{escaped_key}\",\n"));
    }
    
    code.push_str("];\n\n");
    
    // Generate lazy HashSet
    code.push_str(&format!(
        "/// {} boolean set table\n/// Built from static data on first access\n",
        metadata.description
    ));
    code.push_str(&format!(
        "pub static {constant_name}: LazyLock<HashSet<&'static str>> =\n    LazyLock::new(|| {data_name}.iter().copied().collect());\n\n"
    ));
    
    // Generate check function
    let fn_name = constant_name.to_lowercase();
    let description_lower = metadata.description.to_lowercase();
    let fn_suffix = fn_name.strip_suffix("_types").unwrap_or(&fn_name);
    code.push_str(&format!(
        "/// Check if a file type is in the {description_lower} set\npub fn is_{fn_suffix}(file_type: &str) -> bool {{\n"
    ));
    
    code.push_str(&format!("    {constant_name}.contains(file_type)\n"));
    code.push_str("}\n");
    
    Ok(code)
}