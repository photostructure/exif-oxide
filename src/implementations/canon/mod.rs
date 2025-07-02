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
