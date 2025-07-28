use clap::{Arg, Command};
use std::collections::HashSet;
use std::path::PathBuf;
use tracing::{debug, error, info};

// Import our library modules
use exif_oxide::formats::extract_metadata;
use exif_oxide::types::FilterOptions;

/// Parse command line arguments into file paths and filter options
/// Supports ExifTool-style tag filtering patterns:
/// - `-TagName` - extract specific tag
/// - `-TagName#` - extract tag with numeric value (ValueConv)  
/// - `-GroupName:all` - extract all tags from group
/// - `-all` - extract all tags
///
/// Returns (file_paths, filter_options) or exits on error
fn parse_exiftool_args(args: Vec<&String>) -> (Vec<&String>, FilterOptions) {
    let mut file_paths = Vec::new();
    let mut requested_tags = Vec::new();
    let requested_groups = Vec::new();
    let mut group_all_patterns = Vec::new();
    let mut glob_patterns = Vec::new();
    let mut numeric_tags = HashSet::new();
    let mut extract_all = false;

    // Debug: print all received arguments
    debug!("CLI args received: {:?}", args);

    for arg in args {
        if arg == "-all" || arg == "--all" {
            // Special case: extract all tags
            extract_all = true;
        } else if arg == "-ver" {
            // Version flag - print version and exit
            println!("{}", env!("CARGO_PKG_VERSION"));
            std::process::exit(0);
        } else if arg == "-j" || arg == "-struct" || arg == "-G" {
            // ExifTool compatibility flags - ignore these (no-op)
            // -j: JSON output format (we always output JSON)
            // -struct: structured output (we always use structured output)
            // -G: group names in output (we always include group names)
            debug!("Ignoring ExifTool compatibility flag: {}", arg);
            continue;
        } else if arg.starts_with('-') && arg.len() > 1 {
            // Process tag/group filters
            let filter_arg = &arg[1..]; // Remove leading '-'

            // Check for invalid short options (1-2 characters or unknown single-char flags)
            if filter_arg.len() <= 2 {
                eprintln!("Unknown option {}", arg);
                std::process::exit(1);
            }

            if filter_arg.ends_with('#') && filter_arg.len() > 1 {
                // Numeric tag: -TagName# or -Pattern#
                let tag_name = &filter_arg[..filter_arg.len() - 1];
                if tag_name.contains('*') {
                    // Glob pattern with numeric: -GPS*#
                    glob_patterns.push(tag_name.to_string());
                    numeric_tags.insert(tag_name.to_string());
                } else {
                    // Regular numeric tag: -TagName#
                    requested_tags.push(tag_name.to_string());
                    numeric_tags.insert(tag_name.to_string());
                }
            } else if filter_arg.ends_with(":all") {
                // Group all pattern: -GroupName:all
                group_all_patterns.push(filter_arg.to_string());
            } else if filter_arg.contains('*') {
                // Glob pattern: -GPS*, -*tude, -*Date*, -EXIF:*
                glob_patterns.push(filter_arg.to_string());
            } else if filter_arg.contains(':') {
                // Group:tag pattern (future extension)
                // For now, treat as specific tag request
                requested_tags.push(filter_arg.to_string());
            } else {
                // Simple tag name: -TagName
                requested_tags.push(filter_arg.to_string());
            }
        } else if arg == "-" || arg == "--" {
            // Stdin markers
            file_paths.push(arg);
        } else {
            // File path (doesn't start with -, or is stdin marker)
            file_paths.push(arg);
        }
    }

    // Build FilterOptions based on parsed arguments
    let filter_options = if extract_all {
        // -all flag overrides everything else
        FilterOptions {
            requested_tags: Vec::new(),
            requested_groups: Vec::new(),
            group_all_patterns: Vec::new(),
            extract_all: true,
            numeric_tags,
            glob_patterns: Vec::new(),
        }
    } else if requested_tags.is_empty()
        && requested_groups.is_empty()
        && group_all_patterns.is_empty()
        && glob_patterns.is_empty()
    {
        // No filters specified - extract all tags (backward compatibility)
        FilterOptions {
            requested_tags: Vec::new(),
            requested_groups: Vec::new(),
            group_all_patterns: Vec::new(),
            extract_all: true,
            numeric_tags,
            glob_patterns: Vec::new(),
        }
    } else {
        // Specific filters requested
        FilterOptions {
            requested_tags,
            requested_groups,
            group_all_patterns,
            extract_all: false,
            numeric_tags,
            glob_patterns,
        }
    };

    // Debug: print final filter options
    debug!("Final FilterOptions: {:?}", filter_options);

    (file_paths, filter_options)
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
            "EXIFTOOL COMPATIBILITY:\n",
            "  -ver             Print version number and exit\n",
            "  -j, -struct, -G  Ignored (we always output JSON with structure and groups)\n",
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
        .get_matches();

    // Extract all arguments and parse ExifTool-style filters
    let args: Vec<&String> = matches.get_many::<String>("args").unwrap().collect();
    let show_missing = matches.get_flag("show-missing");
    let show_warnings = matches.get_flag("warnings");
    let binary_extraction = matches.get_flag("binary");

    // Parse arguments into files and filter options using ExifTool patterns
    let (file_paths, filter_options) = parse_exiftool_args(args);

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
                    let no_filters = FilterOptions {
                        requested_tags: Vec::new(),
                        requested_groups: Vec::new(),
                        group_all_patterns: Vec::new(),
                        extract_all: true,
                        numeric_tags: std::collections::HashSet::new(),
                        glob_patterns: Vec::new(),
                    };
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

    // Prepare for serialization by converting tags to legacy format
    // Pass numeric_tags to determine which tags should use numeric values
    let numeric_tags_ref = if filter_options.numeric_tags.is_empty() {
        None
    } else {
        Some(&filter_options.numeric_tags)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_exiftool_args_files_before_tags() {
        let image1 = "image1.jpg".to_string();
        let image2 = "image2.png".to_string();
        let fnumber = "-FNumber#".to_string();
        let exposure = "-ExposureTime#".to_string();
        let args = vec![&image1, &image2, &fnumber, &exposure];

        let (files, filter_opts) = parse_exiftool_args(args);

        assert_eq!(files, vec!["image1.jpg", "image2.png"]);
        assert!(filter_opts.requested_tags.contains(&"FNumber".to_string()));
        assert!(filter_opts
            .requested_tags
            .contains(&"ExposureTime".to_string()));
        assert!(filter_opts.numeric_tags.contains("FNumber"));
        assert!(filter_opts.numeric_tags.contains("ExposureTime"));
        assert_eq!(filter_opts.requested_tags.len(), 2);
    }

    #[test]
    fn test_parse_exiftool_args_group_all_patterns() {
        let image = "image.jpg".to_string();
        let file_all = "-File:all".to_string();
        let exif_all = "-EXIF:all".to_string();
        let args = vec![&image, &file_all, &exif_all];

        let (files, filter_opts) = parse_exiftool_args(args);

        assert_eq!(files, vec!["image.jpg"]);
        assert!(filter_opts
            .group_all_patterns
            .contains(&"File:all".to_string()));
        assert!(filter_opts
            .group_all_patterns
            .contains(&"EXIF:all".to_string()));
        assert_eq!(filter_opts.group_all_patterns.len(), 2);
        assert!(!filter_opts.extract_all);
    }

    #[test]
    fn test_parse_exiftool_args_extract_all() {
        let image = "image.jpg".to_string();
        let all_flag = "-all".to_string();
        let args = vec![&image, &all_flag];

        let (files, filter_opts) = parse_exiftool_args(args);

        assert_eq!(files, vec!["image.jpg"]);
        assert!(filter_opts.extract_all);
        assert!(filter_opts.requested_tags.is_empty());
        assert!(filter_opts.group_all_patterns.is_empty());
    }

    #[test]
    fn test_parse_exiftool_args_numeric_tags() {
        let image = "image.jpg".to_string();
        let orientation_num = "-Orientation#".to_string();
        let fnumber_norm = "-FNumber".to_string();
        let args = vec![&image, &orientation_num, &fnumber_norm];

        let (files, filter_opts) = parse_exiftool_args(args);

        assert_eq!(files, vec!["image.jpg"]);
        assert!(filter_opts
            .requested_tags
            .contains(&"Orientation".to_string()));
        assert!(filter_opts.requested_tags.contains(&"FNumber".to_string()));
        assert!(filter_opts.numeric_tags.contains("Orientation"));
        assert!(!filter_opts.numeric_tags.contains("FNumber"));
    }

    #[test]
    fn test_parse_exiftool_args_edge_cases() {
        // Test with stdin marker "-"
        let dash = "-".to_string();
        let fnumber = "-FNumber".to_string();
        let args = vec![&dash, &fnumber];
        let (files, filter_opts) = parse_exiftool_args(args);
        assert_eq!(files, vec!["-"]);
        assert!(filter_opts.requested_tags.contains(&"FNumber".to_string()));

        // Test with no filters (should default to extract_all)
        let image = "image.jpg".to_string();
        let args = vec![&image];
        let (files, filter_opts) = parse_exiftool_args(args);
        assert_eq!(files, vec!["image.jpg"]);
        assert!(filter_opts.extract_all);
    }

    #[test]
    fn test_parse_exiftool_args_compatibility_flags() {
        // Test ExifTool compatibility flags are ignored as no-ops
        let image = "image.jpg".to_string();
        let json_flag = "-j".to_string();
        let struct_flag = "-struct".to_string();
        let group_flag = "-G".to_string();
        let fnumber = "-FNumber".to_string();
        let args = vec![&image, &json_flag, &struct_flag, &group_flag, &fnumber];

        let (files, filter_opts) = parse_exiftool_args(args);

        // Should have only the image file, compatibility flags ignored
        assert_eq!(files, vec!["image.jpg"]);
        // Should only have the FNumber tag, not the compatibility flags
        assert_eq!(filter_opts.requested_tags.len(), 1);
        assert!(filter_opts.requested_tags.contains(&"FNumber".to_string()));
        assert!(!filter_opts.extract_all);
    }

    #[test]
    fn test_parse_exiftool_args_compatibility_flags_only() {
        // Test with only compatibility flags (should default to extract_all)
        let image = "image.jpg".to_string();
        let json_flag = "-j".to_string();
        let struct_flag = "-struct".to_string();
        let group_flag = "-G".to_string();
        let args = vec![&image, &json_flag, &struct_flag, &group_flag];

        let (files, filter_opts) = parse_exiftool_args(args);

        // Should have only the image file
        assert_eq!(files, vec!["image.jpg"]);
        // Since no actual tag filters were specified, should default to extract_all
        assert!(filter_opts.extract_all);
        assert!(filter_opts.requested_tags.is_empty());
    }

    #[test]
    fn test_parse_exiftool_args_boundary_lengths() {
        // Test boundary cases for filter length validation - only valid 3+ char tags accepted
        let image = "image.jpg".to_string();
        let three_char = "-abc".to_string(); // 3 chars - should be accepted
        let args = vec![&image, &three_char];

        let (files, filter_opts) = parse_exiftool_args(args);

        // Should have only the image file
        assert_eq!(files, vec!["image.jpg"]);
        // Should only have the 3-character tag
        assert_eq!(filter_opts.requested_tags.len(), 1);
        assert!(filter_opts.requested_tags.contains(&"abc".to_string()));
        assert!(!filter_opts.extract_all);
    }

    #[test]
    fn test_parse_exiftool_args_all_compatibility_flags() {
        // Test all compatibility flags together
        let image = "image.jpg".to_string();
        let j_flag = "-j".to_string();
        let struct_flag = "-struct".to_string();
        let g_flag = "-G".to_string();
        let valid_tag = "-MIMEType".to_string();
        let args = vec![&image, &j_flag, &struct_flag, &g_flag, &valid_tag];

        let (files, filter_opts) = parse_exiftool_args(args);

        // Should have only the image file
        assert_eq!(files, vec!["image.jpg"]);
        // Should only have the valid tag, compatibility flags ignored
        assert_eq!(filter_opts.requested_tags.len(), 1);
        assert!(filter_opts.requested_tags.contains(&"MIMEType".to_string()));
        assert!(!filter_opts.extract_all);
    }
}
