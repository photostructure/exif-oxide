//! Canon-specific BinaryDataProcessor implementations
//!
//! These processors implement the BinaryDataProcessor trait for Canon camera data
//! while delegating to the existing Canon implementation modules. This maintains
//! the "Trust ExifTool" principle by reusing proven processing logic.
//!
//! ## ExifTool Reference
//!
//! Canon.pm ProcessBinaryData, ProcessSerialData, and related processing functions

use super::super::{
    BinaryDataProcessor, ProcessorCapability, ProcessorContext, ProcessorMetadata, ProcessorResult,
};
use crate::implementations::canon;
use crate::types::{Result, TagValue};
use std::collections::HashMap;
use tracing::debug;

/// Canon Serial Data processor using existing implementation
///
/// Processes Canon camera serial data using the existing `canon::binary_data`
/// implementation. This processor is selected for Canon cameras with table
/// names containing "SerialData".
///
/// ## ExifTool Reference
///
/// Canon.pm ProcessSerialData function and related binary data processing
pub struct CanonSerialDataProcessor;

impl BinaryDataProcessor for CanonSerialDataProcessor {
    fn can_process(&self, context: &ProcessorContext) -> ProcessorCapability {
        // Perfect match if Canon manufacturer and SerialData table
        if context.is_manufacturer("Canon") && context.table_name.contains("SerialData") {
            return ProcessorCapability::Perfect;
        }

        // Good match for Canon with any binary data table
        if context.is_manufacturer("Canon") && context.table_name.contains("Binary") {
            return ProcessorCapability::Good;
        }

        // Fallback for Canon cameras (can process but not optimal)
        if context.is_manufacturer("Canon") {
            return ProcessorCapability::Fallback;
        }

        ProcessorCapability::Incompatible
    }

    fn process_data(&self, data: &[u8], context: &ProcessorContext) -> Result<ProcessorResult> {
        debug!(
            "Processing Canon SerialData with {} bytes for table: {}",
            data.len(),
            context.table_name
        );

        let mut result = ProcessorResult::new();

        // Delegate to existing Canon binary data processing
        // This extracts camera settings and serial data using proven ExifTool logic
        let byte_order = context
            .byte_order
            .unwrap_or(crate::tiff_types::ByteOrder::LittleEndian);
        let camera_settings = canon::binary_data::extract_camera_settings(
            data,
            context.data_offset,
            data.len(),
            byte_order,
        );

        match camera_settings {
            Ok(tags) => {
                // Convert HashMap<String, TagValue> to ProcessorResult format
                for (tag_name, tag_value) in tags {
                    result.add_tag(tag_name, tag_value);
                }

                debug!(
                    "Canon SerialData processor extracted {} tags",
                    result.extracted_tags.len()
                );
            }
            Err(e) => {
                let warning = format!("Canon SerialData processing failed: {e}");
                result.add_warning(warning);
                debug!("Canon SerialData processing error: {}", e);
            }
        }

        Ok(result)
    }

    fn get_metadata(&self) -> ProcessorMetadata {
        ProcessorMetadata::new(
            "Canon Serial Data Processor".to_string(),
            "Processes Canon camera serial data using existing implementation".to_string(),
        )
        .with_manufacturer("Canon".to_string())
        .with_required_context("manufacturer".to_string())
        .with_example_condition(
            "manufacturer == 'Canon' && table.contains('SerialData')".to_string(),
        )
    }
}

/// Canon Camera Settings processor for binary data tables
///
/// Processes Canon CameraSettings binary data using the existing implementation.
/// This handles the complex binary data formats used by Canon cameras.
///
/// ## ExifTool Reference
///
/// Canon.pm CameraSettings table processing with ProcessBinaryData
pub struct CanonCameraSettingsProcessor;

impl BinaryDataProcessor for CanonCameraSettingsProcessor {
    fn can_process(&self, context: &ProcessorContext) -> ProcessorCapability {
        // Perfect match for Canon CameraSettings table
        if context.is_manufacturer("Canon") && context.table_name.contains("CameraSettings") {
            return ProcessorCapability::Perfect;
        }

        // Good match for Canon binary data processing
        if context.is_manufacturer("Canon")
            && (context.table_name.contains("Canon") || context.table_name.contains("Binary"))
        {
            return ProcessorCapability::Good;
        }

        // Fallback for Canon cameras
        if context.is_manufacturer("Canon") {
            return ProcessorCapability::Fallback;
        }

        ProcessorCapability::Incompatible
    }

    fn process_data(&self, data: &[u8], context: &ProcessorContext) -> Result<ProcessorResult> {
        debug!(
            "Processing Canon CameraSettings with {} bytes for table: {}",
            data.len(),
            context.table_name
        );

        let mut result = ProcessorResult::new();

        // Create the Canon CameraSettings table using existing implementation
        let table = canon::binary_data::create_canon_camera_settings_table();

        // Use existing binary data extraction logic
        // This mimics what an ExifReader would do with the table
        let extracted_tags = extract_binary_data_using_table(data, context, &table)?;

        for (tag_name, tag_value) in extracted_tags {
            result.add_tag(tag_name, tag_value);
        }

        if result.extracted_tags.is_empty() {
            result.add_warning("No Canon CameraSettings tags extracted".to_string());
        } else {
            debug!(
                "Canon CameraSettings processor extracted {} tags",
                result.extracted_tags.len()
            );
        }

        Ok(result)
    }

    fn get_metadata(&self) -> ProcessorMetadata {
        ProcessorMetadata::new(
            "Canon Camera Settings Processor".to_string(),
            "Processes Canon CameraSettings binary data table".to_string(),
        )
        .with_manufacturer("Canon".to_string())
        .with_required_context("manufacturer".to_string())
        .with_example_condition(
            "manufacturer == 'Canon' && table.contains('CameraSettings')".to_string(),
        )
    }
}

/// Enhanced Canon processor for newer models with additional data
///
/// Handles Canon R5, R6, and other newer models that have enhanced
/// binary data formats and additional metadata fields.
///
/// ## ExifTool Reference
///
/// Canon.pm model-specific processing for newer camera bodies
pub struct CanonSerialDataMkIIProcessor;

impl BinaryDataProcessor for CanonSerialDataMkIIProcessor {
    fn can_process(&self, context: &ProcessorContext) -> ProcessorCapability {
        // Perfect match for newer Canon models with enhanced data
        if context.is_manufacturer("Canon") {
            if let Some(model) = &context.model {
                if (model.contains("EOS R5")
                    || model.contains("EOS R6")
                    || model.contains("EOS R3"))
                    && context.table_name.contains("SerialData")
                {
                    return ProcessorCapability::Perfect;
                }
            }
        }

        ProcessorCapability::Incompatible
    }

    fn process_data(&self, data: &[u8], context: &ProcessorContext) -> Result<ProcessorResult> {
        debug!(
            "Processing Canon MkII SerialData with {} bytes for model: {:?}",
            data.len(),
            context.model
        );

        let mut result = ProcessorResult::new();

        // Enhanced serial data processing for newer Canon models
        if data.len() < 12 {
            result.add_warning("Canon MkII serial data too short".to_string());
            return Ok(result);
        }

        // Extract extended serial information (following ExifTool patterns)
        // ExifTool: Enhanced data extraction for newer models

        // Serial number (first 8 bytes)
        let serial_bytes = &data[0..8];
        let serial_number = String::from_utf8_lossy(serial_bytes)
            .trim_end_matches('\0')
            .to_string();
        if !serial_number.is_empty() {
            result.add_tag("SerialNumber".to_string(), TagValue::String(serial_number));
        }

        // Firmware version (bytes 8-11) - enhanced format for newer models
        if data.len() >= 12 {
            let firmware_major = data[8];
            let firmware_minor = data[9];
            let firmware_patch = u16::from_le_bytes([data[10], data[11]]);

            let firmware_version = format!("{firmware_major}.{firmware_minor}.{firmware_patch}");
            result.add_tag(
                "FirmwareVersion".to_string(),
                TagValue::String(firmware_version),
            );
        }

        // Additional model-specific data extraction could go here
        if let Some(model) = &context.model {
            result.add_tag("CameraModel".to_string(), TagValue::String(model.clone()));
        }

        debug!(
            "Canon MkII SerialData processor extracted {} tags",
            result.extracted_tags.len()
        );

        Ok(result)
    }

    fn get_metadata(&self) -> ProcessorMetadata {
        ProcessorMetadata::new(
            "Canon Serial Data MkII".to_string(),
            "Enhanced serial data processing for newer Canon models (R5, R6, R3)".to_string(),
        )
        .with_manufacturer("Canon".to_string())
        .with_required_context("manufacturer".to_string())
        .with_required_context("model".to_string())
        .with_example_condition(
            "manufacturer == 'Canon' && (model.contains('EOS R5') || model.contains('EOS R6'))"
                .to_string(),
        )
    }
}

/// Helper function to extract binary data using a Canon table definition
///
/// This function mimics what ExifReader.process_binary_data would do but works
/// independently for processor implementations.
fn extract_binary_data_using_table(
    data: &[u8],
    context: &ProcessorContext,
    table: &crate::types::BinaryDataTable,
) -> Result<HashMap<String, TagValue>> {
    let mut extracted = HashMap::new();

    // Process each defined tag in the table
    for (&index, tag_def) in &table.tags {
        // Calculate position based on FIRST_ENTRY
        let first_entry = table.first_entry.unwrap_or(0);
        if index < first_entry {
            continue;
        }

        let entry_offset = (index - first_entry) as usize * table.default_format.byte_size();
        if entry_offset + table.default_format.byte_size() > data.len() {
            debug!("Tag {} at index {} beyond data bounds", tag_def.name, index);
            continue;
        }

        let data_offset = context.data_offset + entry_offset;

        // Extract the raw value using the table's default format
        let format = tag_def.format.unwrap_or(table.default_format);
        let byte_order = context
            .byte_order
            .unwrap_or(crate::tiff_types::ByteOrder::LittleEndian);
        let raw_value = extract_binary_value_direct(data, data_offset, format, byte_order)?;

        // Apply PrintConv if available
        let final_value = if let Some(print_conv) = &tag_def.print_conv {
            apply_print_conv(&raw_value, print_conv)
        } else {
            raw_value
        };

        // Store with group prefix like ExifTool
        let group_0 = table
            .groups
            .get(&0)
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string());

        let tag_name = format!("{}:{}", group_0, tag_def.name);

        debug!(
            "Extracted Canon binary tag {} (index {}) = {:?}",
            tag_def.name, index, final_value
        );

        extracted.insert(tag_name, final_value);
    }

    Ok(extracted)
}

/// Extract binary value directly from data
///
/// This is a simplified version of the binary value extraction that works
/// without requiring a full ExifReader instance.
fn extract_binary_value_direct(
    data: &[u8],
    offset: usize,
    format: crate::types::BinaryDataFormat,
    byte_order: crate::tiff_types::ByteOrder,
) -> Result<TagValue> {
    use crate::types::{BinaryDataFormat, ExifError};

    match format {
        BinaryDataFormat::Int8u => {
            if offset >= data.len() {
                return Err(ExifError::ParseError(
                    "Offset beyond data bounds".to_string(),
                ));
            }
            Ok(TagValue::U8(data[offset]))
        }
        BinaryDataFormat::Int16s => {
            if offset + 2 > data.len() {
                return Err(ExifError::ParseError(
                    "Offset beyond data bounds for int16s".to_string(),
                ));
            }
            let value = byte_order.read_u16(data, offset)? as i16;
            Ok(TagValue::I16(value))
        }
        BinaryDataFormat::Int16u => {
            if offset + 2 > data.len() {
                return Err(ExifError::ParseError(
                    "Offset beyond data bounds for int16u".to_string(),
                ));
            }
            let value = byte_order.read_u16(data, offset)?;
            Ok(TagValue::U16(value))
        }
        BinaryDataFormat::Int32u => {
            if offset + 4 > data.len() {
                return Err(ExifError::ParseError(
                    "Offset beyond data bounds for int32u".to_string(),
                ));
            }
            let value = byte_order.read_u32(data, offset)?;
            Ok(TagValue::U32(value))
        }
        BinaryDataFormat::String => {
            if offset >= data.len() {
                return Err(ExifError::ParseError(
                    "Offset beyond data bounds for string".to_string(),
                ));
            }

            let mut end = offset;
            while end < data.len() && data[end] != 0 {
                end += 1;
            }

            let string_bytes = &data[offset..end];
            let string_value = String::from_utf8_lossy(string_bytes).to_string();
            Ok(TagValue::String(string_value))
        }
        _ => Err(ExifError::ParseError(format!(
            "Binary format {format:?} not yet implemented"
        ))),
    }
}

/// Apply PrintConv lookup to a raw value
fn apply_print_conv(raw_value: &TagValue, print_conv: &HashMap<u32, String>) -> TagValue {
    match raw_value {
        TagValue::I16(val) => {
            if let Some(converted) = print_conv.get(&(*val as u32)) {
                TagValue::String(converted.clone())
            } else {
                raw_value.clone()
            }
        }
        TagValue::U16(val) => {
            if let Some(converted) = print_conv.get(&(*val as u32)) {
                TagValue::String(converted.clone())
            } else {
                raw_value.clone()
            }
        }
        TagValue::U32(val) => {
            if let Some(converted) = print_conv.get(val) {
                TagValue::String(converted.clone())
            } else {
                raw_value.clone()
            }
        }
        _ => raw_value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::FileFormat;
    use crate::processor_registry::ProcessorContext;

    #[test]
    fn test_canon_serial_data_processor_capability() {
        let processor = CanonSerialDataProcessor;

        // Perfect match for Canon SerialData
        let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::SerialData".to_string())
            .with_manufacturer("Canon".to_string());

        assert_eq!(
            processor.can_process(&context),
            ProcessorCapability::Perfect
        );

        // Incompatible for non-Canon
        let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::SerialData".to_string())
            .with_manufacturer("Nikon".to_string());

        assert_eq!(
            processor.can_process(&context),
            ProcessorCapability::Incompatible
        );
    }

    #[test]
    fn test_canon_mk2_processor_capability() {
        let processor = CanonSerialDataMkIIProcessor;

        // Perfect match for Canon R5
        let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::SerialData".to_string())
            .with_manufacturer("Canon".to_string())
            .with_model("Canon EOS R5".to_string());

        assert_eq!(
            processor.can_process(&context),
            ProcessorCapability::Perfect
        );

        // Incompatible for older Canon models
        let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::SerialData".to_string())
            .with_manufacturer("Canon".to_string())
            .with_model("Canon EOS 5D Mark IV".to_string());

        assert_eq!(
            processor.can_process(&context),
            ProcessorCapability::Incompatible
        );
    }

    #[test]
    fn test_canon_processor_metadata() {
        let processor = CanonSerialDataProcessor;
        let metadata = processor.get_metadata();

        assert_eq!(metadata.name, "Canon Serial Data Processor");
        assert!(metadata
            .supported_manufacturers
            .contains(&"Canon".to_string()));
        assert!(metadata
            .required_context
            .contains(&"manufacturer".to_string()));
    }
}
