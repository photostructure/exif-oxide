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
    // `-stay_open True -@ -` (the exiftool-vendored.js spawn) switches to the
    // REPL BEFORE clap and BEFORE any tracing subscriber: stay_open mode must
    // be completely silent on stdout/stderr outside task output, because the
    // consumer kills the child on any stray bytes (batch-cluster
    // StreamHandler.ts:75-81, :95-100).
    let raw_args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(seed_args) = exif_oxide::cli::detect_stay_open(&raw_args) {
        // Debug escape hatch (M1a decision 7): EXIF_OXIDE_LOG=/path routes
        // tracing to that file - NEVER a std stream. Without it, no
        // subscriber is installed and all tracing events are discarded.
        // (The consumer replaces the child env with {LANG:"C"}, so this can
        // only be set deliberately.)
        if let Some(path) = std::env::var_os("EXIF_OXIDE_LOG").filter(|p| !p.is_empty()) {
            if let Ok(file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
            {
                let filter = tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("debug"));
                tracing_subscriber::fmt()
                    .with_env_filter(filter)
                    .with_writer(std::sync::Mutex::new(file))
                    .with_ansi(false)
                    .init();
            }
            // An unopenable log path is silently ignored: there is nowhere
            // safe to report it.
        }

        // Panics must never write to stderr (the default hook prints
        // "thread panicked at ..." there, which would poison the protocol).
        // catch_unwind in the REPL turns them into task errors; the hook
        // forwards details to tracing for EXIF_OXIDE_LOG debugging.
        std::panic::set_hook(Box::new(|info| {
            tracing::error!("panic: {info}");
        }));

        let stdin = std::io::stdin();
        let stdout = std::io::stdout();
        let stderr = std::io::stderr();
        let code =
            exif_oxide::cli::stay_open::run(stdin.lock(), stdout.lock(), stderr.lock(), seed_args);
        std::process::exit(code);
    }

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
    // Binary extraction is a classic-mode-only feature: exactly one tag and
    // one file (validated by the caller). It needs the full unfiltered
    // metadata to locate offset/length tags.
    if binary_extraction {
        let path = &paths[0];
        let tag_name = &filter_options.requested_tags[0];
        let no_filters = FilterOptions::extract_all();
        let full_metadata = process_single_file(path, show_missing, show_warnings, &no_filters)
            .map_err(|e| format!("Failed to extract full metadata for binary extraction: {e}"))?;
        return extract_binary_data(&full_metadata, tag_name, path);
    }

    // The per-file loop (missing-file stderr lines, ExifTool:Error entries,
    // serialization prep) is shared with the stay_open REPL.
    let files: Vec<String> = paths
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    let mut stderr = std::io::stderr();
    let results =
        exif_oxide::cli::collect_entries(&files, &filter_options, show_missing, &mut stderr);

    // Output as JSON array matching ExifTool format. When every file was
    // missing there is nothing to print - ExifTool emits no JSON at all in
    // that case (probed: `exiftool -j /nonexistent.jpg` => empty stdout).
    if !results.is_empty() {
        println!("{}", serde_json::to_string_pretty(&results)?);
    }

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
    // Existence is checked by the caller (missing files never reach here, so
    // they never get a JSON entry); a file vanishing in between surfaces as
    // the File::open error inside extract_metadata.

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
