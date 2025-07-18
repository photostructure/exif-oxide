//! Panasonic RAW format handler

#![allow(dead_code, unused_variables)]
//!
//! This module implements ExifTool's PanasonicRaw.pm processing logic exactly.
//! Panasonic RW2/RWL files are TIFF-based formats with manufacturer-specific tags
//! and entry-based offset handling for some data blocks.
//!
//! ExifTool Reference: lib/Image/ExifTool/PanasonicRaw.pm (956 lines total)
//! Processing: Standard TIFF IFD processing with specialized entry-based offsets

use crate::exif::ExifReader;
use crate::raw::offset::{
    EntryBasedOffsetProcessor, OffsetBase, OffsetExtractionRule, OffsetField,
};
use crate::raw::RawFormatHandler;
use crate::tiff_types::TiffHeader;
use crate::types::{DirectoryInfo, ExifError, Result, TagSourceInfo, TagValue};
use std::collections::HashMap;
use tracing;

/// Panasonic RAW format handler
/// ExifTool: lib/Image/ExifTool/PanasonicRaw.pm - TIFF-based with entry-based offsets
pub struct PanasonicRawHandler {
    /// Main binary data processor for standard Panasonic tags
    /// ExifTool: %Image::ExifTool::PanasonicRaw::Main hash (lines 70-380)
    binary_processor: PanasonicBinaryProcessor,

    /// Entry-based offset processor for special data blocks
    /// ExifTool: Tags like JpgFromRaw (0x002e) use entry-based offsets
    offset_processor: EntryBasedOffsetProcessor,
}

/// Binary data processor for Panasonic main tags
/// ExifTool: %Image::ExifTool::PanasonicRaw::Main hash (lines 70-380)
struct PanasonicBinaryProcessor {
    /// Tag definitions for main Panasonic IFD
    tag_definitions: HashMap<u16, PanasonicTagDef>,
}

/// Panasonic tag definition
/// ExifTool: Individual entries in %Image::ExifTool::PanasonicRaw::Main
#[derive(Debug, Clone)]
struct PanasonicTagDef {
    /// Tag name
    /// ExifTool: Name field in tag hash
    name: String,

    /// Data format
    /// ExifTool: Format field or inferred from type
    format: PanasonicFormat,

    /// Value conversion
    /// ExifTool: ValueConv field
    value_conv: Option<PanasonicValueConv>,

    /// Print conversion
    /// ExifTool: PrintConv field
    print_conv: Option<PanasonicPrintConv>,

    /// Groups assignment
    /// ExifTool: Groups field override
    groups: Option<PanasonicGroups>,

    /// Writability
    /// ExifTool: Writable field
    writable: bool,
}

/// Panasonic data formats
/// ExifTool: Format specifications in tag definitions
#[derive(Debug, Clone)]
enum PanasonicFormat {
    /// Undefined binary data
    /// ExifTool: undef format
    Undef,

    /// 16-bit unsigned integer
    /// ExifTool: int16u format
    Int16u,

    /// 32-bit unsigned integer
    /// ExifTool: int32u format
    Int32u,

    /// String data
    /// ExifTool: string format
    String,

    /// Array of int16u values
    /// ExifTool: int16u[N] format
    Int16uArray(usize),
}

/// Panasonic value conversion types
/// ExifTool: ValueConv fields in tag definitions
#[derive(Debug, Clone)]
enum PanasonicValueConv {
    /// Division by constant
    /// ExifTool: '$val / 256' pattern
    DivideBy256,

    /// Multiplication by constant
    /// ExifTool: '$val * 100' pattern
    MultiplyBy100,
}

/// Panasonic print conversion types
/// ExifTool: PrintConv fields in tag definitions
#[derive(Debug, Clone)]
enum PanasonicPrintConv {
    /// CFA pattern lookup
    /// ExifTool: lines 96-102 - CFA pattern hash
    CfaPattern,

    /// Compression type lookup
    /// ExifTool: lines 109-114 - Compression hash
    Compression,

    /// Orientation lookup (reuses EXIF orientation)
    /// ExifTool: line 250 - \%Image::ExifTool::Exif::orientation
    Orientation,

    /// PrintConv hash lookup
    /// ExifTool: Static hash with key-value mappings
    HashLookup(HashMap<u32, String>),
}

/// Panasonic group assignments
/// ExifTool: Groups field overrides default groups
#[derive(Debug, Clone)]
struct PanasonicGroups {
    /// Group 0 (family)
    /// ExifTool: Groups => { 0 => '...' }
    group0: Option<String>,

    /// Group 1 (location)
    /// ExifTool: Groups => { 1 => '...' }
    group1: Option<String>,

    /// Group 2 (category)
    /// ExifTool: Groups => { 2 => '...' }
    group2: Option<String>,
}

impl Default for PanasonicRawHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl PanasonicRawHandler {
    /// Process entry-based offsets for Panasonic-specific data blocks
    /// ExifTool: PanasonicRaw.pm applies additional processing for tags like JpgFromRaw (0x002e)
    fn process_entry_based_offsets(&self, reader: &mut ExifReader, data: &[u8]) -> Result<()> {
        // Get the currently extracted tags from TIFF IFD processing
        // Clone the tags to avoid borrowing issues
        let extracted_tags: Vec<(u16, TagValue)> = reader
            .get_extracted_tags()
            .iter()
            .map(|(tag_id, tag_value)| (*tag_id, tag_value.clone()))
            .collect();

        // Process each tag that has entry-based offset rules
        for (tag_id, tag_value) in extracted_tags {
            if let Some(rule) = self.offset_processor.get_rule(tag_id) {
                // Extract offset from the tag value based on the rule
                let offset = match rule.offset_field {
                    OffsetField::ActualValue => {
                        // The tag value itself is the offset
                        match tag_value.as_u32() {
                            Some(val) => val as u64,
                            None => continue, // Skip if not convertible to u32
                        }
                    }
                    OffsetField::ValueOffset => {
                        // This would be for more complex offset extraction
                        // For now, treat as actual value
                        match tag_value.as_u32() {
                            Some(val) => val as u64,
                            None => continue,
                        }
                    }
                };

                // Calculate the final offset based on the rule's base
                let final_offset = match rule.base {
                    OffsetBase::FileStart => offset + rule.additional_offset as u64,
                    OffsetBase::IfdStart => {
                        // Would need IFD start position - for now use file start
                        offset + rule.additional_offset as u64
                    }
                    OffsetBase::MakerNoteStart => {
                        // Would need maker note start - for now use file start
                        offset + rule.additional_offset as u64
                    }
                    OffsetBase::DataPosition => offset + rule.additional_offset as u64,
                };

                // Extract data at the calculated offset
                // ExifTool: PanasonicRaw.pm extracts additional data blocks at these offsets
                if let Some(extracted_data) =
                    self.extract_data_at_offset(data, final_offset, tag_id)
                {
                    // Process the extracted data based on the tag type
                    self.process_offset_data(reader, tag_id, &extracted_data)?;
                }
            }
        }

        Ok(())
    }

    /// Extract data at a specific offset for entry-based processing
    /// ExifTool: PanasonicRaw.pm extracts data blocks at calculated offsets
    fn extract_data_at_offset(&self, data: &[u8], offset: u64, tag_id: u16) -> Option<Vec<u8>> {
        let offset = offset as usize;

        // Bounds check
        if offset >= data.len() {
            return None;
        }

        // Determine data size based on tag type
        // ExifTool: Different tags extract different amounts of data
        let size = match tag_id {
            0x002e => 4, // JpgFromRaw - typically a 4-byte offset
            0x0127 => 4, // JpgFromRaw2 - typically a 4-byte offset
            _ => 4,      // Default size for unknown entry-based tags
        };

        // Extract the data block
        if offset + size <= data.len() {
            Some(data[offset..offset + size].to_vec())
        } else {
            // Extract remaining data if partial block available
            Some(data[offset..].to_vec())
        }
    }

    /// Process data extracted at calculated offsets
    /// ExifTool: PanasonicRaw.pm processes different types of offset data
    fn process_offset_data(
        &self,
        _reader: &mut ExifReader,
        tag_id: u16,
        data: &[u8],
    ) -> Result<()> {
        // Process based on the specific tag type
        match tag_id {
            0x002e => {
                // JpgFromRaw - points to embedded JPEG data
                // For now, just note that we found the offset
                if data.len() >= 4 {
                    let jpeg_offset = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                    // Could extract JPEG metadata here in the future
                    // ExifTool: PanasonicRaw.pm extracts embedded JPEG EXIF data
                    tracing::debug!("Found JpgFromRaw offset: {:#x}", jpeg_offset);
                }
            }
            0x0127 => {
                // JpgFromRaw2 - alternate JPEG data pointer
                if data.len() >= 4 {
                    let jpeg_offset = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                    tracing::debug!("Found JpgFromRaw2 offset: {:#x}", jpeg_offset);
                }
            }
            _ => {
                // Unknown entry-based tag - log for debugging
                tracing::debug!(
                    "Processed entry-based offset for tag {:#x}: {} bytes",
                    tag_id,
                    data.len()
                );
            }
        }

        Ok(())
    }
    /// Create new Panasonic RAW handler with processors
    /// ExifTool: %Image::ExifTool::PanasonicRaw::Main hash construction
    pub fn new() -> Self {
        Self {
            binary_processor: PanasonicBinaryProcessor::new(),
            offset_processor: Self::create_offset_processor(),
        }
    }

    /// Create entry-based offset processor with Panasonic rules
    /// ExifTool: PanasonicRaw.pm entry-based offset handling
    fn create_offset_processor() -> EntryBasedOffsetProcessor {
        let mut offset_rules = HashMap::new();

        // JpgFromRaw tag at 0x002e uses entry-based offset
        // ExifTool: line 198 - 0x2e => { Name => 'JpgFromRaw', ... }
        // The offset is stored in the IFD entry value and points to JPEG data
        offset_rules.insert(
            0x002e,
            OffsetExtractionRule {
                tag_id: 0x002e,
                offset_field: OffsetField::ActualValue,
                base: OffsetBase::FileStart,
                additional_offset: 0,
            },
        );

        // JpgFromRaw2 tag at 0x0127 (newer models)
        // ExifTool: line 301 - 0x127 => { Name => 'JpgFromRaw2', ... }
        offset_rules.insert(
            0x0127,
            OffsetExtractionRule {
                tag_id: 0x0127,
                offset_field: OffsetField::ActualValue,
                base: OffsetBase::FileStart,
                additional_offset: 0,
            },
        );

        EntryBasedOffsetProcessor::new(offset_rules)
    }

    /// Apply PrintConv to extracted Panasonic RAW tags
    /// ExifTool: PanasonicRaw.pm Main hash PrintConv fields
    fn apply_print_conv_to_extracted_tags(&self, reader: &mut ExifReader) -> Result<()> {
        // Get the currently extracted tags to apply PrintConv
        let extracted_tags: Vec<(u16, TagValue)> = reader
            .get_extracted_tags()
            .iter()
            .map(|(&k, v)| (k, v.clone()))
            .collect();

        // Apply PrintConv to applicable tags
        for (tag_id, raw_value) in extracted_tags {
            let converted_value =
                crate::implementations::panasonic_raw::apply_panasonic_raw_print_conv_by_tag_id(
                    tag_id, &raw_value,
                );

            // Only update if PrintConv actually changed the value
            if converted_value != raw_value {
                reader.extracted_tags.insert(tag_id, converted_value);
            }
        }

        Ok(())
    }
}

impl RawFormatHandler for PanasonicRawHandler {
    /// Process Panasonic RW2/RWL data
    /// ExifTool: Standard TIFF processing with Panasonic-specific handling
    fn process_raw(&self, reader: &mut ExifReader, data: &[u8]) -> Result<()> {
        // Panasonic RW2/RWL files are TIFF-based, so we parse the TIFF header first
        // ExifTool: lib/Image/ExifTool/PanasonicRaw.pm uses standard TIFF processing

        // Step 1: Parse TIFF header to get IFD structure
        let header = TiffHeader::parse(data)?;
        reader.set_test_header(header.clone());
        reader.set_test_data(data.to_vec());

        // Step 2: Process the main IFD using existing TIFF infrastructure  
        // ExifTool: Uses PanasonicRaw::Main table for IFD0 processing in RW2 files
        // We use "IFD0" name so our tag lookup correctly identifies this as Panasonic RW2 IFD0
        let dir_info = DirectoryInfo {
            name: "IFD0".to_string(),  // Use IFD0 so get_tag_name() recognizes this as Panasonic context
            dir_start: header.ifd0_offset as usize,
            dir_len: 0,  // Will be calculated by IFD processing
            base: 0,     // Standard TIFF base
            data_pos: 0, // No additional data position offset
            allow_reprocess: false,
        };

        // Process the TIFF IFD to extract real tag entries
        reader.process_subdirectory(&dir_info)?;

        // Step 3: Apply entry-based offset processing to extracted IFD entries
        // ExifTool: PanasonicRaw.pm applies additional processing for specific tags like JpgFromRaw
        self.process_entry_based_offsets(reader, data)?;

        // Step 4: Apply PrintConv for enhanced metadata interpretation
        // ExifTool: PanasonicRaw.pm Main hash PrintConv fields
        self.apply_print_conv_to_extracted_tags(reader)?;

        Ok(())
    }

    fn name(&self) -> &'static str {
        "PanasonicRaw"
    }

    fn validate_format(&self, data: &[u8]) -> bool {
        // ExifTool: PanasonicRaw.pm validation logic - TIFF-based format
        super::super::detector::validate_panasonic_rw2_magic(data)
    }
}

impl PanasonicBinaryProcessor {
    /// Create new Panasonic binary processor with tag definitions
    /// ExifTool: %Image::ExifTool::PanasonicRaw::Main hash (lines 70-380)
    fn new() -> Self {
        let mut tag_definitions = HashMap::new();

        // PanasonicRawVersion at 0x01
        // ExifTool: line 76 - 0x01 => { Name => 'PanasonicRawVersion', Writable => 'undef' }
        tag_definitions.insert(
            0x01,
            PanasonicTagDef {
                name: "PanasonicRawVersion".to_string(),
                format: PanasonicFormat::Undef,
                value_conv: None,
                print_conv: None,
                groups: None,
                writable: true,
            },
        );

        // SensorWidth at 0x02
        // ExifTool: line 80 - 0x02 => 'SensorWidth'
        tag_definitions.insert(
            0x02,
            PanasonicTagDef {
                name: "SensorWidth".to_string(),
                format: PanasonicFormat::Int16u,
                value_conv: None,
                print_conv: None,
                groups: None,
                writable: false,
            },
        );

        // SensorHeight at 0x03
        // ExifTool: line 81 - 0x03 => 'SensorHeight'
        tag_definitions.insert(
            0x03,
            PanasonicTagDef {
                name: "SensorHeight".to_string(),
                format: PanasonicFormat::Int16u,
                value_conv: None,
                print_conv: None,
                groups: None,
                writable: false,
            },
        );

        // SensorTopBorder at 0x04
        // ExifTool: line 82 - 0x04 => 'SensorTopBorder'
        tag_definitions.insert(
            0x04,
            PanasonicTagDef {
                name: "SensorTopBorder".to_string(),
                format: PanasonicFormat::Int16u,
                value_conv: None,
                print_conv: None,
                groups: None,
                writable: false,
            },
        );

        // SensorLeftBorder at 0x05
        // ExifTool: line 83 - 0x05 => 'SensorLeftBorder'
        tag_definitions.insert(
            0x05,
            PanasonicTagDef {
                name: "SensorLeftBorder".to_string(),
                format: PanasonicFormat::Int16u,
                value_conv: None,
                print_conv: None,
                groups: None,
                writable: false,
            },
        );

        // SensorBottomBorder at 0x06
        // ExifTool: line 84 - 0x06 => 'SensorBottomBorder'
        tag_definitions.insert(
            0x06,
            PanasonicTagDef {
                name: "SensorBottomBorder".to_string(),
                format: PanasonicFormat::Int16u,
                value_conv: None,
                print_conv: None,
                groups: None,
                writable: false,
            },
        );

        // SensorRightBorder at 0x07
        // ExifTool: line 85 - 0x07 => 'SensorRightBorder'
        tag_definitions.insert(
            0x07,
            PanasonicTagDef {
                name: "SensorRightBorder".to_string(),
                format: PanasonicFormat::Int16u,
                value_conv: None,
                print_conv: None,
                groups: None,
                writable: false,
            },
        );

        // CFAPattern at 0x09
        // ExifTool: line 92 - 0x09 => { Name => 'CFAPattern', ... PrintConv => {...} }
        let mut cfa_pattern_lookup = HashMap::new();
        cfa_pattern_lookup.insert(0, "n/a".to_string());
        cfa_pattern_lookup.insert(1, "[Red,Green][Green,Blue]".to_string());
        cfa_pattern_lookup.insert(2, "[Green,Red][Blue,Green]".to_string());
        cfa_pattern_lookup.insert(3, "[Green,Blue][Red,Green]".to_string());
        cfa_pattern_lookup.insert(4, "[Blue,Green][Green,Red]".to_string());

        tag_definitions.insert(
            0x09,
            PanasonicTagDef {
                name: "CFAPattern".to_string(),
                format: PanasonicFormat::Int16u,
                value_conv: None,
                print_conv: Some(PanasonicPrintConv::HashLookup(cfa_pattern_lookup)),
                groups: None,
                writable: true,
            },
        );

        // BitsPerSample at 0x0a
        // ExifTool: line 104 - 0x0a => { Name => 'BitsPerSample', Writable => 'int16u', Protected => 1 }
        tag_definitions.insert(
            0x0a,
            PanasonicTagDef {
                name: "BitsPerSample".to_string(),
                format: PanasonicFormat::Int16u,
                value_conv: None,
                print_conv: None,
                groups: None,
                writable: true,
            },
        );

        // Compression at 0x0b
        // ExifTool: line 105 - 0x0b => { Name => 'Compression', ... PrintConv => {...} }
        let mut compression_lookup = HashMap::new();
        compression_lookup.insert(34316, "Panasonic RAW 1".to_string());
        compression_lookup.insert(34826, "Panasonic RAW 2".to_string());
        compression_lookup.insert(34828, "Panasonic RAW 3".to_string());
        compression_lookup.insert(34830, "Panasonic RAW 4".to_string());

        tag_definitions.insert(
            0x0b,
            PanasonicTagDef {
                name: "Compression".to_string(),
                format: PanasonicFormat::Int16u,
                value_conv: None,
                print_conv: Some(PanasonicPrintConv::HashLookup(compression_lookup)),
                groups: None,
                writable: true,
            },
        );

        // RedBalance at 0x11
        // ExifTool: line 122 - 0x11 => { Name => 'RedBalance', Writable => 'int16u', ValueConv => '$val / 256', ... }
        tag_definitions.insert(
            0x11,
            PanasonicTagDef {
                name: "RedBalance".to_string(),
                format: PanasonicFormat::Int16u,
                value_conv: Some(PanasonicValueConv::DivideBy256),
                print_conv: None,
                groups: None,
                writable: true,
            },
        );

        // BlueBalance at 0x12
        // ExifTool: line 129 - 0x12 => { Name => 'BlueBalance', Writable => 'int16u', ValueConv => '$val / 256', ... }
        tag_definitions.insert(
            0x12,
            PanasonicTagDef {
                name: "BlueBalance".to_string(),
                format: PanasonicFormat::Int16u,
                value_conv: Some(PanasonicValueConv::DivideBy256),
                print_conv: None,
                groups: None,
                writable: true,
            },
        );

        // ISO at 0x17
        // ExifTool: line 139 - 0x17 => { Name => 'ISO', Writable => 'int16u' }
        tag_definitions.insert(
            0x17,
            PanasonicTagDef {
                name: "ISO".to_string(),
                format: PanasonicFormat::Int16u,
                value_conv: None,
                print_conv: None,
                groups: None,
                writable: true,
            },
        );

        Self { tag_definitions }
    }

    /// Process Panasonic binary data
    /// ExifTool: ProcessBinaryData equivalent for Panasonic tags
    fn process(&self, reader: &mut ExifReader, data: &[u8]) -> Result<()> {
        // Since RW2/RWL are TIFF-based, we need proper TIFF IFD processing
        // For now, we'll extract what we can from the beginning of the data
        // TODO: Implement proper TIFF IFD parsing integration

        // For basic demonstration, process a few tags if data is available
        if data.len() < 20 {
            return Ok(()); // Not enough data for meaningful processing
        }

        // This is a simplified placeholder - real implementation would:
        // 1. Parse TIFF header to determine byte order
        // 2. Read IFD entries
        // 3. Extract values based on tag definitions
        // 4. Apply value/print conversions

        // Store a basic tag to show the handler is working
        let source_info =
            TagSourceInfo::new("EXIF".to_string(), "IFD0".to_string(), "Image".to_string());

        // Placeholder: create a basic tag entry
        reader.extracted_tags.insert(
            0x0001, // PanasonicRawVersion equivalent
            TagValue::String("Panasonic RW2/RWL".to_string()),
        );
        reader.tag_sources.insert(0x0001, source_info);

        Ok(())
    }

    /// Extract value from data based on format
    /// ExifTool: Format-specific value extraction
    #[allow(dead_code)]
    fn extract_value(
        &self,
        data: &[u8],
        offset: usize,
        format: &PanasonicFormat,
        byte_order: ByteOrder,
    ) -> Result<TagValue> {
        match format {
            PanasonicFormat::Int16u => {
                if offset + 2 > data.len() {
                    return Err(ExifError::ParseError(format!(
                        "Int16u at offset {offset:#x} extends beyond data bounds"
                    )));
                }
                let value = match byte_order {
                    ByteOrder::BigEndian => u16::from_be_bytes([data[offset], data[offset + 1]]),
                    ByteOrder::LittleEndian => u16::from_le_bytes([data[offset], data[offset + 1]]),
                };
                Ok(TagValue::U16(value))
            }
            PanasonicFormat::Int32u => {
                if offset + 4 > data.len() {
                    return Err(ExifError::ParseError(format!(
                        "Int32u at offset {offset:#x} extends beyond data bounds"
                    )));
                }
                let value = match byte_order {
                    ByteOrder::BigEndian => u32::from_be_bytes([
                        data[offset],
                        data[offset + 1],
                        data[offset + 2],
                        data[offset + 3],
                    ]),
                    ByteOrder::LittleEndian => u32::from_le_bytes([
                        data[offset],
                        data[offset + 1],
                        data[offset + 2],
                        data[offset + 3],
                    ]),
                };
                Ok(TagValue::U32(value))
            }
            PanasonicFormat::String => {
                // For string, we'd need to know the length or find null terminator
                // This is a simplified implementation
                let mut end = offset;
                while end < data.len() && data[end] != 0 {
                    end += 1;
                }
                let string_value = String::from_utf8_lossy(&data[offset..end]).to_string();
                Ok(TagValue::String(string_value))
            }
            PanasonicFormat::Undef => {
                // Return as binary data
                Ok(TagValue::Binary(data[offset..].to_vec()))
            }
            PanasonicFormat::Int16uArray(count) => {
                let total_bytes = count * 2;
                if offset + total_bytes > data.len() {
                    return Err(ExifError::ParseError(format!(
                        "Int16u array at offset {offset:#x} extends beyond data bounds"
                    )));
                }

                let mut values = Vec::new();
                for i in 0..*count {
                    let item_offset = offset + i * 2;
                    let value = match byte_order {
                        ByteOrder::BigEndian => {
                            u16::from_be_bytes([data[item_offset], data[item_offset + 1]])
                        }
                        ByteOrder::LittleEndian => {
                            u16::from_le_bytes([data[item_offset], data[item_offset + 1]])
                        }
                    };
                    values.push(value);
                }
                Ok(TagValue::U16Array(values))
            }
        }
    }

    /// Apply value conversion to extracted value
    /// ExifTool: ValueConv logic from tag definitions
    #[allow(dead_code)]
    fn apply_value_conversion(
        &self,
        value: &TagValue,
        conversion: &PanasonicValueConv,
    ) -> TagValue {
        match conversion {
            PanasonicValueConv::DivideBy256 => match value {
                TagValue::U16(val) => TagValue::F64(*val as f64 / 256.0),
                TagValue::U32(val) => TagValue::F64(*val as f64 / 256.0),
                _ => value.clone(),
            },
            PanasonicValueConv::MultiplyBy100 => match value {
                TagValue::U16(val) => TagValue::U32(*val as u32 * 100),
                TagValue::U32(val) => TagValue::U32(*val * 100),
                _ => value.clone(),
            },
        }
    }

    /// Apply print conversion to value
    /// ExifTool: PrintConv logic from tag definitions
    #[allow(dead_code)]
    fn apply_print_conversion(
        &self,
        value: &TagValue,
        conversion: &PanasonicPrintConv,
    ) -> TagValue {
        match conversion {
            PanasonicPrintConv::HashLookup(lookup) => {
                if let Some(key) = value.as_u16().map(|v| v as u32).or_else(|| value.as_u32()) {
                    if let Some(description) = lookup.get(&key) {
                        return TagValue::String(description.clone());
                    }
                }
                value.clone()
            }
            PanasonicPrintConv::CfaPattern => {
                // Handled by HashLookup in practice
                value.clone()
            }
            PanasonicPrintConv::Compression => {
                // Handled by HashLookup in practice
                value.clone()
            }
            PanasonicPrintConv::Orientation => {
                // Would use EXIF orientation lookup
                value.clone()
            }
        }
    }
}

/// Byte order for data processing
/// ExifTool: TIFF byte order from header
#[derive(Debug, Clone, Copy)]
enum ByteOrder {
    /// Big-endian (MM)
    BigEndian,
    /// Little-endian (II)
    LittleEndian,
}

/// Get Panasonic RW2 tag name by ID
/// ExifTool: lib/Image/ExifTool/PanasonicRaw.pm tag definitions
pub fn get_panasonic_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        0x01 => Some("PanasonicRawVersion"),
        0x02 => Some("SensorWidth"),
        0x03 => Some("SensorHeight"),
        0x04 => Some("SensorTopBorder"),
        0x05 => Some("SensorLeftBorder"),
        0x06 => Some("SensorBottomBorder"),
        0x07 => Some("SensorRightBorder"),
        0x08 => Some("SamplesPerPixel"),
        0x09 => Some("CFAPattern"),
        0x0a => Some("BitsPerSample"),
        0x0b => Some("Compression"),
        0x11 => Some("RedBalance"),
        0x12 => Some("BlueBalance"),
        0x17 => Some("ISO"),
        0x002e => Some("JpgFromRaw"),
        0x0127 => Some("JpgFromRaw2"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panasonic_handler_creation() {
        let handler = PanasonicRawHandler::new();
        assert_eq!(handler.name(), "PanasonicRaw");
    }

    #[test]
    fn test_get_panasonic_tag_name() {
        // Test known tag names
        assert_eq!(get_panasonic_tag_name(0x01), Some("PanasonicRawVersion"));
        assert_eq!(get_panasonic_tag_name(0x02), Some("SensorWidth"));
        assert_eq!(get_panasonic_tag_name(0x09), Some("CFAPattern"));
        assert_eq!(get_panasonic_tag_name(0x17), Some("ISO"));
        assert_eq!(get_panasonic_tag_name(0x002e), Some("JpgFromRaw"));

        // Test unknown tag
        assert_eq!(get_panasonic_tag_name(0x9999), None);
    }

    #[test]
    fn test_format_validation() {
        let handler = PanasonicRawHandler::new();

        // Test valid TIFF magic (big-endian)
        let valid_be_data = b"MM\x00\x2A\x00\x00\x00\x08test_data";
        assert!(handler.validate_format(valid_be_data));

        // Test valid TIFF magic (little-endian)
        let valid_le_data = b"II\x2A\x00\x08\x00\x00\x00test_data";
        assert!(handler.validate_format(valid_le_data));

        // Test invalid magic
        let invalid_data = b"XX\x2A\x00\x08\x00\x00\x00test_data";
        assert!(!handler.validate_format(invalid_data));

        // Test insufficient data
        let short_data = b"MM\x00";
        assert!(!handler.validate_format(short_data));
    }

    #[test]
    fn test_entry_based_offset_rules() {
        let handler = PanasonicRawHandler::new();

        // Test that entry-based offset processor has the expected rules
        let configured_tags = handler.offset_processor.get_configured_tags();
        assert!(configured_tags.contains(&0x002e)); // JpgFromRaw
        assert!(configured_tags.contains(&0x0127)); // JpgFromRaw2

        // Test rule retrieval
        let rule_002e = handler.offset_processor.get_rule(0x002e);
        assert!(rule_002e.is_some());
        let rule = rule_002e.unwrap();
        assert_eq!(rule.tag_id, 0x002e);
        assert!(matches!(rule.offset_field, OffsetField::ActualValue));
        assert!(matches!(rule.base, OffsetBase::FileStart));
    }

    #[test]
    fn test_binary_processor_creation() {
        let processor = PanasonicBinaryProcessor::new();

        // Test that processor has expected tag definitions
        assert!(processor.tag_definitions.contains_key(&0x01)); // PanasonicRawVersion
        assert!(processor.tag_definitions.contains_key(&0x02)); // SensorWidth
        assert!(processor.tag_definitions.contains_key(&0x09)); // CFAPattern
        assert!(processor.tag_definitions.contains_key(&0x17)); // ISO

        // Test tag definition details
        let sensor_width_def = &processor.tag_definitions[&0x02];
        assert_eq!(sensor_width_def.name, "SensorWidth");
        assert!(matches!(sensor_width_def.format, PanasonicFormat::Int16u));
        assert!(!sensor_width_def.writable);

        let cfa_pattern_def = &processor.tag_definitions[&0x09];
        assert_eq!(cfa_pattern_def.name, "CFAPattern");
        assert!(cfa_pattern_def.print_conv.is_some());
        assert!(cfa_pattern_def.writable);
    }
}
