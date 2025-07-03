//! Canon-specific EXIF processing coordinator
//!
//! This module coordinates Canon manufacturer-specific processing,
//! dispatching to specialized sub-modules for different aspects.

pub mod binary_data;

// Re-export binary data processing functions for backwards compatibility
pub use binary_data::{
    create_camera_settings_table, extract_camera_settings, CanonCameraSettingsTag,
};

// Temporary: Re-export everything from the original canon.rs file
// This will be updated as we extract more modules

use crate::tiff_types::ByteOrder;
use crate::types::{Result, TagValue};
use std::collections::HashMap;

/// Canon offset schemes based on camera model
/// ExifTool: lib/Image/ExifTool/MakerNotes.pm:1135-1141 GetMakerNoteOffset
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanonOffsetScheme {
    /// Default Canon offset: 4 bytes after IFD end
    /// ExifTool: MakerNotes.pm:1136 "4" default
    FourByte = 4,
    /// Special models: 20D, 350D, REBEL XT, Kiss Digital N: 6 bytes
    /// ExifTool: MakerNotes.pm:1136 "($model =~ /\b(20D|350D|REBEL XT|Kiss Digital N)\b/) ? 6"
    SixByte = 6,
    /// PowerShot/IXUS/IXY models: 16 bytes (12 unused bytes)
    /// ExifTool: MakerNotes.pm:1140-1141 "push @offsets, 16 if $model =~ /(PowerShot|IXUS|IXY)/"
    SixteenByte = 16,
    /// Video models FV-M30, Optura series: 28 bytes (24 unused bytes, 2 spare IFD entries?)
    /// ExifTool: MakerNotes.pm:1137-1139 "push @offsets, 28 if $model =~ /\b(FV\b|OPTURA)/"
    TwentyEightByte = 28,
}

impl CanonOffsetScheme {
    /// Get the offset value in bytes
    pub fn as_bytes(self) -> u32 {
        self as u32
    }
}

/// Detect Canon MakerNote signature
/// ExifTool: lib/Image/ExifTool/MakerNotes.pm:60-68 MakerNoteCanon condition
///
/// Canon MakerNotes are detected by:
/// - Condition: '$$self{Make} =~ /^Canon/' (Make field starts with "Canon")
/// - No header signature pattern (unlike Nikon which has "Nikon\x00\x02")
/// - Starts with a standard IFD
pub fn detect_canon_signature(make: &str) -> bool {
    // ExifTool: MakerNotes.pm:63 '$$self{Make} =~ /^Canon/'
    make.starts_with("Canon")
}

// TODO: Include other functions from canon.rs that haven't been extracted yet
// This will be updated as we progress through the phases

// Placeholder implementations - these will be moved to appropriate modules:

/// Detect Canon offset scheme based on camera model
/// ExifTool: lib/Image/ExifTool/MakerNotes.pm:1135-1141 GetMakerNoteOffset
pub fn detect_offset_scheme(model: &str) -> CanonOffsetScheme {
    // ExifTool: MakerNotes.pm:1136 "($model =~ /\b(20D|350D|REBEL XT|Kiss Digital N)\b/) ? 6"
    if model.contains("20D")
        || model.contains("350D")
        || model.contains("REBEL XT")
        || model.contains("Kiss Digital N")
    {
        return CanonOffsetScheme::SixByte;
    }

    // ExifTool: MakerNotes.pm:1137-1139 "push @offsets, 28 if $model =~ /\b(FV\b|OPTURA)/"
    if model.contains("FV") || model.contains("OPTURA") {
        return CanonOffsetScheme::TwentyEightByte;
    }

    // ExifTool: MakerNotes.pm:1140-1141 "push @offsets, 16 if $model =~ /(PowerShot|IXUS|IXY)/"
    if model.contains("PowerShot") || model.contains("IXUS") || model.contains("IXY") {
        return CanonOffsetScheme::SixteenByte;
    }

    // ExifTool: MakerNotes.pm:1136 "4" default
    CanonOffsetScheme::FourByte
}

// Supporting types for AF Info processing (will be moved to af_info.rs in Phase 3)
#[derive(Debug, Clone)]
pub enum CanonAfFormat {
    /// Fixed 16-bit unsigned integer
    Int16u,
    /// Fixed 16-bit signed integer
    Int16s,
    /// Variable array of 16-bit signed integers with dynamic count
    Int16sArray(CanonAfSizeExpr),
}

#[derive(Debug, Clone)]
pub enum CanonAfSizeExpr {
    /// Fixed count
    Fixed(usize),
    /// Reference to previously extracted value: $val{N}
    ValueRef(u32),
    /// Ceiling division: int(($val{N}+15)/16) for bit packing
    CeilDiv(u32, u32), // (value_ref, divisor)
}

#[derive(Debug, Clone)]
pub enum CanonAfCondition {
    /// Model-based condition: $$self{Model} !~ /EOS/
    ModelNotEos,
    /// Model-based condition: $$self{Model} =~ /EOS/
    ModelIsEos,
}

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

/// Placeholder for fix_maker_note_base function
/// TODO: This will be moved to offset_fixing.rs in Phase 4
#[allow(clippy::too_many_arguments)]
pub fn fix_maker_note_base(
    _make: &str,
    _model: &str,
    _maker_note_data: &[u8],
    _dir_start: usize,
    _dir_len: usize,
    _data_pos: u64,
    _byte_order: ByteOrder,
    _val_ptrs: &[usize],
    _val_block: &HashMap<usize, usize>,
) -> Result<Option<i64>> {
    // Placeholder implementation
    Ok(None)
}

/// Placeholder for AF Info table creation
/// TODO: This will be moved to af_info.rs in Phase 3
pub fn create_af_info_table() -> Vec<CanonAfInfoTag> {
    Vec::new()
}

/// Placeholder for AF Info2 table creation
/// TODO: This will be moved to af_info.rs in Phase 3
pub fn create_af_info2_table() -> Vec<CanonAfInfoTag> {
    Vec::new()
}

/// Placeholder for process_serial_data function
/// TODO: This will be moved to af_info.rs in Phase 3
pub fn process_serial_data(
    _data: &[u8],
    _offset: usize,
    _size: usize,
    _byte_order: ByteOrder,
    _table: &[CanonAfInfoTag],
    _model: &str,
) -> Result<HashMap<String, TagValue>> {
    Ok(HashMap::new())
}

// ===== EXTRACTED CANON METHODS FROM src/exif.rs =====

use crate::types::{
    BinaryDataFormat, BinaryDataTable, BinaryDataTag, DirectoryInfo, ExifError, ProcessorType,
    TagSourceInfo,
};
use tracing::{debug, warn};

/// Process Canon MakerNotes data with comprehensive offset fixing and tag extraction
/// ExifTool: Canon.pm processing + MakerNotes.pm offset fixing + ProcessSerialData
pub fn process_canon_makernotes(
    reader: &mut crate::exif::ExifReader,
    start_offset: usize,
    size: usize,
) -> Result<()> {
    // Check minimum size for Canon MakerNotes
    if size < 12 {
        debug!("MakerNotes too small for Canon processing (need at least 12 bytes)");
        return Ok(());
    }

    debug!(
        "Processing Canon MakerNotes at offset {:#x}, size {}",
        start_offset, size
    );

    // Get Make and Model for offset scheme detection
    let make = reader
        .get_extracted_tags()
        .get(&0x010F)
        .and_then(|v| v.as_string())
        .unwrap_or("")
        .to_string();

    let model = reader
        .get_extracted_tags()
        .get(&0x0110)
        .and_then(|v| v.as_string())
        .unwrap_or("")
        .to_string();

    let byte_order = reader.get_header().unwrap().byte_order;

    debug!(
        "Canon MakerNote processing for Make: '{}', Model: '{}'",
        make, model
    );

    // Detect Canon offset scheme based on model
    let offset_scheme = detect_offset_scheme(&model);
    debug!("Detected Canon offset scheme: {:?}", offset_scheme);

    // Canon offset fixing with footer validation
    // ExifTool: lib/Image/ExifTool/MakerNotes.pm:1281-1307 FixBase Canon section
    let data_pos = 0; // MakerNotes data position in file
    let dir_start = 0; // Directory starts at beginning of MakerNotes data
    let dir_len = size; // Directory length is the MakerNotes size

    // For simplified implementation, we'll skip full val_ptrs/val_block analysis
    // and use empty collections. This allows basic footer detection and validation.
    let val_ptrs: Vec<usize> = Vec::new();
    let val_block: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

    // Get data slice for Canon processing
    let data = reader.get_data();
    let canon_data = &data[start_offset..start_offset + size];

    // Attempt Canon offset fixing with footer validation
    let offset_adjustment = match fix_maker_note_base(
        &make, &model, canon_data, dir_start, dir_len, data_pos, byte_order, &val_ptrs, &val_block,
    ) {
        Ok(Some(offset_fix)) => {
            debug!("Canon offset base adjustment calculated: {}", offset_fix);
            offset_fix
        }
        Ok(None) => {
            debug!("Canon offset fixing: no adjustment needed");
            0
        }
        Err(e) => {
            debug!(
                "Canon offset fixing failed: {}, using offset scheme default",
                e
            );
            // Fall back to offset scheme
            offset_scheme.as_bytes() as i64
        }
    };

    // Parse Canon MakerNotes IFD with offset adjustment
    parse_canon_makernote_ifd(
        reader,
        start_offset,
        size,
        offset_adjustment,
        byte_order,
        &model,
    )?;

    Ok(())
}

/// Parse Canon MakerNote IFD and extract all supported Canon tags
/// ExifTool: Canon.pm Main table processing
pub fn parse_canon_makernote_ifd(
    reader: &mut crate::exif::ExifReader,
    start_offset: usize,
    _size: usize,
    offset_adjustment: i64,
    byte_order: ByteOrder,
    model: &str,
) -> Result<()> {
    // First, collect all the entry data without holding a reference to reader
    let entries = {
        let data = reader.get_data();
        if start_offset + 2 > data.len() {
            return Err(ExifError::ParseError(
                "Not enough data for Canon MakerNotes IFD".to_string(),
            ));
        }

        // Read number of IFD entries
        let num_entries = byte_order.read_u16(data, start_offset)? as usize;
        debug!("Canon MakerNotes IFD has {} entries", num_entries);

        if num_entries > 256 {
            return Err(ExifError::ParseError(format!(
                "Invalid Canon MakerNotes entry count: {num_entries}"
            )));
        }

        // Collect entry data
        let mut entries = Vec::new();
        for i in 0..num_entries {
            let entry_offset = start_offset + 2 + (i * 12);
            if entry_offset + 12 > data.len() {
                debug!("Canon MakerNote entry {} beyond data bounds", i);
                break;
            }

            let tag_id = byte_order.read_u16(data, entry_offset)?;
            let format = byte_order.read_u16(data, entry_offset + 2)?;
            let count = byte_order.read_u32(data, entry_offset + 4)?;
            let value_offset = byte_order.read_u32(data, entry_offset + 8)?;

            debug!(
                "Canon tag {:#04x}: format={}, count={}, value_offset={:#x}",
                tag_id, format, count, value_offset
            );

            // Calculate adjusted offset for Canon values
            let format_size = format_size(format)? as u32;
            let adjusted_offset = if count * format_size <= 4 {
                // Inline value (4 bytes or less)
                entry_offset + 8
            } else {
                // External value - apply offset adjustment
                (value_offset as i64 + offset_adjustment) as usize + start_offset
            };

            entries.push((tag_id, format, count, adjusted_offset));
        }

        entries
    };

    // Now process the entries using the reader
    for (tag_id, format, count, adjusted_offset) in entries {
        // Process Canon-specific tags
        match tag_id {
            0x0001 => {
                // Canon CameraSettings (ProcessBinaryData)
                debug!("Processing Canon CameraSettings tag");
                process_canon_camera_settings(reader, adjusted_offset, count as usize, byte_order)?;
            }
            0x0012 => {
                // Canon AFInfo (ProcessSerialData)
                debug!("Processing Canon AFInfo tag");
                process_canon_af_info(reader, adjusted_offset, count as usize, byte_order, model)?;
            }
            0x0026 => {
                // Canon AFInfo2 (ProcessSerialData)
                debug!("Processing Canon AFInfo2 tag");
                process_canon_af_info2(reader, adjusted_offset, count as usize, byte_order, model)?;
            }
            _ => {
                // Other Canon tags - basic extraction for now
                if let Ok(tag_value) = extract_basic_canon_tag(
                    reader,
                    tag_id,
                    format,
                    count,
                    adjusted_offset,
                    byte_order,
                ) {
                    let source_info = TagSourceInfo::new(
                        "MakerNotes".to_string(),
                        "Canon::Main".to_string(),
                        ProcessorType::Canon(crate::types::CanonProcessor::Main),
                    );
                    // Use wrapping_add to prevent overflow for large Canon tag IDs
                    let synthetic_tag_id = 0xC000u16.wrapping_add(tag_id);
                    reader.store_tag_with_precedence(synthetic_tag_id, tag_value, source_info);
                }
            }
        }
    }

    Ok(())
}

/// Process Canon CameraSettings using ProcessBinaryData
/// ExifTool: Canon.pm CameraSettings table
pub fn process_canon_camera_settings(
    reader: &mut crate::exif::ExifReader,
    offset: usize,
    count: usize,
    byte_order: ByteOrder,
) -> Result<()> {
    let size = count * 2; // int16s format = 2 bytes each
    let data = reader.get_data();
    if offset + size > data.len() {
        debug!("Canon CameraSettings data beyond buffer bounds");
        return Ok(());
    }

    match extract_camera_settings(data, offset, size, byte_order) {
        Ok(canon_tags) => {
            debug!(
                "Successfully extracted {} Canon CameraSettings tags",
                canon_tags.len()
            );

            // Add Canon tags with synthetic tag IDs
            for (tag_name, tag_value) in canon_tags {
                let synthetic_tag_id = match tag_name.as_str() {
                    "MakerNotes:MacroMode" => 0xC001,
                    "MakerNotes:SelfTimer" => 0xC002,
                    "MakerNotes:Quality" => 0xC003,
                    "MakerNotes:CanonFlashMode" => 0xC004,
                    "MakerNotes:ContinuousDrive" => 0xC005,
                    "MakerNotes:FocusMode" => 0xC007,
                    _ => continue,
                };

                let source_info = TagSourceInfo::new(
                    "MakerNotes".to_string(),
                    "Canon::CameraSettings".to_string(),
                    ProcessorType::Canon(crate::types::CanonProcessor::CameraSettings),
                );
                reader.store_tag_with_precedence(synthetic_tag_id, tag_value, source_info);
            }
            Ok(())
        }
        Err(e) => {
            warn!("Failed to extract Canon CameraSettings: {}", e);
            Ok(())
        }
    }
}

/// Process Canon AFInfo using ProcessSerialData
/// ExifTool: Canon.pm AFInfo table
pub fn process_canon_af_info(
    reader: &mut crate::exif::ExifReader,
    offset: usize,
    count: usize,
    byte_order: ByteOrder,
    model: &str,
) -> Result<()> {
    let data = reader.get_data();
    if offset + count > data.len() {
        debug!("Canon AFInfo data beyond buffer bounds");
        return Ok(());
    }

    let af_info_table = create_af_info_table();
    match process_serial_data(data, offset, count, byte_order, &af_info_table, model) {
        Ok(af_tags) => {
            debug!("Successfully extracted {} Canon AFInfo tags", af_tags.len());

            // Add AF tags with synthetic tag IDs
            for (tag_name, tag_value) in af_tags {
                let synthetic_tag_id = match tag_name.as_str() {
                    "MakerNotes:NumAFPoints" => 0xC020,
                    "MakerNotes:ValidAFPoints" => 0xC021,
                    "MakerNotes:CanonImageWidth" => 0xC022,
                    "MakerNotes:CanonImageHeight" => 0xC023,
                    "MakerNotes:AFImageWidth" => 0xC024,
                    "MakerNotes:AFImageHeight" => 0xC025,
                    "MakerNotes:AFAreaWidth" => 0xC026,
                    "MakerNotes:AFAreaHeight" => 0xC027,
                    "MakerNotes:AFAreaXPositions" => 0xC028,
                    "MakerNotes:AFAreaYPositions" => 0xC029,
                    "MakerNotes:AFPointsInFocus" => 0xC02A,
                    "MakerNotes:PrimaryAFPoint" => 0xC02B,
                    _ => continue,
                };

                let source_info = TagSourceInfo::new(
                    "MakerNotes".to_string(),
                    "Canon::AFInfo".to_string(),
                    ProcessorType::Canon(crate::types::CanonProcessor::AfInfo),
                );
                reader.store_tag_with_precedence(synthetic_tag_id, tag_value, source_info);
            }
            Ok(())
        }
        Err(e) => {
            warn!("Failed to extract Canon AFInfo: {}", e);
            Ok(())
        }
    }
}

/// Process Canon AFInfo2 using ProcessSerialData
/// ExifTool: Canon.pm AFInfo2 table
pub fn process_canon_af_info2(
    reader: &mut crate::exif::ExifReader,
    offset: usize,
    count: usize,
    byte_order: ByteOrder,
    model: &str,
) -> Result<()> {
    let data = reader.get_data();
    if offset + count > data.len() {
        debug!("Canon AFInfo2 data beyond buffer bounds");
        return Ok(());
    }

    let af_info2_table = create_af_info2_table();
    match process_serial_data(data, offset, count, byte_order, &af_info2_table, model) {
        Ok(af_tags) => {
            debug!(
                "Successfully extracted {} Canon AFInfo2 tags",
                af_tags.len()
            );

            // Add AF2 tags with synthetic tag IDs
            for (tag_name, tag_value) in af_tags {
                let synthetic_tag_id = match tag_name.as_str() {
                    "MakerNotes:AFInfoSize" => 0xC030,
                    "MakerNotes:AFAreaMode" => 0xC031,
                    "MakerNotes:NumAFPoints" => 0xC032,
                    "MakerNotes:ValidAFPoints" => 0xC033,
                    "MakerNotes:CanonImageWidth" => 0xC034,
                    "MakerNotes:CanonImageHeight" => 0xC035,
                    "MakerNotes:AFImageWidth" => 0xC036,
                    "MakerNotes:AFImageHeight" => 0xC037,
                    "MakerNotes:AFAreaWidths" => 0xC038,
                    "MakerNotes:AFAreaHeights" => 0xC039,
                    "MakerNotes:AFAreaXPositions" => 0xC03A,
                    "MakerNotes:AFAreaYPositions" => 0xC03B,
                    "MakerNotes:AFPointsInFocus" => 0xC03C,
                    "MakerNotes:AFPointsSelected" => 0xC03D,
                    _ => continue,
                };

                let source_info = TagSourceInfo::new(
                    "MakerNotes".to_string(),
                    "Canon::AFInfo2".to_string(),
                    ProcessorType::Canon(crate::types::CanonProcessor::AfInfo2),
                );
                reader.store_tag_with_precedence(synthetic_tag_id, tag_value, source_info);
            }
            Ok(())
        }
        Err(e) => {
            warn!("Failed to extract Canon AFInfo2: {}", e);
            Ok(())
        }
    }
}

/// Extract basic Canon tag value (for tags not yet fully implemented)
pub fn extract_basic_canon_tag(
    reader: &crate::exif::ExifReader,
    tag_id: u16,
    format: u16,
    count: u32,
    offset: usize,
    byte_order: ByteOrder,
) -> Result<TagValue> {
    let format_size = format_size(format)?;
    let total_size = count as usize * format_size;
    let data = reader.get_data();

    if offset + total_size > data.len() {
        return Err(ExifError::ParseError(format!(
            "Canon tag {tag_id:#04x} data beyond bounds"
        )));
    }

    // Extract basic value based on format
    match format {
        2 => {
            // ASCII string
            let string_data = &data[offset..offset + total_size];
            let string = String::from_utf8_lossy(string_data)
                .trim_end_matches('\0')
                .to_string();
            Ok(TagValue::String(string))
        }
        3 => {
            // int16u
            let value = byte_order.read_u16(data, offset)?;
            Ok(TagValue::U16(value))
        }
        4 => {
            // int32u
            let value = byte_order.read_u32(data, offset)?;
            Ok(TagValue::U32(value))
        }
        _ => {
            // Other formats - return as raw bytes for now
            let raw_data = data[offset..offset + total_size].to_vec();
            Ok(TagValue::String(format!(
                "(Binary data {} bytes)",
                raw_data.len()
            )))
        }
    }
}

/// Get format size in bytes
pub fn format_size(format: u16) -> Result<usize> {
    match format {
        1 | 2 | 6 | 7 => Ok(1), // int8u, string, int8s, undef
        3 | 8 => Ok(2),         // int16u, int16s
        4 | 9 | 11 => Ok(4),    // int32u, int32s, float
        5 | 10 | 12 => Ok(8),   // rational64u, rational64s, double
        _ => Err(ExifError::ParseError(format!("Invalid format: {format}"))),
    }
}

/// Find Canon CameraSettings tag (0x0001) in MakerNotes IFD
/// ExifTool: Canon.pm Main table tag 0x1
pub fn find_canon_camera_settings_tag(
    reader: &crate::exif::ExifReader,
    start_offset: usize,
    _size: usize,
) -> Result<usize> {
    let data = reader.get_data();
    if start_offset + 14 > data.len() {
        return Err(ExifError::ParseError(
            "Not enough data for Canon MakerNotes IFD".to_string(),
        ));
    }

    let byte_order = reader
        .get_header()
        .map(|h| h.byte_order)
        .unwrap_or(ByteOrder::LittleEndian);

    // Read number of IFD entries
    let num_entries = byte_order.read_u16(data, start_offset)? as usize;
    debug!("Canon MakerNotes IFD has {} entries", num_entries);

    if num_entries == 0 || num_entries > 100 {
        return Err(ExifError::ParseError(format!(
            "Invalid Canon MakerNotes entry count: {num_entries}"
        )));
    }

    // Search for tag 0x0001 (CanonCameraSettings)
    for i in 0..num_entries {
        let entry_offset = start_offset + 2 + (i * 12);
        if entry_offset + 12 > data.len() {
            break;
        }

        let tag_id = byte_order.read_u16(data, entry_offset)?;
        if tag_id == 0x0001 {
            // Found Canon CameraSettings tag
            let format = byte_order.read_u16(data, entry_offset + 2)?;
            let count = byte_order.read_u32(data, entry_offset + 4)?;
            let value_offset = byte_order.read_u32(data, entry_offset + 8)?;

            debug!(
                "Canon CameraSettings: format={}, count={}, offset={:#x}",
                format, count, value_offset
            );

            // Calculate absolute offset for CameraSettings data
            // For Canon, the value_offset is relative to the start of the MakerNotes
            let camera_settings_offset = if count * 2 <= 4 {
                // Data is inline in the offset field
                entry_offset + 8
            } else {
                // Data is at offset
                start_offset + value_offset as usize
            };

            if camera_settings_offset < data.len() {
                return Ok(camera_settings_offset);
            }
        }
    }

    Err(ExifError::ParseError(
        "Canon CameraSettings tag (0x0001) not found".to_string(),
    ))
}

/// Create Canon CameraSettings binary data table
/// ExifTool: Canon.pm %Canon::CameraSettings table
pub fn create_canon_camera_settings_table() -> BinaryDataTable {
    use std::collections::HashMap;

    let mut table = BinaryDataTable {
        default_format: BinaryDataFormat::Int16s,
        first_entry: Some(1),
        groups: {
            let mut groups = HashMap::new();
            groups.insert(0, "MakerNotes".to_string());
            groups.insert(2, "Camera".to_string());
            groups
        },
        tags: HashMap::new(),
    };

    // Add MacroMode tag at index 1
    table.tags.insert(
        1,
        BinaryDataTag {
            name: "MacroMode".to_string(),
            format: None, // Uses table default (int16s)
            mask: None,
            print_conv: {
                let mut conv = HashMap::new();
                conv.insert(1, "Macro".to_string());
                conv.insert(2, "Normal".to_string());
                Some(conv)
            },
        },
    );

    // Add FocusMode tag at index 7
    table.tags.insert(
        7,
        BinaryDataTag {
            name: "FocusMode".to_string(),
            format: None, // Uses table default (int16s)
            mask: None,
            print_conv: {
                let mut conv = HashMap::new();
                conv.insert(0, "One-shot AF".to_string());
                conv.insert(1, "AI Servo AF".to_string());
                conv.insert(2, "AI Focus AF".to_string());
                conv.insert(3, "Manual Focus (3)".to_string());
                conv.insert(4, "Single".to_string());
                conv.insert(5, "Continuous".to_string());
                conv.insert(6, "Manual Focus (6)".to_string());
                Some(conv)
            },
        },
    );

    table
}

/// Extract binary data tags using table definition
/// ExifTool: ProcessBinaryData main processing loop
pub fn extract_binary_data_tags(
    reader: &mut crate::exif::ExifReader,
    start_offset: usize,
    size: usize,
    table: &BinaryDataTable,
) -> Result<()> {
    let increment = table.default_format.byte_size();

    debug!(
        "Extracting binary data tags: start={:#x}, size={}, increment={}, format={:?}",
        start_offset, size, increment, table.default_format
    );

    // Process defined tags
    for (&index, tag_def) in &table.tags {
        let entry_offset = (index as usize) * increment;
        if entry_offset + increment > size {
            debug!("Tag {} at index {} beyond data bounds", tag_def.name, index);
            continue;
        }

        let data_offset = start_offset + entry_offset;
        let format = tag_def.format.unwrap_or(table.default_format);

        // Extract value based on format
        if let Ok(value) = extract_binary_value(reader, data_offset, format, 1) {
            debug!(
                "Extracted {} = {:?} at index {}",
                tag_def.name, value, index
            );

            // Apply PrintConv if available
            let final_value = if let Some(print_conv) = &tag_def.print_conv {
                // Try to get the value as u32 for lookup
                let lookup_val = match value {
                    TagValue::U8(v) => Some(v as u32),
                    TagValue::U16(v) => Some(v as u32),
                    TagValue::U32(v) => Some(v),
                    TagValue::I16(v) => Some(v as u32),
                    TagValue::I32(v) => Some(v as u32),
                    _ => None,
                };

                if let Some(int_val) = lookup_val {
                    if let Some(converted) = print_conv.get(&int_val) {
                        TagValue::String(converted.clone())
                    } else {
                        value
                    }
                } else {
                    value
                }
            } else {
                value
            };

            // Store with group prefix
            let unknown_default = "Unknown".to_string();
            let group_prefix = table.groups.get(&0).unwrap_or(&unknown_default);
            let source_info = TagSourceInfo::new(
                group_prefix.clone(),
                format!("BinaryData/{group_prefix}"),
                ProcessorType::BinaryData,
            );
            reader.store_tag_with_precedence(index as u16, final_value, source_info);
        }
    }

    Ok(())
}

/// Extract a single binary value from data
/// ExifTool: Value extraction with format-specific handling
pub fn extract_binary_value(
    reader: &crate::exif::ExifReader,
    offset: usize,
    format: BinaryDataFormat,
    count: usize,
) -> Result<TagValue> {
    let data = reader.get_data();
    if offset >= data.len() {
        return Err(ExifError::ParseError(
            "Offset beyond data bounds".to_string(),
        ));
    }

    let byte_order = reader
        .get_header()
        .map(|h| h.byte_order)
        .unwrap_or(ByteOrder::LittleEndian);

    match format {
        BinaryDataFormat::Int8u => Ok(TagValue::U8(data[offset])),
        BinaryDataFormat::Int8s => Ok(TagValue::I16(data[offset] as i8 as i16)),
        BinaryDataFormat::Int16u => {
            if offset + 2 > data.len() {
                return Err(ExifError::ParseError(
                    "Not enough data for int16u".to_string(),
                ));
            }
            let value = byte_order.read_u16(data, offset)?;
            Ok(TagValue::U16(value))
        }
        BinaryDataFormat::Int16s => {
            if offset + 2 > data.len() {
                return Err(ExifError::ParseError(
                    "Not enough data for int16s".to_string(),
                ));
            }
            let value = byte_order.read_u16(data, offset)? as i16;
            Ok(TagValue::I16(value))
        }
        BinaryDataFormat::Int32u => {
            if offset + 4 > data.len() {
                return Err(ExifError::ParseError(
                    "Not enough data for int32u".to_string(),
                ));
            }
            let value = byte_order.read_u32(data, offset)?;
            Ok(TagValue::U32(value))
        }
        BinaryDataFormat::Int32s => {
            if offset + 4 > data.len() {
                return Err(ExifError::ParseError(
                    "Not enough data for int32s".to_string(),
                ));
            }
            let value = byte_order.read_u32(data, offset)? as i32;
            Ok(TagValue::I32(value))
        }
        BinaryDataFormat::String => {
            let remaining = data.len() - offset;
            let max_len = if count > 0 {
                count.min(remaining)
            } else {
                remaining
            };

            // Find null terminator or use max length
            let end = data[offset..offset + max_len]
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(max_len);

            let bytes = &data[offset..offset + end];
            match std::str::from_utf8(bytes) {
                Ok(s) => Ok(TagValue::String(s.to_string())),
                Err(_) => Ok(TagValue::Binary(bytes.to_vec())),
            }
        }
        BinaryDataFormat::PString => {
            if offset >= data.len() {
                return Err(ExifError::ParseError(
                    "Not enough data for pstring length".to_string(),
                ));
            }
            let len = data[offset] as usize;
            if offset + 1 + len > data.len() {
                return Err(ExifError::ParseError(
                    "Not enough data for pstring content".to_string(),
                ));
            }
            let bytes = &data[offset + 1..offset + 1 + len];
            match std::str::from_utf8(bytes) {
                Ok(s) => Ok(TagValue::String(s.to_string())),
                Err(_) => Ok(TagValue::Binary(bytes.to_vec())),
            }
        }
        _ => {
            debug!("Unsupported binary format: {:?}", format);
            Ok(TagValue::Binary(vec![0]))
        }
    }
}

/// Get Canon-specific tag name for synthetic tag IDs
/// Maps synthetic Canon tag IDs back to their proper names
pub fn get_canon_tag_name(tag_id: u16) -> Option<String> {
    match tag_id {
        0xC001 => Some("MacroMode".to_string()),
        0xC002 => Some("SelfTimer".to_string()),
        0xC003 => Some("Quality".to_string()),
        0xC004 => Some("CanonFlashMode".to_string()),
        0xC005 => Some("ContinuousDrive".to_string()),
        0xC007 => Some("FocusMode".to_string()),
        _ => None,
    }
}

/// Process Canon manufacturer-specific data
/// ExifTool: Canon.pm processing procedures
pub fn process_canon(
    reader: &mut crate::exif::ExifReader,
    _canon_proc: crate::types::CanonProcessor,
    dir_info: &DirectoryInfo,
) -> Result<()> {
    // TODO: Implement Canon-specific processing for different CanonProcessor types
    // This will be expanded in future milestones for ProcessSerialData, etc.
    debug!("Canon processing not yet implemented for {}", dir_info.name);
    reader.process_exif_ifd(dir_info.dir_start, &dir_info.name)
}
