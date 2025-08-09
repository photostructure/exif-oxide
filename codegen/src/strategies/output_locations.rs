//! Standardized output location generation for strategy system
//!
//! This module provides consistent path generation for all extraction strategies,
//! ensuring ExifTool-faithful naming with _pm suffix for clear traceability.

use regex::Regex;
use std::sync::LazyLock;

/// Generate standardized module path with ExifTool-faithful naming
///
/// Maintains ExifTool module names with _pm suffix for clear traceability
/// (e.g., "Canon" → "Canon_pm", "GPS" → "GPS_pm", "PanasonicRaw" → "PanasonicRaw_pm")
/// while converting symbol names to snake_case for Rust conventions.
///
/// # Arguments
/// * `module_name` - ExifTool module name (e.g., "Canon", "GPS", "DNG", "PanasonicRaw")
/// * `symbol_name` - Symbol name from the module (e.g., "canonWhiteBalance")
///
/// # Returns
/// Relative path from output directory: `[module_name]_pm/[symbol_name].rs`
///
/// # Examples
/// ```
/// # use codegen::strategies::output_locations::generate_module_path;
/// assert_eq!(generate_module_path("Canon", "canonWhiteBalance"), "Canon_pm/canon_white_balance.rs");
/// assert_eq!(generate_module_path("GPS", "coordConv"), "GPS_pm/coord_conv.rs");
/// assert_eq!(generate_module_path("PanasonicRaw", "whiteBalance"), "PanasonicRaw_pm/white_balance.rs");
/// ```
pub fn generate_module_path(module_name: &str, symbol_name: &str) -> String {
    let module_dir = module_name_to_directory(module_name);
    let filename = format!("{}.rs", to_snake_case(symbol_name));
    format!("{module_dir}/{filename}")
}

/// Convert ExifTool module name to directory name with _pm suffix
///
/// All modules get the _pm suffix to maintain clear traceability to ExifTool source files.
/// This makes it immediately obvious that generated code comes from ExifTool perl modules.
///
/// # Arguments
/// * `module_name` - ExifTool module name (e.g., "Canon", "PanasonicRaw")
///
/// # Returns
/// Directory name string with _pm suffix
///
/// # Examples
/// ```
/// # use codegen::strategies::output_locations::module_name_to_directory;
/// assert_eq!(module_name_to_directory("Canon"), "Canon_pm");
/// assert_eq!(module_name_to_directory("PanasonicRaw"), "PanasonicRaw_pm");
/// assert_eq!(module_name_to_directory("GPS"), "GPS_pm");
/// assert_eq!(module_name_to_directory("ExifTool"), "ExifTool_pm");
/// ```
pub fn module_name_to_directory(module_name: &str) -> String {
    // Simply append _pm to maintain ExifTool module name visibility
    format!("{}_pm", module_name)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_module_path() {
        assert_eq!(
            generate_module_path("Canon", "canonWhiteBalance"),
            "Canon_pm/canon_white_balance.rs"
        );
        assert_eq!(
            generate_module_path("Canon", "CameraInfo7D"),
            "Canon_pm/camera_info7d.rs"
        );
        assert_eq!(
            generate_module_path("GPS", "coordConv"),
            "GPS_pm/coord_conv.rs"
        );
        assert_eq!(
            generate_module_path("DNG", "AdobeData"),
            "DNG_pm/adobe_data.rs"
        );
        assert_eq!(
            generate_module_path("Nikon", "AFPoints105"),
            "Nikon_pm/af_points105.rs"
        );
        assert_eq!(
            generate_module_path("PanasonicRaw", "whiteBalance"),
            "PanasonicRaw_pm/white_balance.rs"
        );
        assert_eq!(
            generate_module_path("MinoltaRaw", "testPattern"),
            "MinoltaRaw_pm/test_pattern.rs"
        );
        assert_eq!(
            generate_module_path("ExifTool", "createTypes"),
            "ExifTool_pm/create_types.rs"
        );
        assert_eq!(
            generate_module_path("Jpeg2000", "boxHandler"),
            "Jpeg2000_pm/box_handler.rs"
        );
    }

    #[test]
    fn test_module_name_to_directory() {
        // All names get _pm suffix for ExifTool traceability
        assert_eq!(module_name_to_directory("Canon"), "Canon_pm");
        assert_eq!(module_name_to_directory("GPS"), "GPS_pm");
        assert_eq!(module_name_to_directory("DNG"), "DNG_pm");
        assert_eq!(module_name_to_directory("JPEG"), "JPEG_pm");
        assert_eq!(module_name_to_directory("XMP"), "XMP_pm");

        // Compound names preserve ExifTool casing
        assert_eq!(module_name_to_directory("PanasonicRaw"), "PanasonicRaw_pm");
        assert_eq!(module_name_to_directory("MinoltaRaw"), "MinoltaRaw_pm");
        assert_eq!(module_name_to_directory("CanonRaw"), "CanonRaw_pm");
        assert_eq!(module_name_to_directory("ExifTool"), "ExifTool_pm");
        assert_eq!(module_name_to_directory("FujiFilm"), "FujiFilm_pm");
        assert_eq!(module_name_to_directory("QuickTime"), "QuickTime_pm");
        assert_eq!(module_name_to_directory("Jpeg2000"), "Jpeg2000_pm");
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
}
