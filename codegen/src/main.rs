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
        current_path = current_path.parent()
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

    // Read and parse input JSON
    let json_content = fs::read_to_string(input_file)
        .with_context(|| format!("Failed to read input file: {}", input_file))?;

    let extracted_data: ExtractedData = serde_json::from_str(&json_content)
        .with_context(|| "Failed to parse JSON input")?;

    println!("Loaded {} tags and {} composite tags from ExifTool {}", 
             extracted_data.total_tags, extracted_data.composite_tags.len(), extracted_data.exiftool_version);

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
    println!("  PrintConv: {} functions", extracted_data.conversion_refs.print_conv.len());
    println!("  ValueConv: {} functions", extracted_data.conversion_refs.value_conv.len());

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

    // Generate module file
    generate_mod_file(output_dir)?;

    let total_conv_refs = extracted_data.conversion_refs.print_conv.len() + 
                         extracted_data.conversion_refs.value_conv.len();
    println!("Generated {} tags and {} composite tags with {} conversion references", 
             tags.len(), composite_tags.len(), total_conv_refs);
    println!("Code generated in: {}", output_dir);
    println!("\nNext steps:");
    println!("1. Add 'mod generated;' to src/lib.rs");
    println!("2. Use --show-missing on real images to see what implementations are needed");
    println!("3. Implement missing PrintConv/ValueConv and composite functions in implementations/");

    Ok(())
}

fn parse_hex_id(hex_str: &str) -> Result<u32> {
    let cleaned = hex_str.trim_start_matches("0x");
    u32::from_str_radix(cleaned, 16)
        .with_context(|| format!("Failed to parse hex ID: {}", hex_str))
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
    }.to_string()
}

fn generate_tag_table(tags: &[GeneratedTag], output_dir: &str) -> Result<()> {
    let mut code = String::new();
    
    // File header
    code.push_str("//! Generated EXIF tag definitions\n");
    code.push_str("//!\n");
    code.push_str("//! This file is automatically generated by codegen/generate_rust.\n");
    code.push_str("//! DO NOT EDIT MANUALLY - changes will be overwritten.\n\n");
    
    code.push_str("use lazy_static::lazy_static;\n");
    code.push_str("use std::collections::HashMap;\n\n");

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
            if i > 0 { code.push_str(", "); }
            code.push_str(&format!("\"{}\"", group));
        }
        code.push_str("],\n");
        
        code.push_str(&format!("        writable: {},\n", tag.writable));
        
        // Optional fields
        if let Some(desc) = &tag.description {
            code.push_str(&format!("        description: Some(\"{}\"),\n", escape_string(desc)));
        } else {
            code.push_str("        description: None,\n");
        }
        
        if let Some(print_ref) = &tag.print_conv_ref {
            code.push_str(&format!("        print_conv_ref: Some(\"{}\"),\n", print_ref));
        } else {
            code.push_str("        print_conv_ref: None,\n");
        }
        
        if let Some(value_ref) = &tag.value_conv_ref {
            code.push_str(&format!("        value_conv_ref: Some(\"{}\"),\n", value_ref));
        } else {
            code.push_str("        value_conv_ref: None,\n");
        }
        
        if let Some(notes) = &tag.notes {
            code.push_str(&format!("        notes: Some(\"{}\"),\n", escape_string(notes)));
        } else {
            code.push_str("        notes: None,\n");
        }
        
        code.push_str("    },\n");
    }
    
    code.push_str("];\n\n");

    // Lookup by ID
    code.push_str("lazy_static! {\n");
    code.push_str("    pub static ref TAG_BY_ID: HashMap<u32, &'static TagDef> = {\n");
    code.push_str("        let mut map = HashMap::new();\n");
    code.push_str("        for tag in EXIF_MAIN_TAGS {\n");
    code.push_str("            map.insert(tag.id, tag);\n");
    code.push_str("        }\n");
    code.push_str("        map\n");
    code.push_str("    };\n");
    code.push_str("}\n\n");

    // Lookup by name
    code.push_str("lazy_static! {\n");
    code.push_str("    pub static ref TAG_BY_NAME: HashMap<&'static str, &'static TagDef> = {\n");
    code.push_str("        let mut map = HashMap::new();\n");
    code.push_str("        for tag in EXIF_MAIN_TAGS {\n");
    code.push_str("            map.insert(tag.name, tag);\n");
    code.push_str("        }\n");
    code.push_str("        map\n");
    code.push_str("    };\n");
    code.push_str("}\n");

    // Write file
    let output_path = Path::new(output_dir).join("tags.rs");
    fs::write(&output_path, code)
        .with_context(|| format!("Failed to write tags.rs to {:?}", output_path))?;

    println!("Generated: tags.rs");
    Ok(())
}

fn generate_conversion_refs(conversion_refs: &ConversionRefs, tags: &[GeneratedTag], output_dir: &str) -> Result<()> {
    let mut code = String::new();
    
    // File header
    code.push_str("//! Generated conversion registry stubs\n");
    code.push_str("//!\n");
    code.push_str("//! This file is automatically generated by codegen/generate_rust.\n");
    code.push_str("//! DO NOT EDIT MANUALLY - changes will be overwritten.\n");
    code.push_str("//!\n");
    code.push_str("//! These are just the registry keys - actual implementations\n");
    code.push_str("//! must be provided in the implementations/ directory.\n\n");
    
    code.push_str("use lazy_static::lazy_static;\n");
    code.push_str("use std::collections::HashSet;\n\n");

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
    code.push_str("lazy_static! {\n");
    code.push_str("    static ref REQUIRED_PRINT_CONV_SET: HashSet<&'static str> =\n");
    code.push_str("        REQUIRED_PRINT_CONV.iter().copied().collect();\n");
    code.push_str("    static ref REQUIRED_VALUE_CONV_SET: HashSet<&'static str> =\n");
    code.push_str("        REQUIRED_VALUE_CONV.iter().copied().collect();\n");
    code.push_str("}\n\n");

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
    const MILESTONE_COMPLETIONS: &[(&str, &[&str])] = &[
        ("basic", &[
            "Make", "Model", "MIMEType", "SourceFile", "FileName", "Directory", 
            "FileSize", "FileModifyDate", "ExifToolVersion"
        ]),
        ("Milestone 4", &[
            "Orientation",       // orientation_print_conv ✅
            "ResolutionUnit",    // resolutionunit_print_conv ✅  
            "YCbCrPositioning",  // ycbcrpositioning_print_conv ✅
        ]),
        ("Milestone 7", &[
            "Flash",             // flash_print_conv ✅
            "ColorSpace",        // colorspace_print_conv ✅
            "ExposureProgram",   // exposureprogram_print_conv ✅
            "WhiteBalance",      // whitebalance_print_conv ✅
            "MeteringMode",      // meteringmode_print_conv ✅
        ]),
        ("Milestone 10", &[
            "FocusMode",         // Canon CameraSettings PrintConv ✅
            "CanonFlashMode",    // Canon CameraSettings PrintConv ✅
            "ContinuousDrive",   // Canon CameraSettings PrintConv ✅
        ]),
        ("Milestone 8b", &[
            "FNumber",           // fnumber_print_conv ✅
            "ExposureTime",      // exposuretime_print_conv ✅
            "FocalLength",       // focallength_print_conv ✅
        ]),
        ("Milestone 8f", &[
            "ImageSize",         // Composite tag
            "ShutterSpeed",      // Composite tag
        ]),
        ("Milestone 8c", &[
            "GPSLatitude",       // GPS ValueConv to decimal degrees ✅
            "GPSLongitude",      // GPS ValueConv to decimal degrees ✅
            "GPSDestLatitude",   // GPS ValueConv to decimal degrees ✅
            "GPSDestLongitude",  // GPS ValueConv to decimal degrees ✅
        ]),
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
    rust_code.push_str("//! Configuration-driven approach: Update MILESTONE_COMPLETIONS in codegen/src/main.rs\n");
    rust_code.push_str("//! as new milestones with working PrintConv implementations are completed.\n\n");
    
    rust_code.push_str("/// Tags currently supported by exif-oxide with working implementations\n");
    rust_code.push_str("/// Generated from milestone-based configuration to ensure quality control\n");
    rust_code.push_str("pub static SUPPORTED_TAGS: &[&str] = &[\n");
    for tag_name in &supported_tag_names {
        rust_code.push_str(&format!("    \"{}\",\n", tag_name));
    }
    rust_code.push_str("];\n");

    let rust_path = Path::new(output_dir).join("supported_tags.rs");
    fs::write(&rust_path, rust_code)
        .with_context(|| format!("Failed to write supported_tags.rs to {:?}", rust_path))?;

    println!("Generated: supported_tags.rs and config/supported_tags.json ({} tags total)", supported_tag_names.len());
    Ok(())
}

fn generate_mod_file(output_dir: &str) -> Result<()> {
    let code = "//! Generated code module
//!
//! This module contains all code generated from ExifTool tables.

pub mod tags;
pub mod composite_tags;
pub mod conversion_refs;
pub mod supported_tags;

pub use tags::*;
pub use composite_tags::*;
pub use conversion_refs::*;
pub use supported_tags::*;
";

    let output_path = Path::new(output_dir).join("mod.rs");
    fs::write(&output_path, code)
        .with_context(|| format!("Failed to write mod.rs to {:?}", output_path))?;

    println!("Generated: mod.rs");
    Ok(())
}

fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")   // Must be first to avoid double-escaping
     .replace('\"', "\\\"")
     .replace('\n', "\\n")
     .replace('\r', "\\r")
     .replace('\t', "\\t")
}

fn parse_dependency_map(input: HashMap<String, String>) -> HashMap<u8, String> {
    input.into_iter()
        .filter_map(|(k, v)| k.parse::<u8>().ok().map(|key| (key, v)))
        .collect()
}

fn parse_group_map(input: HashMap<String, String>) -> HashMap<u8, String> {
    input.into_iter()
        .filter_map(|(k, v)| k.parse::<u8>().ok().map(|key| (key, v)))
        .collect()
}

fn generate_composite_tag_table(composite_tags: &[GeneratedCompositeTag], output_dir: &str) -> Result<()> {
    let mut code = String::new();
    
    // File header
    code.push_str("//! Generated composite tag definitions\n");
    code.push_str("//!\n");
    code.push_str("//! This file is automatically generated by codegen/generate_rust.\n");
    code.push_str("//! DO NOT EDIT MANUALLY - changes will be overwritten.\n\n");
    
    code.push_str("use lazy_static::lazy_static;\n");
    code.push_str("use std::collections::HashMap;\n\n");

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
            code.push_str(&format!("        description: Some(\"{}\"),\n", escape_string(desc)));
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
            code.push_str(&format!("        value_conv: Some(\"{}\"),\n", escape_string(value_conv)));
        } else {
            code.push_str("        value_conv: None,\n");
        }
        
        // PrintConv reference
        if let Some(print_ref) = &comp_tag.print_conv_ref {
            code.push_str(&format!("        print_conv_ref: Some(\"{}\"),\n", print_ref));
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
            code.push_str(&format!("        notes: Some(\"{}\"),\n", escape_string(notes)));
        } else {
            code.push_str("        notes: None,\n");
        }
        
        code.push_str("    },\n");
    }
    
    code.push_str("];\n\n");

    // Lookup by name
    code.push_str("lazy_static! {\n");
    code.push_str("    pub static ref COMPOSITE_TAG_BY_NAME: HashMap<&'static str, &'static CompositeTagDef> = {\n");
    code.push_str("        let mut map = HashMap::new();\n");
    code.push_str("        for tag in COMPOSITE_TAGS {\n");
    code.push_str("            map.insert(tag.name, tag);\n");
    code.push_str("        }\n");
    code.push_str("        map\n");
    code.push_str("    };\n");
    code.push_str("}\n");

    // Write file
    let output_path = Path::new(output_dir).join("composite_tags.rs");
    fs::write(&output_path, code)
        .with_context(|| format!("Failed to write composite_tags.rs to {:?}", output_path))?;

    println!("Generated: composite_tags.rs");
    Ok(())
}