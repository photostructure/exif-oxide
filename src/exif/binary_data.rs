//! Variable ProcessBinaryData implementation for EXIF module
//!
//! This module implements ExifTool's variable-length ProcessBinaryData functionality
//! with DataMember dependencies and format expression evaluation.
//!
//! **Trust ExifTool**: This code translates ExifTool's ProcessBinaryData verbatim,
//! including the two-phase processing and $val{} expression evaluation.
//!
//! Primary ExifTool References:
//! - lib/Image/ExifTool.pm:9750+ ProcessBinaryData function
//! - lib/Image/ExifTool.pm:9850-9856 format expression evaluation

use crate::tiff_types::ByteOrder;
use crate::types::{
    BinaryDataFormat, DataMemberValue, ExifError, ExpressionEvaluator, ResolvedFormat, Result,
    TagSourceInfo, TagValue,
};
use tracing::debug;

use super::ExifReader;

impl ExifReader {
    /// Process binary data with variable-length formats and DataMember dependencies
    /// ExifTool: ProcessBinaryData with two-phase processing for DataMember tags
    /// Reference: third-party/exiftool/lib/Image/ExifTool.pm:9750+ ProcessBinaryData function
    /// Milestone 12: Variable ProcessBinaryData implementation
    pub fn process_binary_data_with_dependencies(
        &mut self,
        data: &[u8],
        offset: usize,
        size: usize,
        table: &crate::types::BinaryDataTable,
    ) -> Result<()> {
        debug!(
            "Processing binary data with dependencies: offset={:#x}, size={}, format={:?}",
            offset, size, table.default_format
        );

        // Build dependency order if not already analyzed
        let mut table = table.clone();
        if table.dependency_order.is_empty() {
            table.analyze_dependencies();
        }

        // Create expression evaluator with current DataMember context
        let val_hash = std::collections::HashMap::new();
        let mut evaluator = ExpressionEvaluator::new(val_hash, &self.data_members);

        // Track cumulative offset for variable-length entries
        // ExifTool: ProcessBinaryData processes entries sequentially, accounting for variable sizes
        let mut cumulative_offset = 0;
        let first_entry = table.first_entry.unwrap_or(0);

        // Process tags in dependency order
        for &index in &table.dependency_order {
            if let Some(tag_def) = table.tags.get(&index) {
                if index < first_entry {
                    continue;
                }

                // Calculate current data position
                let data_offset = offset + cumulative_offset;

                debug!(
                    "Processing tag {} at index {}: offset={:#x}, cumulative_offset={}",
                    tag_def.name, index, data_offset, cumulative_offset
                );

                // Get format specification and resolve expressions if needed
                let format_spec = table
                    .get_tag_format_spec(index)
                    .unwrap_or(crate::types::FormatSpec::Fixed(table.default_format));

                let resolved_format = if format_spec.needs_evaluation() {
                    match evaluator.resolve_format(&format_spec) {
                        Ok(resolved) => resolved,
                        Err(e) => {
                            debug!("Failed to resolve format for tag {}: {}", tag_def.name, e);
                            continue;
                        }
                    }
                } else {
                    match format_spec {
                        crate::types::FormatSpec::Fixed(format) => {
                            crate::types::ResolvedFormat::Single(format)
                        }
                        _ => {
                            debug!(
                                "Unexpected format spec that doesn't need evaluation: {:?}",
                                format_spec
                            );
                            continue;
                        }
                    }
                };

                // Extract value based on resolved format
                let raw_value =
                    match self.extract_with_resolved_format(data, data_offset, &resolved_format) {
                        Ok(value) => value,
                        Err(e) => {
                            debug!("Failed to extract value for tag {}: {}", tag_def.name, e);
                            continue;
                        }
                    };

                // Convert to DataMember value for $val hash
                let data_member_value = match &raw_value {
                    TagValue::U8(v) => DataMemberValue::U8(*v),
                    TagValue::U16(v) => DataMemberValue::U16(*v),
                    TagValue::U32(v) => DataMemberValue::U32(*v),
                    TagValue::String(s) => DataMemberValue::String(s.clone()),
                    // Convert other types to appropriate DataMember types
                    TagValue::I16(v) => DataMemberValue::U16(*v as u16),
                    TagValue::I32(v) => DataMemberValue::U32(*v as u32),
                    _ => {
                        if let Some(data_member_name) = &tag_def.data_member {
                            debug!(
                                "Cannot convert tag value {:?} to DataMember for {}",
                                raw_value, data_member_name
                            );
                        }
                        // Still store in $val hash as U16 for index reference
                        DataMemberValue::U16(0)
                    }
                };

                // Store in DataMember system if this tag is a DataMember
                if let Some(data_member_name) = &tag_def.data_member {
                    // Store in global DataMember collection (need to refactor to avoid borrow conflicts)
                    debug!(
                        "Would store DataMember '{}' = {:?} from tag {}",
                        data_member_name, raw_value, tag_def.name
                    );
                    // TODO: Fix borrowing issue - need to restructure the evaluation
                }

                // Store in $val hash for current block references
                evaluator.set_val(index, data_member_value);

                // Update cumulative offset based on actual data size consumed
                let consumed_bytes = match &resolved_format {
                    crate::types::ResolvedFormat::Single(format) => format.byte_size(),
                    crate::types::ResolvedFormat::Array(format, count) => {
                        format.byte_size() * count
                    }
                    crate::types::ResolvedFormat::StringWithLength(length) => *length,
                    crate::types::ResolvedFormat::VarString => {
                        // Find actual string length in data
                        let mut string_len = 0;
                        let start_pos = data_offset;
                        while start_pos + string_len < data.len()
                            && data[start_pos + string_len] != 0
                        {
                            string_len += 1;
                        }
                        string_len + 1 // Include null terminator
                    }
                };

                debug!(
                    "Tag {} consumed {} bytes, new cumulative_offset={}",
                    tag_def.name,
                    consumed_bytes,
                    cumulative_offset + consumed_bytes
                );

                cumulative_offset += consumed_bytes;

                // Bounds check for next iteration
                if cumulative_offset > size {
                    debug!(
                        "Cumulative offset {} exceeds data bounds {}",
                        cumulative_offset, size
                    );
                    break;
                }

                // Apply PrintConv if available
                let final_value = if let Some(print_conv) = &tag_def.print_conv {
                    match &raw_value {
                        TagValue::I16(val) => {
                            if let Some(converted) = print_conv.get(&(*val as u32)) {
                                TagValue::String(converted.clone())
                            } else {
                                raw_value
                            }
                        }
                        TagValue::U16(val) => {
                            if let Some(converted) = print_conv.get(&(*val as u32)) {
                                TagValue::String(converted.clone())
                            } else {
                                raw_value
                            }
                        }
                        _ => raw_value,
                    }
                } else {
                    raw_value
                };

                // Store the tag with source info
                let group_0 = table
                    .groups
                    .get(&0)
                    .cloned()
                    .unwrap_or_else(|| "Unknown".to_string());
                let namespace = group_0.clone();
                let source_info =
                    TagSourceInfo::new(group_0, "BinaryData".to_string(), "BinaryData".to_string());
                let key = (index as u16, namespace);
                self.extracted_tags.insert(key.clone(), final_value);
                self.tag_sources.insert(key.clone(), source_info);

                debug!(
                    "Extracted binary tag {} (index {}) = {:?}",
                    tag_def.name,
                    index,
                    self.extracted_tags.get(&key)
                );
            }
        }

        Ok(())
    }

    /// Extract value using resolved format specification
    /// Handles single values, arrays, and variable-length strings
    fn extract_with_resolved_format(
        &self,
        data: &[u8],
        offset: usize,
        resolved_format: &ResolvedFormat,
    ) -> Result<TagValue> {
        let byte_order = if let Some(header) = &self.header {
            header.byte_order
        } else {
            ByteOrder::LittleEndian
        };

        match resolved_format {
            ResolvedFormat::Single(format) => {
                self.extract_single_binary_value(data, offset, *format, byte_order)
            }
            ResolvedFormat::Array(format, count) => {
                self.extract_binary_array(data, offset, *format, *count, byte_order)
            }
            ResolvedFormat::StringWithLength(length) => {
                if offset + length > data.len() {
                    return Err(ExifError::ParseError(
                        "String with length extends beyond data bounds".to_string(),
                    ));
                }
                let string_bytes = &data[offset..offset + length];
                let string_value = String::from_utf8_lossy(string_bytes).to_string();
                Ok(TagValue::String(string_value))
            }
            ResolvedFormat::VarString => {
                // Find null terminator
                let mut end = offset;
                while end < data.len() && data[end] != 0 {
                    end += 1;
                }
                let string_bytes = &data[offset..end];
                let string_value = String::from_utf8_lossy(string_bytes).to_string();
                Ok(TagValue::String(string_value))
            }
        }
    }

    /// Extract a single binary value
    fn extract_single_binary_value(
        &self,
        _data: &[u8],
        offset: usize,
        format: BinaryDataFormat,
        _byte_order: ByteOrder,
    ) -> Result<TagValue> {
        // Use existing extract_binary_value from Canon module
        crate::implementations::canon::extract_binary_value(self, offset, format, 1)
    }

    /// Extract an array of binary values
    fn extract_binary_array(
        &self,
        data: &[u8],
        offset: usize,
        format: BinaryDataFormat,
        count: usize,
        byte_order: ByteOrder,
    ) -> Result<TagValue> {
        // BinaryDataFormat already imported at module level

        if count == 0 {
            return Ok(TagValue::U8Array(vec![]));
        }

        let format_size = format.byte_size();
        let total_size = format_size * count;

        if offset + total_size > data.len() {
            return Err(ExifError::ParseError(format!(
                "Array of {count} {format_size} elements extends beyond data bounds"
            )));
        }

        match format {
            BinaryDataFormat::Int16s => {
                let mut values = Vec::new();
                for i in 0..count {
                    let value_offset = offset + i * format_size;
                    let value = byte_order.read_u16(data, value_offset)? as i16;
                    values.push(value);
                }
                // Convert to appropriate array type
                Ok(TagValue::U16Array(
                    values.into_iter().map(|v| v as u16).collect(),
                ))
            }
            BinaryDataFormat::Int16u => {
                let mut values = Vec::new();
                for i in 0..count {
                    let value_offset = offset + i * format_size;
                    let value = byte_order.read_u16(data, value_offset)?;
                    values.push(value);
                }
                Ok(TagValue::U16Array(values))
            }
            BinaryDataFormat::Int32u => {
                let mut values = Vec::new();
                for i in 0..count {
                    let value_offset = offset + i * format_size;
                    let value = byte_order.read_u32(data, value_offset)?;
                    values.push(value);
                }
                Ok(TagValue::U32Array(values))
            }
            // Add more format types as needed
            _ => Err(ExifError::ParseError(format!(
                "Array extraction not yet implemented for format {format:?}"
            ))),
        }
    }
}
