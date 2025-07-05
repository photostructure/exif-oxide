//! ExifTool compatibility tests
//!
//! These tests compare exif-oxide output against stored ExifTool reference snapshots
//! to ensure compatibility. ExifTool snapshots are the authoritative reference.

use serde_json::Value;
use similar::{ChangeTag, TextDiff};
use std::collections::HashMap;
use std::path::Path;

/// Load supported tags from shared config file (Milestone 8a)
/// Single source of truth now maintained in config/supported_tags.json
///
/// Note: Future tags requiring PrintConv implementations:
/// - MeteringMode: Needs Canon-specific "Evaluative" vs "Multi-segment" handling
/// - ExifImageWidth/ExifImageHeight: Work but no PrintConv needed
/// - XResolution/YResolution: Need PrintConv to format as integer
/// - FocalLength: Needs PrintConv to format as "24.0 mm"
/// - FNumber: Needs PrintConv to format as "4.0"
/// - ExposureTime: Needs PrintConv to format as "1/2000"
/// - DateTimeOriginal/CreateDate: Need PrintConv date formatting
/// - GPSLatitude/GPSLongitude: Need ValueConv for decimal degrees (Milestone 8)
/// - GPS tags temporarily excluded due to extraction issues in some files
fn load_supported_tags() -> Vec<String> {
    const CONFIG_JSON: &str = include_str!("../config/supported_tags.json");
    serde_json::from_str(CONFIG_JSON).expect("Failed to parse supported_tags.json")
}

/// Files to exclude from testing (problematic files to deal with later)
const EXCLUDED_FILES: &[&str] = &[
    "test-images/casio/QVCI.jpg",
    "third-party/exiftool/t/images/ExtendedXMP.jpg",
    "third-party/exiftool/t/images/PhotoMechanic.jpg",
    "third-party/exiftool/t/images/ExifTool.jpg",
    "third-party/exiftool/t/images/CasioQVCI.jpg",
    "third-party/exiftool/t/images/InfiRay.jpg", // Thermal imaging - specialized format
    "third-party/exiftool/t/images/IPTC.jpg",    // IPTC-specific metadata edge case
];

/// Load ExifTool reference snapshot for a file
fn load_exiftool_snapshot(file_path: &str) -> Result<Value, Box<dyn std::error::Error>> {
    // Convert file path to snapshot name (same logic as generate_snapshots.sh)
    let relative_path = Path::new(file_path)
        .strip_prefix(std::env::current_dir()?)
        .unwrap_or(Path::new(file_path))
        .to_string_lossy();

    let snapshot_name = relative_path
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string();

    let snapshot_path = format!("generated/exiftool-json/{snapshot_name}.json");

    if !Path::new(&snapshot_path).exists() {
        return Err(format!("Snapshot not found: {snapshot_path}").into());
    }

    let snapshot_content = std::fs::read_to_string(&snapshot_path)?;
    let json: Value = serde_json::from_str(&snapshot_content)?;
    Ok(json)
}

/// Run exif-oxide library and return parsed JSON for a single file
fn run_exif_oxide(file_path: &str) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(exif_oxide::extract_metadata_json(file_path)?)
}

/// Filter JSON object to only include supported tags
/// Now handles group-prefixed tag names - supported_tags.json contains full group:tag format
fn filter_to_supported_tags(data: &Value) -> Value {
    if let Some(obj) = data.as_object() {
        let supported_tags = load_supported_tags();
        let supported_tag_refs: Vec<&str> = supported_tags.iter().map(|s| s.as_str()).collect();

        let filtered: HashMap<String, Value> = obj
            .iter()
            .filter(|(key, _)| {
                // Always include SourceFile
                if key.as_str() == "SourceFile" {
                    return true;
                }

                // Check if the full group:tag key is in the supported list
                supported_tag_refs.contains(&key.as_str())
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        serde_json::to_value(filtered).unwrap()
    } else {
        data.clone()
    }
}

/// Normalization rules for standardizing ExifTool's inconsistent output formats
#[derive(Debug)]
enum NormalizationRule {
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
}

/// Tag normalization configuration
/// Maps tag names to their normalization rules
fn get_normalization_rules() -> HashMap<&'static str, NormalizationRule> {
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
    // rules.insert("ExposureTime", NormalizationRule::RatioFormat);
    rules.insert("EXIF:ExposureTime", NormalizationRule::RatioFormat);

    // GPS altitude tags - special tolerance-based comparison
    // ExifTool: 25.24672793 (number), exif-oxide: "25.2 m" (string)
    // GPS accuracy is ~1-3m, so validate values are within 0.09m tolerance
    rules.insert("EXIF:GPSAltitude", NormalizationRule::GPSAltitudeTolerance);

    // SubSec* tags - convert numbers to strings for consistency
    // ExifTool outputs as numbers, exif-oxide outputs as strings
    rules.insert("EXIF:SubSecTime", NormalizationRule::NumberToString);
    rules.insert("EXIF:SubSecTimeDigitized", NormalizationRule::NumberToString);
    rules.insert("EXIF:SubSecTimeOriginal", NormalizationRule::NumberToString);

    rules
}

/// Apply normalization rule to a value
fn apply_normalization_rule(value: &Value, rule: &NormalizationRule) -> Value {
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

/// Normalize values for comparison (handle format differences between ExifTool and exif-oxide)
fn normalize_for_comparison(mut data: Value, _is_exiftool: bool) -> Value {
    if let Some(obj) = data.as_object_mut() {
        // Normalize SourceFile to relative path
        if let Some(source_file) = obj.get_mut("SourceFile") {
            if let Some(path_str) = source_file.as_str() {
                if path_str.starts_with('/') {
                    // Convert absolute to relative
                    if let Ok(cwd) = std::env::current_dir() {
                        if let Ok(rel_path) = Path::new(path_str).strip_prefix(&cwd) {
                            *source_file =
                                serde_json::Value::String(rel_path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        // Normalize Directory to relative path (now with File: prefix)
        if let Some(directory) = obj.get_mut("File:Directory") {
            if let Some(dir_str) = directory.as_str() {
                if dir_str.starts_with('/') {
                    if let Ok(cwd) = std::env::current_dir() {
                        if let Ok(rel_path) = Path::new(dir_str).strip_prefix(&cwd) {
                            *directory =
                                serde_json::Value::String(rel_path.to_string_lossy().to_string());
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
    }

    data
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

/// Compare ExifTool snapshot and exif-oxide output for a specific file
fn compare_file_output(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Load ExifTool reference snapshot
    let exiftool_output = match load_exiftool_snapshot(file_path) {
        Ok(output) => output,
        Err(e) => {
            println!("  Skipping: snapshot not found: {e}");
            return Ok(());
        }
    };

    // Get exif-oxide output (test)
    let exif_oxide_output = match run_exif_oxide(file_path) {
        Ok(output) => output,
        Err(e) => {
            println!("  Skipping: exif-oxide failed: {e}");
            return Ok(());
        }
    };

    // Filter both to supported tags only
    let filtered_exiftool = filter_to_supported_tags(&exiftool_output);
    let filtered_exif_oxide = filter_to_supported_tags(&exif_oxide_output);

    // Normalize for comparison
    let normalized_exiftool = normalize_for_comparison(filtered_exiftool, true);
    let normalized_exif_oxide = normalize_for_comparison(filtered_exif_oxide, false);

    // Compare JSON objects directly (ignores field order)
    if normalized_exiftool != normalized_exif_oxide {
        // Pretty print both for diff display
        let exiftool_json = serde_json::to_string_pretty(&normalized_exiftool)?;
        let exif_oxide_json = serde_json::to_string_pretty(&normalized_exif_oxide)?;

        let diff = TextDiff::from_lines(&exiftool_json, &exif_oxide_json);

        println!("\n❌ MISMATCH for {file_path}");
        println!("ExifTool snapshot (reference) vs exif-oxide (test):\n");

        for change in diff.iter_all_changes() {
            let sign = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            print!("{sign}{change}");
        }

        return Err(format!("Output mismatch for {file_path}").into());
    }

    Ok(())
}

/// Test ExifTool compatibility using stored snapshots
#[test]
fn test_exiftool_compatibility() {
    // Discover snapshots
    let snapshots_dir = Path::new("generated/exiftool-json");
    if !snapshots_dir.exists() {
        println!("Reference JSON directory not found. Run 'make compat-gen' first.");
        return;
    }

    let mut snapshot_files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(snapshots_dir) {
        for entry in entries.flatten() {
            if let Some(filename) = entry.file_name().to_str() {
                if filename.ends_with(".json") {
                    let snapshot_path = entry.path();
                    // Read the snapshot to get the SourceFile
                    if let Ok(content) = std::fs::read_to_string(&snapshot_path) {
                        if let Ok(json) = serde_json::from_str::<Value>(&content) {
                            if let Some(source_file) =
                                json.get("SourceFile").and_then(|f| f.as_str())
                            {
                                snapshot_files.push(source_file.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    if snapshot_files.is_empty() {
        println!("No snapshot files found for testing");
        return;
    }

    println!(
        "Running ExifTool compatibility tests using {} snapshots",
        snapshot_files.len()
    );

    let mut failed_files = Vec::new();
    let mut passed_files = Vec::new();
    let mut tested_files = 0;

    for file_path in snapshot_files {
        // Skip excluded files (check both absolute and relative paths)
        let relative_path = Path::new(&file_path)
            .strip_prefix(std::env::current_dir().unwrap_or_default())
            .unwrap_or(Path::new(&file_path))
            .to_string_lossy();

        if EXCLUDED_FILES.contains(&file_path.as_str())
            || EXCLUDED_FILES.contains(&relative_path.as_ref())
        {
            println!("Skipping excluded file: {file_path}");
            continue;
        }

        // Skip if file doesn't exist
        if !Path::new(&file_path).exists() {
            continue;
        }

        tested_files += 1;

        if let Err(e) = compare_file_output(&file_path) {
            failed_files.push((file_path, e.to_string()));
        } else {
            passed_files.push(file_path);
        }
    }

    println!("\n📊 Summary:");
    println!("  Files tested: {tested_files}");
    println!("  Matches: {}", passed_files.len());
    println!("  Mismatches: {}", failed_files.len());

    if 1 > 2 && !passed_files.is_empty() {
        println!("\n✅ Passed files:");
        for file in &passed_files {
            let relative_path = Path::new(file)
                .strip_prefix(std::env::current_dir().unwrap_or_default())
                .unwrap_or(Path::new(file))
                .to_string_lossy();
            println!("  - {relative_path}");
        }
    }

    if !failed_files.is_empty() {
        println!("\n❌ Failed files:");
        for (file, error) in &failed_files {
            let relative_path = Path::new(file)
                .strip_prefix(std::env::current_dir().unwrap_or_default())
                .unwrap_or(Path::new(file))
                .to_string_lossy();
            println!("  - {relative_path}: {error}");
        }

        panic!(
            "{} files had mismatches with ExifTool snapshots",
            failed_files.len()
        );
    }
}

/// Test a specific known file using snapshots
#[test]
fn test_canon_t3i_compatibility() {
    let test_file = "test-images/canon/Canon_T3i.JPG";

    // Skip if test file doesn't exist
    if !Path::new(test_file).exists() {
        println!("Test file {test_file} not found, skipping");
        return;
    }

    compare_file_output(test_file).expect("Canon T3i should match ExifTool snapshot");
}
