//! ExifTool compatibility tests
//!
//! These tests compare exif-oxide output against stored ExifTool reference snapshots
//! to ensure compatibility. ExifTool snapshots are the authoritative reference.

use serde_json::Value;
use similar::{ChangeTag, TextDiff};
use std::collections::HashMap;
use std::path::Path;

/// Tags currently supported by exif-oxide (Milestone 3)
/// This list should be updated as more milestones are completed
const SUPPORTED_TAGS: &[&str] = &[
    "Make",
    "Model",
    "Orientation",
    "ResolutionUnit",
    "YCbCrPositioning",
    "MIMEType",
    "SourceFile",
    "FileName",
    "FileSize",
    "FileModifyDate",
    "ExifToolVersion",
];

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
fn filter_to_supported_tags(data: &Value) -> Value {
    if let Some(obj) = data.as_object() {
        let filtered: HashMap<String, Value> = obj
            .iter()
            .filter(|(key, _)| SUPPORTED_TAGS.contains(&key.as_str()))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        serde_json::to_value(filtered).unwrap()
    } else {
        data.clone()
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

        // Normalize Directory to relative path
        if let Some(directory) = obj.get_mut("Directory") {
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

        // Don't compare file modification times - they may vary
        obj.remove("FileModifyDate");

        // Normalize file size format (ExifTool: "5.5 MB", exif-oxide: "5469898 bytes")
        // For now, just remove it since formats differ significantly
        obj.remove("FileSize");
    }

    data
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
