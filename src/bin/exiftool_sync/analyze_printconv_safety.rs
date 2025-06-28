//! PrintConv Safety Analysis Module
//!
//! Analyzes ExifTool tag definitions to identify safe universal PrintConv patterns
//! and detect potential name collisions across different contexts (EXIF/MakerNote/XMP).
//!
//! This module uses Perl introspection (via scripts/analyze_printconv_safety.pl) to extract
//! actual PrintConv data from ExifTool modules, ensuring accurate analysis.

use crate::extractors::{emit_sync_issue, PerlSource, Priority, SyncIssue};
use crate::tag_metadata::TagMetadata;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;

#[derive(Debug)]
pub struct Args {
    pub output: String,
    pub verbose: bool,
    pub exiftool_path: String,
}

/// Parse command-line style arguments into our Args struct
pub fn parse_args(args: &[String]) -> Result<Args, String> {
    let mut output = "printconv_safety_analysis.csv".to_string();
    let mut exiftool_path = "third-party/exiftool".to_string();
    let mut verbose = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--output" => {
                if i + 1 < args.len() {
                    output = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--output requires a filename".to_string());
                }
            }
            "--exiftool-path" => {
                if i + 1 < args.len() {
                    exiftool_path = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--exiftool-path requires a path".to_string());
                }
            }
            "--verbose" => {
                verbose = true;
                i += 1;
            }
            "--help" => {
                print_help();
                std::process::exit(0);
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }

    Ok(Args {
        output,
        verbose,
        exiftool_path,
    })
}

fn print_help() {
    println!("exiftool_sync analyze printconv-safety [OPTIONS]");
    println!();
    println!("Analyzes ExifTool tag definitions to identify safe universal PrintConv patterns");
    println!(
        "and detect potential name collisions across different contexts (EXIF/MakerNote/XMP)."
    );
    println!();
    println!("OPTIONS:");
    println!("    --output <FILE>        Output CSV file [default: printconv_safety_analysis.csv]");
    println!("    --exiftool-path <PATH> Path to ExifTool source [default: third-party/exiftool]");
    println!("    --verbose              Enable verbose output");
    println!("    --help                 Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    # Basic analysis with default settings");
    println!("    cargo run --bin exiftool_sync analyze printconv-safety");
    println!();
    println!("    # Custom output file and verbose logging");
    println!("    cargo run --bin exiftool_sync analyze printconv-safety --output my_report.csv --verbose");
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TagAnalysis {
    tag_name: String,
    tag_id: String,
    #[serde(default)]
    module: String,
    #[serde(default)]
    table_name: String,
    context: String,              // EXIF, MakerNote, XMP, etc.
    manufacturer: Option<String>, // Canon, Nikon, etc. for MakerNotes
    #[serde(default)]
    printconv_type: String,
    printconv_signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    printconv_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    printconv_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    printconv_function: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resolved_signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resolved_entry_count: Option<usize>,
    group_id: String,
    safety_level: String,
    recommended_printconv_id: String,
    #[serde(default)]
    collision_details: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CollisionGroup {
    tags: Vec<TagAnalysis>,
    signatures: HashMap<String, Vec<TagAnalysis>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SafeGroup {
    tags: Vec<TagAnalysis>,
    signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SharedLookup {
    module: String,
    entry_count: usize,
    signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnalysisOutput {
    metadata: AnalysisMetadata,
    statistics: AnalysisStats,
    tags: Vec<TagAnalysis>,
    collision_groups: HashMap<String, CollisionGroup>,
    safe_groups: HashMap<String, SafeGroup>,
    shared_lookups: HashMap<String, SharedLookup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnalysisMetadata {
    extraction_date: String,
    exiftool_version: String,
    modules_analyzed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnalysisStats {
    total_modules: usize,
    total_tags: usize,
    unique_tag_names: usize,
    safe_universal: usize,
    collision_risks: usize,
    unique_context: usize,
    tags_with_printconv: usize,
    shared_lookups: usize,
    printconv_types: HashMap<String, usize>,
}

// TagMetadataEntry is now imported from crate::tag_metadata

#[derive(Debug)]
struct PrintConvAnalyzer {
    verbose: bool,
    tag_metadata: TagMetadata,
}

impl PrintConvAnalyzer {
    fn new(verbose: bool) -> Self {
        // Load TagMetadata.json using the new module
        let tag_metadata = TagMetadata::new().unwrap_or_else(|e| {
            if verbose {
                eprintln!("Warning: Failed to load TagMetadata.json: {}", e);
            }
            TagMetadata::empty()
        });

        Self {
            verbose,
            tag_metadata,
        }
    }

    fn get_priority(&self, tag_name: &str) -> Priority {
        self.tag_metadata.get_priority(tag_name)
    }

    /// Run the Perl script to extract PrintConv data
    fn extract_printconv_data(&self) -> Result<AnalysisOutput, Box<dyn std::error::Error>> {
        let perl_script = Path::new("scripts/analyze_printconv_safety.pl");

        if !perl_script.exists() {
            return Err(format!("Perl script not found: {}", perl_script.display()).into());
        }

        if self.verbose {
            println!("Running Perl script to analyze PrintConv safety...");
        }

        let mut cmd = Command::new("perl");
        cmd.arg(perl_script);

        if self.verbose {
            cmd.arg("--verbose");
        }

        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Perl script failed: {}", stderr).into());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let analysis: AnalysisOutput = serde_json::from_str(&stdout)
            .map_err(|e| format!("Failed to parse Perl script output: {}", e))?;

        Ok(analysis)
    }

    /// Emit sync issues for PrintConv problems
    fn emit_sync_issues(
        &self,
        analysis: &AnalysisOutput,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Emit issues for collision risks
        for (tag_name, collision_group) in &analysis.collision_groups {
            if let Some(first_tag) = collision_group.tags.first() {
                // Use priority based on TagMetadata.json
                let priority = self.get_priority(tag_name);

                // Deduplicate the recommended PrintConvId variants
                let unique_variants: HashSet<&str> = collision_group
                    .tags
                    .iter()
                    .map(|t| t.recommended_printconv_id.as_str())
                    .collect();

                emit_sync_issue(SyncIssue {
                    priority,
                    command: "analyze-printconv-safety".to_string(),
                    perl_source: PerlSource {
                        file: format!("lib/Image/ExifTool/{}.pm", first_tag.module.replace("Image::ExifTool::", "")),
                        lines: None,
                    },
                    rust_target: Some("src/core/print_conv.rs".to_string()),
                    description: format!(
                        "PrintConv collision for tag '{}' - {} different implementations across contexts",
                        tag_name,
                        collision_group.signatures.len()
                    ),
                    suggested_implementation: format!(
                        "Create context-specific PrintConvId variants: {}",
                        unique_variants.into_iter().collect::<Vec<_>>().join(", ")
                    ),
                })?;
            }
        }

        // Emit issues for missing PrintConv implementations
        let missing_printconv: Vec<&TagAnalysis> = analysis
            .tags
            .iter()
            .filter(|t| {
                t.printconv_type != "none"
                    && t.printconv_type != "hash_ref"
                    && t.safety_level == "Safe"
            })
            .collect();

        for tag in missing_printconv.iter().take(10) {
            // Limit to first 10 to avoid spam
            // Use priority based on TagMetadata.json
            let priority = self.get_priority(&tag.tag_name);

            emit_sync_issue(SyncIssue {
                priority,
                command: "analyze-printconv-safety".to_string(),
                perl_source: PerlSource {
                    file: format!(
                        "lib/Image/ExifTool/{}.pm",
                        tag.module.replace("Image::ExifTool::", "")
                    ),
                    lines: None,
                },
                rust_target: Some("src/core/print_conv.rs".to_string()),
                description: format!(
                    "Missing PrintConv implementation for {} tag '{}'",
                    tag.context, tag.tag_name
                ),
                suggested_implementation: match tag.printconv_type.as_str() {
                    "string" => format!(
                        "Add pattern '{}' to PERL_STRING_PATTERNS",
                        tag.printconv_source.as_ref().unwrap_or(&"".to_string())
                    ),
                    "hash" => format!(
                        "Add normalized pattern to HASH_PATTERNS for {}",
                        tag.tag_name
                    ),
                    "code_ref" | "sub_ref" => format!(
                        "Implement {} function in apply_print_conv()",
                        tag.printconv_function.as_ref().unwrap_or(&tag.tag_name)
                    ),
                    _ => format!(
                        "Analyze PrintConv type '{}' and implement",
                        tag.printconv_type
                    ),
                },
            })?;
        }

        Ok(())
    }

    /// Generate CSV report
    fn generate_report(
        &self,
        analysis: &AnalysisOutput,
        output_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut wtr = csv::Writer::from_path(output_path)?;

        // Write header
        wtr.write_record([
            "tag_name",
            "tag_id",
            "module",
            "table_name",
            "context",
            "manufacturer",
            "printconv_type",
            "printconv_signature",
            "group_id",
            "safety_level",
            "recommended_printconv_id",
            "collision_count",
            "collision_details",
        ])?;

        // Write data rows
        for tag in &analysis.tags {
            wtr.write_record([
                &tag.tag_name,
                &tag.tag_id,
                &tag.module,
                &tag.table_name,
                &tag.context,
                tag.manufacturer.as_deref().unwrap_or(""),
                &tag.printconv_type,
                &tag.printconv_signature,
                &tag.group_id,
                &tag.safety_level,
                &tag.recommended_printconv_id,
                &tag.collision_details.len().to_string(),
                &tag.collision_details.join(" | "),
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }

    /// Print summary statistics
    fn print_summary(&self, analysis: &AnalysisOutput) {
        let stats = &analysis.statistics;

        println!("\n=== PrintConv Safety Analysis Summary ===");
        println!("Total modules analyzed: {}", stats.total_modules);
        println!("Total tags analyzed: {}", stats.total_tags);
        println!("Unique tag names: {}", stats.unique_tag_names);
        println!(
            "Safe for universal inference: {} ({:.1}%)",
            stats.safe_universal,
            stats.safe_universal as f64 / stats.total_tags as f64 * 100.0
        );
        println!(
            "Name collision risks: {} ({:.1}%)",
            stats.collision_risks,
            stats.collision_risks as f64 / stats.total_tags as f64 * 100.0
        );
        println!(
            "Unique context (safe): {} ({:.1}%)",
            stats.unique_context,
            stats.unique_context as f64 / stats.total_tags as f64 * 100.0
        );
        println!(
            "Tags with PrintConv: {} ({:.1}%)",
            stats.tags_with_printconv,
            stats.tags_with_printconv as f64 / stats.total_tags as f64 * 100.0
        );
        println!("Shared lookup tables: {}", stats.shared_lookups);

        // Print PrintConv type breakdown
        println!("\nPrintConv type breakdown:");
        for (pc_type, count) in &stats.printconv_types {
            println!("  {}: {}", pc_type, count);
        }

        // Show some safe universal patterns
        if !analysis.safe_groups.is_empty() {
            println!("\nSafe universal patterns found:");
            for (tag_name, _) in analysis.safe_groups.iter().take(10) {
                println!("  {}", tag_name);
            }
            if analysis.safe_groups.len() > 10 {
                println!("  ... and {} more", analysis.safe_groups.len() - 10);
            }
        }

        // Show collision risks
        if !analysis.collision_groups.is_empty() {
            println!("\nName collision risks found:");
            for (tag_name, collision_group) in analysis.collision_groups.iter().take(10) {
                println!(
                    "  {} ({} different implementations)",
                    tag_name,
                    collision_group.signatures.len()
                );
                for tag in collision_group.tags.iter().take(3) {
                    println!(
                        "    - {}: {} ({})",
                        tag.context,
                        tag.manufacturer.as_deref().unwrap_or(""),
                        tag.printconv_signature
                    );
                }
                if collision_group.tags.len() > 3 {
                    println!("    ... and {} more", collision_group.tags.len() - 3);
                }
            }
            if analysis.collision_groups.len() > 10 {
                println!(
                    "  ... and {} more collision groups",
                    analysis.collision_groups.len() - 10
                );
            }
        }
    }
}

/// Run the PrintConv safety analysis
pub fn run(args: Args) -> Result<(), String> {
    println!("Analyzing PrintConv safety across ExifTool implementations...");
    println!("ExifTool path: {}", args.exiftool_path);
    println!("Output file: {}", args.output);

    let analyzer = PrintConvAnalyzer::new(args.verbose);

    // Extract all tags using Perl script
    let analysis = analyzer
        .extract_printconv_data()
        .map_err(|e| format!("Failed to extract PrintConv data: {}", e))?;
    println!(
        "Extracted {} tag definitions from {} modules",
        analysis.statistics.total_tags, analysis.statistics.total_modules
    );

    // Emit sync issues for problems found
    analyzer
        .emit_sync_issues(&analysis)
        .map_err(|e| format!("Failed to emit sync issues: {}", e))?;

    // Generate CSV report
    analyzer
        .generate_report(&analysis, &args.output)
        .map_err(|e| format!("Failed to generate report: {}", e))?;
    println!("Report written to: {}", args.output);

    // Print summary
    analyzer.print_summary(&analysis);

    println!("\n✅ PrintConv safety analysis completed successfully!");
    println!("📊 Report written to: {}", args.output);
    println!("\n💡 Use this data to identify safe universal patterns for automatic inference.");
    println!("   See doc/SYNC-PRINTCONV-DESIGN.md for implementation guidance.");

    Ok(())
}
