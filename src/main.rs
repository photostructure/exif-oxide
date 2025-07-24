use clap::{Arg, Command};
use std::collections::HashSet;
use std::path::PathBuf;
use tracing::{debug, error, info};

// Import our library modules
use exif_oxide::formats::extract_metadata;

/// Parse -TagName# syntax to extract tag name
/// ExifTool uses -TagName# to show numeric value instead of PrintConv
fn parse_numeric_tag(arg: &str) -> Result<String, String> {
    if arg.starts_with('-') && arg.ends_with('#') && arg.len() > 2 {
        // Extract tag name between - and #
        Ok(arg[1..arg.len() - 1].to_string())
    } else if !arg.starts_with('-') {
        // Plain tag name without -# syntax
        Ok(arg.to_string())
    } else {
        Err(format!("Invalid numeric tag format: {arg}"))
    }
}

/// Separate mixed CLI arguments into files and numeric tags
/// Returns (file_paths, numeric_tags)
fn parse_mixed_args(args: Vec<&String>) -> (Vec<&String>, HashSet<String>) {
    let mut file_paths = Vec::new();
    let mut numeric_tags = HashSet::new();

    for arg in args {
        if arg.starts_with('-') && arg.ends_with('#') && arg.len() > 2 {
            // This is a numeric tag like -FNumber#
            match parse_numeric_tag(arg) {
                Ok(tag_name) => {
                    numeric_tags.insert(tag_name);
                }
                Err(e) => {
                    eprintln!("Warning: {e}");
                }
            }
        } else if !arg.starts_with('-') || arg == "-" || arg == "--" {
            // This is a file path (not starting with -, or is - or --)
            file_paths.push(arg);
        } else {
            // Unrecognized argument format
            eprintln!("Warning: Unrecognized argument: {arg}");
            file_paths.push(arg); // Treat as file path
        }
    }

    (file_paths, numeric_tags)
}

/// Main CLI application for exif-oxide
///
/// This is the entry point that matches ExifTool's usage:
/// exif-oxide image.jpg
/// exif-oxide image1.jpg image2.jpg image3.jpg
/// exif-oxide --show-missing *.jpg
fn main() {
    // Initialize tracing subscriber for structured logging
    // Use environment variable RUST_LOG to control logging level (e.g., RUST_LOG=debug)
    // Ensure all log output goes to stderr, not stdout, so JSON output is clean
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    info!("Starting exif-oxide");

    // Build CLI interface using clap
    // Clap is Rust's most popular CLI argument parsing library
    let matches = Command::new("exif-oxide")
        .version("0.1.0")
        .author("exif-oxide@photostructure.com")
        .about("High-performance Rust implementation of ExifTool")
        .arg(
            Arg::new("args")
                .help("Image files and/or -TagName# flags")
                .value_name("ARG")
                .num_args(1..) // Accept one or more arguments
                .allow_hyphen_values(true) // Allow -TagName# format
                .required(true)
                .trailing_var_arg(true), // Allow mixed positional arguments
        )
        .arg(
            Arg::new("show-missing")
                .long("show-missing")
                .help("Show unimplemented features for development")
                .action(clap::ArgAction::SetTrue), // Boolean flag
        )
        .arg(
            Arg::new("warnings")
                .long("warnings")
                .help("Include parsing warnings in output (suppressed by default)")
                .action(clap::ArgAction::SetTrue), // Boolean flag
        )
        .get_matches();

    // Extract all arguments and separate them into files and numeric tags
    let args: Vec<&String> = matches.get_many::<String>("args").unwrap().collect();
    let show_missing = matches.get_flag("show-missing");
    let show_warnings = matches.get_flag("warnings");

    // Separate arguments into files and numeric tags based on their format
    let (file_paths, numeric_tags) = parse_mixed_args(args);

    // Validate we have at least one file
    if file_paths.is_empty() {
        eprintln!("Error: No files specified");
        std::process::exit(1);
    }

    // Convert strings to PathBufs for proper file handling
    let paths: Vec<PathBuf> = file_paths.iter().map(PathBuf::from).collect();

    debug!("Processing {} files", paths.len());
    debug!("Show missing implementations: {}", show_missing);
    debug!("Show warnings: {}", show_warnings);
    debug!("Numeric tags: {:?}", numeric_tags);

    // Process all files - this will output a JSON array like ExifTool
    match process_files(&paths, show_missing, show_warnings, numeric_tags) {
        Ok(()) => {
            // Success - output has already been printed
        }
        Err(e) => {
            // Rust error handling - print to stderr and exit with error code
            error!("Fatal error: {}", e);
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

/// Process multiple image files and output JSON array
///
/// This function matches ExifTool's behavior of outputting a JSON array
/// containing one object per file, even for a single file.
/// Result<T, E> means either Ok(T) for success or Err(E) for errors.
fn process_files(
    paths: &[PathBuf],
    show_missing: bool,
    show_warnings: bool,
    numeric_tags: HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    use exif_oxide::types::ExifData;

    let mut results = Vec::new();

    // Process each file
    for path in paths {
        debug!("Processing file: {}", path.display());
        match process_single_file(path, show_missing, show_warnings) {
            Ok(metadata) => {
                info!("Successfully processed: {}", path.display());
                results.push(metadata);
            }
            Err(e) => {
                // ExifTool continues processing other files on error
                // Create error entry similar to ExifTool's behavior
                error!("Failed to process {}: {}", path.display(), e);
                let error_metadata = ExifData {
                    source_file: path.to_string_lossy().to_string(),
                    exif_tool_version: "0.1.0-oxide".to_string(),
                    tags: vec![],
                    legacy_tags: indexmap::IndexMap::new(),
                    errors: vec![format!("Error processing file: {e}")],
                    missing_implementations: None,
                };
                results.push(error_metadata);
            }
        }
    }

    // Prepare for serialization by converting tags to legacy format
    // Pass numeric_tags to determine which tags should use numeric values
    let numeric_tags_ref = if numeric_tags.is_empty() {
        None
    } else {
        Some(&numeric_tags)
    };

    for result in &mut results {
        result.prepare_for_serialization(numeric_tags_ref);
    }

    // Output as JSON array matching ExifTool format
    println!("{}", serde_json::to_string_pretty(&results)?);

    Ok(())
}

/// Process a single image file and return metadata
///
/// This function extracts metadata from one file and returns it,
/// allowing the caller to handle multiple files and error aggregation.
fn process_single_file(
    path: &std::path::Path,
    show_missing: bool,
    show_warnings: bool,
) -> Result<exif_oxide::types::ExifData, Box<dyn std::error::Error>> {
    // Verify file exists
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()).into());
    }

    // Extract metadata using our library
    let metadata = extract_metadata(path, show_missing, show_warnings)?;

    Ok(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_numeric_tag() {
        // Valid numeric tag formats
        assert_eq!(parse_numeric_tag("-FNumber#").unwrap(), "FNumber");
        assert_eq!(parse_numeric_tag("-ExposureTime#").unwrap(), "ExposureTime");
        assert_eq!(parse_numeric_tag("-ISO#").unwrap(), "ISO");
        assert_eq!(parse_numeric_tag("-LongTagName#").unwrap(), "LongTagName");

        // Plain tag names (without -#)
        assert_eq!(parse_numeric_tag("FNumber").unwrap(), "FNumber");
        assert_eq!(parse_numeric_tag("ExposureTime").unwrap(), "ExposureTime");

        // Invalid formats
        assert!(parse_numeric_tag("-#").is_err());
        assert!(parse_numeric_tag("-").is_err());
        assert!(parse_numeric_tag("-Tag").is_err());
        assert!(parse_numeric_tag("-Tag#Extra").is_err());
    }

    #[test]
    fn test_parse_mixed_args_files_before_tags() {
        let image1 = "image1.jpg".to_string();
        let image2 = "image2.png".to_string();
        let fnumber = "-FNumber#".to_string();
        let exposure = "-ExposureTime#".to_string();
        let args = vec![&image1, &image2, &fnumber, &exposure];

        let (files, tags) = parse_mixed_args(args);

        assert_eq!(files, vec!["image1.jpg", "image2.png"]);
        assert!(tags.contains("FNumber"));
        assert!(tags.contains("ExposureTime"));
        assert_eq!(tags.len(), 2);
    }

    #[test]
    fn test_parse_mixed_args_tags_before_files() {
        let fnumber = "-FNumber#".to_string();
        let exposure = "-ExposureTime#".to_string();
        let image1 = "image1.jpg".to_string();
        let image2 = "image2.png".to_string();
        let args = vec![&fnumber, &exposure, &image1, &image2];

        let (files, tags) = parse_mixed_args(args);

        assert_eq!(files, vec!["image1.jpg", "image2.png"]);
        assert!(tags.contains("FNumber"));
        assert!(tags.contains("ExposureTime"));
        assert_eq!(tags.len(), 2);
    }

    #[test]
    fn test_parse_mixed_args_interleaved() {
        let fnumber = "-FNumber#".to_string();
        let image1 = "image1.jpg".to_string();
        let exposure = "-ExposureTime#".to_string();
        let image2 = "image2.png".to_string();
        let iso = "-ISO#".to_string();
        let args = vec![&fnumber, &image1, &exposure, &image2, &iso];

        let (files, tags) = parse_mixed_args(args);

        assert_eq!(files, vec!["image1.jpg", "image2.png"]);
        assert!(tags.contains("FNumber"));
        assert!(tags.contains("ExposureTime"));
        assert!(tags.contains("ISO"));
        assert_eq!(tags.len(), 3);
    }

    #[test]
    fn test_parse_mixed_args_edge_cases() {
        // Test with stdin marker "-"
        let dash = "-".to_string();
        let fnumber = "-FNumber#".to_string();
        let args = vec![&dash, &fnumber];
        let (files, tags) = parse_mixed_args(args);
        assert_eq!(files, vec!["-"]);
        assert!(tags.contains("FNumber"));

        // Test with stdin marker "--"
        let ddash = "--".to_string();
        let fnumber2 = "-FNumber#".to_string();
        let args = vec![&ddash, &fnumber2];
        let (files, tags) = parse_mixed_args(args);
        assert_eq!(files, vec!["--"]);
        assert!(tags.contains("FNumber"));

        // Test with files that start with hyphen (treated as unrecognized)
        let weird = "-weirdfile.jpg".to_string();
        let fnumber3 = "-FNumber#".to_string();
        let args = vec![&weird, &fnumber3];
        let (files, tags) = parse_mixed_args(args);
        assert_eq!(files, vec!["-weirdfile.jpg"]); // Treated as file
        assert!(tags.contains("FNumber"));
    }

    #[test]
    fn test_parse_mixed_args_no_tags() {
        let image1 = "image1.jpg".to_string();
        let image2 = "image2.png".to_string();
        let image3 = "image3.tiff".to_string();
        let args = vec![&image1, &image2, &image3];

        let (files, tags) = parse_mixed_args(args);

        assert_eq!(files, vec!["image1.jpg", "image2.png", "image3.tiff"]);
        assert!(tags.is_empty());
    }

    #[test]
    fn test_parse_mixed_args_no_files() {
        let fnumber = "-FNumber#".to_string();
        let exposure = "-ExposureTime#".to_string();
        let iso = "-ISO#".to_string();
        let args = vec![&fnumber, &exposure, &iso];

        let (files, tags) = parse_mixed_args(args);

        assert!(files.is_empty());
        assert!(tags.contains("FNumber"));
        assert!(tags.contains("ExposureTime"));
        assert!(tags.contains("ISO"));
        assert_eq!(tags.len(), 3);
    }

    #[test]
    fn test_parse_mixed_args_duplicate_tags() {
        let image = "image.jpg".to_string();
        let fnumber1 = "-FNumber#".to_string();
        let fnumber2 = "-FNumber#".to_string();
        let exposure = "-ExposureTime#".to_string();
        let args = vec![&image, &fnumber1, &fnumber2, &exposure];

        let (files, tags) = parse_mixed_args(args);

        assert_eq!(files, vec!["image.jpg"]);
        assert!(tags.contains("FNumber"));
        assert!(tags.contains("ExposureTime"));
        assert_eq!(tags.len(), 2); // Set removes duplicates
    }
}
