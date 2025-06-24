//! Command-line tool for extracting EXIF data

use exif_oxide::binary::{extract_binary_tag, extract_mpf_preview};
use exif_oxide::core::ifd::IfdParser;
use exif_oxide::core::mpf::ParsedMpf;
use exif_oxide::core::print_conv::{apply_print_conv, PrintConvId};
use exif_oxide::core::ExifValue;
use exif_oxide::tables::{fujifilm_tags, nikon_tags, olympus_tags, pentax_tags, sony_tags};
use exif_oxide::tables::{lookup_canon_tag, lookup_tag};
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::io::{self, Write};
use std::process;

/// Command represents the operation to perform
#[derive(Debug)]
enum Command {
    /// List all tags as JSON
    ListAll {
        include_groups: bool,
        show_converted_values: bool,
    },
    /// List specific tags as JSON
    ListSpecific {
        tags: Vec<String>,
        include_groups: bool,
        show_converted_values: bool,
    },
    /// Extract binary data for a single tag
    ExtractBinary { tag: String },
}

/// Parse command line arguments into a Command
fn parse_args(args: Vec<String>) -> Result<(Command, Vec<String>), String> {
    let mut include_groups = false;
    let mut binary_mode = false;
    let mut show_converted_values = true; // Default to showing converted values
    let mut tag_filters = Vec::new();
    let mut image_paths = Vec::new();

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];

        if arg == "-G" {
            include_groups = true;
            i += 1;
        } else if arg == "-b" {
            binary_mode = true;
            i += 1;
        } else if arg == "-n" {
            show_converted_values = false; // Show numeric/raw values only
            i += 1;
        } else if arg.starts_with('-') && !arg.starts_with("--") {
            // This is a tag specification (e.g., -ThumbnailImage)
            let tag_name = &arg[1..]; // Remove the leading dash
            tag_filters.push(tag_name.to_string());
            i += 1;
        } else if arg.starts_with('-') {
            return Err(format!("Unknown option: {}", arg));
        } else {
            // This is an image file path
            image_paths.push(arg.clone());
            i += 1;
        }
    }

    if image_paths.is_empty() {
        return Err("No image file specified".to_string());
    }

    // Determine the command based on parsed arguments
    let command = if binary_mode {
        // Binary mode requires exactly one tag and one file
        if tag_filters.len() != 1 {
            return Err("Binary mode (-b) requires exactly one tag specification".to_string());
        }
        if image_paths.len() != 1 {
            return Err("Binary mode (-b) requires exactly one image file".to_string());
        }
        Command::ExtractBinary {
            tag: tag_filters.into_iter().next().unwrap(),
        }
    } else if tag_filters.is_empty() {
        // No tags specified, list all
        Command::ListAll {
            include_groups,
            show_converted_values,
        }
    } else {
        // Specific tags requested
        Command::ListSpecific {
            tags: tag_filters,
            include_groups,
            show_converted_values,
        }
    };

    Ok((command, image_paths))
}

/// Display usage information
fn show_usage(program: &str) {
    eprintln!(
        "Usage: {} [-G] [-n] [-b] [-TagName]... <image_file>...",
        program
    );
    eprintln!();
    eprintln!("Extracts EXIF data from image files.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -G         Include group names in output");
    eprintln!("  -n         Show numeric/raw values (no PrintConv)");
    eprintln!("  -b         Extract raw binary data (requires single -TagName and single file)");
    eprintln!("  -TagName   Include only specified tag(s) in output");
    eprintln!();
    eprintln!("Examples:");
    eprintln!(
        "  {} photo.jpg                          # All tags as JSON",
        program
    );
    eprintln!(
        "  {} photo1.jpg photo2.jpg              # Multiple files",
        program
    );
    eprintln!(
        "  {} -G photo.jpg                       # All tags with groups",
        program
    );
    eprintln!(
        "  {} -Make -Model photo.jpg             # Only Make and Model",
        program
    );
    eprintln!(
        "  {} -b -ThumbnailImage photo.jpg       # Extract thumbnail to stdout",
        program
    );
    eprintln!(
        "  {} -b -ThumbnailImage photo.jpg > thumb.jpg  # Save thumbnail",
        program
    );
}

/// Resolve a tag name to its ID, handling various formats
fn resolve_tag_name(tag_name: &str) -> Option<(u16, Option<&'static str>)> {
    // Handle hex notation (e.g., 0x201 or 0x0201)
    if tag_name.starts_with("0x") || tag_name.starts_with("0X") {
        if let Ok(tag_id) = u16::from_str_radix(&tag_name[2..], 16) {
            return Some((tag_id, None));
        }
    }

    // Handle group:tag notation (e.g., IFD1:ThumbnailImage)
    let (group, name) = if let Some(colon_pos) = tag_name.find(':') {
        let group = &tag_name[..colon_pos];
        let name = &tag_name[colon_pos + 1..];
        (Some(group), name)
    } else {
        (None, tag_name)
    };

    // Special handling for common image tags
    match name.to_lowercase().as_str() {
        "thumbnailimage" => return Some((0x1201, Some("IFD1"))), // IFD1 tag with prefix
        "thumbnailoffset" => return Some((0x1201, Some("IFD1"))),
        "thumbnaillength" => return Some((0x1202, Some("IFD1"))),
        "previewimage" => return Some((0xFFFF, Some("MPF"))), // Special marker for MPF preview
        "previewimagestart" => return Some((0x111, Some("IFD0"))),
        "previewimagelength" => return Some((0x117, Some("IFD0"))),
        _ => {}
    }

    // Search standard EXIF tags
    for tag_id in 0..=0xFFFF {
        if let Some(tag_info) = lookup_tag(tag_id) {
            if tag_info.name.eq_ignore_ascii_case(name) {
                // Check group match if specified
                if let Some(req_group) = group {
                    if let Some(tag_group) = tag_info.group {
                        if tag_group.eq_ignore_ascii_case(req_group) {
                            return Some((tag_id, tag_info.group));
                        }
                    }
                } else {
                    return Some((tag_id, tag_info.group));
                }
            }
        }
    }

    // Search Canon maker note tags
    for tag_id in 0..=0xFFFF {
        if let Some(tag_info) = lookup_canon_tag(tag_id) {
            if tag_info.name.eq_ignore_ascii_case(name) {
                // Canon tags are stored with 0xC000 prefix
                return Some((0xC000 + tag_id, Some("Canon")));
            }
        }
    }

    None
}

/// Extract binary data for a specific tag
fn extract_binary(tag_name: &str, image_path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Resolve tag name to ID
    let (tag_id, group) =
        resolve_tag_name(tag_name).ok_or_else(|| format!("Unknown tag: {}", tag_name))?;

    // Check if this is a special MPF preview request
    if tag_id == 0xFFFF && group == Some("MPF") {
        // This is a PreviewImage request - try MPF first
        return extract_mpf_preview_from_file(image_path);
    }

    // Standard EXIF binary extraction
    let metadata_segment = exif_oxide::core::find_metadata_segment(image_path)?
        .ok_or_else(|| format!("No EXIF data found in '{}'", image_path))?;
    let ifd = IfdParser::parse(metadata_segment.data)?;
    let original_data = std::fs::read(image_path)?;

    // Use the standard binary extraction function
    if let Some(data) = extract_binary_tag(&ifd, tag_id, &original_data)? {
        Ok(data)
    } else {
        Err(format!("Tag not found or contains no data: {}", tag_name).into())
    }
}

/// Extract MPF preview from a file
fn extract_mpf_preview_from_file(image_path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Get all metadata segments
    let metadata = exif_oxide::core::find_all_metadata_segments(image_path)?;

    // Check if MPF segment exists
    if let Some(mpf_segment) = metadata.mpf {
        // Parse MPF data
        let mpf = ParsedMpf::parse(mpf_segment.data)?;

        // Read original file data
        let original_data = std::fs::read(image_path)?;

        // Extract MPF preview
        if let Some(preview_data) =
            extract_mpf_preview(&mpf, &original_data, mpf_segment.offset as usize)?
        {
            return Ok(preview_data);
        }
    }

    // Fallback to standard EXIF preview extraction
    let metadata_segment = exif_oxide::core::find_metadata_segment(image_path)?
        .ok_or_else(|| format!("No EXIF data found in '{}'", image_path))?;
    let ifd = IfdParser::parse(metadata_segment.data)?;
    let original_data = std::fs::read(image_path)?;

    // Try IFD0 PreviewImage (StripOffsets)
    if let Some(data) = extract_binary_tag(&ifd, 0x111, &original_data)? {
        return Ok(data);
    }

    Err("No preview image found in EXIF or MPF data".into())
}

/// Check if a value should be replaced with a binary length indicator
fn should_replace_with_length(value: &ExifValue, tag_name: &str) -> bool {
    // Replace large binary data with length indicators
    match value {
        ExifValue::Undefined(data) => {
            // Keep small binary data (under 64 bytes) as-is
            // Or if it's a known tag that should show binary data
            data.len() > 64 && !is_binary_display_tag(tag_name)
        }
        ExifValue::U8Array(data) => data.len() > 64,
        _ => false,
    }
}

/// Convert a value to a binary data length indicator
fn convert_to_binary_length(value: &ExifValue) -> ExifValue {
    match value {
        ExifValue::Undefined(data) => ExifValue::BinaryData(data.len()),
        ExifValue::U8Array(data) => ExifValue::BinaryData(data.len()),
        _ => value.clone(),
    }
}

/// Check if a tag should display its binary data (not just length)
fn is_binary_display_tag(tag_name: &str) -> bool {
    // Some tags like ExifVersion should show their binary data
    matches!(
        tag_name,
        "ExifVersion" | "FlashpixVersion" | "InteroperabilityVersion"
    )
}

#[derive(Serialize)]
struct TagEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    group: Option<String>,
    #[serde(flatten)]
    value: ExifValue,
}

#[derive(Serialize)]
struct FileOutput {
    #[serde(rename = "SourceFile")]
    source_file: String,
    #[serde(flatten)]
    tags: HashMap<String, TagEntry>,
}

/// Get tag key with optional group prefix
fn format_tag_key(tag_name: &str, group: Option<&str>, include_groups: bool) -> String {
    if include_groups {
        if let Some(g) = group {
            format!("{}:{}", g, tag_name)
        } else {
            tag_name.to_string()
        }
    } else {
        tag_name.to_string()
    }
}

/// Get PrintConv for standard EXIF tags
fn get_standard_exif_printconv(tag_id: u16) -> PrintConvId {
    match tag_id {
        0x8827 => PrintConvId::IsoSpeed,     // ISO Speed
        0x9207 => PrintConvId::MeteringMode, // MeteringMode
        0xA403 => PrintConvId::WhiteBalance, // WhiteBalance
        _ => PrintConvId::None,              // Only use existing variants for now
    }
}

/// Get PrintConv for maker note tags
fn get_maker_note_printconv(tag_id: u16, manufacturer_prefix: u16) -> PrintConvId {
    match manufacturer_prefix {
        0x534F => {
            // Sony
            if let Some(sony_tag) = sony_tags::get_sony_tag(tag_id) {
                sony_tag.print_conv
            } else {
                PrintConvId::None
            }
        }
        0x5045 => {
            // Pentax
            if let Some(pentax_tag) = pentax_tags::get_pentax_tag(tag_id) {
                pentax_tag.print_conv
            } else {
                PrintConvId::None
            }
        }
        0x4F4C => {
            // Olympus
            if let Some(olympus_tag) = olympus_tags::get_olympus_tag(tag_id) {
                olympus_tag.print_conv
            } else {
                PrintConvId::None
            }
        }
        0x4E00 => {
            // Nikon
            if let Some(nikon_tag) = nikon_tags::get_nikon_tag(tag_id) {
                nikon_tag.print_conv
            } else {
                PrintConvId::None
            }
        }
        0x4655 => {
            // Fujifilm
            if let Some(fujifilm_tag) = fujifilm_tags::get_fujifilm_tag(tag_id) {
                fujifilm_tag.print_conv
            } else {
                PrintConvId::None
            }
        }
        _ => PrintConvId::None,
    }
}

/// Process entries and build the tag map
fn build_tag_map(
    entries: &HashMap<u16, ExifValue>,
    include_groups: bool,
    tag_filter: Option<&[String]>,
    show_converted_values: bool,
) -> HashMap<String, TagEntry> {
    let mut tags: HashMap<String, TagEntry> = HashMap::new();

    for (tag_id, value) in entries {
        let (tag_key, group, print_conv_id) = if *tag_id >= 0xC000 {
            // This is a maker note tag (prefixed with 0xC000 for Canon)
            let original_tag = tag_id - 0xC000;
            if let Some(tag_info) = lookup_canon_tag(original_tag) {
                let tag_name = tag_info.name;
                let tag_key = format_tag_key(tag_name, Some("Canon"), include_groups);
                // Canon tags use a different PrintConv system - for now use None
                (tag_key, Some("Canon".to_string()), PrintConvId::None)
            } else {
                let tag_name = format!("Unknown0x{:04X}", original_tag);
                let tag_key = format_tag_key(&tag_name, Some("Canon"), include_groups);
                (tag_key, Some("Canon".to_string()), PrintConvId::None)
            }
        } else if *tag_id >= 0x8000 {
            // This is a converted maker note tag (high bit set)
            // These are already converted by the maker note parser
            continue; // Skip - already processed
        } else if *tag_id >= 0x534F && *tag_id < 0x5350 {
            // Sony maker note tags (0x534F prefix)
            let original_tag = tag_id - 0x534F;
            let tag_name = format!("Sony0x{:04X}", original_tag);
            let tag_key = format_tag_key(&tag_name, Some("Sony"), include_groups);
            let print_conv_id = get_maker_note_printconv(original_tag, 0x534F);
            (tag_key, Some("Sony".to_string()), print_conv_id)
        } else if *tag_id >= 0x5045 && *tag_id < 0x5046 {
            // Pentax maker note tags (0x5045 prefix)
            let original_tag = tag_id - 0x5045;
            let tag_name = format!("Pentax0x{:04X}", original_tag);
            let tag_key = format_tag_key(&tag_name, Some("Pentax"), include_groups);
            let print_conv_id = get_maker_note_printconv(original_tag, 0x5045);
            (tag_key, Some("Pentax".to_string()), print_conv_id)
        } else if *tag_id >= 0x4F4C && *tag_id < 0x4F4D {
            // Olympus maker note tags (0x4F4C prefix)
            let original_tag = tag_id - 0x4F4C;
            let tag_name = format!("Olympus0x{:04X}", original_tag);
            let tag_key = format_tag_key(&tag_name, Some("Olympus"), include_groups);
            let print_conv_id = get_maker_note_printconv(original_tag, 0x4F4C);
            (tag_key, Some("Olympus".to_string()), print_conv_id)
        } else if *tag_id >= 0x4E00 && *tag_id < 0x4E01 {
            // Nikon maker note tags (0x4E00 prefix)
            let original_tag = tag_id - 0x4E00;
            let tag_name = format!("Nikon0x{:04X}", original_tag);
            let tag_key = format_tag_key(&tag_name, Some("Nikon"), include_groups);
            let print_conv_id = get_maker_note_printconv(original_tag, 0x4E00);
            (tag_key, Some("Nikon".to_string()), print_conv_id)
        } else if *tag_id >= 0x4655 && *tag_id < 0x4656 {
            // Fujifilm maker note tags (0x4655 prefix)
            let original_tag = tag_id - 0x4655;
            let tag_name = format!("Fujifilm0x{:04X}", original_tag);
            let tag_key = format_tag_key(&tag_name, Some("Fujifilm"), include_groups);
            let print_conv_id = get_maker_note_printconv(original_tag, 0x4655);
            (tag_key, Some("Fujifilm".to_string()), print_conv_id)
        } else if *tag_id >= 0x1000 {
            // This is an IFD1 tag (prefixed with 0x1000)
            let original_tag = tag_id - 0x1000;
            if let Some(tag_info) = lookup_tag(original_tag) {
                let tag_name = tag_info.name;
                let tag_key = format_tag_key(tag_name, Some("IFD1"), include_groups);
                let print_conv_id = get_standard_exif_printconv(original_tag);
                (tag_key, Some("IFD1".to_string()), print_conv_id)
            } else {
                let tag_name = format!("0x{:04X}", original_tag);
                let tag_key = format_tag_key(&tag_name, Some("IFD1"), include_groups);
                (tag_key, Some("IFD1".to_string()), PrintConvId::None)
            }
        } else {
            // Standard EXIF tag
            if let Some(tag_info) = lookup_tag(*tag_id) {
                let group = tag_info.group.unwrap_or("ExifIFD");
                let tag_key = format_tag_key(tag_info.name, Some(group), include_groups);
                let print_conv_id = get_standard_exif_printconv(*tag_id);
                (tag_key, Some(group.to_string()), print_conv_id)
            } else {
                let tag_name = format!("0x{:04X}", tag_id);
                let tag_key = format_tag_key(&tag_name, Some("Unknown"), include_groups);
                (tag_key, Some("Unknown".to_string()), PrintConvId::None)
            }
        };

        // Check if this tag should be included based on filter
        if let Some(filter) = tag_filter {
            let mut include = false;
            for filter_tag in filter {
                // Simple case-insensitive contains check for now
                if tag_key.to_lowercase().contains(&filter_tag.to_lowercase()) {
                    include = true;
                    break;
                }
            }
            if !include {
                continue;
            }
        }

        // Handle name collisions: first-with-a-non-blank-value wins
        let should_insert = if let Some(existing) = tags.get(&tag_key) {
            // Check if existing value is "blank" (empty string, zero, etc.)
            match &existing.value {
                ExifValue::Ascii(s) => s.is_empty(),
                ExifValue::U8(0) => true,
                ExifValue::U16(0) => true,
                ExifValue::U32(0) => true,
                ExifValue::I16(0) => true,
                ExifValue::I32(0) => true,
                ExifValue::Rational(0, _) => true,
                ExifValue::SignedRational(0, _) => true,
                ExifValue::U8Array(arr) => arr.is_empty() || arr.iter().all(|&x| x == 0),
                ExifValue::U16Array(arr) => arr.is_empty() || arr.iter().all(|&x| x == 0),
                ExifValue::U32Array(arr) => arr.is_empty() || arr.iter().all(|&x| x == 0),
                ExifValue::I16Array(arr) => arr.is_empty() || arr.iter().all(|&x| x == 0),
                ExifValue::I32Array(arr) => arr.is_empty() || arr.iter().all(|&x| x == 0),
                ExifValue::RationalArray(arr) => arr.is_empty() || arr.iter().all(|(n, _)| *n == 0),
                ExifValue::SignedRationalArray(arr) => {
                    arr.is_empty() || arr.iter().all(|(n, _)| *n == 0)
                }
                ExifValue::Undefined(arr) => arr.is_empty(),
                // For non-zero values, consider them as "non-blank"
                _ => false,
            }
        } else {
            true
        };

        if should_insert {
            // Check if value should be replaced with binary length indicator
            let processed_value = if should_replace_with_length(value, &tag_key) {
                convert_to_binary_length(value)
            } else if show_converted_values && print_conv_id != PrintConvId::None {
                // Apply PrintConv to get human-readable value
                let converted_string = apply_print_conv(value, print_conv_id);
                ExifValue::Ascii(converted_string)
            } else {
                value.clone()
            };

            let tag_entry = TagEntry {
                group: if include_groups { None } else { group },
                value: processed_value,
            };
            tags.insert(tag_key, tag_entry);
        }
    }

    tags
}

/// Process a single image file and return its output
fn process_image(
    image_path: &str,
    command: &Command,
) -> Result<FileOutput, Box<dyn std::error::Error>> {
    // Extract metadata segment using format dispatch
    let metadata_segment = exif_oxide::core::find_metadata_segment(image_path)?
        .ok_or_else(|| format!("No EXIF data found in '{}'", image_path))?;

    // Parse IFD to get all EXIF data
    let ifd = IfdParser::parse(metadata_segment.data.clone())?;

    // Process based on command type
    match command {
        Command::ListAll {
            include_groups,
            show_converted_values,
        } => {
            // Build complete tag map
            let mut tags =
                build_tag_map(ifd.entries(), *include_groups, None, *show_converted_values);

            // Add file metadata
            let file_name = std::path::Path::new(image_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(image_path);
            let directory = std::path::Path::new(image_path)
                .parent()
                .and_then(|p| p.to_str())
                .unwrap_or(".");

            let filename_key = format_tag_key("FileName", Some("File"), *include_groups);
            let directory_key = format_tag_key("Directory", Some("File"), *include_groups);

            tags.insert(
                filename_key,
                TagEntry {
                    group: if *include_groups {
                        None
                    } else {
                        Some("File".to_string())
                    },
                    value: ExifValue::Ascii(file_name.to_string()),
                },
            );
            tags.insert(
                directory_key,
                TagEntry {
                    group: if *include_groups {
                        None
                    } else {
                        Some("File".to_string())
                    },
                    value: ExifValue::Ascii(directory.to_string()),
                },
            );

            Ok(FileOutput {
                source_file: image_path.to_string(),
                tags,
            })
        }
        Command::ListSpecific {
            tags: tag_filter,
            include_groups,
            show_converted_values,
        } => {
            // Build filtered tag map
            let mut tags = build_tag_map(
                ifd.entries(),
                *include_groups,
                Some(tag_filter),
                *show_converted_values,
            );

            // Check if file metadata was requested
            for filter in tag_filter {
                if filter.eq_ignore_ascii_case("FileName") {
                    let file_name = std::path::Path::new(image_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(image_path);
                    let filename_key = format_tag_key("FileName", Some("File"), *include_groups);
                    tags.insert(
                        filename_key,
                        TagEntry {
                            group: if *include_groups {
                                None
                            } else {
                                Some("File".to_string())
                            },
                            value: ExifValue::Ascii(file_name.to_string()),
                        },
                    );
                }
                if filter.eq_ignore_ascii_case("Directory") {
                    let directory = std::path::Path::new(image_path)
                        .parent()
                        .and_then(|p| p.to_str())
                        .unwrap_or(".");
                    let directory_key = format_tag_key("Directory", Some("File"), *include_groups);
                    tags.insert(
                        directory_key,
                        TagEntry {
                            group: if *include_groups {
                                None
                            } else {
                                Some("File".to_string())
                            },
                            value: ExifValue::Ascii(directory.to_string()),
                        },
                    );
                }
            }

            Ok(FileOutput {
                source_file: image_path.to_string(),
                tags,
            })
        }
        Command::ExtractBinary { .. } => {
            // Binary extraction is handled separately in main
            unreachable!("ExtractBinary should be handled in main")
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // Parse command line arguments
    let (command, image_paths) = match parse_args(args.clone()) {
        Ok(result) => result,
        Err(err) => {
            eprintln!("Error: {}", err);
            eprintln!();
            show_usage(&args[0]);
            process::exit(1);
        }
    };

    // Handle different command types
    match command {
        Command::ExtractBinary { tag } => {
            // Binary extraction only works with single file
            if image_paths.len() != 1 {
                eprintln!("Error: Binary mode requires exactly one image file");
                process::exit(1);
            }

            let image_path = &image_paths[0];

            // Extract and output binary data
            let binary_data = extract_binary(&tag, image_path)?;
            io::stdout().write_all(&binary_data)?;
            io::stdout().flush()?;
        }
        _ => {
            // Process all files and collect results
            let mut all_outputs = Vec::new();

            for image_path in &image_paths {
                match process_image(image_path, &command) {
                    Ok(output) => all_outputs.push(output),
                    Err(e) => {
                        eprintln!("Error processing '{}': {}", image_path, e);
                        // Continue processing other files
                    }
                }
            }

            // Output all results as JSON array
            let json = serde_json::to_string_pretty(&all_outputs)?;
            println!("{}", json);
        }
    }

    Ok(())
}
