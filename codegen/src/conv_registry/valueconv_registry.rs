//! ValueConv registry for mapping Perl expressions to Rust function paths
//!
//! This module provides lookup tables for ValueConv expressions and classification
//! logic for determining whether expressions can be compiled or need custom functions.

use super::types::ValueConvType;
use crate::expression_compiler::CompiledExpression;
use std::collections::HashMap;
use std::sync::LazyLock;

static VALUECONV_REGISTRY: LazyLock<HashMap<&'static str, (&'static str, &'static str)>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();

        // GPS conversions
        m.insert(
            "Image::ExifTool::GPS::ToDegrees($val)",
            (
                "crate::implementations::value_conv",
                "gps_coordinate_value_conv",
            ),
        );
        m.insert(
            "Image::ExifTool::GPS::ConvertTimeStamp($val)",
            (
                "crate::implementations::value_conv",
                "gpstimestamp_value_conv",
            ),
        );

        // APEX conversions
        m.insert(
            "IsFloat($val) && abs($val) < 100 ? 2**(-$val) : 0",
            (
                "crate::implementations::value_conv",
                "apex_shutter_speed_value_conv",
            ),
        );
        m.insert(
            "2**($val / 2)",
            (
                "crate::implementations::value_conv",
                "apex_aperture_value_conv",
            ),
        );

        // Canon ValueConv expressions (normalized)
        m.insert(
            "exp($val / 32 * log(2)) * 100",
            (
                "crate::implementations::value_conv",
                "canon_auto_iso_value_conv",
            ),
        );
        m.insert(
            "exp($val / 32 * log(2)) * 100 / 32",
            (
                "crate::implementations::value_conv",
                "canon_base_iso_value_conv",
            ),
        );
        m.insert(
            "($val >> 16) | (($val & 0xffff) << 16)",
            (
                "crate::implementations::value_conv",
                "canon_file_number_value_conv",
            ),
        );
        m.insert(
            "(($val & 0xffc0) >> 6) * 10000 + (($val >> 16) & 0xff) + (($val & 0x3f) << 8)",
            (
                "crate::implementations::value_conv",
                "canon_directory_number_value_conv",
            ),
        );

        // Manual function mappings
        m.insert(
            "gpslatitude_value_conv",
            (
                "crate::implementations::value_conv",
                "gps_coordinate_value_conv",
            ),
        );
        m.insert(
            "gpslongitude_value_conv",
            (
                "crate::implementations::value_conv",
                "gps_coordinate_value_conv",
            ),
        );
        m.insert(
            "gpsdestlatitude_value_conv",
            (
                "crate::implementations::value_conv",
                "gps_coordinate_value_conv",
            ),
        );
        m.insert(
            "gpsdestlongitude_value_conv",
            (
                "crate::implementations::value_conv",
                "gps_coordinate_value_conv",
            ),
        );
        m.insert(
            "gpstimestamp_value_conv",
            (
                "crate::implementations::value_conv",
                "gpstimestamp_value_conv",
            ),
        );
        m.insert(
            "gpsdatestamp_value_conv",
            (
                "crate::implementations::value_conv",
                "gpsdatestamp_value_conv",
            ),
        );
        m.insert(
            "whitebalance_value_conv",
            (
                "crate::implementations::value_conv",
                "whitebalance_value_conv",
            ),
        );
        m.insert(
            "apex_shutter_speed_value_conv",
            (
                "crate::implementations::value_conv",
                "apex_shutter_speed_value_conv",
            ),
        );
        m.insert(
            "apex_aperture_value_conv",
            (
                "crate::implementations::value_conv",
                "apex_aperture_value_conv",
            ),
        );
        m.insert(
            "apex_exposure_compensation_value_conv",
            (
                "crate::implementations::value_conv",
                "apex_exposure_compensation_value_conv",
            ),
        );
        m.insert(
            "fnumber_value_conv",
            ("crate::implementations::value_conv", "fnumber_value_conv"),
        );
        m.insert(
            "exposuretime_value_conv",
            (
                "crate::implementations::value_conv",
                "exposuretime_value_conv",
            ),
        );
        m.insert(
            "focallength_value_conv",
            (
                "crate::implementations::value_conv",
                "focallength_value_conv",
            ),
        );

        // Common simple patterns found in supported tags
        m.insert(
            "$val =~ s/ +$//;\n$val",
            (
                "crate::implementations::value_conv",
                "trim_whitespace_value_conv",
            ),
        );
        m.insert(
            "$val =~ s/^.*: //;\n$val",
            (
                "crate::implementations::value_conv",
                "remove_prefix_colon_value_conv",
            ),
        );
        m.insert(
            "2**(-$val / 3)",
            (
                "crate::implementations::value_conv",
                "power_neg_div_3_value_conv",
            ),
        );
        m.insert(
            "$val ? 10 / $val : 0",
            (
                "crate::implementations::value_conv",
                "reciprocal_10_value_conv",
            ),
        );
        m.insert(
            "$val ? 2**(6 - $val / 8) : 0",
            (
                "crate::implementations::value_conv",
                "sony_exposure_time_value_conv",
            ),
        );
        // REMOVED: "$val ? exp(($val/8-6)*log(2))*100 : $val" - now compiled automatically by expression compiler
        m.insert(
            "2**(($val / 8 - 1) / 2)",
            (
                "crate::implementations::value_conv",
                "sony_fnumber_value_conv",
            ),
        );
        m.insert(
            "Image::ExifTool::Exif::ExifDate($val)",
            ("crate::implementations::value_conv", "exif_date_value_conv"),
        );

        // ExifTool function calls for datetime conversions
        m.insert(
            "require Image::ExifTool::XMP;\nreturn Image::ExifTool::XMP::ConvertXMPDate($val);",
            ("crate::implementations::value_conv", "xmp_date_value_conv"),
        );

        // String processing patterns
        m.insert(
            "length($val) > 32 ? \\$val : $val",
            (
                "crate::implementations::value_conv",
                "reference_long_string_value_conv",
            ),
        );
        m.insert(
            "length($val) > 64 ? \\$val : $val",
            (
                "crate::implementations::value_conv",
                "reference_very_long_string_value_conv",
            ),
        );

        m
    });

/// Look up ValueConv implementation by Perl expression
pub fn lookup_valueconv(expr: &str, module: &str) -> Option<(&'static str, &'static str)> {
    use super::normalization::normalize_expression;

    // First try exact match (more efficient and avoids normalization issues)
    if let Some(value) = VALUECONV_REGISTRY.get(expr) {
        return Some(*value);
    }

    // Try module-scoped exact match
    let normalized_module = module.replace("_pm", "");
    let scoped_key = format!("{}::{}", normalized_module, expr);
    if let Some(value) = VALUECONV_REGISTRY.get(scoped_key.as_str()) {
        return Some(*value);
    }

    // Fall back to normalization for complex expressions
    let normalized_expr = normalize_expression(expr);

    // Try normalized module-scoped lookup
    let normalized_scoped_key = format!("{}::{}", normalized_module, normalized_expr);
    if let Some(value) = VALUECONV_REGISTRY.get(normalized_scoped_key.as_str()) {
        return Some(*value);
    }

    // Try normalized global lookup
    VALUECONV_REGISTRY.get(normalized_expr.as_str()).copied()
}

/// Classify a ValueConv expression for code generation
///
/// Determines whether an expression can be compiled to inline arithmetic code
/// or requires a custom function implementation.
pub fn classify_valueconv_expression(expr: &str, module: &str) -> ValueConvType {
    // CRITICAL: Check registry FIRST before trying compilation
    // This fixes GPS ValueConv regression - registry lookups take precedence over compilation
    if let Some((module_path, func_name)) = lookup_valueconv(expr, module) {
        return ValueConvType::CustomFunction(module_path, func_name);
    }

    // Only try compilation if no registry entry exists
    if CompiledExpression::is_compilable(expr) {
        match CompiledExpression::compile(expr) {
            Ok(compiled) => return ValueConvType::CompiledExpression(compiled),
            Err(_) => {
                // Fall through to unimplemented case
                eprintln!(
                    "Warning: Expression '{}' looked compilable but failed compilation",
                    expr
                );
            }
        }
    }

    // No registry entry and not compilable - return unimplemented
    // Fallback - treat as unregistered custom function
    // This preserves existing behavior for unknown expressions
    ValueConvType::CustomFunction("crate::implementations::missing", "missing_value_conv")
}

/// Get access to the VALUECONV_REGISTRY for testing
#[cfg(test)]
pub fn get_valueconv_registry() -> &'static HashMap<&'static str, (&'static str, &'static str)> {
    &VALUECONV_REGISTRY
}
