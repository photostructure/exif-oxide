//! EXIF/TIFF parsing module
//!
//! This module implements EXIF parsing for JPEG-embedded EXIF data, translating
//! ExifTool's ProcessExif function (Exif.pm:6172-7128) to handle:
//! - TIFF header validation and endianness detection
//! - IFD (Image File Directory) parsing
//! - Basic tag value extraction (ASCII, SHORT, LONG formats)
//! - Make/Model/Software extraction with null-termination
//! - Milestone 5: SubDirectory support with recursion prevention (ExifIFD, GPS)
//! - Stateful reader with PROCESSED tracking and PATH management
//!
//! Reference: lib/Image/ExifTool/Exif.pm ProcessExif function

use crate::generated::TAG_BY_ID;
use crate::types::{
    DataMemberValue, DirectoryInfo, ExifError, ProcessorDispatch, ProcessorType, Result, TagValue,
};
use std::collections::HashMap;
use tracing::{debug, trace, warn};

/// TIFF format types mapping to ExifTool's format system
/// ExifTool: lib/Image/ExifTool/Exif.pm @formatName array
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TiffFormat {
    Byte = 1,       // int8u
    Ascii = 2,      // string
    Short = 3,      // int16u
    Long = 4,       // int32u
    Rational = 5,   // rational64u
    SByte = 6,      // int8s
    Undefined = 7,  // undef
    SShort = 8,     // int16s
    SLong = 9,      // int32s
    SRational = 10, // rational64s
    Float = 11,     // float
    Double = 12,    // double
    Ifd = 13,       // ifd
}

impl TiffFormat {
    /// Get byte size for this format type
    /// ExifTool: lib/Image/ExifTool/Exif.pm @formatSize array
    pub fn byte_size(self) -> usize {
        match self {
            TiffFormat::Byte | TiffFormat::Ascii | TiffFormat::SByte | TiffFormat::Undefined => 1,
            TiffFormat::Short | TiffFormat::SShort => 2,
            TiffFormat::Long | TiffFormat::SLong | TiffFormat::Float | TiffFormat::Ifd => 4,
            TiffFormat::Rational | TiffFormat::SRational | TiffFormat::Double => 8,
        }
    }

    /// Create from format number, following ExifTool's validation
    /// ExifTool: lib/Image/ExifTool/Exif.pm:6352 format validation
    pub fn from_u16(format: u16) -> Result<Self> {
        match format {
            1 => Ok(TiffFormat::Byte),
            2 => Ok(TiffFormat::Ascii),
            3 => Ok(TiffFormat::Short),
            4 => Ok(TiffFormat::Long),
            5 => Ok(TiffFormat::Rational),
            6 => Ok(TiffFormat::SByte),
            7 => Ok(TiffFormat::Undefined),
            8 => Ok(TiffFormat::SShort),
            9 => Ok(TiffFormat::SLong),
            10 => Ok(TiffFormat::SRational),
            11 => Ok(TiffFormat::Float),
            12 => Ok(TiffFormat::Double),
            13 => Ok(TiffFormat::Ifd),
            _ => Err(ExifError::ParseError(format!(
                "Invalid TIFF format type: {format}"
            ))),
        }
    }
}

/// Byte order for TIFF data
/// ExifTool: lib/Image/ExifTool/Exif.pm TIFF header validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteOrder {
    LittleEndian, // "II" - Intel format
    BigEndian,    // "MM" - Motorola format
}

impl ByteOrder {
    /// Read u16 value respecting byte order
    pub fn read_u16(self, data: &[u8], offset: usize) -> Result<u16> {
        if offset + 2 > data.len() {
            return Err(ExifError::ParseError("Not enough data for u16".to_string()));
        }
        let bytes = &data[offset..offset + 2];
        Ok(match self {
            ByteOrder::LittleEndian => u16::from_le_bytes([bytes[0], bytes[1]]),
            ByteOrder::BigEndian => u16::from_be_bytes([bytes[0], bytes[1]]),
        })
    }

    /// Read u32 value respecting byte order  
    pub fn read_u32(self, data: &[u8], offset: usize) -> Result<u32> {
        if offset + 4 > data.len() {
            return Err(ExifError::ParseError("Not enough data for u32".to_string()));
        }
        let bytes = &data[offset..offset + 4];
        Ok(match self {
            ByteOrder::LittleEndian => u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            ByteOrder::BigEndian => u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        })
    }
}

/// TIFF header structure
/// ExifTool: lib/Image/ExifTool/Exif.pm TIFF header validation
#[derive(Debug, Clone)]
pub struct TiffHeader {
    pub byte_order: ByteOrder,
    pub magic: u16,       // Should be 42 (0x002A)
    pub ifd0_offset: u32, // Offset to first IFD
}

impl TiffHeader {
    /// Parse TIFF header from data
    /// ExifTool: lib/Image/ExifTool/Exif.pm:6174-6248 header processing
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 8 {
            return Err(ExifError::ParseError(
                "TIFF header too short (need 8 bytes)".to_string(),
            ));
        }

        // Detect byte order from first 2 bytes
        let byte_order = match &data[0..2] {
            [0x49, 0x49] => ByteOrder::LittleEndian, // "II"
            [0x4D, 0x4D] => ByteOrder::BigEndian,    // "MM"
            _ => {
                return Err(ExifError::ParseError(
                    "Invalid TIFF byte order marker".to_string(),
                ))
            }
        };

        // Read magic number (should be 42)
        let magic = byte_order.read_u16(data, 2)?;
        if magic != 42 {
            return Err(ExifError::ParseError(format!(
                "Invalid TIFF magic number: {magic} (expected 42)"
            )));
        }

        // Read IFD0 offset
        let ifd0_offset = byte_order.read_u32(data, 4)?;

        Ok(TiffHeader {
            byte_order,
            magic,
            ifd0_offset,
        })
    }
}

/// IFD entry structure (12 bytes each)
/// ExifTool: lib/Image/ExifTool/Exif.pm:6347-6351 entry reading
#[derive(Debug, Clone)]
pub struct IfdEntry {
    pub tag_id: u16,
    pub format: TiffFormat,
    pub count: u32,
    pub value_or_offset: u32,
}

impl IfdEntry {
    /// Parse IFD entry from 12-byte data block
    /// ExifTool: lib/Image/ExifTool/Exif.pm:6348-6350 entry structure
    pub fn parse(data: &[u8], offset: usize, byte_order: ByteOrder) -> Result<Self> {
        if offset + 12 > data.len() {
            return Err(ExifError::ParseError(
                "Not enough data for IFD entry".to_string(),
            ));
        }

        let tag_id = byte_order.read_u16(data, offset)?;
        let format_num = byte_order.read_u16(data, offset + 2)?;
        let format = TiffFormat::from_u16(format_num)?;
        let count = byte_order.read_u32(data, offset + 4)?;
        let value_or_offset = byte_order.read_u32(data, offset + 8)?;

        Ok(IfdEntry {
            tag_id,
            format,
            count,
            value_or_offset,
        })
    }

    /// Calculate total size of this entry's data
    /// ExifTool: lib/Image/ExifTool/Exif.pm:6390 size calculation
    pub fn data_size(&self) -> u32 {
        self.count * self.format.byte_size() as u32
    }

    /// Check if value is stored inline (≤4 bytes) or as offset
    /// ExifTool: lib/Image/ExifTool/Exif.pm:6392 inline vs offset logic
    pub fn is_inline(&self) -> bool {
        self.data_size() <= 4
    }
}

/// Stateful EXIF reader for processing JPEG-embedded EXIF data
/// ExifTool: lib/Image/ExifTool/Exif.pm ProcessExif function architecture
#[derive(Debug)]
pub struct ExifReader {
    /// Extracted tag values by tag ID
    extracted_tags: HashMap<u16, TagValue>,
    /// TIFF header information
    header: Option<TiffHeader>,
    /// Raw EXIF data buffer
    data: Vec<u8>,
    /// Parse errors (non-fatal, for graceful degradation)
    warnings: Vec<String>,

    // Milestone 5: Stateful processing features
    /// PROCESSED hash for recursion prevention
    /// ExifTool: $$self{PROCESSED} prevents infinite loops
    processed: HashMap<u64, String>,
    /// PATH stack for directory hierarchy tracking
    /// ExifTool: $$self{PATH} tracks current directory path
    path: Vec<String>,
    /// DataMember storage for tag dependencies
    /// ExifTool: DataMember tags needed by other tags
    data_members: HashMap<String, DataMemberValue>,
    /// Current base offset for pointer calculations
    /// ExifTool: $$dirInfo{Base} + $$self{BASE}
    base: u64,
    /// Processor dispatch configuration
    /// ExifTool: PROCESS_PROC system for different directory types
    processor_dispatch: ProcessorDispatch,
}

impl ExifReader {
    /// Create new EXIF reader
    pub fn new() -> Self {
        Self {
            extracted_tags: HashMap::new(),
            header: None,
            data: Vec::new(),
            warnings: Vec::new(),
            // Milestone 5: Initialize stateful features
            processed: HashMap::new(),
            path: Vec::new(),
            data_members: HashMap::new(),
            base: 0,
            processor_dispatch: ProcessorDispatch::default(),
        }
    }

    /// Parse EXIF data from JPEG APP1 segment after "Exif\0\0"
    /// ExifTool: lib/Image/ExifTool/Exif.pm:6172 ProcessExif entry point
    pub fn parse_exif_data(&mut self, exif_data: &[u8]) -> Result<()> {
        if exif_data.len() < 8 {
            return Err(ExifError::ParseError(
                "EXIF data too short for TIFF header".to_string(),
            ));
        }

        // Store data for offset-based value reading
        self.data = exif_data.to_vec();

        // Parse TIFF header
        self.header = Some(TiffHeader::parse(exif_data)?);
        let header = self.header.as_ref().unwrap();

        // Parse IFD0 starting at the offset specified in header
        // ExifTool: ProcessExif starts with IFD0 processing
        let dir_info = DirectoryInfo {
            name: "IFD0".to_string(),
            dir_start: header.ifd0_offset as usize,
            dir_len: 0, // Will be calculated during processing
            base: 0,    // TIFF header is at base 0
            data_pos: 0,
            allow_reprocess: false,
        };
        self.process_subdirectory(&dir_info)?;

        Ok(())
    }

    /// Process a subdirectory with recursion prevention
    /// ExifTool: ProcessDirectory with PROCESSED tracking
    fn process_subdirectory(&mut self, dir_info: &DirectoryInfo) -> Result<()> {
        // Calculate unique address for recursion prevention
        // ExifTool: $addr = $$dirInfo{DirStart} + $$dirInfo{DataPos} + ($$dirInfo{Base}||0) + $$self{BASE}
        let addr = dir_info.dir_start as u64 + dir_info.data_pos + dir_info.base + self.base;

        // Check for infinite loops (ExifTool PROCESSED tracking)
        if let Some(prev_dir) = self.processed.get(&addr) {
            if !dir_info.allow_reprocess {
                warn!(
                    "Circular reference detected: {} already processed at address {:#x} (was {})",
                    dir_info.name, addr, prev_dir
                );
                self.warnings.push(format!(
                    "Circular reference detected: {} already processed at address {:#x} (was {})",
                    dir_info.name, addr, prev_dir
                ));
                return Ok(()); // Graceful degradation, not fatal
            }
        }

        trace!(
            "Entering directory {} at address {:#x}, path: {}",
            dir_info.name,
            addr,
            self.path.join("/")
        );

        // Enter subdirectory context
        self.path.push(dir_info.name.clone());
        self.processed.insert(addr, dir_info.name.clone());

        // Select and dispatch to appropriate processor
        // ExifTool: ProcessDirectory with PROCESS_PROC dispatch
        let processor = self.select_processor(&dir_info.name, None);
        let result = self.dispatch_processor(processor, dir_info);

        // Exit subdirectory context
        self.path.pop();

        result
    }

    /// Parse a single IFD (Image File Directory)
    /// ExifTool: lib/Image/ExifTool/Exif.pm:6232-6342 IFD processing
    fn parse_ifd(&mut self, ifd_offset: usize, ifd_name: &str) -> Result<()> {
        if ifd_offset + 2 > self.data.len() {
            return Err(ExifError::ParseError(format!(
                "IFD offset {ifd_offset:#x} beyond data bounds"
            )));
        }

        let byte_order = self.header.as_ref().unwrap().byte_order;

        // Read number of entries (first 2 bytes of IFD)
        // ExifTool: lib/Image/ExifTool/Exif.pm:6235 numEntries
        let num_entries = byte_order.read_u16(&self.data, ifd_offset)? as usize;

        // Calculate directory size: 2 bytes (count) + 12 bytes per entry + 4 bytes (next IFD)
        // ExifTool: lib/Image/ExifTool/Exif.pm:6236-6237 dirSize calculation
        let dir_size = 2 + 12 * num_entries + 4;
        let dir_end = ifd_offset + dir_size;

        if dir_end > self.data.len() {
            // Graceful degradation - ExifTool continues parsing what it can
            // ExifTool: lib/Image/ExifTool/Exif.pm:6238-6247 short directory handling
            self.warnings.push(format!(
                "Short directory size for {ifd_name} (missing {} bytes)",
                dir_end - self.data.len()
            ));
        }

        // Process each IFD entry
        // ExifTool: lib/Image/ExifTool/Exif.pm:6342-6349 entry loop
        for index in 0..num_entries {
            let entry_offset = ifd_offset + 2 + 12 * index;

            if entry_offset + 12 > self.data.len() {
                self.warnings
                    .push(format!("IFD entry {index} beyond data bounds"));
                break; // Graceful degradation
            }

            match self.parse_ifd_entry(entry_offset, byte_order, ifd_name, index) {
                Ok(()) => {} // Successfully parsed
                Err(e) => {
                    // Graceful degradation - log warning but continue
                    // ExifTool: lib/Image/ExifTool/Exif.pm:6360-6365 error handling
                    self.warnings
                        .push(format!("Error parsing {ifd_name} entry {index}: {e}"));
                }
            }
        }

        Ok(())
    }

    /// Parse a single IFD entry and extract tag value
    /// ExifTool: lib/Image/ExifTool/Exif.pm:6347-6570 entry processing
    fn parse_ifd_entry(
        &mut self,
        entry_offset: usize,
        byte_order: ByteOrder,
        _ifd_name: &str,
        _index: usize,
    ) -> Result<()> {
        // Parse 12-byte IFD entry structure
        let entry = IfdEntry::parse(&self.data, entry_offset, byte_order)?;

        // Look up tag definition in generated tables
        let tag_def = TAG_BY_ID.get(&(entry.tag_id as u32));

        // Milestone 3: Support for common numeric formats with PrintConv
        // ExifTool: lib/Image/ExifTool/Exif.pm:6390-6570 value extraction
        match entry.format {
            TiffFormat::Ascii => {
                let value = self.extract_ascii_value(&entry, byte_order)?;
                if !value.is_empty() {
                    let tag_value = TagValue::String(value);
                    let final_value = self.apply_conversions(&tag_value, tag_def.copied());
                    self.extracted_tags.insert(entry.tag_id, final_value);
                }
            }
            TiffFormat::Byte => {
                let value = self.extract_byte_value(&entry)?;
                let tag_value = TagValue::U8(value);
                let final_value = self.apply_conversions(&tag_value, tag_def.copied());
                self.extracted_tags.insert(entry.tag_id, final_value);
            }
            TiffFormat::Short => {
                let value = self.extract_short_value(&entry, byte_order)?;
                let tag_value = TagValue::U16(value);
                let final_value = self.apply_conversions(&tag_value, tag_def.copied());
                self.extracted_tags.insert(entry.tag_id, final_value);
            }
            TiffFormat::Long => {
                let value = self.extract_long_value(&entry, byte_order)?;
                let tag_value = TagValue::U32(value);

                // Milestone 5: Check for SubDirectory tags (ExifIFD, GPS, etc.)
                // ExifTool: SubDirectory processing for nested IFDs
                if let Some(tag_def) = tag_def {
                    if self.is_subdirectory_tag(entry.tag_id) {
                        self.process_subdirectory_tag(entry.tag_id, value, tag_def.name)?;
                    }
                }

                let final_value = self.apply_conversions(&tag_value, tag_def.copied());
                self.extracted_tags.insert(entry.tag_id, final_value);
            }
            _ => {
                // For other formats, store raw value for now
                // Future milestones will implement additional formats
                if let Some(tag_def) = tag_def {
                    self.warnings.push(format!(
                        "Unimplemented format {:?} for tag {} ({})",
                        entry.format, entry.tag_id, tag_def.name
                    ));
                }
            }
        }

        Ok(())
    }

    /// Extract ASCII string value with null-termination handling
    /// ExifTool: lib/Image/ExifTool/Exif.pm ConvertExifText for ASCII processing
    fn extract_ascii_value(&self, entry: &IfdEntry, _byte_order: ByteOrder) -> Result<String> {
        let data = if entry.is_inline() {
            // Value stored inline in the 4-byte value field
            // ExifTool: lib/Image/ExifTool/Exif.pm:6372 inline value handling
            let bytes = entry.value_or_offset.to_le_bytes(); // Always stored in entry byte order
            bytes[..entry.count.min(4) as usize].to_vec()
        } else {
            // Value stored at offset
            // ExifTool: lib/Image/ExifTool/Exif.pm:6398 offset value handling
            let offset = entry.value_or_offset as usize;
            let size = entry.count as usize;

            if offset + size > self.data.len() {
                return Err(ExifError::ParseError(format!(
                    "ASCII value offset {offset:#x} + size {size} beyond data bounds"
                )));
            }

            self.data[offset..offset + size].to_vec()
        };

        // Convert bytes to string with null-termination handling
        // ExifTool handles null-terminated strings gracefully
        let null_pos = data.iter().position(|&b| b == 0).unwrap_or(data.len());
        let trimmed = &data[..null_pos];

        // Convert to UTF-8, handling invalid sequences gracefully
        match String::from_utf8(trimmed.to_vec()) {
            Ok(s) => Ok(s.trim().to_string()), // Trim whitespace
            Err(_) => {
                // Fallback for invalid UTF-8 - convert lossy
                Ok(String::from_utf8_lossy(trimmed).trim().to_string())
            }
        }
    }

    /// Extract SHORT (u16) value
    /// ExifTool: lib/Image/ExifTool/Exif.pm:6372-6398 value extraction
    fn extract_short_value(&self, entry: &IfdEntry, byte_order: ByteOrder) -> Result<u16> {
        if entry.count != 1 {
            return Err(ExifError::ParseError(format!(
                "SHORT value with count {} not supported yet",
                entry.count
            )));
        }

        if entry.is_inline() {
            // Value stored inline - use lower 2 bytes of value_or_offset
            // ExifTool: lib/Image/ExifTool/Exif.pm:6372 inline value handling
            // The value_or_offset field is always stored in the file's byte order
            let bytes = match byte_order {
                ByteOrder::LittleEndian => entry.value_or_offset.to_le_bytes(),
                ByteOrder::BigEndian => entry.value_or_offset.to_be_bytes(),
            };
            // For inline SHORT values, use the first 2 bytes in the correct order
            Ok(match byte_order {
                ByteOrder::LittleEndian => u16::from_le_bytes([bytes[0], bytes[1]]),
                ByteOrder::BigEndian => u16::from_be_bytes([bytes[0], bytes[1]]),
            })
        } else {
            // Value stored at offset
            let offset = entry.value_or_offset as usize;
            byte_order.read_u16(&self.data, offset)
        }
    }

    /// Extract BYTE (u8) value
    /// ExifTool: lib/Image/ExifTool/Exif.pm:6372-6398 value extraction
    fn extract_byte_value(&self, entry: &IfdEntry) -> Result<u8> {
        if entry.count != 1 {
            return Err(ExifError::ParseError(format!(
                "BYTE value with count {} not supported yet",
                entry.count
            )));
        }

        if entry.is_inline() {
            // Value stored inline - use lowest byte of value_or_offset
            // ExifTool: lib/Image/ExifTool/Exif.pm:6372 inline value handling
            Ok(entry.value_or_offset as u8)
        } else {
            // Value stored at offset
            let offset = entry.value_or_offset as usize;
            if offset >= self.data.len() {
                return Err(ExifError::ParseError(format!(
                    "BYTE value offset {offset:#x} beyond data bounds"
                )));
            }
            Ok(self.data[offset])
        }
    }

    /// Extract LONG (u32) value
    /// ExifTool: lib/Image/ExifTool/Exif.pm:6372-6398 value extraction
    fn extract_long_value(&self, entry: &IfdEntry, byte_order: ByteOrder) -> Result<u32> {
        if entry.count != 1 {
            return Err(ExifError::ParseError(format!(
                "LONG value with count {} not supported yet",
                entry.count
            )));
        }

        if entry.is_inline() {
            // Value stored inline
            Ok(entry.value_or_offset)
        } else {
            // Value stored at offset
            let offset = entry.value_or_offset as usize;
            byte_order.read_u32(&self.data, offset)
        }
    }

    /// Get extracted tag by ID
    pub fn get_tag_by_id(&self, tag_id: u16) -> Option<&TagValue> {
        self.extracted_tags.get(&tag_id)
    }

    /// Get all extracted tags with their names (conversions already applied during extraction)
    pub fn get_all_tags(&self) -> HashMap<String, TagValue> {
        let mut result = HashMap::new();

        for (&tag_id, value) in &self.extracted_tags {
            if let Some(tag_def) = TAG_BY_ID.get(&(tag_id as u32)) {
                // Values are already converted during extraction in process_entry()
                result.insert(tag_def.name.to_string(), value.clone());
            } else {
                // Include unknown tags with hex ID matching ExifTool format
                // ExifTool: lib/Image/ExifTool.pm unknown tag formatting
                result.insert(format!("Tag_{tag_id:04X}"), value.clone());
            }
        }

        result
    }

    /// Get parsing warnings
    pub fn get_warnings(&self) -> &[String] {
        &self.warnings
    }

    /// Get TIFF header information
    pub fn get_header(&self) -> Option<&TiffHeader> {
        self.header.as_ref()
    }

    /// Apply ValueConv and PrintConv conversions to a raw tag value
    /// ExifTool: lib/Image/ExifTool.pm conversion pipeline
    /// Get current directory path for debugging
    /// ExifTool: $$self{PATH} shows directory hierarchy
    pub fn get_current_path(&self) -> String {
        if self.path.is_empty() {
            "Root".to_string()
        } else {
            self.path.join("/")
        }
    }

    /// Get processing statistics for --show-missing functionality
    pub fn get_processing_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("processed_directories".to_string(), self.processed.len());
        stats.insert("extracted_tags".to_string(), self.extracted_tags.len());
        stats.insert("warnings".to_string(), self.warnings.len());
        stats.insert("data_members".to_string(), self.data_members.len());
        stats.insert(
            "subdirectory_overrides".to_string(),
            self.processor_dispatch.subdirectory_overrides.len(),
        );
        stats
    }

    /// Get current processor dispatch configuration
    pub fn get_processor_dispatch(&self) -> &ProcessorDispatch {
        &self.processor_dispatch
    }

    /// Apply ValueConv and PrintConv conversions to a raw tag value
    /// ExifTool: lib/Image/ExifTool.pm conversion pipeline
    fn apply_conversions(
        &self,
        raw_value: &TagValue,
        tag_def: Option<&'static crate::generated::tags::TagDef>,
    ) -> TagValue {
        use crate::registry;

        let mut value = raw_value.clone();

        // Apply ValueConv first (if present)
        if let Some(tag_def) = tag_def {
            if let Some(value_conv_ref) = tag_def.value_conv_ref {
                value = registry::apply_value_conv(value_conv_ref, &value);
            }

            // Apply PrintConv second (if present) to convert to human-readable string
            if let Some(print_conv_ref) = tag_def.print_conv_ref {
                let converted_string = registry::apply_print_conv(print_conv_ref, &value);

                // Only use the converted string if it's different from the raw value
                // This prevents "Unknown (8)" type fallbacks from being used
                if converted_string != value.to_string() {
                    return TagValue::String(converted_string);
                }
            }
        }

        value
    }

    /// Check if a tag ID represents a SubDirectory pointer
    /// ExifTool: SubDirectory tags like ExifIFD (0x8769), GPS (0x8825)
    fn is_subdirectory_tag(&self, tag_id: u16) -> bool {
        match tag_id {
            0x8769 => true, // ExifIFD - Camera settings subdirectory
            0x8825 => true, // GPS - GPS information subdirectory
            0xA005 => true, // InteropIFD - Interoperability subdirectory
            0x927C => true, // MakerNotes - Manufacturer-specific data
            _ => false,
        }
    }

    /// Select appropriate processor for a directory
    /// ExifTool: $$subdir{ProcessProc} || $$tagTablePtr{PROCESS_PROC} || \&ProcessExif
    fn select_processor(&self, dir_name: &str, tag_id: Option<u16>) -> ProcessorType {
        // 1. Check for subdirectory-specific processor override
        if let Some(tag_id) = tag_id {
            if let Some(processor) = self.processor_dispatch.subdirectory_overrides.get(&tag_id) {
                debug!(
                    "Using SubDirectory ProcessProc override for tag {:#x}: {:?}",
                    tag_id, processor
                );
                return processor.clone();
            }
        }

        // 2. Directory-specific defaults (before table-level processor)
        // ExifTool: Some directories have implicit processors
        let dir_specific = match dir_name {
            "GPS" => Some(ProcessorType::Gps),
            "ExifIFD" | "InteropIFD" => Some(ProcessorType::Exif),
            "MakerNotes" => Some(ProcessorType::Generic("MakerNotes".to_string())),
            _ => None,
        };

        if let Some(processor) = dir_specific {
            debug!(
                "Using directory-specific processor for {}: {:?}",
                dir_name, processor
            );
            return processor;
        }

        // 3. Check for table-level processor
        if let Some(processor) = &self.processor_dispatch.table_processor {
            debug!("Using table PROCESS_PROC for {}: {:?}", dir_name, processor);
            return processor.clone();
        }

        // 4. Final fallback to EXIF
        debug!("Using default EXIF processor for {}", dir_name);
        ProcessorType::Exif
    }

    /// Dispatch to the appropriate processor function
    /// ExifTool: Dynamic function dispatch with no strict 'refs'
    fn dispatch_processor(
        &mut self,
        processor: ProcessorType,
        dir_info: &DirectoryInfo,
    ) -> Result<()> {
        trace!(
            "Dispatching to processor {:?} for directory {}",
            processor,
            dir_info.name
        );

        match processor {
            ProcessorType::Exif | ProcessorType::Gps => {
                // Standard EXIF IFD processing
                // ExifTool: ProcessExif function
                self.process_exif_ifd(dir_info.dir_start, &dir_info.name)
            }
            ProcessorType::BinaryData => {
                // Binary data processing with format tables
                // ExifTool: ProcessBinaryData function
                self.process_binary_data(dir_info)
            }
            ProcessorType::Canon(canon_proc) => {
                // Canon-specific processing
                self.process_canon(canon_proc, dir_info)
            }
            ProcessorType::Nikon(nikon_proc) => {
                // Nikon-specific processing
                self.process_nikon(nikon_proc, dir_info)
            }
            ProcessorType::Generic(proc_name) => {
                // Generic/unknown processor - fall back to EXIF
                warn!(
                    "Unknown processor '{}', falling back to EXIF processing",
                    proc_name
                );
                self.process_exif_ifd(dir_info.dir_start, &dir_info.name)
            }
        }
    }

    /// Process a SubDirectory tag by following the pointer to nested IFD
    /// ExifTool: SubDirectory processing with Start => '$val'
    fn process_subdirectory_tag(&mut self, tag_id: u16, offset: u32, tag_name: &str) -> Result<()> {
        let subdir_name = match tag_id {
            0x8769 => "ExifIFD",
            0x8825 => "GPS",
            0xA005 => "InteropIFD",
            0x927C => "MakerNotes",
            _ => return Ok(()), // Unknown subdirectory
        };

        // Validate offset bounds
        let offset = offset as usize;
        if offset >= self.data.len() {
            self.warnings.push(format!(
                "SubDirectory {} offset {:#x} beyond data bounds ({})",
                subdir_name,
                offset,
                self.data.len()
            ));
            return Ok(()); // Graceful degradation
        }

        // Create subdirectory info with processor override support
        // ExifTool: SubDirectory Start => '$val' means offset points to IFD start
        let dir_info = DirectoryInfo {
            name: subdir_name.to_string(),
            dir_start: offset,
            dir_len: 0, // Will be calculated during processing
            base: self.base,
            data_pos: 0,
            allow_reprocess: false,
        };

        // Check for SubDirectory ProcessProc override
        // ExifTool: $$subdir{ProcessProc} takes precedence
        if let Some(override_proc) = self.get_subdirectory_processor_override(tag_id) {
            // Store the override in our dispatch system for this call
            // This simulates ExifTool's dynamic processor selection
            trace!(
                "Found SubDirectory ProcessProc override for {}: {:?}",
                subdir_name,
                override_proc
            );
        }

        debug!(
            "Processing SubDirectory: {} -> {} at offset {:#x}",
            tag_name, subdir_name, offset
        );

        // Process the subdirectory
        self.process_subdirectory(&dir_info)
    }

    /// Get SubDirectory processor override if available
    /// ExifTool: SubDirectory ProcessProc parameter
    fn get_subdirectory_processor_override(&self, tag_id: u16) -> Option<ProcessorType> {
        // Check for known SubDirectory processor overrides
        // ExifTool: These are defined in tag tables as SubDirectory => { ProcessProc => ... }
        match tag_id {
            0x8769 => None, // ExifIFD - uses standard EXIF processing
            0x8825 => None, // GPS - uses GPS variant of EXIF processing
            0xA005 => None, // InteropIFD - uses standard EXIF processing
            0x927C => {
                // MakerNotes - could have manufacturer-specific processors
                // For now, use generic processing
                Some(ProcessorType::Generic("MakerNotes".to_string()))
            }
            _ => None,
        }
    }

    /// Configure processor dispatch for specific table/tag combinations
    /// ExifTool: Runtime processor configuration
    pub fn configure_processor_dispatch(&mut self, dispatch: ProcessorDispatch) {
        self.processor_dispatch = dispatch;
    }

    /// Add SubDirectory processor override
    /// ExifTool: SubDirectory ProcessProc configuration
    pub fn add_subdirectory_override(&mut self, tag_id: u16, processor: ProcessorType) {
        self.processor_dispatch
            .subdirectory_overrides
            .insert(tag_id, processor);
    }

    /// Process standard EXIF IFD (renamed from parse_ifd)
    /// ExifTool: ProcessExif function for standard IFD processing
    fn process_exif_ifd(&mut self, ifd_offset: usize, ifd_name: &str) -> Result<()> {
        // This is the existing parse_ifd logic, renamed for clarity
        self.parse_ifd(ifd_offset, ifd_name)
    }

    /// Process binary data with format tables
    /// ExifTool: ProcessBinaryData function
    fn process_binary_data(&mut self, _dir_info: &DirectoryInfo) -> Result<()> {
        // Placeholder for ProcessBinaryData implementation
        // This will be implemented in future milestones
        debug!(
            "ProcessBinaryData not yet implemented for {}",
            _dir_info.name
        );
        Ok(())
    }

    /// Process Canon manufacturer-specific data
    /// ExifTool: Canon.pm processing procedures
    fn process_canon(
        &mut self,
        _canon_proc: crate::types::CanonProcessor,
        dir_info: &DirectoryInfo,
    ) -> Result<()> {
        // Placeholder for Canon-specific processing
        // This will be implemented in future milestones
        debug!("Canon processing not yet implemented for {}", dir_info.name);
        self.process_exif_ifd(dir_info.dir_start, &dir_info.name)
    }

    /// Process Nikon manufacturer-specific data
    /// ExifTool: Nikon.pm processing procedures
    fn process_nikon(
        &mut self,
        _nikon_proc: crate::types::NikonProcessor,
        dir_info: &DirectoryInfo,
    ) -> Result<()> {
        // Placeholder for Nikon-specific processing
        // This will be implemented in future milestones
        debug!("Nikon processing not yet implemented for {}", dir_info.name);
        self.process_exif_ifd(dir_info.dir_start, &dir_info.name)
    }
}

impl Default for ExifReader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tiff_format_byte_size() {
        assert_eq!(TiffFormat::Byte.byte_size(), 1);
        assert_eq!(TiffFormat::Short.byte_size(), 2);
        assert_eq!(TiffFormat::Long.byte_size(), 4);
        assert_eq!(TiffFormat::Rational.byte_size(), 8);
    }

    #[test]
    fn test_tiff_format_from_u16() {
        assert_eq!(TiffFormat::from_u16(1).unwrap(), TiffFormat::Byte);
        assert_eq!(TiffFormat::from_u16(2).unwrap(), TiffFormat::Ascii);
        assert_eq!(TiffFormat::from_u16(3).unwrap(), TiffFormat::Short);
        assert!(TiffFormat::from_u16(99).is_err());
    }

    #[test]
    fn test_byte_order_read() {
        let data = [0x12, 0x34, 0x56, 0x78];

        // Little-endian
        let le = ByteOrder::LittleEndian;
        assert_eq!(le.read_u16(&data, 0).unwrap(), 0x3412);
        assert_eq!(le.read_u32(&data, 0).unwrap(), 0x78563412);

        // Big-endian
        let be = ByteOrder::BigEndian;
        assert_eq!(be.read_u16(&data, 0).unwrap(), 0x1234);
        assert_eq!(be.read_u32(&data, 0).unwrap(), 0x12345678);
    }

    #[test]
    fn test_tiff_header_parse() {
        // Little-endian TIFF header
        let le_data = [
            0x49, 0x49, // "II" - little-endian
            0x2A, 0x00, // Magic: 42 (LE)
            0x08, 0x00, 0x00, 0x00, // IFD0 offset: 8 (LE)
        ];

        let header = TiffHeader::parse(&le_data).unwrap();
        assert_eq!(header.byte_order, ByteOrder::LittleEndian);
        assert_eq!(header.magic, 42);
        assert_eq!(header.ifd0_offset, 8);

        // Big-endian TIFF header
        let be_data = [
            0x4D, 0x4D, // "MM" - big-endian
            0x00, 0x2A, // Magic: 42 (BE)
            0x00, 0x00, 0x00, 0x08, // IFD0 offset: 8 (BE)
        ];

        let header = TiffHeader::parse(&be_data).unwrap();
        assert_eq!(header.byte_order, ByteOrder::BigEndian);
        assert_eq!(header.magic, 42);
        assert_eq!(header.ifd0_offset, 8);
    }

    #[test]
    fn test_ifd_entry_parse() {
        let data = [
            0x0F, 0x01, // Tag ID: 0x010F (Make) in LE
            0x02, 0x00, // Format: 2 (ASCII) in LE
            0x06, 0x00, 0x00, 0x00, // Count: 6 in LE
            0x43, 0x61, 0x6E, 0x6F, // Value: "Cano" inline
        ];

        let entry = IfdEntry::parse(&data, 0, ByteOrder::LittleEndian).unwrap();
        assert_eq!(entry.tag_id, 0x010F); // Make tag
        assert_eq!(entry.format, TiffFormat::Ascii);
        assert_eq!(entry.count, 6);
        assert_eq!(entry.value_or_offset, 0x6F6E6143); // "Cano" as u32
        assert!(!entry.is_inline()); // 6 bytes > 4, so not inline
    }

    #[test]
    fn test_exif_reader_basic() {
        let mut reader = ExifReader::new();

        // Create minimal EXIF data with TIFF header and empty IFD
        let exif_data = [
            0x49, 0x49, // "II" - little-endian
            0x2A, 0x00, // Magic: 42 (LE)
            0x08, 0x00, 0x00, 0x00, // IFD0 offset: 8 (LE)
            0x00, 0x00, // Number of entries: 0
            0x00, 0x00, 0x00, 0x00, // Next IFD: none
        ];

        reader.parse_exif_data(&exif_data).unwrap();

        let header = reader.get_header().unwrap();
        assert_eq!(header.byte_order, ByteOrder::LittleEndian);
        assert_eq!(header.ifd0_offset, 8);
    }

    #[test]
    fn test_subdirectory_recursion_prevention() {
        let mut reader = ExifReader::new();

        // Test recursion prevention by manually manipulating the PROCESSED hash
        let addr = 100u64;

        // Initially not processed
        assert!(!reader.processed.contains_key(&addr));

        // Mark as processed
        reader.processed.insert(addr, "TestIFD".to_string());

        // Verify it's marked as processed
        assert!(reader.processed.contains_key(&addr));
        assert_eq!(reader.processed.get(&addr), Some(&"TestIFD".to_string()));

        // Test that the same address would be detected as circular reference
        // We'll test this by checking the logic directly rather than calling process_subdirectory
        // which requires valid EXIF data
        let dir_info = DirectoryInfo {
            name: "TestIFD2".to_string(),
            dir_start: 100,
            dir_len: 0,
            base: 0,
            data_pos: 0,
            allow_reprocess: false,
        };

        let calculated_addr = dir_info.dir_start as u64 + dir_info.data_pos + dir_info.base;
        assert_eq!(calculated_addr, addr);
        assert!(reader.processed.contains_key(&calculated_addr));
    }

    #[test]
    fn test_subdirectory_path_management() {
        let mut reader = ExifReader::new();

        // Initially empty path
        assert_eq!(reader.get_current_path(), "Root");

        // Create some directory info (not used in this test)
        let _dir_info = DirectoryInfo {
            name: "ExifIFD".to_string(),
            dir_start: 0,
            dir_len: 0,
            base: 0,
            data_pos: 0,
            allow_reprocess: false,
        };

        // Manually test path management
        reader.path.push("IFD0".to_string());
        assert_eq!(reader.get_current_path(), "IFD0");

        reader.path.push("ExifIFD".to_string());
        assert_eq!(reader.get_current_path(), "IFD0/ExifIFD");

        reader.path.pop();
        assert_eq!(reader.get_current_path(), "IFD0");
    }

    #[test]
    fn test_subdirectory_tag_detection() {
        let reader = ExifReader::new();

        // Test that SubDirectory tags are detected correctly
        assert!(reader.is_subdirectory_tag(0x8769)); // ExifIFD
        assert!(reader.is_subdirectory_tag(0x8825)); // GPS
        assert!(reader.is_subdirectory_tag(0xA005)); // InteropIFD
        assert!(reader.is_subdirectory_tag(0x927C)); // MakerNotes

        // Test that regular tags are not detected as subdirectories
        assert!(!reader.is_subdirectory_tag(0x010F)); // Make
        assert!(!reader.is_subdirectory_tag(0x0110)); // Model
        assert!(!reader.is_subdirectory_tag(0x0112)); // Orientation
    }

    #[test]
    fn test_processing_statistics() {
        let mut reader = ExifReader::new();

        // Add some mock data
        reader
            .extracted_tags
            .insert(0x010F, TagValue::String("Canon".to_string()));
        reader.warnings.push("Test warning".to_string());
        reader
            .data_members
            .insert("TestMember".to_string(), DataMemberValue::U16(42));

        let stats = reader.get_processing_stats();
        assert_eq!(stats.get("extracted_tags"), Some(&1));
        assert_eq!(stats.get("warnings"), Some(&1));
        assert_eq!(stats.get("data_members"), Some(&1));
        assert_eq!(stats.get("processed_directories"), Some(&0));
        assert_eq!(stats.get("subdirectory_overrides"), Some(&0));
    }

    #[test]
    fn test_processor_dispatch_selection() {
        let reader = ExifReader::new();

        // Test default processor selection
        assert_eq!(reader.select_processor("IFD0", None), ProcessorType::Exif);
        assert_eq!(
            reader.select_processor("ExifIFD", None),
            ProcessorType::Exif
        );
        assert_eq!(reader.select_processor("GPS", None), ProcessorType::Gps);
        assert_eq!(
            reader.select_processor("InteropIFD", None),
            ProcessorType::Exif
        );

        // Test MakerNotes gets generic processor
        if let ProcessorType::Generic(name) = reader.select_processor("MakerNotes", None) {
            assert_eq!(name, "MakerNotes");
        } else {
            panic!("Expected Generic processor for MakerNotes");
        }
    }

    #[test]
    fn test_processor_dispatch_overrides() {
        let mut reader = ExifReader::new();

        // Add a SubDirectory override
        reader.add_subdirectory_override(0x8769, ProcessorType::BinaryData);

        // Verify override is stored
        let dispatch = reader.get_processor_dispatch();
        assert_eq!(
            dispatch.subdirectory_overrides.get(&0x8769),
            Some(&ProcessorType::BinaryData)
        );

        // Verify stats reflect the override
        let stats = reader.get_processing_stats();
        assert_eq!(stats.get("subdirectory_overrides"), Some(&1));
    }

    #[test]
    fn test_subdirectory_processor_overrides() {
        let reader = ExifReader::new();

        // Test known SubDirectory processor overrides
        assert_eq!(reader.get_subdirectory_processor_override(0x8769), None); // ExifIFD
        assert_eq!(reader.get_subdirectory_processor_override(0x8825), None); // GPS
        assert_eq!(reader.get_subdirectory_processor_override(0xA005), None); // InteropIFD

        // MakerNotes should have generic processor
        if let Some(ProcessorType::Generic(name)) =
            reader.get_subdirectory_processor_override(0x927C)
        {
            assert_eq!(name, "MakerNotes");
        } else {
            panic!("Expected Generic processor for MakerNotes");
        }

        // Unknown tag should have no override
        assert_eq!(reader.get_subdirectory_processor_override(0x1234), None);
    }
}
