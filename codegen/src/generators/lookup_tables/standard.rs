//! Standard lookup table generator for simple key-value mappings

use crate::common::escape_string;
use crate::schemas::ExtractedTable;
use anyhow::Result;

/// Generate code for a simple lookup table using static array + lazy HashMap pattern
pub fn generate_lookup_table(_hash_name: &str, table_data: &ExtractedTable) -> Result<String> {
    let metadata = &table_data.metadata;
    let _source = &table_data.source;
    let mut code = String::new();

    let constant_name = &metadata.constant_name;

    // Determine key type
    let key_type = &metadata.key_type;

    let key_rust_type = match key_type.as_str() {
        "u8" => "u8",
        "u16" => "u16",
        "u32" => "u32",
        "i8" => "i8",
        "i16" => "i16",
        "i32" => "i32",
        "f32" => "f32",
        "String" => "&'static str",
        _ => "&'static str",
    };

    // Determine value type
    // For now, all values are strings since metadata doesn't include value_type
    let value_rust_type = "&'static str";

    // Sort entries for deterministic output
    let mut sorted_entries = table_data.entries.clone();
    sorted_entries.sort_by(|a, b| {
        if key_type == "String" {
            a.key
                .as_ref()
                .unwrap_or(&String::new())
                .cmp(b.key.as_ref().unwrap_or(&String::new()))
        } else {
            let a_num: i64 = a
                .key
                .as_ref()
                .unwrap_or(&String::new())
                .parse()
                .unwrap_or(0);
            let b_num: i64 = b
                .key
                .as_ref()
                .unwrap_or(&String::new())
                .parse()
                .unwrap_or(0);
            a_num.cmp(&b_num)
        }
    });

    // Generate static data array
    let data_name = format!("{constant_name}_DATA");
    code.push_str(&format!(
        "/// Raw data ({} entries)\n",
        metadata.entry_count
    ));
    code.push_str(&format!(
        "static {data_name}: &[({key_rust_type}, {value_rust_type})] = &[\n"
    ));

    // Add entries to data array
    for entry in &sorted_entries {
        if let (Some(key), Some(value)) = (&entry.key, &entry.value) {
            let key_value = if key_type == "String" {
                // Handle single quote specially for Rust string literals
                if key == "'" {
                    "\"'\"".to_string()
                } else {
                    format!("\"{}\"", escape_string(key))
                }
            } else {
                key.clone()
            };

            // Handle value - convert from JSON Value to string
            let value_str = {
                let value_string = match value {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    _ => value.to_string(),
                };
                // String value - escape and quote
                format!("\"{}\"", escape_string(&value_string))
            };

            code.push_str(&format!("    ({key_value}, {value_str}),\n"));
        }
    }

    code.push_str("];\n\n");

    // Generate lazy HashMap
    code.push_str("/// Lookup table (lazy-initialized)\n");
    code.push_str(&format!(
        "pub static {constant_name}: LazyLock<HashMap<{key_rust_type}, {value_rust_type}>> = LazyLock::new(|| {{\n"
    ));

    let collect_expr = if key_type == "String" {
        format!("    {data_name}.iter().copied().collect()")
    } else {
        format!("    {data_name}.iter().cloned().collect()")
    };

    code.push_str(&format!("{collect_expr}\n"));
    code.push_str("});\n\n");

    // Generate lookup function
    let fn_name = constant_name.to_lowercase();
    let fn_param_type = if key_type == "String" {
        "&str"
    } else {
        key_rust_type
    };

    code.push_str(&format!(
        "/// Look up value by key\npub fn lookup_{fn_name}(key: {fn_param_type}) -> Option<{value_rust_type}> {{\n"
    ));

    let key_ref = if key_type == "String" { "key" } else { "&key" };
    let return_expr = format!("{constant_name}.get({key_ref}).copied()");
    code.push_str(&format!("    {return_expr}\n"));
    code.push_str("}\n");

    Ok(code)
}
