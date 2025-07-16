//! Table processing logic for code generation
//!
//! This module handles the conversion of extracted tag data into generated format
//! and processes ExtractedTable data structures.

use anyhow::{Context, Result};
use crate::common::{normalize_format, parse_hex_id};
use crate::schemas::{CompositeData, ExtractedData, GeneratedCompositeTag, GeneratedTag};

/// Convert extracted tags to generated format
///
/// This function processes both EXIF and GPS tags from the extracted data
/// and converts them to the generated format used by the code generation system.
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
        println!("\n📋 Processing tag tables...");
        let json_data = read_utf8_with_fallback(Path::new(tag_data_path))?;

        let extracted: ExtractedData = serde_json::from_str(&json_data)
            .with_context(|| "Failed to parse tag extraction JSON")?;

        // Convert extracted tags to generated format
        let generated_tags = convert_tags(&extracted)?;

        // Generate code for tag tables
        generate_tag_table(&generated_tags, output_dir)?;

        // Process composite tags if available
        if file_exists(Path::new(composite_data_path)) {
            println!("\n🔗 Processing composite tags...");
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
        println!("Tag data file not found!");
    }

    Ok(())
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
        assert_eq!(generated_tags[0].writable, true);
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