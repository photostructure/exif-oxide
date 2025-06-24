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
    println!("    help                             Show this help message");
    println!();
    println!("EXTRACT COMPONENTS:");
    println!("    binary-formats                   Extract ProcessBinaryData table definitions");
    println!("    magic-numbers                    Extract file type detection patterns");
    println!("    datetime-patterns                Extract date parsing patterns");
    println!("    binary-tags                      Extract composite tag definitions");
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
}
