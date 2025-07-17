//! MIME Type Compatibility Validation System
//!
//! This module implements the comprehensive integration test system that validates
//! our MIME type detection against ExifTool's output across all test images.
//!
//! Based on MILESTONE-MIME-TYPE-COMPAT.md Phase 1-3 implementation plan.
//!
//! The system validates that our FileTypeDetector produces identical results to
//! ExifTool across hundreds of real-world test files, ensuring ongoing compatibility
//! and regression prevention.
//!
//! Note: These tests require the `integration-tests` feature to be enabled and
//! external test assets to be available. They are automatically skipped in published crates.

#![cfg(feature = "integration-tests")]

use exif_oxide::file_detection::{FileDetectionError, FileTypeDetector};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;

/// Known acceptable differences between our implementation and ExifTool
/// These represent cases where we intentionally differ (e.g., providing fallback MIME types)
static KNOWN_DIFFERENCES: LazyLock<HashMap<&'static str, KnownDifference>> = LazyLock::new(|| {
    // Currently empty - all known differences have been resolved
    HashMap::new()
});

#[derive(Debug, Clone)]
enum KnownDifference {
    /// We provide fallback MIME type, ExifTool doesn't
    #[allow(dead_code)]
    FallbackMime(&'static str),
    /// ExifTool doesn't recognize format but we do
    #[allow(dead_code)]
    ExifToolUnsupported,
    /// Different but equivalent MIME types
    #[allow(dead_code)]
    StandardVariation,
    /// File type detection differs based on content analysis vs extension
    #[allow(dead_code)]
    ContentBasedOverride,
    /// Magic pattern doesn't match the specific test file format
    #[allow(dead_code)]
    PatternMismatch,
}

#[derive(Debug, PartialEq)]
struct ExifToolMimeResult {
    file_path: PathBuf,
    mime_type: Option<String>,
    error: Option<String>,
}

#[derive(Debug)]
struct MimeComparison {
    file_path: PathBuf,
    #[allow(dead_code)]
    exiftool_mime: Option<String>,
    #[allow(dead_code)]
    our_mime: Option<String>,
    match_result: MatchResult,
}

#[derive(Debug)]
enum MatchResult {
    ExactMatch,
    KnownDifference(String), // e.g., our fallback vs ExifTool's missing
    Mismatch(String),        // unexpected difference
    ExifToolError(String),   // ExifTool couldn't process
    #[allow(dead_code)]
    OurError(String), // Our detector failed
}

/// File Discovery System
/// Discovers all test files from both test-images and ExifTool test suite
fn discover_test_files() -> Vec<PathBuf> {
    let mut files = Vec::new();

    // Discover files in test-images directory (real camera files)
    if let Ok(test_images_dir) = std::fs::read_dir("test-images") {
        for entry in test_images_dir.flatten() {
            if entry.path().is_dir() {
                files.extend(discover_files_recursively(&entry.path()));
            } else if is_supported_test_file(&entry.path()) {
                files.push(entry.path());
            }
        }
    }

    // Discover files in ExifTool test suite
    if let Ok(exiftool_dir) = std::fs::read_dir("third-party/exiftool/t/images") {
        for entry in exiftool_dir.flatten() {
            if is_supported_test_file(&entry.path()) {
                files.push(entry.path());
            }
        }
    }

    // Sort for deterministic test order
    files.sort();
    files
}

/// Recursively discover files in a directory
fn discover_files_recursively(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(discover_files_recursively(&path));
            } else if is_supported_test_file(&path) {
                files.push(path);
            }
        }
    }

    files
}

/// Check if a file should be included in testing
/// Filters for image, video, and metadata files while excluding README and system files
fn is_supported_test_file(path: &Path) -> bool {
    // Skip system files and documentation
    if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
        if filename.starts_with('.') || filename.eq_ignore_ascii_case("readme.md") {
            return false;
        }
    }

    // Only test files with extensions that we expect to support
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let ext_lower = ext.to_lowercase();
        matches!(
            ext_lower.as_str(),
            // Common image formats
            "jpg" | "jpeg" | "png" | "tiff" | "tif" | "gif" | "bmp" | "webp"
            // RAW formats
            | "cr2" | "cr3" | "crw" | "nef" | "nrw" | "arw" | "sr2" | "srf" 
            | "raf" | "orf" | "rw2" | "dng" | "pef" | "mrw" | "x3f" | "iiq"
            // Video formats  
            | "mp4" | "mov" | "avi" | "mkv" | "mts" | "m2ts" | "3gp" | "3g2"
            | "m4v" | "wmv" | "webm" | "dv" | "flv"
            // Other supported formats
            | "pdf" | "xmp" |  "icm" | "psd" | "psb" | "eps" | "ai"
            | "heic" | "heif" | "avif" | "j2c" | "jp2" | "jxl" | "bpg"
            // Audio formats for completeness
            | "mp3" | "aac" | "flac" | "ogg" | "wav" | "aiff" | "aif" // TODO: add "icc" for ICC profiles if needed
        )
    } else {
        false
    }
}

/// Batch ExifTool Output Parser
/// Runs ExifTool once in recursive mode on both test directories
fn run_exiftool_batch() -> HashMap<PathBuf, ExifToolMimeResult> {
    let mut results = HashMap::new();

    // Test directories to scan
    let test_dirs = ["test-images", "third-party/exiftool/t/images"];

    for test_dir in &test_dirs {
        if !std::path::Path::new(test_dir).exists() {
            eprintln!("Warning: Test directory {test_dir} not found, skipping");
            continue;
        }

        println!("Running ExifTool on {test_dir} directory...");

        // Run ExifTool in recursive mode with JSON output
        let output = Command::new("exiftool")
            .args([
                "-r",        // Recursive
                "-MIMEType", // Extract MIME type
                "-j",        // JSON output
                test_dir,
            ])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);

                // Parse JSON output
                match serde_json::from_str::<Value>(&stdout) {
                    Ok(json) => {
                        if let Some(array) = json.as_array() {
                            for item in array {
                                if let Some(obj) = item.as_object() {
                                    // Get the SourceFile (full path) and MIMEType
                                    let source_file = obj
                                        .get("SourceFile")
                                        .and_then(|v| v.as_str())
                                        .map(PathBuf::from);

                                    let mime_type = obj
                                        .get("MIMEType")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string());

                                    if let Some(file_path) = source_file {
                                        let result = ExifToolMimeResult {
                                            file_path: file_path.clone(),
                                            mime_type,
                                            error: None,
                                        };
                                        results.insert(file_path, result);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to parse ExifTool JSON output for {test_dir}: {e}");
                        eprintln!("Output was: {stdout}");
                    }
                }
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("ExifTool failed on {test_dir}: {stderr}");
            }
            Err(e) => {
                eprintln!("Failed to run ExifTool on {test_dir}: {e}");
            }
        }
    }

    println!("ExifTool processed {} files total", results.len());
    results
}

/// Run our file type detection on a file
fn detect_our_mime_type(
    detector: &FileTypeDetector,
    file_path: &Path,
) -> Result<String, FileDetectionError> {
    let mut file = std::fs::File::open(file_path)?;
    let result = detector.detect_file_type(file_path, &mut file)?;
    Ok(result.mime_type)
}

/// Core Comparison Logic with tolerance for expected differences
fn compare_mime_types(
    file_path: &Path,
    exiftool_result: &ExifToolMimeResult,
    our_result: &Result<String, FileDetectionError>,
) -> MimeComparison {
    // Handle our detection errors
    let our_mime = match our_result {
        Ok(mime) => Some(mime.clone()),
        Err(_) => None,
    };

    // First check if this file has a known difference by full path
    let path_str = file_path.to_str().unwrap_or("");
    if let Some(known_diff) = KNOWN_DIFFERENCES.get(path_str) {
        let description = match known_diff {
            KnownDifference::ContentBasedOverride => {
                "Content-based file type override - test uses FileTypeDetector only".to_string()
            }
            KnownDifference::PatternMismatch => {
                "Magic pattern doesn't match this specific file format variant".to_string()
            }
            KnownDifference::FallbackMime(mime) => {
                format!("Known fallback MIME type: {mime}")
            }
            KnownDifference::ExifToolUnsupported => {
                "ExifTool doesn't support this format".to_string()
            }
            KnownDifference::StandardVariation => {
                "Different but equivalent MIME type standards".to_string()
            }
        };

        return MimeComparison {
            file_path: file_path.to_path_buf(),
            exiftool_mime: exiftool_result.mime_type.clone(),
            our_mime,
            match_result: MatchResult::KnownDifference(description),
        };
    }

    // Compare results with tolerance for known differences
    let match_result = match (
        &exiftool_result.error,
        &exiftool_result.mime_type,
        &our_mime,
    ) {
        // ExifTool had an error
        (Some(error), _, _) => MatchResult::ExifToolError(error.clone()),

        // Both successful - compare MIME types
        (None, Some(et_mime), Some(our_mime_val)) if et_mime == our_mime_val => {
            MatchResult::ExactMatch
        }

        // ExifTool has no MIME type, we have one - check if it's a known fallback
        (None, None, Some(our_mime_val)) => {
            if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
                let ext_upper = ext.to_uppercase();
                if let Some(known_diff) = KNOWN_DIFFERENCES.get(ext_upper.as_str()) {
                    match known_diff {
                        KnownDifference::FallbackMime(expected_mime) => {
                            if our_mime_val == expected_mime {
                                MatchResult::KnownDifference(format!(
                                    "We provide fallback MIME type '{our_mime_val}', ExifTool has none"
                                ))
                            } else {
                                MatchResult::Mismatch(format!(
                                    "Expected fallback '{expected_mime}' but got '{our_mime_val}'"
                                ))
                            }
                        }
                        _ => MatchResult::KnownDifference(format!(
                            "We detect '{our_mime_val}', ExifTool has no MIME type"
                        )),
                    }
                } else {
                    MatchResult::Mismatch(format!(
                        "We detect '{our_mime_val}', ExifTool has no MIME type"
                    ))
                }
            } else {
                MatchResult::Mismatch(format!(
                    "We detect '{our_mime_val}', ExifTool has no MIME type"
                ))
            }
        }

        // ExifTool has MIME type, we don't
        (None, Some(et_mime), None) => {
            MatchResult::Mismatch(format!("ExifTool detects '{et_mime}', we failed to detect"))
        }

        // Both have MIME types but they differ
        (None, Some(et_mime), Some(our_mime_val)) => MatchResult::Mismatch(format!(
            "MIME type mismatch: ExifTool='{et_mime}', Ours='{our_mime_val}'"
        )),

        // Both have no MIME type
        (None, None, None) => MatchResult::ExactMatch,
    };

    MimeComparison {
        file_path: file_path.to_path_buf(),
        exiftool_mime: exiftool_result.mime_type.clone(),
        our_mime,
        match_result,
    }
}

/// Run compatibility tests using batch ExifTool results
fn run_compatibility_tests(
    exiftool_results: HashMap<PathBuf, ExifToolMimeResult>,
) -> Vec<MimeComparison> {
    let detector = FileTypeDetector::new();
    let mut comparisons = Vec::new();

    for (file_path, exiftool_result) in exiftool_results {
        // Only test files that we expect to support
        if is_supported_test_file(&file_path) {
            let our_result = detect_our_mime_type(&detector, &file_path);
            let comparison = compare_mime_types(&file_path, &exiftool_result, &our_result);
            comparisons.push(comparison);
        }
    }

    comparisons
}

/// Generate detailed compatibility report
fn generate_compatibility_report(comparisons: &[MimeComparison]) -> String {
    let mut report = String::new();

    let (matches, issues): (Vec<_>, Vec<_>) = comparisons.iter().partition(|c| {
        matches!(
            c.match_result,
            MatchResult::ExactMatch | MatchResult::KnownDifference(_)
        )
    });

    report.push_str("MIME Type Compatibility Report\n");
    report.push_str("============================\n\n");
    report.push_str(&format!("Total files tested: {}\n", comparisons.len()));
    report.push_str(&format!("Successful matches: {}\n", matches.len()));
    report.push_str(&format!("Issues found: {}\n\n", issues.len()));

    // Group issues by type
    let mut mismatches = Vec::new();
    let mut exiftool_errors = Vec::new();
    let mut our_errors = Vec::new();

    for issue in &issues {
        match &issue.match_result {
            MatchResult::Mismatch(_) => mismatches.push(issue),
            MatchResult::ExifToolError(_) => exiftool_errors.push(issue),
            MatchResult::OurError(_) => our_errors.push(issue),
            _ => {}
        }
    }

    if !mismatches.is_empty() {
        report.push_str(&format!("MIME Type Mismatches ({}):\n", mismatches.len()));
        report.push_str("=====================================\n");
        for mismatch in &mismatches {
            if let MatchResult::Mismatch(description) = &mismatch.match_result {
                report.push_str(&format!(
                    "  {}: {}\n",
                    mismatch.file_path.display(),
                    description
                ));
            }
        }
        report.push('\n');
    }

    if !our_errors.is_empty() {
        report.push_str(&format!("Our Detection Failures ({}):\n", our_errors.len()));
        report.push_str("================================\n");
        for error in &our_errors {
            if let MatchResult::OurError(description) = &error.match_result {
                report.push_str(&format!(
                    "  {}: {}\n",
                    error.file_path.display(),
                    description
                ));
            }
        }
        report.push('\n');
    }

    if !exiftool_errors.is_empty() {
        report.push_str(&format!("ExifTool Errors ({}):\n", exiftool_errors.len()));
        report.push_str("====================\n");
        for error in &exiftool_errors {
            if let MatchResult::ExifToolError(description) = &error.match_result {
                report.push_str(&format!(
                    "  {}: {}\n",
                    error.file_path.display(),
                    description
                ));
            }
        }
        report.push('\n');
    }

    // Summary of successful matches
    if !matches.is_empty() {
        let exact_matches = matches
            .iter()
            .filter(|m| matches!(m.match_result, MatchResult::ExactMatch))
            .count();
        let known_differences = matches.len() - exact_matches;

        report.push_str("Successful Matches Summary:\n");
        report.push_str("===========================\n");
        report.push_str(&format!("  Exact matches: {exact_matches}\n"));
        report.push_str(&format!("  Known differences: {known_differences}\n"));

        if known_differences > 0 {
            report.push_str("\nKnown Differences:\n");
            for m in &matches {
                if let MatchResult::KnownDifference(description) = &m.match_result {
                    report.push_str(&format!("  {}: {}\n", m.file_path.display(), description));
                }
            }
        }
    }

    report
}

/// Main compatibility test
#[test]
fn test_mime_type_compatibility() {
    // Run ExifTool once in batch mode to get all MIME types
    let exiftool_results = run_exiftool_batch();

    if exiftool_results.is_empty() {
        panic!("No test files found! Make sure test-images/ and third-party/exiftool/t/images/ exist and ExifTool is installed");
    }

    println!(
        "Testing MIME type compatibility on {} files",
        exiftool_results.len()
    );

    let comparisons = run_compatibility_tests(exiftool_results);

    let issues: Vec<_> = comparisons
        .iter()
        .filter(|c| {
            !matches!(
                c.match_result,
                MatchResult::ExactMatch | MatchResult::KnownDifference(_)
            )
        })
        .collect();

    if !issues.is_empty() {
        eprintln!("\n{}", generate_compatibility_report(&comparisons));
        panic!("Found {} MIME type compatibility issues", issues.len());
    }

    println!(
        "✅ All {} files passed MIME type compatibility tests",
        comparisons.len()
    );

    // Performance validation - calculate average detection time
    let start = std::time::Instant::now();
    let detector = FileTypeDetector::new();

    // Test 10 random files for performance measurement
    let test_files: Vec<_> = comparisons.iter().take(10).collect();
    for comparison in &test_files {
        if let Ok(mut file) = std::fs::File::open(&comparison.file_path) {
            let _result = detector.detect_file_type(&comparison.file_path, &mut file);
        }
    }

    let elapsed = start.elapsed();
    let avg_time = elapsed / test_files.len() as u32;

    println!("Average detection time: {avg_time:?}");

    // Should be well under 1ms per detection (milestone requirement)
    assert!(
        avg_time.as_millis() < 1,
        "Detection too slow: {avg_time:?} per detection (should be <1ms)"
    );
}

/// Test file discovery system
#[test]
fn test_file_discovery() {
    let files = discover_test_files();

    println!("Discovered {} test files:", files.len());

    let mut by_extension: HashMap<String, usize> = HashMap::new();
    for file in &files {
        if let Some(ext) = file.extension().and_then(|e| e.to_str()) {
            *by_extension.entry(ext.to_lowercase()).or_insert(0) += 1;
        }
    }

    // Print summary by extension
    let mut extensions: Vec<_> = by_extension.into_iter().collect();
    extensions.sort_by(|a, b| a.0.cmp(&b.0));

    for (ext, count) in extensions {
        println!("  .{ext}: {count} files");
    }

    // Should have a reasonable number of test files
    assert!(
        files.len() >= 50,
        "Expected at least 50 test files, found {}",
        files.len()
    );
}

/// Test ExifTool integration
#[test]
fn test_exiftool_integration() {
    // Test that ExifTool is available and working
    let output = Command::new("exiftool").args(["-ver"]).output();

    match output {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("ExifTool version: {}", version.trim());
        }
        Ok(_) => panic!("ExifTool command failed - ensure ExifTool is installed"),
        Err(e) => panic!("Failed to run ExifTool: {e} - ensure ExifTool is installed"),
    }

    // Test ExifTool MIME type detection on a simple JPEG
    let files = discover_test_files();
    let jpeg_file = files.iter().find(|f| {
        f.extension()
            .and_then(|e| e.to_str())
            .map(|ext| ext.to_lowercase() == "jpg" || ext.to_lowercase() == "jpeg")
            .unwrap_or(false)
    });

    if let Some(jpeg_path) = jpeg_file {
        // Test single file ExifTool execution for validation
        let output = Command::new("exiftool")
            .args(["-MIMEType", "-s", "-S"])
            .arg(jpeg_path)
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mime_type = stdout.trim();
                if !mime_type.is_empty() {
                    println!(
                        "ExifTool detected MIME type '{}' for {}",
                        mime_type,
                        jpeg_path.display()
                    );
                    assert_eq!(mime_type, "image/jpeg", "Expected JPEG MIME type");
                } else {
                    println!("ExifTool returned no MIME type for {}", jpeg_path.display());
                }
            }
            _ => println!("ExifTool failed to process {}", jpeg_path.display()),
        }
    } else {
        println!("No JPEG files found for ExifTool integration test");
    }
}
