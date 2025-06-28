//! Command-line tool for extracting EXIF data

use clap::{error::ErrorKind, ArgAction, Parser};
use exif_oxide::binary::{extract_binary_tag, extract_mpf_preview};
use exif_oxide::core::ifd::IfdParser;
use exif_oxide::core::mpf::ParsedMpf;
use exif_oxide::core::print_conv::{apply_print_conv, PrintConvId};
use exif_oxide::core::ExifValue;
use exif_oxide::tables::{
    exif_tags, fujifilm_tags, nikon_tags, olympus_tags, pentax_tags, sony_tags,
};
use exif_oxide::tables::{lookup_canon_tag, lookup_tag};
use serde::Serialize;
use std::collections::HashMap;
use std::io::{self, Write};

/// EXIF metadata extraction tool  
#[derive(Parser)]
#[command(name = "exif-oxide")]
#[command(about = "Extracts EXIF data from image files")]
#[command(after_help = "EXAMPLES:\n  \
exif-oxide photo.jpg                          # All tags as JSON\n  \
exif-oxide photo1.jpg photo2.jpg              # Multiple files\n  \
exif-oxide -G photo.jpg                       # All tags with groups\n  \
exif-oxide -Make -Model photo.jpg             # Only Make and Model\n  \
exif-oxide -b -ThumbnailImage photo.jpg       # Extract thumbnail to stdout\n  \
exif-oxide -b -ThumbnailImage photo.jpg > thumb.jpg  # Save thumbnail")]
#[command(disable_help_flag = true)]
#[command(arg_required_else_help = true)]
struct Cli {
    /// Include group names in output
    #[arg(short = 'G', long)]
    groups: bool,

    /// Show numeric/raw values (no PrintConv)
    #[arg(short = 'n', long)]
    numeric: bool,

    /// Extract raw binary data (requires single tag and single file)
    #[arg(short = 'b', long)]
    binary: bool,

    /// API mode: show both raw and formatted values with full type information
    #[arg(long)]
    api: bool,

    /// Show help information
    #[arg(short = 'h', long = "help", action = ArgAction::Help)]
    help: Option<bool>,

    /// All remaining arguments (files and -TagName patterns)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

/// Command represents the operation to perform
#[derive(Debug)]
enum Command {
    /// List all tags as JSON
    ListAll {
        include_groups: bool,
        show_converted_values: bool,
        api_mode: bool,
    },
    /// List specific tags as JSON
    ListSpecific {
        tags: Vec<String>,
        include_groups: bool,
        show_converted_values: bool,
        api_mode: bool,
    },
    /// Extract binary data for a single tag
    ExtractBinary { tag: String },
}

/// Parse CLI arguments into a Command and extract file list
fn parse_cli_to_command(cli: &Cli) -> Result<(Command, Vec<String>), String> {
    let show_converted_values = !cli.numeric; // -n flag inverts the default

    // Parse remaining args to separate tags from files
    let mut tag_filters = Vec::new();
    let mut image_files = Vec::new();

    for arg in &cli.args {
        if arg.starts_with('-') && !arg.starts_with("--") && arg.len() > 1 {
            // ExifTool-compatible -TagName syntax
            let tag_name = &arg[1..]; // Remove the leading dash
            tag_filters.push(tag_name.to_string());
        } else {
            // This is a file
            image_files.push(arg.clone());
        }
    }

    if image_files.is_empty() {
        return Err("No image file specified".to_string());
    }

    if cli.binary {
        // Binary mode requires exactly one tag and one file
        if tag_filters.len() != 1 {
            return Err("Binary mode (-b) requires exactly one tag specification".to_string());
        }
        if image_files.len() != 1 {
            return Err("Binary mode (-b) requires exactly one image file".to_string());
        }
        Ok((
            Command::ExtractBinary {
                tag: tag_filters[0].clone(),
            },
            image_files,
        ))
    } else if tag_filters.is_empty() {
        // No tags specified, list all
        Ok((
            Command::ListAll {
                include_groups: cli.groups,
                show_converted_values,
                api_mode: cli.api,
            },
            image_files,
        ))
    } else {
        // Specific tags requested
        Ok((
            Command::ListSpecific {
                tags: tag_filters,
                include_groups: cli.groups,
                show_converted_values,
                api_mode: cli.api,
            },
            image_files,
        ))
    }
}

/// Custom parser for ExifTool-style arguments (fallback when clap fails)
fn parse_exiftool_style_args(args: &[String]) -> Result<(Command, Vec<String>), String> {
    let mut include_groups = false;
    let mut binary_mode = false;
    let mut show_converted_values = true; // Default to showing converted values
    let mut api_mode = false;
    let mut tag_filters = Vec::new();
    let mut image_paths = Vec::new();

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];

        if arg == "-G" {
            include_groups = true;
            i += 1;
        } else if arg == "-b" || arg == "--binary" {
            binary_mode = true;
            i += 1;
        } else if arg == "-n" || arg == "--numeric" {
            show_converted_values = false; // Show numeric/raw values only
            i += 1;
        } else if arg == "--api" {
            api_mode = true;
            i += 1;
        } else if arg == "-h" || arg == "--help" {
            // Show help - use clap's help
            let _cli = Cli::parse_from(["exif-oxide", "--help"]);
            unreachable!(); // clap will exit with help
        } else if arg.starts_with('-') && !arg.starts_with("--") && arg.len() > 1 {
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
            api_mode,
        }
    } else {
        // Specific tags requested
        Command::ListSpecific {
            tags: tag_filters,
            include_groups,
            show_converted_values,
            api_mode,
        }
    };

    Ok((command, image_paths))
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

    // Search standard EXIF tags (new table-driven approach)
    for tag in exif_tags::EXIF_TAGS.iter() {
        if tag.name.eq_ignore_ascii_case(name) {
            return Some((tag.id, None)); // EXIF tags don't have groups
        }
    }

    // Search standard EXIF tags (legacy lookup from build.rs)
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

/// Default CLI mode - just the value directly (string | number)
type DefaultTagEntry = serde_json::Value;

/// API mode tag entry with full type information
#[derive(Serialize)]
struct ApiTagEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    raw: Option<ExifValue>,
    formatted: ExifValue,
}

/// Unified tag entry that can serialize to either format
#[derive(Serialize)]
#[serde(untagged)]
enum TagEntry {
    Default(DefaultTagEntry),
    Api(ApiTagEntry),
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
    // First try the new table-driven approach with full PrintConv support
    if let Some(exif_tag) = exif_tags::get_exif_tag(tag_id) {
        return exif_tag.print_conv;
    }

    // Fallback to legacy hardcoded mappings for compatibility
    match tag_id {
        0x8827 => PrintConvId::IsoSpeed,     // ISO Speed
        0x9207 => PrintConvId::MeteringMode, // MeteringMode
        0xA403 => PrintConvId::WhiteBalance, // WhiteBalance
        _ => PrintConvId::None,
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

/// Convert ExifValue to serde_json::Value for CLI mode output
fn exif_value_to_default_value(
    value: &ExifValue,
    print_conv_result: Option<&str>,
) -> serde_json::Value {
    // If we have a PrintConv result, use it as a string
    if let Some(converted) = print_conv_result {
        return serde_json::Value::String(converted.to_string());
    }

    // Otherwise convert the raw value to appropriate JSON types
    match value {
        ExifValue::Ascii(s) => serde_json::Value::String(s.clone()),
        ExifValue::U8(n) => serde_json::Value::Number(serde_json::Number::from(*n)),
        ExifValue::U16(n) => serde_json::Value::Number(serde_json::Number::from(*n)),
        ExifValue::U32(n) => serde_json::Value::Number(serde_json::Number::from(*n)),
        ExifValue::I16(n) => serde_json::Value::Number(serde_json::Number::from(*n)),
        ExifValue::I32(n) => serde_json::Value::Number(serde_json::Number::from(*n)),
        ExifValue::Rational(num, den) => {
            if *den == 0 {
                serde_json::Value::String("undef".to_string())
            } else {
                let float_val = *num as f64 / *den as f64;
                serde_json::Number::from_f64(float_val)
                    .map(serde_json::Value::Number)
                    .unwrap_or_else(|| serde_json::Value::String("undef".to_string()))
            }
        }
        ExifValue::SignedRational(num, den) => {
            if *den == 0 {
                serde_json::Value::String("undef".to_string())
            } else {
                let float_val = *num as f64 / *den as f64;
                serde_json::Number::from_f64(float_val)
                    .map(serde_json::Value::Number)
                    .unwrap_or_else(|| serde_json::Value::String("undef".to_string()))
            }
        }
        // Arrays and complex types become strings
        _ => serde_json::Value::String(format!("{:?}", value)),
    }
}

/// Check if raw and formatted values are equivalent (for smart omission)
fn values_are_equivalent(raw: &ExifValue, formatted: &ExifValue) -> bool {
    match (raw, formatted) {
        (ExifValue::Ascii(a), ExifValue::Ascii(b)) => a == b,
        (raw_val, ExifValue::Ascii(formatted_str)) => {
            // Compare raw value string representation to formatted string
            format!("{:?}", raw_val) == *formatted_str
        }
        _ => false,
    }
}

/// Match ExifTool-style glob patterns for tag filtering
/// Examples:
/// - "Make" matches exactly "Make"
/// - "*Make" matches "CanonMake", "NikonMake", etc.
/// - "Make*" matches "MakeNotes", etc.
/// - "*Model*" matches "CanonModelID", etc.
fn matches_exiftool_pattern(pattern: &str, tag_key: &str) -> bool {
    // Handle group:tag notation - extract just the tag name for matching
    let tag_name = if let Some(colon_pos) = tag_key.find(':') {
        &tag_key[colon_pos + 1..]
    } else {
        tag_key
    };

    // Simple glob matching (case-insensitive)
    let pattern_lower = pattern.to_lowercase();
    let tag_lower = tag_name.to_lowercase();

    if pattern_lower.contains('*') {
        // Handle glob patterns
        if pattern_lower == "*" {
            // Match everything
            true
        } else if let Some(stripped) = pattern_lower
            .strip_prefix('*')
            .and_then(|s| s.strip_suffix('*'))
        {
            // *pattern* - contains match
            tag_lower.contains(stripped)
        } else if let Some(suffix) = pattern_lower.strip_prefix('*') {
            // *pattern - ends with match
            tag_lower.ends_with(suffix)
        } else if let Some(prefix) = pattern_lower.strip_suffix('*') {
            // pattern* - starts with match
            tag_lower.starts_with(prefix)
        } else {
            // pattern*other*pattern - more complex, use simple approach
            // Convert glob to regex-like matching
            let parts: Vec<&str> = pattern_lower.split('*').collect();
            if parts.len() == 2 {
                tag_lower.starts_with(parts[0]) && tag_lower.ends_with(parts[1])
            } else {
                // For now, fall back to contains for complex patterns
                tag_lower.contains(&pattern_lower.replace('*', ""))
            }
        }
    } else {
        // Exact match (case-insensitive)
        tag_lower == pattern_lower
    }
}

/// Process entries and build the tag map
fn build_tag_map(
    entries: &HashMap<u16, ExifValue>,
    include_groups: bool,
    tag_filter: Option<&[String]>,
    show_converted_values: bool,
    api_mode: bool,
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
            // Standard EXIF tag - first try new table-driven approach
            if let Some(exif_tag) = exif_tags::get_exif_tag(*tag_id) {
                let tag_key = format_tag_key(exif_tag.name, Some("ExifIFD"), include_groups);
                (tag_key, Some("ExifIFD".to_string()), exif_tag.print_conv)
            } else if let Some(tag_info) = lookup_tag(*tag_id) {
                // Fallback to legacy lookup for compatibility
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
                // ExifTool-style glob matching
                if matches_exiftool_pattern(filter_tag, &tag_key) {
                    include = true;
                    break;
                }
            }
            if !include {
                continue;
            }
        }

        // Handle name collisions: first-with-a-non-blank-value wins
        let should_insert = if tags.contains_key(&tag_key) {
            // For simplicity, if a tag already exists, don't replace it
            // This preserves the original collision behavior
            false
        } else {
            true
        };

        if should_insert {
            let tag_entry = if api_mode {
                // API mode: return both raw and formatted values
                let formatted_value = if should_replace_with_length(value, &tag_key) {
                    convert_to_binary_length(value)
                } else if show_converted_values && print_conv_id != PrintConvId::None {
                    // Apply PrintConv to get human-readable value
                    let converted_string = apply_print_conv(value, print_conv_id);
                    ExifValue::Ascii(converted_string)
                } else if show_converted_values {
                    // Convert common cases like Undefined strings
                    match value {
                        ExifValue::Undefined(data) => {
                            if let Some(null_pos) = data.iter().position(|&b| b == 0) {
                                match std::str::from_utf8(&data[..null_pos]) {
                                    Ok(s) => ExifValue::Ascii(s.to_string()),
                                    Err(_) => value.clone(),
                                }
                            } else if data.iter().all(|&b| {
                                b.is_ascii() && (b.is_ascii_graphic() || b.is_ascii_whitespace())
                            }) {
                                match std::str::from_utf8(data) {
                                    Ok(s) => ExifValue::Ascii(s.to_string()),
                                    Err(_) => value.clone(),
                                }
                            } else {
                                value.clone()
                            }
                        }
                        _ => value.clone(),
                    }
                } else {
                    value.clone()
                };

                // Check if we should omit raw (when it's the same as formatted)
                let raw_value = if values_are_equivalent(value, &formatted_value) {
                    None
                } else {
                    Some(value.clone())
                };

                TagEntry::Api(ApiTagEntry {
                    group: if include_groups { None } else { group },
                    raw: raw_value,
                    formatted: formatted_value,
                })
            } else {
                // Default CLI mode: return simple string/number values
                let print_conv_result =
                    if show_converted_values && print_conv_id != PrintConvId::None {
                        Some(apply_print_conv(value, print_conv_id))
                    } else {
                        None
                    };

                let default_value =
                    exif_value_to_default_value(value, print_conv_result.as_deref());

                TagEntry::Default(default_value)
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
    // Extract all metadata segments using format dispatch
    let metadata_collection = exif_oxide::core::find_all_metadata_segments(image_path)?;

    // Start with EXIF entries if available
    let mut entries = std::collections::HashMap::new();

    if let Some(exif_segment) = metadata_collection.exif {
        // Parse IFD to get all EXIF data
        let ifd = IfdParser::parse(exif_segment.data.clone())?;
        entries.extend(ifd.entries().clone());
    }

    // Check if we have any data at all (EXIF or GPMF)
    let has_exif = !entries.is_empty();
    let has_gpmf = !metadata_collection.gpmf.is_empty();

    if !has_exif && !has_gpmf {
        return Err(format!("No EXIF or GPMF data found in '{}'", image_path).into());
    }

    // Process based on command type
    match command {
        Command::ListAll {
            include_groups,
            show_converted_values,
            api_mode,
        } => {
            // Build complete tag map from EXIF data
            let mut tags = if has_exif {
                build_tag_map(
                    &entries,
                    *include_groups,
                    None,
                    *show_converted_values,
                    *api_mode,
                )
            } else {
                HashMap::new()
            };

            // Add GPMF data if available
            if has_gpmf {
                for gpmf_segment in &metadata_collection.gpmf {
                    let gpmf_parser = exif_oxide::gpmf::GpmfParser::new();
                    if let Ok(gpmf_data) = gpmf_parser.parse(&gpmf_segment.data) {
                        for (tag_id, value) in gpmf_data {
                            let tag_key = if *include_groups {
                                tag_id.clone()
                            } else {
                                format!("GPMF:{}", tag_id)
                            };

                            let tag_entry = if *api_mode {
                                TagEntry::Api(ApiTagEntry {
                                    group: if *include_groups {
                                        None
                                    } else {
                                        Some("GPMF".to_string())
                                    },
                                    raw: Some(value.clone()),
                                    formatted: value,
                                })
                            } else {
                                let json_value = exif_value_to_default_value(&value, None);
                                TagEntry::Default(json_value)
                            };

                            tags.insert(tag_key, tag_entry);
                        }
                    }
                }
            }

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

            let file_tag_entry = if *api_mode {
                TagEntry::Api(ApiTagEntry {
                    group: if *include_groups {
                        None
                    } else {
                        Some("File".to_string())
                    },
                    raw: None, // File metadata doesn't need raw values
                    formatted: ExifValue::Ascii(file_name.to_string()),
                })
            } else {
                TagEntry::Default(serde_json::Value::String(file_name.to_string()))
            };

            let dir_tag_entry = if *api_mode {
                TagEntry::Api(ApiTagEntry {
                    group: if *include_groups {
                        None
                    } else {
                        Some("File".to_string())
                    },
                    raw: None, // File metadata doesn't need raw values
                    formatted: ExifValue::Ascii(directory.to_string()),
                })
            } else {
                TagEntry::Default(serde_json::Value::String(directory.to_string()))
            };

            tags.insert(filename_key, file_tag_entry);
            tags.insert(directory_key, dir_tag_entry);

            Ok(FileOutput {
                source_file: image_path.to_string(),
                tags,
            })
        }
        Command::ListSpecific {
            tags: tag_filter,
            include_groups,
            show_converted_values,
            api_mode,
        } => {
            // Build filtered tag map from EXIF data
            let mut tags = if has_exif {
                build_tag_map(
                    &entries,
                    *include_groups,
                    Some(tag_filter),
                    *show_converted_values,
                    *api_mode,
                )
            } else {
                HashMap::new()
            };

            // Add GPMF data if available and requested
            if has_gpmf {
                for gpmf_segment in &metadata_collection.gpmf {
                    let gpmf_parser = exif_oxide::gpmf::GpmfParser::new();
                    if let Ok(gpmf_data) = gpmf_parser.parse(&gpmf_segment.data) {
                        for (tag_id, value) in gpmf_data {
                            let tag_key = if *include_groups {
                                tag_id.clone()
                            } else {
                                format!("GPMF:{}", tag_id)
                            };

                            // Check if this GPMF tag matches the filter
                            let mut include = false;
                            for filter_tag in tag_filter {
                                if matches_exiftool_pattern(filter_tag, &tag_key)
                                    || matches_exiftool_pattern(filter_tag, &tag_id)
                                {
                                    include = true;
                                    break;
                                }
                            }

                            if include {
                                let tag_entry = if *api_mode {
                                    TagEntry::Api(ApiTagEntry {
                                        group: if *include_groups {
                                            None
                                        } else {
                                            Some("GPMF".to_string())
                                        },
                                        raw: Some(value.clone()),
                                        formatted: value,
                                    })
                                } else {
                                    let json_value = exif_value_to_default_value(&value, None);
                                    TagEntry::Default(json_value)
                                };

                                tags.insert(tag_key, tag_entry);
                            }
                        }
                    }
                }
            }

            // Check if file metadata was requested
            for filter in tag_filter {
                if filter.eq_ignore_ascii_case("FileName") {
                    let file_name = std::path::Path::new(image_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(image_path);
                    let filename_key = format_tag_key("FileName", Some("File"), *include_groups);

                    let file_tag_entry = if *api_mode {
                        TagEntry::Api(ApiTagEntry {
                            group: if *include_groups {
                                None
                            } else {
                                Some("File".to_string())
                            },
                            raw: None,
                            formatted: ExifValue::Ascii(file_name.to_string()),
                        })
                    } else {
                        TagEntry::Default(serde_json::Value::String(file_name.to_string()))
                    };

                    tags.insert(filename_key, file_tag_entry);
                }
                if filter.eq_ignore_ascii_case("Directory") {
                    let directory = std::path::Path::new(image_path)
                        .parent()
                        .and_then(|p| p.to_str())
                        .unwrap_or(".");
                    let directory_key = format_tag_key("Directory", Some("File"), *include_groups);

                    let dir_tag_entry = if *api_mode {
                        TagEntry::Api(ApiTagEntry {
                            group: if *include_groups {
                                None
                            } else {
                                Some("File".to_string())
                            },
                            raw: None,
                            formatted: ExifValue::Ascii(directory.to_string()),
                        })
                    } else {
                        TagEntry::Default(serde_json::Value::String(directory.to_string()))
                    };

                    tags.insert(directory_key, dir_tag_entry);
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
    // Custom argument parsing to handle ExifTool-style -TagName syntax
    let args: Vec<String> = std::env::args().collect();

    // Try clap parsing first, but if it fails with unknown arguments, use custom parsing
    let (command, image_files) = match Cli::try_parse() {
        Ok(cli) => match parse_cli_to_command(&cli) {
            Ok(result) => result,
            Err(err) => {
                eprintln!("Error: {}", err);
                std::process::exit(1);
            }
        },
        Err(clap_err) if clap_err.kind() == ErrorKind::UnknownArgument => {
            // Fallback to custom parsing for ExifTool-style arguments
            match parse_exiftool_style_args(&args) {
                Ok(result) => result,
                Err(err) => {
                    eprintln!("Error: {}", err);
                    std::process::exit(1);
                }
            }
        }
        Err(clap_err) => {
            clap_err.exit();
        }
    };

    // Handle different command types
    match command {
        Command::ExtractBinary { tag } => {
            // Extract and output binary data
            let binary_data = extract_binary(&tag, &image_files[0])?;
            io::stdout().write_all(&binary_data)?;
            io::stdout().flush()?;
        }
        _ => {
            // Process all files and collect results
            let mut all_outputs = Vec::new();

            for image_path in &image_files {
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
