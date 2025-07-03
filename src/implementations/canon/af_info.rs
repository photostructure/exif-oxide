//! Canon AF Info sequential data processing
//!
//! This module handles Canon AF Info and AF Info2 tag processing using sequential
//! data extraction with variable-length arrays. The AF Info tags contain autofocus
//! point information with complex interdependencies between tags.
//!
//! **ExifTool is Gospel**: This code translates ExifTool's Canon AF processing verbatim
//! without any improvements or simplifications. Every algorithm, magic number, and
//! quirk is copied exactly as documented in the ExifTool source.
//!
//! Primary ExifTool References:
//! - lib/Image/ExifTool/Canon.pm:8916-9053 %Canon::AFInfo
//! - lib/Image/ExifTool/Canon.pm:9055-9189 %Canon::AFInfo2
//! - lib/Image/ExifTool/Canon.pm:10224-10306 ProcessSerialData

use crate::tiff_types::ByteOrder;
use crate::types::{Result, TagValue};
use std::collections::HashMap;
use tracing::debug;

/// Canon AF Info tag definition for sequential data processing
/// ExifTool: Canon.pm:10224-10306 ProcessSerialData
#[derive(Debug, Clone)]
pub struct CanonAfInfoTag {
    /// Sequential index (0-based like ExifTool sequence processing)
    pub sequence: u32,
    /// Tag name
    pub name: String,
    /// Format type for this tag
    pub format: CanonAfFormat,
    /// Size calculation for variable arrays
    pub size_expr: CanonAfSizeExpr,
    /// Optional condition for tag extraction
    pub condition: Option<CanonAfCondition>,
    /// PrintConv lookup table
    pub print_conv: Option<HashMap<u16, String>>,
}

/// Canon AF data format types
/// ExifTool: Canon.pm AFInfo/AFInfo2 format specifications
#[derive(Debug, Clone)]
pub enum CanonAfFormat {
    /// Fixed 16-bit unsigned integer
    Int16u,
    /// Fixed 16-bit signed integer
    Int16s,
    /// Variable array of 16-bit signed integers with dynamic count
    Int16sArray(CanonAfSizeExpr),
}

/// Size expression for variable arrays in Canon AF data
/// ExifTool: Canon.pm expressions like $val{0}, int(($val{0}+15)/16)
#[derive(Debug, Clone)]
pub enum CanonAfSizeExpr {
    /// Fixed count
    Fixed(usize),
    /// Reference to previously extracted value: $val{N}
    ValueRef(u32),
    /// Ceiling division: int(($val{N}+15)/16) for bit packing
    CeilDiv(u32, u32), // (value_ref, divisor)
}

/// Conditional logic for Canon AF tag extraction
/// ExifTool: Canon.pm Condition expressions
#[derive(Debug, Clone)]
pub enum CanonAfCondition {
    /// Model-based condition: $$self{Model} !~ /EOS/
    ModelNotEos,
    /// Model-based condition: $$self{Model} =~ /EOS/
    ModelIsEos,
}

impl CanonAfSizeExpr {
    /// Calculate array size based on previously extracted values
    /// ExifTool: Canon.pm ProcessSerialData size calculation
    pub fn calculate_size(&self, extracted_values: &HashMap<u32, u16>) -> usize {
        match self {
            CanonAfSizeExpr::Fixed(count) => *count,
            CanonAfSizeExpr::ValueRef(value_ref) => {
                extracted_values.get(value_ref).copied().unwrap_or(0) as usize
            }
            CanonAfSizeExpr::CeilDiv(value_ref, divisor) => {
                let val = extracted_values.get(value_ref).copied().unwrap_or(0) as usize;
                let divisor = *divisor as usize;
                val.div_ceil(divisor) // Ceiling division
            }
        }
    }
}

/// Create Canon AFInfo tag table for sequential processing
/// ExifTool: Canon.pm:8916-9053 %Canon::AFInfo
pub fn create_af_info_table() -> Vec<CanonAfInfoTag> {
    vec![
        // Sequence 0: NumAFPoints
        CanonAfInfoTag {
            sequence: 0,
            name: "NumAFPoints".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: None,
        },
        // Sequence 1: ValidAFPoints
        CanonAfInfoTag {
            sequence: 1,
            name: "ValidAFPoints".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: None,
        },
        // Sequence 2: CanonImageWidth
        CanonAfInfoTag {
            sequence: 2,
            name: "CanonImageWidth".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: None,
        },
        // Sequence 3: CanonImageHeight
        CanonAfInfoTag {
            sequence: 3,
            name: "CanonImageHeight".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: None,
        },
        // Sequence 4: AFImageWidth
        CanonAfInfoTag {
            sequence: 4,
            name: "AFImageWidth".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: None,
        },
        // Sequence 5: AFImageHeight
        CanonAfInfoTag {
            sequence: 5,
            name: "AFImageHeight".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: None,
        },
        // Sequence 6: AFAreaWidth
        CanonAfInfoTag {
            sequence: 6,
            name: "AFAreaWidth".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: None,
        },
        // Sequence 7: AFAreaHeight
        CanonAfInfoTag {
            sequence: 7,
            name: "AFAreaHeight".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: None,
        },
        // Sequence 8: AFAreaXPositions - Variable array: int16s[$val{0}]
        CanonAfInfoTag {
            sequence: 8,
            name: "AFAreaXPositions".to_string(),
            format: CanonAfFormat::Int16sArray(CanonAfSizeExpr::ValueRef(0)),
            size_expr: CanonAfSizeExpr::ValueRef(0),
            condition: None,
            print_conv: None,
        },
        // Sequence 9: AFAreaYPositions - Variable array: int16s[$val{0}]
        CanonAfInfoTag {
            sequence: 9,
            name: "AFAreaYPositions".to_string(),
            format: CanonAfFormat::Int16sArray(CanonAfSizeExpr::ValueRef(0)),
            size_expr: CanonAfSizeExpr::ValueRef(0),
            condition: None,
            print_conv: None,
        },
        // Sequence 10: AFPointsInFocus - Variable array: int16s[int(($val{0}+15)/16)]
        CanonAfInfoTag {
            sequence: 10,
            name: "AFPointsInFocus".to_string(),
            format: CanonAfFormat::Int16sArray(CanonAfSizeExpr::CeilDiv(0, 16)),
            size_expr: CanonAfSizeExpr::CeilDiv(0, 16),
            condition: None,
            print_conv: None,
        },
        // Sequence 11: PrimaryAFPoint - conditional based on camera model
        CanonAfInfoTag {
            sequence: 11,
            name: "PrimaryAFPoint".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: Some(CanonAfCondition::ModelIsEos),
            print_conv: None,
        },
    ]
}

/// Create Canon AFInfo2 tag table for sequential processing  
/// ExifTool: Canon.pm:9055-9189 %Canon::AFInfo2
pub fn create_af_info2_table() -> Vec<CanonAfInfoTag> {
    vec![
        // Sequence 0: AFInfoSize
        CanonAfInfoTag {
            sequence: 0,
            name: "AFInfoSize".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: None,
        },
        // Sequence 1: AFAreaMode
        CanonAfInfoTag {
            sequence: 1,
            name: "AFAreaMode".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: {
                let mut conv = HashMap::new();
                conv.insert(0, "Off (Manual Focus)".to_string());
                conv.insert(1, "AF Point Expansion (surround)".to_string());
                conv.insert(2, "Single-point AF".to_string());
                Some(conv)
            },
        },
        // Sequence 2: NumAFPoints
        CanonAfInfoTag {
            sequence: 2,
            name: "NumAFPoints".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: None,
        },
        // Sequence 3: ValidAFPoints
        CanonAfInfoTag {
            sequence: 3,
            name: "ValidAFPoints".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: None,
        },
        // Sequence 4: CanonImageWidth
        CanonAfInfoTag {
            sequence: 4,
            name: "CanonImageWidth".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: None,
        },
        // Sequence 5: CanonImageHeight
        CanonAfInfoTag {
            sequence: 5,
            name: "CanonImageHeight".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: None,
        },
        // Sequence 6: AFImageWidth
        CanonAfInfoTag {
            sequence: 6,
            name: "AFImageWidth".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: None,
        },
        // Sequence 7: AFImageHeight
        CanonAfInfoTag {
            sequence: 7,
            name: "AFImageHeight".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: None,
        },
        // Sequence 8: AFAreaWidths - Variable array: int16s[$val{2}]
        CanonAfInfoTag {
            sequence: 8,
            name: "AFAreaWidths".to_string(),
            format: CanonAfFormat::Int16sArray(CanonAfSizeExpr::ValueRef(2)),
            size_expr: CanonAfSizeExpr::ValueRef(2),
            condition: None,
            print_conv: None,
        },
        // Sequence 9: AFAreaHeights - Variable array: int16s[$val{2}]
        CanonAfInfoTag {
            sequence: 9,
            name: "AFAreaHeights".to_string(),
            format: CanonAfFormat::Int16sArray(CanonAfSizeExpr::ValueRef(2)),
            size_expr: CanonAfSizeExpr::ValueRef(2),
            condition: None,
            print_conv: None,
        },
        // Sequence 10: AFAreaXPositions - Variable array: int16s[$val{2}]
        CanonAfInfoTag {
            sequence: 10,
            name: "AFAreaXPositions".to_string(),
            format: CanonAfFormat::Int16sArray(CanonAfSizeExpr::ValueRef(2)),
            size_expr: CanonAfSizeExpr::ValueRef(2),
            condition: None,
            print_conv: None,
        },
        // Sequence 11: AFAreaYPositions - Variable array: int16s[$val{2}]
        CanonAfInfoTag {
            sequence: 11,
            name: "AFAreaYPositions".to_string(),
            format: CanonAfFormat::Int16sArray(CanonAfSizeExpr::ValueRef(2)),
            size_expr: CanonAfSizeExpr::ValueRef(2),
            condition: None,
            print_conv: None,
        },
        // Sequence 12: AFPointsInFocus - Variable array: int16s[int(($val{2}+15)/16)]
        CanonAfInfoTag {
            sequence: 12,
            name: "AFPointsInFocus".to_string(),
            format: CanonAfFormat::Int16sArray(CanonAfSizeExpr::CeilDiv(2, 16)),
            size_expr: CanonAfSizeExpr::CeilDiv(2, 16),
            condition: None,
            print_conv: None,
        },
        // Sequence 13: AFPointsSelected - Variable array: int16s[int(($val{2}+15)/16)]
        CanonAfInfoTag {
            sequence: 13,
            name: "AFPointsSelected".to_string(),
            format: CanonAfFormat::Int16sArray(CanonAfSizeExpr::CeilDiv(2, 16)),
            size_expr: CanonAfSizeExpr::CeilDiv(2, 16),
            condition: Some(CanonAfCondition::ModelIsEos),
            print_conv: None,
        },
    ]
}

/// Process Canon AFInfo/AFInfo2 serial data with variable-length arrays
/// ExifTool: Canon.pm:10224-10306 ProcessSerialData implementation
///
/// This implements ExifTool's sequential data processing where:
/// - Data is processed in sequence order (0, 1, 2, ...)
/// - Array sizes are calculated from previously extracted values
/// - Position advances based on each tag's calculated size
pub fn process_serial_data(
    data: &[u8],
    offset: usize,
    size: usize,
    byte_order: ByteOrder,
    table: &[CanonAfInfoTag],
    model: &str,
) -> Result<HashMap<String, TagValue>> {
    debug!(
        "Processing Canon AF serial data: offset={:#x}, size={}, entries={}",
        offset,
        size,
        table.len()
    );

    let mut results = HashMap::new();
    let mut extracted_values: HashMap<u32, u16> = HashMap::new();
    let mut current_pos = 0;

    // Process each tag in sequence order
    // ExifTool: Canon.pm:10245-10278 sequential processing loop
    for tag in table {
        if current_pos >= size {
            debug!(
                "Reached end of data at position {}, stopping processing",
                current_pos
            );
            break;
        }

        // Check conditions
        if let Some(condition) = &tag.condition {
            match condition {
                CanonAfCondition::ModelNotEos => {
                    if model.contains("EOS") {
                        debug!("Skipping tag {} due to ModelNotEos condition", tag.name);
                        continue;
                    }
                }
                CanonAfCondition::ModelIsEos => {
                    if !model.contains("EOS") {
                        debug!("Skipping tag {} due to ModelIsEos condition", tag.name);
                        continue;
                    }
                }
            }
        }

        // Calculate array size for this tag
        let array_size = tag.size_expr.calculate_size(&extracted_values);

        match &tag.format {
            CanonAfFormat::Int16u | CanonAfFormat::Int16s => {
                // Single value extraction
                if current_pos + 2 > size {
                    debug!(
                        "Not enough data for tag {} at position {}",
                        tag.name, current_pos
                    );
                    break;
                }

                let data_offset = offset + current_pos;
                if data_offset + 2 > data.len() {
                    debug!(
                        "Data offset {:#x} beyond buffer bounds for tag {}",
                        data_offset, tag.name
                    );
                    break;
                }

                let raw_value = byte_order.read_u16(data, data_offset)?;
                let value = match tag.format {
                    CanonAfFormat::Int16s => TagValue::I16(raw_value as i16),
                    _ => TagValue::U16(raw_value),
                };

                // Store extracted value for later reference
                extracted_values.insert(tag.sequence, raw_value);

                // Apply PrintConv if available
                let final_value = if let Some(print_conv) = &tag.print_conv {
                    if let Some(converted) = print_conv.get(&raw_value) {
                        TagValue::String(converted.clone())
                    } else {
                        value
                    }
                } else {
                    value
                };

                debug!(
                    "Extracted {} = {:?} (raw: {}) at sequence {} position {}",
                    tag.name, final_value, raw_value, tag.sequence, current_pos
                );

                results.insert(format!("MakerNotes:{}", tag.name), final_value);
                current_pos += 2;
            }
            CanonAfFormat::Int16sArray(_) => {
                // Variable array extraction
                let total_bytes = array_size * 2; // int16s = 2 bytes each
                if current_pos + total_bytes > size {
                    debug!(
                        "Not enough data for array tag {} (need {} bytes, have {})",
                        tag.name,
                        total_bytes,
                        size - current_pos
                    );
                    break;
                }

                let mut array_values = Vec::new();
                for i in 0..array_size {
                    let data_offset = offset + current_pos + (i * 2);
                    if data_offset + 2 > data.len() {
                        debug!(
                            "Array element {} beyond buffer bounds for tag {}",
                            i, tag.name
                        );
                        break;
                    }

                    let raw_value = byte_order.read_u16(data, data_offset)? as i16;
                    array_values.push(TagValue::I16(raw_value));
                }

                debug!(
                    "Extracted array {} with {} elements at sequence {} position {}",
                    tag.name,
                    array_values.len(),
                    tag.sequence,
                    current_pos
                );

                // Format array as space-separated string for compatibility with ExifTool
                let array_string = array_values
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(" ");

                results.insert(
                    format!("MakerNotes:{}", tag.name),
                    TagValue::String(array_string),
                );
                current_pos += total_bytes;
            }
        }
    }

    Ok(results)
}
