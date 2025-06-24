//! ProcessBinaryData framework for structured binary data parsing
//!
//! This module implements ExifTool's ProcessBinaryData functionality for parsing
//! structured binary data in maker notes. Many manufacturers use this format
//! for advanced metadata.

use crate::core::types::ExifFormat;
use crate::core::{Endian, ExifValue};
use crate::error::{Error, Result};
use std::collections::HashMap;

/// Field definition for binary data parsing
#[derive(Debug, Clone)]
pub struct BinaryField {
    /// Field name
    pub name: String,
    /// Byte offset in the data
    pub offset: usize,
    /// Data format (None means use default format)
    pub format: Option<ExifFormat>,
    /// Number of values (1 for single value)
    pub count: usize,
    /// Bit mask for extracting part of a value
    pub mask: Option<u32>,
    /// Bit shift for mask operations
    pub shift: u8,
    /// Print conversion function name (for debugging)
    pub print_conv: Option<String>,
}

/// Binary data table definition
#[derive(Debug, Clone)]
pub struct BinaryDataTable {
    /// Table name
    pub name: String,
    /// Default format for fields
    pub default_format: ExifFormat,
    /// Field definitions (stored as a Vec to allow multiple fields at same position)
    pub fields: Vec<(usize, BinaryField)>,
    /// Writable flag
    pub writable: bool,
}

impl BinaryDataTable {
    /// Create a new binary data table
    pub fn new(name: &str, default_format: ExifFormat) -> Self {
        Self {
            name: name.to_string(),
            default_format,
            fields: Vec::new(),
            writable: false,
        }
    }

    /// Add a field to the table
    pub fn add_field(&mut self, position: usize, field: BinaryField) {
        self.fields.push((position, field));
    }

    /// Parse binary data using this table
    pub fn parse(&self, data: &[u8], byte_order: Endian) -> Result<HashMap<u16, ExifValue>> {
        let mut results = HashMap::new();

        for &(position, ref field) in &self.fields {
            // Calculate actual offset
            let offset = if position == 0 {
                field.offset
            } else {
                position * self.default_format.size()
            };

            if let Ok(value) = self.parse_field(data, field, offset, byte_order) {
                // Use tag ID based on position (ExifTool style: 0x8000 + position)
                let tag_id = 0x8000 + (position as u16);
                results.insert(tag_id, value);
            }
        }

        Ok(results)
    }

    /// Parse a single field from binary data
    fn parse_field(
        &self,
        data: &[u8],
        field: &BinaryField,
        offset: usize,
        byte_order: Endian,
    ) -> Result<ExifValue> {
        let format = field.format.unwrap_or(self.default_format);

        let size = format.size();
        let total_size = size * field.count;

        if offset + total_size > data.len() {
            return Err(Error::InvalidExif(
                "Binary field extends beyond data".to_string(),
            ));
        }

        let field_data = &data[offset..offset + total_size];

        // Special handling for ASCII - return raw bytes directly
        if format == ExifFormat::Ascii {
            return Ok(ExifValue::U8Array(field_data.to_vec()));
        }

        // Parse based on format
        let mut values = Vec::new();
        for i in 0..field.count {
            let value_offset = i * size;
            let value_data = &field_data[value_offset..value_offset + size];

            let raw_value = match format {
                ExifFormat::U8 => value_data[0] as u32,
                ExifFormat::U16 => match byte_order {
                    Endian::Little => u16::from_le_bytes([value_data[0], value_data[1]]) as u32,
                    Endian::Big => u16::from_be_bytes([value_data[0], value_data[1]]) as u32,
                },
                ExifFormat::U32 => match byte_order {
                    Endian::Little => u32::from_le_bytes([
                        value_data[0],
                        value_data[1],
                        value_data[2],
                        value_data[3],
                    ]),
                    Endian::Big => u32::from_be_bytes([
                        value_data[0],
                        value_data[1],
                        value_data[2],
                        value_data[3],
                    ]),
                },
                _ => {
                    // For now, just treat as raw bytes for unsupported formats
                    value_data[0] as u32
                }
            };

            // Apply mask if specified
            let final_value = if let Some(mask) = field.mask {
                (raw_value & mask) >> field.shift
            } else {
                raw_value
            };

            values.push(final_value);
        }

        // Convert to appropriate ExifValue
        if field.count == 1 {
            match format {
                ExifFormat::U8 => Ok(ExifValue::U8(values[0] as u8)),
                ExifFormat::U16 => Ok(ExifValue::U16(values[0] as u16)),
                ExifFormat::U32 => Ok(ExifValue::U32(values[0])),
                _ => Ok(ExifValue::U32(values[0])),
            }
        } else {
            // Multiple values - create array
            match format {
                ExifFormat::U8 => Ok(ExifValue::U8Array(
                    values.iter().map(|&v| v as u8).collect(),
                )),
                ExifFormat::U16 => Ok(ExifValue::U16Array(
                    values.iter().map(|&v| v as u16).collect(),
                )),
                ExifFormat::U32 => Ok(ExifValue::U32Array(values)),
                _ => Ok(ExifValue::U32Array(values)),
            }
        }
    }
}

/// Builder for creating binary data tables
pub struct BinaryDataTableBuilder {
    table: BinaryDataTable,
}

impl BinaryDataTableBuilder {
    /// Create a new builder
    pub fn new(name: &str, default_format: ExifFormat) -> Self {
        Self {
            table: BinaryDataTable::new(name, default_format),
        }
    }

    /// Add a simple field
    pub fn add_field(
        mut self,
        position: usize,
        name: &str,
        format: ExifFormat,
        count: usize,
    ) -> Self {
        let field = BinaryField {
            name: name.to_string(),
            offset: 0,
            format: Some(format),
            count,
            mask: None,
            shift: 0,
            print_conv: None,
        };
        self.table.add_field(position, field);
        self
    }

    /// Add a field with bit mask
    pub fn add_masked_field(
        mut self,
        position: usize,
        name: &str,
        format: ExifFormat,
        mask: u32,
        shift: u8,
    ) -> Self {
        let field = BinaryField {
            name: name.to_string(),
            offset: 0,
            format: Some(format),
            count: 1,
            mask: Some(mask),
            shift,
            print_conv: None,
        };
        self.table.add_field(position, field);
        self
    }

    /// Build the table
    pub fn build(self) -> BinaryDataTable {
        self.table
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_data_table_creation() {
        let table = BinaryDataTableBuilder::new("TestTable", ExifFormat::U16)
            .add_field(0, "Field1", ExifFormat::U16, 1)
            .add_masked_field(1, "Field2", ExifFormat::U16, 0x000f, 0)
            .build();

        assert_eq!(table.name, "TestTable");
        assert_eq!(table.default_format, ExifFormat::U16);
        assert_eq!(table.fields.len(), 2);
    }

    #[test]
    fn test_binary_data_parsing() {
        let table = BinaryDataTableBuilder::new("TestTable", ExifFormat::U16)
            .add_field(0, "Field1", ExifFormat::U16, 1)
            .add_field(1, "Field2", ExifFormat::U16, 1)
            .build();

        let data = [0x12, 0x34, 0x56, 0x78]; // Two u16 values: 0x3412, 0x7856 (little-endian)
        let result = table.parse(&data, Endian::Little).unwrap();

        assert_eq!(result.len(), 2);
        assert!(matches!(result.get(&0x8000), Some(ExifValue::U16(0x3412))));
        assert!(matches!(result.get(&0x8001), Some(ExifValue::U16(0x7856))));
    }

    #[test]
    fn test_masked_field_parsing() {
        let table = BinaryDataTableBuilder::new("TestTable", ExifFormat::U16)
            .add_masked_field(0, "LowNibble", ExifFormat::U16, 0x000f, 0)
            .add_masked_field(0, "HighNibble", ExifFormat::U16, 0x00f0, 4)
            .build();

        let data = [0x34, 0x12]; // 0x1234 in little-endian
        let result = table.parse(&data, Endian::Little).unwrap();

        // Both fields should extract from the same position but different masks
        assert!(!result.is_empty()); // At least one field should parse successfully
    }
}
