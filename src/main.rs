use clap::{Arg, Command};
use std::path::PathBuf;

// Import our library modules
use exif_oxide::formats::extract_metadata;

/// Main CLI application for exif-oxide
///
/// This is the entry point that matches ExifTool's usage:
/// exif-oxide image.jpg
/// exif-oxide image1.jpg image2.jpg image3.jpg
/// exif-oxide --show-missing *.jpg
fn main() {
    // Build CLI interface using clap
    // Clap is Rust's most popular CLI argument parsing library
    let matches = Command::new("exif-oxide")
        .version("0.1.0")
        .author("exif-oxide@photostructure.com")
        .about("High-performance Rust implementation of ExifTool")
        .arg(
            Arg::new("files")
                .help("Image files to process")
                .required(true)
                .value_name("FILE")
                .num_args(1..), // Accept one or more files
        )
        .arg(
            Arg::new("show-missing")
                .long("show-missing")
                .help("Show unimplemented features for development")
                .action(clap::ArgAction::SetTrue), // Boolean flag
        )
        .get_matches();

    // Extract arguments - Rust's type system ensures these are safe
    let file_paths: Vec<&String> = matches.get_many::<String>("files").unwrap().collect();
    let show_missing = matches.get_flag("show-missing");

    // Convert strings to PathBufs for proper file handling
    let paths: Vec<PathBuf> = file_paths.iter().map(PathBuf::from).collect();

    // Process all files - this will output a JSON array like ExifTool
    match process_files(&paths, show_missing) {
        Ok(()) => {
            // Success - output has already been printed
        }
        Err(e) => {
            // Rust error handling - print to stderr and exit with error code
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
fn process_files(paths: &[PathBuf], show_missing: bool) -> Result<(), Box<dyn std::error::Error>> {
    use exif_oxide::types::ExifData;

    let mut results = Vec::new();

    // Process each file
    for path in paths {
        match process_single_file(path, show_missing) {
            Ok(metadata) => {
                results.push(metadata);
            }
            Err(e) => {
                // ExifTool continues processing other files on error
                // Create error entry similar to ExifTool's behavior
                let error_metadata = ExifData {
                    source_file: path.to_string_lossy().to_string(),
                    exif_tool_version: "0.1.0-oxide".to_string(),
                    tags: std::collections::HashMap::new(),
                    errors: vec![format!("Error processing file: {e}")],
                    missing_implementations: None,
                };
                results.push(error_metadata);
            }
        }
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
) -> Result<exif_oxide::types::ExifData, Box<dyn std::error::Error>> {
    // Verify file exists
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()).into());
    }

    // Extract metadata using our library
    let metadata = extract_metadata(path, show_missing)?;

    Ok(metadata)
}
