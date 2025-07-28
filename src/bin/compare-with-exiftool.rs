//! Compare exif-oxide output with ExifTool using normalization
//!
//! This tool provides a debugging interface for comparing exif-oxide metadata extraction
//! with ExifTool's output, using the same sophisticated normalization logic as our
//! compatibility tests.
//!
//! Features:
//! - Runs both exif-oxide and ExifTool on the same file
//! - Applies normalization to handle format differences
//! - Shows meaningful differences with proper tolerance handling
//! - Supports group filtering (File:, EXIF:, MakerNotes:, etc.)
//! - Uses the same comparison logic as `make compat` tests

use clap::{Arg, Command};
use exif_oxide::compat::{
    analyze_all_tag_differences, apply_exiftool_filter, filter_to_groups, filter_to_supported_tags,
    normalize_for_comparison, parse_exiftool_filters, run_exif_oxide, run_exiftool,
    CompatibilityReport, DifferenceType,
};
use std::process;

fn main() {
    let matches = Command::new("compare-with-exiftool")
        .about("Compare exif-oxide output with ExifTool using normalization")
        .arg(
            Arg::new("file")
                .help("Image file to analyze")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("group")
                .help("Filter to specific tag groups (e.g., 'File:', 'EXIF:', 'MakerNotes:')")
                .long("group")
                .short('g')
                .value_name("GROUP")
                .action(clap::ArgAction::Append),
        )
        .arg(
            Arg::new("filter")
                .help("ExifTool-style filters: '-EXIF:all' (all EXIF tags), '-Orientation#' (numeric), '-GPS*' (glob)")
                .long("filter")
                .short('f')
                .value_name("FILTER")
                .action(clap::ArgAction::Append),
        )
        .arg(
            Arg::new("supported-only")
                .help("Only compare supported tags from config/supported_tags_final.json")
                .long("supported-only")
                .short('s')
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("json")
                .help("Output differences as JSON")
                .long("json")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("verbose")
                .help("Show detailed comparison report")
                .long("verbose")
                .short('v')
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let file_path = matches.get_one::<String>("file").unwrap();
    let groups: Vec<&str> = matches
        .get_many::<String>("group")
        .map(|values| values.map(|s| s.as_str()).collect())
        .unwrap_or_default();
    let filters: Vec<&str> = matches
        .get_many::<String>("filter")
        .map(|values| values.map(|s| s.as_str()).collect())
        .unwrap_or_default();
    let supported_only = matches.get_flag("supported-only");
    let json_output = matches.get_flag("json");
    let verbose = matches.get_flag("verbose");

    // Run both tools
    println!("🔍 Comparing {} with ExifTool...", file_path);

    let exiftool_result = run_exiftool(file_path);
    let exif_oxide_result = run_exif_oxide(file_path);

    let (mut exiftool_data, mut exif_oxide_data) = match (exiftool_result, exif_oxide_result) {
        (Ok(et), Ok(eo)) => (et, eo),
        (Err(e), _) => {
            eprintln!("❌ ExifTool failed: {}", e);
            process::exit(1);
        }
        (_, Err(e)) => {
            eprintln!("❌ exif-oxide failed: {}", e);
            process::exit(1);
        }
    };

    // Apply filtering - ExifTool-style filters take precedence
    if !filters.is_empty() {
        let filter_options = parse_exiftool_filters(&filters);
        exiftool_data = apply_exiftool_filter(&exiftool_data, &filter_options);
        exif_oxide_data = apply_exiftool_filter(&exif_oxide_data, &filter_options);
    } else {
        // Fallback to legacy filtering options
        if supported_only {
            exiftool_data = filter_to_supported_tags(&exiftool_data);
            exif_oxide_data = filter_to_supported_tags(&exif_oxide_data);
        }

        if !groups.is_empty() {
            exiftool_data = filter_to_groups(&exiftool_data, &groups);
            exif_oxide_data = filter_to_groups(&exif_oxide_data, &groups);
        }
    }

    // Apply normalization
    let normalized_exiftool = normalize_for_comparison(exiftool_data, true);
    let normalized_exif_oxide = normalize_for_comparison(exif_oxide_data, false);

    // Analyze differences
    let differences = if supported_only {
        // Use supported tags analysis when filtering is enabled
        use exif_oxide::compat::analyze_tag_differences;

        // Create temporary JSON with just the supported tags for analysis
        let temp_exiftool = filter_to_supported_tags(&normalized_exiftool);
        let temp_exif_oxide = filter_to_supported_tags(&normalized_exif_oxide);

        analyze_tag_differences(file_path, &temp_exiftool, &temp_exif_oxide)
    } else {
        // Analyze all tags when no filtering
        analyze_all_tag_differences(file_path, &normalized_exiftool, &normalized_exif_oxide)
    };

    // Generate report
    let mut report = CompatibilityReport::new();
    report.total_files_tested = 1;

    for diff in differences {
        match diff.difference_type {
            DifferenceType::Working => report.working_tags.push(diff.tag),
            DifferenceType::ValueFormatMismatch => report.value_format_mismatches.push(diff),
            DifferenceType::Missing => report.missing_tags.push(diff),
            DifferenceType::TypeMismatch => report.type_mismatches.push(diff),
            DifferenceType::OnlyInOurs => report.only_in_ours.push(diff),
        }
    }

    report.total_tags_tested = report.working_tags.len()
        + report.value_format_mismatches.len()
        + report.missing_tags.len()
        + report.type_mismatches.len()
        + report.only_in_ours.len();

    // Output results
    if json_output {
        output_json_report(&report);
    } else if verbose {
        report.print_summary();
    } else {
        report.print_simple_differences(10);
    }

    // Exit with appropriate code
    let critical_issues = report.missing_tags.len() + report.type_mismatches.len();
    if critical_issues > 0 {
        process::exit(1);
    }
}

fn output_json_report(report: &CompatibilityReport) {
    let mut json_report = serde_json::Map::new();

    json_report.insert(
        "total_tags_tested".to_string(),
        serde_json::Value::Number(report.total_tags_tested.into()),
    );
    json_report.insert(
        "total_files_tested".to_string(),
        serde_json::Value::Number(report.total_files_tested.into()),
    );

    // Working tags
    json_report.insert(
        "working_tags".to_string(),
        serde_json::Value::Array(
            report
                .working_tags
                .iter()
                .map(|tag| serde_json::Value::String(tag.clone()))
                .collect(),
        ),
    );

    // Missing tags
    let missing: Vec<serde_json::Value> = report
        .missing_tags
        .iter()
        .map(|diff| {
            let mut obj = serde_json::Map::new();
            obj.insert(
                "tag".to_string(),
                serde_json::Value::String(diff.tag.clone()),
            );
            obj.insert(
                "expected".to_string(),
                diff.expected.clone().unwrap_or(serde_json::Value::Null),
            );
            serde_json::Value::Object(obj)
        })
        .collect();
    json_report.insert(
        "missing_tags".to_string(),
        serde_json::Value::Array(missing),
    );

    // Only in ours
    let only_ours: Vec<serde_json::Value> = report
        .only_in_ours
        .iter()
        .map(|diff| {
            let mut obj = serde_json::Map::new();
            obj.insert(
                "tag".to_string(),
                serde_json::Value::String(diff.tag.clone()),
            );
            obj.insert(
                "actual".to_string(),
                diff.actual.clone().unwrap_or(serde_json::Value::Null),
            );
            serde_json::Value::Object(obj)
        })
        .collect();
    json_report.insert(
        "only_in_ours".to_string(),
        serde_json::Value::Array(only_ours),
    );

    // Type mismatches
    let type_mismatches: Vec<serde_json::Value> = report
        .type_mismatches
        .iter()
        .map(|diff| {
            let mut obj = serde_json::Map::new();
            obj.insert(
                "tag".to_string(),
                serde_json::Value::String(diff.tag.clone()),
            );
            obj.insert(
                "expected".to_string(),
                diff.expected.clone().unwrap_or(serde_json::Value::Null),
            );
            obj.insert(
                "actual".to_string(),
                diff.actual.clone().unwrap_or(serde_json::Value::Null),
            );
            serde_json::Value::Object(obj)
        })
        .collect();
    json_report.insert(
        "type_mismatches".to_string(),
        serde_json::Value::Array(type_mismatches),
    );

    // Value format mismatches
    let format_mismatches: Vec<serde_json::Value> = report
        .value_format_mismatches
        .iter()
        .map(|diff| {
            let mut obj = serde_json::Map::new();
            obj.insert(
                "tag".to_string(),
                serde_json::Value::String(diff.tag.clone()),
            );
            obj.insert(
                "expected".to_string(),
                diff.expected.clone().unwrap_or(serde_json::Value::Null),
            );
            obj.insert(
                "actual".to_string(),
                diff.actual.clone().unwrap_or(serde_json::Value::Null),
            );
            serde_json::Value::Object(obj)
        })
        .collect();
    json_report.insert(
        "value_format_mismatches".to_string(),
        serde_json::Value::Array(format_mismatches),
    );

    let json_value = serde_json::Value::Object(json_report);
    println!("{}", serde_json::to_string_pretty(&json_value).unwrap());
}
