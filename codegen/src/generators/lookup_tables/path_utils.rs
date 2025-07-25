//! Path construction utilities for lookup table generation

use std::path::{Path, PathBuf};

/// Base directory for extracted data
pub const EXTRACT_BASE_DIR: &str = "generated/extract";

/// Module name suffix
pub const MODULE_SUFFIX: &str = "_pm";

/// Get the extract subdirectory for a given config type
#[allow(dead_code)]
pub fn get_extract_subdir(config_type: &str) -> &'static str {
    match config_type {
        "tag_table_structure" => "tag_structures",
        "process_binary_data" => "binary_data", 
        "model_detection" => "model_detection",
        "conditional_tags" => "conditional_tags",
        "runtime_table" => "runtime_tables",
        "simple_table" => "simple_tables",
        "boolean_set" => "boolean_sets",
        "tag_kit" => "tag_kits",
        "inline_printconv" => "inline_printconv",
        "regex_patterns" => "file_types",
        _ => "unknown",
    }
}

/// Get the module base name (removes "_pm" suffix)
pub fn get_module_base(module_name: &str) -> &str {
    module_name.trim_end_matches(MODULE_SUFFIX)
}

/// Construct path to extracted data directory
pub fn get_extract_dir(subdir: &str) -> PathBuf {
    Path::new(EXTRACT_BASE_DIR).join(subdir)
}

/// Construct standardized extract filename
pub fn construct_extract_filename(module_name: &str, pattern: &str) -> String {
    format!("{}__{}",
        get_module_base(module_name).to_lowercase(),
        pattern
    )
}

/// Construct extract file path
#[allow(dead_code)]
pub fn construct_extract_path(module_name: &str, subdir: &str, pattern: &str) -> PathBuf {
    let extract_dir = get_extract_dir(subdir);
    let filename = construct_extract_filename(module_name, pattern);
    extract_dir.join(filename)
}

/// Convert hash name to a valid Rust filename
pub fn hash_name_to_filename(hash_name: &str) -> String {
    hash_name
        .trim_start_matches('%')
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

/// Convert table name to snake_case to match Perl transformation
/// Replicates: s/([A-Z])/_\L$1/g; s/^_//; lc($filename)
pub fn convert_table_name_to_snake_case(table_name: &str) -> String {
    let mut result = String::new();
    
    for ch in table_name.chars() {
        if ch.is_uppercase() {
            result.push('_');
            result.push(ch.to_ascii_lowercase());
        } else {
            result.push(ch);
        }
    }
    
    // Remove leading underscore if present
    if result.starts_with('_') {
        result.remove(0);
    }
    
    result
}