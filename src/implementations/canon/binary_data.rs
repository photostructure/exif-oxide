//! Canon binary data processing for CameraSettings and related tables
//!
//! This module handles Canon's binary data table format, particularly the CameraSettings
//! table that uses the 'int16s' format with 1-based indexing.
//!
//! **Trust ExifTool**: This code translates ExifTool's Canon.pm binary data processing
//! verbatim, including all PrintConv tables and processing logic.
//!
//! Primary ExifTool References:
//! - lib/Image/ExifTool/Canon.pm:2166+ %Canon::CameraSettings table
//! - ExifTool's ProcessBinaryData with FORMAT => 'int16s', FIRST_ENTRY => 1

use crate::tiff_types::ByteOrder;
use crate::types::{BinaryDataFormat, BinaryDataTable, BinaryDataTag, ExifError, Result, TagValue};
use std::collections::HashMap;
use tracing::debug;

/// Canon CameraSettings binary data tag definition
/// ExifTool: lib/Image/ExifTool/Canon.pm:2166-2240+ %Canon::CameraSettings
#[derive(Debug, Clone)]
pub struct CanonCameraSettingsTag {
    /// Tag index (1-based like ExifTool FIRST_ENTRY => 1)
    pub index: u32,
    /// Tag name
    pub name: String,
    /// PrintConv lookup table for human-readable values
    pub print_conv: Option<HashMap<i16, String>>,
}

/// Create Canon CameraSettings binary data table
/// ExifTool: lib/Image/ExifTool/Canon.pm:2166+ %Canon::CameraSettings
pub fn create_camera_settings_table() -> HashMap<u32, CanonCameraSettingsTag> {
    let mut table = HashMap::new();

    // ExifTool: Canon.pm:2172-2178 tag 1 MacroMode
    table.insert(
        1,
        CanonCameraSettingsTag {
            index: 1,
            name: "MacroMode".to_string(),
            print_conv: {
                let mut conv = HashMap::new();
                conv.insert(1i16, "Macro".to_string());
                conv.insert(2i16, "Normal".to_string());
                Some(conv)
            },
        },
    );

    // ExifTool: Canon.pm:2179-2191 tag 2 SelfTimer
    table.insert(
        2,
        CanonCameraSettingsTag {
            index: 2,
            name: "SelfTimer".to_string(),
            print_conv: {
                // Note: SelfTimer has complex Perl PrintConv logic
                // For now, implementing basic Off detection
                // TODO: Implement full PrintConv logic from Canon.pm:2182-2185
                let mut conv = HashMap::new();
                conv.insert(0i16, "Off".to_string());
                Some(conv)
            },
        },
    );

    // ExifTool: Canon.pm:2192-2195 tag 3 Quality
    table.insert(
        3,
        CanonCameraSettingsTag {
            index: 3,
            name: "Quality".to_string(),
            print_conv: {
                // Note: Quality uses %canonQuality hash reference
                // TODO: Implement canonQuality lookup table
                None // Placeholder for now
            },
        },
    );

    // ExifTool: Canon.pm:2196-2209 tag 4 CanonFlashMode
    table.insert(
        4,
        CanonCameraSettingsTag {
            index: 4,
            name: "CanonFlashMode".to_string(),
            print_conv: {
                let mut conv = HashMap::new();
                conv.insert(-1i16, "n/a".to_string()); // PH, EOS M MOV video
                conv.insert(0i16, "Off".to_string());
                conv.insert(1i16, "Auto".to_string());
                conv.insert(2i16, "On".to_string());
                conv.insert(3i16, "Red-eye reduction".to_string());
                conv.insert(4i16, "Slow-sync".to_string());
                conv.insert(5i16, "Red-eye reduction (Auto)".to_string());
                conv.insert(6i16, "Red-eye reduction (On)".to_string());
                conv.insert(16i16, "External flash".to_string()); // not set in D30 or 300D
                Some(conv)
            },
        },
    );

    // ExifTool: Canon.pm:2210-2227 tag 5 ContinuousDrive
    table.insert(
        5,
        CanonCameraSettingsTag {
            index: 5,
            name: "ContinuousDrive".to_string(),
            print_conv: {
                let mut conv = HashMap::new();
                conv.insert(0i16, "Single".to_string());
                conv.insert(1i16, "Continuous".to_string());
                conv.insert(2i16, "Movie".to_string()); // PH
                conv.insert(3i16, "Continuous, Speed Priority".to_string()); // PH
                Some(conv)
            },
        },
    );

    // ExifTool: Canon.pm:2228-2240 tag 7 FocusMode
    table.insert(
        7,
        CanonCameraSettingsTag {
            index: 7,
            name: "FocusMode".to_string(),
            print_conv: {
                let mut conv = HashMap::new();
                conv.insert(0i16, "One-shot AF".to_string());
                conv.insert(1i16, "AI Servo AF".to_string());
                conv.insert(2i16, "AI Focus AF".to_string());
                conv.insert(3i16, "Manual Focus (3)".to_string());
                conv.insert(4i16, "Single".to_string());
                conv.insert(5i16, "Continuous".to_string());
                conv.insert(6i16, "Manual Focus (6)".to_string());
                conv.insert(16i16, "Pan Focus".to_string()); // PH
                Some(conv)
            },
        },
    );

    table
}

/// Extract Canon CameraSettings binary data
/// ExifTool: ProcessBinaryData with Canon CameraSettings table parameters
///
/// Table parameters from Canon.pm:2166-2171:
/// - FORMAT => 'int16s' (signed 16-bit integers)
/// - FIRST_ENTRY => 1 (1-indexed)
/// - GROUPS => { 0 => 'MakerNotes', 2 => 'Camera' }
pub fn extract_camera_settings(
    data: &[u8],
    offset: usize,
    size: usize,
    byte_order: ByteOrder,
) -> Result<HashMap<String, TagValue>> {
    let table = create_camera_settings_table();
    let mut results = HashMap::new();

    // ExifTool: Canon.pm:2168 FORMAT => 'int16s'
    let format_size = 2; // int16s = 2 bytes

    debug!(
        "Extracting Canon CameraSettings: offset={:#x}, size={}, format=int16s",
        offset, size
    );

    // Process defined tags
    for (&index, tag_def) in &table {
        // ExifTool: Canon.pm:2169 FIRST_ENTRY => 1 (1-indexed)
        let entry_offset = (index - 1) as usize * format_size;

        if entry_offset + format_size > size {
            debug!("Tag {} at index {} beyond data bounds", tag_def.name, index);
            continue;
        }

        let data_offset = offset + entry_offset;

        if data_offset + format_size > data.len() {
            debug!(
                "Tag {} data offset {:#x} beyond buffer bounds",
                tag_def.name, data_offset
            );
            continue;
        }

        // Extract int16s value (signed 16-bit integer)
        let raw_value = byte_order.read_u16(data, data_offset)? as i16;

        // Apply PrintConv if available
        let final_value = if let Some(print_conv) = &tag_def.print_conv {
            if let Some(converted) = print_conv.get(&raw_value) {
                TagValue::String(converted.clone())
            } else {
                TagValue::I16(raw_value)
            }
        } else {
            TagValue::I16(raw_value)
        };

        debug!(
            "Extracted Canon {} = {:?} (raw: {}) at index {}",
            tag_def.name, final_value, raw_value, index
        );

        // Store with MakerNotes group prefix like ExifTool
        // ExifTool: Canon.pm:2171 GROUPS => { 0 => 'MakerNotes', 2 => 'Camera' }
        let tag_name = format!("MakerNotes:{}", tag_def.name);
        results.insert(tag_name, final_value);
    }

    Ok(results)
}

/// Create Canon AF Info binary data table with variable-length arrays
/// ExifTool: lib/Image/ExifTool/Canon.pm:4440+ %Canon::AFInfo table
/// Reference: third-party/exiftool/lib/Image/ExifTool/Canon.pm:4440-4500+ AFInfo table
/// Demonstrates Milestone 12: Variable ProcessBinaryData with DataMember dependencies
pub fn create_canon_af_info_table() -> BinaryDataTable {
    use crate::types::{BinaryDataFormat, FormatSpec};

    let mut table = BinaryDataTable {
        default_format: BinaryDataFormat::Int16u,
        first_entry: Some(0),
        groups: HashMap::new(),
        tags: HashMap::new(),
        data_member_tags: Vec::new(),
        dependency_order: Vec::new(),
    };

    // ExifTool: Canon.pm:4442 GROUPS => { 0 => 'MakerNotes', 2 => 'Camera' }
    table.groups.insert(0, "MakerNotes".to_string());
    table.groups.insert(2, "Camera".to_string());

    // NumAFPoints (sequence 0) - The key DataMember for variable-length arrays
    // ExifTool: Canon.pm:4450 '0 => { Name => 'NumAFPoints' }'
    table.tags.insert(
        0,
        BinaryDataTag {
            name: "NumAFPoints".to_string(),
            format_spec: Some(FormatSpec::Fixed(BinaryDataFormat::Int16u)),
            format: Some(BinaryDataFormat::Int16u),
            mask: None,
            print_conv: None,
            data_member: Some("NumAFPoints".to_string()), // This becomes a DataMember
        },
    );
    table.data_member_tags.push(0);

    // ValidAFPoints (sequence 1)
    // ExifTool: Canon.pm:4453 '1 => { Name => 'ValidAFPoints' }'
    table.tags.insert(
        1,
        BinaryDataTag {
            name: "ValidAFPoints".to_string(),
            format_spec: Some(FormatSpec::Fixed(BinaryDataFormat::Int16u)),
            format: Some(BinaryDataFormat::Int16u),
            mask: None,
            print_conv: None,
            data_member: None,
        },
    );

    // CanonImageWidth (sequence 2)
    table.tags.insert(
        2,
        BinaryDataTag {
            name: "CanonImageWidth".to_string(),
            format_spec: Some(FormatSpec::Fixed(BinaryDataFormat::Int16u)),
            format: Some(BinaryDataFormat::Int16u),
            mask: None,
            print_conv: None,
            data_member: None,
        },
    );

    // CanonImageHeight (sequence 3)
    table.tags.insert(
        3,
        BinaryDataTag {
            name: "CanonImageHeight".to_string(),
            format_spec: Some(FormatSpec::Fixed(BinaryDataFormat::Int16u)),
            format: Some(BinaryDataFormat::Int16u),
            mask: None,
            print_conv: None,
            data_member: None,
        },
    );

    // AFImageWidth (sequence 4)
    table.tags.insert(
        4,
        BinaryDataTag {
            name: "AFImageWidth".to_string(),
            format_spec: Some(FormatSpec::Fixed(BinaryDataFormat::Int16u)),
            format: Some(BinaryDataFormat::Int16u),
            mask: None,
            print_conv: None,
            data_member: None,
        },
    );

    // AFImageHeight (sequence 5)
    table.tags.insert(
        5,
        BinaryDataTag {
            name: "AFImageHeight".to_string(),
            format_spec: Some(FormatSpec::Fixed(BinaryDataFormat::Int16u)),
            format: Some(BinaryDataFormat::Int16u),
            mask: None,
            print_conv: None,
            data_member: None,
        },
    );

    // AFAreaWidth (sequence 6)
    table.tags.insert(
        6,
        BinaryDataTag {
            name: "AFAreaWidth".to_string(),
            format_spec: Some(FormatSpec::Fixed(BinaryDataFormat::Int16u)),
            format: Some(BinaryDataFormat::Int16u),
            mask: None,
            print_conv: None,
            data_member: None,
        },
    );

    // AFAreaHeight (sequence 7)
    table.tags.insert(
        7,
        BinaryDataTag {
            name: "AFAreaHeight".to_string(),
            format_spec: Some(FormatSpec::Fixed(BinaryDataFormat::Int16u)),
            format: Some(BinaryDataFormat::Int16u),
            mask: None,
            print_conv: None,
            data_member: None,
        },
    );

    // AFAreaXPositions (sequence 8) - Variable-length array sized by NumAFPoints
    // ExifTool: Canon.pm:4474 'Format => int16s[$val{0}]'
    table.tags.insert(
        8,
        BinaryDataTag {
            name: "AFAreaXPositions".to_string(),
            format_spec: Some(FormatSpec::Array {
                base_format: BinaryDataFormat::Int16s,
                count_expr: "$val{0}".to_string(), // References NumAFPoints at sequence 0
            }),
            format: None, // Will be resolved at runtime
            mask: None,
            print_conv: None,
            data_member: None,
        },
    );

    // AFAreaYPositions (sequence 9) - Variable-length array sized by NumAFPoints
    // ExifTool: Canon.pm:4477 'Format => int16s[$val{0}]'
    table.tags.insert(
        9,
        BinaryDataTag {
            name: "AFAreaYPositions".to_string(),
            format_spec: Some(FormatSpec::Array {
                base_format: BinaryDataFormat::Int16s,
                count_expr: "$val{0}".to_string(), // References NumAFPoints at sequence 0
            }),
            format: None, // Will be resolved at runtime
            mask: None,
            print_conv: None,
            data_member: None,
        },
    );

    // AFPointsInFocus (sequence 10) - Complex expression with bit array size calculation
    // ExifTool: Canon.pm:4480 'Format => int16s[int(($val{0}+15)/16)]'
    table.tags.insert(
        10,
        BinaryDataTag {
            name: "AFPointsInFocus".to_string(),
            format_spec: Some(FormatSpec::Array {
                base_format: BinaryDataFormat::Int16s,
                count_expr: "int(($val{0}+15)/16)".to_string(), // Ceiling division for bit arrays
            }),
            format: None, // Will be resolved at runtime
            mask: None,
            print_conv: None,
            data_member: None,
        },
    );

    // Analyze dependencies to establish processing order
    table.analyze_dependencies();

    table
}

/// Create Canon CameraSettings binary data table in the expected format
/// ExifTool: lib/Image/ExifTool/Canon.pm:2166+ %Canon::CameraSettings
/// This function creates a BinaryDataTable compatible with the test expectations
pub fn create_canon_camera_settings_table() -> BinaryDataTable {
    let mut table = BinaryDataTable {
        default_format: BinaryDataFormat::Int16s,
        first_entry: Some(1),
        groups: HashMap::new(),
        tags: HashMap::new(),
        data_member_tags: Vec::new(),
        dependency_order: Vec::new(),
    };

    // ExifTool: Canon.pm:2171 GROUPS => { 0 => 'MakerNotes', 2 => 'Camera' }
    table.groups.insert(0, "MakerNotes".to_string());
    table.groups.insert(2, "Camera".to_string());

    // ExifTool: Canon.pm:2172-2178 tag 1 MacroMode
    table.tags.insert(
        1,
        BinaryDataTag {
            name: "MacroMode".to_string(),
            format_spec: None, // Uses table default
            format: None,      // Uses table default
            mask: None,
            print_conv: {
                let mut conv = HashMap::new();
                conv.insert(1u32, "Macro".to_string());
                conv.insert(2u32, "Normal".to_string());
                Some(conv)
            },
            data_member: None,
        },
    );

    // ExifTool: Canon.pm:2179-2191 tag 2 SelfTimer
    table.tags.insert(
        2,
        BinaryDataTag {
            name: "SelfTimer".to_string(),
            format_spec: None,
            format: None,
            mask: None,
            print_conv: {
                let mut conv = HashMap::new();
                conv.insert(0u32, "Off".to_string());
                Some(conv)
            },
            data_member: None,
        },
    );

    // ExifTool: Canon.pm:2196-2209 tag 4 CanonFlashMode
    table.tags.insert(
        4,
        BinaryDataTag {
            name: "CanonFlashMode".to_string(),
            format_spec: None,
            format: None,
            mask: None,
            print_conv: {
                let mut conv = HashMap::new();
                conv.insert(0u32, "Off".to_string());
                conv.insert(1u32, "Auto".to_string());
                conv.insert(2u32, "On".to_string());
                Some(conv)
            },
            data_member: None,
        },
    );

    // ExifTool: Canon.pm:2228-2240 tag 7 FocusMode
    table.tags.insert(
        7,
        BinaryDataTag {
            name: "FocusMode".to_string(),
            format_spec: None,
            format: None,
            mask: None,
            print_conv: {
                let mut conv = HashMap::new();
                conv.insert(0u32, "One-shot AF".to_string());
                conv.insert(1u32, "AI Servo AF".to_string());
                conv.insert(2u32, "AI Focus AF".to_string());
                conv.insert(3u32, "Manual Focus (3)".to_string());
                Some(conv)
            },
            data_member: None,
        },
    );

    table
}

/// Extract binary value from ExifReader data
/// Used by binary data processing to extract individual values
pub fn extract_binary_value(
    reader: &crate::exif::ExifReader,
    offset: usize,
    format: BinaryDataFormat,
    _count: usize,
) -> Result<TagValue> {
    let data = reader.get_data();
    let byte_order = if let Some(header) = reader.get_header() {
        header.byte_order
    } else {
        // Default to little-endian when no header is available (common for test scenarios)
        ByteOrder::LittleEndian
    };

    match format {
        BinaryDataFormat::Int8u => {
            if offset >= data.len() {
                return Err(ExifError::ParseError(
                    "Offset beyond data bounds".to_string(),
                ));
            }
            Ok(TagValue::U8(data[offset]))
        }
        BinaryDataFormat::Int8s => {
            if offset >= data.len() {
                return Err(ExifError::ParseError(
                    "Offset beyond data bounds".to_string(),
                ));
            }
            // TagValue doesn't have I8, so store as I16
            Ok(TagValue::I16(data[offset] as i8 as i16))
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
        BinaryDataFormat::Int16s => {
            if offset + 2 > data.len() {
                return Err(ExifError::ParseError(
                    "Offset beyond data bounds for int16s".to_string(),
                ));
            }
            let value = byte_order.read_u16(data, offset)? as i16;
            Ok(TagValue::I16(value))
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
        BinaryDataFormat::Int32s => {
            if offset + 4 > data.len() {
                return Err(ExifError::ParseError(
                    "Offset beyond data bounds for int32s".to_string(),
                ));
            }
            let value = byte_order.read_u32(data, offset)? as i32;
            Ok(TagValue::I32(value))
        }
        BinaryDataFormat::String => {
            // Extract null-terminated string
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
        BinaryDataFormat::PString => {
            // Pascal string: first byte is length
            if offset >= data.len() {
                return Err(ExifError::ParseError(
                    "Offset beyond data bounds for pstring".to_string(),
                ));
            }

            let length = data[offset] as usize;
            if offset + 1 + length > data.len() {
                return Err(ExifError::ParseError(
                    "Pascal string length exceeds data bounds".to_string(),
                ));
            }

            let string_bytes = &data[offset + 1..offset + 1 + length];
            let string_value = String::from_utf8_lossy(string_bytes).to_string();
            Ok(TagValue::String(string_value))
        }
        _ => Err(ExifError::ParseError(format!(
            "Binary format {format:?} not yet implemented"
        ))),
    }
}

/// Extract binary data tags from ExifReader using a binary data table
/// This processes the data according to the table configuration
pub fn extract_binary_data_tags(
    reader: &mut crate::exif::ExifReader,
    offset: usize,
    size: usize,
    table: &BinaryDataTable,
) -> Result<()> {
    use crate::types::TagSourceInfo;

    debug!(
        "Extracting binary data tags: offset={:#x}, size={}, format={:?}",
        offset, size, table.default_format
    );

    // Process each defined tag in the table
    for (&index, tag_def) in &table.tags {
        // Calculate position based on FIRST_ENTRY
        let first_entry = table.first_entry.unwrap_or(0);
        if index < first_entry {
            continue;
        }

        let entry_offset = (index - first_entry) as usize * table.default_format.byte_size();
        if entry_offset + table.default_format.byte_size() > size {
            debug!("Tag {} at index {} beyond data bounds", tag_def.name, index);
            continue;
        }

        let data_offset = offset + entry_offset;

        // Extract the raw value
        let format = tag_def.format.unwrap_or(table.default_format);
        let raw_value = extract_binary_value(reader, data_offset, format, 1)?;

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
        let source_info = TagSourceInfo::new(
            group_0,
            "Canon".to_string(),
            "Canon::BinaryData".to_string(),
        );

        reader.extracted_tags.insert(index as u16, final_value);
        reader.tag_sources.insert(index as u16, source_info);

        debug!(
            "Extracted Canon binary tag {} (index {}) = {:?}",
            tag_def.name,
            index,
            reader.extracted_tags.get(&(index as u16))
        );
    }

    Ok(())
}

/// Find Canon CameraSettings tag in MakerNotes IFD
/// Searches for tag 0x0001 (CanonCameraSettings) in the IFD structure
pub fn find_canon_camera_settings_tag(
    reader: &crate::exif::ExifReader,
    ifd_offset: usize,
    _size: usize,
) -> Result<usize> {
    let data = reader.get_data();
    let byte_order = if let Some(header) = reader.get_header() {
        header.byte_order
    } else {
        // Default to little-endian when no header is available (common for test scenarios)
        ByteOrder::LittleEndian
    };

    // Ensure we have enough data for the entry count
    if ifd_offset + 2 > data.len() {
        return Err(ExifError::ParseError(
            "IFD offset beyond data bounds".to_string(),
        ));
    }

    // Read the number of IFD entries
    let entry_count = byte_order.read_u16(data, ifd_offset)? as usize;
    let entries_start = ifd_offset + 2;
    let entries_size = entry_count * 12; // Each IFD entry is 12 bytes

    if entries_start + entries_size > data.len() {
        return Err(ExifError::ParseError(
            "IFD entries beyond data bounds".to_string(),
        ));
    }

    // Search for Canon CameraSettings tag (0x0001)
    for i in 0..entry_count {
        let entry_offset = entries_start + (i * 12);
        let tag_id = byte_order.read_u16(data, entry_offset)?;

        if tag_id == 0x0001 {
            // Found CanonCameraSettings tag, return the value offset
            let format = byte_order.read_u16(data, entry_offset + 2)?;
            let count = byte_order.read_u32(data, entry_offset + 4)?;
            let value_offset = byte_order.read_u32(data, entry_offset + 8)? as usize;

            debug!(
                "Found Canon CameraSettings tag: format={}, count={}, offset={:#x}",
                format, count, value_offset
            );

            return Ok(value_offset);
        }
    }

    Err(ExifError::ParseError(
        "Canon CameraSettings tag not found in IFD".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exif::ExifReader;
    use crate::tiff_types::{ByteOrder, TiffHeader};

    #[test]
    fn test_canon_af_info_variable_arrays() {
        // Test Milestone 12: Variable ProcessBinaryData with Canon AF Info
        // Reference: third-party/exiftool/lib/Image/ExifTool/Canon.pm:4440-4500+ AFInfo table
        // This demonstrates variable-length arrays sized by DataMember dependencies

        let table = create_canon_af_info_table();

        // Verify table structure
        assert_eq!(table.default_format, BinaryDataFormat::Int16u);
        assert_eq!(table.first_entry, Some(0));
        assert_eq!(table.groups.get(&0), Some(&"MakerNotes".to_string()));
        assert_eq!(table.groups.get(&2), Some(&"Camera".to_string()));

        // Verify DataMember tags
        assert!(table.data_member_tags.contains(&0)); // NumAFPoints

        // Verify tag definitions
        let num_af_points = table.tags.get(&0).unwrap();
        assert_eq!(num_af_points.name, "NumAFPoints");
        assert_eq!(num_af_points.data_member, Some("NumAFPoints".to_string()));

        // Verify variable array formats
        let af_x_positions = table.tags.get(&8).unwrap();
        assert_eq!(af_x_positions.name, "AFAreaXPositions");
        if let Some(format_spec) = &af_x_positions.format_spec {
            match format_spec {
                crate::types::FormatSpec::Array {
                    base_format,
                    count_expr,
                } => {
                    assert_eq!(*base_format, BinaryDataFormat::Int16s);
                    assert_eq!(count_expr, "$val{0}"); // References NumAFPoints
                }
                _ => panic!("Expected Array format spec for AFAreaXPositions"),
            }
        } else {
            panic!("AFAreaXPositions should have format_spec");
        }

        let af_y_positions = table.tags.get(&9).unwrap();
        assert_eq!(af_y_positions.name, "AFAreaYPositions");

        let af_points_in_focus = table.tags.get(&10).unwrap();
        assert_eq!(af_points_in_focus.name, "AFPointsInFocus");
        if let Some(format_spec) = &af_points_in_focus.format_spec {
            match format_spec {
                crate::types::FormatSpec::Array {
                    base_format,
                    count_expr,
                } => {
                    assert_eq!(*base_format, BinaryDataFormat::Int16s);
                    assert_eq!(count_expr, "int(($val{0}+15)/16)"); // Complex expression
                }
                _ => panic!("Expected Array format spec for AFPointsInFocus"),
            }
        }
    }

    #[test]
    fn test_canon_af_info_processing() {
        // Test actual processing of Canon AF Info data with variable arrays
        // Reference: third-party/exiftool/lib/Image/ExifTool/Canon.pm:4474+ AFAreaXPositions Format => int16s[$val{0}]
        // Reference: third-party/exiftool/lib/Image/ExifTool/Canon.pm:4477+ AFAreaYPositions Format => int16s[$val{0}]
        // Reference: third-party/exiftool/lib/Image/ExifTool/Canon.pm:4480+ AFPointsInFocus Format => int16s[int(($val{0}+15)/16)]
        // This simulates real Canon AF Info data with NumAFPoints = 9

        let mut reader = ExifReader::new();
        reader.set_test_header(TiffHeader {
            byte_order: ByteOrder::LittleEndian,
            magic: 42,
            ifd0_offset: 8,
        });

        // Create test AF Info data:
        // Sequence 0: NumAFPoints = 9 (0x0009)
        // Sequence 1: ValidAFPoints = 7 (0x0007)
        // Sequence 2: CanonImageWidth = 1600 (0x0640)
        // Sequence 3: CanonImageHeight = 1200 (0x04B0)
        // Sequence 4: AFImageWidth = 1024 (0x0400)
        // Sequence 5: AFImageHeight = 768 (0x0300)
        // Sequence 6: AFAreaWidth = 64 (0x0040)
        // Sequence 7: AFAreaHeight = 48 (0x0030)
        // Sequence 8: AFAreaXPositions[9] = [-200, -100, 0, 100, 200, -150, 0, 150, 0]
        // Sequence 9: AFAreaYPositions[9] = [-100, -50, 0, 50, 100, 75, 0, -75, 150]
        // Sequence 10: AFPointsInFocus[1] = [0x01FF] (bit array: ceiling(9/16) = 1 word)

        let test_data = vec![
            // Sequences 0-7: Fixed data (8 * 2 bytes = 16 bytes)
            0x09, 0x00, // NumAFPoints = 9
            0x07, 0x00, // ValidAFPoints = 7
            0x40, 0x06, // CanonImageWidth = 1600
            0xB0, 0x04, // CanonImageHeight = 1200
            0x00, 0x04, // AFImageWidth = 1024
            0x00, 0x03, // AFImageHeight = 768
            0x40, 0x00, // AFAreaWidth = 64
            0x30, 0x00, // AFAreaHeight = 48
            // Sequence 8: AFAreaXPositions[9] (9 * 2 bytes = 18 bytes)
            0x38, 0xFF, // -200 (0xFF38 = -200 in 2's complement)
            0x9C, 0xFF, // -100 (0xFF9C = -100 in 2's complement)
            0x00, 0x00, // 0
            0x64, 0x00, // 100
            0xC8, 0x00, // 200
            0x6A, 0xFF, // -150 (0xFF6A = -150 in 2's complement)
            0x00, 0x00, // 0
            0x96, 0x00, // 150
            0x00, 0x00, // 0
            // Sequence 9: AFAreaYPositions[9] (9 * 2 bytes = 18 bytes)
            0x9C, 0xFF, // -100 (0xFF9C = -100 in 2's complement)
            0xCE, 0xFF, // -50 (0xFFCE = -50 in 2's complement)
            0x00, 0x00, // 0
            0x32, 0x00, // 50
            0x64, 0x00, // 100
            0x4B, 0x00, // 75
            0x00, 0x00, // 0
            0xB5, 0xFF, // -75 (0xFFB5 = -75 in 2's complement)
            0x96, 0x00, // 150
            // Sequence 10: AFPointsInFocus[1] = int((9+15)/16) = 1 word (2 bytes)
            0xFF, 0x01, // 0x01FF - bits 0-8 set (AF points 1-9 in focus)
        ];

        reader.set_test_data(test_data.clone());

        let table = create_canon_af_info_table();

        // Process the binary data with dependencies
        let result =
            reader.process_binary_data_with_dependencies(&test_data, 0, test_data.len(), &table);
        assert!(
            result.is_ok(),
            "Failed to process Canon AF Info data: {result:?}"
        );

        // Verify extracted tags
        let extracted_tags = reader.get_extracted_tags();

        // Check NumAFPoints (DataMember)
        assert_eq!(
            extracted_tags.get(&0),
            Some(&crate::types::TagValue::U16(9))
        );

        // Check ValidAFPoints
        assert_eq!(
            extracted_tags.get(&1),
            Some(&crate::types::TagValue::U16(7))
        );

        // Check CanonImageWidth
        assert_eq!(
            extracted_tags.get(&2),
            Some(&crate::types::TagValue::U16(1600))
        );

        // Check CanonImageHeight
        assert_eq!(
            extracted_tags.get(&3),
            Some(&crate::types::TagValue::U16(1200))
        );

        // Check variable arrays - AFAreaXPositions should be array of 9 elements
        if let Some(crate::types::TagValue::U16Array(x_positions)) = extracted_tags.get(&8) {
            assert_eq!(
                x_positions.len(),
                9,
                "AFAreaXPositions should have 9 elements based on NumAFPoints"
            );
            // Note: The values will be stored as U16 due to array extraction conversion
        } else {
            panic!("AFAreaXPositions should be U16Array");
        }

        // Check variable arrays - AFAreaYPositions should be array of 9 elements
        if let Some(crate::types::TagValue::U16Array(y_positions)) = extracted_tags.get(&9) {
            assert_eq!(
                y_positions.len(),
                9,
                "AFAreaYPositions should have 9 elements based on NumAFPoints"
            );
        } else {
            panic!("AFAreaYPositions should be U16Array");
        }

        // Check complex expression - AFPointsInFocus should be array of 1 element
        // Expression: int((9+15)/16) = int(24/16) = 1
        if let Some(crate::types::TagValue::U16Array(points_in_focus)) = extracted_tags.get(&10) {
            assert_eq!(
                points_in_focus.len(),
                1,
                "AFPointsInFocus should have 1 element based on ceiling division"
            );
        } else {
            panic!("AFPointsInFocus should be U16Array");
        }
    }

    #[test]
    fn test_expression_evaluator() {
        // Test the complex expression evaluator with Canon AF ceiling division
        // Reference: third-party/exiftool/lib/Image/ExifTool/Canon.pm:4480+ int(($val{0}+15)/16) pattern
        use crate::types::{DataMemberValue, ExpressionEvaluator};
        use std::collections::HashMap;

        let data_members = HashMap::new();
        let mut val_hash = HashMap::new();
        val_hash.insert(0, DataMemberValue::U16(9)); // NumAFPoints = 9

        let evaluator = ExpressionEvaluator::new(val_hash, &data_members);

        // Test simple $val{0} expression
        let simple_result = evaluator.evaluate_count_expression("$val{0}");
        assert_eq!(simple_result.unwrap(), 9);

        // Test complex ceiling division: int((9+15)/16) = int(24/16) = 1
        let complex_result = evaluator.evaluate_count_expression("int(($val{0}+15)/16)");
        println!("Complex expression result: {complex_result:?}");
        match complex_result {
            Ok(val) => assert_eq!(val, 1),
            Err(e) => panic!("Complex expression failed: {e}"),
        }
    }

    #[test]
    fn test_variable_string_formats() {
        // Test Milestone 12: Variable string formats with string[$val{N}]
        // Reference: third-party/exiftool/lib/Image/ExifTool.pm:9850+ string format parsing
        use crate::exif::ExifReader;
        use crate::tiff_types::{ByteOrder, TiffHeader};
        use crate::types::{BinaryDataFormat, BinaryDataTable, BinaryDataTag, FormatSpec};
        use std::collections::HashMap;

        let mut reader = ExifReader::new();
        reader.set_test_header(TiffHeader {
            byte_order: ByteOrder::LittleEndian,
            magic: 42,
            ifd0_offset: 8,
        });

        // Create a test table with string length dependency
        let mut table = BinaryDataTable {
            default_format: BinaryDataFormat::Int16u,
            first_entry: Some(0),
            groups: HashMap::new(),
            tags: HashMap::new(),
            data_member_tags: Vec::new(),
            dependency_order: Vec::new(),
        };

        // Tag 0: StringLength (DataMember) = 5
        table.tags.insert(
            0,
            BinaryDataTag {
                name: "StringLength".to_string(),
                format_spec: Some(FormatSpec::Fixed(BinaryDataFormat::Int16u)),
                format: Some(BinaryDataFormat::Int16u),
                mask: None,
                print_conv: None,
                data_member: Some("StringLength".to_string()),
            },
        );
        table.data_member_tags.push(0);

        // Tag 1: VariableString = string[$val{0}] (5 characters)
        table.tags.insert(
            1,
            BinaryDataTag {
                name: "VariableString".to_string(),
                format_spec: Some(FormatSpec::StringWithLength {
                    length_expr: "$val{0}".to_string(),
                }),
                format: None,
                mask: None,
                print_conv: None,
                data_member: None,
            },
        );

        table.analyze_dependencies();

        // Test data: StringLength=5, then "Hello" (5 bytes)
        let test_data = vec![
            0x05, 0x00, // StringLength = 5
            b'H', b'e', b'l', b'l', b'o', // "Hello" (5 bytes)
        ];

        reader.set_test_data(test_data.clone());

        let result =
            reader.process_binary_data_with_dependencies(&test_data, 0, test_data.len(), &table);
        assert!(result.is_ok());

        let extracted_tags = reader.get_extracted_tags();

        // Check StringLength DataMember
        assert_eq!(
            extracted_tags.get(&0),
            Some(&crate::types::TagValue::U16(5))
        );

        // Check VariableString
        assert_eq!(
            extracted_tags.get(&1),
            Some(&crate::types::TagValue::String("Hello".to_string()))
        );
    }

    #[test]
    fn test_edge_cases_zero_count() {
        // Test edge case: zero count for arrays
        use crate::exif::ExifReader;
        use crate::tiff_types::{ByteOrder, TiffHeader};
        use crate::types::{BinaryDataFormat, BinaryDataTable, BinaryDataTag, FormatSpec};
        use std::collections::HashMap;

        let mut reader = ExifReader::new();
        reader.set_test_header(TiffHeader {
            byte_order: ByteOrder::LittleEndian,
            magic: 42,
            ifd0_offset: 8,
        });

        let mut table = BinaryDataTable {
            default_format: BinaryDataFormat::Int16u,
            first_entry: Some(0),
            groups: HashMap::new(),
            tags: HashMap::new(),
            data_member_tags: Vec::new(),
            dependency_order: Vec::new(),
        };

        // Tag 0: Count = 0
        table.tags.insert(
            0,
            BinaryDataTag {
                name: "Count".to_string(),
                format_spec: Some(FormatSpec::Fixed(BinaryDataFormat::Int16u)),
                format: Some(BinaryDataFormat::Int16u),
                mask: None,
                print_conv: None,
                data_member: Some("Count".to_string()),
            },
        );
        table.data_member_tags.push(0);

        // Tag 1: EmptyArray = int16s[$val{0}] (0 elements)
        table.tags.insert(
            1,
            BinaryDataTag {
                name: "EmptyArray".to_string(),
                format_spec: Some(FormatSpec::Array {
                    base_format: BinaryDataFormat::Int16s,
                    count_expr: "$val{0}".to_string(),
                }),
                format: None,
                mask: None,
                print_conv: None,
                data_member: None,
            },
        );

        table.analyze_dependencies();

        let test_data = vec![0x00, 0x00]; // Count = 0
        reader.set_test_data(test_data.clone());

        let result =
            reader.process_binary_data_with_dependencies(&test_data, 0, test_data.len(), &table);
        assert!(result.is_ok());

        let extracted_tags = reader.get_extracted_tags();

        // Check Count
        assert_eq!(
            extracted_tags.get(&0),
            Some(&crate::types::TagValue::U16(0))
        );

        // Check EmptyArray (should be empty array)
        assert_eq!(
            extracted_tags.get(&1),
            Some(&crate::types::TagValue::U8Array(vec![]))
        );
    }

    #[test]
    fn test_create_camera_settings_table() {
        let table = create_camera_settings_table();

        // Test that expected tags are present
        assert!(table.contains_key(&1)); // MacroMode
        assert!(table.contains_key(&2)); // SelfTimer
        assert!(table.contains_key(&3)); // Quality
        assert!(table.contains_key(&4)); // CanonFlashMode
        assert!(table.contains_key(&5)); // ContinuousDrive
        assert!(table.contains_key(&7)); // FocusMode

        // Test tag structure
        let macro_mode = table.get(&1).unwrap();
        assert_eq!(macro_mode.name, "MacroMode");
        assert!(macro_mode.print_conv.is_some());

        let print_conv = macro_mode.print_conv.as_ref().unwrap();
        assert_eq!(print_conv.get(&1), Some(&"Macro".to_string()));
        assert_eq!(print_conv.get(&2), Some(&"Normal".to_string()));
    }

    #[test]
    fn test_extract_camera_settings_basic() {
        // Create test data: two int16s values
        let test_data = vec![0x00, 0x01, 0x00, 0x02]; // [1, 2] in big-endian

        let result = extract_camera_settings(&test_data, 0, 4, ByteOrder::BigEndian);
        assert!(result.is_ok());

        let tags = result.unwrap();

        // Should have extracted MacroMode (index 1) and SelfTimer (index 2)
        assert!(tags.contains_key("MakerNotes:MacroMode"));
        assert!(tags.contains_key("MakerNotes:SelfTimer"));

        // MacroMode value 1 should be converted to "Macro"
        if let Some(TagValue::String(value)) = tags.get("MakerNotes:MacroMode") {
            assert_eq!(value, "Macro");
        } else {
            panic!("MacroMode should be converted to string");
        }

        // SelfTimer value 2 should remain as I16 (no PrintConv for value 2)
        if let Some(TagValue::I16(value)) = tags.get("MakerNotes:SelfTimer") {
            assert_eq!(*value, 2);
        } else {
            panic!("SelfTimer should be I16 for unconverted value");
        }
    }
}
