use clap::{Arg, Command};
use std::path::PathBuf;

// Import our library modules
use exif_oxide::formats::extract_metadata;

/// Main CLI application for exif-oxide
///
/// This is the entry point that matches ExifTool's basic usage:
/// exif-oxide image.jpg
/// exif-oxide --show-missing image.jpg
fn main() {
    // Build CLI interface using clap
    // Clap is Rust's most popular CLI argument parsing library
    let matches = Command::new("exif-oxide")
        .version("0.1.0")
        .author("exif-oxide@photostructure.com")
        .about("High-performance Rust implementation of ExifTool")
        .arg(
            Arg::new("file")
                .help("Image file to process")
                .required(true)
                .value_name("FILE")
                .index(1), // Positional argument
        )
        .arg(
            Arg::new("show-missing")
                .long("show-missing")
                .help("Show unimplemented features for development")
                .action(clap::ArgAction::SetTrue), // Boolean flag
        )
        .get_matches();

    // Extract arguments - Rust's type system ensures these are safe
    let file_path = matches.get_one::<String>("file").unwrap(); // Safe because required=true
    let show_missing = matches.get_flag("show-missing");

    // Convert string to PathBuf for proper file handling
    let path = PathBuf::from(file_path);

    // Process the file - this will be our main processing function
    match process_image(&path, show_missing) {
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

/// Process an image file and output JSON
///
/// This function demonstrates Rust's Result type for error handling.
/// Result<T, E> means either Ok(T) for success or Err(E) for errors.
fn process_image(
    path: &std::path::Path,
    show_missing: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Verify file exists
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()).into());
    }

    // Extract metadata using our library
    let metadata = extract_metadata(path, show_missing)?;

    // Output as JSON matching ExifTool format
    println!("{}", serde_json::to_string_pretty(&metadata)?);

    Ok(())
}
