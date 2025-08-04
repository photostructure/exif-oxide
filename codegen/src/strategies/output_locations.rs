//! Standardized output location generation for strategy system
//!
//! This module provides consistent path generation for all extraction strategies,
//! ensuring snake_case naming convention and predictable directory structure.

use std::path::Path;

/// Generate standardized module path following Rust naming conventions
/// 
/// Converts ExifTool module names to lowercase directory names (e.g., "Canon" → "canon_pm", 
/// "GPS" → "gps_pm") and symbol names to snake_case filenames for consistent Rust organization.
///
/// # Arguments
/// * `module_name` - ExifTool module name (e.g., "Canon", "GPS", "DNG")
/// * `symbol_name` - Symbol name from the module (e.g., "canonWhiteBalance")
/// 
/// # Returns
/// Relative path from output directory: `[module_name]_pm/[symbol_name].rs`
///
/// # Examples
/// ```
/// # use crate::strategies::output_locations::generate_module_path;
/// assert_eq!(generate_module_path("Canon", "canonWhiteBalance"), "canon_pm/canon_white_balance.rs");
/// assert_eq!(generate_module_path("GPS", "coordConv"), "gps_pm/coord_conv.rs");
/// ```
pub fn generate_module_path(module_name: &str, symbol_name: &str) -> String {
    let module_dir = format!("{}_pm", module_name.to_lowercase());
    let filename = format!("{}.rs", to_snake_case(symbol_name));
    format!("{}/{}", module_dir, filename)
}

/// Generate path for specialized pattern types (arrays, binary data, etc.)
///
/// Creates subdirectories for different pattern types to organize complex
/// generated code structures.
///
/// # Arguments
/// * `module_name` - ExifTool module name (e.g., "Canon")
/// * `pattern_type` - Type of pattern (e.g., "arrays", "binary_data")
/// * `symbol_name` - Symbol name from the module
///
/// # Returns
/// Relative path: `[module_name]_pm/[pattern_type]/[symbol_name].rs`
///
/// # Examples
/// ```
/// # use crate::strategies::output_locations::generate_pattern_path;
/// assert_eq!(generate_pattern_path("Nikon", "arrays", "xlat_0"), "nikon_pm/arrays/xlat_0.rs");
/// ```
pub fn generate_pattern_path(module_name: &str, pattern_type: &str, symbol_name: &str) -> String {
    let module_dir = format!("{}_pm", module_name.to_lowercase());
    let filename = format!("{}.rs", to_snake_case(symbol_name));
    format!("{}/{}/{}", module_dir, pattern_type, filename)
}

/// Convert string to snake_case following Rust naming conventions
///
/// Handles common ExifTool naming patterns:
/// - PascalCase: "Canon" → "canon"
/// - camelCase: "canonWhiteBalance" → "canon_white_balance"  
/// - Acronyms: "GPS" → "gps", "DNG" → "dng"
/// - Mixed: "AFInfo2" → "a_f_info2"
///
/// # Arguments
/// * `name` - Input string in any case format
///
/// # Returns
/// String converted to snake_case
///
/// # Examples
/// ```
/// # use crate::strategies::output_locations::to_snake_case;
/// assert_eq!(to_snake_case("Canon"), "canon");
/// assert_eq!(to_snake_case("canonWhiteBalance"), "canon_white_balance");
/// assert_eq!(to_snake_case("GPS"), "gps");
/// assert_eq!(to_snake_case("AFInfo2"), "a_f_info2");
/// ```
pub fn to_snake_case(name: &str) -> String {
    // Use the same simple algorithm as SimpleTableStrategy for compatibility
    let mut result = String::new();
    let mut chars = name.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch.is_uppercase() && !result.is_empty() {
            result.push('_');
        }
        result.push(ch.to_lowercase().next().unwrap_or(ch));
    }
    
    result
}

/// Create directory path from generated file path
///
/// Extracts the directory portion from a generated file path for creating
/// necessary parent directories.
///
/// # Arguments
/// * `file_path` - Generated file path (e.g., "canon_pm/white_balance.rs")
///
/// # Returns
/// Directory path (e.g., "canon_pm")
///
/// # Examples
/// ```
/// # use crate::strategies::output_locations::extract_directory_path;
/// assert_eq!(extract_directory_path("canon_pm/white_balance.rs"), "canon_pm");
/// assert_eq!(extract_directory_path("nikon_pm/arrays/xlat_0.rs"), "nikon_pm/arrays");
/// ```
pub fn extract_directory_path(file_path: &str) -> &str {
    Path::new(file_path)
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or("")
}

/// Validate that a path follows standardized naming conventions
///
/// Checks that generated file paths conform to expected patterns:
/// - snake_case module directories ending in "_pm"
/// - snake_case filenames ending in ".rs"
/// - No mixed case violations
///
/// # Arguments
/// * `path` - File path to validate
///
/// # Returns
/// `true` if path follows conventions, `false` otherwise
///
/// # Examples
/// ```
/// # use crate::strategies::output_locations::is_valid_path;
/// assert!(is_valid_path("canon_pm/white_balance.rs"));
/// assert!(!is_valid_path("Canon_pm/WhiteBalance.rs")); // PascalCase violation
/// ```
pub fn is_valid_path(path: &str) -> bool {
    let path_obj = Path::new(path);
    
    // Check if file extension is .rs
    if path_obj.extension().and_then(|s| s.to_str()) != Some("rs") {
        return false;
    }
    
    // Check each component for snake_case compliance
    for component in path_obj.components() {
        if let Some(comp_str) = component.as_os_str().to_str() {
            if comp_str.ends_with(".rs") {
                // Check filename (without extension) is snake_case
                let filename = comp_str.strip_suffix(".rs").unwrap_or(comp_str);
                if !is_snake_case(filename) {
                    return false;
                }
            } else if comp_str.ends_with("_pm") {
                // Check module directory is snake_case
                let module_name = comp_str.strip_suffix("_pm").unwrap_or(comp_str);
                if !is_snake_case(module_name) {
                    return false;
                }
            } else if comp_str != "arrays" && comp_str != "binary_data" {
                // Other directory components should be snake_case
                if !is_snake_case(comp_str) {
                    return false;
                }
            }
        }
    }
    
    true
}

/// Check if a string is in valid snake_case format
fn is_snake_case(s: &str) -> bool {
    // Empty string is considered valid
    if s.is_empty() {
        return true;
    }
    
    // Must start with lowercase letter or digit
    let mut chars = s.chars();
    if let Some(first) = chars.next() {
        if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
            return false;
        }
    }
    
    // Rest must be lowercase, digits, or underscores
    // No consecutive underscores, no trailing underscores
    let mut prev_was_underscore = false;
    for ch in chars {
        if ch == '_' {
            if prev_was_underscore {
                return false; // No consecutive underscores
            }
            prev_was_underscore = true;
        } else if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
            prev_was_underscore = false;
        } else {
            return false; // Invalid character
        }
    }
    
    // No trailing underscore
    !s.ends_with('_')
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_module_path() {
        assert_eq!(generate_module_path("Canon", "canonWhiteBalance"), "canon_pm/canon_white_balance.rs");
        assert_eq!(generate_module_path("GPS", "coordConv"), "gps_pm/coord_conv.rs");
        assert_eq!(generate_module_path("DNG", "AdobeData"), "dng_pm/adobe_data.rs");
        assert_eq!(generate_module_path("Nikon", "AFPoints105"), "nikon_pm/a_f_points105.rs");
    }
    
    #[test]
    fn test_generate_pattern_path() {
        assert_eq!(generate_pattern_path("Nikon", "arrays", "xlat_0"), "nikon_pm/arrays/xlat_0.rs");
        assert_eq!(generate_pattern_path("Canon", "binary_data", "ProcessingBinaryData"), "canon_pm/binary_data/processing_binary_data.rs");
    }
    
    #[test]
    fn test_to_snake_case() {
        // Basic cases - single words just lowercase
        assert_eq!(to_snake_case("Canon"), "canon");
        
        // Acronyms - each letter gets underscore
        assert_eq!(to_snake_case("GPS"), "g_p_s");
        assert_eq!(to_snake_case("DNG"), "d_n_g");
        
        // camelCase - lowercase with underscores
        assert_eq!(to_snake_case("canonWhiteBalance"), "canon_white_balance");
        assert_eq!(to_snake_case("coordConv"), "coord_conv");
        
        // Mixed cases - each uppercase gets underscore
        assert_eq!(to_snake_case("AFInfo2"), "a_f_info2");
        assert_eq!(to_snake_case("AFPoints105"), "a_f_points105");
        
        // Already snake_case
        assert_eq!(to_snake_case("already_snake_case"), "already_snake_case");
        
        // Single character
        assert_eq!(to_snake_case("A"), "a");
        assert_eq!(to_snake_case("a"), "a");
        
        // Empty string
        assert_eq!(to_snake_case(""), "");
    }
    
    #[test]
    fn test_extract_directory_path() {
        assert_eq!(extract_directory_path("canon_pm/white_balance.rs"), "canon_pm");
        assert_eq!(extract_directory_path("nikon_pm/arrays/xlat_0.rs"), "nikon_pm/arrays");
        assert_eq!(extract_directory_path("single_file.rs"), "");
    }
    
    #[test]
    fn test_is_valid_path() {
        // Valid paths
        assert!(is_valid_path("canon_pm/white_balance.rs"));
        assert!(is_valid_path("gps_pm/coord_conv.rs"));
        assert!(is_valid_path("nikon_pm/arrays/xlat_0.rs"));
        
        // Invalid paths (PascalCase)
        assert!(!is_valid_path("Canon_pm/WhiteBalance.rs"));
        assert!(!is_valid_path("GPS_pm/CoordConv.rs"));
        
        // Invalid file extension
        assert!(!is_valid_path("canon_pm/white_balance.txt"));
        
        // Invalid snake_case
        assert!(!is_valid_path("canon_pm/White_Balance.rs"));
        assert!(!is_valid_path("Canon_PM/white_balance.rs"));
    }
    
    #[test]
    fn test_is_snake_case() {
        // Valid snake_case
        assert!(is_snake_case("canon"));
        assert!(is_snake_case("white_balance"));
        assert!(is_snake_case("a_f_info2"));
        assert!(is_snake_case(""));
        assert!(is_snake_case("x"));
        assert!(is_snake_case("test123"));
        
        // Invalid snake_case
        assert!(!is_snake_case("Canon")); // PascalCase
        assert!(!is_snake_case("camelCase")); // camelCase
        assert!(!is_snake_case("snake__case")); // Double underscore
        assert!(!is_snake_case("snake_case_")); // Trailing underscore
        assert!(!is_snake_case("_snake_case")); // Leading underscore
        assert!(!is_snake_case("UPPER_CASE")); // All caps
    }
}