//! ExifTool synchronization tool
//!
//! Tool to synchronize exif-oxide with ExifTool updates and extract algorithms

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

mod extractors;
use extractors::Extractor;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        std::process::exit(1);
    }

    let result = match args[1].as_str() {
        "status" => cmd_status(),
        "diff" => {
            if args.len() != 4 {
                Err("Usage: exiftool_sync diff <from_version> <to_version>".to_string())
            } else {
                cmd_diff(&args[2], &args[3])
            }
        }
        "scan" => cmd_scan(),
        "extract" => {
            if args.len() < 3 {
                Err("Usage: exiftool_sync extract <component> [options]".to_string())
            } else {
                cmd_extract(&args[2], &args[3..])
            }
        }
        "analyze" => {
            if args.len() < 4 || args[2] != "printconv-patterns" {
                Err("Usage: exiftool_sync analyze printconv-patterns <Manufacturer.pm>".to_string())
            } else {
                cmd_analyze_printconv(&args[3])
            }
        }
        "generate" => {
            if args.len() < 4 || args[2] != "printconv-functions" {
                Err(
                    "Usage: exiftool_sync generate printconv-functions <Manufacturer.pm>"
                        .to_string(),
                )
            } else {
                cmd_generate_printconv(&args[3])
            }
        }
        "diff-printconv" => {
            if args.len() != 5 {
                Err("Usage: exiftool_sync diff-printconv <from_version> <to_version> <Manufacturer.pm>".to_string())
            } else {
                cmd_diff_printconv(&args[2], &args[3], &args[4])
            }
        }
        "extract-all" => cmd_extract_all(),
        "add-manufacturer" => {
            if args.len() < 3 {
                Err("Usage: exiftool_sync add-manufacturer <ManufacturerName>".to_string())
            } else {
                cmd_add_manufacturer(&args[2])
            }
        }
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        _ => Err(format!("Unknown command: {}", args[1])),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn cmd_status() -> Result<(), String> {
    // Read simple config
    let config = fs::read_to_string("exiftool-sync.toml")
        .map_err(|e| format!("Failed to read config: {}", e))?;

    let version = extract_value(&config, "version").unwrap_or_else(|| "unknown".to_string());
    let last_sync = extract_value(&config, "last_sync").unwrap_or_else(|| "unknown".to_string());

    println!("ExifTool Synchronization Status");
    println!("==============================");
    println!();
    println!("Current ExifTool version: {}", version);
    println!("Last synchronization: {}", last_sync);

    Ok(())
}

fn cmd_diff(from_version: &str, to_version: &str) -> Result<(), String> {
    println!("ExifTool Version Diff: {} → {}", from_version, to_version);
    println!("=====================================");
    println!();

    // Check if ExifTool directory exists
    if !Path::new("exiftool").exists() {
        return Err(
            "ExifTool directory not found. Please ensure exiftool submodule is initialized."
                .to_string(),
        );
    }

    // Get the list of changed Perl modules
    println!("Fetching changes from git...");
    let output = Command::new("git")
        .args([
            "diff",
            &format!("v{}", from_version),
            &format!("v{}", to_version),
            "--name-only",
            "lib/Image/ExifTool/",
        ])
        .current_dir("exiftool")
        .output()
        .map_err(|e| format!("Failed to run git diff: {}", e))?;

    if !output.status.success() {
        return Err("Git diff failed. Make sure both versions exist.".to_string());
    }

    let changed_files = String::from_utf8_lossy(&output.stdout);

    // Find Rust files that depend on changed Perl modules
    let impacts = find_impacted_rust_files(&changed_files)?;

    if impacts.is_empty() {
        println!("No implemented features affected by these changes.");
    } else {
        println!("CHANGED FILES WITH IMPLEMENTATIONS:");
        for (perl_file, rust_files) in impacts {
            println!("\n{} → impacts:", perl_file);
            for rust_file in rust_files {
                if rust_file.contains("generated") || rust_file.contains("OUT_DIR") {
                    println!("  - {} [AUTO-GENERATED]", rust_file);
                } else {
                    println!("  - {}", rust_file);
                }
            }
        }

        println!("\nAction required:");
        println!("- For [AUTO-GENERATED] files: Run `cargo build` to regenerate");
        println!("- For manual implementations: Review the Perl diff and update accordingly");
    }

    Ok(())
}

fn cmd_scan() -> Result<(), String> {
    println!("Scanning for ExifTool source attributions...");
    println!();

    let mut source_map: HashMap<String, Vec<String>> = HashMap::new();

    // Walk through src directory looking for EXIFTOOL-SOURCE attributes
    scan_directory(Path::new("src"), &mut source_map)?;

    // Also check generated files
    if let Ok(out_dir) = env::var("OUT_DIR") {
        if let Some(generated) = find_generated_attribution(&out_dir) {
            for (perl_file, _) in generated {
                source_map
                    .entry(perl_file)
                    .or_default()
                    .push("[Generated files]".to_string());
            }
        }
    }

    if source_map.is_empty() {
        println!("No EXIFTOOL-SOURCE attributions found.");
    } else {
        println!("ExifTool Source Dependencies:");
        for (perl_file, rust_files) in source_map {
            println!("\n{}:", perl_file);
            for rust_file in rust_files {
                println!("  ← {}", rust_file);
            }
        }
    }

    Ok(())
}

fn scan_directory(dir: &Path, source_map: &mut HashMap<String, Vec<String>>) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            scan_directory(&path, source_map)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            if let Ok(content) = fs::read_to_string(&path) {
                // Look for EXIFTOOL-SOURCE doc attributes
                for line in content.lines() {
                    if let Some(source) = extract_exiftool_source(line) {
                        if !source.is_empty() {
                            source_map
                                .entry(source.to_string())
                                .or_default()
                                .push(path.display().to_string());
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn extract_exiftool_source(line: &str) -> Option<&str> {
    if line.contains("EXIFTOOL-SOURCE:") {
        // Extract the source file path
        if let Some(start) = line.find("EXIFTOOL-SOURCE:") {
            let rest = &line[start + 16..].trim();
            if let Some(end) = rest.find('"') {
                return Some(&rest[..end]);
            } else {
                return Some(rest);
            }
        }
    }
    None
}

fn find_impacted_rust_files(
    changed_perl_files: &str,
) -> Result<HashMap<String, Vec<String>>, String> {
    let mut impacts = HashMap::new();
    let mut source_map: HashMap<String, Vec<String>> = HashMap::new();

    // First, scan all Rust files to build source map
    scan_directory(Path::new("src"), &mut source_map)?;

    // Check each changed Perl file
    for line in changed_perl_files.lines() {
        let perl_file = line.trim();
        if perl_file.is_empty() {
            continue;
        }

        // Check if any Rust files depend on this Perl file
        if let Some(rust_files) = source_map.get(perl_file) {
            impacts.insert(perl_file.to_string(), rust_files.clone());
        }
    }

    Ok(impacts)
}

fn find_generated_attribution(_out_dir: &str) -> Option<Vec<(String, String)>> {
    // This would parse generated files to find their sources
    // For now, return None
    None
}

fn extract_value(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        if line.trim().starts_with(key) {
            if let Some((_, value)) = line.split_once('=') {
                return Some(value.trim().trim_matches('"').to_string());
            }
        }
    }
    None
}

fn cmd_extract(component: &str, _options: &[String]) -> Result<(), String> {
    use extractors::Extractor;

    println!("Extracting component: {}", component);
    println!();

    let extractor: Box<dyn Extractor> = match component {
        "binary-formats" => Box::new(extractors::BinaryFormatsExtractor::new()),
        "magic-numbers" => Box::new(extractors::MagicNumbersExtractor::new()),
        "datetime-patterns" => Box::new(extractors::DateTimePatternsExtractor::new()),
        "binary-tags" => Box::new(extractors::BinaryTagsExtractor::new()),
        "exif-tags" => Box::new(extractors::ExifTagsExtractor),
        "gpmf-tags" => Box::new(extractors::GpmfTagsExtractor::new()),
        "gpmf-format" => Box::new(extractors::GpmfFormatExtractor::new()),
        "maker-detection" => Box::new(extractors::MakerDetectionExtractor::new()),
        "printconv-tables" => {
            if _options.is_empty() {
                return Err(
                    "Usage: exiftool_sync extract printconv-tables <Manufacturer.pm>".to_string(),
                );
            }
            Box::new(extractors::PrintConvTablesExtractor::new(&_options[0]))
        }
        _ => return Err(format!("Unknown component: {}", component)),
    };

    // Get ExifTool source directory
    let exiftool_path = Path::new("third-party/exiftool");
    if !exiftool_path.exists() {
        return Err("ExifTool source not found at third-party/exiftool".to_string());
    }

    // Run extraction
    extractor.extract(exiftool_path)?;

    println!("Extraction complete!");
    Ok(())
}

fn cmd_extract_all() -> Result<(), String> {
    use extractors::Extractor;

    println!("Extracting all components from ExifTool...");
    println!("==========================================");
    println!();

    // Get ExifTool source directory
    let exiftool_path = Path::new("third-party/exiftool");
    if !exiftool_path.exists() {
        return Err("ExifTool source not found at third-party/exiftool".to_string());
    }

    // List of all extractors in order (dependency order doesn't matter since they're all independent)
    let components = vec![
        ("binary-formats", "ProcessBinaryData table definitions"),
        ("magic-numbers", "File type detection patterns"),
        ("datetime-patterns", "Date parsing patterns"),
        ("binary-tags", "Composite tag definitions"),
        ("exif-tags", "Standard EXIF tag definitions"),
        ("gpmf-tags", "GoPro GPMF tag definitions"),
        ("gpmf-format", "GoPro GPMF format definitions"),
        ("maker-detection", "Maker note detection patterns"),
    ];

    let mut successes = 0;
    let mut failures = Vec::new();

    for (component, description) in &components {
        println!("🔄 Extracting {} ({})", component, description);

        let extractor: Box<dyn Extractor> = match *component {
            "binary-formats" => Box::new(extractors::BinaryFormatsExtractor::new()),
            "magic-numbers" => Box::new(extractors::MagicNumbersExtractor::new()),
            "datetime-patterns" => Box::new(extractors::DateTimePatternsExtractor::new()),
            "binary-tags" => Box::new(extractors::BinaryTagsExtractor::new()),
            "exif-tags" => Box::new(extractors::ExifTagsExtractor),
            "gpmf-tags" => Box::new(extractors::GpmfTagsExtractor::new()),
            "gpmf-format" => Box::new(extractors::GpmfFormatExtractor::new()),
            "maker-detection" => Box::new(extractors::MakerDetectionExtractor::new()),
            _ => unreachable!(),
        };

        match extractor.extract(exiftool_path) {
            Ok(()) => {
                println!("   ✅ {} extraction complete", component);
                successes += 1;
            }
            Err(e) => {
                println!("   ❌ {} extraction failed: {}", component, e);
                failures.push((component, e));
            }
        }
        println!();
    }

    // Summary
    println!("===============================================");
    println!("Extraction Summary:");
    println!("  ✅ Successful: {}/{}", successes, components.len());
    if !failures.is_empty() {
        println!("  ❌ Failed: {}", failures.len());
        for (component, error) in &failures {
            println!("     - {}: {}", component, error);
        }
    }

    if failures.is_empty() {
        println!();
        println!("🎉 All ExifTool algorithms successfully extracted!");
        println!("   Next steps:");
        println!("   - Run 'cargo build' to compile with extracted data");
        println!("   - Run 'cargo test' to validate integration");
        Ok(())
    } else {
        Err(format!("{} component(s) failed extraction", failures.len()))
    }
}

fn cmd_analyze_printconv(manufacturer_file: &str) -> Result<(), String> {
    println!("Analyzing PrintConv patterns in {}", manufacturer_file);
    println!("===============================================");
    println!();

    // Get ExifTool source directory
    let exiftool_path = Path::new("third-party/exiftool");
    if !exiftool_path.exists() {
        return Err("ExifTool source not found at third-party/exiftool".to_string());
    }

    let manufacturer_path = exiftool_path
        .join("lib/Image/ExifTool")
        .join(manufacturer_file);
    if !manufacturer_path.exists() {
        return Err(format!(
            "Manufacturer file not found: {}",
            manufacturer_path.display()
        ));
    }

    // Use the PrintConv analyzer extractor
    let analyzer = extractors::PrintConvAnalyzer::new(manufacturer_file);
    analyzer.analyze(&manufacturer_path)?;

    Ok(())
}

fn cmd_generate_printconv(manufacturer_file: &str) -> Result<(), String> {
    println!("Generating PrintConv functions for {}", manufacturer_file);
    println!("==============================================");
    println!();

    // Get ExifTool source directory
    let exiftool_path = Path::new("third-party/exiftool");
    if !exiftool_path.exists() {
        return Err("ExifTool source not found at third-party/exiftool".to_string());
    }

    let manufacturer_path = exiftool_path
        .join("lib/Image/ExifTool")
        .join(manufacturer_file);
    if !manufacturer_path.exists() {
        return Err(format!(
            "Manufacturer file not found: {}",
            manufacturer_path.display()
        ));
    }

    // Use the PrintConv generator extractor
    let generator = extractors::PrintConvGenerator::new(manufacturer_file);
    generator.extract(exiftool_path)?;

    Ok(())
}

fn cmd_diff_printconv(
    from_version: &str,
    to_version: &str,
    manufacturer_file: &str,
) -> Result<(), String> {
    println!(
        "PrintConv Diff: {} {} → {}",
        manufacturer_file, from_version, to_version
    );
    println!("===============================================");
    println!();

    // Get ExifTool source directory
    let exiftool_path = Path::new("third-party/exiftool");
    if !exiftool_path.exists() {
        return Err("ExifTool source not found at third-party/exiftool".to_string());
    }

    // Check if this is a git repository
    let git_dir = exiftool_path.join(".git");
    if !git_dir.exists() {
        println!("⚠️  ExifTool directory is not a git repository.");
        println!("   Analyzing current version only...");
        return analyze_current_version_only(exiftool_path, manufacturer_file);
    }

    // Try to extract patterns from both versions
    println!("📥 Extracting patterns from version {}...", from_version);
    let from_patterns =
        extract_patterns_for_version(exiftool_path, from_version, manufacturer_file)?;

    println!("📥 Extracting patterns from version {}...", to_version);
    let to_patterns = extract_patterns_for_version(exiftool_path, to_version, manufacturer_file)?;

    // Compare patterns
    println!("🔍 Comparing PrintConv patterns...");
    let changes = compare_printconv_patterns(&from_patterns, &to_patterns);

    // Report findings
    print_printconv_diff_report(&changes, from_version, to_version, manufacturer_file);

    Ok(())
}

fn analyze_current_version_only(
    exiftool_path: &Path,
    manufacturer_file: &str,
) -> Result<(), String> {
    let manufacturer_path = exiftool_path
        .join("lib/Image/ExifTool")
        .join(manufacturer_file);
    if !manufacturer_path.exists() {
        return Err(format!(
            "Manufacturer file not found: {}",
            manufacturer_path.display()
        ));
    }

    // Extract current patterns
    println!("📊 Analyzing current PrintConv patterns...");
    let analyzer = extractors::PrintConvAnalyzer::new(manufacturer_file);
    analyzer.analyze(&manufacturer_path)?;

    let patterns = analyzer.get_patterns();
    println!("Found {} PrintConv patterns", patterns.len());
    println!();

    // Show framework capabilities
    print_change_detection_framework();

    // Show optimization opportunities
    print_optimization_analysis(patterns);

    println!("\n💡 To enable version comparison:");
    println!("1. Initialize ExifTool as git repository:");
    println!("   cd third-party/exiftool && git init && git remote add origin https://github.com/exiftool/exiftool.git");
    println!("2. Fetch tags: git fetch --tags");
    println!("3. List available versions: git tag -l");

    Ok(())
}

fn extract_patterns_for_version(
    exiftool_path: &Path,
    version: &str,
    manufacturer_file: &str,
) -> Result<Vec<PrintConvPatternSnapshot>, String> {
    // Save current state
    let current_branch = get_current_git_ref(exiftool_path)?;

    // Checkout specific version
    checkout_git_version(exiftool_path, version)?;

    // Extract patterns
    let manufacturer_path = exiftool_path
        .join("lib/Image/ExifTool")
        .join(manufacturer_file);
    let result = if manufacturer_path.exists() {
        let analyzer = extractors::PrintConvAnalyzer::new(manufacturer_file);
        match analyzer.analyze(&manufacturer_path) {
            Ok(()) => {
                let patterns = analyzer
                    .get_patterns()
                    .iter()
                    .map(|p| PrintConvPatternSnapshot {
                        tag_id: p.tag_id.clone(),
                        tag_name: p.tag_name.clone(),
                        pattern_type: format!("{:?}", p.pattern_type),
                        content_hash: calculate_pattern_hash(p),
                        values: p.values.clone(),
                    })
                    .collect();
                Ok(patterns)
            }
            Err(e) => Err(format!("Failed to analyze patterns for {}: {}", version, e)),
        }
    } else {
        Err(format!(
            "Manufacturer file {} not found in version {}",
            manufacturer_file, version
        ))
    };

    // Restore original state
    checkout_git_version(exiftool_path, &current_branch)?;

    result
}

fn get_current_git_ref(repo_path: &Path) -> Result<String, String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_path)
        .output()
        .map_err(|e| format!("Failed to get current git ref: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err("Failed to get current git reference".to_string())
    }
}

fn checkout_git_version(repo_path: &Path, version: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["checkout", version])
        .current_dir(repo_path)
        .output()
        .map_err(|e| format!("Failed to checkout version {}: {}", version, e))?;

    if output.status.success() {
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to checkout version {}: {}", version, error))
    }
}

fn calculate_pattern_hash(pattern: &extractors::PrintConvPattern) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    pattern.tag_id.hash(&mut hasher);
    pattern.values.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

#[derive(Debug, Clone)]
struct PrintConvPatternSnapshot {
    tag_id: String,
    tag_name: String,
    pattern_type: String,
    content_hash: String,
    values: Vec<(String, String)>,
}

#[derive(Debug)]
enum PrintConvChange {
    Added {
        pattern: PrintConvPatternSnapshot,
    },
    Removed {
        pattern: PrintConvPatternSnapshot,
    },
    Modified {
        from: PrintConvPatternSnapshot,
        to: PrintConvPatternSnapshot,
        change_type: ChangeType,
    },
}

#[derive(Debug)]
enum ChangeType {
    LookupTableExtended, // New entries added
    LookupTableModified, // Existing entries changed
    AlgorithmChanged,    // Complete logic change
    TypeChanged,         // Simple lookup became complex, etc.
}

fn compare_printconv_patterns(
    from_patterns: &[PrintConvPatternSnapshot],
    to_patterns: &[PrintConvPatternSnapshot],
) -> Vec<PrintConvChange> {
    let mut changes = Vec::new();

    // Create lookup maps for efficient comparison
    let from_map: HashMap<String, &PrintConvPatternSnapshot> = from_patterns
        .iter()
        .map(|p| (p.tag_id.clone(), p))
        .collect();
    let to_map: HashMap<String, &PrintConvPatternSnapshot> =
        to_patterns.iter().map(|p| (p.tag_id.clone(), p)).collect();

    // Find added patterns
    for pattern in to_patterns {
        if !from_map.contains_key(&pattern.tag_id) {
            changes.push(PrintConvChange::Added {
                pattern: pattern.clone(),
            });
        }
    }

    // Find removed patterns
    for pattern in from_patterns {
        if !to_map.contains_key(&pattern.tag_id) {
            changes.push(PrintConvChange::Removed {
                pattern: pattern.clone(),
            });
        }
    }

    // Find modified patterns
    for (tag_id, from_pattern) in &from_map {
        if let Some(to_pattern) = to_map.get(tag_id) {
            if from_pattern.content_hash != to_pattern.content_hash {
                let change_type = classify_change_type(from_pattern, to_pattern);
                changes.push(PrintConvChange::Modified {
                    from: (*from_pattern).clone(),
                    to: (*to_pattern).clone(),
                    change_type,
                });
            }
        }
    }

    changes
}

fn classify_change_type(
    from: &PrintConvPatternSnapshot,
    to: &PrintConvPatternSnapshot,
) -> ChangeType {
    // Simple heuristics for change classification
    if from.pattern_type != to.pattern_type {
        ChangeType::TypeChanged
    } else if from.values.len() < to.values.len() {
        // Check if it's just new entries added
        let from_keys: std::collections::HashSet<_> = from.values.iter().map(|(k, _)| k).collect();
        let to_keys: std::collections::HashSet<_> = to.values.iter().map(|(k, _)| k).collect();

        if from_keys.is_subset(&to_keys) {
            ChangeType::LookupTableExtended
        } else {
            ChangeType::LookupTableModified
        }
    } else if from.values.len() == to.values.len() {
        ChangeType::LookupTableModified
    } else {
        ChangeType::AlgorithmChanged
    }
}

fn print_printconv_diff_report(
    changes: &[PrintConvChange],
    from_version: &str,
    to_version: &str,
    manufacturer_file: &str,
) {
    println!("📋 PrintConv Change Report");
    println!("=========================");
    println!("File: {}", manufacturer_file);
    println!("Versions: {} → {}", from_version, to_version);
    println!("Total changes: {}", changes.len());
    println!();

    let mut added_count = 0;
    let mut removed_count = 0;
    let mut modified_count = 0;

    // Group changes by type
    for change in changes {
        match change {
            PrintConvChange::Added { .. } => added_count += 1,
            PrintConvChange::Removed { .. } => removed_count += 1,
            PrintConvChange::Modified { .. } => modified_count += 1,
        }
    }

    println!("📊 Change Summary:");
    println!("- Added patterns: {}", added_count);
    println!("- Removed patterns: {}", removed_count);
    println!("- Modified patterns: {}", modified_count);
    println!();

    if added_count > 0 {
        println!("➕ Added Patterns:");
        for change in changes {
            if let PrintConvChange::Added { pattern } = change {
                println!(
                    "- {} '{}' → NEW PrintConvId variant needed",
                    pattern.tag_id, pattern.tag_name
                );
                if pattern.values.len() <= 3 {
                    print!("  Values: {{ ");
                    for (i, (k, v)) in pattern.values.iter().enumerate() {
                        if i > 0 {
                            print!(", ");
                        }
                        print!("{} => '{}'", k, v);
                    }
                    println!(" }}");
                }
            }
        }
        println!();
    }

    if removed_count > 0 {
        println!("➖ Removed Patterns:");
        for change in changes {
            if let PrintConvChange::Removed { pattern } = change {
                println!(
                    "- {} '{}' → PrintConvId variant can be deprecated",
                    pattern.tag_id, pattern.tag_name
                );
            }
        }
        println!();
    }

    if modified_count > 0 {
        println!("🔄 Modified Patterns:");
        for change in changes {
            if let PrintConvChange::Modified {
                from,
                to,
                change_type,
            } = change
            {
                println!("- {} '{}' → {:?}", from.tag_id, from.tag_name, change_type);
                match change_type {
                    ChangeType::LookupTableExtended => {
                        let new_entries = to.values.len() - from.values.len();
                        println!(
                            "  Action: Auto-regenerate lookup table ({} new entries)",
                            new_entries
                        );
                        println!("  Risk: Low (backward compatible)");
                    }
                    ChangeType::LookupTableModified => {
                        println!("  Action: Auto-regenerate + validate outputs");
                        println!("  Risk: Medium (output values changed)");
                    }
                    ChangeType::AlgorithmChanged => {
                        println!("  Action: Manual review required");
                        println!("  Risk: High (may need new implementation)");
                    }
                    ChangeType::TypeChanged => {
                        println!("  Action: Update PrintConvId assignment");
                        println!("  Risk: Medium (conversion type changed)");
                    }
                }
            }
        }
        println!();
    }

    if changes.is_empty() {
        println!("✅ No PrintConv changes detected between versions.");
    } else {
        print_action_recommendations(changes);
    }
}

fn print_action_recommendations(changes: &[PrintConvChange]) {
    println!("🛠️  Recommended Actions:");
    println!("========================");

    let auto_actions = changes
        .iter()
        .filter(|c| {
            matches!(
                c,
                PrintConvChange::Added { .. }
                    | PrintConvChange::Modified {
                        change_type: ChangeType::LookupTableExtended
                            | ChangeType::LookupTableModified,
                        ..
                    }
            )
        })
        .count();

    let manual_actions = changes.len() - auto_actions;

    if auto_actions > 0 {
        println!(
            "1. Auto-regenerate affected components ({} changes):",
            auto_actions
        );
        println!("   cargo run --bin exiftool_sync extract printconv-tables <Manufacturer.pm>");
        println!();
    }

    if manual_actions > 0 {
        println!("2. Manual review required ({} changes):", manual_actions);
        println!("   - Review algorithm changes for correctness");
        println!("   - Update PrintConvId enum if needed");
        println!("   - Test conversion outputs");
        println!();
    }

    println!("3. Validation testing:");
    println!("   cargo test printconv");
    println!("   # Compare outputs with ExifTool for affected tags");
}

fn print_change_detection_framework() {
    println!("Change Detection Categories:");
    println!("1. LOOKUP_TABLE_EXTENDED");
    println!("   - New entries added to existing lookup table");
    println!("   - Action: Auto-regenerate lookup table");
    println!("   - Risk: Low (backward compatible)");
    println!();

    println!("2. LOOKUP_TABLE_MODIFIED");
    println!("   - Existing entries changed values");
    println!("   - Action: Auto-regenerate + validation");
    println!("   - Risk: Medium (output changes)");
    println!();

    println!("3. ALGORITHM_CHANGED");
    println!("   - PrintConv logic completely different");
    println!("   - Action: Manual review required");
    println!("   - Risk: High (may need new PrintConvId)");
    println!();

    println!("4. PATTERN_ADDED");
    println!("   - New tag with PrintConv added");
    println!("   - Action: Generate new PrintConvId variant");
    println!("   - Risk: Low (additive change)");
    println!();

    println!("5. PATTERN_REMOVED");
    println!("   - Existing tag PrintConv removed");
    println!("   - Action: Deprecate PrintConvId variant");
    println!("   - Risk: Medium (breaking change)");
}

fn print_optimization_analysis(patterns: &[extractors::PrintConvPattern]) {
    // Count patterns by type for optimization analysis
    let mut universal_count = 0;
    let mut lookup_count = 0;
    let mut complex_count = 0;
    let mut shared_lookup_groups: HashMap<String, Vec<String>> = HashMap::new();

    // Analyze patterns for shared lookup opportunities
    for pattern in patterns {
        match &pattern.pattern_type {
            extractors::PrintConvType::Universal(_) => universal_count += 1,
            extractors::PrintConvType::Lookup(name) => {
                lookup_count += 1;
                if name.starts_with("Canon") && !name.contains("Lookup") {
                    shared_lookup_groups
                        .entry(name.clone())
                        .or_default()
                        .push(format!("{} '{}'", pattern.tag_id, pattern.tag_name));
                }
            }
            extractors::PrintConvType::Complex(_) => complex_count += 1,
        }
    }

    println!("🔍 Optimization Analysis:");
    println!(
        "- Universal patterns: {} (can reuse existing)",
        universal_count
    );
    println!("- Lookup patterns: {}", lookup_count);
    println!("- Complex patterns: {}", complex_count);

    if !shared_lookup_groups.is_empty() {
        println!("\n🔗 Shared lookup optimization opportunities:");
        for (shared_name, tag_patterns) in &shared_lookup_groups {
            if tag_patterns.len() > 1 {
                println!(
                    "- {}: {} tags could share implementation",
                    shared_name,
                    tag_patterns.len()
                );
                for tag_pattern in tag_patterns.iter().take(2) {
                    println!("  • {}", tag_pattern);
                }
                if tag_patterns.len() > 2 {
                    println!("  • ... {} more", tag_patterns.len() - 2);
                }
            }
        }

        let total_shared = shared_lookup_groups
            .values()
            .map(|v| v.len())
            .sum::<usize>();
        let duplicates_eliminated = total_shared - shared_lookup_groups.len();
        println!(
            "\n📊 Potential savings: {} duplicate implementations could be eliminated",
            duplicates_eliminated
        );
    }
}

fn print_help() {
    println!("ExifTool Synchronization Tool");
    println!();
    println!("USAGE:");
    println!("    cargo run --bin exiftool_sync <COMMAND>");
    println!();
    println!("COMMANDS:");
    println!("    status                           Show current synchronization status");
    println!("    diff <from> <to>                 Show which Rust files are affected by ExifTool changes");
    println!("    scan                             List all ExifTool source dependencies");
    println!("    extract <component>              Extract algorithms from ExifTool source");
    println!("    extract-all                      Extract all components in one command");
    println!(
        "    analyze printconv-patterns <pm>  Analyze PrintConv patterns in manufacturer file"
    );
    println!("    generate printconv-functions <pm> Generate PrintConv functions for manufacturer");
    println!("    diff-printconv <from> <to> <pm>  Compare PrintConv changes between versions");
    println!("    add-manufacturer <name>          Add complete manufacturer support with automated workflow");
    println!("    help                             Show this help message");
    println!();
    println!("EXTRACT COMPONENTS:");
    println!("    binary-formats                   Extract ProcessBinaryData table definitions");
    println!("    magic-numbers                    Extract file type detection patterns");
    println!("    datetime-patterns                Extract date parsing patterns");
    println!("    binary-tags                      Extract composite tag definitions");
    println!("    exif-tags                        Extract standard EXIF tag definitions");
    println!("    gpmf-tags                        Extract GoPro GPMF tag definitions");
    println!("    gpmf-format                      Extract GoPro GPMF format definitions");
    println!("    maker-detection                  Extract maker note detection patterns");
    println!(
        "    printconv-tables <pm>            Extract complete tag tables with PrintConv mappings"
    );
    println!();
    println!("EXAMPLES:");
    println!("    cargo run --bin exiftool_sync status");
    println!("    cargo run --bin exiftool_sync diff 12.65 12.66");
    println!("    cargo run --bin exiftool_sync scan");
    println!("    cargo run --bin exiftool_sync extract binary-formats");
    println!("    cargo run --bin exiftool_sync extract-all");
    println!("    cargo run --bin exiftool_sync extract maker-detection");
    println!("    cargo run --bin exiftool_sync analyze printconv-patterns Canon.pm");
    println!("    cargo run --bin exiftool_sync generate printconv-functions Canon.pm");
    println!("    cargo run --bin exiftool_sync extract printconv-tables Canon.pm");
    println!("    cargo run --bin exiftool_sync diff-printconv 12.65 12.66 Canon.pm");
    println!("    cargo run --bin exiftool_sync add-manufacturer Sony");
}

/// Add complete manufacturer support with automated workflow
/// This eliminates all the manual hassles by automating:
/// - Detection pattern extraction
/// - PrintConv table generation with enum updates
/// - Module structure creation with conflict resolution
/// - Parser template generation
/// - Integration with existing systems
/// - Basic test generation
fn cmd_add_manufacturer(manufacturer_name: &str) -> Result<(), String> {
    println!("🚀 Adding manufacturer support: {}", manufacturer_name);
    println!("====================================");
    println!();

    let manufacturer_lower = manufacturer_name.to_lowercase();
    let manufacturer_title = format!(
        "{}{}",
        &manufacturer_name[0..1].to_uppercase(),
        &manufacturer_name[1..].to_lowercase()
    );

    // Get ExifTool source directory
    let exiftool_path = Path::new("third-party/exiftool");
    if !exiftool_path.exists() {
        return Err("ExifTool source not found at third-party/exiftool".to_string());
    }

    // Find the manufacturer's .pm file
    let pm_file = format!("{}.pm", manufacturer_title);
    let manufacturer_pm_path = exiftool_path.join("lib/Image/ExifTool").join(&pm_file);

    if !manufacturer_pm_path.exists() {
        return Err(format!(
            "Manufacturer file not found: {}\nAvailable files: {}",
            manufacturer_pm_path.display(),
            list_available_manufacturer_files(exiftool_path)?
        ));
    }

    // Progress tracker
    let mut step = 1;
    let total_steps = 7;

    println!("📋 Automated Implementation Plan:");
    println!("1. Extract detection patterns from ExifTool");
    println!("2. Generate tag tables with PrintConv mappings");
    println!("3. Update PrintConv enum with new variants");
    println!("4. Resolve module structure conflicts");
    println!("5. Generate parser from template");
    println!("6. Integrate with maker note system");
    println!("7. Generate basic tests");
    println!();

    // Step 1: Extract detection patterns
    println!(
        "🔄 Step {}/{}: Extracting detection patterns...",
        step, total_steps
    );
    step += 1;

    let detection_extractor = extractors::MakerDetectionExtractor::new();
    detection_extractor.extract(exiftool_path)?;
    println!("   ✅ Detection patterns generated");

    // Step 2: Generate PrintConv tables
    println!(
        "🔄 Step {}/{}: Generating PrintConv tables...",
        step, total_steps
    );
    step += 1;

    let printconv_extractor = extractors::PrintConvTablesExtractor::new(&pm_file);
    printconv_extractor.extract(exiftool_path)?;
    println!("   ✅ Tag tables with PrintConv mappings generated");

    // Step 3: Update PrintConv enum (automatically handled by PrintConvTablesExtractor)
    println!(
        "🔄 Step {}/{}: PrintConv enum updated automatically",
        step, total_steps
    );
    step += 1;
    println!("   ✅ PrintConvId enum variants added");

    // Step 4: Resolve module conflicts
    println!(
        "🔄 Step {}/{}: Resolving module structure conflicts...",
        step, total_steps
    );
    step += 1;

    resolve_module_conflicts(&manufacturer_lower)?;
    println!("   ✅ Module conflicts resolved");

    // Step 5: Generate parser from template
    println!(
        "🔄 Step {}/{}: Generating parser from template...",
        step, total_steps
    );
    step += 1;

    generate_parser_from_template(&manufacturer_lower, &manufacturer_title)?;
    println!("   ✅ Parser implementation generated");

    // Step 6: Integrate with maker note system
    println!(
        "🔄 Step {}/{}: Integrating with maker note system...",
        step, total_steps
    );
    step += 1;

    integrate_with_maker_system(&manufacturer_lower, &manufacturer_title)?;
    println!("   ✅ Integration complete");

    // Step 7: Generate basic tests and validation
    println!(
        "🔄 Step {}/{}: Generating tests and running validation...",
        step, total_steps
    );

    generate_basic_tests(&manufacturer_lower, &manufacturer_title)?;

    // Run validation pipeline
    run_validation_pipeline(&manufacturer_lower, &manufacturer_title)?;
    println!("   ✅ Basic tests generated and validation passed");

    println!();
    println!(
        "🎉 {} manufacturer support successfully added!",
        manufacturer_title
    );
    println!();
    println!("📝 Next steps:");
    println!("1. Implementation is ready - compilation and basic tests passed");
    println!(
        "2. Run 'cargo test {}' for comprehensive testing",
        manufacturer_lower
    );
    println!(
        "3. Test with actual {} files if available",
        manufacturer_title
    );
    println!(
        "4. Review generated parser in src/maker/{}.rs",
        manufacturer_lower
    );

    Ok(())
}

/// List available manufacturer .pm files for helpful error messages
fn list_available_manufacturer_files(exiftool_path: &Path) -> Result<String, String> {
    let lib_path = exiftool_path.join("lib/Image/ExifTool");
    let entries = fs::read_dir(&lib_path)
        .map_err(|e| format!("Failed to read ExifTool library directory: {}", e))?;

    let mut pm_files = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if let Some(file_name) = path.file_name() {
            if let Some(file_str) = file_name.to_str() {
                if file_str.ends_with(".pm")
                    && !file_str.starts_with("ExifTool")
                    && file_str.chars().next().unwrap().is_uppercase()
                {
                    pm_files.push(file_str.replace(".pm", ""));
                }
            }
        }
    }

    pm_files.sort();
    Ok(pm_files.join(", "))
}

/// Resolve module structure conflicts (both file and directory)
fn resolve_module_conflicts(manufacturer_lower: &str) -> Result<(), String> {
    let src_maker_path = Path::new("src/maker");
    let manufacturer_file = src_maker_path.join(format!("{}.rs", manufacturer_lower));
    let manufacturer_dir = src_maker_path.join(manufacturer_lower);

    // Check for conflicts and resolve them
    if manufacturer_file.exists() && manufacturer_dir.exists() {
        println!(
            "   ⚠️  Conflict detected: Both {}.rs and {}/ directory exist",
            manufacturer_lower, manufacturer_lower
        );

        // Remove the directory structure in favor of single file approach
        fs::remove_dir_all(&manufacturer_dir)
            .map_err(|e| format!("Failed to remove conflicting directory: {}", e))?;

        println!("   🔧 Resolved: Removed directory, keeping single file approach");
    }

    Ok(())
}

/// Generate parser implementation from proven template pattern
fn generate_parser_from_template(
    manufacturer_lower: &str,
    manufacturer_title: &str,
) -> Result<(), String> {
    let parser_path = Path::new("src/maker").join(format!("{}.rs", manufacturer_lower));

    // Read the Fujifilm parser as our proven template
    let template_path = Path::new("src/maker/fujifilm.rs");
    let template_content = fs::read_to_string(template_path)
        .map_err(|e| format!("Failed to read template parser: {}", e))?;

    // Generate parser content by replacing Fujifilm-specific parts
    let parser_content = template_content
        .replace("Fujifilm", manufacturer_title)
        .replace("fujifilm", manufacturer_lower)
        .replace("FUJIFILM", &manufacturer_title.to_uppercase())
        .replace("FujiFilm.pm", &format!("{}.pm", manufacturer_title))
        .replace(
            "FUJIFILM_",
            &format!("{}_", manufacturer_title.to_uppercase()),
        )
        .replace(
            "b\"FUJIFILM\"",
            &format!("b\"{}\"", manufacturer_title.to_uppercase()),
        );

    fs::write(&parser_path, parser_content)
        .map_err(|e| format!("Failed to write parser file: {}", e))?;

    Ok(())
}

/// Integrate new manufacturer with the maker note system
fn integrate_with_maker_system(
    manufacturer_lower: &str,
    manufacturer_title: &str,
) -> Result<(), String> {
    // Update src/maker/mod.rs
    let mod_file_path = Path::new("src/maker/mod.rs");
    let mod_content = fs::read_to_string(mod_file_path)
        .map_err(|e| format!("Failed to read maker mod.rs: {}", e))?;

    let mut new_content = mod_content;

    // Add module declaration if not already present
    let module_line = format!("pub mod {};", manufacturer_lower);
    if !new_content.contains(&module_line) {
        // Find a good insertion point after existing module declarations
        let last_mod = new_content.rfind("pub mod").unwrap_or(0);
        let insertion_point = new_content[last_mod..]
            .find('\n')
            .map(|pos| last_mod + pos + 1)
            .unwrap_or(new_content.len());
        new_content.insert_str(insertion_point, &format!("{}\n", module_line));
    }

    // Add to Manufacturer enum if not already present
    let enum_variant = format!("    {},", manufacturer_title);
    if !new_content.contains(&enum_variant) {
        // Find the Manufacturer enum and add the variant
        if let Some(enum_start) = new_content.find("pub enum Manufacturer {") {
            let enum_body_start = new_content[enum_start..].find('{').unwrap() + enum_start + 1;
            // Find a good spot to insert (before Unknown variant)
            if let Some(unknown_pos) = new_content[enum_body_start..].find("    Unknown,") {
                let insertion_point = enum_body_start + unknown_pos;
                new_content.insert_str(insertion_point, &format!("{}\n", enum_variant));
            }
        }
    }

    // Add to from_make method if not already present
    let make_check = format!("make_lower.contains(\"{}\") {{", manufacturer_lower);
    if !new_content.contains(&make_check) {
        // Find the from_make method and add the manufacturer detection
        if let Some(from_make_start) = new_content.find("pub fn from_make(make: &str) -> Self {") {
            // Find before the "Unknown" case
            if let Some(unknown_case) =
                new_content[from_make_start..].find("} else {\n            Manufacturer::Unknown")
            {
                let insertion_point = from_make_start + unknown_case;
                let detection_case = format!(
                    " else if {} \n            Manufacturer::{}\n        }}",
                    make_check, manufacturer_title
                );
                new_content.insert_str(insertion_point, &detection_case);
            }
        }
    }

    // Add to parser method if not already present
    let parser_case = format!(
        "Manufacturer::{} => Some(Box::new({}::{}MakerNoteParser)),",
        manufacturer_title, manufacturer_lower, manufacturer_title
    );
    if !new_content.contains(&parser_case) {
        // Find the parser method and add the case
        if let Some(parser_start) =
            new_content.find("pub fn parser(&self) -> Option<Box<dyn MakerNoteParser>> {")
        {
            // Find before the "_ => None," case
            if let Some(default_case) = new_content[parser_start..].find(
                "            // Other manufacturers not implemented yet\n            _ => None,",
            ) {
                let insertion_point = parser_start + default_case;
                new_content.insert_str(insertion_point, &format!("            {} \n", parser_case));
            }
        }
    }

    fs::write(mod_file_path, new_content)
        .map_err(|e| format!("Failed to update maker mod.rs: {}", e))?;

    // Update src/tables/mod.rs
    let tables_mod_path = Path::new("src/tables/mod.rs");
    let tables_content = fs::read_to_string(tables_mod_path)
        .map_err(|e| format!("Failed to read tables mod.rs: {}", e))?;

    let table_module_line = format!("pub mod {}_tags;", manufacturer_lower);

    if !tables_content.contains(&table_module_line) {
        let mut new_tables_content = tables_content;
        let insertion_point = new_tables_content.find("pub mod").unwrap_or(0);
        new_tables_content.insert_str(insertion_point, &format!("{}\n", table_module_line));

        fs::write(tables_mod_path, new_tables_content)
            .map_err(|e| format!("Failed to update tables mod.rs: {}", e))?;
    }

    Ok(())
}

/// Generate basic tests for the new manufacturer
fn generate_basic_tests(_manufacturer_lower: &str, manufacturer_title: &str) -> Result<(), String> {
    // Tests are included in the generated parser file from the template
    // The Fujifilm template already contains the necessary test structure
    // No additional test generation needed since template includes comprehensive tests

    println!(
        "   📝 Tests included in generated parser ({}MakerNoteParser tests)",
        manufacturer_title
    );

    Ok(())
}

/// Run validation pipeline to ensure implementation works correctly
fn run_validation_pipeline(
    manufacturer_lower: &str,
    manufacturer_title: &str,
) -> Result<(), String> {
    use std::process::Command;

    println!("   🔍 Running validation pipeline...");

    // Step 1: Check compilation
    println!("   📦 Testing compilation...");
    let build_result = Command::new("cargo")
        .args(["check", "--quiet"])
        .output()
        .map_err(|e| format!("Failed to run cargo check: {}", e))?;

    if !build_result.status.success() {
        let error_output = String::from_utf8_lossy(&build_result.stderr);
        return Err(format!("Compilation failed:\n{}", error_output));
    }
    println!("   ✅ Compilation successful");

    // Step 2: Test basic functionality
    println!("   🧪 Testing basic functionality...");
    let test_result = Command::new("cargo")
        .args(["test", &format!("{}::tests", manufacturer_lower), "--quiet"])
        .output()
        .map_err(|e| format!("Failed to run basic tests: {}", e))?;

    if !test_result.status.success() {
        let error_output = String::from_utf8_lossy(&test_result.stderr);
        return Err(format!("Basic tests failed:\n{}", error_output));
    }
    println!("   ✅ Basic tests passed");

    // Step 3: Verify integration
    println!("   🔗 Verifying integration...");
    verify_integration(manufacturer_lower, manufacturer_title)?;
    println!("   ✅ Integration verified");

    // Step 4: Check generated files
    println!("   📄 Validating generated files...");
    validate_generated_files(manufacturer_lower, manufacturer_title)?;
    println!("   ✅ Generated files validated");

    Ok(())
}

/// Verify that the manufacturer is properly integrated into the system
fn verify_integration(manufacturer_lower: &str, manufacturer_title: &str) -> Result<(), String> {
    // Check that parser file exists and compiles
    let parser_path = Path::new("src/maker").join(format!("{}.rs", manufacturer_lower));
    if !parser_path.exists() {
        return Err(format!("Parser file not found: {}", parser_path.display()));
    }

    // Check that tables file exists
    let tables_path = Path::new("src/tables").join(format!("{}_tags.rs", manufacturer_lower));
    if !tables_path.exists() {
        return Err(format!("Tables file not found: {}", tables_path.display()));
    }

    // Check that detection file exists
    let detection_path = Path::new("src/maker")
        .join(manufacturer_lower)
        .join("detection.rs");
    if !detection_path.exists() {
        return Err(format!(
            "Detection file not found: {}",
            detection_path.display()
        ));
    }

    // Verify module integration
    let maker_mod_path = Path::new("src/maker/mod.rs");
    let maker_mod_content = fs::read_to_string(maker_mod_path)
        .map_err(|e| format!("Failed to read maker mod.rs: {}", e))?;

    if !maker_mod_content.contains(&format!("pub mod {};", manufacturer_lower)) {
        return Err("Module not properly declared in maker/mod.rs".to_string());
    }

    // Check for enum-based integration (current system)
    if !maker_mod_content.contains(&format!("Manufacturer::{}", manufacturer_title)) {
        return Err("Parser not properly registered in Manufacturer enum".to_string());
    }

    // Verify tables integration
    let tables_mod_path = Path::new("src/tables/mod.rs");
    let tables_mod_content = fs::read_to_string(tables_mod_path)
        .map_err(|e| format!("Failed to read tables mod.rs: {}", e))?;

    if !tables_mod_content.contains(&format!("pub mod {}_tags;", manufacturer_lower)) {
        return Err("Tables module not properly declared in tables/mod.rs".to_string());
    }

    Ok(())
}

/// Validate that all generated files contain expected content
fn validate_generated_files(
    manufacturer_lower: &str,
    manufacturer_title: &str,
) -> Result<(), String> {
    // Validate parser file structure
    let parser_path = Path::new("src/maker").join(format!("{}.rs", manufacturer_lower));
    let parser_content = fs::read_to_string(&parser_path)
        .map_err(|e| format!("Failed to read parser file: {}", e))?;

    // Check for essential components
    let required_components = [
        &format!("pub struct {}MakerNoteParser", manufacturer_title),
        "impl MakerNoteParser for",
        "fn parse(",
        "fn manufacturer(",
        "#[cfg(test)]",
        "mod tests",
    ];

    for component in &required_components {
        if !parser_content.contains(component) {
            return Err(format!(
                "Parser file missing required component: {}",
                component
            ));
        }
    }

    // Validate tables file structure
    let tables_path = Path::new("src/tables").join(format!("{}_tags.rs", manufacturer_lower));
    let tables_content = fs::read_to_string(&tables_path)
        .map_err(|e| format!("Failed to read tables file: {}", e))?;

    let required_table_components = [
        &format!("pub struct {}Tag", manufacturer_title.to_uppercase()),
        &format!("pub const {}_TAGS", manufacturer_title.to_uppercase()),
        &format!("pub fn get_{}_tag", manufacturer_lower),
        "use crate::core::print_conv::PrintConvId",
    ];

    for component in &required_table_components {
        if !tables_content.contains(component) {
            return Err(format!(
                "Tables file missing required component: {}",
                component
            ));
        }
    }

    // Validate detection file structure
    let detection_path = Path::new("src/maker")
        .join(manufacturer_lower)
        .join("detection.rs");
    let detection_content = fs::read_to_string(&detection_path)
        .map_err(|e| format!("Failed to read detection file: {}", e))?;

    let required_detection_components = [
        "AUTO-GENERATED by exiftool_sync extract maker-detection",
        &format!(
            "pub struct {}DetectionResult",
            manufacturer_title.to_uppercase()
        ),
        &format!("pub fn detect_{}_maker_note", manufacturer_lower),
        "#[cfg(test)]",
    ];

    for component in &required_detection_components {
        if !detection_content.contains(component) {
            return Err(format!(
                "Detection file missing required component: {}",
                component
            ));
        }
    }

    Ok(())
}
