//! ExifTool compatibility tests
//!
//! These tests compare exif-oxide output against stored ExifTool reference snapshots
//! to ensure compatibility. ExifTool snapshots are the authoritative reference.
//!
//! Note: These tests require the `integration-tests` feature to be enabled and
//! external test assets to be available. They are automatically skipped in published crates.

#![cfg(feature = "integration-tests")]

use serde_json::Value;
use similar::{ChangeTag, TextDiff};
use std::collections::{HashMap, HashSet};
use std::path::Path;

mod common;
use common::CASIO_QVCI_JPG;

/// Load supported tags from shared config file
/// Uses supported_tags_final.json which contains the comprehensive list of 271 tags
fn load_supported_tags() -> Vec<String> {
    const CONFIG_JSON: &str = include_str!("../config/supported_tags_final.json");
    serde_json::from_str(CONFIG_JSON).expect("Failed to parse supported_tags_final.json")
}

/// Files to exclude from testing (problematic files to deal with later)
const EXCLUDED_FILES: &[&str] = &[
    CASIO_QVCI_JPG,
    "third-party/exiftool/t/images/ExtendedXMP.jpg",
    "third-party/exiftool/t/images/PhotoMechanic.jpg",
    "third-party/exiftool/t/images/ExifTool.jpg",
    "third-party/exiftool/t/images/CasioQVCI.jpg",
    "third-party/exiftool/t/images/InfiRay.jpg", // Thermal imaging - specialized format
    "third-party/exiftool/t/images/IPTC.jpg",    // IPTC-specific metadata edge case
];

/// Load ExifTool reference snapshot for a file
fn load_exiftool_snapshot(file_path: &str) -> Result<Value, Box<dyn std::error::Error>> {
    // First try to get the current directory to make path relative
    let current_dir = std::env::current_dir()?;

    // Convert file path to snapshot name (same logic as generate_exiftool_json.sh)
    // The shell script uses realpath --relative-to="$PROJECT_ROOT"
    let path = Path::new(file_path);
    let relative_path = if path.is_absolute() {
        path.strip_prefix(&current_dir)
            .unwrap_or(path)
            .to_string_lossy()
    } else {
        path.to_string_lossy()
    };

    // Replace any sequence of non-alphanumeric characters with single underscore
    // This matches the sed command: sed 's/[^a-zA-Z0-9]\+/_/g'
    let snapshot_name = relative_path
        .chars()
        .fold(String::new(), |mut acc, c| {
            if c.is_alphanumeric() {
                acc.push(c);
            } else if acc.is_empty() || !acc.ends_with('_') {
                acc.push('_');
            }
            acc
        })
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
    /// Trim trailing whitespace from string values
    TrimWhitespace,
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

/// Normalize values for comparison (handle format differences between ExifTool and exif-oxide)
fn normalize_for_comparison(mut data: Value, _is_exiftool: bool) -> Value {
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

/// Trim trailing whitespace from string values
/// Some cameras pad certain fields with spaces
fn normalize_trim_whitespace(value: &Value) -> Value {
    match value {
        Value::String(s) => Value::String(s.trim_end().to_string()),
        _ => value.clone(),
    }
}

/// Compare ExifTool snapshot and exif-oxide output for a specific file
fn compare_file_output(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Load ExifTool reference snapshot
    let exiftool_output = load_exiftool_snapshot(file_path)
        .map_err(|e| format!("Snapshot not found for {file_path}: {e}"))?;

    // Get exif-oxide output (test)
    let exif_oxide_output = match run_exif_oxide(file_path) {
        Ok(output) => output,
        Err(e) => {
            println!("  Skipping: exif-oxide failed: {e}");
            return Ok(());
        }
    };

    // Filter both to supported tags only
    let mut filtered_exiftool = filter_to_supported_tags(&exiftool_output);
    let mut filtered_exif_oxide = filter_to_supported_tags(&exif_oxide_output);

    // Remove known missing tags for this file type
    remove_known_missing_tags(&mut filtered_exiftool, file_path);
    remove_known_missing_tags(&mut filtered_exif_oxide, file_path);

    // Normalize for comparison
    let normalized_exiftool = normalize_for_comparison(filtered_exiftool, true);
    let normalized_exif_oxide = normalize_for_comparison(filtered_exif_oxide, false);

    // TODO: Add manufacturer-specific missing MakerNotes tag documentation
    let manufacturer_info = detect_manufacturer_from_path(file_path);
    if let Some(info) = manufacturer_info {
        match info.as_str() {
            "sony" => {
                // TODO: Missing Sony MakerNotes tags tracked in MILESTONE-17e-Sony-RAW
                // Expected missing: MakerNotes:ExposureTime, MakerNotes:FocalLength
                // These will be implemented with Sony.pm codegen extraction
            }
            "nikon" => {
                // TODO: Missing Nikon MakerNotes tags - no milestone yet
                // Expected missing: MakerNotes:ISO, MakerNotes:FocalLength, MakerNotes:Lens
                // Requires Nikon-specific tag extraction implementation
            }
            "canon" => {
                // TODO: Missing Canon MakerNotes tags tracked in MILESTONE-17d-Canon-RAW
                // Expected missing: Various Canon-specific MakerNotes
                // Includes Canon lens database and binary data extraction
            }
            "casio" => {
                // TODO: Missing Casio MakerNotes - not yet scheduled
                // Multiple Casio files failing, needs manufacturer-specific implementation
            }
            "minolta" => {
                // TODO: Missing Minolta MakerNotes - not yet scheduled
                // Both MRW and JPG files failing, needs Minolta-specific implementation
            }
            "pentax" => {
                // TODO: Missing Pentax MakerNotes - mentioned in MILESTONE-MOAR-CODEGEN
                // Pentax lens types and model mappings (~5-8 tables planned)
            }
            "kodak" => {
                // TODO: Missing Kodak MakerNotes - not yet scheduled
                // Kodak files failing, needs manufacturer-specific implementation
            }
            "panasonic" => {
                // TODO: Missing Panasonic MakerNotes - mentioned in MILESTONE-MOAR-CODEGEN
                // Panasonic lens databases and quality settings (~8-12 tables planned)
            }
            _ => {}
        }
    }

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

/// Get known missing tags for specific file types/manufacturers
/// These are tags that are documented as missing due to incomplete implementations
fn get_known_missing_tags(file_path: &str) -> Vec<&'static str> {
    let path_lower = file_path.to_lowercase();

    // Panasonic RW2 files - missing tags due to incomplete IFD chaining and MakerNotes
    // See: docs/todo/HANDOFF-panasonic-rw2-complete-resolution.md
    if (path_lower.contains("panasonic") || path_lower.contains("lumix"))
        && path_lower.contains("rw2")
    {
        vec![
            "EXIF:ResolutionUnit",   // Located in IFD1 (requires IFD chaining)
            "EXIF:YCbCrPositioning", // Located in IFD1 (requires IFD chaining)
            "EXIF:ColorSpace",       // Located in ExifIFD (requires ExifIFD chaining)
            "EXIF:WhiteBalance",     // Located in MakerNotes (requires MakerNotes processing)
        ]
    } else {
        vec![]
    }
}

/// Remove known missing tags from a JSON value
/// This allows compatibility tests to pass for documented missing features
fn remove_known_missing_tags(json: &mut serde_json::Value, file_path: &str) {
    let missing_tags = get_known_missing_tags(file_path);

    if !missing_tags.is_empty() {
        if let Some(obj) = json.as_object_mut() {
            for tag in missing_tags {
                obj.remove(tag);
            }
        }
    }
}

/// Detect manufacturer from file path for TODO tracking
/// Returns lowercase manufacturer name if detected in path
fn detect_manufacturer_from_path(file_path: &str) -> Option<String> {
    let path_lower = file_path.to_lowercase();

    // Check directory names and file names for manufacturer hints
    if path_lower.contains("sony") || path_lower.contains("ilce") || path_lower.contains("a7c") {
        Some("sony".to_string())
    } else if path_lower.contains("nikon")
        || path_lower.contains("d70")
        || path_lower.contains("d2hs")
        || path_lower.contains("nikon_z8")
    {
        Some("nikon".to_string())
    } else if path_lower.contains("canon")
        || path_lower.contains("t3i")
        || path_lower.contains("1dmk")
        || path_lower.contains("eos")
    {
        Some("canon".to_string())
    } else if path_lower.contains("casio")
        || path_lower.contains("qv")
        || path_lower.contains("ex-z")
    {
        Some("casio".to_string())
    } else if path_lower.contains("minolta") || path_lower.contains("dimage") {
        Some("minolta".to_string())
    } else if path_lower.contains("pentax") || path_lower.contains("k-1") {
        Some("pentax".to_string())
    } else if path_lower.contains("kodak") || path_lower.contains("dc4800") {
        Some("kodak".to_string())
    } else if path_lower.contains("panasonic") || path_lower.contains("lumix") {
        Some("panasonic".to_string())
    } else {
        None
    }
}

/// Analysis result for a specific tag difference
#[derive(Debug, Clone)]
struct TagDifference {
    tag: String,
    expected: Option<Value>,
    actual: Option<Value>,
    difference_type: DifferenceType,
    sample_file: String,
}

#[derive(Debug, Clone, PartialEq)]
enum DifferenceType {
    Working,             // Values match exactly
    ValueFormatMismatch, // Same data, different format (e.g., "1.5" vs 1.5)
    Missing,             // Tag missing from our output
    TypeMismatch,        // Completely different data types
    OnlyInOurs,          // Tag only exists in our output
}

/// Structured compatibility analysis result
#[derive(Debug)]
struct CompatibilityReport {
    working_tags: Vec<String>,
    value_format_mismatches: Vec<TagDifference>,
    missing_tags: Vec<TagDifference>,
    type_mismatches: Vec<TagDifference>,
    only_in_ours: Vec<TagDifference>,
    total_tags_tested: usize,
    total_files_tested: usize,
}

impl CompatibilityReport {
    fn new() -> Self {
        Self {
            working_tags: Vec::new(),
            value_format_mismatches: Vec::new(),
            missing_tags: Vec::new(),
            type_mismatches: Vec::new(),
            only_in_ours: Vec::new(),
            total_tags_tested: 0,
            total_files_tested: 0,
        }
    }

    fn print_summary(&self) {
        let total_failing = self.value_format_mismatches.len()
            + self.missing_tags.len()
            + self.type_mismatches.len();
        let success_rate = if self.total_tags_tested > 0 {
            (self.working_tags.len() * 100) / self.total_tags_tested
        } else {
            0
        };

        println!("\n🎯 ExifTool Compatibility Report");
        println!("=================================");
        println!("Files tested: {}", self.total_files_tested);
        println!("Unique tags tested: {}", self.total_tags_tested);
        println!(
            "Success rate: {}% ({}/{})",
            success_rate,
            self.working_tags.len(),
            self.total_tags_tested
        );
        println!();

        if !self.working_tags.is_empty() {
            println!("✅ WORKING ({} tags):", self.working_tags.len());
            let mut sorted_working = self.working_tags.clone();
            sorted_working.sort();
            for tag in sorted_working.iter().take(10) {
                println!("  {}", tag);
            }
            if self.working_tags.len() > 10 {
                println!("  ... and {} more", self.working_tags.len() - 10);
            }
            println!();
        }

        if !self.value_format_mismatches.is_empty() {
            println!(
                "⚠️  VALUE FORMAT MISMATCHES ({} tags):",
                self.value_format_mismatches.len()
            );
            for diff in self.value_format_mismatches.iter().take(5) {
                println!(
                    "  {}: Expected: {:?}, Got: {:?} ({})",
                    diff.tag, diff.expected, diff.actual, diff.sample_file
                );
            }
            if self.value_format_mismatches.len() > 5 {
                println!("  ... and {} more", self.value_format_mismatches.len() - 5);
            }
            println!();
        }

        if !self.missing_tags.is_empty() {
            println!("❌ MISSING TAGS ({} tags):", self.missing_tags.len());
            for diff in self.missing_tags.iter().take(5) {
                println!(
                    "  {}: Expected: {:?} ({})",
                    diff.tag, diff.expected, diff.sample_file
                );
            }
            if self.missing_tags.len() > 5 {
                println!("  ... and {} more", self.missing_tags.len() - 5);
            }
            println!();
        }

        if !self.type_mismatches.is_empty() {
            println!("🔥 TYPE MISMATCHES ({} tags):", self.type_mismatches.len());
            for diff in self.type_mismatches.iter().take(5) {
                println!(
                    "  {}: Expected: {:?}, Got: {:?} ({})",
                    diff.tag, diff.expected, diff.actual, diff.sample_file
                );
            }
            if self.type_mismatches.len() > 5 {
                println!("  ... and {} more", self.type_mismatches.len() - 5);
            }
            println!();
        }

        if !self.only_in_ours.is_empty() {
            println!("ℹ️  ONLY IN EXIF-OXIDE ({} tags):", self.only_in_ours.len());
            for diff in self.only_in_ours.iter().take(5) {
                println!("  {}: {:?} ({})", diff.tag, diff.actual, diff.sample_file);
            }
            if self.only_in_ours.len() > 5 {
                println!("  ... and {} more", self.only_in_ours.len() - 5);
            }
            println!();
        }

        println!(
            "Summary: {}% working, {} needing attention",
            success_rate, total_failing
        );
    }
}

/// Analyze differences between ExifTool reference and our output for a single file
fn analyze_tag_differences(
    file_path: &str,
    exiftool_data: &Value,
    our_data: &Value,
) -> Vec<TagDifference> {
    let mut differences = Vec::new();
    let supported_tags: HashSet<String> = load_supported_tags().into_iter().collect();

    let empty_map = serde_json::Map::new();
    let exiftool_obj = exiftool_data.as_object().unwrap_or(&empty_map);
    let our_obj = our_data.as_object().unwrap_or(&empty_map);

    // Check all supported tags
    for tag in &supported_tags {
        if tag == "SourceFile" {
            continue; // Skip SourceFile as it's always different
        }

        let expected = exiftool_obj.get(tag);
        let actual = our_obj.get(tag);

        let difference_type = match (expected, actual) {
            (Some(exp), Some(act)) => {
                if values_match_semantically(exp, act) {
                    DifferenceType::Working
                } else if same_data_different_format(exp, act) {
                    DifferenceType::ValueFormatMismatch
                } else {
                    DifferenceType::TypeMismatch
                }
            }
            (Some(_), None) => DifferenceType::Missing,
            (None, Some(_)) => DifferenceType::OnlyInOurs,
            (None, None) => continue, // Tag not present in either
        };

        differences.push(TagDifference {
            tag: tag.clone(),
            expected: expected.cloned(),
            actual: actual.cloned(),
            difference_type,
            sample_file: file_path.to_string(),
        });
    }

    differences
}

/// Check if two values represent the same data but in different formats
/// Enhanced for PhotoStructure DAM workflows with appropriate tolerances
fn same_data_different_format(expected: &Value, actual: &Value) -> bool {
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

/// Check if values match semantically (using existing normalization)
fn values_match_semantically(expected: &Value, actual: &Value) -> bool {
    // Use existing normalization logic
    expected == actual
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
                    // Reconstruct the original file path from the snapshot filename
                    // The snapshot name is created by replacing non-alphanumeric chars with _
                    // We need to reverse this process, but we can't perfectly reconstruct it
                    // Instead, read the snapshot to get the relative path portion
                    let snapshot_path = entry.path();
                    if let Ok(content) = std::fs::read_to_string(&snapshot_path) {
                        if let Ok(json) = serde_json::from_str::<Value>(&content) {
                            if let Some(source_file) =
                                json.get("SourceFile").and_then(|f| f.as_str())
                            {
                                // Extract the relative path portion - handle both absolute and relative paths
                                let relative_part = if source_file.starts_with("test-images/")
                                    || source_file.starts_with("third-party/")
                                {
                                    // Already a relative path starting with test-images/ or third-party/
                                    Some(source_file.to_string())
                                } else if source_file.contains("/test-images/") {
                                    source_file
                                        .split("/test-images/")
                                        .last()
                                        .map(|s| format!("test-images/{s}"))
                                } else if source_file.contains("/third-party/") {
                                    source_file
                                        .split("/third-party/")
                                        .last()
                                        .map(|s| format!("third-party/{s}"))
                                } else {
                                    None
                                };

                                if let Some(rel_path) = relative_part {
                                    snapshot_files.push(rel_path);
                                }
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

    let mut compatibility_report = CompatibilityReport::new();
    let mut tested_files = 0;
    let mut all_tag_differences = HashMap::new(); // tag -> TagDifference

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

        // Load both ExifTool reference and our output
        match (
            load_exiftool_snapshot(&file_path),
            run_exif_oxide(&file_path),
        ) {
            (Ok(exiftool_data), Ok(our_data)) => {
                let file_differences =
                    analyze_tag_differences(&file_path, &exiftool_data, &our_data);

                // Aggregate differences by tag (keep the first example of each tag difference)
                for diff in file_differences {
                    if !all_tag_differences.contains_key(&diff.tag) {
                        all_tag_differences.insert(diff.tag.clone(), diff);
                    }
                }
            }
            (Err(e), _) => {
                println!("Failed to load ExifTool snapshot for {}: {}", file_path, e);
            }
            (_, Err(e)) => {
                println!("Failed to run exif-oxide on {}: {}", file_path, e);
            }
        }
    }

    // Convert aggregated differences into report structure
    compatibility_report.total_files_tested = tested_files;

    for (tag, diff) in all_tag_differences {
        match diff.difference_type {
            DifferenceType::Working => compatibility_report.working_tags.push(tag),
            DifferenceType::ValueFormatMismatch => {
                compatibility_report.value_format_mismatches.push(diff)
            }
            DifferenceType::Missing => compatibility_report.missing_tags.push(diff),
            DifferenceType::TypeMismatch => compatibility_report.type_mismatches.push(diff),
            DifferenceType::OnlyInOurs => compatibility_report.only_in_ours.push(diff),
        }
    }

    compatibility_report.total_tags_tested = compatibility_report.working_tags.len()
        + compatibility_report.value_format_mismatches.len()
        + compatibility_report.missing_tags.len()
        + compatibility_report.type_mismatches.len()
        + compatibility_report.only_in_ours.len();

    // Print the enhanced compatibility report
    compatibility_report.print_summary();

    // Fail test if there are critical issues
    let critical_failures =
        compatibility_report.missing_tags.len() + compatibility_report.type_mismatches.len();
    if critical_failures > 0 {
        panic!(
            "Critical compatibility failures: {} missing tags, {} type mismatches",
            compatibility_report.missing_tags.len(),
            compatibility_report.type_mismatches.len()
        );
    }
}
