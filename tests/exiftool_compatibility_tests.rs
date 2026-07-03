//! ExifTool compatibility tests
//!
//! These tests compare exif-oxide output against stored ExifTool reference snapshots
//! to ensure compatibility. ExifTool snapshots are the authoritative reference.
//!
//! `test_exiftool_compatibility` is a hard gate, not a dashboard: every tag whose
//! value diverges from the committed ExifTool snapshot must be listed in
//! `config/compat_known_gaps.json` with a reason + reference, or the test fails.
//! Allowlisted tags that start matching again also fail the test (the ratchet).
//!
//! Note: These tests require the `integration-tests` feature to be enabled and
//! external test assets to be available. They are automatically skipped in published crates.

#![cfg(feature = "integration-tests")]

use exif_oxide::compat::{
    analyze_tag_differences, filter_to_custom_tags, filter_to_supported_tags,
    normalize_for_comparison, run_exif_oxide, CompatibilityReport, DifferenceType, KnownGaps,
    TagDifference,
};
use serde_json::Value;
use std::path::Path;

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

/// Human-readable label for a difference type, used in gate failure messages
/// and the COMPAT_DUMP_GAPS machine-readable dump.
fn difference_type_label(dt: &DifferenceType) -> &'static str {
    match dt {
        DifferenceType::Working => "Working",
        DifferenceType::ValueFormatMismatch => "ValueFormatMismatch",
        DifferenceType::Missing => "Missing",
        DifferenceType::DependencyFailure => "DependencyFailure",
        DifferenceType::TypeMismatch => "TypeMismatch",
        DifferenceType::OnlyInOurs => "OnlyInOurs",
    }
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

/// Parse a Perl `$VERSION = '13.59';` assignment, returning the version string.
fn parse_version_line(line: &str) -> Option<String> {
    let rest = line.trim_start().strip_prefix("$VERSION")?;
    let rest = rest.trim_start().strip_prefix('=')?.trim_start();
    let quote = rest.chars().next()?;
    if quote != '\'' && quote != '"' {
        return None;
    }
    let after = &rest[1..];
    let end = after.find(quote)?;
    Some(after[..end].to_string())
}

/// Guard against snapshot/submodule ExifTool version skew.
///
/// `tools/generate_exiftool_json.sh` uses the vendored `third-party/exiftool/exiftool`
/// and records its version in `generated/exiftool-json/.exiftool-version`. That marker
/// must match `$VERSION` in the pinned submodule; otherwise the snapshots were generated
/// against a different ExifTool than the one the repo claims to track (a live skew hazard,
/// since the machine's PATH exiftool is often an older release).
#[test]
fn test_snapshot_exiftool_version_matches_submodule() {
    let pm_path = "third-party/exiftool/lib/Image/ExifTool.pm";
    let pm = std::fs::read_to_string(pm_path).unwrap_or_else(|e| {
        panic!("failed to read {pm_path}: {e} (is the ExifTool submodule initialized?)")
    });
    let submodule_version = pm
        .lines()
        .find_map(parse_version_line)
        .unwrap_or_else(|| panic!("could not find `$VERSION = '...'` line in {pm_path}"));

    let marker_path = "generated/exiftool-json/.exiftool-version";
    let marker = std::fs::read_to_string(marker_path).unwrap_or_else(|e| {
        panic!(
            "missing snapshot version marker {marker_path}: {e}\n\
             → Run `make compat-gen-force` to regenerate snapshots with the vendored ExifTool."
        )
    });

    assert_eq!(
        marker.trim(),
        submodule_version.trim(),
        "\nSnapshot/submodule ExifTool version skew!\n  \
         submodule ({pm_path}) = {sub}\n  \
         snapshots ({marker_path}) = {mark}\n\
         → Run `make compat-gen-force` to regenerate all snapshots with the pinned submodule ExifTool.",
        sub = submodule_version.trim(),
        mark = marker.trim(),
    );
}

/// Test ExifTool compatibility using stored snapshots.
///
/// This is a GATE: it fails on any tag that diverges from the ExifTool snapshot
/// without a matching entry in `config/compat_known_gaps.json`, and also fails when
/// an allowlisted tag starts matching again (stale-entry ratchet). Set `TAGS_FILTER`
/// for a non-asserting debugging run, or `COMPAT_DUMP_GAPS=1` to emit the machine-readable
/// gap list used to (re)seed the allowlist.
#[test]
fn test_exiftool_compatibility() {
    // Discover snapshots
    let snapshots_dir = Path::new("generated/exiftool-json");
    if !snapshots_dir.exists() {
        // The oracle is missing entirely; the version-skew guard test covers this
        // case with a hard failure. Nothing to compare here.
        println!("Reference JSON directory not found. Run 'make compat-gen-force' first.");
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

    // Deterministic iteration order: read_dir order is filesystem-dependent, and the
    // per-tag aggregation below depends on a stable order to pick a representative
    // sample file for each difference.
    snapshot_files.sort();

    println!(
        "Running ExifTool compatibility tests using {} snapshots",
        snapshot_files.len()
    );

    let mut compatibility_report = CompatibilityReport::new();
    let mut tested_files = 0;
    let mut missing_source_files = 0;
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

        // Skip if file doesn't exist (test-images/ is B2-synced, not in git).
        // Absent files mean fewer observations, which can only under-report
        // gaps — but it would make the stale-entry ratchet below false-fire,
        // so we count them and disable that assertion when any are missing.
        if !Path::new(&file_path).exists() {
            missing_source_files += 1;
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
                let (filtered_exiftool, filtered_exif_oxide) =
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

                // Normalize for comparison
                let normalized_exiftool = normalize_for_comparison(filtered_exiftool, true);
                let normalized_exif_oxide = normalize_for_comparison(filtered_exif_oxide, false);

                let file_differences = analyze_tag_differences(
                    &file_path,
                    &normalized_exiftool,
                    &normalized_exif_oxide,
                );

                // Aggregate differences by tag. A non-Working difference always
                // replaces a Working entry, a Working result never replaces a
                // non-Working entry, and among non-Working results the first (in
                // sorted file order) wins. This closes the hole where a tag that
                // is Working in an early file masks a regression in a later file.
                for diff in file_differences {
                    match all_tag_differences.get(&diff.tag) {
                        None => {
                            all_tag_differences.insert(diff.tag.clone(), diff);
                        }
                        Some(existing) => {
                            let existing_working =
                                existing.difference_type == DifferenceType::Working;
                            let new_working = diff.difference_type == DifferenceType::Working;
                            if existing_working && !new_working {
                                all_tag_differences.insert(diff.tag.clone(), diff);
                            }
                        }
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

    // Collect every non-Working difference across all five categories.
    let non_working: Vec<&TagDifference> = compatibility_report
        .value_format_mismatches
        .iter()
        .chain(&compatibility_report.missing_tags)
        .chain(&compatibility_report.dependency_failures)
        .chain(&compatibility_report.type_mismatches)
        .chain(&compatibility_report.only_in_ours)
        .collect();

    // COMPAT_DUMP_GAPS=1 emits a machine-readable list of every current gap, used
    // to (re)seed config/compat_known_gaps.json.
    if std::env::var("COMPAT_DUMP_GAPS").is_ok() {
        let mut dump: Vec<_> = non_working
            .iter()
            .map(|d| {
                serde_json::json!({
                    "tag": d.tag,
                    "type": difference_type_label(&d.difference_type),
                    "sample_file": d.sample_file,
                })
            })
            .collect();
        dump.sort_by(|a, b| a["tag"].as_str().cmp(&b["tag"].as_str()));
        println!(
            "COMPAT_DUMP_GAPS_JSON_BEGIN\n{}\nCOMPAT_DUMP_GAPS_JSON_END",
            serde_json::to_string_pretty(&dump).unwrap()
        );
    }

    // TAGS_FILTER is a debugging mode (arbitrary tag subset). The allowlist is
    // written against the full supported-tag set, so assertions don't apply.
    if parse_tags_filter().is_some() {
        println!(
            "\nℹ️  TAGS_FILTER is set — debugging mode; ExifTool oracle assertions are skipped."
        );
        return;
    }

    // Load the reviewed allowlist. A malformed config (duplicate tag, empty
    // reason/reference) fails the gate loudly.
    let known_gaps = KnownGaps::load()
        .unwrap_or_else(|e| panic!("failed to load config/compat_known_gaps.json: {e}"));

    // (1) Every non-Working difference must be allowlisted.
    let mut unexpected: Vec<&TagDifference> = non_working
        .iter()
        .copied()
        .filter(|d| !known_gaps.contains(&d.tag))
        .collect();
    unexpected.sort_by(|a, b| a.tag.cmp(&b.tag));

    // (2) Every allowlisted tag must actually reproduce as a non-Working difference
    // (stale-entry detection: the ratchet that removes entries once they pass).
    // Only meaningful when every snapshot's source image was on disk: with an
    // unsynced test-images/ corpus (160 of the 168 entries anchor there), absent
    // files would make correct allowlist entries look stale and the failure
    // message would wrongly advise deleting them.
    let non_working_tags: std::collections::HashSet<&str> =
        non_working.iter().map(|d| d.tag.as_str()).collect();
    let mut stale: Vec<&str> = if missing_source_files == 0 {
        known_gaps
            .tags()
            .filter(|t| !non_working_tags.contains(t))
            .collect()
    } else {
        println!(
            "\nℹ️  {missing_source_files} source image(s) missing (run `make pull-test-images`); \
             stale-entry ratchet skipped."
        );
        Vec::new()
    };
    stale.sort();

    let mut failure = String::new();

    if !unexpected.is_empty() {
        failure.push_str(&format!(
            "\n❌ {} tag(s) diverge from ExifTool but are NOT in the allowlist:\n",
            unexpected.len()
        ));
        for d in &unexpected {
            failure.push_str(&format!(
                "  {} [{}] sample={} expected={} got={}\n",
                d.tag,
                difference_type_label(&d.difference_type),
                d.sample_file,
                TagDifference::format_value_truncated(&d.expected),
                TagDifference::format_value_truncated(&d.actual),
            ));
        }
        failure.push_str(
            "\n→ Fix each tag, or add it to config/compat_known_gaps.json with a reason + reference.\n",
        );
    }

    if !stale.is_empty() {
        failure.push_str(&format!(
            "\n❌ {} allowlisted tag(s) now MATCH ExifTool and must be removed from \
             config/compat_known_gaps.json (ratchet):\n",
            stale.len()
        ));
        for t in &stale {
            failure.push_str(&format!("  {t}\n"));
        }
    }

    assert!(
        failure.is_empty(),
        "ExifTool compatibility gate failed:{failure}"
    );

    println!(
        "\n✅ ExifTool compatibility gate passed: {} working tag(s), {} allowlisted gap(s), 0 unexpected divergences.",
        compatibility_report.working_tags.len(),
        known_gaps.tags().count(),
    );
}
