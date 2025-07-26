//! Codegen-time registry for PrintConv/ValueConv mappings
//! 
//! This module provides compile-time lookup of Perl expressions to Rust function paths.
//! The registry is used during code generation to emit direct function calls,
//! eliminating runtime lookup overhead.

use std::collections::HashMap;
use std::sync::LazyLock;

// Registry maps Perl expressions to (module_path, function_name)
static PRINTCONV_REGISTRY: LazyLock<HashMap<&'static str, (&'static str, &'static str)>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    
    // Common sprintf patterns
    m.insert("sprintf(\"%.1f mm\",$val)", ("crate::implementations::print_conv", "focallength_print_conv"));
    m.insert("sprintf(\"%.1f\",$val)", ("crate::implementations::print_conv", "decimal_1_print_conv"));
    m.insert("sprintf(\"%.2f\",$val)", ("crate::implementations::print_conv", "decimal_2_print_conv"));
    m.insert("sprintf(\"%+d\",$val)", ("crate::implementations::print_conv", "signed_int_print_conv"));
    m.insert("sprintf(\"%.3f mm\",$val)", ("crate::implementations::print_conv", "focal_length_3_decimals_print_conv"));
    
    // Conditional expressions
    m.insert("$val =~ /^(inf|undef)$/ ? $val : \"$val m\"", ("crate::implementations::print_conv", "gpsaltitude_print_conv"));
    
    // Module-scoped functions
    m.insert("GPS.pm::ConvertTimeStamp($val)", ("crate::implementations::value_conv", "gpstimestamp_value_conv"));
    m.insert("ID3.pm::ConvertTimeStamp($val)", ("crate::implementations::value_conv", "id3_timestamp_value_conv"));
    
    // Complex expressions (placeholder names from tag_kit.pl)
    // These need to be mapped to appropriate implementations based on the tag
    // For now, we'll need specific mappings
    m.insert("complex_expression_printconv", ("crate::implementations::print_conv", "complex_expression_print_conv"));
    
    // ExifTool function calls that should be mapped to our implementations
    m.insert("Image::ExifTool::Exif::PrintExposureTime($val)", ("crate::implementations::print_conv", "exposuretime_print_conv"));
    m.insert("Image::ExifTool::Exif::PrintFNumber($val)", ("crate::implementations::print_conv", "fnumber_print_conv"));
    m.insert("Image::ExifTool::Exif::PrintFraction($val)", ("crate::implementations::print_conv", "print_fraction"));
    
    // Manual function mappings (these come through as Manual type with function names)
    m.insert("fnumber_print_conv", ("crate::implementations::print_conv", "fnumber_print_conv"));
    m.insert("exposuretime_print_conv", ("crate::implementations::print_conv", "exposuretime_print_conv"));
    m.insert("focallength_print_conv", ("crate::implementations::print_conv", "focallength_print_conv"));
    m.insert("lensinfo_print_conv", ("crate::implementations::print_conv", "lensinfo_print_conv"));
    m.insert("iso_print_conv", ("crate::implementations::print_conv", "iso_print_conv"));
    m.insert("orientation_print_conv", ("crate::implementations::print_conv", "orientation_print_conv"));
    m.insert("resolutionunit_print_conv", ("crate::implementations::print_conv", "resolutionunit_print_conv"));
    m.insert("ycbcrpositioning_print_conv", ("crate::implementations::print_conv", "ycbcrpositioning_print_conv"));
    m.insert("gpsaltitude_print_conv", ("crate::implementations::print_conv", "gpsaltitude_print_conv"));
    m.insert("gpsaltituderef_print_conv", ("crate::implementations::print_conv", "gpsaltituderef_print_conv"));
    m.insert("gpslatituderef_print_conv", ("crate::implementations::print_conv", "gpslatituderef_print_conv"));
    m.insert("gpslongituderef_print_conv", ("crate::implementations::print_conv", "gpslongituderef_print_conv"));
    m.insert("gpslatitude_print_conv", ("crate::implementations::print_conv", "gpslatitude_print_conv"));
    m.insert("gpslongitude_print_conv", ("crate::implementations::print_conv", "gpslongitude_print_conv"));
    m.insert("gpsdestlatitude_print_conv", ("crate::implementations::print_conv", "gpsdestlatitude_print_conv"));
    m.insert("gpsdestlongitude_print_conv", ("crate::implementations::print_conv", "gpsdestlongitude_print_conv"));
    m.insert("flash_print_conv", ("crate::implementations::print_conv", "flash_print_conv"));
    m.insert("colorspace_print_conv", ("crate::implementations::print_conv", "colorspace_print_conv"));
    m.insert("whitebalance_print_conv", ("crate::implementations::print_conv", "whitebalance_print_conv"));
    m.insert("meteringmode_print_conv", ("crate::implementations::print_conv", "meteringmode_print_conv"));
    m.insert("exposureprogram_print_conv", ("crate::implementations::print_conv", "exposureprogram_print_conv"));
    m.insert("composite_gps_gpsaltitude_print_conv", ("crate::implementations::print_conv", "composite_gps_gpsaltitude_print_conv"));
    
    m
});

static VALUECONV_REGISTRY: LazyLock<HashMap<&'static str, (&'static str, &'static str)>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    
    // GPS conversions
    m.insert("Image::ExifTool::GPS::ToDegrees($val)", ("crate::implementations::value_conv", "gps_coordinate_value_conv"));
    m.insert("Image::ExifTool::GPS::ConvertTimeStamp($val)", ("crate::implementations::value_conv", "gpstimestamp_value_conv"));
    
    // APEX conversions
    m.insert("IsFloat($val) && abs($val)<100 ? 2**(-$val) : 0", ("crate::implementations::value_conv", "apex_shutter_speed_value_conv"));
    m.insert("2 ** ($val / 2)", ("crate::implementations::value_conv", "apex_aperture_value_conv"));
    
    // Manual function mappings
    m.insert("gpslatitude_value_conv", ("crate::implementations::value_conv", "gps_coordinate_value_conv"));
    m.insert("gpslongitude_value_conv", ("crate::implementations::value_conv", "gps_coordinate_value_conv"));
    m.insert("gpsdestlatitude_value_conv", ("crate::implementations::value_conv", "gps_coordinate_value_conv"));
    m.insert("gpsdestlongitude_value_conv", ("crate::implementations::value_conv", "gps_coordinate_value_conv"));
    m.insert("gpstimestamp_value_conv", ("crate::implementations::value_conv", "gpstimestamp_value_conv"));
    m.insert("gpsdatestamp_value_conv", ("crate::implementations::value_conv", "gpsdatestamp_value_conv"));
    m.insert("whitebalance_value_conv", ("crate::implementations::value_conv", "whitebalance_value_conv"));
    m.insert("apex_shutter_speed_value_conv", ("crate::implementations::value_conv", "apex_shutter_speed_value_conv"));
    m.insert("apex_aperture_value_conv", ("crate::implementations::value_conv", "apex_aperture_value_conv"));
    m.insert("apex_exposure_compensation_value_conv", ("crate::implementations::value_conv", "apex_exposure_compensation_value_conv"));
    m.insert("fnumber_value_conv", ("crate::implementations::value_conv", "fnumber_value_conv"));
    m.insert("exposuretime_value_conv", ("crate::implementations::value_conv", "exposuretime_value_conv"));
    m.insert("focallength_value_conv", ("crate::implementations::value_conv", "focallength_value_conv"));
    
    m
});

/// Look up PrintConv implementation by Perl expression
/// Tries module-scoped lookup first, then unscoped
pub fn lookup_printconv(expr: &str, module: &str) -> Option<(&'static str, &'static str)> {
    // Normalize module name (GPS_pm -> GPS.pm)
    let normalized_module = module.replace("_pm", ".pm");
    
    // Try module-scoped first
    let scoped_key = format!("{}::{}", normalized_module, expr);
    if let Some(value) = PRINTCONV_REGISTRY.get(scoped_key.as_str()) {
        return Some(*value);
    }
    
    // Fall back to exact match
    PRINTCONV_REGISTRY.get(expr).copied()
}

/// Look up ValueConv implementation by Perl expression
pub fn lookup_valueconv(expr: &str, module: &str) -> Option<(&'static str, &'static str)> {
    // Same module-scoped logic as PrintConv
    let normalized_module = module.replace("_pm", ".pm");
    let scoped_key = format!("{}::{}", normalized_module, expr);
    
    if let Some(value) = VALUECONV_REGISTRY.get(scoped_key.as_str()) {
        return Some(*value);
    }
    
    VALUECONV_REGISTRY.get(expr).copied()
}

/// Normalize expression for consistent lookup
/// Handles whitespace normalization and other variations
pub fn normalize_expression(expr: &str) -> String {
    // Remove spaces around punctuation and collapse whitespace
    let mut result = String::new();
    let mut last_was_space = false;
    let mut chars = expr.chars().peekable();
    
    while let Some(ch) = chars.next() {
        match ch {
            ' ' | '\t' | '\n' | '\r' => {
                // Skip spaces before punctuation
                if let Some(&next) = chars.peek() {
                    if matches!(next, '(' | ')' | ',' | '"') {
                        continue;
                    }
                }
                if !last_was_space && !result.is_empty() {
                    result.push(' ');
                    last_was_space = true;
                }
            }
            '(' | ')' | ',' => {
                // Remove trailing space before punctuation
                if last_was_space && !result.is_empty() {
                    result.pop();
                }
                result.push(ch);
                last_was_space = false;
                // Skip spaces after punctuation
                while chars.peek() == Some(&' ') {
                    chars.next();
                }
            }
            _ => {
                result.push(ch);
                last_was_space = false;
            }
        }
    }
    
    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_module_scoped_lookup() {
        // Test direct lookup of a known value
        let result = lookup_valueconv("Image::ExifTool::GPS::ConvertTimeStamp($val)", "GPS_pm");
        assert_eq!(result, Some(("crate::implementations::value_conv", "gpstimestamp_value_conv")));
    }
    
    #[test]
    fn test_expression_normalization() {
        assert_eq!(
            normalize_expression("sprintf( \"%.1f mm\" , $val )"),
            "sprintf(\"%.1f mm\",$val)"
        );
    }
    
    #[test]
    fn test_manual_printconv_lookup() {
        let result = lookup_printconv("fnumber_print_conv", "Exif_pm");
        assert_eq!(result, Some(("crate::implementations::print_conv", "fnumber_print_conv")));
    }
}