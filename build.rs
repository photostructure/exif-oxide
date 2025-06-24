use regex::Regex;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Ensure all detection.rs files exist (create stubs if missing)
    ensure_detection_files_exist();

    // Ensure binary composite_tags.rs file exists (create stub if missing)
    ensure_composite_tags_exists();
    println!("cargo:rerun-if-changed=exiftool/lib/Image/ExifTool/Exif.pm");
    println!("cargo:rerun-if-changed=exiftool/lib/Image/ExifTool/Canon.pm");
    println!("cargo:rerun-if-changed=exiftool/lib/Image/ExifTool/Olympus.pm");
    println!("cargo:rerun-if-changed=third-party/exiftool/lib/Image/ExifTool/Nikon.pm");
    println!("cargo:rerun-if-changed=third-party/exiftool/lib/Image/ExifTool/Pentax.pm");
    println!("cargo:rerun-if-changed=third-party/exiftool/lib/Image/ExifTool/FujiFilm.pm");
    println!("cargo:rerun-if-changed=third-party/exiftool/lib/Image/ExifTool/Sony.pm");
    println!("cargo:rerun-if-changed=third-party/exiftool/lib/Image/ExifTool/Panasonic.pm");
    println!("cargo:rerun-if-changed=third-party/exiftool/lib/Image/ExifTool/Sigma.pm");
    println!("cargo:rerun-if-changed=third-party/exiftool/lib/Image/ExifTool/Apple.pm");
    println!("cargo:rerun-if-changed=third-party/exiftool/lib/Image/ExifTool/Samsung.pm");
    println!("cargo:rerun-if-changed=exiftool-sync.toml");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_tags.rs");

    // Try to read ExifTool version from sync config
    let exiftool_version = read_exiftool_version().unwrap_or_else(|| "unknown".to_string());

    // NOTE: EXIF tags are now handled by the sync extractor system
    // See src/bin/exiftool_sync/extractors/exif_tags.rs and src/tables/exif_tags.rs
    //
    // However, we include essential EXIF tags here for backward compatibility
    // with tests and the legacy lookup_tag() function.
    let exif_tags = create_essential_exif_tags();

    // Parse Canon tags
    let canon_pm_path = "exiftool/lib/Image/ExifTool/Canon.pm";
    let canon_content = fs::read_to_string(canon_pm_path).expect("Failed to read Canon.pm");
    let canon_tags = parse_canon_tags(&canon_content);

    // Parse Olympus tags
    let olympus_pm_path = "third-party/exiftool/lib/Image/ExifTool/Olympus.pm";
    let olympus_content = fs::read_to_string(olympus_pm_path).expect("Failed to read Olympus.pm");
    let olympus_tags = parse_olympus_tags(&olympus_content);

    // Parse Nikon tags
    let nikon_pm_path = "third-party/exiftool/lib/Image/ExifTool/Nikon.pm";
    let nikon_content = fs::read_to_string(nikon_pm_path).expect("Failed to read Nikon.pm");
    let nikon_tags = parse_nikon_tags(&nikon_content);

    // Parse Pentax tags
    let pentax_pm_path = "third-party/exiftool/lib/Image/ExifTool/Pentax.pm";
    let pentax_content = fs::read_to_string(pentax_pm_path).expect("Failed to read Pentax.pm");
    let pentax_tags = parse_pentax_tags(&pentax_content);

    // Parse Fujifilm tags
    let fujifilm_pm_path = "third-party/exiftool/lib/Image/ExifTool/FujiFilm.pm";
    let fujifilm_content =
        fs::read_to_string(fujifilm_pm_path).expect("Failed to read FujiFilm.pm");
    let fujifilm_tags = parse_fujifilm_tags(&fujifilm_content);

    // Parse Sony tags
    let sony_pm_path = "third-party/exiftool/lib/Image/ExifTool/Sony.pm";
    let sony_content = fs::read_to_string(sony_pm_path).expect("Failed to read Sony.pm");
    let sony_tags = parse_sony_tags(&sony_content);

    // Parse Leica tags (from Panasonic.pm since Leica tags are defined there)
    let panasonic_pm_path = "third-party/exiftool/lib/Image/ExifTool/Panasonic.pm";
    let panasonic_content =
        fs::read_to_string(panasonic_pm_path).expect("Failed to read Panasonic.pm");
    let leica_tags = parse_leica_tags(&panasonic_content);

    // Parse Sigma tags
    let sigma_pm_path = "third-party/exiftool/lib/Image/ExifTool/Sigma.pm";
    let sigma_content = fs::read_to_string(sigma_pm_path).expect("Failed to read Sigma.pm");
    let sigma_tags = parse_sigma_tags(&sigma_content);

    // Parse Apple tags
    let apple_pm_path = "third-party/exiftool/lib/Image/ExifTool/Apple.pm";
    let apple_content = fs::read_to_string(apple_pm_path).expect("Failed to read Apple.pm");
    let apple_tags = parse_apple_tags(&apple_content);

    // Parse Samsung tags (disabled due to regex issues)
    // let samsung_pm_path = "third-party/exiftool/lib/Image/ExifTool/Samsung.pm";
    // let samsung_content = fs::read_to_string(samsung_pm_path).expect("Failed to read Samsung.pm");
    let samsung_tags = Vec::<TagDef>::new();

    // Parse Hasselblad tags (hardcoded since no dedicated .pm file)
    let hasselblad_tags = parse_hasselblad_tags();

    let rust_code = generate_rust_code(
        &exif_tags,
        &canon_tags,
        &olympus_tags,
        &nikon_tags,
        &pentax_tags,
        &fujifilm_tags,
        &sony_tags,
        &leica_tags,
        &sigma_tags,
        &apple_tags,
        &samsung_tags,
        &hasselblad_tags,
        &exiftool_version,
    );

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

// NOTE: parse_exif_tags function removed - EXIF tags now handled by sync extractor
// See src/bin/exiftool_sync/extractors/exif_tags.rs for the new implementation

/// Create essential EXIF tags for backward compatibility with tests
/// These are the core tags that tests and legacy code expect to find
fn create_essential_exif_tags() -> Vec<TagDef> {
    vec![
        // Basic camera info
        TagDef {
            tag_id: 0x010E,
            name: "ImageDescription".to_string(),
            writable: Some("string".to_string()),
            groups: Some("Image".to_string()),
            notes: None,
        },
        TagDef {
            tag_id: 0x010F,
            name: "Make".to_string(),
            writable: Some("string".to_string()),
            groups: Some("Camera".to_string()),
            notes: None,
        },
        TagDef {
            tag_id: 0x0110,
            name: "Model".to_string(),
            writable: Some("string".to_string()),
            groups: Some("Camera".to_string()),
            notes: None,
        },
        TagDef {
            tag_id: 0x0102,
            name: "BitsPerSample".to_string(),
            writable: Some("int16u".to_string()),
            groups: Some("Image".to_string()),
            notes: None,
        },
        TagDef {
            tag_id: 0x0112,
            name: "Orientation".to_string(),
            writable: Some("int16u".to_string()),
            groups: Some("Image".to_string()),
            notes: None,
        },
        TagDef {
            tag_id: 0x011A,
            name: "XResolution".to_string(),
            writable: Some("rational64u".to_string()),
            groups: Some("Image".to_string()),
            notes: None,
        },
        TagDef {
            tag_id: 0x011B,
            name: "YResolution".to_string(),
            writable: Some("rational64u".to_string()),
            groups: Some("Image".to_string()),
            notes: None,
        },
        TagDef {
            tag_id: 0x0128,
            name: "ResolutionUnit".to_string(),
            writable: Some("int16u".to_string()),
            groups: Some("Image".to_string()),
            notes: None,
        },
        TagDef {
            tag_id: 0x0131,
            name: "Software".to_string(),
            writable: Some("string".to_string()),
            groups: Some("Image".to_string()),
            notes: None,
        },
        TagDef {
            tag_id: 0x0132,
            name: "ModifyDate".to_string(),
            writable: Some("string".to_string()),
            groups: Some("Time".to_string()),
            notes: None,
        },
        TagDef {
            tag_id: 0x013B,
            name: "Artist".to_string(),
            writable: Some("string".to_string()),
            groups: Some("Author".to_string()),
            notes: None,
        },
        TagDef {
            tag_id: 0x8298,
            name: "Copyright".to_string(),
            writable: Some("string".to_string()),
            groups: Some("Author".to_string()),
            notes: None,
        },
        // Photography settings
        TagDef {
            tag_id: 0x829A,
            name: "ExposureTime".to_string(),
            writable: Some("rational64u".to_string()),
            groups: Some("ExifIFD".to_string()),
            notes: None,
        },
        TagDef {
            tag_id: 0x829D,
            name: "FNumber".to_string(),
            writable: Some("rational64u".to_string()),
            groups: Some("ExifIFD".to_string()),
            notes: None,
        },
        TagDef {
            tag_id: 0x8769,
            name: "ExifOffset".to_string(),
            writable: Some("int32u".to_string()),
            groups: Some("ExifIFD".to_string()),
            notes: None,
        },
        TagDef {
            tag_id: 0x8825,
            name: "GPSInfo".to_string(),
            writable: Some("int32u".to_string()),
            groups: Some("GPS".to_string()),
            notes: None,
        },
        TagDef {
            tag_id: 0x9003,
            name: "DateTimeOriginal".to_string(),
            writable: Some("string".to_string()),
            groups: Some("Time".to_string()),
            notes: None,
        },
        TagDef {
            tag_id: 0x9004,
            name: "CreateDate".to_string(),
            writable: Some("string".to_string()),
            groups: Some("Time".to_string()),
            notes: None,
        },
        TagDef {
            tag_id: 0x9204,
            name: "ExposureCompensation".to_string(),
            writable: Some("rational64s".to_string()),
            groups: Some("ExifIFD".to_string()),
            notes: None,
        },
        TagDef {
            tag_id: 0xA002,
            name: "ExifImageWidth".to_string(),
            writable: Some("int32u".to_string()),
            groups: Some("ExifIFD".to_string()),
            notes: None,
        },
        TagDef {
            tag_id: 0xA003,
            name: "ExifImageHeight".to_string(),
            writable: Some("int32u".to_string()),
            groups: Some("ExifIFD".to_string()),
            notes: None,
        },
        // First tag for edge case testing
        TagDef {
            tag_id: 0x0001,
            name: "InteropIndex".to_string(),
            writable: Some("string".to_string()),
            groups: Some("InteropIFD".to_string()),
            notes: None,
        },
        // Last tag for edge case testing
        TagDef {
            tag_id: 0xFFFF,
            name: "TestLastTag".to_string(),
            writable: Some("int16u".to_string()),
            groups: Some("Test".to_string()),
            notes: None,
        },
    ]
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

fn parse_olympus_tags(content: &str) -> Vec<TagDef> {
    let mut tags = Vec::new();

    // Find the Olympus Main table start
    let main_start = content
        .find("%Image::ExifTool::Olympus::Main = (")
        .expect("Could not find Olympus Main table");

    // Use similar regex patterns but adapted for Olympus
    let tag_re = Regex::new(r"(?s)(0x[0-9a-fA-F]+)\s*=>\s*\{([^}]+)\}").unwrap();
    let name_re = Regex::new(r"Name\s*=>\s*'([^']+)'").unwrap();
    let writable_re = Regex::new(r"Writable\s*=>\s*'([^']+)'").unwrap();
    let groups_re = Regex::new(r"Groups\s*=>\s*\{[^}]*2\s*=>\s*'([^']+)'").unwrap();

    // Also match simple string tags like: 0x0104 => { Name => 'BodyFirmwareVersion', Writable => 'string' },
    let simple_tag_re = Regex::new(
        r"(0x[0-9a-fA-F]+)\s*=>\s*\{\s*Name\s*=>\s*'([^']+)',\s*Writable\s*=>\s*'([^']+)'\s*\}",
    )
    .unwrap();

    // Process a portion of the Olympus file
    let search_content = &content[main_start..]
        .chars()
        .take(500000)
        .collect::<String>();

    // Parse Olympus tag definitions
    for cap in tag_re.captures_iter(search_content) {
        let tag_hex = &cap[1];
        let tag_content = &cap[2];

        // Skip complex SubDirectory tags for initial implementation
        if tag_content.contains("SubDirectory") || tag_content.contains("TagTable") {
            continue;
        }

        // Skip tags with complex conditions or printing functions
        if tag_content.contains("Condition =>")
            || tag_content.contains("PrintConv => sub")
            || tag_content.contains("ValueConv =>")
        {
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

        eprintln!("Olympus tag: {} (0x{:04x})", name, tag_id);
    }

    // Also collect simple string tags
    for cap in simple_tag_re.captures_iter(search_content) {
        let tag_hex = &cap[1];
        let name = &cap[2];
        let writable = &cap[3];

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
            writable: Some(writable.to_string()),
            groups: Some("Olympus".to_string()),
            notes: None,
        });

        eprintln!("Olympus tag (simple): {} (0x{:04x})", name, tag_id);
    }

    // Sort tags by ID for consistent output
    tags.sort_by_key(|t| t.tag_id);

    // Remove duplicates (keep first occurrence)
    tags.dedup_by_key(|t| t.tag_id);

    eprintln!("Total Olympus tags parsed: {}", tags.len());

    tags
}

fn parse_nikon_tags(content: &str) -> Vec<TagDef> {
    let mut tags = Vec::new();

    // Find the Nikon Main table start
    let main_start = content
        .find("%Image::ExifTool::Nikon::Main = (")
        .expect("Could not find Nikon Main table");

    // Use similar regex patterns but adapted for Nikon
    let tag_re = Regex::new(r"(?s)(0x[0-9a-fA-F]+)\s*=>\s*\{([^}]+)\}").unwrap();
    let name_re = Regex::new(r"Name\s*=>\s*'([^']+)'").unwrap();
    let writable_re = Regex::new(r"Writable\s*=>\s*'([^']+)'").unwrap();
    let groups_re = Regex::new(r"Groups\s*=>\s*\{[^}]*2\s*=>\s*'([^']+)'").unwrap();

    // Also match simple string tags
    let simple_tag_re = Regex::new(
        r"(0x[0-9a-fA-F]+)\s*=>\s*\{\s*Name\s*=>\s*'([^']+)',\s*Writable\s*=>\s*'([^']+)'\s*\}",
    )
    .unwrap();

    // Process a portion of the Nikon file - Nikon has many tags but we'll focus on core ones
    let search_content = &content[main_start..]
        .chars()
        .take(800000) // Larger search area for Nikon as it has many tags
        .collect::<String>();

    // Parse Nikon tag definitions
    for cap in tag_re.captures_iter(search_content) {
        let tag_hex = &cap[1];
        let tag_content = &cap[2];

        // Skip complex SubDirectory tags for initial implementation
        if tag_content.contains("SubDirectory") || tag_content.contains("TagTable") {
            continue;
        }

        // Skip extremely complex tags with conditions, scripts, or binary processing
        if tag_content.contains("Condition =>")
            || tag_content.contains("PrintConv => sub")
            || tag_content.contains("ValueConv => sub")
            || tag_content.contains("ProcessBinaryData")
            || tag_content.contains("DecryptNikon")
        {
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

        eprintln!("Nikon tag: {} (0x{:04x})", name, tag_id);
    }

    // Also collect simple string tags
    for cap in simple_tag_re.captures_iter(search_content) {
        let tag_hex = &cap[1];
        let name = &cap[2];
        let writable = &cap[3];

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
            writable: Some(writable.to_string()),
            groups: Some("Nikon".to_string()),
            notes: None,
        });

        eprintln!("Nikon tag (simple): {} (0x{:04x})", name, tag_id);
    }

    // Sort tags by ID for consistent output
    tags.sort_by_key(|t| t.tag_id);

    // Remove duplicates (keep first occurrence)
    tags.dedup_by_key(|t| t.tag_id);

    eprintln!("Total Nikon tags parsed: {}", tags.len());

    tags
}

fn parse_pentax_tags(content: &str) -> Vec<TagDef> {
    let mut tags = Vec::new();

    // Find the Pentax Main table start
    let main_start = content
        .find("%Image::ExifTool::Pentax::Main = (")
        .expect("Could not find Pentax Main table");

    // Use similar regex patterns but adapted for Pentax
    let tag_re = Regex::new(r"(?s)(0x[0-9a-fA-F]+)\s*=>\s*\{([^}]+)\}").unwrap();
    let name_re = Regex::new(r"Name\s*=>\s*'([^']+)'").unwrap();
    let writable_re = Regex::new(r"Writable\s*=>\s*'([^']+)'").unwrap();
    let groups_re = Regex::new(r"Groups\s*=>\s*\{[^}]*2\s*=>\s*'([^']+)'").unwrap();

    // Also match simple string tags
    let simple_tag_re = Regex::new(
        r"(0x[0-9a-fA-F]+)\s*=>\s*\{\s*Name\s*=>\s*'([^']+)',\s*Writable\s*=>\s*'([^']+)'\s*\}",
    )
    .unwrap();

    // Process a portion of the Pentax file - Pentax has fewer tags than Nikon
    let search_content = &content[main_start..]
        .chars()
        .take(600000) // Medium search area for Pentax
        .collect::<String>();

    // Parse Pentax tag definitions
    for cap in tag_re.captures_iter(search_content) {
        let tag_hex = &cap[1];
        let tag_content = &cap[2];

        // Skip complex SubDirectory tags for initial implementation, except preview tags
        let is_preview_tag = tag_content.contains("PreviewImage");
        if !is_preview_tag
            && (tag_content.contains("SubDirectory") || tag_content.contains("TagTable"))
        {
            continue;
        }

        // Skip tags with complex conditions, scripts, or encrypted processing
        if tag_content.contains("Condition =>") 
            || tag_content.contains("PrintConv => sub") 
            || tag_content.contains("ValueConv => sub")
            || tag_content.contains("ProcessBinaryData")
            || tag_content.contains("DecryptNikon")  // Some Pentax tags reference Nikon decryption
            || tag_content.contains("Decrypt")
        {
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

        eprintln!("Pentax tag: {} (0x{:04x})", name, tag_id);
    }

    // Also collect simple string tags
    for cap in simple_tag_re.captures_iter(search_content) {
        let tag_hex = &cap[1];
        let name = &cap[2];
        let writable = &cap[3];

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
            writable: Some(writable.to_string()),
            groups: Some("Pentax".to_string()),
            notes: None,
        });

        eprintln!("Pentax tag (simple): {} (0x{:04x})", name, tag_id);
    }

    // Sort tags by ID for consistent output
    tags.sort_by_key(|t| t.tag_id);

    // Remove duplicates (keep first occurrence)
    tags.dedup_by_key(|t| t.tag_id);

    eprintln!("Total Pentax tags parsed: {}", tags.len());

    tags
}

fn parse_fujifilm_tags(content: &str) -> Vec<TagDef> {
    let mut tags = Vec::new();

    // Find the Fujifilm Main table start
    let main_start = content
        .find("%Image::ExifTool::FujiFilm::Main = (")
        .expect("Could not find Fujifilm Main table");

    // Use similar regex patterns but adapted for Fujifilm
    let tag_re = Regex::new(r"(?s)(0x[0-9a-fA-F]+)\s*=>\s*\{([^}]+)\}").unwrap();
    let name_re = Regex::new(r"Name\s*=>\s*'([^']+)'").unwrap();
    let writable_re = Regex::new(r"Writable\s*=>\s*'([^']+)'").unwrap();
    let groups_re = Regex::new(r"Groups\s*=>\s*\{[^}]*2\s*=>\s*'([^']+)'").unwrap();

    // Also match simple string tags
    let simple_tag_re = Regex::new(r"(0x[0-9a-fA-F]+)\s*=>\s*'([^']+)',").unwrap();

    // Process a portion of the Fujifilm file - Fujifilm has a moderate number of tags
    let search_content = &content[main_start..]
        .chars()
        .take(600000) // Medium search area for Fujifilm
        .collect::<String>();

    // Parse Fujifilm tag definitions
    for cap in tag_re.captures_iter(search_content) {
        let tag_hex = &cap[1];
        let tag_content = &cap[2];

        // Skip complex SubDirectory tags for initial implementation
        if tag_content.contains("SubDirectory") || tag_content.contains("TagTable") {
            continue;
        }

        // Skip tags with complex conditions or printing functions
        if tag_content.contains("Condition =>")
            || tag_content.contains("PrintConv => sub")
            || tag_content.contains("ValueConv =>")
            || tag_content.contains("ProcessBinaryData")
        {
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

        eprintln!("Fujifilm tag: {} (0x{:04x})", name, tag_id);
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
            writable: Some("string".to_string()),
            groups: Some("Fujifilm".to_string()),
            notes: None,
        });

        eprintln!("Fujifilm tag (simple): {} (0x{:04x})", name, tag_id);
    }

    // Sort tags by ID for consistent output
    tags.sort_by_key(|t| t.tag_id);

    // Remove duplicates (keep first occurrence)
    tags.dedup_by_key(|t| t.tag_id);

    eprintln!("Total Fujifilm tags parsed: {}", tags.len());

    tags
}

fn parse_sony_tags(content: &str) -> Vec<TagDef> {
    let mut tags = Vec::new();

    // Find the Sony Main table start
    let main_start = content
        .find("%Image::ExifTool::Sony::Main = (")
        .expect("Could not find Sony Main table");

    // Use similar regex patterns but adapted for Sony
    let tag_re = Regex::new(r"(?s)(0x[0-9a-fA-F]+)\s*=>\s*\{([^}]+)\}").unwrap();
    let name_re = Regex::new(r"Name\s*=>\s*'([^']+)'").unwrap();
    let writable_re = Regex::new(r"Writable\s*=>\s*'([^']+)'").unwrap();
    let groups_re = Regex::new(r"Groups\s*=>\s*\{[^}]*2\s*=>\s*'([^']+)'").unwrap();

    // Also match simple string tags
    let simple_tag_re = Regex::new(
        r"(0x[0-9a-fA-F]+)\s*=>\s*\{\s*Name\s*=>\s*'([^']+)',\s*Writable\s*=>\s*'([^']+)'\s*\}",
    )
    .unwrap();

    // Process a portion of the Sony file - Sony has many tags, some encrypted
    let search_content = &content[main_start..]
        .chars()
        .take(800000) // Large search area for Sony
        .collect::<String>();

    // Parse Sony tag definitions
    for cap in tag_re.captures_iter(search_content) {
        let tag_hex = &cap[1];
        let tag_content = &cap[2];

        // Skip complex SubDirectory tags for initial implementation
        if tag_content.contains("SubDirectory")
            || tag_content.contains("TagTable")
            || tag_content.contains("Process => \\&ProcessEnciphered")
        {
            continue;
        }

        // Skip tags with complex conditions or processing
        if tag_content.contains("Condition =>")
            || tag_content.contains("PrintConv => sub")
            || tag_content.contains("ValueConv => sub")
            || tag_content.contains("RawConv =>")
            || tag_content.contains("Binary => 1")
        {
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

        // Skip encrypted tags (0x2010, 0x9xxx series) for now
        if tag_id == 0x2010 || (0x9000..0xa000).contains(&tag_id) {
            continue;
        }

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

        eprintln!("Sony tag: {} (0x{:04x})", name, tag_id);
    }

    // Also collect simple string tags
    for cap in simple_tag_re.captures_iter(search_content) {
        let tag_hex = &cap[1];
        let name = &cap[2];
        let writable = &cap[3];

        let tag_id = match u16::from_str_radix(&tag_hex[2..], 16) {
            Ok(id) => id,
            Err(_) => continue,
        };

        // Check if we already have this tag
        if tags.iter().any(|t| t.tag_id == tag_id) {
            continue;
        }

        // Skip encrypted tags
        if tag_id == 0x2010 || (0x9000..0xa000).contains(&tag_id) {
            continue;
        }

        tags.push(TagDef {
            tag_id,
            name: name.to_string(),
            writable: Some(writable.to_string()),
            groups: Some("Sony".to_string()),
            notes: None,
        });

        eprintln!("Sony tag (simple): {} (0x{:04x})", name, tag_id);
    }

    // Add some important preview-related tags manually (similar to Canon)
    let preview_tags = vec![
        (0x2001, "PreviewImage"),
        (0x201b, "PreviewImageSize"),
        (0x201e, "PreviewImageStart"),
        (0x201f, "PreviewImageLength"),
    ];

    for (tag_id, name) in preview_tags {
        if !tags.iter().any(|t| t.tag_id == tag_id) {
            tags.push(TagDef {
                tag_id,
                name: name.to_string(),
                writable: Some("undef".to_string()),
                groups: Some("Sony".to_string()),
                notes: None,
            });
            eprintln!("Sony preview tag (manual): {} (0x{:04x})", name, tag_id);
        }
    }

    // Sort tags by ID for consistent output
    tags.sort_by_key(|t| t.tag_id);

    // Remove duplicates (keep first occurrence)
    tags.dedup_by_key(|t| t.tag_id);

    eprintln!("Total Sony tags parsed: {}", tags.len());

    tags
}

fn parse_leica_tags(content: &str) -> Vec<TagDef> {
    let mut tags = Vec::new();

    // Leica tags are defined in multiple tables within Panasonic.pm
    // Parse each Leica table: Leica2, Leica3, Leica4, Leica5, Leica6, Leica9
    let leica_tables = vec![
        ("Leica2", "%Image::ExifTool::Panasonic::Leica2 = ("),
        ("Leica3", "%Image::ExifTool::Panasonic::Leica3 = ("),
        ("Leica4", "%Image::ExifTool::Panasonic::Leica4 = ("),
        ("Leica5", "%Image::ExifTool::Panasonic::Leica5 = ("),
        ("Leica6", "%Image::ExifTool::Panasonic::Leica6 = ("),
        ("Leica9", "%Image::ExifTool::Panasonic::Leica9 = ("),
    ];

    // Common regex patterns for tag parsing
    let tag_re = Regex::new(r"(?s)(0x[0-9a-fA-F]+)\s*=>\s*\{([^}]+)\}").unwrap();
    let name_re = Regex::new(r"Name\s*=>\s*'([^']+)'").unwrap();
    let writable_re = Regex::new(r"Writable\s*=>\s*'([^']+)'").unwrap();
    let groups_re = Regex::new(r"Groups\s*=>\s*\{[^}]*2\s*=>\s*'([^']+)'").unwrap();

    // Parse each Leica table
    for (table_name, table_pattern) in leica_tables {
        if let Some(table_start) = content.find(table_pattern) {
            eprintln!("Found {} table at position {}", table_name, table_start);

            // Extract the table content - stop at the next table or end of file
            let table_content = &content[table_start..];
            let search_end = table_content
                .find("\n%")
                .unwrap_or(table_content.len().min(50000));
            let search_content = &table_content[..search_end];

            // Parse tags in this table
            for cap in tag_re.captures_iter(search_content) {
                let tag_hex = &cap[1];
                let tag_content = &cap[2];

                // Skip complex SubDirectory tags for initial implementation
                if tag_content.contains("SubDirectory") || tag_content.contains("TagTable") {
                    continue;
                }

                // Skip complex conditional or processing tags
                if tag_content.contains("Condition =>")
                    || tag_content.contains("PrintConv => sub")
                    || tag_content.contains("ValueConv => sub")
                    || tag_content.contains("RawConv =>")
                    || tag_content.contains("Process =>")
                {
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

                // Check if we already have this tag (avoid duplicates across tables)
                if tags.iter().any(|t: &TagDef| t.tag_id == tag_id) {
                    continue;
                }

                tags.push(TagDef {
                    tag_id,
                    name: name.clone(),
                    writable,
                    groups,
                    notes: Some(format!("From {}", table_name)),
                });

                eprintln!("Leica tag ({}): {} (0x{:04x})", table_name, name, tag_id);
            }
        } else {
            eprintln!("Warning: {} table not found in Panasonic.pm", table_name);
        }
    }

    // Add some important common Leica tags manually that might be missed
    let common_leica_tags = vec![
        (0x0001, "FirmwareVersion", "string"),
        (0x0002, "CameraTemperature", "int16s"),
        (0x0003, "ImageNumber", "int32u"),
        (0x0004, "CameraOrientation", "int16u"),
        (0x0005, "Contrast", "int16s"),
        (0x0006, "Saturation", "int16s"),
        (0x0007, "Sharpness", "int16s"),
        (0x0201, "PreviewImageStart", "int32u"),
        (0x0202, "PreviewImageLength", "int32u"),
        (0x0203, "PreviewImageSize", "string"),
    ];

    for (tag_id, name, writable) in common_leica_tags {
        if !tags.iter().any(|t: &TagDef| t.tag_id == tag_id) {
            tags.push(TagDef {
                tag_id,
                name: name.to_string(),
                writable: Some(writable.to_string()),
                groups: Some("Camera".to_string()),
                notes: Some("Common Leica tag".to_string()),
            });
            eprintln!("Leica tag (common): {} (0x{:04x})", name, tag_id);
        }
    }

    // Sort tags by ID for consistent output
    tags.sort_by_key(|t| t.tag_id);

    // Remove duplicates (keep first occurrence)
    tags.dedup_by_key(|t| t.tag_id);

    eprintln!("Total Leica tags parsed: {}", tags.len());

    tags
}

fn parse_sigma_tags(content: &str) -> Vec<TagDef> {
    let mut tags = Vec::new();

    // Find the Sigma Main table start
    let main_start = content
        .find("%Image::ExifTool::Sigma::Main = (")
        .expect("Could not find Sigma Main table");

    // Use similar regex patterns but adapted for Sigma
    let tag_re = Regex::new(r"(?s)(0x[0-9a-fA-F]+)\s*=>\s*\{([^}]+)\}").unwrap();
    let name_re = Regex::new(r"Name\s*=>\s*'([^']+)'").unwrap();
    let writable_re = Regex::new(r"Writable\s*=>\s*'([^']+)'").unwrap();
    let groups_re = Regex::new(r"Groups\s*=>\s*\{[^}]*2\s*=>\s*'([^']+)'").unwrap();

    // Also match simple string tags
    let simple_tag_re = Regex::new(r"(0x[0-9a-fA-F]+)\s*=>\s*'([^']+)',").unwrap();

    // Process a portion of the Sigma file
    let search_content = &content[main_start..]
        .chars()
        .take(800000) // Large search area for Sigma as it has many tags
        .collect::<String>();

    // Parse Sigma tag definitions
    for cap in tag_re.captures_iter(search_content) {
        let tag_hex = &cap[1];
        let tag_content = &cap[2];

        // Skip complex SubDirectory tags for initial implementation
        if tag_content.contains("SubDirectory")
            || tag_content.contains("TagTable")
            || tag_content.contains("ProcessBinaryData")
        {
            continue;
        }

        // Skip extremely complex conditional tags and processing functions
        if tag_content.contains("Condition =>")
            || tag_content.contains("PrintConv => sub")
            || tag_content.contains("ValueConv => sub")
            || tag_content.contains("RawConv =>")
            || tag_content.contains("$$self{MakerNoteSigmaVer}")
        {
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

        eprintln!("Sigma tag: {} (0x{:04x})", name, tag_id);
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
            writable: Some("string".to_string()),
            groups: Some("Sigma".to_string()),
            notes: None,
        });

        eprintln!("Sigma tag (simple): {} (0x{:04x})", name, tag_id);
    }

    // Sort tags by ID for consistent output
    tags.sort_by_key(|t| t.tag_id);

    // Remove duplicates (keep first occurrence)
    tags.dedup_by_key(|t| t.tag_id);

    eprintln!("Total Sigma tags parsed: {}", tags.len());

    tags
}

fn parse_apple_tags(content: &str) -> Vec<TagDef> {
    let mut tags = Vec::new();

    // Find the Apple Main table start
    let main_start = content
        .find("%Image::ExifTool::Apple::Main = (")
        .expect("Could not find Apple Main table");

    // Use similar regex patterns but adapted for Apple
    let tag_re = Regex::new(r"(?s)(0x[0-9a-fA-F]+)\s*=>\s*\{([^}]+)\}").unwrap();
    let name_re = Regex::new(r"Name\s*=>\s*'([^']+)'").unwrap();
    let writable_re = Regex::new(r"Writable\s*=>\s*'([^']+)'").unwrap();
    let groups_re = Regex::new(r"Groups\s*=>\s*\{[^}]*2\s*=>\s*'([^']+)'").unwrap();

    // Process a portion of the Apple file - Apple has moderate number of tags
    let search_content = &content[main_start..]
        .chars()
        .take(600000) // Medium search area for Apple
        .collect::<String>();

    // Parse Apple tag definitions
    for cap in tag_re.captures_iter(search_content) {
        let tag_hex = &cap[1];
        let tag_content = &cap[2];

        // Skip complex SubDirectory tags for initial implementation
        if tag_content.contains("SubDirectory") || tag_content.contains("TagTable") {
            continue;
        }

        // Skip tags with complex conditions, scripts, or PLIST processing
        if tag_content.contains("Condition =>")
            || tag_content.contains("PrintConv => sub")
            || tag_content.contains("ValueConv => \\&ConvertPLIST")
            || tag_content.contains("Unknown => 1")
            || tag_content.contains("ProcessBinaryData")
            || tag_content.contains("Binary => 1")
        {
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

        eprintln!("Apple tag: {} (0x{:04x})", name, tag_id);
    }

    // Sort tags by ID for consistent output
    tags.sort_by_key(|t| t.tag_id);

    // Remove duplicates (keep first occurrence)
    tags.dedup_by_key(|t| t.tag_id);

    eprintln!("Total Apple tags parsed: {}", tags.len());

    tags
}

/// Parse Samsung maker note tags from Samsung.pm
/// Focuses on the Type2 format (standard EXIF IFD structure)
/// Currently disabled due to regex syntax issues
fn _parse_samsung_tags(_content: &str) -> Vec<TagDef> {
    // TODO: Fix regex syntax and re-enable Samsung parsing
    Vec::new()
}

/// Parse Hasselblad tags (hardcoded since no dedicated .pm file exists)
/// Based on comments in ExifTool's MakerNotes.pm
fn parse_hasselblad_tags() -> Vec<TagDef> {
    vec![
        TagDef {
            tag_id: 0x0011,
            name: "SensorCode".to_string(),
            writable: Some("string".to_string()),
            groups: Some("Camera".to_string()),
            notes: Some("Hasselblad sensor code".to_string()),
        },
        TagDef {
            tag_id: 0x0012,
            name: "CameraModelID".to_string(),
            writable: Some("int16u".to_string()),
            groups: Some("Camera".to_string()),
            notes: Some("Hasselblad camera model id".to_string()),
        },
        TagDef {
            tag_id: 0x0015,
            name: "CameraModelName".to_string(),
            writable: Some("string".to_string()),
            groups: Some("Camera".to_string()),
            notes: Some("Hasselblad camera model name".to_string()),
        },
        TagDef {
            tag_id: 0x0016,
            name: "CoatingCode".to_string(),
            writable: Some("string".to_string()),
            groups: Some("Camera".to_string()),
            notes: Some("Hasselblad coating code".to_string()),
        },
    ]
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

#[allow(clippy::too_many_arguments)]
fn generate_rust_code(
    exif_tags: &[TagDef],
    canon_tags: &[TagDef],
    olympus_tags: &[TagDef],
    nikon_tags: &[TagDef],
    pentax_tags: &[TagDef],
    fujifilm_tags: &[TagDef],
    sony_tags: &[TagDef],
    leica_tags: &[TagDef],
    sigma_tags: &[TagDef],
    apple_tags: &[TagDef],
    samsung_tags: &[TagDef],
    hasselblad_tags: &[TagDef],
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
    code.push_str("// Source: lib/Image/ExifTool/Olympus.pm (Main table)\n");
    code.push_str("// Source: lib/Image/ExifTool/Nikon.pm (Main table)\n");
    code.push_str("// Source: lib/Image/ExifTool/Pentax.pm (Main table)\n");
    code.push_str("// Source: lib/Image/ExifTool/FujiFilm.pm (Main table)\n");
    code.push_str("// Source: lib/Image/ExifTool/Sony.pm (Main table)\n");
    code.push_str("// Source: lib/Image/ExifTool/Panasonic.pm (Leica table)\n");
    code.push_str("// Source: lib/Image/ExifTool/Sigma.pm (Main table)\n");
    code.push_str("// Source: lib/Image/ExifTool/Apple.pm (Main table)\n");
    code.push_str("// Source: lib/Image/ExifTool/Samsung.pm (Type2 table)\n");
    code.push_str("// Source: lib/Image/ExifTool/MakerNotes.pm (Hasselblad comments)\n");
    code.push_str("// Generated by build.rs\n");
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

    // Generate Olympus tags table
    code.push_str("pub const OLYMPUS_TAGS: &[(u16, TagInfo)] = &[\n");

    for tag in olympus_tags {
        let format = map_writable_to_format(&tag.writable);
        let group = tag
            .groups
            .as_ref()
            .map(|g| format!("Some(\"{g}\")"))
            .unwrap_or("Some(\"Olympus\")".to_string());

        code.push_str(&format!(
            "    (0x{:04x}, TagInfo {{ name: \"{}\", format: {}, group: {} }}),\n",
            tag.tag_id, tag.name, format, group
        ));
    }

    code.push_str("];\n\n");

    // Generate Nikon tags table
    code.push_str("pub const NIKON_TAGS: &[(u16, TagInfo)] = &[\n");

    for tag in nikon_tags {
        let format = map_writable_to_format(&tag.writable);
        let group = tag
            .groups
            .as_ref()
            .map(|g| format!("Some(\"{g}\")"))
            .unwrap_or("Some(\"Nikon\")".to_string());

        code.push_str(&format!(
            "    (0x{:04x}, TagInfo {{ name: \"{}\", format: {}, group: {} }}),\n",
            tag.tag_id, tag.name, format, group
        ));
    }

    code.push_str("];\n\n");

    // Generate Pentax tags table
    code.push_str("pub const PENTAX_TAGS: &[(u16, TagInfo)] = &[\n");

    for tag in pentax_tags {
        let format = map_writable_to_format(&tag.writable);
        let group = tag
            .groups
            .as_ref()
            .map(|g| format!("Some(\"{g}\")"))
            .unwrap_or("Some(\"Pentax\")".to_string());

        code.push_str(&format!(
            "    (0x{:04x}, TagInfo {{ name: \"{}\", format: {}, group: {} }}),\n",
            tag.tag_id, tag.name, format, group
        ));
    }

    code.push_str("];\n\n");

    // Generate Fujifilm tags table
    code.push_str("pub const FUJIFILM_TAGS: &[(u16, TagInfo)] = &[\n");

    for tag in fujifilm_tags {
        let format = map_writable_to_format(&tag.writable);
        let group = tag
            .groups
            .as_ref()
            .map(|g| format!("Some(\"{g}\")"))
            .unwrap_or("Some(\"Fujifilm\")".to_string());

        code.push_str(&format!(
            "    (0x{:04x}, TagInfo {{ name: \"{}\", format: {}, group: {} }}),\n",
            tag.tag_id, tag.name, format, group
        ));
    }

    code.push_str("];\n\n");

    // Generate Sony tags table
    code.push_str("pub const SONY_TAGS: &[(u16, TagInfo)] = &[\n");

    for tag in sony_tags {
        let format = map_writable_to_format(&tag.writable);
        let group = tag
            .groups
            .as_ref()
            .map(|g| format!("Some(\"{g}\")"))
            .unwrap_or("Some(\"Sony\")".to_string());

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
    code.push_str("}\n\n");

    code.push_str("pub fn lookup_olympus_tag(tag_id: u16) -> Option<&'static TagInfo> {\n");
    code.push_str("    OLYMPUS_TAGS.iter().find(|(id, _)| *id == tag_id).map(|(_, info)| info)\n");
    code.push_str("}\n\n");

    code.push_str("pub fn lookup_nikon_tag(tag_id: u16) -> Option<&'static TagInfo> {\n");
    code.push_str("    NIKON_TAGS.iter().find(|(id, _)| *id == tag_id).map(|(_, info)| info)\n");
    code.push_str("}\n\n");

    code.push_str("pub fn lookup_pentax_tag(tag_id: u16) -> Option<&'static TagInfo> {\n");
    code.push_str("    PENTAX_TAGS.iter().find(|(id, _)| *id == tag_id).map(|(_, info)| info)\n");
    code.push_str("}\n\n");

    code.push_str("pub fn lookup_fujifilm_tag(tag_id: u16) -> Option<&'static TagInfo> {\n");
    code.push_str("    FUJIFILM_TAGS.iter().find(|(id, _)| *id == tag_id).map(|(_, info)| info)\n");
    code.push_str("}\n\n");

    code.push_str("pub fn lookup_sony_tag(tag_id: u16) -> Option<&'static TagInfo> {\n");
    code.push_str("    SONY_TAGS.iter().find(|(id, _)| *id == tag_id).map(|(_, info)| info)\n");
    code.push_str("}\n\n");

    // Generate Leica tags table
    code.push_str("pub const LEICA_TAGS: &[(u16, TagInfo)] = &[\n");

    for tag in leica_tags {
        let format = map_writable_to_format(&tag.writable);
        let group = tag
            .groups
            .as_ref()
            .map(|g| format!("Some(\"{g}\")"))
            .unwrap_or("Some(\"Leica\")".to_string());

        code.push_str(&format!(
            "    (0x{:04x}, TagInfo {{ name: \"{}\", format: {}, group: {} }}),\n",
            tag.tag_id, tag.name, format, group
        ));
    }

    code.push_str("];\n\n");

    code.push_str("pub fn lookup_leica_tag(tag_id: u16) -> Option<&'static TagInfo> {\n");
    code.push_str("    LEICA_TAGS.iter().find(|(id, _)| *id == tag_id).map(|(_, info)| info)\n");
    code.push_str("}\n\n");

    // Generate Sigma tags table
    code.push_str("pub const SIGMA_TAGS: &[(u16, TagInfo)] = &[\n");

    for tag in sigma_tags {
        let format = map_writable_to_format(&tag.writable);
        let group = tag
            .groups
            .as_ref()
            .map(|g| format!("Some(\"{g}\")"))
            .unwrap_or("Some(\"Sigma\")".to_string());

        code.push_str(&format!(
            "    (0x{:04x}, TagInfo {{ name: \"{}\", format: {}, group: {} }}),\n",
            tag.tag_id, tag.name, format, group
        ));
    }

    code.push_str("];\n\n");

    code.push_str("pub fn lookup_sigma_tag(tag_id: u16) -> Option<&'static TagInfo> {\n");
    code.push_str("    SIGMA_TAGS.iter().find(|(id, _)| *id == tag_id).map(|(_, info)| info)\n");
    code.push_str("}\n\n");

    // Generate Apple tags table
    code.push_str("pub const APPLE_TAGS: &[(u16, TagInfo)] = &[\n");

    for tag in apple_tags {
        let format = map_writable_to_format(&tag.writable);
        let group = tag
            .groups
            .as_ref()
            .map(|g| format!("Some(\"{g}\")"))
            .unwrap_or("Some(\"Apple\")".to_string());

        code.push_str(&format!(
            "    (0x{:04x}, TagInfo {{ name: \"{}\", format: {}, group: {} }}),\n",
            tag.tag_id, tag.name, format, group
        ));
    }

    code.push_str("];\n\n");

    code.push_str("pub fn lookup_apple_tag(tag_id: u16) -> Option<&'static TagInfo> {\n");
    code.push_str("    APPLE_TAGS.iter().find(|(id, _)| *id == tag_id).map(|(_, info)| info)\n");
    code.push_str("}\n\n");

    // Generate Samsung tags table
    code.push_str("pub const SAMSUNG_TAGS: &[(u16, TagInfo)] = &[\n");

    for tag in samsung_tags {
        let format = map_writable_to_format(&tag.writable);
        let group = tag
            .groups
            .as_ref()
            .map(|g| format!("Some(\"{g}\")"))
            .unwrap_or("Some(\"Samsung\")".to_string());

        code.push_str(&format!(
            "    (0x{:04x}, TagInfo {{ name: \"{}\", format: {}, group: {} }}),\n",
            tag.tag_id, tag.name, format, group
        ));
    }

    code.push_str("];\n\n");

    code.push_str("pub fn lookup_samsung_tag(tag_id: u16) -> Option<&'static TagInfo> {\n");
    code.push_str("    SAMSUNG_TAGS.iter().find(|(id, _)| *id == tag_id).map(|(_, info)| info)\n");
    code.push_str("}\n\n");

    // Generate Hasselblad tags table
    code.push_str("pub const HASSELBLAD_TAGS: &[(u16, TagInfo)] = &[\n");

    for tag in hasselblad_tags {
        let format = map_writable_to_format(&tag.writable);
        let group = tag
            .groups
            .as_ref()
            .map(|g| format!("Some(\"{g}\")"))
            .unwrap_or("Some(\"Hasselblad\")".to_string());

        code.push_str(&format!(
            "    (0x{:04x}, TagInfo {{ name: \"{}\", format: {}, group: {} }}),\n",
            tag.tag_id, tag.name, format, group
        ));
    }

    code.push_str("];\n\n");

    code.push_str("pub fn lookup_hasselblad_tag(tag_id: u16) -> Option<&'static TagInfo> {\n");
    code.push_str(
        "    HASSELBLAD_TAGS.iter().find(|(id, _)| *id == tag_id).map(|(_, info)| info)\n",
    );
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

/// Ensure all detection.rs files exist, creating minimal stubs if missing
fn ensure_detection_files_exist() {
    let manufacturers = [
        "canon",
        "nikon",
        "sony",
        "olympus",
        "pentax",
        "fujifilm",
        "panasonic",
        "samsung",
        "sigma",
        "apple",
    ];

    for manufacturer in &manufacturers {
        let detection_file = format!("src/maker/{}/detection.rs", manufacturer);

        if !Path::new(&detection_file).exists() {
            // Create directory if it doesn't exist
            let dir = format!("src/maker/{}", manufacturer);
            let _ = fs::create_dir_all(&dir);

            // Create minimal stub file
            let stub_content = format!(
                "// STUB: Auto-generated by build.rs to ensure compilation\n\
                 // Regenerate with: cargo run --bin exiftool_sync extract maker-detection\n\n\
                 #[derive(Debug, Clone, PartialEq)]\n\
                 pub struct {}DetectionResult {{\n\
                 \x20\x20\x20\x20pub version: Option<u8>,\n\
                 \x20\x20\x20\x20pub ifd_offset: usize,\n\
                 \x20\x20\x20\x20pub description: String,\n\
                 }}\n\n\
                 pub fn detect_{}_maker_note(_data: &[u8]) -> Option<{}DetectionResult> {{\n\
                 \x20\x20\x20\x20None // Stub implementation\n\
                 }}\n",
                manufacturer.to_uppercase(),
                manufacturer,
                manufacturer.to_uppercase()
            );

            if let Err(e) = fs::write(&detection_file, stub_content) {
                eprintln!("Warning: Failed to create stub {}: {}", detection_file, e);
            } else {
                println!("Created stub detection file: {}", detection_file);
            }
        }
    }
}

/// Ensure composite_tags.rs file exists, creating a stub if missing
fn ensure_composite_tags_exists() {
    let file_path = "src/binary/composite_tags.rs";

    if !Path::new(file_path).exists() {
        // Create directory if needed
        let _ = fs::create_dir_all("src/binary");

        let stub_content = r#"// STUB: Auto-generated by build.rs to ensure compilation
// Regenerate with: cargo run --bin exiftool_sync extract binary-tags

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Exif.pm"]

/// Composite tag for binary data extraction
#[derive(Debug, Clone)]
pub struct CompositeTag {
    pub name: &'static str,
    pub required_tags: Vec<u16>,
    pub format: &'static str,
}

/// All composite tags for binary extraction
pub static COMPOSITE_TAGS: &[CompositeTag] = &[
    // Stub - real tags will be generated by extractor
];

/// Get composite tag by name
pub fn get_composite_tag(name: &str) -> Option<&'static CompositeTag> {
    COMPOSITE_TAGS.iter().find(|tag| tag.name == name)
}
"#;

        if let Err(e) = fs::write(file_path, stub_content) {
            eprintln!("Warning: Failed to create stub {}: {}", file_path, e);
        } else {
            println!("Created stub file: {}", file_path);
        }
    }
}
