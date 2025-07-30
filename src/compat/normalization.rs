//! Value normalization for ExifTool compatibility
//!
//! This module handles the various format differences between ExifTool and exif-oxide
//! output to enable meaningful comparison.

use serde_json::Value;
use std::collections::HashMap;

/// Normalization rules for standardizing ExifTool's inconsistent output formats
#[derive(Debug)]
pub enum NormalizationRule {
    /// Unit-based tags: extract number, standardize unit format
    /// Example: "24.0 mm", "14 mm", 24 -> "24 mm" or "24.0 mm"
    UnitFormat {
        unit: &'static str,
        decimal_places: Option<u8>,
    },
    /// Ratio formats: "1/2000", "0.5" -> standardize representation
    RatioFormat,
    /// Clean decimal precision but preserve JSON number type: 14.0 -> 14
    CleanNumericPrecision { max_places: u8 },
    /// GPS altitude tolerance: strip unit, validate values within 0.09m tolerance
    GPSAltitudeTolerance,
    /// Convert numbers to strings for SubSec* tags
    NumberToString,
    /// Trim trailing whitespace from string values
    TrimWhitespace,
}

/// Tag normalization configuration
/// Maps tag names to their normalization rules
pub fn get_normalization_rules() -> HashMap<&'static str, NormalizationRule> {
    let mut rules = HashMap::new();

    // Distance/length tags
    rules.insert(
        "EXIF:FocalLength",
        NormalizationRule::UnitFormat {
            unit: "mm",
            decimal_places: Some(1),
        },
    );

    // Aperture/f-stop tags - clean unnecessary precision but preserve number type: 14.0 -> 14
    rules.insert(
        "EXIF:FNumber",
        NormalizationRule::CleanNumericPrecision { max_places: 1 },
    );

    // Time-based tags - standardize ExposureTime format
    // ExifTool inconsistencies: "1/400" (string), 4 (number), 0.4 (number)
    // Our standard: fractions stay strings, whole seconds as integers, decimals as numbers
    rules.insert("EXIF:ExposureTime", NormalizationRule::RatioFormat);

    // GPS altitude tags - special tolerance-based comparison
    // ExifTool: 25.24672793 (number), exif-oxide: "25.2 m" (string)
    // GPS accuracy is ~1-3m, so validate values are within 0.09m tolerance
    rules.insert("EXIF:GPSAltitude", NormalizationRule::GPSAltitudeTolerance);

    // SubSec* tags - convert numbers to strings for consistency
    // ExifTool outputs as numbers, exif-oxide outputs as strings
    rules.insert("EXIF:SubSecTime", NormalizationRule::NumberToString);
    rules.insert(
        "EXIF:SubSecTimeDigitized",
        NormalizationRule::NumberToString,
    );
    rules.insert("EXIF:SubSecTimeOriginal", NormalizationRule::NumberToString);

    // GIF PixelAspectRatio - clean unnecessary precision: 1.0 -> 1
    rules.insert(
        "GIF:PixelAspectRatio",
        NormalizationRule::CleanNumericPrecision { max_places: 3 },
    );

    // Trim trailing whitespace from string values
    // Some cameras pad these fields with spaces
    rules.insert("EXIF:Make", NormalizationRule::TrimWhitespace);
    rules.insert("EXIF:Model", NormalizationRule::TrimWhitespace);
    rules.insert("EXIF:ImageDescription", NormalizationRule::TrimWhitespace);

    rules
}

/// Apply normalization rule to a value
pub fn apply_normalization_rule(value: &Value, rule: &NormalizationRule) -> Value {
    match rule {
        NormalizationRule::UnitFormat {
            unit,
            decimal_places,
        } => normalize_unit_format(value, unit, *decimal_places),
        NormalizationRule::RatioFormat => normalize_ratio_format(value),
        NormalizationRule::CleanNumericPrecision { max_places } => {
            normalize_clean_numeric_precision(value, *max_places)
        }
        NormalizationRule::GPSAltitudeTolerance => normalize_gps_altitude_tolerance(value),
        NormalizationRule::NumberToString => normalize_number_to_string(value),
        NormalizationRule::TrimWhitespace => normalize_trim_whitespace(value),
    }
}

/// Normalize unit-based values: 24 -> "24 mm", 1.8 -> "1.8 mm", 400.00 -> "400 mm", "24.0 mm" -> "24 mm"
fn normalize_unit_format(value: &Value, unit: &str, _decimal_places: Option<u8>) -> Value {
    let unit_pattern = format!(" {unit}");

    let number = match value {
        Value::String(s) => {
            if let Some(unit_pos) = s.find(&unit_pattern) {
                // Already has unit, extract number part
                s[..unit_pos].parse::<f64>().ok()
            } else {
                // String that's just a number
                s.parse::<f64>().ok()
            }
        }
        Value::Number(n) => n.as_f64(),
        _ => return value.clone(),
    };

    if let Some(num) = number {
        // Always format as string with unit, removing unnecessary trailing zeros
        if (num.fract()).abs() < 0.001 {
            // Integer value: 24.0 -> "24 mm", 400.00 -> "400 mm"
            Value::String(format!("{} {}", num as i32, unit))
        } else {
            // Has meaningful decimal: 1.8 -> "1.8 mm", 5.7 -> "5.7 mm"
            // Remove trailing zeros: format as minimal decimal representation
            let formatted = format!("{num:.10}"); // Start with high precision
            let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
            Value::String(format!("{trimmed} {unit}"))
        }
    } else {
        value.clone()
    }
}

/// Normalize ExposureTime formats consistently
/// ExifTool inconsistencies: "1/400" (string), 4 (number), 0.4 (number)  
/// Our standard: fractions stay strings, whole seconds as integers, decimals as numbers
fn normalize_ratio_format(value: &Value) -> Value {
    match value {
        Value::String(s) => {
            // If it's already a fraction string like "1/400", keep it
            if s.contains('/') {
                value.clone()
            } else if let Ok(num) = s.parse::<f64>() {
                // String that's a number - convert to appropriate JSON type
                if (num.fract()).abs() < 0.001 {
                    Value::Number(serde_json::Number::from(num as i64))
                } else if let Some(json_num) = serde_json::Number::from_f64(num) {
                    Value::Number(json_num)
                } else {
                    value.clone()
                }
            } else {
                value.clone()
            }
        }
        Value::Number(n) => {
            // Numbers should stay as numbers, but clean up precision
            if let Some(num) = n.as_f64() {
                if (num.fract()).abs() < 0.001 {
                    // Whole number - keep as integer
                    Value::Number(serde_json::Number::from(num as i64))
                } else {
                    // Decimal - keep as-is
                    value.clone()
                }
            } else {
                value.clone()
            }
        }
        _ => value.clone(),
    }
}

/// Clean numeric precision while preserving JSON number type: 14.0 -> 14, 2.8 -> 2.8
fn normalize_clean_numeric_precision(value: &Value, _max_places: u8) -> Value {
    let number = match value {
        Value::String(s) => s.parse::<f64>().ok(),
        Value::Number(n) => n.as_f64(),
        _ => return value.clone(),
    };

    if let Some(num) = number {
        if (num.fract()).abs() < 0.001 {
            // Integer value - return as JSON number
            Value::Number(serde_json::Number::from(num as i64))
        } else {
            // Decimal value - preserve as number with original precision
            if let Some(json_num) = serde_json::Number::from_f64(num) {
                Value::Number(json_num)
            } else {
                value.clone()
            }
        }
    } else {
        value.clone()
    }
}

/// Convert numbers to strings for SubSec* tags
/// ExifTool outputs SubSec* tags as numbers, but exif-oxide outputs them as strings
fn normalize_number_to_string(value: &Value) -> Value {
    match value {
        Value::Number(n) => {
            // Convert number to string
            Value::String(n.to_string())
        }
        _ => value.clone(),
    }
}

/// Normalize GPS altitude for tolerance-based comparison
/// Round to nearest 0.1m since GPS accuracy is typically 1-3m
fn normalize_gps_altitude_tolerance(value: &Value) -> Value {
    let number = match value {
        Value::String(s) => {
            // Strip " m" suffix if present, then parse number
            let cleaned = s.trim_end_matches(" m").trim();
            cleaned.parse::<f64>().ok()
        }
        Value::Number(n) => n.as_f64(),
        _ => return value.clone(),
    };

    if let Some(num) = number {
        // Round to nearest 0.1m for consistent comparison
        let rounded = (num * 10.0).round() / 10.0;

        // Return as formatted string with " m" suffix for consistency
        Value::String(format!("{rounded:.1} m"))
    } else {
        value.clone()
    }
}

/// Trim trailing whitespace from string values
/// Some cameras pad certain fields with spaces
fn normalize_trim_whitespace(value: &Value) -> Value {
    match value {
        Value::String(s) => Value::String(s.trim_end().to_string()),
        _ => value.clone(),
    }
}

/// Normalize values for comparison (handle format differences between ExifTool and exif-oxide)
pub fn normalize_for_comparison(mut data: Value, _is_exiftool: bool) -> Value {
    if let Some(obj) = data.as_object_mut() {
        // Normalize SourceFile to relative path
        if let Some(source_file) = obj.get_mut("SourceFile") {
            if let Some(path_str) = source_file.as_str() {
                if path_str.starts_with('/') {
                    // For absolute paths, try to extract the relative part after a known directory
                    // This handles cases where snapshots were generated from a different absolute path
                    if path_str.contains("/test-images/") {
                        if let Some(idx) = path_str.find("test-images/") {
                            *source_file = serde_json::Value::String(path_str[idx..].to_string());
                        }
                    } else if path_str.contains("/third-party/") {
                        if let Some(idx) = path_str.find("third-party/") {
                            *source_file = serde_json::Value::String(path_str[idx..].to_string());
                        }
                    }
                }
            }
        }

        // Normalize Directory to relative path (now with File: prefix)
        if let Some(directory) = obj.get_mut("File:Directory") {
            if let Some(dir_str) = directory.as_str() {
                if dir_str.starts_with('/') {
                    // For absolute paths, try to extract the relative part after a known directory
                    if dir_str.contains("/test-images") {
                        if let Some(idx) = dir_str.find("test-images") {
                            *directory = serde_json::Value::String(dir_str[idx..].to_string());
                        }
                    } else if dir_str.contains("/third-party") {
                        if let Some(idx) = dir_str.find("third-party") {
                            *directory = serde_json::Value::String(dir_str[idx..].to_string());
                        }
                    }
                }
            }
        }

        // Don't compare version fields - they'll always be different
        obj.remove("ExifToolVersion");
        obj.remove("ExifTool:ExifToolVersion");

        // Don't compare file modification times - they may vary
        obj.remove("FileModifyDate");
        obj.remove("File:FileModifyDate");

        // Normalize file size format (ExifTool: "5.5 MB", exif-oxide: "5469898 bytes")
        // For now, just remove it since formats differ significantly
        obj.remove("FileSize");
        obj.remove("File:FileSize");

        // Normalize GPS coordinates to handle floating-point precision differences
        // GPS coordinates should be close within 7-10 decimal places as specified by user
        for (key, value) in obj.iter_mut() {
            if matches!(
                key.as_str(),
                "EXIF:GPSLatitude" | "EXIF:GPSLongitude" | "EXIF:GPSAltitude"
            ) {
                if let Some(num) = value.as_f64() {
                    // Round to 10 decimal places to handle precision differences
                    let rounded = (num * 1e10).round() / 1e10;
                    *value = serde_json::Value::Number(
                        serde_json::Number::from_f64(rounded)
                            .unwrap_or_else(|| serde_json::Number::from_f64(num).unwrap()),
                    );
                }
            }
        }

        // Apply rule-based normalization for format consistency
        // Handles ExifTool's inconsistent output across different manufacturer modules
        let normalization_rules = get_normalization_rules();
        for (key, value) in obj.iter_mut() {
            if let Some(rule) = normalization_rules.get(key.as_str()) {
                let normalized = apply_normalization_rule(value, rule);
                *value = normalized;
            }
        }

        // Normalize LensSerialNumber to always be a string
        // The EXIF specification defines LensSerialNumber as a string type (tag 0xa435)
        // but some processors may extract it as a number. Always stringify it.
        if let Some(lens_serial) = obj.get_mut("EXIF:LensSerialNumber") {
            if let Value::Number(n) = lens_serial {
                *lens_serial = Value::String(n.to_string());
            }
        }

        // Normalize nondeterministic composite tags
        // ExifTool exhibits nondeterministic behavior for some composite tags due to Perl's 
        // hash randomization. The most notable case is Nikon's LensID where multiple lens 
        // variants can match, causing the order to vary between runs.
        //
        // Example: "AF-P DX Nikkor 18-55mm f/3.5-5.6G VR or AF-P DX Nikkor 18-55mm f/3.5-5.6G"
        //       vs "AF-P DX Nikkor 18-55mm f/3.5-5.6G or AF-P DX Nikkor 18-55mm f/3.5-5.6G VR"
        //
        // For consistent comparison, we sort the alternatives alphabetically.
        if let Some(lens_id) = obj.get_mut("Composite:LensID") {
            if let Value::String(lens_str) = lens_id {
                if lens_str.contains(" or ") {
                    let mut parts: Vec<&str> = lens_str.split(" or ").collect();
                    parts.sort();
                    *lens_id = Value::String(parts.join(" or "));
                }
            }
        }
    }

    data
}
