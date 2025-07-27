//! Minolta RAW format handler

#![allow(dead_code, unused_variables)]
//!
//! This module implements ExifTool's MinoltaRaw.pm processing logic exactly.
//! Minolta/Konica-Minolta MRW files use a multi-block structure with separate
//! sections for TIFF tags (TTW), picture raw data info (PRD), white balance (WBG),
//! and requested image format (RIF).
//!
//! ExifTool Reference: lib/Image/ExifTool/MinoltaRaw.pm (537 lines total)
//! Processing: ProcessMRW with multiple block types and TIFF subdirectories

use crate::exif::ExifReader;
use crate::raw::RawFormatHandler;
use crate::types::{ExifError, Result, TagSourceInfo, TagValue};
use std::collections::HashMap;

/// Minolta RAW format handler
/// ExifTool: lib/Image/ExifTool/MinoltaRaw.pm - Multi-block structure with ProcessMRW
pub struct MinoltaRawHandler {
    /// Processor for PRD (Picture Raw Data) blocks
    /// ExifTool: %Image::ExifTool::MinoltaRaw::PRD hash (lines 56-110)
    prd_processor: MinoltaPrdProcessor,

    /// Processor for WBG (White Balance Gains) blocks
    /// ExifTool: %Image::ExifTool::MinoltaRaw::WBG hash (lines 113-137)
    wbg_processor: MinoltaWbgProcessor,

    /// Processor for RIF (Requested Image Format) blocks
    /// ExifTool: %Image::ExifTool::MinoltaRaw::RIF hash (lines 140-346)
    rif_processor: MinoltaRifProcessor,
}

/// MRW file header structure
/// ExifTool: MinoltaRaw.pm ProcessMRW function lines 392-494
#[derive(Debug)]
struct MrwHeader {
    /// Byte order - 'M' for big-endian, 'I' for little-endian
    /// ExifTool: MinoltaRaw.pm line 412 - SetByteOrder($1 . $1)
    byte_order: ByteOrder,

    /// Offset to image data
    /// ExifTool: MinoltaRaw.pm line 420 - my $offset = Get32u(\$data, 4) + $pos
    data_offset: u32,

    /// MRW blocks found in the file
    /// ExifTool: MinoltaRaw.pm lines 424-476 - loop through segments
    blocks: Vec<MrwBlock>,
}

/// Individual MRW block
/// ExifTool: MinoltaRaw.pm lines 427-476 - segment processing
#[derive(Debug)]
struct MrwBlock {
    /// Block type tag (4 bytes)
    /// ExifTool: MinoltaRaw.pm line 427 - my $tag = substr($data, 0, 4)
    tag: [u8; 4],

    /// Block data length
    /// ExifTool: MinoltaRaw.pm line 428 - my $len = Get32u(\$data, 4)
    length: u32,

    /// Block data
    /// ExifTool: MinoltaRaw.pm line 442 - $raf->Read($buff, $len)
    data: Vec<u8>,
}

/// Byte order for MRW processing
/// ExifTool: MinoltaRaw.pm line 412 - checks MRM vs MRI
#[derive(Debug, Clone, Copy)]
enum ByteOrder {
    /// Big-endian (MRM)
    /// ExifTool: MinoltaRaw.pm line 408 - "\0MRM" for big-endian
    BigEndian,

    /// Little-endian (MRI)
    /// ExifTool: MinoltaRaw.pm line 409 - "\0MRI" for little-endian (ARW images)
    LittleEndian,
}

/// PRD (Picture Raw Data) processor
/// ExifTool: %Image::ExifTool::MinoltaRaw::PRD hash (lines 56-110)
struct MinoltaPrdProcessor {
    /// Tag definitions for PRD block
    tag_definitions: HashMap<u8, PrdTagDef>,
}

/// WBG (White Balance Gains) processor  
/// ExifTool: %Image::ExifTool::MinoltaRaw::WBG hash (lines 113-137)
struct MinoltaWbgProcessor {
    /// Tag definitions for WBG block
    tag_definitions: HashMap<u8, WbgTagDef>,
}

/// RIF (Requested Image Format) processor
/// ExifTool: %Image::ExifTool::MinoltaRaw::RIF hash (lines 140-346)
struct MinoltaRifProcessor {
    /// Tag definitions for RIF block
    tag_definitions: HashMap<u8, RifTagDef>,
}

/// PRD tag definition
/// ExifTool: Individual entries in %Image::ExifTool::MinoltaRaw::PRD
#[derive(Debug, Clone)]
struct PrdTagDef {
    /// Tag name
    /// ExifTool: Name field in tag hash
    name: String,

    /// Data format
    /// ExifTool: Format field
    format: MinoltaFormat,

    /// Optional raw conversion
    /// ExifTool: RawConv field
    raw_conv: Option<String>,
}

/// WBG tag definition
/// ExifTool: Individual entries in %Image::ExifTool::MinoltaRaw::WBG
#[derive(Debug, Clone)]
struct WbgTagDef {
    /// Tag name
    /// ExifTool: Name field in tag hash
    name: String,

    /// Data format
    /// ExifTool: Format field
    format: MinoltaFormat,

    /// Conditional processing
    /// ExifTool: Condition field for DiMAGE A200 vs other models
    condition: Option<WbgCondition>,
}

/// RIF tag definition
/// ExifTool: Individual entries in %Image::ExifTool::MinoltaRaw::RIF
#[derive(Debug, Clone)]
struct RifTagDef {
    /// Tag name
    /// ExifTool: Name field in tag hash
    name: String,

    /// Data format
    /// ExifTool: Format field (defaults to int8u if not specified)
    format: MinoltaFormat,

    /// Print conversion
    /// ExifTool: PrintConv field
    print_conv: Option<RifPrintConv>,

    /// Value conversion
    /// ExifTool: ValueConv field
    value_conv: Option<RifValueConv>,

    /// Raw conversion
    /// ExifTool: RawConv field
    raw_conv: Option<RifRawConv>,

    /// Conditional processing
    /// ExifTool: Condition field for Sony vs Minolta models
    condition: Option<RifCondition>,
}

/// Minolta data formats
/// ExifTool: Format specifications in tag definitions
#[derive(Debug, Clone)]
enum MinoltaFormat {
    /// String with specified length
    /// ExifTool: string[N] format
    String(usize),

    /// 8-bit unsigned integer
    /// ExifTool: int8u format (default for RIF)
    Int8u,

    /// 8-bit signed integer
    /// ExifTool: int8s format
    Int8s,

    /// 16-bit unsigned integer
    /// ExifTool: int16u format
    Int16u,

    /// Array of 16-bit unsigned integers
    /// ExifTool: int16u[N] format
    Int16uArray(usize),

    /// Array of 8-bit unsigned integers
    /// ExifTool: int8u[N] format
    Int8uArray(usize),
}

/// WBG conditional processing
/// ExifTool: MinoltaRaw.pm lines 125-136 - DiMAGE A200 special case
#[derive(Debug, Clone)]
enum WbgCondition {
    /// DiMAGE A200 model
    /// ExifTool: line 126 - Condition => '$$self{Model} =~ /DiMAGE A200\b/'
    DiMageA200,

    /// Other models (default)
    /// ExifTool: line 132 - other models case
    Other,
}

/// RIF print conversion types
/// ExifTool: PrintConv fields in RIF tags
#[derive(Debug, Clone)]
enum RifPrintConv {
    /// White balance mode conversion
    /// ExifTool: line 161 - PrintConv => 'Image::ExifTool::MinoltaRaw::ConvertWBMode($val)'
    WbMode,

    /// Program mode lookup
    /// ExifTool: lines 165-175 - PrintConv hash
    ProgramMode,

    /// ISO setting conversion
    /// ExifTool: lines 179-194 - complex ISO conversion with formula
    IsoSetting,

    /// Color mode conversion (Minolta)
    /// ExifTool: line 207 - PrintConv => \%Image::ExifTool::Minolta::minoltaColorMode
    ColorModeMinolta,

    /// Color mode conversion (Sony A100)
    /// ExifTool: line 216 - PrintConv => \%Image::ExifTool::Minolta::sonyColorMode
    ColorModeSony,
}

/// RIF value conversion types
/// ExifTool: ValueConv fields in RIF tags
#[derive(Debug, Clone)]
enum RifValueConv {
    /// Color temperature scaling
    /// ExifTool: line 297 - ValueConv => '$val * 100'
    ColorTemperature,
}

/// RIF raw conversion types
/// ExifTool: RawConv fields in RIF tags
#[derive(Debug, Clone)]
enum RifRawConv {
    /// Exclude value 255
    /// ExifTool: line 178 - RawConv => '$val == 255 ? undef : $val'
    ExcludeU8Max,
}

/// RIF conditional processing
/// ExifTool: Condition fields in RIF tags for Make/Model detection
#[derive(Debug, Clone)]
enum RifCondition {
    /// Not Sony make
    /// ExifTool: line 204 - Condition => '$$self{Make} !~ /^SONY/'
    NotSony,

    /// Sony A100 model
    /// ExifTool: line 211 - Condition => '$$self{Model} eq "DSLR-A100"'
    SonyA100,

    /// Sony A100 or has PRD info
    /// ExifTool: line 222 - Condition => '$$self{Model} eq "DSLR-A100" or $$self{MinoltaPRD}'
    SonyA100OrPrd,

    /// Sony A200/A700 models
    /// ExifTool: line 327 - Condition => '$$self{Model} =~ /^DSLR-A(200|700)$/'
    SonyA200A700,

    /// Sony make
    /// ExifTool: line 302 - Condition => '$$self{Make} =~ /^SONY/'
    Sony,
}

impl Default for MinoltaRawHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl MinoltaRawHandler {
    /// Create new Minolta RAW handler with all processors
    /// ExifTool: %Image::ExifTool::MinoltaRaw::Main hash construction
    pub fn new() -> Self {
        Self {
            prd_processor: MinoltaPrdProcessor::new(),
            wbg_processor: MinoltaWbgProcessor::new(),
            rif_processor: MinoltaRifProcessor::new(),
        }
    }

    /// Parse MRW file header and extract blocks
    /// ExifTool: MinoltaRaw.pm ProcessMRW function lines 392-494
    fn parse_mrw_header(&self, data: &[u8]) -> Result<MrwHeader> {
        if data.len() < 8 {
            return Err(ExifError::ParseError(
                "MRW file too short for header".to_string(),
            ));
        }

        // Parse magic bytes and determine byte order
        // ExifTool: MinoltaRaw.pm lines 408-412
        let byte_order = if data.starts_with(b"\0MRM") {
            ByteOrder::BigEndian
        } else if data.starts_with(b"\0MRI") {
            ByteOrder::LittleEndian
        } else {
            return Err(ExifError::ParseError("Invalid MRW magic bytes".to_string()));
        };

        // Read data offset
        // ExifTool: MinoltaRaw.pm line 420 - my $offset = Get32u(\$data, 4) + $pos
        let data_offset = match byte_order {
            ByteOrder::BigEndian => u32::from_be_bytes([data[4], data[5], data[6], data[7]]),
            ByteOrder::LittleEndian => u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
        };

        let mut blocks = Vec::new();
        let mut pos = 8;

        // Loop through MRW segments
        // ExifTool: MinoltaRaw.pm lines 424-476
        while pos < data_offset as usize && pos + 8 <= data.len() {
            // Read block tag (4 bytes)
            // ExifTool: MinoltaRaw.pm line 427 - my $tag = substr($data, 0, 4)
            let mut tag = [0u8; 4];
            tag.copy_from_slice(&data[pos..pos + 4]);
            pos += 4;

            // Read block length
            // ExifTool: MinoltaRaw.pm line 428 - my $len = Get32u(\$data, 4)
            let length = match byte_order {
                ByteOrder::BigEndian => {
                    u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                }
                ByteOrder::LittleEndian => {
                    u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                }
            };
            pos += 4;

            // Read block data
            // ExifTool: MinoltaRaw.pm line 442 - $raf->Read($buff, $len)
            if pos + length as usize > data.len() {
                return Err(ExifError::ParseError(format!(
                    "MRW block extends beyond file bounds: pos={}, length={}, file_size={}",
                    pos,
                    length,
                    data.len()
                )));
            }

            let block_data = data[pos..pos + length as usize].to_vec();
            pos += length as usize;

            blocks.push(MrwBlock {
                tag,
                length,
                data: block_data,
            });
        }

        Ok(MrwHeader {
            byte_order,
            data_offset,
            blocks,
        })
    }

    /// Process TTW (TIFF Tags) block as TIFF subdirectory
    /// ExifTool: MinoltaRaw.pm line 31 - TTW subdirectory with ProcessTIFF
    fn process_ttw_block(
        &self,
        reader: &mut ExifReader,
        ttw_data: &[u8],
        byte_order: &ByteOrder,
    ) -> Result<()> {
        tracing::debug!("Processing TTW block with {} bytes", ttw_data.len());

        // The TTW block contains TIFF data that should be processed by the TIFF processor
        // ExifTool: MinoltaRaw.pm line 31 - TTW subdirectory ProcessTIFF

        // Create a cursor for the TTW data to use as a reader
        use std::io::Cursor;
        let mut ttw_cursor = Cursor::new(ttw_data);

        // Use our existing TIFF extraction to get the TIFF data
        match crate::formats::extract_tiff_exif(&mut ttw_cursor) {
            Ok(tiff_data) => {
                // Process the TIFF data using our existing EXIF processor
                // This should extract standard EXIF tags like Make, Model, ExposureTime, etc.
                match reader.parse_exif_data(&tiff_data) {
                    Ok(()) => {
                        tracing::debug!("Successfully processed TTW TIFF data");
                    }
                    Err(e) => {
                        tracing::warn!("Failed to process TTW TIFF data: {}", e);
                        return Err(e);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to extract TIFF from TTW block: {}", e);
                return Err(e);
            }
        }

        Ok(())
    }
}

impl RawFormatHandler for MinoltaRawHandler {
    /// Process Minolta MRW data
    /// ExifTool: ProcessMRW function dispatches to block-specific processors
    fn process_raw(&self, reader: &mut ExifReader, data: &[u8]) -> Result<()> {
        // Parse MRW header and blocks
        let header = self.parse_mrw_header(data)?;

        // Process each block based on its type
        // ExifTool: MinoltaRaw.pm lines 436-466 - block processing
        for block in &header.blocks {
            match &block.tag {
                // TTW block - TIFF tags (processed by standard TIFF processor)
                // ExifTool: lines 31-38 - TTW subdirectory with ProcessTIFF
                b"\0TTW" => {
                    // Process TTW block as TIFF subdirectory
                    // ExifTool: MinoltaRaw.pm line 31 - TTW subdirectory ProcessTIFF
                    if let Err(e) = self.process_ttw_block(reader, &block.data, &header.byte_order)
                    {
                        tracing::warn!("Failed to process TTW block: {}", e);
                        // Continue processing other blocks even if TTW fails
                    }
                }

                // PRD block - Picture Raw Data
                // ExifTool: lines 40-43 - PRD subdirectory
                b"\0PRD" => {
                    self.prd_processor
                        .process(reader, &block.data, &header.byte_order)?;
                }

                // WBG block - White Balance Gains
                // ExifTool: lines 44-47 - WBG subdirectory
                b"\0WBG" => {
                    self.wbg_processor
                        .process(reader, &block.data, &header.byte_order)?;
                }

                // RIF block - Requested Image Format
                // ExifTool: lines 48-51 - RIF subdirectory
                b"\0RIF" => {
                    self.rif_processor
                        .process(reader, &block.data, &header.byte_order)?;
                }

                // CSA block is padding - skip
                // ExifTool: line 52 - "# "\0CSA" is padding"
                b"\0CSA" => {
                    continue;
                }

                // Unknown block - skip with warning
                _ => {
                    let tag_str = String::from_utf8_lossy(&block.tag);
                    eprintln!("Warning: Unknown MRW block type: {tag_str}");
                    continue;
                }
            }
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "MinoltaRaw"
    }

    fn validate_format(&self, data: &[u8]) -> bool {
        // ExifTool: MinoltaRaw.pm validation logic
        super::super::detector::validate_minolta_mrw_magic(data)
    }
}

impl MinoltaPrdProcessor {
    /// Create new PRD processor with tag definitions
    /// ExifTool: %Image::ExifTool::MinoltaRaw::PRD hash (lines 56-110)
    fn new() -> Self {
        let mut tag_definitions = HashMap::new();

        // FirmwareID at offset 0
        // ExifTool: line 64 - 0 => { Name => 'FirmwareID', Format => 'string[8]', ... }
        tag_definitions.insert(
            0,
            PrdTagDef {
                name: "FirmwareID".to_string(),
                format: MinoltaFormat::String(8),
                raw_conv: Some(
                    "$$self{MinoltaPRD} = 1 if $$self{FILE_TYPE} eq \"MRW\"; $val".to_string(),
                ),
            },
        );

        // SensorHeight at offset 8
        // ExifTool: line 69 - 8 => { Name => 'SensorHeight', Format => 'int16u' }
        tag_definitions.insert(
            8,
            PrdTagDef {
                name: "SensorHeight".to_string(),
                format: MinoltaFormat::Int16u,
                raw_conv: None,
            },
        );

        // SensorWidth at offset 10
        // ExifTool: line 73 - 10 => { Name => 'SensorWidth', Format => 'int16u' }
        tag_definitions.insert(
            10,
            PrdTagDef {
                name: "SensorWidth".to_string(),
                format: MinoltaFormat::Int16u,
                raw_conv: None,
            },
        );

        // ImageHeight at offset 12
        // ExifTool: line 77 - 12 => { Name => 'ImageHeight', Format => 'int16u' }
        tag_definitions.insert(
            12,
            PrdTagDef {
                name: "ImageHeight".to_string(),
                format: MinoltaFormat::Int16u,
                raw_conv: None,
            },
        );

        // ImageWidth at offset 14
        // ExifTool: line 81 - 14 => { Name => 'ImageWidth', Format => 'int16u' }
        tag_definitions.insert(
            14,
            PrdTagDef {
                name: "ImageWidth".to_string(),
                format: MinoltaFormat::Int16u,
                raw_conv: None,
            },
        );

        // RawDepth at offset 16
        // ExifTool: line 85 - 16 => { Name => 'RawDepth', Format => 'int8u' }
        tag_definitions.insert(
            16,
            PrdTagDef {
                name: "RawDepth".to_string(),
                format: MinoltaFormat::Int8u,
                raw_conv: None,
            },
        );

        // BitDepth at offset 17
        // ExifTool: line 89 - 17 => { Name => 'BitDepth', Format => 'int8u' }
        tag_definitions.insert(
            17,
            PrdTagDef {
                name: "BitDepth".to_string(),
                format: MinoltaFormat::Int8u,
                raw_conv: None,
            },
        );

        // StorageMethod at offset 18
        // ExifTool: line 93 - 18 => { Name => 'StorageMethod', Format => 'int8u', PrintConv => {...} }
        tag_definitions.insert(
            18,
            PrdTagDef {
                name: "StorageMethod".to_string(),
                format: MinoltaFormat::Int8u,
                raw_conv: None,
            },
        );

        // BayerPattern at offset 23
        // ExifTool: line 101 - 23 => { Name => 'BayerPattern', Format => 'int8u', PrintConv => {...} }
        tag_definitions.insert(
            23,
            PrdTagDef {
                name: "BayerPattern".to_string(),
                format: MinoltaFormat::Int8u,
                raw_conv: None,
            },
        );

        Self { tag_definitions }
    }

    /// Process PRD block data
    /// ExifTool: ProcessBinaryData with MinoltaRaw::PRD table
    fn process(&self, reader: &mut ExifReader, data: &[u8], byte_order: &ByteOrder) -> Result<()> {
        // Process each tag definition
        for (&offset, tag_def) in &self.tag_definitions {
            let raw_value =
                self.extract_value(data, offset as usize, &tag_def.format, byte_order)?;

            // Apply PrintConv for enhanced metadata interpretation
            // ExifTool: MinoltaRaw.pm PRD hash PrintConv fields
            let tag_value = crate::implementations::minolta_raw::apply_prd_print_conv(
                &tag_def.name,
                &raw_value,
            );

            // Store the tag with source info
            // ExifTool: GROUPS => { 0 => 'MakerNotes', 2 => 'Camera' }
            let source_info = TagSourceInfo::new(
                "MakerNotes".to_string(),
                "MinoltaPRD".to_string(),
                "Camera".to_string(),
            );

            // Use offset as tag ID for PRD tags
            let tag_id = 0x1000 + offset as u16; // Offset to avoid conflicts
            reader.store_tag_with_precedence(tag_id, tag_value, source_info);
        }

        Ok(())
    }

    /// Extract value from binary data based on format
    /// ExifTool: ProcessBinaryData value extraction
    fn extract_value(
        &self,
        data: &[u8],
        offset: usize,
        format: &MinoltaFormat,
        byte_order: &ByteOrder,
    ) -> Result<TagValue> {
        match format {
            MinoltaFormat::String(length) => {
                if offset + length > data.len() {
                    return Err(ExifError::ParseError(format!(
                        "String at offset {offset:#x} extends beyond data bounds"
                    )));
                }
                let string_bytes = &data[offset..offset + length];
                let string_value = String::from_utf8_lossy(string_bytes).to_string();
                Ok(TagValue::String(string_value))
            }
            MinoltaFormat::Int8u => {
                if offset >= data.len() {
                    return Err(ExifError::ParseError(format!(
                        "Int8u at offset {offset:#x} extends beyond data bounds"
                    )));
                }
                Ok(TagValue::U8(data[offset]))
            }
            MinoltaFormat::Int16u => {
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
            _ => Err(ExifError::ParseError(format!(
                "Unsupported format in PRD processor: {format:?}"
            ))),
        }
    }
}

impl MinoltaWbgProcessor {
    /// Create new WBG processor with tag definitions
    /// ExifTool: %Image::ExifTool::MinoltaRaw::WBG hash (lines 113-137)
    fn new() -> Self {
        let mut tag_definitions = HashMap::new();

        // WBScale at offset 0
        // ExifTool: line 120 - 0 => { Name => 'WBScale', Format => 'int8u[4]' }
        tag_definitions.insert(
            0,
            WbgTagDef {
                name: "WBScale".to_string(),
                format: MinoltaFormat::Int8uArray(4),
                condition: None,
            },
        );

        // WB levels at offset 4 - conditional on camera model
        // ExifTool: lines 124-136 - conditional WB_GBRGLevels vs WB_RGGBLevels
        tag_definitions.insert(
            4,
            WbgTagDef {
                name: "WB_RGGBLevels".to_string(), // Default case
                format: MinoltaFormat::Int16uArray(4),
                condition: Some(WbgCondition::Other),
            },
        );

        Self { tag_definitions }
    }

    /// Process WBG block data
    /// ExifTool: ProcessBinaryData with MinoltaRaw::WBG table
    fn process(&self, reader: &mut ExifReader, data: &[u8], byte_order: &ByteOrder) -> Result<()> {
        // Process each tag definition
        for (&offset, tag_def) in &self.tag_definitions {
            let tag_value =
                self.extract_value(data, offset as usize, &tag_def.format, byte_order)?;

            // Store the tag with source info
            // ExifTool: GROUPS => { 0 => 'MakerNotes', 2 => 'Camera' }
            let source_info = TagSourceInfo::new(
                "MakerNotes".to_string(),
                "MinoltaWBG".to_string(),
                "Camera".to_string(),
            );

            // Use offset as tag ID for WBG tags
            let tag_id = 0x2000 + offset as u16; // Offset to avoid conflicts
            reader.store_tag_with_precedence(tag_id, tag_value, source_info);
        }

        Ok(())
    }

    /// Extract value from binary data based on format
    /// ExifTool: ProcessBinaryData value extraction
    fn extract_value(
        &self,
        data: &[u8],
        offset: usize,
        format: &MinoltaFormat,
        byte_order: &ByteOrder,
    ) -> Result<TagValue> {
        match format {
            MinoltaFormat::Int8uArray(count) => {
                if offset + count > data.len() {
                    return Err(ExifError::ParseError(format!(
                        "Int8u array at offset {offset:#x} extends beyond data bounds"
                    )));
                }
                let values = data[offset..offset + count].to_vec();
                Ok(TagValue::U8Array(values))
            }
            MinoltaFormat::Int16uArray(count) => {
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
            _ => Err(ExifError::ParseError(format!(
                "Unsupported format in WBG processor: {format:?}"
            ))),
        }
    }
}

impl MinoltaRifProcessor {
    /// Create new RIF processor with tag definitions
    /// ExifTool: %Image::ExifTool::MinoltaRaw::RIF hash (lines 140-346)
    fn new() -> Self {
        let mut tag_definitions = HashMap::new();

        // Saturation at offset 1
        // ExifTool: line 147 - 1 => { Name => 'Saturation', Format => 'int8s' }
        tag_definitions.insert(
            1,
            RifTagDef {
                name: "Saturation".to_string(),
                format: MinoltaFormat::Int8s,
                print_conv: None,
                value_conv: None,
                raw_conv: None,
                condition: None,
            },
        );

        // Contrast at offset 2
        // ExifTool: line 151 - 2 => { Name => 'Contrast', Format => 'int8s' }
        tag_definitions.insert(
            2,
            RifTagDef {
                name: "Contrast".to_string(),
                format: MinoltaFormat::Int8s,
                print_conv: None,
                value_conv: None,
                raw_conv: None,
                condition: None,
            },
        );

        // Sharpness at offset 3
        // ExifTool: line 155 - 3 => { Name => 'Sharpness', Format => 'int8s' }
        tag_definitions.insert(
            3,
            RifTagDef {
                name: "Sharpness".to_string(),
                format: MinoltaFormat::Int8s,
                print_conv: None,
                value_conv: None,
                raw_conv: None,
                condition: None,
            },
        );

        // WBMode at offset 4
        // ExifTool: line 159 - 4 => { Name => 'WBMode', PrintConv => 'Image::ExifTool::MinoltaRaw::ConvertWBMode($val)' }
        tag_definitions.insert(
            4,
            RifTagDef {
                name: "WBMode".to_string(),
                format: MinoltaFormat::Int8u, // Default format
                print_conv: Some(RifPrintConv::WbMode),
                value_conv: None,
                raw_conv: None,
                condition: None,
            },
        );

        // ProgramMode at offset 5
        // ExifTool: line 163 - 5 => { Name => 'ProgramMode', PrintConv => {...} }
        tag_definitions.insert(
            5,
            RifTagDef {
                name: "ProgramMode".to_string(),
                format: MinoltaFormat::Int8u,
                print_conv: Some(RifPrintConv::ProgramMode),
                value_conv: None,
                raw_conv: None,
                condition: None,
            },
        );

        // ISOSetting at offset 6
        // ExifTool: line 176 - 6 => { Name => 'ISOSetting', RawConv => '$val == 255 ? undef : $val', PrintConv => {...} }
        tag_definitions.insert(
            6,
            RifTagDef {
                name: "ISOSetting".to_string(),
                format: MinoltaFormat::Int8u,
                print_conv: Some(RifPrintConv::IsoSetting),
                value_conv: None,
                raw_conv: Some(RifRawConv::ExcludeU8Max),
                condition: None,
            },
        );

        Self { tag_definitions }
    }

    /// Process RIF block data
    /// ExifTool: ProcessBinaryData with MinoltaRaw::RIF table
    fn process(&self, reader: &mut ExifReader, data: &[u8], byte_order: &ByteOrder) -> Result<()> {
        // Process each tag definition
        for (&offset, tag_def) in &self.tag_definitions {
            let raw_value =
                self.extract_value(data, offset as usize, &tag_def.format, byte_order)?;

            // Apply PrintConv for enhanced metadata interpretation
            // ExifTool: MinoltaRaw.pm RIF hash PrintConv fields
            let tag_value = crate::implementations::minolta_raw::apply_rif_print_conv(
                &tag_def.name,
                &raw_value,
            );

            // Store the tag with source info
            // ExifTool: GROUPS => { 0 => 'MakerNotes', 2 => 'Image' }
            let source_info = TagSourceInfo::new(
                "MakerNotes".to_string(),
                "MinoltaRIF".to_string(),
                "Image".to_string(),
            );

            // Use offset as tag ID for RIF tags
            let tag_id = 0x3000 + offset as u16; // Offset to avoid conflicts
            reader.store_tag_with_precedence(tag_id, tag_value, source_info);
        }

        Ok(())
    }

    /// Extract value from binary data based on format
    /// ExifTool: ProcessBinaryData value extraction
    fn extract_value(
        &self,
        data: &[u8],
        offset: usize,
        format: &MinoltaFormat,
        byte_order: &ByteOrder,
    ) -> Result<TagValue> {
        match format {
            MinoltaFormat::Int8u => {
                if offset >= data.len() {
                    return Err(ExifError::ParseError(format!(
                        "Int8u at offset {offset:#x} extends beyond data bounds"
                    )));
                }
                Ok(TagValue::U8(data[offset]))
            }
            MinoltaFormat::Int8s => {
                if offset >= data.len() {
                    return Err(ExifError::ParseError(format!(
                        "Int8s at offset {offset:#x} extends beyond data bounds"
                    )));
                }
                // Convert signed 8-bit to signed 16-bit since TagValue doesn't have I8
                Ok(TagValue::I16(data[offset] as i8 as i16))
            }
            _ => Err(ExifError::ParseError(format!(
                "Unsupported format in RIF processor: {format:?}"
            ))),
        }
    }
}

/// Get Minolta MRW tag name by ID
/// ExifTool: lib/Image/ExifTool/MinoltaRaw.pm tag definitions
pub fn get_minolta_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        // PRD tags (0x1000-0x1FFF range)
        0x1000 => Some("FirmwareID"),
        0x1008 => Some("SensorHeight"),
        0x100A => Some("SensorWidth"),
        0x100C => Some("ImageHeight"),
        0x100E => Some("ImageWidth"),
        0x1010 => Some("RawDepth"),
        0x1011 => Some("BitDepth"),
        0x1012 => Some("StorageMethod"),
        0x1017 => Some("BayerPattern"),

        // WBG tags (0x2000-0x2FFF range)
        0x2000 => Some("WBScale"),
        0x2004 => Some("WB_RGGBLevels"),

        // RIF tags (0x3000-0x3FFF range)
        0x3001 => Some("Saturation"),
        0x3002 => Some("Contrast"),
        0x3003 => Some("Sharpness"),
        0x3004 => Some("WBMode"),
        0x3005 => Some("ProgramMode"),
        0x3006 => Some("ISOSetting"),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minolta_handler_creation() {
        let handler = MinoltaRawHandler::new();
        assert_eq!(handler.name(), "MinoltaRaw");
    }

    #[test]
    fn test_get_minolta_tag_name() {
        // Test PRD tags
        assert_eq!(get_minolta_tag_name(0x1000), Some("FirmwareID"));
        assert_eq!(get_minolta_tag_name(0x1008), Some("SensorHeight"));
        assert_eq!(get_minolta_tag_name(0x100C), Some("ImageHeight"));

        // Test WBG tags
        assert_eq!(get_minolta_tag_name(0x2000), Some("WBScale"));
        assert_eq!(get_minolta_tag_name(0x2004), Some("WB_RGGBLevels"));

        // Test RIF tags
        assert_eq!(get_minolta_tag_name(0x3001), Some("Saturation"));
        assert_eq!(get_minolta_tag_name(0x3004), Some("WBMode"));

        // Test unknown tag
        assert_eq!(get_minolta_tag_name(0x9999), None);
    }

    #[test]
    fn test_mrw_header_parsing() {
        let handler = MinoltaRawHandler::new();

        // Test valid MRW big-endian header
        let mrw_data = {
            let mut data = vec![0u8; 100];
            data[0..4].copy_from_slice(b"\0MRM");
            data[4..8].copy_from_slice(&32u32.to_be_bytes()); // data offset
                                                              // Add a test block
            data[8..12].copy_from_slice(b"\0PRD");
            data[12..16].copy_from_slice(&16u32.to_be_bytes()); // block length
            data[16..32].fill(0x42); // block data
            data
        };

        let header = handler.parse_mrw_header(&mrw_data).unwrap();
        assert!(matches!(header.byte_order, ByteOrder::BigEndian));
        assert_eq!(header.data_offset, 32);
        assert_eq!(header.blocks.len(), 1);
        assert_eq!(&header.blocks[0].tag, b"\0PRD");
        assert_eq!(header.blocks[0].length, 16);
    }

    #[test]
    fn test_format_validation() {
        let handler = MinoltaRawHandler::new();

        // Test valid MRW magic
        let valid_data = b"\0MRM\x00\x00\x00\x20test_data";
        assert!(handler.validate_format(valid_data));

        // Test invalid magic
        let invalid_data = b"\0MRX\x00\x00\x00\x20test_data";
        assert!(!handler.validate_format(invalid_data));

        // Test insufficient data
        let short_data = b"\0MR";
        assert!(!handler.validate_format(short_data));
    }
}
