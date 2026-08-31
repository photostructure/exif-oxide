use clap::{Arg, Command};
use std::path::PathBuf;
use tracing::{debug, error, info};

// Import our library modules
use exif_oxide::formats::extract_metadata;
use exif_oxide::hash::ImageHashType;
use exif_oxide::types::FilterOptions;

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
        .version(env!("CARGO_PKG_VERSION"))
        .author("exif-oxide@photostructure.com")
        .about("High-performance Rust implementation of ExifTool")
        .after_help(concat!(
            "EXAMPLES:\n",
            "  exif-oxide image.jpg                    Extract all metadata\n",
            "  exif-oxide -MIMEType image.jpg          Extract only MIMEType tag\n",
            "  exif-oxide -Orientation# image.jpg      Extract Orientation with numeric value\n",
            "  exif-oxide -EXIF:all image.jpg          Extract all EXIF group tags\n",
            "  exif-oxide -GPS* image.jpg              Extract all GPS tags (wildcard)\n",
            "  exif-oxide -*Date* image.jpg            Extract all tags containing 'Date'\n",
            "  exif-oxide -all image.jpg               Extract all available tags\n",
            "  exif-oxide -ver                         Print version (ExifTool compatibility)\n",
            "\n",
            "TAG FILTERING:\n",
            "  -TagName         Extract specific tag (case-insensitive)\n",
            "  -TagName#        Extract tag with numeric value (ValueConv)\n",
            "  -Group:all       Extract all tags from group (File, EXIF, GPS, etc.)\n",
            "  -Pattern*        Prefix wildcard (e.g., -GPS*, -Canon*)\n",
            "  -*Pattern        Suffix wildcard (e.g., -*tude for latitude/longitude)\n",
            "  -*Pattern*       Middle wildcard (e.g., -*Date* for date-related tags)\n",
            "  -all             Extract all available tags\n",
            "\n",
            "BINARY EXTRACTION:\n",
            "  -b, --binary     Extract binary data (use with tag filters, outputs to stdout)\n",
            "                   Example: exif-oxide -b -ThumbnailImage image.jpg > thumb.jpg\n",
            "\n",
            "IMAGE DATA HASH:\n",
            "  --image-hash           Compute hash of image data (excludes metadata)\n",
            "  --image-hash-type ALG  Hash algorithm: MD5 (default), SHA256, SHA512\n",
            "                         Example: exif-oxide --image-hash --image-hash-type SHA256 image.jpg\n",
            "\n",
            "EXIFTOOL COMPATIBILITY:\n",
            "  -ver             Print the emulated ExifTool version\n",
            "  -j -json -struct -G -G1 -g -a -e -q -m -fast[N] -ignoreMinorErrors\n",
            "                   Accepted no-ops (output is always ExifTool's -j -struct -G shape)\n",
            "  -api X -use X -x TAG -charset X -w X -d X -c X -if X -echo X -echo2 X\n",
            "                   Value-taking options, accepted and ignored (except -api\n",
            "                   requesttags=imagedatahash / imagehashtype=ALG, which are honored)\n",
            "\n",
            "Multiple filters can be combined:\n",
            "  exif-oxide -Orientation# -GPS* -EXIF:all image.jpg\n"
        ))
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
        .arg(
            Arg::new("binary")
                .short('b')
                .long("binary")
                .help("Extract binary data for specified tag (outputs raw binary to stdout)")
                .action(clap::ArgAction::SetTrue), // Boolean flag
        )
        .arg(
            Arg::new("image-hash")
                .long("image-hash")
                .help("Compute ImageDataHash (hash of image data, excluding metadata)")
                .long_help(
                    "Compute a cryptographic hash of the image data only, excluding metadata.\n\
                     This allows detecting changes to image content while ignoring metadata edits.\n\
                     The hash is output as Composite:ImageDataHash.\n\n\
                     ExifTool equivalent: -api requesttags=imagedatahash"
                )
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("image-hash-type")
                .long("image-hash-type")
                .help("Hash algorithm for --image-hash (default: MD5)")
                .long_help(
                    "Select the hash algorithm for ImageDataHash computation.\n\
                     Options: MD5 (default), SHA256, SHA512\n\n\
                     ExifTool equivalent: -api imagehashtype=MD5|SHA256|SHA512"
                )
                .value_name("ALGORITHM")
                .value_parser(["MD5", "SHA256", "SHA512", "md5", "sha256", "sha512"])
                .default_value("MD5"),
        )
        .get_matches();

    // Extract all arguments and parse ExifTool-style filters
    let args: Vec<&String> = matches.get_many::<String>("args").unwrap().collect();
    let show_missing = matches.get_flag("show-missing");
    let show_warnings = matches.get_flag("warnings");
    let binary_extraction = matches.get_flag("binary");
    let compute_image_hash = matches.get_flag("image-hash");
    let image_hash_type_str = matches
        .get_one::<String>("image-hash-type")
        .map(|s| s.as_str())
        .unwrap_or("MD5");

    // Parse hash type from string
    let image_hash_type = match image_hash_type_str.to_uppercase().as_str() {
        "MD5" => ImageHashType::Md5,
        "SHA256" => ImageHashType::Sha256,
        "SHA512" => ImageHashType::Sha512,
        _ => {
            eprintln!(
                "Error: Invalid hash type '{}'. Use MD5, SHA256, or SHA512.",
                image_hash_type_str
            );
            std::process::exit(1);
        }
    };

    // Parse arguments the way ExifTool's argument loop does. parse_command
    // never exits; classic mode maps its results onto exit codes here.
    debug!("CLI args received: {:?}", args);
    let parsed = exif_oxide::cli::parse_command(&args);
    debug!("Parsed command: {:?}", parsed);

    // -echo/-echo2 print before any other output (exiftool:1016-1028).
    for line in &parsed.echo_stdout {
        println!("{line}");
    }
    for line in &parsed.echo_stderr {
        eprintln!("{line}");
    }

    // A bad option aborts the command (exiftool sets $badCmd; classic mode
    // exits non-zero, preserving the old CLI contract).
    if !parsed.errors.is_empty() {
        for e in &parsed.errors {
            eprintln!("{e}");
        }
        std::process::exit(1);
    }

    // -ver prints the emulated ExifTool version; like ExifTool, any files on
    // the same command line are still processed afterwards (exiftool:779-793).
    if parsed.print_version {
        println!("{}", exif_oxide::EXIFTOOL_VERSION);
        if parsed.files.is_empty() {
            std::process::exit(0);
        }
    }

    let file_paths = parsed.files;
    let mut filter_options = parsed.filter;

    // Apply image hash options to filter_options
    if compute_image_hash {
        filter_options.compute_image_hash = true;
        filter_options.image_hash_type = image_hash_type;
    }

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
    debug!("Binary extraction mode: {}", binary_extraction);
    debug!("Compute image hash: {}", compute_image_hash);
    if compute_image_hash {
        debug!("Image hash type: {:?}", image_hash_type);
    }
    debug!("Filter options: {:?}", filter_options);

    // Validate binary extraction requirements
    if binary_extraction {
        // Binary extraction requires exactly one tag and one file for simplicity
        if filter_options.requested_tags.len() != 1 {
            eprintln!("Error: Binary extraction requires exactly one tag (e.g., -b -ThumbnailImage image.jpg)");
            std::process::exit(1);
        }
        if paths.len() != 1 {
            eprintln!("Error: Binary extraction requires exactly one file");
            std::process::exit(1);
        }
    }

    // Process all files - this will output a JSON array like ExifTool (or binary data if -b)
    match process_files(
        &paths,
        show_missing,
        show_warnings,
        binary_extraction,
        filter_options,
    ) {
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
    binary_extraction: bool,
    filter_options: FilterOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    use exif_oxide::types::ExifData;

    let mut results = Vec::new();

    // Process each file
    for path in paths {
        debug!("Processing file: {}", path.display());
        match process_single_file(path, show_missing, show_warnings, &filter_options) {
            Ok(metadata) => {
                info!("Successfully processed: {}", path.display());

                // Handle binary extraction if requested
                if binary_extraction {
                    let tag_name = &filter_options.requested_tags[0]; // We validated exactly one tag
                                                                      // For binary extraction, we need full metadata to find offset/length tags
                                                                      // Extract metadata again without filtering to get all tags
                    let no_filters = FilterOptions::extract_all();
                    match process_single_file(path, show_missing, show_warnings, &no_filters) {
                        Ok(full_metadata) => {
                            return extract_binary_data(&full_metadata, tag_name, path);
                        }
                        Err(e) => {
                            return Err(format!(
                                "Failed to extract full metadata for binary extraction: {}",
                                e
                            )
                            .into());
                        }
                    }
                }

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

    // Prepare for serialization by converting tags to legacy format.
    // The ordered request list decides which tags print their ValueConv value.
    for result in &mut results {
        result.prepare_for_serialization(Some(&filter_options.tag_requests));
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
    filter_options: &FilterOptions,
) -> Result<exif_oxide::types::ExifData, Box<dyn std::error::Error>> {
    // Verify file exists
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()).into());
    }

    // Extract metadata using our library with filtering
    let metadata = extract_metadata(
        path,
        show_missing,
        show_warnings,
        Some(filter_options.clone()),
    )?;

    Ok(metadata)
}

/// Extract binary data for the specified tag and write to stdout
/// ExifTool: Follow the same pattern as ExifTool's binary extraction
/// This function finds offset/length tags and streams binary data from the file
///
/// NOTE: ThumbnailOffset/PreviewImageStart values should already be absolute file offsets
/// after IsOffset adjustment during EXIF parsing. See ExifTool Exif.pm lines 7052-7066.
/// TODO: Implement IsOffset handling in EXIF parsing layer - see P0-IFD1-THUMBNAIL-EXTRACTION.md
fn extract_binary_data(
    metadata: &exif_oxide::types::ExifData,
    requested_tag: &str,
    file_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::{self, Read, Seek, SeekFrom, Write};

    debug!("Extracting binary data for tag: {}", requested_tag);

    // Map binary tag names to their offset/length counterparts
    // Based on ExifTool's composite tag definitions and format-specific naming
    let (offset_pattern, length_pattern) = match requested_tag.to_lowercase().as_str() {
        "thumbnailimage" => {
            // Try multiple patterns for thumbnail data
            if let (Some(offset), Some(length)) =
                find_tag_pair(metadata, "ThumbnailOffset", "ThumbnailLength")
            {
                (Some(offset), Some(length))
            } else {
                find_tag_pair(metadata, "OtherImageStart", "OtherImageLength")
            }
        }
        "previewimage" => {
            // Try multiple patterns for preview data
            if let (Some(offset), Some(length)) =
                find_tag_pair(metadata, "PreviewImageStart", "PreviewImageLength")
            {
                (Some(offset), Some(length))
            } else {
                find_tag_pair(metadata, "OtherImageStart", "OtherImageLength")
            }
        }
        "otherimage" => find_tag_pair(metadata, "OtherImageStart", "OtherImageLength"),
        _ => {
            return Err(
                format!("Binary extraction not supported for tag: {}", requested_tag).into(),
            );
        }
    };

    let (offset_value, length_value) = match (offset_pattern, length_pattern) {
        (Some(offset), Some(length)) => (offset, length),
        _ => {
            return Err(format!(
                "Required offset/length tags not found for: {}",
                requested_tag
            )
            .into());
        }
    };

    debug!("Found offset: {}, length: {}", offset_value, length_value);

    // Open file for binary reading
    let mut file = File::open(file_path)?;

    // Seek to offset position
    // NOTE: Offset should be absolute file position after IsOffset adjustment in parsing
    file.seek(SeekFrom::Start(offset_value as u64))?;

    // Read binary data in chunks and stream to stdout
    // This approach handles large previews (500KB+) efficiently without loading into memory
    let mut buffer = vec![0u8; 8192]; // 8KB buffer for streaming
    let mut remaining = length_value as usize;
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    while remaining > 0 {
        let chunk_size = std::cmp::min(buffer.len(), remaining);
        let bytes_read = file.read(&mut buffer[..chunk_size])?;

        if bytes_read == 0 {
            return Err("Unexpected end of file during binary extraction".into());
        }

        handle.write_all(&buffer[..bytes_read])?;
        remaining -= bytes_read;
    }

    handle.flush()?;
    debug!("Successfully extracted {} bytes", length_value);

    Ok(())
}

/// Find a pair of offset/length tags in metadata
/// Returns (offset_value, length_value) if both found, otherwise (None, None)
fn find_tag_pair(
    metadata: &exif_oxide::types::ExifData,
    offset_name: &str,
    length_name: &str,
) -> (Option<u32>, Option<u32>) {
    let mut offset_value = None;
    let mut length_value = None;

    debug!("Looking for tags: {} and {}", offset_name, length_name);
    debug!(
        "Available tags: {:?}",
        metadata.tags.iter().map(|t| &t.name).collect::<Vec<_>>()
    );

    // Search through all tags for the offset and length values
    for tag_entry in &metadata.tags {
        // Check if tag name matches (with or without group prefix)
        if tag_entry.name.ends_with(&format!(":{}", offset_name)) || tag_entry.name == offset_name {
            if let Some(val) = tag_entry.value.as_u32() {
                debug!("Found offset tag {}: {}", tag_entry.name, val);
                offset_value = Some(val);
            }
        } else if tag_entry.name.ends_with(&format!(":{}", length_name))
            || tag_entry.name == length_name
        {
            if let Some(val) = tag_entry.value.as_u32() {
                debug!("Found length tag {}: {}", tag_entry.name, val);
                length_value = Some(val);
            }
        }
    }

    debug!(
        "Result: offset={:?}, length={:?}",
        offset_value, length_value
    );
    (offset_value, length_value)
}
