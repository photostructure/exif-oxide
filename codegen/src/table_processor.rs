//! Table processing logic for code generation
//!
//! This module handles the conversion of extracted tag data into generated format
//! and processes ExtractedTable data structures.

use anyhow::{Context, Result};
use crate::common::{normalize_format, parse_hex_id};
use crate::schemas::{CompositeData, ExtractedData, GeneratedCompositeTag, GeneratedTag};
use crate::file_operations::read_utf8_with_fallback;
use crate::generators::{generate_composite_tag_table, generate_supported_tags, generate_tag_table};
use crate::generators::conversion_refs::generate_conversion_refs;
use std::path::Path;
use std::fs;
use tracing::{debug, warn};

/// Convert extracted tags to generated format
///
/// This function processes both EXIF and GPS tags from the extracted data
/// and converts them to the generated format used by the code generation system.
#[allow(dead_code)]
pub fn convert_tags(data: &ExtractedData) -> Result<Vec<GeneratedTag>> {
    let mut all_tags = Vec::new();

    // Convert EXIF tags
    for tag in &data.tags.exif {
        all_tags.push(GeneratedTag {
            id: parse_hex_id(&tag.id)?,
            name: tag.name.clone(),
            format: normalize_format(&tag.format),
            groups: tag.groups.clone(),
            writable: tag.writable != 0,
            description: tag.description.clone(),
            print_conv_ref: tag.print_conv_ref.clone(),
            value_conv_ref: tag.value_conv_ref.clone(),
            notes: tag.notes.clone(),
        });
    }

    // Convert GPS tags
    for tag in &data.tags.gps {
        all_tags.push(GeneratedTag {
            id: parse_hex_id(&tag.id)?,
            name: tag.name.clone(),
            format: normalize_format(&tag.format),
            groups: tag.groups.clone(),
            writable: tag.writable != 0,
            description: tag.description.clone(),
            print_conv_ref: tag.print_conv_ref.clone(),
            value_conv_ref: tag.value_conv_ref.clone(),
            notes: tag.notes.clone(),
        });
    }

    Ok(all_tags)
}

/// Convert extracted composite tags to generated format
///
/// This function processes composite tag data from the extraction system
/// and converts it to the format used by the code generation system.
#[allow(dead_code)]
pub fn convert_composite_tags_from_data(data: &CompositeData) -> Result<Vec<GeneratedCompositeTag>> {
    Ok(data
        .composite_tags
        .iter()
        .map(|tag| GeneratedCompositeTag {
            name: tag.name.clone(),
            table: tag.table.clone(),
            require: tag.require.clone().unwrap_or_default(),
            desire: tag.desire.clone().unwrap_or_default(),
            print_conv_ref: tag.print_conv_ref.clone(),
            value_conv_ref: tag.value_conv_ref.clone(),
            description: tag.description.clone(),
            writable: tag.writable != 0,
        })
        .collect())
}

/// Process tag tables from extracted data
///
/// This function orchestrates the entire tag processing pipeline,
/// including both regular tags and composite tags.
#[allow(dead_code)]
pub fn process_tag_tables(
    tag_data_path: &str,
    composite_data_path: &str,
    output_dir: &str,
) -> Result<()> {
    use crate::file_operations::{file_exists, read_utf8_with_fallback};
    use crate::generators::{
        generate_composite_tag_table, generate_supported_tags, generate_tag_table,
    };
    use std::path::Path;

    // Process main tag tables
    if file_exists(Path::new(tag_data_path)) {
        debug!("📋 Processing tag tables...");
        let json_data = read_utf8_with_fallback(Path::new(tag_data_path))?;

        let extracted: ExtractedData = serde_json::from_str(&json_data)
            .with_context(|| "Failed to parse tag extraction JSON")?;

        // Convert extracted tags to generated format
        let generated_tags = convert_tags(&extracted)?;

        // Generate code for tag tables
        generate_tag_table(&generated_tags, output_dir)?;

        // Process composite tags if available
        if file_exists(Path::new(composite_data_path)) {
            debug!("🔗 Processing composite tags...");
            let composite_json = read_utf8_with_fallback(Path::new(composite_data_path))?;
            let composite_data: CompositeData = serde_json::from_str(&composite_json)
                .with_context(|| "Failed to parse composite tags JSON")?;

            let generated_composites = convert_composite_tags_from_data(&composite_data)?;
            generate_composite_tag_table(&generated_composites, output_dir)?;
            generate_supported_tags(&generated_tags, &generated_composites, output_dir)?;
        } else {
            // Generate without composite tags
            generate_supported_tags(&generated_tags, &[], output_dir)?;
        }
    } else {
        warn!("Tag data file not found!");
    }

    Ok(())
}

/// Process tag tables from modular extracted files
/// 
/// This function scans the type-specific directories for tag definition and composite tag files
/// organized by source module (e.g., exif_tag_definitions.json, gps_composite_tags.json)
/// and generates the unified tag table code.
#[allow(dead_code)]
pub fn process_tag_tables_modular(extract_dir: &Path, output_dir: &str) -> Result<()> {
    let mut all_tags = Vec::new();
    let mut all_composites = Vec::new();
    let mut all_conversion_refs = crate::schemas::input::ConversionRefs {
        print_conv: Vec::new(),
        value_conv: Vec::new(),
    };

    // Scan for tag definition files in the tag_definitions directory
    let tag_defs_dir = extract_dir.join("tag_definitions");
    if tag_defs_dir.exists() {
        for entry in fs::read_dir(&tag_defs_dir)? {
            let entry = entry?;
            let file_path = entry.path();
            
            if file_path.extension().and_then(|s| s.to_str()) == Some("json") {
                let filename = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                debug!("  📊 Processing {}", filename);
                let json_data = read_utf8_with_fallback(&file_path)?;
                
                // Skip empty files
                if json_data.trim().is_empty() {
                    warn!("    ⚠️  Skipping empty file: {}", filename);
                    continue;
                }
                
                // Parse the modular tag definition format
                let tag_data: serde_json::Value = serde_json::from_str(&json_data)
                    .with_context(|| format!("Failed to parse {filename}"))?;
                
                // Extract tags from the modular format
                if let Some(tags) = tag_data["tags"].as_array() {
                    for tag_val in tags {
                        let tag = extract_tag_from_json(tag_val)?;
                        all_tags.push(tag);
                    }
                }
                
                // Collect conversion references
                if let Some(conv_refs) = tag_data["conversion_refs"].as_object() {
                    if let Some(print_conv) = conv_refs["print_conv"].as_array() {
                        for pc in print_conv {
                            if let Some(pc_str) = pc.as_str() {
                                if !all_conversion_refs.print_conv.contains(&pc_str.to_string()) {
                                    all_conversion_refs.print_conv.push(pc_str.to_string());
                                }
                            }
                        }
                    }
                    if let Some(value_conv) = conv_refs["value_conv"].as_array() {
                        for vc in value_conv {
                            if let Some(vc_str) = vc.as_str() {
                                if !all_conversion_refs.value_conv.contains(&vc_str.to_string()) {
                                    all_conversion_refs.value_conv.push(vc_str.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Scan for composite tag files in the composite_tags directory
    let composite_tags_dir = extract_dir.join("composite_tags");
    if composite_tags_dir.exists() {
        for entry in fs::read_dir(&composite_tags_dir)? {
            let entry = entry?;
            let file_path = entry.path();
            
            if file_path.extension().and_then(|s| s.to_str()) == Some("json") {
                let filename = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                debug!("  🔗 Processing {}", filename);
                let json_data = read_utf8_with_fallback(&file_path)?;
                
                // Skip empty files
                if json_data.trim().is_empty() {
                    warn!("    ⚠️  Skipping empty file: {}", filename);
                    continue;
                }
                
                // Parse the modular composite tag format
                let composite_data: serde_json::Value = serde_json::from_str(&json_data)
                    .with_context(|| format!("Failed to parse {filename}"))?;
                
                // Extract composite tags from the modular format
                if let Some(composites) = composite_data["composite_tags"].as_array() {
                    for comp_val in composites {
                        let composite = extract_composite_from_json(comp_val)?;
                        all_composites.push(composite);
                    }
                }
                
                // Collect conversion references from composite tags too
                if let Some(conv_refs) = composite_data["conversion_refs"].as_object() {
                    if let Some(print_conv) = conv_refs["print_conv"].as_array() {
                        for pc in print_conv {
                            if let Some(pc_str) = pc.as_str() {
                                if !all_conversion_refs.print_conv.contains(&pc_str.to_string()) {
                                    all_conversion_refs.print_conv.push(pc_str.to_string());
                                }
                            }
                        }
                    }
                    if let Some(value_conv) = conv_refs["value_conv"].as_array() {
                        for vc in value_conv {
                            if let Some(vc_str) = vc.as_str() {
                                if !all_conversion_refs.value_conv.contains(&vc_str.to_string()) {
                                    all_conversion_refs.value_conv.push(vc_str.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    debug!("  ✅ Found {} tags and {} composite tags", all_tags.len(), all_composites.len());

    // Generate code if we have any data
    if !all_tags.is_empty() || !all_composites.is_empty() {
        // Generate tag table code
        if !all_tags.is_empty() {
            generate_tag_table(&all_tags, output_dir)?;
        }

        // Generate composite tag code  
        if !all_composites.is_empty() {
            generate_composite_tag_table(&all_composites, output_dir)?;
        }

        // Generate supported tags (unified list)
        generate_supported_tags(&all_tags, &all_composites, output_dir)?;
        
        // Generate conversion references
        generate_conversion_refs(&all_conversion_refs, output_dir)?;
    }

    Ok(())
}

/// Extract a GeneratedTag from JSON value
fn extract_tag_from_json(tag_val: &serde_json::Value) -> Result<GeneratedTag> {
    Ok(GeneratedTag {
        id: parse_hex_id(tag_val["id"].as_str().unwrap_or("0x0"))?,
        name: tag_val["name"].as_str().unwrap_or("Unknown").to_string(),
        format: normalize_format(tag_val["format"].as_str().unwrap_or("string")),
        groups: tag_val["groups"].as_array()
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect())
            .unwrap_or_default(),
        writable: tag_val["writable"].as_u64().unwrap_or(0) != 0,
        description: tag_val["description"].as_str().map(|s| s.to_string()),
        print_conv_ref: tag_val["print_conv_ref"].as_str().map(|s| s.to_string()),
        value_conv_ref: tag_val["value_conv_ref"].as_str().map(|s| s.to_string()),
        notes: tag_val["notes"].as_str().map(|s| s.to_string()),
    })
}

/// Extract a GeneratedCompositeTag from JSON value
fn extract_composite_from_json(comp_val: &serde_json::Value) -> Result<GeneratedCompositeTag> {
    Ok(GeneratedCompositeTag {
        name: comp_val["name"].as_str().unwrap_or("Unknown").to_string(),
        table: comp_val["table"].as_str().unwrap_or("Unknown").to_string(),
        require: comp_val["require"].as_array()
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect())
            .unwrap_or_default(),
        desire: comp_val["desire"].as_array()
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect())
            .unwrap_or_default(),
        description: comp_val["description"].as_str().map(|s| s.to_string()),
        writable: comp_val["writable"].as_u64().unwrap_or(0) != 0,
        print_conv_ref: comp_val["print_conv_ref"].as_str().map(|s| s.to_string()),
        value_conv_ref: comp_val["value_conv_ref"].as_str().map(|s| s.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::{ExtractedTag, input::{ConversionRefs, TagGroups}};
    
    #[test]
    fn test_convert_tags_exif() {
        let extracted_data = ExtractedData {
            extracted_at: "test".to_string(),
            exiftool_version: "test".to_string(),
            filter_criteria: "test".to_string(),
            stats: crate::schemas::input::TagStats {
                exif_count: 1,
                gps_count: 0,
                total_tags: 1,
            },
            tags: TagGroups {
                exif: vec![ExtractedTag {
                    id: "0x0100".to_string(),
                    name: "ImageWidth".to_string(),
                    format: "int32u".to_string(),
                    groups: vec!["IFD0".to_string(), "Image".to_string()],
                    writable: 1,
                    description: Some("Image width".to_string()),
                    print_conv_ref: None,
                    value_conv_ref: None,
                    notes: None,
                    frequency: None,
                    mainstream: None,
                }],
                gps: vec![],
            },
            conversion_refs: ConversionRefs {
                print_conv: vec![],
                value_conv: vec![],
            },
        };

        let generated_tags = convert_tags(&extracted_data).unwrap();
        
        assert_eq!(generated_tags.len(), 1);
        assert_eq!(generated_tags[0].name, "ImageWidth");
        assert_eq!(generated_tags[0].id, 0x0100);
        assert!(generated_tags[0].writable);
    }
    
    #[test]
    fn test_convert_tags_gps() {
        let extracted_data = ExtractedData {
            extracted_at: "test".to_string(),
            exiftool_version: "test".to_string(),
            filter_criteria: "test".to_string(),
            stats: crate::schemas::input::TagStats {
                exif_count: 0,
                gps_count: 1,
                total_tags: 1,
            },
            tags: TagGroups {
                exif: vec![],
                gps: vec![ExtractedTag {
                    id: "0x0001".to_string(),
                    name: "GPSLatitudeRef".to_string(),
                    format: "string".to_string(),
                    groups: vec!["GPS".to_string()],
                    writable: 1,
                    description: Some("GPS latitude reference".to_string()),
                    print_conv_ref: None,
                    value_conv_ref: None,
                    notes: None,
                    frequency: None,
                    mainstream: None,
                }],
            },
            conversion_refs: ConversionRefs {
                print_conv: vec![],
                value_conv: vec![],
            },
        };

        let generated_tags = convert_tags(&extracted_data).unwrap();
        
        assert_eq!(generated_tags.len(), 1);
        assert_eq!(generated_tags[0].name, "GPSLatitudeRef");
        assert_eq!(generated_tags[0].id, 0x0001);
    }
}

/// Process only composite tags from modular extracted files
pub fn process_composite_tags_only(extract_dir: &Path, output_dir: &str) -> Result<()> {
    let mut all_composites = Vec::new();
    let mut all_conversion_refs = crate::schemas::input::ConversionRefs {
        print_conv: Vec::new(),
        value_conv: Vec::new(),
    };
    
    // Scan for composite tag files in the composite_tags directory
    let composite_tags_dir = extract_dir.join("composite_tags");
    if composite_tags_dir.exists() {
        for entry in fs::read_dir(&composite_tags_dir)? {
            let entry = entry?;
            let file_path = entry.path();
            
            if file_path.extension().and_then(|s| s.to_str()) == Some("json") {
                let filename = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                debug!("  🔗 Processing {}", filename);
                let json_data = read_utf8_with_fallback(&file_path)?;
                
                // Skip empty files
                if json_data.trim().is_empty() {
                    warn!("    ⚠️  Skipping empty file: {}", filename);
                    continue;
                }
                
                // Parse the modular composite tag format
                let composite_data: serde_json::Value = serde_json::from_str(&json_data)
                    .with_context(|| format!("Failed to parse {filename}"))?;
                
                // Extract composite tags from the modular format
                if let Some(composites) = composite_data["composite_tags"].as_array() {
                    for comp_val in composites {
                        let composite = extract_composite_from_json(comp_val)?;
                        all_composites.push(composite);
                    }
                }
                
                // Collect conversion references from composite tags too
                if let Some(conv_refs) = composite_data["conversion_refs"].as_object() {
                    if let Some(print_conv) = conv_refs["print_conv"].as_array() {
                        for pc in print_conv {
                            if let Some(pc_str) = pc.as_str() {
                                if !all_conversion_refs.print_conv.contains(&pc_str.to_string()) {
                                    all_conversion_refs.print_conv.push(pc_str.to_string());
                                }
                            }
                        }
                    }
                    if let Some(value_conv) = conv_refs["value_conv"].as_array() {
                        for vc in value_conv {
                            if let Some(vc_str) = vc.as_str() {
                                if !all_conversion_refs.value_conv.contains(&vc_str.to_string()) {
                                    all_conversion_refs.value_conv.push(vc_str.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Sort for deterministic output
    all_composites.sort_by(|a, b| a.name.cmp(&b.name));
    all_conversion_refs.print_conv.sort();
    all_conversion_refs.value_conv.sort();
    
    // Generate code
    if !all_composites.is_empty() {
        debug!("  🔗 Generating composite tag table...");
        generate_composite_tag_table(&all_composites, output_dir)?;
    }
    
    // Generate conversion references if we have any
    if !all_conversion_refs.print_conv.is_empty() || !all_conversion_refs.value_conv.is_empty() {
        debug!("  🔄 Generating conversion references...");
        generate_conversion_refs(&all_conversion_refs, output_dir)?;
    }
    
    // Generate supported tags summary (only for composite tags now)
    debug!("  📊 Generating supported tags summary...");
    generate_supported_tags(&[], &all_composites, output_dir)?;
    
    Ok(())
}