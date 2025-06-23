//! Parse ExifTool tag tables and display extracted tag information
//!
//! This tool demonstrates how we parse ExifTool's Perl modules to extract
//! tag definitions for code generation. It's useful for debugging the parsing
//! logic and understanding what tags are available.

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Exif.pm"]

use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Represents a parsed tag definition from ExifTool
#[derive(Debug)]
struct TagDef {
    tag_id: u16,
    name: String,
    writable: Option<String>,
    groups: Option<String>,
    #[allow(dead_code)] // We collect this but don't display it yet
    notes: Option<String>,
}

fn main() {
    // Allow specifying a different ExifTool path via command line
    let args: Vec<String> = std::env::args().collect();
    let exif_pm_path = if args.len() > 1 {
        Path::new(&args[1])
    } else {
        Path::new("exiftool/lib/Image/ExifTool/Exif.pm")
    };

    if !exif_pm_path.exists() {
        eprintln!(
            "Error: ExifTool Exif.pm not found at: {}",
            exif_pm_path.display()
        );
        if args.len() == 1 {
            eprintln!("Make sure you've cloned the ExifTool repository to ./exiftool/");
            eprintln!("Or provide the path as an argument: cargo run --bin parse_exiftool_tags /path/to/Exif.pm");
        }
        std::process::exit(1);
    }

    let content = fs::read_to_string(exif_pm_path).expect("Failed to read Exif.pm");
    let tags = parse_exif_tags(&content);

    println!(
        "Parsed {} tag definitions from ExifTool's Exif.pm",
        tags.len()
    );
    println!();

    // Group tags by category for better display
    let mut by_group: HashMap<String, Vec<&TagDef>> = HashMap::new();
    for tag in &tags {
        let group = tag.groups.as_deref().unwrap_or("General");
        by_group.entry(group.to_string()).or_default().push(tag);
    }

    // Collect statistics before consuming the HashMap
    let group_count = by_group.len();

    // Display tags organized by group
    for (group, mut group_tags) in by_group {
        println!("=== {} Tags ({}) ===", group, group_tags.len());

        // Sort by tag ID for consistent output
        group_tags.sort_by_key(|t| t.tag_id);

        for tag in group_tags.iter().take(10) {
            // Show first 10 in each group
            println!(
                "  0x{:04X} - {:<25} Format: {}",
                tag.tag_id,
                tag.name,
                tag.writable.as_deref().unwrap_or("unknown")
            );
        }

        if group_tags.len() > 10 {
            println!("  ... and {} more", group_tags.len() - 10);
        }
        println!();
    }

    // Show some statistics
    println!("Tag Statistics:");
    println!("  Total tags: {}", tags.len());
    println!("  Groups: {}", group_count);

    // Count format types
    let mut format_counts: HashMap<String, usize> = HashMap::new();
    for tag in &tags {
        let format = tag.writable.as_deref().unwrap_or("none");
        *format_counts.entry(format.to_string()).or_default() += 1;
    }

    println!("\nFormat Types:");
    let mut formats: Vec<_> = format_counts.into_iter().collect();
    formats.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

    for (format, count) in formats.iter().take(10) {
        println!("  {:<20} {}", format, count);
    }
}

/// Parse EXIF tag definitions from ExifTool's Perl source
fn parse_exif_tags(content: &str) -> Vec<TagDef> {
    let mut tags = Vec::new();

    // Find the Main table start
    let main_start = match content.find("%Image::ExifTool::Exif::Main = (") {
        Some(pos) => pos,
        None => {
            eprintln!("Warning: Could not find Main table in Exif.pm");
            return tags;
        }
    };

    // Regular expressions for parsing tag definitions
    let tag_re = Regex::new(r"(?s)(0x[0-9a-fA-F]+)\s*=>\s*\{([^}]+)\}").unwrap();
    let name_re = Regex::new(r"Name\s*=>\s*'([^']+)'").unwrap();
    let writable_re = Regex::new(r"Writable\s*=>\s*'([^']+)'").unwrap();
    let groups_re = Regex::new(r"Groups\s*=>\s*\{[^}]*2\s*=>\s*'([^']+)'").unwrap();
    let notes_re = Regex::new(r"Notes\s*=>\s*'([^']+)'").unwrap();

    // Also match simple string tags like: 0x10f => 'Make',
    let simple_tag_re = Regex::new(r"(0x[0-9a-fA-F]+)\s*=>\s*'([^']+)',").unwrap();

    // Process a reasonable portion of the file
    let search_content = &content[main_start..]
        .chars()
        .take(100000)
        .collect::<String>();

    // First, collect complex tag definitions
    for cap in tag_re.captures_iter(search_content) {
        let tag_hex = &cap[1];
        let tag_content = &cap[2];

        // Skip overly complex conditional tags
        if tag_content.contains("Condition =>") && tag_content.contains("$$") {
            continue;
        }

        // Parse the tag ID
        let tag_id = match u16::from_str_radix(&tag_hex[2..], 16) {
            Ok(id) => id,
            Err(_) => continue,
        };

        // Extract the name (required)
        let name = match name_re.captures(tag_content) {
            Some(cap) => cap[1].to_string(),
            None => continue, // Skip tags without names
        };

        // Extract optional fields
        let writable = writable_re
            .captures(tag_content)
            .map(|cap| cap[1].to_string());
        let groups = groups_re
            .captures(tag_content)
            .map(|cap| cap[1].to_string());
        let notes = notes_re.captures(tag_content).map(|cap| cap[1].to_string());

        tags.push(TagDef {
            tag_id,
            name,
            writable,
            groups,
            notes,
        });
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

    tags
}
