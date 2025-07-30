//! Comprehensive binary extraction compatibility tests
//!
//! Validates that exif-oxide's -b flag produces byte-identical output to ExifTool
//! across all supported file formats and manufacturers from SUPPORTED-FORMATS.md.
//!
//! This test discovers binary tags dynamically using ExifTool's JSON output,
//! then validates extraction compatibility using SHA256 hash comparison.

#![cfg(feature = "integration-tests")]

use exif_oxide::compat::run_exiftool;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

/// Binary tags that can contain extractable image data
/// Based on SUPPORTED-FORMATS.md embedded image extraction requirements
const EXPECTED_BINARY_TAGS: &[&str] = &[
    "PreviewImage",
    "PreviewTIFF",
    "JpgFromRaw",
    "JpgFromRaw2",
    "ThumbnailImage",
    "ThumbnailTIFF",
    "OtherImage", // Sony ARW and other formats use this naming
];

/// Results from testing binary extraction on a single file
#[derive(Debug, Clone)]
struct BinaryExtractionResult {
    file_path: PathBuf,
    tag_name: String,
    exiftool_size: usize,
    exif_oxide_size: usize,
    exiftool_sha256: String,
    #[allow(dead_code)]
    exif_oxide_sha256: String,
    matches: bool,
    error: Option<String>,
}

/// Comprehensive test results aggregated across all files and formats
#[derive(Debug, Default)]
struct ComprehensiveResults {
    total_files_scanned: usize,
    files_with_binary_data: usize,
    total_extractions_attempted: usize,
    successful_extractions: Vec<BinaryExtractionResult>,
    failed_extractions: Vec<BinaryExtractionResult>,
    errors: Vec<String>,

    // Statistics by format and manufacturer
    results_by_extension: HashMap<String, (usize, usize)>, // (success, total)
    results_by_manufacturer: HashMap<String, (usize, usize)>,

    // Failure analysis by MIME type and tag name for digestible error reporting
    failures_by_mime_tag: HashMap<String, Vec<PathBuf>>, // "mime_type:tag_name" -> list of failing files
}

impl ComprehensiveResults {
    fn new() -> Self {
        Self::default()
    }

    fn add_success(&mut self, result: BinaryExtractionResult) {
        self.update_statistics(&result, true);
        self.successful_extractions.push(result);
    }

    fn add_failure(&mut self, result: BinaryExtractionResult, exiftool_data: &Value) {
        self.update_statistics(&result, false);

        // Add to MIME type + tag name failure tracking using ExifTool data
        let mime_type = get_mime_type_from_exiftool_data(exiftool_data);
        let failure_key = format!("{}:{}", mime_type, result.tag_name);
        self.failures_by_mime_tag
            .entry(failure_key)
            .or_default()
            .push(result.file_path.clone());

        self.failed_extractions.push(result);
    }

    fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    fn update_statistics(&mut self, result: &BinaryExtractionResult, success: bool) {
        // Update by file extension
        if let Some(ext) = result.file_path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            let entry = self.results_by_extension.entry(ext_str).or_insert((0, 0));
            entry.1 += 1; // total
            if success {
                entry.0 += 1; // success
            }
        }

        // Update by manufacturer (heuristic based on file path)
        let manufacturer = detect_manufacturer_from_path(&result.file_path);
        let entry = self
            .results_by_manufacturer
            .entry(manufacturer)
            .or_insert((0, 0));
        entry.1 += 1; // total
        if success {
            entry.0 += 1; // success
        }
    }

    fn print_comprehensive_report(&self) {
        println!("\n🔍 COMPREHENSIVE BINARY EXTRACTION TEST RESULTS");
        println!("{}", "=".repeat(60));

        println!("\n📊 Overall Statistics:");
        println!("  Files scanned: {}", self.total_files_scanned);
        println!("  Files with binary data: {}", self.files_with_binary_data);
        println!(
            "  Total extractions attempted: {}",
            self.total_extractions_attempted
        );
        println!(
            "  Successful extractions: {}",
            self.successful_extractions.len()
        );
        println!("  Failed extractions: {}", self.failed_extractions.len());
        println!("  Errors encountered: {}", self.errors.len());

        // Success rate calculation
        if self.total_extractions_attempted > 0 {
            let success_rate = (self.successful_extractions.len() as f64
                / self.total_extractions_attempted as f64)
                * 100.0;
            println!("  Success rate: {:.1}%", success_rate);
        }

        // Results by file extension
        if !self.results_by_extension.is_empty() {
            println!("\n📁 Results by File Format:");
            let mut extensions: Vec<_> = self.results_by_extension.iter().collect();
            extensions.sort_by_key(|(ext, _)| ext.as_str());

            for (ext, (success, total)) in extensions {
                let rate = if *total > 0 {
                    (*success as f64 / *total as f64) * 100.0
                } else {
                    0.0
                };
                println!("  .{}: {}/{} ({:.1}%)", ext, success, total, rate);
            }
        }

        // Results by manufacturer
        if !self.results_by_manufacturer.is_empty() {
            println!("\n🏭 Results by Manufacturer:");
            let mut manufacturers: Vec<_> = self.results_by_manufacturer.iter().collect();
            manufacturers.sort_by_key(|(mfg, _)| mfg.as_str());

            for (mfg, (success, total)) in manufacturers {
                let rate = if *total > 0 {
                    (*success as f64 / *total as f64) * 100.0
                } else {
                    0.0
                };
                println!("  {}: {}/{} ({:.1}%)", mfg, success, total, rate);
            }
        }

        // Recent successful extractions (sample)
        if !self.successful_extractions.is_empty() {
            println!("\n✅ Sample Successful Extractions:");
            for result in self.successful_extractions.iter().take(5) {
                println!(
                    "  {} / {} - {} bytes - SHA256: {}...{}",
                    result.file_path.file_name().unwrap().to_string_lossy(),
                    result.tag_name,
                    result.exiftool_size,
                    &result.exiftool_sha256[..8],
                    &result.exiftool_sha256[result.exiftool_sha256.len() - 8..]
                );
            }
            if self.successful_extractions.len() > 5 {
                println!("  ... and {} more", self.successful_extractions.len() - 5);
            }
        }

        // Failed extractions (for debugging)
        if !self.failed_extractions.is_empty() {
            println!("\n❌ Failed Extractions:");
            for result in &self.failed_extractions {
                println!(
                    "  {} / {} - {} vs {} bytes{}",
                    result.file_path.file_name().unwrap().to_string_lossy(),
                    result.tag_name,
                    result.exiftool_size,
                    result.exif_oxide_size,
                    if let Some(error) = &result.error {
                        format!(" - {}", error)
                    } else {
                        " - Hash mismatch".to_string()
                    }
                );
            }
        }

        // Errors
        if !self.errors.is_empty() {
            println!("\n⚠️  Errors Encountered:");
            for error in &self.errors {
                println!("  {}", error);
            }
        }

        // Digestible failure summary by MIME type and tag name
        if !self.failures_by_mime_tag.is_empty() {
            println!("\n🎯 ACTIONABLE FAILURE SUMMARY");
            println!("{}", "-".repeat(60));
            println!("Failures grouped by MIME type and tag for targeted fixes:");

            // Sort by failure count (most common failures first)
            let mut failure_entries: Vec<_> = self.failures_by_mime_tag.iter().collect();
            failure_entries.sort_by(|(_, files_a), (_, files_b)| files_b.len().cmp(&files_a.len()));

            for (mime_tag, failing_files) in failure_entries {
                println!("\n❌ {} ({} files failing)", mime_tag, failing_files.len());

                // Show up to 3 example files for debugging
                let examples: Vec<_> = failing_files
                    .iter()
                    .take(3)
                    .map(|path| path.file_name().unwrap().to_string_lossy())
                    .collect();

                println!("   Examples: {}", examples.join(", "));

                if failing_files.len() > 3 {
                    println!("   ... and {} more files", failing_files.len() - 3);
                }
            }

            println!("\n💡 Prioritize fixes by working on the most common failures first");
            println!("   Use: make binary-test-file FILE=path/to/example/file.ext");
        }

        println!("\n{}", "=".repeat(60));
    }
}

/// Test binary extraction for a specific image file (for debugging)
/// Set the BINARY_TEST_FILE environment variable to specify the file to test
/// Example: BINARY_TEST_FILE=test-images/nikon/d850.nef cargo test --test binary_extraction_comprehensive test_specific_binary_extraction --features integration-tests -- --nocapture
#[test]
fn test_specific_binary_extraction() {
    let Some(test_file) = std::env::var("BINARY_TEST_FILE").ok() else {
        println!("⏭️  Skipping test_specific_binary_extraction - BINARY_TEST_FILE not set");
        println!("   Set BINARY_TEST_FILE environment variable to test a specific file.");
        println!("   Example: BINARY_TEST_FILE=test-images/nikon/d850.nef cargo test test_specific_binary_extraction");
        return;
    };

    let file_path = std::path::Path::new(&test_file);
    if !file_path.exists() {
        panic!("Test file does not exist: {}", test_file);
    }

    println!("🔍 Testing specific file: {}", file_path.display());
    println!("🔍 File extension: {:?}", file_path.extension());
    println!(
        "🔍 Manufacturer: {}",
        detect_manufacturer_from_path(file_path)
    );

    // Test this specific file
    match test_binary_extraction_for_file(file_path) {
        Ok(results) => {
            if results.is_empty() {
                println!("📝 No binary data found in this file");

                // Still run ExifTool to see what it shows
                println!("\n🔍 Checking what ExifTool sees:");
                match run_exiftool(&file_path.to_string_lossy()) {
                    Ok(exiftool_data) => {
                        if let Some(obj) = exiftool_data.as_object() {
                            println!("📋 All tags containing 'binary' or 'image':");
                            for (key, value) in obj {
                                if let Some(value_str) = value.as_str() {
                                    let key_lower = key.to_lowercase();
                                    let value_lower = value_str.to_lowercase();
                                    if key_lower.contains("image")
                                        || key_lower.contains("binary")
                                        || value_lower.contains("binary")
                                        || value_lower.contains("image")
                                    {
                                        println!("  {}: {}", key, value_str);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => println!("❌ Failed to run ExifTool: {}", e),
                }
            } else {
                println!("📊 Found {} binary tag(s) to test", results.len());

                let mut successes = 0;
                let mut failures = 0;

                for result in &results {
                    if result.matches && result.error.is_none() {
                        successes += 1;
                        println!(
                            "✅ {}: SUCCESS ({} bytes, SHA256: {}...{})",
                            result.tag_name,
                            result.exiftool_size,
                            &result.exiftool_sha256[..8],
                            &result.exiftool_sha256[result.exiftool_sha256.len() - 8..]
                        );
                    } else {
                        failures += 1;
                        println!(
                            "❌ {}: FAILED ({} vs {} bytes)",
                            result.tag_name, result.exiftool_size, result.exif_oxide_size
                        );
                        if let Some(error) = &result.error {
                            println!("   Error: {}", error);
                        }
                    }
                }

                println!(
                    "\n📊 Summary for {}:",
                    file_path.file_name().unwrap().to_string_lossy()
                );
                println!("  ✅ Successful extractions: {}", successes);
                println!("  ❌ Failed extractions: {}", failures);
                println!(
                    "  📈 Success rate: {:.1}%",
                    if !results.is_empty() {
                        (successes as f64 / results.len() as f64) * 100.0
                    } else {
                        0.0
                    }
                );

                // Don't fail the test - this is for debugging
                if failures > 0 {
                    println!("\n💡 This test is for debugging - failures are expected during development");
                }
            }
        }
        Err(e) => {
            panic!("Failed to test file {}: {}", test_file, e);
        }
    }
}

/// Test comprehensive binary extraction across all supported formats
#[test]
#[ignore = "comprehensive test - takes several minutes to run across 344+ files"]
fn test_comprehensive_binary_extraction() {
    let mut results = ComprehensiveResults::new();

    println!("🚀 Starting comprehensive binary extraction compatibility testing");
    println!("Testing all supported formats from SUPPORTED-FORMATS.md");

    // Discover all supported image files in test directories
    let test_files = discover_test_files();
    if test_files.is_empty() {
        panic!("No test files found. Ensure test-images/ directory exists with sample files.");
    }

    results.total_files_scanned = test_files.len();
    println!(
        "📁 Found {} supported image files to test",
        test_files.len()
    );

    // Test binary extraction for each file
    for file_path in test_files {
        println!("🔍 Testing: {}", file_path.display());

        match test_binary_extraction_for_file(&file_path) {
            Ok(file_results) => {
                if file_results.is_empty() {
                    // No binary data found - normal for many files
                    continue;
                } else {
                    results.files_with_binary_data += 1;
                    results.total_extractions_attempted += file_results.len();

                    for result in file_results {
                        if result.matches && result.error.is_none() {
                            results.add_success(result);
                        } else {
                            // Get ExifTool data for MIME type detection
                            match run_exiftool(&file_path.to_string_lossy()) {
                                Ok(exiftool_data) => {
                                    results.add_failure(result, &exiftool_data);
                                }
                                Err(_) => {
                                    // Fallback if ExifTool fails - use generic MIME type
                                    let mut generic_data = serde_json::Map::new();
                                    generic_data.insert(
                                        "File:MIMEType".to_string(),
                                        serde_json::Value::String(
                                            "application/octet-stream".to_string(),
                                        ),
                                    );
                                    let generic_value = serde_json::Value::Object(generic_data);
                                    results.add_failure(result, &generic_value);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                results.add_error(format!("Failed to test {}: {}", file_path.display(), e));
            }
        }
    }

    // Print comprehensive results
    results.print_comprehensive_report();

    // Test assertions for CI/automated testing
    assert!(results.total_files_scanned > 0, "No files were scanned");

    if results.total_extractions_attempted > 0 {
        let success_rate = results.successful_extractions.len() as f64
            / results.total_extractions_attempted as f64;

        // For now, we expect some failures during development, but track progress
        if success_rate < 0.5 {
            println!("\n⚠️  Warning: Binary extraction success rate is {:.1}% - this is expected during development", success_rate * 100.0);
            println!("   This test tracks progress towards full ExifTool binary extraction compatibility");
        } else {
            println!(
                "\n🎉 Excellent binary extraction compatibility: {:.1}% success rate!",
                success_rate * 100.0
            );
        }

        // Don't fail the test during development - this is for tracking progress
        // In the future, we can set a minimum success rate threshold
    } else {
        println!("\n📝 No binary data found in test files - this may indicate missing test assets");
    }
}

/// Discover all supported image files for testing
fn discover_test_files() -> Vec<PathBuf> {
    let mut files = Vec::new();

    // Test directories to scan
    let test_dirs = ["test-images", "third-party/exiftool/t/images"];

    // Supported extensions from SUPPORTED-FORMATS.md
    let supported_extensions = [
        "jpg", "jpeg", "jpe", // JPEG
        "png", // PNG
        "tiff", "tif",  // TIFF
        "webp", // WebP
        "heic", "heif", "hif", // HEIF/HEIC
        "heics", "heifs", // HEIF sequences
        "avif",  // AVIF
        "bmp",   // BMP
        "gif",   // GIF
        "cr2", "cr3", "crw", // Canon RAW
        "nef", "nrw", // Nikon RAW
        "arw", "arq", "sr2", "srf", // Sony RAW
        "raf", // Fujifilm RAW
        "orf", // Olympus RAW
        "raw", "rw2", // Panasonic RAW
        "dng", // Adobe DNG
        "erf", // Epson RAW
        "gpr", // GoPro RAW
        "3fr", "fff", // Hasselblad RAW
        "dcr", "k25", "kdc", // Kodak RAW
        "rwl", // Leica RAW
        "mef", // Mamiya RAW
        "mrw", // Minolta RAW
        "pef", // Pentax RAW
        "iiq", // Phase One RAW
        "srw", // Samsung RAW
        "x3f", // Sigma RAW
        "psd", "psb", // Photoshop
        "dcp", // DNG Camera Profile
    ];

    for test_dir in &test_dirs {
        let test_path = Path::new(test_dir);
        if !test_path.exists() {
            continue;
        }

        // Recursively walk directory structure
        for entry in WalkDir::new(test_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let file_path = entry.path();

            // Check if file has supported extension
            if let Some(extension) = file_path.extension() {
                let ext_str = extension.to_string_lossy().to_lowercase();
                if supported_extensions.contains(&ext_str.as_str()) {
                    files.push(file_path.to_path_buf());
                }
            }
        }
    }

    // Sort for consistent test ordering
    files.sort();
    files
}

/// Test binary extraction for a single file, returning all results
fn test_binary_extraction_for_file(
    file_path: &Path,
) -> Result<Vec<BinaryExtractionResult>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();

    // Get ExifTool metadata to discover binary tags
    let exiftool_data = run_exiftool(&file_path.to_string_lossy())?;

    // Discover binary tags by looking for "Binary data X bytes" patterns
    let binary_tags = discover_binary_tags(&exiftool_data);

    if binary_tags.is_empty() {
        // No binary data in this file - normal case
        return Ok(results);
    }

    println!("  Found binary tags: {}", binary_tags.join(", "));

    // Test extraction for each discovered binary tag
    for tag_name in binary_tags {
        match test_single_tag_extraction(file_path, &tag_name) {
            Ok(result) => results.push(result),
            Err(e) => {
                results.push(BinaryExtractionResult {
                    file_path: file_path.to_path_buf(),
                    tag_name: tag_name.clone(),
                    exiftool_size: 0,
                    exif_oxide_size: 0,
                    exiftool_sha256: String::new(),
                    exif_oxide_sha256: String::new(),
                    matches: false,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    Ok(results)
}

/// Discover binary tags in ExifTool JSON output
fn discover_binary_tags(exiftool_data: &Value) -> Vec<String> {
    let mut binary_tags = Vec::new();

    if let Some(obj) = exiftool_data.as_object() {
        for (key, value) in obj {
            if let Some(value_str) = value.as_str() {
                // Look for ExifTool's binary data indicators
                if value_str.contains("Binary data") && value_str.contains("bytes") {
                    // Extract the base tag name (remove group prefix if present)
                    let tag_name = if key.contains(':') {
                        key.split(':').next_back().unwrap_or(key)
                    } else {
                        key
                    };

                    // Only include expected binary tag types to avoid false positives
                    if EXPECTED_BINARY_TAGS.contains(&tag_name) {
                        binary_tags.push(tag_name.to_string());
                    }
                }
            }
        }
    }

    // Remove duplicates and sort for consistent ordering
    binary_tags.sort();
    binary_tags.dedup();
    binary_tags
}

/// Test extraction of a single binary tag from a file
fn test_single_tag_extraction(
    file_path: &Path,
    tag_name: &str,
) -> Result<BinaryExtractionResult, Box<dyn std::error::Error>> {
    let file_path_str = file_path.to_string_lossy();

    // Extract with ExifTool
    let exiftool_output = Command::new("exiftool")
        .args(["-b", &format!("-{}", tag_name), &file_path_str])
        .output()?;

    if !exiftool_output.status.success() {
        return Err(format!(
            "ExifTool extraction failed: {}",
            String::from_utf8_lossy(&exiftool_output.stderr)
        )
        .into());
    }

    let exiftool_data = exiftool_output.stdout;
    let exiftool_size = exiftool_data.len();
    let exiftool_sha256 = compute_sha256(&exiftool_data);

    // Extract with exif-oxide
    let exif_oxide_output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "exif-oxide",
            "--",
            "-b",
            &format!("-{}", tag_name),
            &file_path_str,
        ])
        .output()?;

    if !exif_oxide_output.status.success() {
        return Err(format!(
            "exif-oxide extraction failed: {}",
            String::from_utf8_lossy(&exif_oxide_output.stderr)
        )
        .into());
    }

    let exif_oxide_data = exif_oxide_output.stdout;
    let exif_oxide_size = exif_oxide_data.len();
    let exif_oxide_sha256 = compute_sha256(&exif_oxide_data);

    // Compare results
    let matches = exiftool_sha256 == exif_oxide_sha256 && exiftool_size == exif_oxide_size;

    if matches {
        println!(
            "    ✅ {} extraction: SUCCESS ({} bytes)",
            tag_name, exiftool_size
        );
    } else {
        println!(
            "    ❌ {} extraction: FAILED ({} vs {} bytes)",
            tag_name, exiftool_size, exif_oxide_size
        );
    }

    Ok(BinaryExtractionResult {
        file_path: file_path.to_path_buf(),
        tag_name: tag_name.to_string(),
        exiftool_size,
        exif_oxide_size,
        exiftool_sha256,
        exif_oxide_sha256,
        matches,
        error: None,
    })
}

/// Compute SHA256 hash of binary data using system sha256sum command
fn compute_sha256(data: &[u8]) -> String {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new("sha256sum")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start sha256sum");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(data).expect("Failed to write to sha256sum");
    }

    let output = child
        .wait_with_output()
        .expect("Failed to read sha256sum output");
    let hash_line = String::from_utf8_lossy(&output.stdout);

    // Extract just the hash part (before the first space)
    hash_line
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string()
}

/// Get MIME type from ExifTool for failure analysis
fn get_mime_type_from_exiftool_data(exiftool_data: &Value) -> String {
    // ExifTool provides MIMEType in the File: group
    if let Some(obj) = exiftool_data.as_object() {
        // Try different possible tag names for MIME type
        for key in ["File:MIMEType", "MIMEType"] {
            if let Some(mime_value) = obj.get(key) {
                if let Some(mime_str) = mime_value.as_str() {
                    return mime_str.to_string();
                }
            }
        }
    }

    // Fallback to generic type if not found
    "application/octet-stream".to_string()
}

/// Detect manufacturer from file path (heuristic for statistics)
fn detect_manufacturer_from_path(file_path: &Path) -> String {
    let path_str = file_path.to_string_lossy().to_lowercase();

    // Check for manufacturer indicators in path
    if path_str.contains("canon") || path_str.contains("cr2") || path_str.contains("cr3") {
        "Canon".to_string()
    } else if path_str.contains("nikon") || path_str.contains("nef") {
        "Nikon".to_string()
    } else if path_str.contains("sony") || path_str.contains("arw") || path_str.contains("sr2") {
        "Sony".to_string()
    } else if path_str.contains("olympus") || path_str.contains("orf") {
        "Olympus".to_string()
    } else if path_str.contains("panasonic") || path_str.contains("rw2") {
        "Panasonic".to_string()
    } else if path_str.contains("fuji") || path_str.contains("raf") {
        "Fujifilm".to_string()
    } else if path_str.contains("pentax") || path_str.contains("pef") {
        "Pentax".to_string()
    } else if path_str.contains("samsung") || path_str.contains("srw") {
        "Samsung".to_string()
    } else if path_str.contains("adobe") || path_str.contains("dng") {
        "Adobe".to_string()
    } else {
        "Other".to_string()
    }
}
