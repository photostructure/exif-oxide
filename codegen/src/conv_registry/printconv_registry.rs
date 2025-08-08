//! PrintConv registry for mapping Perl expressions to Rust function paths
//!
//! This module provides lookup tables for PrintConv expressions, including both
//! general expressions and tag-specific mappings.

use std::collections::HashMap;
use std::sync::LazyLock;
use tracing::{debug, trace, warn};

// Registry maps Perl expressions to (module_path, function_name)
static PRINTCONV_REGISTRY: LazyLock<HashMap<&'static str, (&'static str, &'static str)>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();

        // Conditional expressions
        m.insert(
            "$val =~ /^(inf|undef)$/ ? $val : \"$val m\"",
            (
                "crate::implementations::print_conv",
                "gpsaltitude_print_conv",
            ),
        );

        // Module-scoped functions
        m.insert(
            "GPS::ConvertTimeStamp($val)",
            (
                "crate::implementations::value_conv",
                "gpstimestamp_value_conv",
            ),
        );
        m.insert(
            "ID3::ConvertTimeStamp($val)",
            (
                "crate::implementations::value_conv",
                "id3_timestamp_value_conv",
            ),
        );

        // Complex expressions (placeholder names from tag_kit.pl)
        // These need to be mapped to appropriate implementations based on the tag
        // For now, we'll need specific mappings
        m.insert(
            "complex_expression_printconv",
            (
                "crate::implementations::print_conv",
                "complex_expression_print_conv",
            ),
        );

        // ExifTool function calls that should be mapped to our implementations
        m.insert(
            "Image::ExifTool::Exif::PrintExposureTime($val)",
            (
                "crate::implementations::print_conv",
                "exposuretime_print_conv",
            ),
        );
        m.insert(
            "Image::ExifTool::Exif::PrintFNumber($val)",
            ("crate::implementations::print_conv", "fnumber_print_conv"),
        );
        m.insert(
            "Image::ExifTool::Exif::PrintFraction($val)",
            ("crate::implementations::print_conv", "print_fraction"),
        );

        // Manual function mappings (these come through as Manual type with function names)
        m.insert(
            "fnumber_print_conv",
            ("crate::implementations::print_conv", "fnumber_print_conv"),
        );
        m.insert(
            "exposuretime_print_conv",
            (
                "crate::implementations::print_conv",
                "exposuretime_print_conv",
            ),
        );
        m.insert(
            "focallength_print_conv",
            (
                "crate::implementations::print_conv",
                "focallength_print_conv",
            ),
        );

        m.insert(
            "lensinfo_print_conv",
            ("crate::implementations::print_conv", "lensinfo_print_conv"),
        );
        m.insert(
            "iso_print_conv",
            ("crate::implementations::print_conv", "iso_print_conv"),
        );
        m.insert(
            "orientation_print_conv",
            (
                "crate::implementations::print_conv",
                "orientation_print_conv",
            ),
        );
        m.insert(
            "resolutionunit_print_conv",
            (
                "crate::implementations::print_conv",
                "resolutionunit_print_conv",
            ),
        );
        m.insert(
            "ycbcrpositioning_print_conv",
            (
                "crate::implementations::print_conv",
                "ycbcrpositioning_print_conv",
            ),
        );
        m.insert(
            "gpsaltitude_print_conv",
            (
                "crate::implementations::print_conv",
                "gpsaltitude_print_conv",
            ),
        );
        m.insert(
            "gpsaltituderef_print_conv",
            (
                "crate::implementations::print_conv",
                "gpsaltituderef_print_conv",
            ),
        );
        m.insert(
            "gpslatituderef_print_conv",
            (
                "crate::implementations::print_conv",
                "gpslatituderef_print_conv",
            ),
        );
        m.insert(
            "gpslongituderef_print_conv",
            (
                "crate::implementations::print_conv",
                "gpslongituderef_print_conv",
            ),
        );
        m.insert(
            "gpslatitude_print_conv",
            (
                "crate::implementations::print_conv",
                "gpslatitude_print_conv",
            ),
        );
        m.insert(
            "gpslongitude_print_conv",
            (
                "crate::implementations::print_conv",
                "gpslongitude_print_conv",
            ),
        );
        m.insert(
            "gpsdestlatitude_print_conv",
            (
                "crate::implementations::print_conv",
                "gpsdestlatitude_print_conv",
            ),
        );
        m.insert(
            "gpsdestlongitude_print_conv",
            (
                "crate::implementations::print_conv",
                "gpsdestlongitude_print_conv",
            ),
        );
        m.insert(
            "flash_print_conv",
            ("crate::implementations::print_conv", "flash_print_conv"),
        );
        m.insert(
            "colorspace_print_conv",
            (
                "crate::implementations::print_conv",
                "colorspace_print_conv",
            ),
        );
        m.insert(
            "whitebalance_print_conv",
            (
                "crate::implementations::print_conv",
                "whitebalance_print_conv",
            ),
        );
        m.insert(
            "meteringmode_print_conv",
            (
                "crate::implementations::print_conv",
                "meteringmode_print_conv",
            ),
        );
        m.insert(
            "exposureprogram_print_conv",
            (
                "crate::implementations::print_conv",
                "exposureprogram_print_conv",
            ),
        );
        m.insert(
            "composite_gps_gpsaltitude_print_conv",
            (
                "crate::implementations::print_conv",
                "composite_gps_gpsaltitude_print_conv",
            ),
        );

        m
    });

// Tag-specific registry for ComplexHash and other special cases
// Key format: "ModuleName::TagName" (e.g., "Exif_pm::Flash") for module-specific
// or just "TagName" (e.g., "Flash") for universal tags
static TAG_SPECIFIC_PRINTCONV: LazyLock<HashMap<&'static str, (&'static str, &'static str)>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();

        // Module-specific tags (highest priority)
        m.insert(
            "Canon_pm::SelfTimer",
            (
                "crate::implementations::print_conv",
                "canon_selftimer_print_conv",
            ),
        );
        // m.insert("Canon_pm::WhiteBalance", ("crate::implementations::print_conv", "canon_white_balance_print_conv"));

        // Universal tags (work across all modules - fallback)
        m.insert(
            "Flash",
            ("crate::implementations::print_conv", "flash_print_conv"),
        );
        m.insert(
            "LensInfo",
            ("crate::implementations::print_conv", "lensinfo_print_conv"),
        );

        // GPS reference tags (ComplexHash types)
        m.insert(
            "GPSAltitudeRef",
            (
                "crate::implementations::print_conv",
                "gpsaltituderef_print_conv",
            ),
        );
        m.insert(
            "GPSLatitudeRef",
            (
                "crate::implementations::print_conv",
                "gpslatituderef_print_conv",
            ),
        );
        m.insert(
            "GPSLongitudeRef",
            (
                "crate::implementations::print_conv",
                "gpslongituderef_print_conv",
            ),
        );

        // EXIF component configuration tags
        m.insert(
            "ComponentsConfiguration",
            (
                "crate::implementations::print_conv",
                "componentsconfiguration_print_conv",
            ),
        );
        m.insert(
            "FileSource",
            (
                "crate::implementations::print_conv",
                "filesource_print_conv",
            ),
        );
        m.insert(
            "InteropVersion",
            (
                "crate::implementations::print_conv",
                "interopversion_print_conv",
            ),
        );

        // Add other tag-specific mappings here as needed

        m
    });

/// Look up a tag-specific PrintConv in the registry
/// First tries module-specific lookup (Module::Tag), then universal lookup (Tag)
pub fn lookup_tag_specific_printconv(
    module: &str,
    tag_name: &str,
) -> Option<(&'static str, &'static str)> {
    // First try module-specific lookup
    let module_key = format!("{}::{}", module, tag_name);
    if let Some(result) = TAG_SPECIFIC_PRINTCONV.get(module_key.as_str()).copied() {
        return Some(result);
    }

    // Then try universal lookup
    TAG_SPECIFIC_PRINTCONV.get(tag_name).copied()
}

/// Look up PrintConv implementation by Perl expression
/// Tries module-scoped lookup first, then unscoped
pub fn lookup_printconv(expr: &str, module: &str) -> Option<(&'static str, &'static str)> {
    use super::normalization::normalize_expression;

    let lookup_start = std::time::Instant::now();
    trace!(
        "🔍 PrintConv lookup for expr: '{}' in module: '{}'",
        expr.chars().take(50).collect::<String>(),
        module
    );

    // Normalize the expression for consistent lookup
    let normalize_start = std::time::Instant::now();
    let normalized_expr = normalize_expression(expr);
    let normalize_time = normalize_start.elapsed();

    if normalize_time.as_millis() > 50 {
        warn!(
            "🐌 SLOW normalization in PrintConv lookup: {:.2}ms",
            normalize_time.as_millis()
        );
    }

    // Normalize module name (GPS_pm -> GPS)
    let normalized_module = module.replace("_pm", "");

    // Try module-scoped first with normalized expression
    let scoped_key = format!("{}::{}", normalized_module, normalized_expr);
    if let Some(value) = PRINTCONV_REGISTRY.get(scoped_key.as_str()) {
        let total_time = lookup_start.elapsed();
        trace!(
            "✅ Found scoped PrintConv: '{}' -> {:?} in {:.2}ms",
            scoped_key,
            value,
            total_time.as_millis()
        );
        return Some(*value);
    }

    // Fall back to exact match with normalized expression
    let result = PRINTCONV_REGISTRY.get(normalized_expr.as_str()).copied();
    let total_time = lookup_start.elapsed();

    if result.is_some() {
        trace!(
            "✅ Found global PrintConv: '{}' -> {:?} in {:.2}ms",
            normalized_expr,
            result,
            total_time.as_millis()
        );
    } else {
        trace!(
            "❌ No PrintConv found for: '{}' in {:.2}ms",
            normalized_expr,
            total_time.as_millis()
        );
    }

    result
}

/// Look up PrintConv implementation by pre-normalized Perl expression (bypasses normalization)
/// Used by batch processing to avoid redundant normalization calls
pub fn lookup_printconv_direct(
    normalized_expr: &str,
    module: &str,
) -> Option<(&'static str, &'static str)> {
    // Normalize module name (GPS_pm -> GPS)
    let normalized_module = module.replace("_pm", "");

    // Try module-scoped first with pre-normalized expression
    let scoped_key = format!("{}::{}", normalized_module, normalized_expr);
    if let Some(value) = PRINTCONV_REGISTRY.get(scoped_key.as_str()) {
        return Some(*value);
    }

    // Fall back to exact match with pre-normalized expression
    PRINTCONV_REGISTRY.get(normalized_expr).copied()
}

/// Get access to the PRINTCONV_REGISTRY for testing
#[cfg(test)]
pub fn get_printconv_registry() -> &'static HashMap<&'static str, (&'static str, &'static str)> {
    &PRINTCONV_REGISTRY
}

/// Get access to the TAG_SPECIFIC_PRINTCONV for testing
#[cfg(test)]
pub fn get_tag_specific_printconv() -> &'static HashMap<&'static str, (&'static str, &'static str)>
{
    &TAG_SPECIFIC_PRINTCONV
}
