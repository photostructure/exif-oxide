//! ExifTool synchronization tool
//!
//! Tool to synchronize exif-oxide with ExifTool updates and extract algorithms

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

mod extractors;

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

fn print_help() {
    println!("ExifTool Synchronization Tool");
    println!();
    println!("USAGE:");
    println!("    cargo run --bin exiftool_sync <COMMAND>");
    println!();
    println!("COMMANDS:");
    println!("    status                   Show current synchronization status");
    println!("    diff <from> <to>         Show which Rust files are affected by ExifTool changes");
    println!("    scan                     List all ExifTool source dependencies");
    println!("    extract <component>      Extract algorithms from ExifTool source");
    println!("    extract-all              Extract all components in one command");
    println!("    help                     Show this help message");
    println!();
    println!("EXTRACT COMPONENTS:");
    println!("    binary-formats           Extract ProcessBinaryData table definitions");
    println!("    magic-numbers            Extract file type detection patterns");
    println!("    datetime-patterns        Extract date parsing patterns");
    println!("    binary-tags              Extract composite tag definitions");
    println!("    maker-detection          Extract maker note detection patterns");
    println!();
    println!("EXAMPLES:");
    println!("    cargo run --bin exiftool_sync status");
    println!("    cargo run --bin exiftool_sync diff 12.65 12.66");
    println!("    cargo run --bin exiftool_sync scan");
    println!("    cargo run --bin exiftool_sync extract binary-formats");
    println!("    cargo run --bin exiftool_sync extract-all");
    println!("    cargo run --bin exiftool_sync extract maker-detection");
}
