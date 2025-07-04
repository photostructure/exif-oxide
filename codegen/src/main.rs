//! Rust code generation tool for exif-oxide
//!
//! This tool reads JSON output from extract_tables.pl and generates
//! Rust code following the "runtime references, no stubs" architecture.

use anyhow::{Context, Result};
use clap::{Arg, Command};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Find the repository root by walking up from the given path until we find Cargo.toml
fn find_repo_root(start_path: &Path) -> Result<PathBuf> {
    let mut current_path = start_path.canonicalize()?;
    while !current_path.join("Cargo.toml").exists() {
        current_path = current_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Could not find repository root (Cargo.toml)"))?
            .to_path_buf();
    }
    Ok(current_path)
}

/// JSON structure from extract_tables.pl
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ExtractedData {
    extracted_at: String,
    exiftool_version: String,
    filter_criteria: String,
    total_tags: usize,
    tags: Vec<ExtractedTag>,
    #[serde(default)]
    composite_tags: Vec<ExtractedCompositeTag>,
    conversion_refs: ConversionRefs,
}

/// Conversion references extracted from tag definitions
#[derive(Debug, Deserialize)]
struct ConversionRefs {
    print_conv: Vec<String>,
    value_conv: Vec<String>,
}

/// JSON structure from extract_simple_tables.pl
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SimpleTablesData {
    extracted_at: String,
    extraction_config: String,
    total_tables: usize,
    tables: HashMap<String, ExtractedTable>,
}

/// Extracted table data from extract_simple_tables.pl
#[derive(Debug, Deserialize)]
struct ExtractedTable {
    config: TableConfig,
    entries: Vec<TableEntry>,
    entry_count: usize,
    #[serde(default)]
    extraction_type: Option<String>,
}

/// Table configuration from simple_tables.json
#[derive(Debug, Deserialize)]
struct TableConfig {
    module: String,
    output_file: String,
    #[serde(default)]
    constant_name: Option<String>,
    #[serde(default)]
    key_type: Option<String>,
    #[serde(default)]
    extraction_type: Option<String>,
    description: String,
}

/// Individual table entry
#[derive(Debug, Deserialize, Clone)]
struct TableEntry {
    #[serde(default)]
    key: Option<String>,
    #[serde(default)]
    value: Option<String>,
    #[serde(default)]
    rust_compatible: Option<bool>,
    #[serde(default)]
    compatibility_notes: Option<String>,
    // File type lookup specific fields
    #[serde(default)]
    extension: Option<String>,
    #[serde(default)]
    entry_type: Option<String>,
    #[serde(default)]
    target: Option<String>,
    #[serde(default)]
    formats: Option<Vec<String>>,
    #[serde(default)]
    description: Option<String>,
}


/// Individual tag extracted from ExifTool
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ExtractedTag {
    id: String,
    name: String,
    format: String,
    groups: Vec<String>,
    writable: u8,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    print_conv_ref: Option<String>,
    #[serde(default)]
    value_conv_ref: Option<String>,
    #[serde(default)]
    frequency: Option<f64>,
    #[serde(default)]
    mainstream: Option<u8>,
    #[serde(default)]
    notes: Option<String>,
}

/// Individual composite tag extracted from ExifTool
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ExtractedCompositeTag {
    name: String,
    table: String,
    full_name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    require: Option<HashMap<String, String>>,
    #[serde(default)]
    desire: Option<HashMap<String, String>>,
    #[serde(default)]
    inhibit: Option<HashMap<String, String>>,
    #[serde(default)]
    value_conv: Option<String>,
    #[serde(default)]
    raw_conv: Option<String>,
    #[serde(default)]
    print_conv_ref: Option<String>,
    #[serde(default)]
    groups: Option<HashMap<String, String>>,
    #[serde(default)]
    writable: Option<u8>,
    #[serde(default)]
    avoid: Option<u8>,
    #[serde(default)]
    priority: Option<i32>,
    #[serde(default)]
    sub_doc: Option<serde_json::Value>, // Can be bool or array
    #[serde(default)]
    frequency: Option<f64>,
    #[serde(default)]
    mainstream: Option<u8>,
    #[serde(default)]
    notes: Option<String>,
}

/// Generated Rust tag structure
#[derive(Debug, Serialize)]
struct GeneratedTag {
    id: u32,
    name: String,
    format: String,
    groups: Vec<String>,
    writable: bool,
    description: Option<String>,
    print_conv_ref: Option<String>,
    value_conv_ref: Option<String>,
    notes: Option<String>,
}

/// Generated Rust composite tag structure
#[derive(Debug, Serialize)]
struct GeneratedCompositeTag {
    name: String,
    table: String,
    description: Option<String>,
    require: HashMap<u8, String>,
    desire: HashMap<u8, String>,
    value_conv: Option<String>,
    print_conv_ref: Option<String>,
    groups: HashMap<u8, String>,
    writable: bool,
    avoid: bool,
    priority: i32,
    notes: Option<String>,
}

fn main() -> Result<()> {
    let matches = Command::new("generate_rust")
        .version("0.1.0")
        .author("exif-oxide@photostructure.com")
        .about("Generate Rust code from ExifTool tag extraction")
        .arg(
            Arg::new("input")
                .help("JSON file from extract_tables.pl")
                .required(false)
                .value_name("FILE")
                .default_value("generated/tag_tables.json")
                .index(1),
        )
        .arg(
            Arg::new("output-dir")
                .long("output-dir")
                .help("Output directory for generated code")
                .value_name("DIR")
                .default_value("../src/generated"),
        )
        .get_matches();

    let input_file = matches.get_one::<String>("input").unwrap();
    let output_dir = matches.get_one::<String>("output-dir").unwrap();

    // Check if this is a simple tables file or tag tables file
    if input_file.contains("simple_tables.json") {
        // Handle simple tables only
        generate_simple_tables(input_file, output_dir)?;
        return Ok(());
    }

    // Read and parse input JSON for regular tag tables
    let json_content = fs::read_to_string(input_file)
        .with_context(|| format!("Failed to read input file: {}", input_file))?;

    let extracted_data: ExtractedData =
        serde_json::from_str(&json_content).with_context(|| "Failed to parse JSON input")?;

    println!(
        "Loaded {} tags and {} composite tags from ExifTool {}",
        extracted_data.total_tags,
        extracted_data.composite_tags.len(),
        extracted_data.exiftool_version
    );

    // Convert extracted tags to generated format
    let mut tags = Vec::new();

    for tag in extracted_data.tags {
        // Parse hex ID
        let id = parse_hex_id(&tag.id)?;

        // Custom ValueConv implementations for exif-oxide TagEntry API (Milestone 8b+)
        //
        // Note: FNumber, ExposureTime, and FocalLength don't have ValueConv in ExifTool
        // because they're already stored as rationals. We add custom ValueConv to convert
        // to float for easier API usage while preserving the original rational data.
        // GPS coordinate ValueConv matches ExifTool GPS.pm %coordConv with ToDegrees.
        let value_conv_ref = match tag.name.as_str() {
            "FNumber" => Some("fnumber_value_conv".to_string()),
            "ExposureTime" => Some("exposuretime_value_conv".to_string()),
            "FocalLength" => Some("focallength_value_conv".to_string()),
            // GPS coordinate ValueConv - ExifTool GPS.pm uses %coordConv with ToDegrees
            "GPSLatitude" => Some("gpslatitude_value_conv".to_string()),
            "GPSLongitude" => Some("gpslongitude_value_conv".to_string()),
            "GPSDestLatitude" => Some("gpsdestlatitude_value_conv".to_string()),
            "GPSDestLongitude" => Some("gpsdestlongitude_value_conv".to_string()),
            _ => tag.value_conv_ref.clone(),
        };

        let generated_tag = GeneratedTag {
            id,
            name: tag.name.clone(),
            format: normalize_format(&tag.format),
            groups: tag.groups,
            writable: tag.writable != 0,
            description: tag.description,
            print_conv_ref: tag.print_conv_ref.clone(),
            value_conv_ref,
            notes: tag.notes,
        };

        tags.push(generated_tag);
    }

    // Convert extracted composite tags to generated format
    let mut composite_tags = Vec::new();

    for comp_tag in extracted_data.composite_tags {
        let generated_comp_tag = GeneratedCompositeTag {
            name: comp_tag.name,
            table: comp_tag.table,
            description: comp_tag.description,
            require: parse_dependency_map(comp_tag.require.unwrap_or_default()),
            desire: parse_dependency_map(comp_tag.desire.unwrap_or_default()),
            value_conv: comp_tag.value_conv,
            print_conv_ref: comp_tag.print_conv_ref,
            groups: parse_group_map(comp_tag.groups.unwrap_or_default()),
            writable: comp_tag.writable.unwrap_or(0) != 0,
            avoid: comp_tag.avoid.unwrap_or(0) != 0,
            priority: comp_tag.priority.unwrap_or(0),
            notes: comp_tag.notes,
        };

        composite_tags.push(generated_comp_tag);
    }

    println!("Extracted conversion references:");
    println!(
        "  PrintConv: {} functions",
        extracted_data.conversion_refs.print_conv.len()
    );
    println!(
        "  ValueConv: {} functions",
        extracted_data.conversion_refs.value_conv.len()
    );

    // Create output directory
    fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create output directory: {}", output_dir))?;

    // Generate tag table
    generate_tag_table(&tags, output_dir)?;

    // Generate composite tag table
    generate_composite_tag_table(&composite_tags, output_dir)?;

    // Generate conversion registry from extracted references
    generate_conversion_refs(&extracted_data.conversion_refs, &tags, output_dir)?;

    // Generate supported tags list for DRY compliance
    generate_supported_tags(&tags, output_dir)?;

    // Generate simple tables if data exists
    let simple_tables_path = "generated/simple_tables.json";
    if Path::new(simple_tables_path).exists() {
        generate_simple_tables(simple_tables_path, output_dir)?;
    }

    // Generate module file
    generate_mod_file(output_dir)?;

    let total_conv_refs = extracted_data.conversion_refs.print_conv.len()
        + extracted_data.conversion_refs.value_conv.len();
    println!(
        "Generated {} tags and {} composite tags with {} conversion references",
        tags.len(),
        composite_tags.len(),
        total_conv_refs
    );
    println!("Code generated in: {}", output_dir);
    println!("\nNext steps:");
    println!("1. Add 'mod generated;' to src/lib.rs");
    println!("2. Use --show-missing on real images to see what implementations are needed");
    println!(
        "3. Implement missing PrintConv/ValueConv and composite functions in implementations/"
    );

    Ok(())
}

fn parse_hex_id(hex_str: &str) -> Result<u32> {
    let cleaned = hex_str.trim_start_matches("0x");
    u32::from_str_radix(cleaned, 16).with_context(|| format!("Failed to parse hex ID: {}", hex_str))
}

fn normalize_format(format: &str) -> String {
    // Convert ExifTool formats to our format enum names
    match format {
        "int8u" => "U8",
        "int16u" => "U16",
        "int32u" => "U32",
        "int8s" => "I8",
        "int16s" => "I16",
        "int32s" => "I32",
        "rational64u" => "RationalU",
        "rational64s" => "RationalS",
        "string" => "String",
        "undef" | "binary" => "Undef",
        "float" => "Float",
        "double" => "Double",
        _ => "Undef", // Unknown formats default to undefined
    }
    .to_string()
}

fn generate_tag_table(tags: &[GeneratedTag], output_dir: &str) -> Result<()> {
    let mut code = String::new();

    // File header
    code.push_str("//! Generated EXIF tag definitions\n");
    code.push_str("//!\n");
    code.push_str("//! This file is automatically generated by codegen/generate_rust.\n");
    code.push_str("//! DO NOT EDIT MANUALLY - changes will be overwritten.\n\n");

    code.push_str("use std::collections::HashMap;\n");
    code.push_str("use std::sync::LazyLock;\n\n");

    // Format enum
    code.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n");
    code.push_str("pub enum TagFormat {\n");
    code.push_str("    U8,\n");
    code.push_str("    U16,\n");
    code.push_str("    U32,\n");
    code.push_str("    I8,\n");
    code.push_str("    I16,\n");
    code.push_str("    I32,\n");
    code.push_str("    RationalU,\n");
    code.push_str("    RationalS,\n");
    code.push_str("    String,\n");
    code.push_str("    Undef,\n");
    code.push_str("    Float,\n");
    code.push_str("    Double,\n");
    code.push_str("}\n\n");

    // Tag structure
    code.push_str("#[derive(Debug, Clone)]\n");
    code.push_str("pub struct TagDef {\n");
    code.push_str("    pub id: u32,\n");
    code.push_str("    pub name: &'static str,\n");
    code.push_str("    pub format: TagFormat,\n");
    code.push_str("    pub groups: &'static [&'static str],\n");
    code.push_str("    pub writable: bool,\n");
    code.push_str("    pub description: Option<&'static str>,\n");
    code.push_str("    pub print_conv_ref: Option<&'static str>,\n");
    code.push_str("    pub value_conv_ref: Option<&'static str>,\n");
    code.push_str("    pub notes: Option<&'static str>,\n");
    code.push_str("}\n\n");

    // Static tag array
    code.push_str("pub static EXIF_MAIN_TAGS: &[TagDef] = &[\n");

    for tag in tags {
        code.push_str("    TagDef {\n");
        code.push_str(&format!("        id: 0x{:x},\n", tag.id));
        code.push_str(&format!("        name: \"{}\",\n", tag.name));
        code.push_str(&format!("        format: TagFormat::{},\n", tag.format));

        // Groups array
        code.push_str("        groups: &[");
        for (i, group) in tag.groups.iter().enumerate() {
            if i > 0 {
                code.push_str(", ");
            }
            code.push_str(&format!("\"{}\"", group));
        }
        code.push_str("],\n");

        code.push_str(&format!("        writable: {},\n", tag.writable));

        // Optional fields
        if let Some(desc) = &tag.description {
            code.push_str(&format!(
                "        description: Some(\"{}\"),\n",
                escape_string(desc)
            ));
        } else {
            code.push_str("        description: None,\n");
        }

        if let Some(print_ref) = &tag.print_conv_ref {
            code.push_str(&format!(
                "        print_conv_ref: Some(\"{}\"),\n",
                print_ref
            ));
        } else {
            code.push_str("        print_conv_ref: None,\n");
        }

        if let Some(value_ref) = &tag.value_conv_ref {
            code.push_str(&format!(
                "        value_conv_ref: Some(\"{}\"),\n",
                value_ref
            ));
        } else {
            code.push_str("        value_conv_ref: None,\n");
        }

        if let Some(notes) = &tag.notes {
            code.push_str(&format!(
                "        notes: Some(\"{}\"),\n",
                escape_string(notes)
            ));
        } else {
            code.push_str("        notes: None,\n");
        }

        code.push_str("    },\n");
    }

    code.push_str("];\n\n");

    // Lookup by ID
    code.push_str(
        "pub static TAG_BY_ID: LazyLock<HashMap<u32, &'static TagDef>> = LazyLock::new(|| {\n",
    );
    code.push_str("    let mut map = HashMap::new();\n");
    code.push_str("    for tag in EXIF_MAIN_TAGS {\n");
    code.push_str("        map.insert(tag.id, tag);\n");
    code.push_str("    }\n");
    code.push_str("    map\n");
    code.push_str("});\n\n");

    // Lookup by name
    code.push_str("pub static TAG_BY_NAME: LazyLock<HashMap<&'static str, &'static TagDef>> = LazyLock::new(|| {\n");
    code.push_str("    let mut map = HashMap::new();\n");
    code.push_str("    for tag in EXIF_MAIN_TAGS {\n");
    code.push_str("        map.insert(tag.name, tag);\n");
    code.push_str("    }\n");
    code.push_str("    map\n");
    code.push_str("});\n");

    // Write file
    let output_path = Path::new(output_dir).join("tags.rs");
    fs::write(&output_path, code)
        .with_context(|| format!("Failed to write tags.rs to {:?}", output_path))?;

    println!("Generated: tags.rs");
    Ok(())
}

fn generate_conversion_refs(
    conversion_refs: &ConversionRefs,
    tags: &[GeneratedTag],
    output_dir: &str,
) -> Result<()> {
    let mut code = String::new();

    // File header
    code.push_str("//! Generated conversion registry stubs\n");
    code.push_str("//!\n");
    code.push_str("//! This file is automatically generated by codegen/generate_rust.\n");
    code.push_str("//! DO NOT EDIT MANUALLY - changes will be overwritten.\n");
    code.push_str("//!\n");
    code.push_str("//! These are just the registry keys - actual implementations\n");
    code.push_str("//! must be provided in the implementations/ directory.\n\n");

    code.push_str("use std::collections::HashSet;\n");
    code.push_str("use std::sync::LazyLock;\n\n");

    // Collect all ValueConv references actually used in generated tags (including manual ones)
    let mut all_value_conv_refs = std::collections::HashSet::new();
    for tag in tags {
        if let Some(value_ref) = &tag.value_conv_ref {
            all_value_conv_refs.insert(value_ref.clone());
        }
    }

    // Convert to sorted vector for consistent output
    let mut sorted_value_refs: Vec<_> = all_value_conv_refs.into_iter().collect();
    sorted_value_refs.sort();

    // List all required PrintConv references
    code.push_str("/// All PrintConv references that need implementation\n");
    code.push_str("/// These match the print_conv_ref values used in generated/tags.rs\n");
    code.push_str("pub static REQUIRED_PRINT_CONV: &[&str] = &[\n");
    for conv_ref in &conversion_refs.print_conv {
        code.push_str(&format!("    \"{}\",\n", conv_ref));
    }
    code.push_str("];\n\n");

    // List all required ValueConv references
    code.push_str("/// All ValueConv references that need implementation\n");
    code.push_str("/// Includes both ExifTool-extracted and custom implementations\n");
    code.push_str("pub static REQUIRED_VALUE_CONV: &[&str] = &[\n");
    for conv_ref in &sorted_value_refs {
        code.push_str(&format!("    \"{}\",\n", conv_ref));
    }
    code.push_str("];\n\n");

    // Runtime check functions
    code.push_str(
        "static REQUIRED_PRINT_CONV_SET: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {\n",
    );
    code.push_str("    REQUIRED_PRINT_CONV.iter().copied().collect()\n");
    code.push_str("});\n\n");
    code.push_str(
        "static REQUIRED_VALUE_CONV_SET: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {\n",
    );
    code.push_str("    REQUIRED_VALUE_CONV.iter().copied().collect()\n");
    code.push_str("});\n\n");

    code.push_str("/// Check if a PrintConv reference is required by generated tags\n");
    code.push_str("pub fn is_print_conv_required(reference: &str) -> bool {\n");
    code.push_str("    REQUIRED_PRINT_CONV_SET.contains(reference)\n");
    code.push_str("}\n\n");

    code.push_str("/// Check if a ValueConv reference is required by generated tags\n");
    code.push_str("pub fn is_value_conv_required(reference: &str) -> bool {\n");
    code.push_str("    REQUIRED_VALUE_CONV_SET.contains(reference)\n");
    code.push_str("}\n");

    // Write file
    let output_path = Path::new(output_dir).join("conversion_refs.rs");
    fs::write(&output_path, code)
        .with_context(|| format!("Failed to write conversion_refs.rs to {:?}", output_path))?;

    println!("Generated: conversion_refs.rs");
    Ok(())
}

fn generate_supported_tags(_tags: &[GeneratedTag], output_dir: &str) -> Result<()> {
    // Configuration-driven supported tags generation (Milestone 8a)
    //
    // IMPORTANT: Update this configuration as milestones complete!
    // Only add tags that have fully working PrintConv implementations and pass compatibility tests.
    // This ensures the supported tags list represents a quality statement, not just existence.
    // Group-prefixed format for compatibility test filtering
    const MILESTONE_COMPLETIONS: &[(&str, &[&str])] = &[
        (
            "basic",
            &[
                "EXIF:Make",
                "EXIF:Model",
                "File:MIMEType",
                "SourceFile",
                "File:FileName",
                "File:Directory",
                "File:FileSize",
                "File:FileModifyDate",
            ],
        ),
        (
            "Milestone 4",
            &[
                "EXIF:Orientation",      // orientation_print_conv ✅
                "EXIF:ResolutionUnit",   // resolutionunit_print_conv ✅
                "EXIF:YCbCrPositioning", // ycbcrpositioning_print_conv ✅
            ],
        ),
        (
            "Milestone 7",
            &[
                "EXIF:Flash",           // flash_print_conv ✅
                "EXIF:ColorSpace",      // colorspace_print_conv ✅
                "EXIF:ExposureProgram", // exposureprogram_print_conv ✅
                "EXIF:WhiteBalance",    // whitebalance_print_conv ✅
                "EXIF:MeteringMode",    // meteringmode_print_conv ✅
            ],
        ),
        (
            "Milestone 8b",
            &[
                "EXIF:FNumber",             // fnumber_print_conv ✅
                "EXIF:ExposureTime",        // exposuretime_print_conv ✅
                "EXIF:FocalLength",         // focallength_print_conv ✅
                "EXIF:ISO",                 // Standard ISO value ✅
                "EXIF:SubSecTimeDigitized", // Sub-second precision for DateTimeDigitized ✅
            ],
        ),
        (
            "Milestone 8c",
            &[
                "EXIF:GPSLatitude",  // GPS ValueConv to decimal degrees ✅
                "EXIF:GPSLongitude", // GPS ValueConv to decimal degrees ✅
                "EXIF:GPSAltitude",
                "EXIF:GPSDestLatitude",  // GPS ValueConv to decimal degrees ✅
                "EXIF:GPSDestLongitude", // GPS ValueConv to decimal degrees ✅
            ],
        ),
        (
            "Lens Support",
            &[
                "EXIF:LensModel",        // Standard EXIF lens model name
                "EXIF:LensMake",         // Standard EXIF lens make name
                "EXIF:LensInfo",         // Standard EXIF lens information
                "EXIF:LensSerialNumber", // Standard EXIF lens serial number
            ],
        ),
        (
            "ExifIFD Core Tags",
            &[
                "EXIF:DateTimeOriginal", // Original capture date/time
                "EXIF:CreateDate",       // File creation date/time
                // "EXIF:ExifImageWidth",      // Image width in ExifIFD -- worthless tag
                // "EXIF:ExifImageHeight",     // Image height in ExifIFD -- worthless tag
                // "EXIF:ExposureMode",        // Exposure mode setting (TODO: needs PrintConv)
                // "EXIF:SceneCaptureType",    // Scene type (TODO: needs PrintConv)
                "EXIF:SubSecTimeOriginal", // Sub-second precision for DateTimeOriginal
                "EXIF:SubSecTime",         // Sub-second precision for DateTime
                "EXIF:SubSecTimeDigitized", // Sub-second precision for DateTimeDigitized
            ],
        ),
    ];

    // Flatten all completed milestone tags
    let supported_tag_names: Vec<&str> = MILESTONE_COMPLETIONS
        .iter()
        .flat_map(|(milestone, tags)| {
            println!("  Including {} tags from {}", tags.len(), milestone);
            tags.iter().copied()
        })
        .collect();

    // Generate JSON file for shell script consumption
    let json_content = serde_json::to_string_pretty(&supported_tag_names)?;
    let repo_root = find_repo_root(Path::new(output_dir))?;
    let json_path = repo_root.join("config/supported_tags.json");

    // Create config directory if it doesn't exist
    if let Some(parent) = json_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
    }

    fs::write(&json_path, json_content)
        .with_context(|| format!("Failed to write supported_tags.json to {:?}", json_path))?;

    // Generate Rust constant for use in tests
    let mut rust_code = String::new();
    rust_code.push_str("//! Generated supported tags list\n");
    rust_code.push_str("//!\n");
    rust_code.push_str("//! This file is automatically generated by codegen/generate_rust.\n");
    rust_code.push_str("//! DO NOT EDIT MANUALLY - changes will be overwritten.\n");
    rust_code.push_str("//!\n");
    rust_code.push_str(
        "//! Configuration-driven approach: Update MILESTONE_COMPLETIONS in codegen/src/main.rs\n",
    );
    rust_code.push_str(
        "//! as new milestones with working PrintConv implementations are completed.\n\n",
    );

    rust_code.push_str("/// Tags currently supported by exif-oxide with working implementations\n");
    rust_code
        .push_str("/// Generated from milestone-based configuration to ensure quality control\n");
    rust_code.push_str("pub static SUPPORTED_TAGS: &[&str] = &[\n");
    for tag_name in &supported_tag_names {
        rust_code.push_str(&format!("    \"{}\",\n", tag_name));
    }
    rust_code.push_str("];\n");

    let rust_path = Path::new(output_dir).join("supported_tags.rs");
    fs::write(&rust_path, rust_code)
        .with_context(|| format!("Failed to write supported_tags.rs to {:?}", rust_path))?;

    println!(
        "Generated: supported_tags.rs and config/supported_tags.json ({} tags total)",
        supported_tag_names.len()
    );
    Ok(())
}

fn generate_mod_file(output_dir: &str) -> Result<()> {
    let code = "//! Generated code module
//!
//! This module contains all code generated from ExifTool tables.

pub mod canon;
pub mod composite_tags;
pub mod conversion_refs;
pub mod file_types;
pub mod nikon;
pub mod supported_tags;
pub mod tags;

pub use composite_tags::*;
pub use conversion_refs::*;
pub use supported_tags::*;
pub use tags::*;
";

    let output_path = Path::new(output_dir).join("mod.rs");
    fs::write(&output_path, code)
        .with_context(|| format!("Failed to write mod.rs to {:?}", output_path))?;

    println!("Generated: mod.rs");
    Ok(())
}

fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\") // Must be first to avoid double-escaping
        .replace('\"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn parse_dependency_map(input: HashMap<String, String>) -> HashMap<u8, String> {
    input
        .into_iter()
        .filter_map(|(k, v)| k.parse::<u8>().ok().map(|key| (key, v)))
        .collect()
}

fn parse_group_map(input: HashMap<String, String>) -> HashMap<u8, String> {
    input
        .into_iter()
        .filter_map(|(k, v)| k.parse::<u8>().ok().map(|key| (key, v)))
        .collect()
}

fn generate_composite_tag_table(
    composite_tags: &[GeneratedCompositeTag],
    output_dir: &str,
) -> Result<()> {
    let mut code = String::new();

    // File header
    code.push_str("//! Generated composite tag definitions\n");
    code.push_str("//!\n");
    code.push_str("//! This file is automatically generated by codegen/generate_rust.\n");
    code.push_str("//! DO NOT EDIT MANUALLY - changes will be overwritten.\n\n");

    code.push_str("use std::collections::HashMap;\n");
    code.push_str("use std::sync::LazyLock;\n\n");

    // Composite tag structure
    code.push_str("#[derive(Debug, Clone)]\n");
    code.push_str("pub struct CompositeTagDef {\n");
    code.push_str("    pub name: &'static str,\n");
    code.push_str("    pub table: &'static str,\n");
    code.push_str("    pub description: Option<&'static str>,\n");
    code.push_str("    pub require: &'static [(u8, &'static str)],\n");
    code.push_str("    pub desire: &'static [(u8, &'static str)],\n");
    code.push_str("    pub value_conv: Option<&'static str>,\n");
    code.push_str("    pub print_conv_ref: Option<&'static str>,\n");
    code.push_str("    pub groups: &'static [(u8, &'static str)],\n");
    code.push_str("    pub writable: bool,\n");
    code.push_str("    pub avoid: bool,\n");
    code.push_str("    pub priority: i32,\n");
    code.push_str("    pub notes: Option<&'static str>,\n");
    code.push_str("}\n\n");

    // Static composite tag array
    code.push_str("pub static COMPOSITE_TAGS: &[CompositeTagDef] = &[\n");

    for comp_tag in composite_tags {
        code.push_str("    CompositeTagDef {\n");
        code.push_str(&format!("        name: \"{}\",\n", comp_tag.name));
        code.push_str(&format!("        table: \"{}\",\n", comp_tag.table));

        // Optional description
        if let Some(desc) = &comp_tag.description {
            code.push_str(&format!(
                "        description: Some(\"{}\"),\n",
                escape_string(desc)
            ));
        } else {
            code.push_str("        description: None,\n");
        }

        // Require dependencies
        code.push_str("        require: &[");
        let mut sorted_require: Vec<_> = comp_tag.require.iter().collect();
        sorted_require.sort_by_key(|(index, _)| *index);
        for (index, tag_name) in sorted_require {
            code.push_str(&format!("({}, \"{}\"), ", index, tag_name));
        }
        code.push_str("],\n");

        // Desire dependencies
        code.push_str("        desire: &[");
        let mut sorted_desire: Vec<_> = comp_tag.desire.iter().collect();
        sorted_desire.sort_by_key(|(index, _)| *index);
        for (index, tag_name) in sorted_desire {
            code.push_str(&format!("({}, \"{}\"), ", index, tag_name));
        }
        code.push_str("],\n");

        // Value conversion
        if let Some(value_conv) = &comp_tag.value_conv {
            code.push_str(&format!(
                "        value_conv: Some(\"{}\"),\n",
                escape_string(value_conv)
            ));
        } else {
            code.push_str("        value_conv: None,\n");
        }

        // PrintConv reference
        if let Some(print_ref) = &comp_tag.print_conv_ref {
            code.push_str(&format!(
                "        print_conv_ref: Some(\"{}\"),\n",
                print_ref
            ));
        } else {
            code.push_str("        print_conv_ref: None,\n");
        }

        // Groups
        code.push_str("        groups: &[");
        let mut sorted_groups: Vec<_> = comp_tag.groups.iter().collect();
        sorted_groups.sort_by_key(|(index, _)| *index);
        for (index, group_name) in sorted_groups {
            code.push_str(&format!("({}, \"{}\"), ", index, group_name));
        }
        code.push_str("],\n");

        code.push_str(&format!("        writable: {},\n", comp_tag.writable));
        code.push_str(&format!("        avoid: {},\n", comp_tag.avoid));
        code.push_str(&format!("        priority: {},\n", comp_tag.priority));

        // Optional notes
        if let Some(notes) = &comp_tag.notes {
            code.push_str(&format!(
                "        notes: Some(\"{}\"),\n",
                escape_string(notes)
            ));
        } else {
            code.push_str("        notes: None,\n");
        }

        code.push_str("    },\n");
    }

    code.push_str("];\n\n");

    // Lookup by name
    code.push_str("pub static COMPOSITE_TAG_BY_NAME: LazyLock<HashMap<&'static str, &'static CompositeTagDef>> = LazyLock::new(|| {\n");
    code.push_str("    let mut map = HashMap::new();\n");
    code.push_str("    for tag in COMPOSITE_TAGS {\n");
    code.push_str("        map.insert(tag.name, tag);\n");
    code.push_str("    }\n");
    code.push_str("    map\n");
    code.push_str("});\n");

    // Write file
    let output_path = Path::new(output_dir).join("composite_tags.rs");
    fs::write(&output_path, code)
        .with_context(|| format!("Failed to write composite_tags.rs to {:?}", output_path))?;

    println!("Generated: composite_tags.rs");
    Ok(())
}

/// Generate simple tables from extracted JSON data
fn generate_simple_tables(input_path: &str, output_dir: &str) -> Result<()> {
    let json_content = fs::read_to_string(input_path)
        .with_context(|| format!("Failed to read simple tables JSON: {}", input_path))?;

    let tables_data: SimpleTablesData = serde_json::from_str(&json_content)
        .with_context(|| "Failed to parse simple tables JSON")?;

    for (hash_name, table_data) in &tables_data.tables {
        let config = &table_data.config;

        // Create manufacturer directory
        let output_path = Path::new(output_dir).join(&config.output_file);
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }

        // Generate Rust code for this table
        let code = generate_table_code(hash_name, table_data)?;
        fs::write(&output_path, code)
            .with_context(|| format!("Failed to write: {:?}", output_path))?;

        println!(
            "Generated: {} ({} entries)",
            config.output_file, table_data.entry_count
        );
    }

    println!("Generated {} simple tables total", tables_data.total_tables);
    Ok(())
}

/// Generate Rust code for a single simple table
fn generate_table_code(hash_name: &str, table_data: &ExtractedTable) -> Result<String> {
    let _config = &table_data.config;
    
    // Check extraction type and delegate to appropriate generator
    let extraction_type = table_data.extraction_type.as_deref().unwrap_or("simple_table");
    match extraction_type {
        "regex_strings" => generate_regex_table_code(hash_name, table_data),
        "file_type_lookup" => generate_file_type_lookup_code(hash_name, table_data),
        _ => generate_simple_table_code(hash_name, table_data),
    }
}

/// Generate Rust code for a simple lookup table
fn generate_simple_table_code(hash_name: &str, table_data: &ExtractedTable) -> Result<String> {
    let config = &table_data.config;
    let mut code = String::new();

    // File header
    let constant_name = config.constant_name.as_ref()
        .ok_or_else(|| anyhow::anyhow!("constant_name required for simple tables"))?;
    code.push_str(&format!(
        "//! Generated {} lookup table\n//!\n//! This file is automatically generated.\n//! DO NOT EDIT MANUALLY - changes will be overwritten.\n//!\n//! Source: ExifTool {} {}\n//! Description: {}\n\n",
        constant_name.to_lowercase(),
        config.module,
        hash_name,
        config.description
    ));

    code.push_str("use std::collections::HashMap;\n");
    code.push_str("use std::sync::LazyLock;\n\n");

    // Determine HashMap type based on key_type
    let key_type = config.key_type.as_ref()
        .ok_or_else(|| anyhow::anyhow!("key_type required for simple tables"))?;
    let key_rust_type = match key_type.as_str() {
        "u8" => "u8",
        "u16" => "u16",
        "u32" => "u32",
        "i8" => "i8",
        "i16" => "i16",
        "i32" => "i32",
        "f32" => "f32",
        "String" => "&'static str",
        _ => "&'static str", // Default fallback
    };

    // Generate static HashMap
    code.push_str(&format!(
        "/// {} lookup table\n/// Source: ExifTool {} {} ({} entries)\npub static {}: LazyLock<HashMap<{}, &'static str>> = LazyLock::new(|| {{\n",
        config.description,
        config.module,
        hash_name,
        table_data.entry_count,
        constant_name,
        key_rust_type
    ));

    code.push_str("    let mut map = HashMap::new();\n");

    // Sort entries by key for deterministic output
    let mut sorted_entries = table_data.entries.clone();
    sorted_entries.sort_by(|a, b| {
        if key_type == "String" {
            a.key.as_ref().unwrap_or(&String::new()).cmp(b.key.as_ref().unwrap_or(&String::new()))
        } else {
            // Parse numeric keys for proper sorting
            let a_num: i64 = a.key.as_ref().unwrap_or(&String::new()).parse().unwrap_or(0);
            let b_num: i64 = b.key.as_ref().unwrap_or(&String::new()).parse().unwrap_or(0);
            a_num.cmp(&b_num)
        }
    });

    // Add entries
    for entry in &sorted_entries {
        if let (Some(key), Some(value)) = (&entry.key, &entry.value) {
            let key_value = if key_type == "String" {
                format!("\"{}\"", key)
            } else {
                key.clone()
            };

            code.push_str(&format!(
                "    map.insert({}, \"{}\");\n",
                key_value,
                escape_rust_string(value)
            ));
        }
    }

    code.push_str("    map\n});\n\n");

    // Generate lookup function
    let fn_name = constant_name.to_lowercase();
    let fn_param_type = if key_type == "String" {
        "&str"
    } else {
        key_rust_type
    };
    code.push_str(&format!(
        "/// Look up {} value by key\npub fn lookup_{}(key: {}) -> Option<&'static str> {{\n",
        config.description.to_lowercase(),
        fn_name,
        fn_param_type
    ));
    let key_ref = if key_type == "String" {
        "key"
    } else {
        "&key"
    };
    code.push_str(&format!(
        "    {}.get({}).copied()\n",
        constant_name, key_ref
    ));
    code.push_str("}\n");

    Ok(code)
}

/// Generate Rust code for regex patterns table (magic numbers)
fn generate_regex_table_code(hash_name: &str, table_data: &ExtractedTable) -> Result<String> {
    let config = &table_data.config;
    let mut code = String::new();

    // File header
    let constant_name = config.constant_name.as_ref()
        .ok_or_else(|| anyhow::anyhow!("constant_name required for regex tables"))?;
    code.push_str(&format!(
        "//! Generated {} regex patterns table\n//!\n//! This file is automatically generated.\n//! DO NOT EDIT MANUALLY - changes will be overwritten.\n//!\n//! Source: ExifTool {} {}\n//! Description: {}\n\n",
        constant_name.to_lowercase(),
        config.module,
        hash_name,
        config.description
    ));

    code.push_str("use std::collections::HashMap;\n");
    code.push_str("use std::sync::LazyLock;\n\n");

    // Determine HashMap type based on key_type
    let key_type = config.key_type.as_ref()
        .ok_or_else(|| anyhow::anyhow!("key_type required for regex tables"))?;
    let key_rust_type = match key_type.as_str() {
        "String" => "&'static str",
        _ => {
            return Err(anyhow::anyhow!(
                "regex_strings extraction only supports String key_type, got: {}",
                key_type
            ));
        }
    };

    // Generate static HashMap for regex patterns
    code.push_str(&format!(
        "/// {} regex patterns table\n/// Source: ExifTool {} {} ({} entries)\n/// Each value is a regex pattern for file type detection\npub static {}: LazyLock<HashMap<{}, &'static str>> = LazyLock::new(|| {{\n",
        config.description,
        config.module,
        hash_name,
        table_data.entry_count,
        constant_name,
        key_rust_type
    ));

    code.push_str("    let mut map = HashMap::new();\n");

    // Sort entries by key for deterministic output
    let mut sorted_entries = table_data.entries.clone();
    sorted_entries.sort_by(|a, b| {
        a.key.as_ref().unwrap_or(&String::new()).cmp(b.key.as_ref().unwrap_or(&String::new()))
    });

    // Add entries with traceability comments
    for entry in &sorted_entries {
        if let (Some(key), Some(value)) = (&entry.key, &entry.value) {
            // Add comment showing source and compatibility info
            let compat_info = if let Some(compat_note) = &entry.compatibility_notes {
                if !entry.rust_compatible.unwrap_or(true) {
                    format!(" // WARNING: {}", compat_note)
                } else {
                    "".to_string()  // Don't clutter output with compatibility OK messages
                }
            } else {
                "".to_string()
            };

            // Handle raw string vs regular string based on quotes
            let pattern_str = if value.contains('"') {
                format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
            } else {
                format!("r\"{}\"", value)
            };

            code.push_str(&format!(
                "    map.insert(\"{}\", {});{}\n",
                key,
                pattern_str,
                compat_info
            ));
        }
    }

    code.push_str("    map\n});\n\n");

    // Generate lookup function
    let fn_name = constant_name.to_lowercase();
    code.push_str(&format!(
        "/// Look up regex pattern for file type\n/// Returns the regex pattern string that can be compiled for file detection\npub fn lookup_{}(file_type: &str) -> Option<&'static str> {{\n",
        fn_name
    ));
    code.push_str(&format!(
        "    {}.get(file_type).copied()\n",
        constant_name
    ));
    code.push_str("}\n\n");

    // Generate all file types function for convenience
    code.push_str(&format!(
        "/// Get all supported file types\npub fn {}_file_types() -> Vec<&'static str> {{\n",
        fn_name
    ));
    code.push_str(&format!(
        "    {}.keys().copied().collect()\n",
        constant_name
    ));
    code.push_str("}\n");

    Ok(code)
}

/// Generate Rust code for file type lookup table (discriminated union pattern)
fn generate_file_type_lookup_code(hash_name: &str, table_data: &ExtractedTable) -> Result<String> {
    let config = &table_data.config;
    let mut code = String::new();

    // File header
    let constant_name = config.constant_name.as_ref()
        .ok_or_else(|| anyhow::anyhow!("constant_name required for file_type_lookup"))?;
    code.push_str(&format!(
        "//! Generated file type lookup infrastructure\n//!\n//! This file is automatically generated.\n//! DO NOT EDIT MANUALLY - changes will be overwritten.\n//!\n//! Source: ExifTool {} {}\n//! Description: {}\n\n",
        config.module,
        hash_name,
        config.description
    ));

    code.push_str("use std::collections::HashMap;\n");
    code.push_str("use std::sync::LazyLock;\n\n");

    // Generate the FileTypeEntry enum
    code.push_str("#[derive(Debug, Clone, PartialEq)]\n");
    code.push_str("pub enum FileTypeEntry {\n");
    code.push_str("    /// Extension alias requiring follow-up lookup\n");
    code.push_str("    Alias(String),\n");
    code.push_str("    /// File type definition with formats and description\n");
    code.push_str("    Definition {\n");
    code.push_str("        formats: Vec<String>,\n");
    code.push_str("        description: String,\n");
    code.push_str("    },\n");
    code.push_str("}\n\n");

    // Generate static HashMap
    code.push_str(&format!(
        "/// Core file type lookup table\n/// Source: ExifTool {} {} ({} entries)\npub static {}: LazyLock<HashMap<&'static str, FileTypeEntry>> = LazyLock::new(|| {{\n",
        config.module,
        hash_name,
        table_data.entry_count,
        constant_name
    ));

    code.push_str("    let mut map = HashMap::new();\n\n");

    // Separate entries by type for cleaner output
    let mut aliases = Vec::new();
    let mut definitions = Vec::new();

    for entry in &table_data.entries {
        if let Some(entry_type) = &entry.entry_type {
            match entry_type.as_str() {
                "alias" => aliases.push(entry),
                "definition" => definitions.push(entry),
                _ => {
                    eprintln!("Warning: Unknown entry_type: {}", entry_type);
                }
            }
        }
    }

    // Sort both categories alphabetically
    aliases.sort_by(|a, b| {
        a.extension.as_ref().unwrap_or(&String::new())
            .cmp(b.extension.as_ref().unwrap_or(&String::new()))
    });
    definitions.sort_by(|a, b| {
        a.extension.as_ref().unwrap_or(&String::new())
            .cmp(b.extension.as_ref().unwrap_or(&String::new()))
    });

    // Add aliases first
    if !aliases.is_empty() {
        code.push_str("    // Aliases\n");
        for entry in aliases {
            if let (Some(ext), Some(target)) = (&entry.extension, &entry.target) {
                code.push_str(&format!(
                    "    map.insert(\"{}\", FileTypeEntry::Alias(\"{}\".to_string()));\n",
                    ext, target
                ));
            }
        }
        code.push_str("\n");
    }

    // Add definitions
    if !definitions.is_empty() {
        code.push_str("    // Definitions\n");
        for entry in definitions {
            if let (Some(ext), Some(formats), Some(desc)) = 
                (&entry.extension, &entry.formats, &entry.description) {
                
                // Format the formats vector
                let formats_str = formats.iter()
                    .map(|f| format!("\"{}\".to_string()", f))
                    .collect::<Vec<_>>()
                    .join(", ");

                code.push_str(&format!(
                    "    map.insert(\"{}\", FileTypeEntry::Definition {{\n        formats: vec![{}],\n        description: \"{}\".to_string(),\n    }});\n",
                    ext,
                    formats_str,
                    escape_rust_string(desc)
                ));
            }
        }
    }

    code.push_str("\n    map\n});\n\n");

    // Generate helper functions
    code.push_str("/// Resolve file type with alias following\n");
    code.push_str("/// Returns (formats, description) tuple if found\n");
    code.push_str("pub fn resolve_file_type(extension: &str) -> Option<(Vec<String>, String)> {\n");
    code.push_str("    const MAX_ALIAS_DEPTH: u8 = 10; // Prevent infinite loops\n\n");
    code.push_str("    let mut current_ext = extension;\n");
    code.push_str("    let mut depth = 0;\n\n");
    code.push_str("    while depth < MAX_ALIAS_DEPTH {\n");
    code.push_str(&format!("        match {}.get(current_ext.to_uppercase().as_str()) {{\n", constant_name));
    code.push_str("            Some(FileTypeEntry::Alias(target)) => {\n");
    code.push_str("                current_ext = target;\n");
    code.push_str("                depth += 1;\n");
    code.push_str("            },\n");
    code.push_str("            Some(FileTypeEntry::Definition { formats, description }) => {\n");
    code.push_str("                return Some((formats.clone(), description.clone()));\n");
    code.push_str("            },\n");
    code.push_str("            None => return None,\n");
    code.push_str("        }\n");
    code.push_str("    }\n\n");
    code.push_str("    None // Alias chain too deep or circular\n");
    code.push_str("}\n\n");

    // Get primary format function
    code.push_str("/// Get primary format for extension (first in formats list)\n");
    code.push_str("pub fn get_primary_format(extension: &str) -> Option<String> {\n");
    code.push_str("    resolve_file_type(extension)\n");
    code.push_str("        .and_then(|(formats, _)| formats.into_iter().next())\n");
    code.push_str("}\n\n");

    // Check format support function
    code.push_str("/// Check if extension supports a specific format\n");
    code.push_str("pub fn supports_format(extension: &str, format: &str) -> bool {\n");
    code.push_str("    resolve_file_type(extension)\n");
    code.push_str("        .map(|(formats, _)| formats.iter().any(|f| f == format))\n");
    code.push_str("        .unwrap_or(false)\n");
    code.push_str("}\n\n");

    // Get all extensions for format function
    code.push_str("/// Get all supported extensions for a format\n");
    code.push_str("pub fn extensions_for_format(format: &str) -> Vec<String> {\n");
    code.push_str(&format!("    {}\n", constant_name));
    code.push_str("        .iter()\n");
    code.push_str("        .filter_map(|(ext, entry)| {\n");
    code.push_str("            if let FileTypeEntry::Definition { formats, .. } = entry {\n");
    code.push_str("                if formats.contains(&format.to_string()) {\n");
    code.push_str("                    Some(ext.to_string())\n");
    code.push_str("                } else {\n");
    code.push_str("                    None\n");
    code.push_str("                }\n");
    code.push_str("            } else {\n");
    code.push_str("                None\n");
    code.push_str("            }\n");
    code.push_str("        })\n");
    code.push_str("        .collect()\n");
    code.push_str("}\n");

    Ok(code)
}

/// Escape regex pattern for Rust raw string literals
/// Uses raw strings to minimize escaping issues
fn escape_regex_for_rust(pattern: &str) -> String {
    // For raw strings, we just need to handle quotes carefully
    // Most regex patterns work fine in raw strings r"..."
    // But if pattern contains quotes, we need to use regular strings
    if pattern.contains('"') {
        // Use regular string with minimal escaping
        pattern.replace('\"', "\\\"")
    } else {
        // Use the pattern as-is in raw string
        pattern.to_string()
    }
}


/// Escape string for Rust string literals
fn escape_rust_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
