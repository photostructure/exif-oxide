//! Command-line tool for extracting EXIF data

use exif_oxide::core::ifd::IfdParser;
use exif_oxide::core::jpeg;
use exif_oxide::core::ExifValue;
use exif_oxide::extract::thumbnail::extract_thumbnail;
use exif_oxide::tables::{lookup_canon_tag, lookup_tag};
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::process;

/// Command represents the operation to perform
#[derive(Debug)]
enum Command {
    /// List all tags as JSON
    ListAll { include_groups: bool },
    /// List specific tags as JSON
    ListSpecific {
        tags: Vec<String>,
        include_groups: bool,
    },
    /// Extract binary data for a single tag
    ExtractBinary { tag: String },
}

/// Parse command line arguments into a Command
fn parse_args(args: Vec<String>) -> Result<(Command, String), String> {
    let mut include_groups = false;
    let mut binary_mode = false;
    let mut tag_filters = Vec::new();
    let mut image_path = None;

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];

        if arg == "-G" {
            include_groups = true;
            i += 1;
        } else if arg == "-b" {
            binary_mode = true;
            i += 1;
        } else if arg.starts_with('-') && !arg.starts_with("--") {
            // This is a tag specification (e.g., -ThumbnailImage)
            let tag_name = &arg[1..]; // Remove the leading dash
            tag_filters.push(tag_name.to_string());
            i += 1;
        } else if arg.starts_with('-') {
            return Err(format!("Unknown option: {}", arg));
        } else {
            // This is the image file path
            if image_path.is_some() {
                return Err("Multiple image files specified".to_string());
            }
            image_path = Some(arg.clone());
            i += 1;
        }
    }

    let image_path = image_path.ok_or_else(|| "No image file specified".to_string())?;

    // Determine the command based on parsed arguments
    let command = if binary_mode {
        // Binary mode requires exactly one tag
        if tag_filters.len() != 1 {
            return Err("Binary mode (-b) requires exactly one tag specification".to_string());
        }
        Command::ExtractBinary {
            tag: tag_filters.into_iter().next().unwrap(),
        }
    } else if tag_filters.is_empty() {
        // No tags specified, list all
        Command::ListAll { include_groups }
    } else {
        // Specific tags requested
        Command::ListSpecific {
            tags: tag_filters,
            include_groups,
        }
    };

    Ok((command, image_path))
}

/// Display usage information
fn show_usage(program: &str) {
    eprintln!("Usage: {} [-G] [-b] [-TagName]... <image_file>", program);
    eprintln!();
    eprintln!("Extracts EXIF data from image files.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -G         Include group names in output");
    eprintln!("  -b         Extract raw binary data (requires single -TagName)");
    eprintln!("  -TagName   Include only specified tag(s) in output");
    eprintln!();
    eprintln!("Examples:");
    eprintln!(
        "  {} photo.jpg                          # All tags as JSON",
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

    // Special handling for common thumbnail tags
    match name.to_lowercase().as_str() {
        "thumbnailimage" => return Some((0x1201, Some("IFD1"))), // IFD1 tag with prefix
        "thumbnailoffset" => return Some((0x1201, Some("IFD1"))),
        "thumbnaillength" => return Some((0x1202, Some("IFD1"))),
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
fn extract_binary(
    ifd: &exif_oxide::core::ifd::ParsedIfd,
    tag_name: &str,
    exif_data: &[u8],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Resolve tag name to ID
    let (tag_id, _group) =
        resolve_tag_name(tag_name).ok_or_else(|| format!("Unknown tag: {}", tag_name))?;

    // Special handling for ThumbnailImage
    if tag_id == 0x1201 {
        // Use the thumbnail extraction function
        if let Some(thumbnail) = extract_thumbnail(ifd, exif_data)? {
            return Ok(thumbnail);
        } else {
            return Err("No thumbnail found in image".into());
        }
    }

    // Get the tag value
    let value = ifd
        .entries()
        .get(&tag_id)
        .ok_or_else(|| format!("Tag not found: {}", tag_name))?;

    // Extract binary data based on value type
    match value {
        ExifValue::Undefined(data) => Ok(data.clone()),
        ExifValue::U8Array(data) => Ok(data.clone()),
        _ => Err(format!("Tag {} does not contain binary data", tag_name).into()),
    }
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

/// Process entries and build the tag map
fn build_tag_map(
    entries: &HashMap<u16, ExifValue>,
    include_groups: bool,
    tag_filter: Option<&[String]>,
) -> HashMap<String, TagEntry> {
    let mut tags: HashMap<String, TagEntry> = HashMap::new();

    for (tag_id, value) in entries {
        let (tag_key, group) = if *tag_id >= 0xC000 {
            // This is a maker note tag (prefixed with 0xC000)
            let original_tag = tag_id - 0xC000;
            if let Some(tag_info) = lookup_canon_tag(original_tag) {
                let tag_name = tag_info.name;
                let tag_key = format_tag_key(tag_name, Some("Canon"), include_groups);
                (tag_key, Some("Canon".to_string()))
            } else {
                let tag_name = format!("Unknown{:04X}", original_tag);
                let tag_key = format_tag_key(&tag_name, Some("Canon"), include_groups);
                (tag_key, Some("Canon".to_string()))
            }
        } else if *tag_id >= 0x1000 {
            // This is an IFD1 tag (prefixed with 0x1000)
            let original_tag = tag_id - 0x1000;
            if let Some(tag_info) = lookup_tag(original_tag) {
                let tag_name = tag_info.name;
                let tag_key = format_tag_key(tag_name, Some("IFD1"), include_groups);
                (tag_key, Some("IFD1".to_string()))
            } else {
                let tag_name = format!("0x{:04X}", original_tag);
                let tag_key = format_tag_key(&tag_name, Some("IFD1"), include_groups);
                (tag_key, Some("IFD1".to_string()))
            }
        } else {
            // Standard EXIF tag
            if let Some(tag_info) = lookup_tag(*tag_id) {
                let group = tag_info.group.unwrap_or("ExifIFD");
                let tag_key = format_tag_key(tag_info.name, Some(group), include_groups);
                (tag_key, Some(group.to_string()))
            } else {
                let tag_name = format!("0x{:04X}", tag_id);
                let tag_key = format_tag_key(&tag_name, Some("Unknown"), include_groups);
                (tag_key, Some("Unknown".to_string()))
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // Parse command line arguments
    let (command, image_path) = match parse_args(args.clone()) {
        Ok(result) => result,
        Err(err) => {
            eprintln!("Error: {}", err);
            eprintln!();
            show_usage(&args[0]);
            process::exit(1);
        }
    };

    // Open the image file
    let mut file =
        File::open(&image_path).map_err(|e| format!("Failed to open '{}': {}", image_path, e))?;

    // Extract EXIF segment
    let exif_segment = jpeg::find_exif_segment(&mut file)?.ok_or("No EXIF data found in image")?;

    // Parse IFD to get all EXIF data
    let ifd = IfdParser::parse(exif_segment.data.clone())?;

    // Execute the command
    match command {
        Command::ExtractBinary { tag } => {
            // Extract and output binary data
            let binary_data = extract_binary(&ifd, &tag, &exif_segment.data)?;
            io::stdout().write_all(&binary_data)?;
            io::stdout().flush()?;
        }
        Command::ListAll { include_groups } => {
            // Build complete tag map
            let mut tags = build_tag_map(ifd.entries(), include_groups, None);

            // Add file metadata
            let file_name = std::path::Path::new(&image_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&image_path);
            let directory = std::path::Path::new(&image_path)
                .parent()
                .and_then(|p| p.to_str())
                .unwrap_or(".");

            let filename_key = format_tag_key("FileName", Some("File"), include_groups);
            let directory_key = format_tag_key("Directory", Some("File"), include_groups);

            tags.insert(
                filename_key,
                TagEntry {
                    group: if include_groups {
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
                    group: if include_groups {
                        None
                    } else {
                        Some("File".to_string())
                    },
                    value: ExifValue::Ascii(directory.to_string()),
                },
            );

            // Output as JSON array
            let output = FileOutput {
                source_file: image_path,
                tags,
            };
            let output_array = vec![output];
            let json = serde_json::to_string_pretty(&output_array)?;
            println!("{}", json);
        }
        Command::ListSpecific {
            tags: tag_filter,
            include_groups,
        } => {
            // Build filtered tag map
            let mut tags = build_tag_map(ifd.entries(), include_groups, Some(&tag_filter));

            // Check if file metadata was requested
            for filter in &tag_filter {
                if filter.eq_ignore_ascii_case("FileName") {
                    let file_name = std::path::Path::new(&image_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(&image_path);
                    let filename_key = format_tag_key("FileName", Some("File"), include_groups);
                    tags.insert(
                        filename_key,
                        TagEntry {
                            group: if include_groups {
                                None
                            } else {
                                Some("File".to_string())
                            },
                            value: ExifValue::Ascii(file_name.to_string()),
                        },
                    );
                }
                if filter.eq_ignore_ascii_case("Directory") {
                    let directory = std::path::Path::new(&image_path)
                        .parent()
                        .and_then(|p| p.to_str())
                        .unwrap_or(".");
                    let directory_key = format_tag_key("Directory", Some("File"), include_groups);
                    tags.insert(
                        directory_key,
                        TagEntry {
                            group: if include_groups {
                                None
                            } else {
                                Some("File".to_string())
                            },
                            value: ExifValue::Ascii(directory.to_string()),
                        },
                    );
                }
            }

            // Output as JSON array
            let output = FileOutput {
                source_file: image_path,
                tags,
            };
            let output_array = vec![output];
            let json = serde_json::to_string_pretty(&output_array)?;
            println!("{}", json);
        }
    }

    Ok(())
}
