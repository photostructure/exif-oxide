//! Rust code generation tool for exif-oxide
//!
//! This tool reads JSON output from extract_tables.pl and generates
//! Rust code following the "runtime references, no stubs" architecture.

use anyhow::{Context, Result};
use clap::{Arg, Command};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// JSON structure from extract_tables.pl
#[derive(Debug, Deserialize)]
struct ExtractedData {
    extracted_at: String,
    exiftool_version: String,
    filter_criteria: String,
    total_tags: usize,
    tags: Vec<ExtractedTag>,
}

/// Individual tag extracted from ExifTool
#[derive(Debug, Deserialize)]
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
                .default_value("tag_tables.json")
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

    println!("Loaded {} tags from ExifTool {}", 
             extracted_data.total_tags, extracted_data.exiftool_version);

    // Convert extracted tags to generated format
    let mut tags = Vec::new();
    let mut conversion_refs = HashMap::new();

    for tag in extracted_data.tags {
        // Parse hex ID
        let id = parse_hex_id(&tag.id)?;

        let generated_tag = GeneratedTag {
            id,
            name: tag.name.clone(),
            format: normalize_format(&tag.format),
            groups: tag.groups,
            writable: tag.writable != 0,
            description: tag.description,
            print_conv_ref: tag.print_conv_ref.clone(),
            value_conv_ref: tag.value_conv_ref.clone(),
            notes: tag.notes,
        };

        // Collect conversion references for registry
        if let Some(ref conv_ref) = tag.print_conv_ref {
            conversion_refs.insert(conv_ref.clone(), "PrintConv");
        }
        if let Some(ref conv_ref) = tag.value_conv_ref {
            conversion_refs.insert(conv_ref.clone(), "ValueConv");
        }

        tags.push(generated_tag);
    }

    // Create output directory
    fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create output directory: {}", output_dir))?;

    // Generate tag table
    generate_tag_table(&tags, output_dir)?;

    // Generate conversion registry stubs
    generate_conversion_registry(&conversion_refs, output_dir)?;

    // Generate module file
    generate_mod_file(output_dir)?;

    println!("Generated {} tags with {} conversion references", 
             tags.len(), conversion_refs.len());
    println!("Code generated in: {}", output_dir);
    println!("\nNext steps:");
    println!("1. Add 'mod generated;' to src/lib.rs");
    println!("2. Use --show-missing on real images to see what implementations are needed");
    println!("3. Implement missing PrintConv/ValueConv functions in implementations/");

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
    
    code.push_str("use crate::types::*;\n");
    code.push_str("use std::collections::HashMap;\n");
    code.push_str("use lazy_static::lazy_static;\n\n");

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

fn generate_conversion_registry(conversion_refs: &HashMap<String, &str>, output_dir: &str) -> Result<()> {
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
    code.push_str("use lazy_static::lazy_static;\n\n");

    // List all required conversion references
    code.push_str("/// All PrintConv references that need implementation\n");
    code.push_str("pub static REQUIRED_PRINT_CONV: &[&str] = &[\n");
    for (conv_ref, conv_type) in conversion_refs {
        if *conv_type == "PrintConv" {
            code.push_str(&format!("    \"{}\",\n", conv_ref));
        }
    }
    code.push_str("];\n\n");

    code.push_str("/// All ValueConv references that need implementation\n");
    code.push_str("pub static REQUIRED_VALUE_CONV: &[&str] = &[\n");
    for (conv_ref, conv_type) in conversion_refs {
        if *conv_type == "ValueConv" {
            code.push_str(&format!("    \"{}\",\n", conv_ref));
        }
    }
    code.push_str("];\n\n");

    // Runtime check functions
    code.push_str("lazy_static! {\n");
    code.push_str("    static ref REQUIRED_PRINT_CONV_SET: HashSet<&'static str> = {\n");
    code.push_str("        REQUIRED_PRINT_CONV.iter().copied().collect()\n");
    code.push_str("    };\n");
    code.push_str("    static ref REQUIRED_VALUE_CONV_SET: HashSet<&'static str> = {\n");
    code.push_str("        REQUIRED_VALUE_CONV.iter().copied().collect()\n");
    code.push_str("    };\n");
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

fn generate_mod_file(output_dir: &str) -> Result<()> {
    let code = "//! Generated code module
//!
//! This module contains all code generated from ExifTool tables.

pub mod tags;
pub mod conversion_refs;

pub use tags::*;
pub use conversion_refs::*;
";

    let output_path = Path::new(output_dir).join("mod.rs");
    fs::write(&output_path, code)
        .with_context(|| format!("Failed to write mod.rs to {:?}", output_path))?;

    println!("Generated: mod.rs");
    Ok(())
}

fn escape_string(s: &str) -> String {
    s.replace('\"', "\\\"")
     .replace('\n', "\\n")
     .replace('\r', "\\r")
     .replace('\t', "\\t")
}