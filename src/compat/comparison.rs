//! Value comparison logic for ExifTool compatibility
//!
//! This module provides sophisticated comparison functions that handle the various
//! format differences between ExifTool and exif-oxide output.

use serde_json::Value;

/// Check if values match semantically (using existing normalization)
pub fn values_match_semantically(expected: &Value, actual: &Value) -> bool {
    expected == actual
}

/// Check if values match within acceptable tolerance for PhotoStructure DAM
pub fn values_match_with_tolerance(tag_name: &str, expected: &Value, actual: &Value) -> bool {
    // Handle GPS coordinate precision differences specifically
    if tag_name.contains("GPS") && (tag_name.contains("Latitude") || tag_name.contains("Longitude"))
    {
        if let (Value::Number(exp_num), Value::Number(act_num)) = (expected, actual) {
            let diff = (exp_num.as_f64().unwrap_or(0.0) - act_num.as_f64().unwrap_or(0.0)).abs();
            return diff < 0.0001; // GPS coordinate tolerance (consumer GPS precision)
        }
    }

    // Handle other numeric tolerance cases
    if let (Value::Number(exp_num), Value::Number(act_num)) = (expected, actual) {
        let diff = (exp_num.as_f64().unwrap_or(0.0) - act_num.as_f64().unwrap_or(0.0)).abs();
        return diff < 0.001; // General numeric tolerance
    }

    false
}

/// Check if two values represent the same data but in different formats (with tag context)
/// Enhanced for PhotoStructure DAM workflows with appropriate tolerances
pub fn same_data_different_format_with_tag(
    _tag_name: &str,
    expected: &Value,
    actual: &Value,
) -> bool {
    // Fall back to generic handling since tolerance is now handled separately
    same_data_different_format(expected, actual)
}

/// Check if two values represent the same data but in different formats
/// Enhanced for PhotoStructure DAM workflows with appropriate tolerances
pub fn same_data_different_format(expected: &Value, actual: &Value) -> bool {
    match (expected, actual) {
        // String vs Number with same numeric value
        (Value::String(s), Value::Number(n)) => {
            if let Some(tolerance) = get_tolerance_for_values(expected, actual) {
                s.parse::<f64>()
                    .map(|parsed| (parsed - n.as_f64().unwrap_or(0.0)).abs() < tolerance)
                    .unwrap_or(false)
            } else {
                false
            }
        }
        (Value::Number(n), Value::String(s)) => {
            if let Some(tolerance) = get_tolerance_for_values(expected, actual) {
                s.parse::<f64>()
                    .map(|parsed| (parsed - n.as_f64().unwrap_or(0.0)).abs() < tolerance)
                    .unwrap_or(false)
            } else {
                false
            }
        }
        // Rational array vs formatted string (e.g., [500, 10] vs "50.0 mm")
        (Value::Array(arr), Value::String(s)) if arr.len() == 2 => {
            if let (Some(num), Some(den)) = (arr[0].as_f64(), arr[1].as_f64()) {
                if den != 0.0 {
                    let ratio = num / den;
                    if let Some(numeric_part) = extract_numeric_from_string(s) {
                        let tolerance = get_tolerance_for_values(expected, actual).unwrap_or(0.001);
                        return (ratio - numeric_part).abs() < tolerance;
                    }
                }
            }
            false
        }
        (Value::String(s), Value::Array(arr)) if arr.len() == 2 => {
            if let (Some(num), Some(den)) = (arr[0].as_f64(), arr[1].as_f64()) {
                if den != 0.0 {
                    let ratio = num / den;
                    if let Some(numeric_part) = extract_numeric_from_string(s) {
                        let tolerance = get_tolerance_for_values(expected, actual).unwrap_or(0.001);
                        return (ratio - numeric_part).abs() < tolerance;
                    }
                }
            }
            false
        }
        // Array vs single value (like rational array [5, 1] vs number 5)
        (Value::Array(arr), Value::Number(n)) if arr.len() == 2 => {
            if let (Some(num), Some(den)) = (arr[0].as_f64(), arr[1].as_f64()) {
                if den != 0.0 {
                    let tolerance = get_tolerance_for_values(expected, actual).unwrap_or(0.001);
                    return (num / den - n.as_f64().unwrap_or(0.0)).abs() < tolerance;
                }
            }
            false
        }
        (Value::Number(n), Value::Array(arr)) if arr.len() == 2 => {
            if let (Some(num), Some(den)) = (arr[0].as_f64(), arr[1].as_f64()) {
                if den != 0.0 {
                    let tolerance = get_tolerance_for_values(expected, actual).unwrap_or(0.001);
                    return (num / den - n.as_f64().unwrap_or(0.0)).abs() < tolerance;
                }
            }
            false
        }
        _ => false,
    }
}

/// Get appropriate tolerance based on value types and PhotoStructure DAM requirements
fn get_tolerance_for_values(expected: &Value, actual: &Value) -> Option<f64> {
    // Check for GPS coordinates - need 0.0001° tolerance (consumer GPS precision)
    if is_gps_coordinate(expected) || is_gps_coordinate(actual) {
        return Some(0.0001);
    }

    // Check for timestamp sub-second precision - important for burst photos
    if is_timestamp_value(expected) || is_timestamp_value(actual) {
        return Some(0.001); // 1ms tolerance for timestamp precision
    }

    // Default tolerance for other numeric comparisons
    Some(0.001)
}

/// Check if a value represents GPS coordinates
fn is_gps_coordinate(value: &Value) -> bool {
    match value {
        Value::Number(n) => {
            let abs_val = n.as_f64().unwrap_or(0.0).abs();
            // GPS coordinates are typically -180 to 180 for longitude, -90 to 90 for latitude
            abs_val <= 180.0 && abs_val > 0.0001
        }
        _ => false,
    }
}

/// Check if a value represents timestamp data
fn is_timestamp_value(value: &Value) -> bool {
    match value {
        Value::String(s) => {
            // Check for timestamp formats like "14:58:24" or ISO datetime
            s.contains(':') && (s.len() >= 8)
        }
        Value::Number(n) => {
            // Unix timestamps or sub-second values
            let val = n.as_f64().unwrap_or(0.0);
            val > 1000000000.0 || (val > 0.0 && val < 86400.0) // Unix epoch or seconds in day
        }
        _ => false,
    }
}

/// Extract numeric value from strings like "50.0 mm", "1/2000", "F4.0"
fn extract_numeric_from_string(s: &str) -> Option<f64> {
    // Handle fraction format "1/2000"
    if let Some(slash_pos) = s.find('/') {
        let num_str = &s[..slash_pos];
        let den_str = &s[slash_pos + 1..];
        if let (Ok(num), Ok(den)) = (num_str.parse::<f64>(), den_str.parse::<f64>()) {
            if den != 0.0 {
                return Some(num / den);
            }
        }
    }

    // Handle strings with units "50.0 mm", "F4.0", or prefixes
    let cleaned = s
        .chars()
        .skip_while(|c| !c.is_ascii_digit() && *c != '-' && *c != '.')
        .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .collect::<String>();

    cleaned.parse::<f64>().ok()
}
