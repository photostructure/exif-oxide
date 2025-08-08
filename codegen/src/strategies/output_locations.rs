//! Standardized output location generation for strategy system
//!
//! This module provides consistent path generation for all extraction strategies,
//! ensuring snake_case naming convention and predictable directory structure.

use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

/// Generate standardized module path following Rust naming conventions
///
/// Converts ExifTool module names to snake_case directory names (e.g., "Canon" → "canon",
/// "GPS" → "gps", "PanasonicRaw" → "panasonic_raw") and symbol names to snake_case filenames
/// for consistent Rust organization. Only adds suffix for the 2 known name collisions.
///
/// # Arguments
/// * `module_name` - ExifTool module name (e.g., "Canon", "GPS", "DNG", "PanasonicRaw")
/// * `symbol_name` - Symbol name from the module (e.g., "canonWhiteBalance")
///
/// # Returns
/// Relative path from output directory: `[module_name]/[symbol_name].rs` (with suffix only for collisions)
///
/// # Examples
/// ```
/// # use codegen::strategies::output_locations::generate_module_path;
/// assert_eq!(generate_module_path("Canon", "canonWhiteBalance"), "canon/canon_white_balance.rs");
/// assert_eq!(generate_module_path("GPS", "coordConv"), "gps/coord_conv.rs");
/// assert_eq!(generate_module_path("PanasonicRaw", "whiteBalance"), "panasonic_raw/white_balance.rs");
/// ```
pub fn generate_module_path(module_name: &str, symbol_name: &str) -> String {
    let module_dir = module_name_to_directory(module_name);
    let filename = format!("{}.rs", to_snake_case(symbol_name));
    format!("{}/{}", module_dir, filename)
}

/// Convert ExifTool module name to directory name
///
/// Handles compound module names and known collisions:
/// - Simple names: "Canon" → "canon"
/// - Acronyms: "GPS" → "gps", "DNG" → "dng" (idiomatic lowercase)  
/// - Compound names: "PanasonicRaw" → "panasonic_raw", "MinoltaRaw" → "minolta_raw"
/// - Known collisions: "Charset" → "charset_pm", "Geolocation" → "geolocation_pm"
///
/// # Arguments
/// * `module_name` - ExifTool module name (e.g., "Canon", "PanasonicRaw")
///
/// # Returns
/// Directory name string
///
/// # Examples
/// ```
/// # use codegen::strategies::output_locations::module_name_to_directory;
/// assert_eq!(module_name_to_directory("Canon"), "canon");
/// assert_eq!(module_name_to_directory("PanasonicRaw"), "panasonic_raw");
/// assert_eq!(module_name_to_directory("Charset"), "charset_pm");
/// ```
pub fn module_name_to_directory(module_name: &str) -> String {
    // Handle known collisions (only 2 exist according to TPP)
    match module_name {
        "Charset" => "charset_pm".to_string(),
        "Geolocation" => "geolocation_pm".to_string(),
        _ => {
            // Convert to snake_case for compound names
            to_snake_case(module_name)
        }
    }
}

/// Generate path for specialized pattern types (arrays, binary data, etc.)
///
/// Creates subdirectories for different pattern types to organize complex
/// generated code structures.
///
/// # Arguments
/// * `module_name` - ExifTool module name (e.g., "Canon", "PanasonicRaw")
/// * `pattern_type` - Type of pattern (e.g., "arrays", "binary_data")
/// * `symbol_name` - Symbol name from the module
///
/// # Returns
/// Relative path: `[module_name]/[pattern_type]/[symbol_name].rs` (with suffix only for collisions)
///
/// # Examples
/// ```
/// # use codegen::strategies::output_locations::generate_pattern_path;
/// assert_eq!(generate_pattern_path("Nikon", "arrays", "xlat_0"), "nikon/arrays/xlat_0.rs");
/// assert_eq!(generate_pattern_path("PanasonicRaw", "binary_data", "wbInfo"), "panasonic_raw/binary_data/wb_info.rs");
/// ```
pub fn generate_pattern_path(module_name: &str, pattern_type: &str, symbol_name: &str) -> String {
    let module_dir = module_name_to_directory(module_name);
    let filename = format!("{}.rs", to_snake_case(symbol_name));
    format!("{}/{}/{}", module_dir, pattern_type, filename)
}

/// Convert string to snake_case following Rust naming conventions
///
/// Handles common ExifTool naming patterns:
/// - PascalCase: "Canon" → "canon"
/// - camelCase: "canonWhiteBalance" → "canon_white_balance"  
/// - Acronyms: "GPS" → "gps", "DNG" → "dng"
/// - Mixed: "AFInfo2" → "af_info2"
///
/// # Arguments
/// * `name` - Input string in any case format
///
/// # Returns
/// String converted to snake_case
///
/// # Examples
/// ```
/// # use codegen::strategies::output_locations::to_snake_case;
/// assert_eq!(to_snake_case("Canon"), "canon");
/// assert_eq!(to_snake_case("canonWhiteBalance"), "canon_white_balance");
/// assert_eq!(to_snake_case("GPS"), "gps");
/// assert_eq!(to_snake_case("AFInfo2"), "af_info2");
/// ```
static SNAKE_CASE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    // Insert underscores before uppercase letters, with special handling for acronyms
    Regex::new(r"([a-z])([A-Z])|([A-Z])([A-Z][a-z])").expect("Snake case regex pattern is valid")
});

pub fn to_snake_case(name: &str) -> String {
    if name.is_empty() {
        return String::new();
    }

    // Simple case: all-uppercase becomes lowercase
    if name.len() > 1 && name.chars().all(|c| c.is_uppercase() || c.is_ascii_digit()) {
        return name.to_lowercase();
    }

    // Insert underscores at word boundaries
    let result = SNAKE_CASE_PATTERN.replace_all(name, |caps: &regex::Captures| {
        if let Some(m1) = caps.get(1) {
            if let Some(m2) = caps.get(2) {
                // camelCase transition: lower followed by upper
                format!("{}_{}", m1.as_str(), m2.as_str())
            } else {
                caps.get(0).unwrap().as_str().to_string()
            }
        } else if let Some(m3) = caps.get(3) {
            if let Some(m4) = caps.get(4) {
                // Acronym transition: upper followed by upper+lower (XMLHttp -> XML_Http)
                format!("{}_{}", m3.as_str(), m4.as_str())
            } else {
                caps.get(0).unwrap().as_str().to_string()
            }
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    });

    result.to_lowercase()
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
/// # use codegen::strategies::output_locations::extract_directory_path;
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
/// - snake_case module directories (suffix only for known collisions)
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
/// # use codegen::strategies::output_locations::is_valid_path;
/// assert!(is_valid_path("canon/white_balance.rs"));
/// assert!(is_valid_path("panasonic_raw/white_balance.rs"));
/// assert!(is_valid_path("charset_pm/encoding.rs")); // Known collision
/// assert!(!is_valid_path("Canon/WhiteBalance.rs")); // PascalCase violation
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
                // Check module directory with _pm suffix is snake_case (only for known collisions)
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
        assert_eq!(
            generate_module_path("Canon", "canonWhiteBalance"),
            "canon/canon_white_balance.rs"
        );
        assert_eq!(
            generate_module_path("GPS", "coordConv"),
            "gps/coord_conv.rs"
        );
        assert_eq!(
            generate_module_path("DNG", "AdobeData"),
            "dng/adobe_data.rs"
        );
        assert_eq!(
            generate_module_path("Nikon", "AFPoints105"),
            "nikon/af_points105.rs"
        );
        assert_eq!(
            generate_module_path("PanasonicRaw", "whiteBalance"),
            "panasonic_raw/white_balance.rs"
        );
        assert_eq!(
            generate_module_path("MinoltaRaw", "testPattern"),
            "minolta_raw/test_pattern.rs"
        );
        // Test known collisions
        assert_eq!(
            generate_module_path("Charset", "encoding"),
            "charset_pm/encoding.rs"
        );
        assert_eq!(
            generate_module_path("Geolocation", "coords"),
            "geolocation_pm/coords.rs"
        );
    }

    #[test]
    fn test_generate_pattern_path() {
        assert_eq!(
            generate_pattern_path("Nikon", "arrays", "xlat_0"),
            "nikon/arrays/xlat_0.rs"
        );
        assert_eq!(
            generate_pattern_path("Canon", "binary_data", "ProcessingBinaryData"),
            "canon/binary_data/processing_binary_data.rs"
        );
        assert_eq!(
            generate_pattern_path("PanasonicRaw", "binary_data", "wbInfo"),
            "panasonic_raw/binary_data/wb_info.rs"
        );
    }

    #[test]
    fn test_module_name_to_directory() {
        // Simple names - just lowercase
        assert_eq!(module_name_to_directory("Canon"), "canon");
        assert_eq!(module_name_to_directory("GPS"), "gps");
        assert_eq!(module_name_to_directory("DNG"), "dng");

        // Compound names - convert to snake_case
        assert_eq!(module_name_to_directory("PanasonicRaw"), "panasonic_raw");
        assert_eq!(module_name_to_directory("MinoltaRaw"), "minolta_raw");
        assert_eq!(module_name_to_directory("CanonRaw"), "canon_raw");

        // Known collisions - keep _pm suffix
        assert_eq!(module_name_to_directory("Charset"), "charset_pm");
        assert_eq!(module_name_to_directory("Geolocation"), "geolocation_pm");
    }

    #[test]
    fn test_to_snake_case() {
        // Basic cases - single words just lowercase
        assert_eq!(to_snake_case("Canon"), "canon");

        // Acronyms - all-caps become simple lowercase (more idiomatic)
        assert_eq!(to_snake_case("GPS"), "gps");
        assert_eq!(to_snake_case("DNG"), "dng");
        assert_eq!(to_snake_case("IPTC"), "iptc");
        assert_eq!(to_snake_case("JPEG"), "jpeg");
        assert_eq!(to_snake_case("PDF"), "pdf");
        assert_eq!(to_snake_case("PNG"), "png");
        assert_eq!(to_snake_case("RIFF"), "riff");
        assert_eq!(to_snake_case("XMP"), "xmp");

        // Acronyms with numbers
        assert_eq!(to_snake_case("JPEG2000"), "jpeg2000");

        // camelCase - lowercase with underscores
        assert_eq!(to_snake_case("canonWhiteBalance"), "canon_white_balance");
        assert_eq!(to_snake_case("canonModelID"), "canon_model_id");

        // Test nikonLensIDs → nikon_lens_ids (the original issue)
        assert_eq!(to_snake_case("nikonLensIDs"), "nikon_lens_ids");
        assert_eq!(to_snake_case("coordConv"), "coord_conv");

        // Test both camelCase and PascalCase should produce same result
        assert_eq!(to_snake_case("nikonTest"), "nikon_test");
        assert_eq!(to_snake_case("NikonTest"), "nikon_test");

        // Test word boundary detection
        assert_eq!(to_snake_case("AFInfo2"), "af_info2");
        assert_eq!(to_snake_case("GPSLatitude"), "gps_latitude");
        assert_eq!(to_snake_case("ISOSpeed"), "iso_speed");
        assert_eq!(to_snake_case("FNumber"), "f_number");
        assert_eq!(to_snake_case("XResolution"), "x_resolution");
        assert_eq!(to_snake_case("YResolution"), "y_resolution");
        assert_eq!(to_snake_case("XMLHttpRequest"), "xml_http_request");
        assert_eq!(to_snake_case("HelloWorld"), "hello_world");

        // Edge cases showing simple algorithm behavior
        assert_eq!(to_snake_case("Fnumber"), "fnumber"); // Different from FNumber -> f_number (collision-safe)
        assert_eq!(to_snake_case("already_snake_case"), "already_snake_case");
        assert_eq!(to_snake_case("AELButton"), "ael_button");
        assert_eq!(to_snake_case("CAFMode"), "caf_mode");
        assert_eq!(to_snake_case("AFInfo"), "af_info");
        assert_eq!(to_snake_case("GPSData"), "gps_data");

        // Numbers with adjacent capitals (critical edge cases from the data)
        assert_eq!(to_snake_case("AFPointsInFocus1D"), "af_points_in_focus1d");
        assert_eq!(to_snake_case("AFPointsInFocus5D"), "af_points_in_focus5d");
        assert_eq!(to_snake_case("AFInfo2Version"), "af_info2version");
        assert_eq!(to_snake_case("Func1Button"), "func1button");
        assert_eq!(to_snake_case("MovieFunc2Button"), "movie_func2button");

        // Multiple consecutive acronyms
        assert_eq!(to_snake_case("USBPowerDelivery"), "usb_power_delivery");
        assert_eq!(to_snake_case("HDRImageType"), "hdr_image_type");
        assert_eq!(to_snake_case("CCDScanMode"), "ccd_scan_mode");
        assert_eq!(to_snake_case("DNGVersion"), "dng_version");

        // ID patterns (very common in the data)
        assert_eq!(to_snake_case("CameraModelID"), "camera_model_id");
        assert_eq!(to_snake_case("VendorID"), "vendor_id");
        assert_eq!(to_snake_case("BurstID"), "burst_id");

        // WB patterns (white balance - extremely common)
        assert_eq!(to_snake_case("WBShiftAB"), "wb_shift_ab");
        assert_eq!(to_snake_case("WBRedLevel"), "wb_red_level");

        // Single letter at end of word patterns -- note the "xposition" instead
        // of "x_position", but hey, the function is much simpler if we don't
        // handle this special case.
        assert_eq!(to_snake_case("AFAreaXPosition"), "af_area_xposition");
        assert_eq!(to_snake_case("AFAreaYPosition"), "af_area_yposition");

        // Single character
        assert_eq!(to_snake_case("A"), "a");
        assert_eq!(to_snake_case("a"), "a");

        // Empty string
        assert_eq!(to_snake_case(""), "");
    }

    #[test]
    fn test_extract_directory_path() {
        assert_eq!(extract_directory_path("canon/white_balance.rs"), "canon");
        assert_eq!(
            extract_directory_path("nikon/arrays/xlat_0.rs"),
            "nikon/arrays"
        );
        assert_eq!(
            extract_directory_path("panasonic_raw/white_balance.rs"),
            "panasonic_raw"
        );
        assert_eq!(
            extract_directory_path("charset_pm/encoding.rs"),
            "charset_pm"
        );
        assert_eq!(extract_directory_path("single_file.rs"), "");
    }

    #[test]
    fn test_is_valid_path() {
        // Valid paths - new naming without _pm suffix
        assert!(is_valid_path("canon/white_balance.rs"));
        assert!(is_valid_path("gps/coord_conv.rs"));
        assert!(is_valid_path("nikon/arrays/xlat_0.rs"));
        assert!(is_valid_path("panasonic_raw/white_balance.rs"));

        // Valid paths - known collisions with _pm suffix
        assert!(is_valid_path("charset_pm/encoding.rs"));
        assert!(is_valid_path("geolocation_pm/coords.rs"));

        // Invalid paths (PascalCase)
        assert!(!is_valid_path("Canon/WhiteBalance.rs"));
        assert!(!is_valid_path("GPS/CoordConv.rs"));
        assert!(!is_valid_path("PanasonicRaw/WhiteBalance.rs"));

        // Invalid file extension
        assert!(!is_valid_path("canon/white_balance.txt"));

        // Invalid snake_case
        assert!(!is_valid_path("canon/White_Balance.rs"));
        assert!(!is_valid_path("CANON/white_balance.rs"));
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
