//! FujiFilm-specific BinaryDataProcessor implementations
//!
//! These processors implement the BinaryDataProcessor trait for FujiFilm camera data
//! using generated ProcessBinaryData tables instead of hardcoded logic. This demonstrates
//! the new table-driven approach that replaces manual implementations.
//!
//! ## ExifTool Reference
//!
//! FujiFilm.pm ProcessBinaryData tables and related processing functions

use super::super::{
    BinaryDataProcessor, ProcessorCapability, ProcessorContext, ProcessorMetadata, ProcessorResult,
};
// use crate::generated::fuji_film::ffmv_binary_data::FujiFilmFFMVTable; // TODO: Generate FujiFilm binary data tables
use crate::types::{Result, TagValue};
use tracing::debug;

/// FujiFilm FFMV (Movie) Data processor using generated ProcessBinaryData table
///
/// Processes FujiFilm movie stream data using the generated `FujiFilmFFMVTable`
/// from ExifTool's ProcessBinaryData extraction. This processor demonstrates
/// the new table-driven approach that eliminates hardcoded offset mapping.
///
/// ## ExifTool Reference
///
/// FujiFilm.pm FFMV ProcessBinaryData table
/// TODO P07: Disabled until ffmv_binary_data is generated
pub struct FujiFilmFFMVProcessor {
    // table: crate::generated::fuji_film::ffmv_binary_data::FujiFilmFFMVTable,
}

impl FujiFilmFFMVProcessor {
    pub fn new() -> Self {
        Self {
            // table: crate::generated::fuji_film::ffmv_binary_data::FujiFilmFFMVTable::new(),
        }
    }
}

impl BinaryDataProcessor for FujiFilmFFMVProcessor {
    fn can_process(&self, context: &ProcessorContext) -> ProcessorCapability {
        // Perfect match if FujiFilm manufacturer and FFMV table
        if context.is_manufacturer("FUJIFILM") && context.table_name.contains("FFMV") {
            return ProcessorCapability::Perfect;
        }

        // Good match for FujiFilm with any movie-related table
        if context.is_manufacturer("FUJIFILM")
            && (context.table_name.contains("Movie") || context.table_name.contains("Stream"))
        {
            return ProcessorCapability::Good;
        }

        // Only compatible with FujiFilm-specific tables
        ProcessorCapability::Incompatible
    }

    fn process_data(&self, data: &[u8], context: &ProcessorContext) -> Result<ProcessorResult> {
        debug!(
            "Processing FujiFilm FFMV data with {} bytes for table: {}",
            data.len(),
            context.table_name
        );

        let mut result = ProcessorResult::new();

        // TODO P07: Use generated table to process binary data when ffmv_binary_data is available
        // This demonstrates the new table-driven approach from ProcessBinaryData extraction
        // let _first_entry = self.table.first_entry as usize;

        // Process data using generated table offsets and formats
        for offset in 0..data.len() {
            let table_offset = offset as u16;

            // TODO P07: Look up tag name and format from generated table when ffmv_binary_data is available
            // if let Some(tag_name) = self.table.get_tag_name(table_offset) {
            //     if let Some(format) = self.table.get_format(table_offset) {
            //         // Extract value based on format specification
            //         if let Some(tag_value) = extract_value_by_format(data, offset, format) {
            //             result.add_tag(tag_name.to_string(), tag_value);
            //             debug!(
            //                 "Extracted tag {} at offset {} with format {}",
            //                 tag_name, offset, format
            //             );
            //         }
            //     }
            // }
        }

        if result.extracted_tags.is_empty() {
            let warning = format!(
                "No tags extracted from FujiFilm FFMV data (table: {}, {} bytes)",
                context.table_name,
                data.len()
            );
            result.add_warning(warning);
        } else {
            debug!(
                "FujiFilm FFMV processor extracted {} tags using generated table",
                result.extracted_tags.len()
            );
        }

        Ok(result)
    }

    fn get_metadata(&self) -> ProcessorMetadata {
        ProcessorMetadata::new(
            "FujiFilm FFMV Processor".to_string(),
            "Processes FujiFilm movie data using generated ProcessBinaryData table".to_string(),
        )
        .with_manufacturer("FUJIFILM".to_string())
        .with_required_context("manufacturer".to_string())
        .with_example_condition("manufacturer == 'FUJIFILM' && table.contains('FFMV')".to_string())
    }
}

impl Default for FujiFilmFFMVProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract value from binary data based on format specification
/// This implements ExifTool's format parsing logic for generated tables
fn extract_value_by_format(data: &[u8], offset: usize, format: &str) -> Option<TagValue> {
    // Parse format specification (e.g., "string[34]", "int32u", "int16u")
    if format.starts_with("string[") {
        // Extract string length from format like "string[34]"
        if let Some(len_str) = format
            .strip_prefix("string[")
            .and_then(|s| s.strip_suffix("]"))
        {
            if let Ok(length) = len_str.parse::<usize>() {
                if offset + length <= data.len() {
                    let string_bytes = &data[offset..offset + length];
                    // Find null terminator or use full length
                    let end = string_bytes.iter().position(|&b| b == 0).unwrap_or(length);
                    let string_value = String::from_utf8_lossy(&string_bytes[..end])
                        .trim()
                        .to_string();
                    if !string_value.is_empty() {
                        return Some(TagValue::String(string_value));
                    }
                }
            }
        }
    } else if format == "int32u" {
        // 32-bit unsigned integer
        if offset + 4 <= data.len() {
            let bytes: [u8; 4] = data[offset..offset + 4].try_into().ok()?;
            let value = u32::from_le_bytes(bytes); // Assume little endian for FujiFilm
            return Some(TagValue::U32(value));
        }
    } else if format == "int16u" {
        // 16-bit unsigned integer
        if offset + 2 <= data.len() {
            let bytes: [u8; 2] = data[offset..offset + 2].try_into().ok()?;
            let value = u16::from_le_bytes(bytes); // Assume little endian for FujiFilm
            return Some(TagValue::U16(value));
        }
    }

    None
}
