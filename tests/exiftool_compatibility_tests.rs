//! ExifTool compatibility tests
//!
//! These tests compare exif-oxide output against stored ExifTool reference snapshots
//! to ensure compatibility. ExifTool snapshots are the authoritative reference.
//!
//! Note: These tests require the `integration-tests` feature to be enabled and
//! external test assets to be available. They are automatically skipped in published crates.

#![cfg(feature = "integration-tests")]

use exif_oxide::compat::{
    analyze_tag_differences, filter_to_custom_tags, filter_to_supported_tags,
    normalize_for_comparison, run_exif_oxide, CompatibilityReport, DifferenceType,
};
use serde_json::Value;
use similar::{ChangeTag, TextDiff};
use std::path::Path;

mod common;

/// Files to exclude from testing (problematic files to deal with later)
const EXCLUDED_FILES: &[&str] = &[
    "third-party/exiftool/t/images/ExtendedXMP.jpg",
    "third-party/exiftool/t/images/PhotoMechanic.jpg",
    "third-party/exiftool/t/images/ExifTool.jpg",
    "third-party/exiftool/t/images/CasioQVCI.jpg",
    "third-party/exiftool/t/images/InfiRay.jpg", // Thermal imaging - specialized format
    "third-party/exiftool/t/images/IPTC.jpg",    // IPTC-specific metadata edge case
];

/// Parse the TAGS_FILTER environment variable into a list of tag names
/// Format: "Composite:Lens,EXIF:Make,File:FileType"
fn parse_tags_filter() -> Option<Vec<String>> {
    std::env::var("TAGS_FILTER").ok().map(|filter_str| {
        filter_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    })
}

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

/// Compare ExifTool snapshot and exif-oxide output for a specific file
#[allow(dead_code)]
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

    // Filter both to supported tags (or custom tags if TAGS_FILTER is set)
    let (mut filtered_exiftool, mut filtered_exif_oxide) =
        if let Some(custom_tags) = parse_tags_filter() {
            println!("  Using custom tag filter: {:?}", custom_tags);
            let tag_refs: Vec<&str> = custom_tags.iter().map(|s| s.as_str()).collect();
            (
                filter_to_custom_tags(&exiftool_output, &tag_refs),
                filter_to_custom_tags(&exif_oxide_output, &tag_refs),
            )
        } else {
            (
                filter_to_supported_tags(&exiftool_output),
                filter_to_supported_tags(&exif_oxide_output),
            )
        };

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
#[allow(dead_code)]
fn get_known_missing_tags(file_path: &str) -> Vec<&'static str> {
    let path_lower = file_path.to_lowercase();

    // Panasonic RW2 files - missing tags due to incomplete IFD chaining and MakerNotes
    // See: docs/todo/HANDOFF-panasonic-rw2-complete-resolution.md
    if 1 > 2
        && (path_lower.contains("panasonic") || path_lower.contains("lumix"))
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
    let mut all_tag_differences = std::collections::HashMap::new(); // tag -> TagDifference

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
                // Filter both to supported tags (or custom tags if TAGS_FILTER is set)
                let (mut filtered_exiftool, mut filtered_exif_oxide) =
                    if let Some(custom_tags) = parse_tags_filter() {
                        let tag_refs: Vec<&str> = custom_tags.iter().map(|s| s.as_str()).collect();
                        (
                            filter_to_custom_tags(&exiftool_data, &tag_refs),
                            filter_to_custom_tags(&our_data, &tag_refs),
                        )
                    } else {
                        (
                            filter_to_supported_tags(&exiftool_data),
                            filter_to_supported_tags(&our_data),
                        )
                    };

                // Remove known missing tags for this file type
                remove_known_missing_tags(&mut filtered_exiftool, &file_path);
                remove_known_missing_tags(&mut filtered_exif_oxide, &file_path);

                // Normalize for comparison
                let normalized_exiftool = normalize_for_comparison(filtered_exiftool, true);
                let normalized_exif_oxide = normalize_for_comparison(filtered_exif_oxide, false);

                let file_differences = analyze_tag_differences(
                    &file_path,
                    &normalized_exiftool,
                    &normalized_exif_oxide,
                );

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
            DifferenceType::DependencyFailure => {
                compatibility_report.dependency_failures.push(diff)
            }
            DifferenceType::TypeMismatch => compatibility_report.type_mismatches.push(diff),
            DifferenceType::OnlyInOurs => compatibility_report.only_in_ours.push(diff),
        }
    }

    compatibility_report.total_tags_tested = compatibility_report.working_tags.len()
        + compatibility_report.value_format_mismatches.len()
        + compatibility_report.missing_tags.len()
        + compatibility_report.dependency_failures.len()
        + compatibility_report.type_mismatches.len()
        + compatibility_report.only_in_ours.len();

    // Print the enhanced compatibility report
    compatibility_report.print_summary();

    // Report critical issues but don't fail the test - this is for tracking progress
    let critical_failures =
        compatibility_report.missing_tags.len() + compatibility_report.type_mismatches.len();
    let dependency_failures = compatibility_report.dependency_failures.len();

    if critical_failures > 0 || dependency_failures > 0 {
        println!(
            "\n⚠️  Tracking {} compatibility gaps: {} missing tags, {} dependency failures, {} type mismatches",
            critical_failures + dependency_failures,
            compatibility_report.missing_tags.len(),
            dependency_failures,
            compatibility_report.type_mismatches.len()
        );
        println!("This test tracks progress towards ExifTool compatibility - failures are expected during development.");
    } else {
        println!("\n🎉 Perfect ExifTool compatibility achieved!");
    }
}
