use regex::Regex;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=exiftool/lib/Image/ExifTool/Exif.pm");
    println!("cargo:rerun-if-changed=exiftool/lib/Image/ExifTool/Canon.pm");
    println!("cargo:rerun-if-changed=exiftool-sync.toml");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_tags.rs");

    // Try to read ExifTool version from sync config
    let exiftool_version = read_exiftool_version().unwrap_or_else(|| "unknown".to_string());

    // Parse EXIF tags
    let exif_pm_path = "exiftool/lib/Image/ExifTool/Exif.pm";
    let exif_content = fs::read_to_string(exif_pm_path).expect("Failed to read Exif.pm");
    let exif_tags = parse_exif_tags(&exif_content);

    // Parse Canon tags
    let canon_pm_path = "exiftool/lib/Image/ExifTool/Canon.pm";
    let canon_content = fs::read_to_string(canon_pm_path).expect("Failed to read Canon.pm");
    let canon_tags = parse_canon_tags(&canon_content);

    let rust_code = generate_rust_code(&exif_tags, &canon_tags, &exiftool_version);

    let mut file = fs::File::create(&dest_path).expect("Failed to create generated file");
    file.write_all(rust_code.as_bytes())
        .expect("Failed to write generated file");
}

#[derive(Debug)]
struct TagDef {
    tag_id: u16,
    name: String,
    writable: Option<String>,
    groups: Option<String>,
    #[allow(dead_code)]
    notes: Option<String>,
}

fn parse_exif_tags(content: &str) -> Vec<TagDef> {
    let mut tags = Vec::new();

    // Find the Main table start
    let main_start = content
        .find("%Image::ExifTool::Exif::Main = (")
        .expect("Could not find Main table");

    // Improved regex to handle multi-line tag definitions
    // This handles cases where the tag definition spans multiple lines
    let tag_re = Regex::new(r"(?s)(0x[0-9a-fA-F]+)\s*=>\s*\{([^}]+)\}").unwrap();
    let name_re = Regex::new(r"Name\s*=>\s*'([^']+)'").unwrap();
    let writable_re = Regex::new(r"Writable\s*=>\s*'([^']+)'").unwrap();
    let groups_re = Regex::new(r"Groups\s*=>\s*\{[^}]*2\s*=>\s*'([^']+)'").unwrap();
    let notes_re = Regex::new(r"Notes\s*=>\s*'([^']+)'").unwrap();

    // Also match simple string tags like: 0x10f => 'Make',
    let simple_tag_re = Regex::new(r"(0x[0-9a-fA-F]+)\s*=>\s*'([^']+)',").unwrap();

    // Process a larger portion of the file to get more tags
    let search_content = &content[main_start..]
        .chars()
        .take(500000)
        .collect::<String>();

    // First, collect complex tag definitions
    for cap in tag_re.captures_iter(search_content) {
        let tag_hex = &cap[1];
        let tag_content = &cap[2];

        // Skip complex conditional tags, but handle simple ones
        if tag_content.contains("Condition =>") && tag_content.contains("$$") {
            continue;
        }

        // Parse the tag ID
        let tag_id = match u16::from_str_radix(&tag_hex[2..], 16) {
            Ok(id) => id,
            Err(_) => continue,
        };

        // Extract the name (required)
        let name = if let Some(name_cap) = name_re.captures(tag_content) {
            name_cap[1].to_string()
        } else {
            continue; // Skip tags without names
        };

        // Extract optional fields
        let writable = writable_re
            .captures(tag_content)
            .map(|cap| cap[1].to_string());
        let groups = groups_re
            .captures(tag_content)
            .map(|cap| cap[1].to_string());
        let notes = notes_re.captures(tag_content).map(|cap| cap[1].to_string());

        // Include standard EXIF tags and common maker note tags
        tags.push(TagDef {
            tag_id,
            name: name.clone(),
            writable,
            groups,
            notes,
        });

        // Debug output to track progress
        if tags.len() % 10 == 0 {
            eprintln!(
                "Parsed {} tags, latest: {} (0x{:04x})",
                tags.len(),
                name,
                tag_id
            );
        }
    }

    // Also collect simple string tags
    for cap in simple_tag_re.captures_iter(search_content) {
        let tag_hex = &cap[1];
        let name = &cap[2];

        let tag_id = match u16::from_str_radix(&tag_hex[2..], 16) {
            Ok(id) => id,
            Err(_) => continue,
        };

        // Check if we already have this tag
        if tags.iter().any(|t| t.tag_id == tag_id) {
            continue;
        }

        tags.push(TagDef {
            tag_id,
            name: name.to_string(),
            writable: Some("string".to_string()), // Default to string type
            groups: None,
            notes: None,
        });
    }

    // Sort tags by ID for consistent output
    tags.sort_by_key(|t| t.tag_id);

    // Remove duplicates (keep first occurrence)
    tags.dedup_by_key(|t| t.tag_id);

    eprintln!("Total tags parsed: {}", tags.len());

    // For spike 1.5, we want at least 50 common tags
    if tags.len() < 50 {
        eprintln!(
            "Warning: Only found {} tags, expected at least 50",
            tags.len()
        );
    }

    tags
}

fn parse_canon_tags(content: &str) -> Vec<TagDef> {
    let mut tags = Vec::new();

    // Find the Canon Main table start
    let main_start = content
        .find("%Image::ExifTool::Canon::Main = (")
        .expect("Could not find Canon Main table");

    // Use similar regex patterns but adapted for Canon
    let tag_re = Regex::new(r"(?s)(0x[0-9a-fA-F]+)\s*=>\s*\{([^}]+)\}").unwrap();
    let name_re = Regex::new(r"Name\s*=>\s*'([^']+)'").unwrap();
    let writable_re = Regex::new(r"Writable\s*=>\s*'([^']+)'").unwrap();
    let groups_re = Regex::new(r"Groups\s*=>\s*\{[^}]*2\s*=>\s*'([^']+)'").unwrap();

    // Process a portion of the Canon file - Canon has fewer main tags than EXIF
    let search_content = &content[main_start..]
        .chars()
        .take(800000)
        .collect::<String>();

    // Parse Canon tag definitions
    for cap in tag_re.captures_iter(search_content) {
        let tag_hex = &cap[1];
        let tag_content = &cap[2];

        // Include PreviewImageInfo tags even though they're SubDirectory
        let is_preview_info = tag_content.contains("PreviewImageInfo");

        // Include important Canon SubDirectory tags
        let is_important_canon_tag = matches!(
            tag_hex,
            "0x1"
                | "0x2"
                | "0x4"
                | "0x5"
                | "0x6"
                | "0x7"
                | "0x9"
                | "0xa"
                | "0xc"
                | "0xd"
                | "0xe"
                | "0xf"
                | "0x10"
                | "0x12"
                | "0x13"
                | "0x15"
                | "0x18"
                | "0x19"
                | "0x1a"
                | "0x1c"
                | "0x1d"
                | "0x1e"
                | "0x81"
                | "0x83"
                | "0x90"
                | "0x93"
                | "0x94"
                | "0x95"
                | "0x96"
                | "0x97"
                | "0x98"
                | "0x99"
                | "0x9a"
                | "0xa0"
                | "0xaa"
                | "0xe0"
                | "0x4001"
                | "0x4002"
                | "0x4003"
                | "0x4005"
                | "0x4008"
                | "0x4009"
                | "0x4010"
                | "0x4011"
                | "0x4013"
                | "0x4015"
                | "0x4016"
                | "0x4018"
                | "0x4019"
                | "0x4020"
                | "0x4021"
                | "0x4024"
                | "0x4025"
                | "0x4028"
        );

        // Skip extremely complex tags like SubDirectory for now, except preview info and important tags
        if !is_preview_info
            && !is_important_canon_tag
            && (tag_content.contains("SubDirectory") || tag_content.contains("TagTable"))
        {
            continue;
        }

        // Skip conditional arrays for now - these are very complex
        if tag_content.contains("Condition =>") && tag_content.contains("$$") {
            continue;
        }

        // Parse the tag ID
        let tag_id = match u16::from_str_radix(&tag_hex[2..], 16) {
            Ok(id) => id,
            Err(_) => continue,
        };

        // Extract the name (required)
        let name = if let Some(name_cap) = name_re.captures(tag_content) {
            name_cap[1].to_string()
        } else {
            continue; // Skip tags without names
        };

        // Extract optional fields
        let writable = writable_re
            .captures(tag_content)
            .map(|cap| cap[1].to_string());
        let groups = groups_re
            .captures(tag_content)
            .map(|cap| cap[1].to_string());

        tags.push(TagDef {
            tag_id,
            name: name.clone(),
            writable,
            groups,
            notes: None,
        });

        eprintln!("Canon tag: {} (0x{:04x})", name, tag_id);
    }

    // Sort tags by ID for consistent output
    tags.sort_by_key(|t| t.tag_id);

    // Remove duplicates (keep first occurrence)
    tags.dedup_by_key(|t| t.tag_id);

    eprintln!("Total Canon tags parsed: {}", tags.len());

    // Manually add the critical Canon PreviewImageInfo tags that are in a special format
    // These are the key tags for preview image extraction
    let preview_tags = vec![
        (0xB601, "PreviewQuality"),
        (0xB602, "PreviewImageLength"),
        (0xB603, "PreviewImageWidth"),
        (0xB604, "PreviewImageHeight"),
        (0xB605, "PreviewImageStart"),
    ];

    for (tag_id, name) in preview_tags {
        tags.push(TagDef {
            tag_id,
            name: name.to_string(),
            writable: Some("int32u".to_string()),
            groups: Some("Canon".to_string()),
            notes: None,
        });
        eprintln!("Canon preview tag (manual): {} (0x{:04x})", name, tag_id);
    }

    // Parse PreviewImageInfo sub-table for additional preview tags
    if let Some(preview_start) = content.find("%Image::ExifTool::Canon::PreviewImageInfo = (") {
        let preview_content = &content[preview_start..]
            .chars()
            .take(5000)
            .collect::<String>();

        // Parse numeric tags from PreviewImageInfo - these use different patterns
        let preview_simple_re = Regex::new(r"(\d+)\s*=>\s*'([^']+)'").unwrap();
        let preview_complex_re =
            Regex::new(r"(?s)(\d+)\s*=>\s*\{[^}]*Name\s*=>\s*'([^']+)'").unwrap();

        // Parse simple string assignments (like "3 => 'PreviewImageWidth'")
        for cap in preview_simple_re.captures_iter(preview_content) {
            if let Ok(tag_id) = cap[1].parse::<u16>() {
                let name = &cap[2];
                tags.push(TagDef {
                    tag_id: 0xB600 + tag_id, // Use 0xB6xx for preview sub-tags
                    name: name.to_string(),
                    writable: Some("int32u".to_string()),
                    groups: Some("Canon".to_string()),
                    notes: None,
                });
                eprintln!(
                    "Canon preview tag (simple): {} (0x{:04x})",
                    name,
                    0xB600 + tag_id
                );
            }
        }

        // Parse complex hash assignments (like "2 => { Name => 'PreviewImageLength', ... }")
        for cap in preview_complex_re.captures_iter(preview_content) {
            if let Ok(tag_id) = cap[1].parse::<u16>() {
                let name = &cap[2];
                tags.push(TagDef {
                    tag_id: 0xB600 + tag_id, // Use 0xB6xx for preview sub-tags
                    name: name.to_string(),
                    writable: Some("int32u".to_string()),
                    groups: Some("Canon".to_string()),
                    notes: None,
                });
                eprintln!(
                    "Canon preview tag (complex): {} (0x{:04x})",
                    name,
                    0xB600 + tag_id
                );
            }
        }
    }

    // Sort tags by ID for consistent output
    tags.sort_by_key(|t| t.tag_id);

    // Remove duplicates (keep first occurrence)
    tags.dedup_by_key(|t| t.tag_id);

    eprintln!(
        "Total Canon tags parsed (including preview): {}",
        tags.len()
    );

    tags
}

/// Read ExifTool version from exiftool-sync.toml
fn read_exiftool_version() -> Option<String> {
    let content = fs::read_to_string("exiftool-sync.toml").ok()?;

    // Simple parsing - look for version line
    for line in content.lines() {
        if line.trim().starts_with("version = ") {
            let version = line
                .trim_start_matches("version = ")
                .trim()
                .trim_matches('"');
            return Some(version.to_string());
        }
    }

    None
}

fn generate_rust_code(
    exif_tags: &[TagDef],
    canon_tags: &[TagDef],
    exiftool_version: &str,
) -> String {
    let mut code = String::new();

    // Add simple attribution header
    code.push_str(&format!(
        "// AUTO-GENERATED from ExifTool v{}\n",
        exiftool_version
    ));
    code.push_str("// Source: lib/Image/ExifTool/Exif.pm (Main table)\n");
    code.push_str("// Source: lib/Image/ExifTool/Canon.pm (Main, PreviewImageInfo tables)\n");
    code.push_str("// Generated: ");
    code.push_str(&chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    code.push_str(" by build.rs\n");
    code.push_str("// DO NOT EDIT - Regenerate with `cargo build`\n\n");
    code.push_str("use crate::core::types::{TagInfo, ExifFormat};\n\n");

    // Generate EXIF tags table
    code.push_str("pub const EXIF_TAGS: &[(u16, TagInfo)] = &[\n");

    for tag in exif_tags {
        let format = map_writable_to_format(&tag.writable);
        let group = tag
            .groups
            .as_ref()
            .map(|g| format!("Some(\"{g}\")"))
            .unwrap_or("None".to_string());

        code.push_str(&format!(
            "    (0x{:04x}, TagInfo {{ name: \"{}\", format: {}, group: {} }}),\n",
            tag.tag_id, tag.name, format, group
        ));
    }

    code.push_str("];\n\n");

    // Generate Canon tags table
    code.push_str("pub const CANON_TAGS: &[(u16, TagInfo)] = &[\n");

    for tag in canon_tags {
        let format = map_writable_to_format(&tag.writable);
        let group = tag
            .groups
            .as_ref()
            .map(|g| format!("Some(\"{g}\")"))
            .unwrap_or("Some(\"Canon\")".to_string());

        code.push_str(&format!(
            "    (0x{:04x}, TagInfo {{ name: \"{}\", format: {}, group: {} }}),\n",
            tag.tag_id, tag.name, format, group
        ));
    }

    code.push_str("];\n\n");

    // Add lookup functions
    code.push_str("pub fn lookup_tag(tag_id: u16) -> Option<&'static TagInfo> {\n");
    code.push_str("    EXIF_TAGS.iter().find(|(id, _)| *id == tag_id).map(|(_, info)| info)\n");
    code.push_str("}\n\n");

    code.push_str("pub fn lookup_canon_tag(tag_id: u16) -> Option<&'static TagInfo> {\n");
    code.push_str("    CANON_TAGS.iter().find(|(id, _)| *id == tag_id).map(|(_, info)| info)\n");
    code.push_str("}\n");

    code
}

fn map_writable_to_format(writable: &Option<String>) -> String {
    match writable.as_ref().map(|s| s.as_str()) {
        Some("string") => "ExifFormat::Ascii",
        Some("int8u") => "ExifFormat::U8",
        Some("int16u") => "ExifFormat::U16",
        Some("int32u") => "ExifFormat::U32",
        Some("int8s") => "ExifFormat::I8",
        Some("int16s") => "ExifFormat::I16",
        Some("int32s") => "ExifFormat::I32",
        Some("rational64u") => "ExifFormat::Rational",
        Some("rational64s") => "ExifFormat::SignedRational",
        Some("float") => "ExifFormat::F32",
        Some("double") => "ExifFormat::F64",
        Some("undef") => "ExifFormat::Undefined",
        Some("binary") => "ExifFormat::Undefined",
        // Handle arrays - default to single value type for now
        Some(s) if s.contains('[') => {
            if s.starts_with("int16u") {
                "ExifFormat::U16"
            } else if s.starts_with("int32u") {
                "ExifFormat::U32"
            } else {
                "ExifFormat::Undefined"
            }
        }
        None => "ExifFormat::Ascii", // Default for tags without explicit format
        _ => "ExifFormat::Undefined", // Default for unknown types
    }
    .to_string()
}
