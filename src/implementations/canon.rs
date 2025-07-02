//! Canon-specific MakerNote processing
//!
//! This module implements Canon MakerNote detection, offset fixing, and processing
//! following ExifTool's Canon.pm implementation exactly.
//!
//! **ExifTool is Gospel**: This code translates ExifTool's Canon processing verbatim
//! without any improvements or simplifications. Every algorithm, magic number, and
//! quirk is copied exactly as documented in the ExifTool source.
//!
//! Primary ExifTool References:
//! - lib/Image/ExifTool/Canon.pm - Canon tag tables and processing
//! - lib/Image/ExifTool/MakerNotes.pm - Canon MakerNote detection and offset fixing

use crate::exif::ByteOrder;
use crate::types::{ExifError, Result, TagValue};
use std::collections::HashMap;
use tracing::{debug, warn};

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
    // ExifTool: MakerNotes.pm:62 '$$self{Make} =~ /^Canon/'
    make.starts_with("Canon")
}

/// Detect Canon offset scheme based on camera model
/// ExifTool: lib/Image/ExifTool/MakerNotes.pm:1135-1141 GetMakerNoteOffset Canon section
pub fn detect_offset_scheme(model: &str) -> CanonOffsetScheme {
    // ExifTool: MakerNotes.pm:1136
    // push @offsets, ($model =~ /\b(20D|350D|REBEL XT|Kiss Digital N)\b/) ? 6 : 4;
    if model.contains("20D")
        || model.contains("350D")
        || model.contains("REBEL XT")
        || model.contains("Kiss Digital N")
    {
        return CanonOffsetScheme::SixByte;
    }

    // ExifTool: MakerNotes.pm:1137-1139
    // some Canon models (FV-M30, Optura50, Optura60) leave 24 unused bytes
    // at the end of the IFD (2 spare IFD entries?)
    // push @offsets, 28 if $model =~ /\b(FV\b|OPTURA)/;
    if model.contains("FV") || model.contains("OPTURA") {
        return CanonOffsetScheme::TwentyEightByte;
    }

    // ExifTool: MakerNotes.pm:1140-1141
    // some Canon PowerShot models leave 12 unused bytes
    // push @offsets, 16 if $model =~ /(PowerShot|IXUS|IXY)/;
    if model.contains("PowerShot") || model.contains("IXUS") || model.contains("IXY") {
        return CanonOffsetScheme::SixteenByte;
    }

    // ExifTool: MakerNotes.pm:1136 default case
    CanonOffsetScheme::FourByte
}

/// Canon TIFF footer structure for offset validation
/// ExifTool: lib/Image/ExifTool/MakerNotes.pm:1281-1307 FixBase Canon footer handling
#[derive(Debug, Clone)]
pub struct CanonTiffFooter {
    /// TIFF header bytes: "II\x2a\0" (little-endian) or "MM\0\x2a" (big-endian)
    /// ExifTool: MakerNotes.pm:1284 footer =~ /^(II\x2a\0|MM\0\x2a)/
    pub tiff_header: [u8; 4],
    /// Original maker note offset stored in footer
    /// ExifTool: MakerNotes.pm:1287 my $oldOffset = Get32u(\$footer, 4);
    pub original_offset: u32,
}

impl CanonTiffFooter {
    /// Parse Canon TIFF footer from 8-byte data
    /// ExifTool: lib/Image/ExifTool/MakerNotes.pm:1283-1285
    pub fn parse(footer_data: &[u8], byte_order: ByteOrder) -> Result<Self> {
        if footer_data.len() < 8 {
            return Err(ExifError::ParseError(
                "Canon TIFF footer too short (need 8 bytes)".to_string(),
            ));
        }

        // ExifTool: MakerNotes.pm:1284 check for TIFF footer
        let tiff_header = [
            footer_data[0],
            footer_data[1],
            footer_data[2],
            footer_data[3],
        ];

        // ExifTool: MakerNotes.pm:1284 footer =~ /^(II\x2a\0|MM\0\x2a)/
        let valid_header = match &tiff_header {
            [0x49, 0x49, 0x2a, 0x00] => true, // "II\x2a\0" - little-endian
            [0x4d, 0x4d, 0x00, 0x2a] => true, // "MM\0\x2a" - big-endian
            _ => false,
        };

        if !valid_header {
            return Err(ExifError::ParseError(
                "Invalid Canon TIFF footer header".to_string(),
            ));
        }

        // ExifTool: MakerNotes.pm:1285 validate byte ordering
        // substr($footer,0,2) eq GetByteOrder()
        let footer_byte_order = match &tiff_header[0..2] {
            [0x49, 0x49] => ByteOrder::LittleEndian,
            [0x4d, 0x4d] => ByteOrder::BigEndian,
            _ => {
                return Err(ExifError::ParseError(
                    "Invalid Canon TIFF footer byte order".to_string(),
                ))
            }
        };

        if footer_byte_order != byte_order {
            return Err(ExifError::ParseError(
                "Canon TIFF footer byte order mismatch".to_string(),
            ));
        }

        // ExifTool: MakerNotes.pm:1287 my $oldOffset = Get32u(\$footer, 4);
        let original_offset = byte_order.read_u32(footer_data, 4)?;

        Ok(CanonTiffFooter {
            tiff_header,
            original_offset,
        })
    }

    /// Validate Canon TIFF footer against expected values
    /// ExifTool: lib/Image/ExifTool/MakerNotes.pm:1287-1307
    pub fn validate_offset(
        &self,
        dir_start: usize,
        data_pos: u64,
        dir_len: usize,
        val_ptrs: &[usize],
        val_block: &HashMap<usize, usize>,
    ) -> Result<Option<i64>> {
        // ExifTool: MakerNotes.pm:1288 my $newOffset = $dirStart + $dataPos;
        let new_offset = dir_start as u64 + data_pos;

        // ExifTool: MakerNotes.pm:1292 $fix = $newOffset - $oldOffset;
        let fix = new_offset as i64 - self.original_offset as i64;

        if fix == 0 {
            // No adjustment needed
            return Ok(None);
        }

        // ExifTool: MakerNotes.pm:1294-1305
        // Picasa and ACDSee have a bug where they update other offsets without
        // updating the TIFF footer (PH - 2009/02/25), so test for this case:
        // validate Canon maker note footer fix by checking offset of last value
        if let Some(&last_ptr) = val_ptrs.last() {
            if let Some(&last_size) = val_block.get(&last_ptr) {
                // ExifTool: MakerNotes.pm:1297 my $maxPt = $valPtrs[-1] + $$valBlock{$valPtrs[-1]};
                let max_pt = last_ptr + last_size;

                // ExifTool: MakerNotes.pm:1299
                // compare to end of maker notes, taking 8-byte footer into account
                // my $endDiff = $dirStart + $$dirInfo{DirLen} - ($maxPt - $dataPos) - 8;
                let end_diff = (dir_start + dir_len) as i64 - (max_pt as i64 - data_pos as i64) - 8;

                // ExifTool: MakerNotes.pm:1301-1302
                // ignore footer offset only if end difference is exactly correct
                // (allow for possible padding byte, although I have never seen this)
                // if (not $endDiff or $endDiff == 1)
                if end_diff == 0 || end_diff == 1 {
                    warn!("Canon maker note footer may be invalid (ignored)");
                    return Ok(None); // Ignore footer offset - ExifTool: return 0
                }
            }
        }

        Ok(Some(fix))
    }
}

/// Parameters for Canon MakerNote base fixing
/// Groups related parameters to reduce function argument count
#[derive(Debug)]
pub struct CanonFixBaseParams<'a> {
    pub make: &'a str,
    pub model: &'a str,
    pub maker_note_data: &'a [u8],
    pub dir_start: usize,
    pub dir_len: usize,
    pub data_pos: u64,
    pub byte_order: ByteOrder,
    pub val_ptrs: &'a [usize],
    pub val_block: &'a HashMap<usize, usize>,
}

/// Canon MakerNote base offset fixing
/// ExifTool: lib/Image/ExifTool/MakerNotes.pm:1281-1307 FixBase Canon section
#[allow(clippy::too_many_arguments)]
pub fn fix_maker_note_base(
    make: &str,
    model: &str,
    maker_note_data: &[u8],
    dir_start: usize,
    dir_len: usize,
    data_pos: u64,
    byte_order: ByteOrder,
    val_ptrs: &[usize],
    val_block: &HashMap<usize, usize>,
) -> Result<Option<i64>> {
    // Create params struct and delegate to the new implementation
    let params = CanonFixBaseParams {
        make,
        model,
        maker_note_data,
        dir_start,
        dir_len,
        data_pos,
        byte_order,
        val_ptrs,
        val_block,
    };
    fix_maker_note_base_impl(&params)
}

/// Canon MakerNote base offset fixing implementation
/// ExifTool: lib/Image/ExifTool/MakerNotes.pm:1281-1307 FixBase Canon section
fn fix_maker_note_base_impl(params: &CanonFixBaseParams) -> Result<Option<i64>> {
    // Only process Canon maker notes
    // ExifTool: MakerNotes.pm:1281 if ($$et{Make} =~ /^Canon/
    if !detect_canon_signature(params.make) {
        return Ok(None);
    }

    // ExifTool: MakerNotes.pm:1281 and $$dirInfo{DirLen} > 8)
    if params.dir_len <= 8 {
        debug!(
            "Canon maker note directory too small for footer (need > 8 bytes, have {})",
            params.dir_len
        );
        return Ok(None);
    }

    // ExifTool: MakerNotes.pm:1282 my $footerPos = $dirStart + $$dirInfo{DirLen} - 8;
    let footer_pos = params.dir_start + params.dir_len - 8;

    if footer_pos + 8 > params.maker_note_data.len() {
        warn!("Canon TIFF footer position beyond data bounds");
        return Ok(None);
    }

    // ExifTool: MakerNotes.pm:1283 my $footer = substr($$dataPt, $footerPos, 8);
    let footer_data = &params.maker_note_data[footer_pos..footer_pos + 8];

    // Parse and validate Canon TIFF footer
    match CanonTiffFooter::parse(footer_data, params.byte_order) {
        Ok(footer) => {
            debug!(
                "Found Canon TIFF footer at offset {:#x}, original offset: {:#x}",
                footer_pos, footer.original_offset
            );

            // Validate the footer and get the base adjustment
            match footer.validate_offset(
                params.dir_start,
                params.data_pos,
                params.dir_len,
                params.val_ptrs,
                params.val_block,
            ) {
                Ok(Some(fix)) => {
                    debug!("Canon maker note base adjustment: {}", fix);
                    Ok(Some(fix))
                }
                Ok(None) => {
                    debug!("Canon maker note footer validation: no adjustment needed");
                    Ok(None)
                }
                Err(e) => {
                    warn!("Canon TIFF footer validation failed: {}", e);
                    // Fall back to offset scheme detection
                    detect_fallback_offset_scheme(params.model)
                }
            }
        }
        Err(e) => {
            debug!(
                "Canon TIFF footer parsing failed: {}, falling back to offset scheme detection",
                e
            );
            // Fall back to offset scheme detection when footer is not valid
            detect_fallback_offset_scheme(params.model)
        }
    }
}

/// Fallback offset scheme detection when TIFF footer is not available or invalid
/// ExifTool: lib/Image/ExifTool/MakerNotes.pm:1135-1141 GetMakerNoteOffset
fn detect_fallback_offset_scheme(model: &str) -> Result<Option<i64>> {
    let scheme = detect_offset_scheme(model);

    // For fallback, we assume the default scheme provides the expected offset
    // The actual offset fixing would need more context about the current base
    // This is a simplified version - full implementation would require
    // the complete directory analysis that ExifTool's FixBase does
    debug!(
        "Using Canon fallback offset scheme: {:?} ({} bytes)",
        scheme,
        scheme.as_bytes()
    );

    // Return None for now - the caller should handle offset scheme application
    // TODO: Implement full offset calculation logic matching ExifTool's FixBase
    Ok(None)
}

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
                conv.insert(1, "Macro".to_string());
                conv.insert(2, "Normal".to_string());
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
                conv.insert(0, "Off".to_string());
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
                conv.insert(-1, "n/a".to_string()); // PH, EOS M MOV video
                conv.insert(0, "Off".to_string());
                conv.insert(1, "Auto".to_string());
                conv.insert(2, "On".to_string());
                conv.insert(3, "Red-eye reduction".to_string());
                conv.insert(4, "Slow-sync".to_string());
                conv.insert(5, "Red-eye reduction (Auto)".to_string());
                conv.insert(6, "Red-eye reduction (On)".to_string());
                conv.insert(16, "External flash".to_string()); // not set in D30 or 300D
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
                conv.insert(0, "Single".to_string());
                conv.insert(1, "Continuous".to_string());
                conv.insert(2, "Movie".to_string()); // PH
                conv.insert(3, "Continuous, Speed Priority".to_string()); // PH
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
                conv.insert(0, "One-shot AF".to_string());
                conv.insert(1, "AI Servo AF".to_string());
                conv.insert(2, "AI Focus AF".to_string());
                conv.insert(3, "Manual Focus (3)".to_string());
                conv.insert(4, "Single".to_string());
                conv.insert(5, "Continuous".to_string());
                conv.insert(6, "Manual Focus (6)".to_string());
                conv.insert(16, "Pan Focus".to_string()); // PH
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canon_signature_detection() {
        assert!(detect_canon_signature("Canon"));
        assert!(detect_canon_signature("Canon EOS REBEL T3i"));
        assert!(!detect_canon_signature("Nikon"));
        assert!(!detect_canon_signature("Sony"));
    }

    #[test]
    fn test_offset_scheme_detection() {
        // Test default case
        assert_eq!(
            detect_offset_scheme("Canon EOS 5D"),
            CanonOffsetScheme::FourByte
        );

        // Test 6-byte models
        assert_eq!(
            detect_offset_scheme("Canon EOS 20D"),
            CanonOffsetScheme::SixByte
        );
        assert_eq!(
            detect_offset_scheme("Canon EOS 350D"),
            CanonOffsetScheme::SixByte
        );
        assert_eq!(
            detect_offset_scheme("Canon EOS REBEL XT"),
            CanonOffsetScheme::SixByte
        );

        // Test 28-byte models
        assert_eq!(
            detect_offset_scheme("Canon FV-M30"),
            CanonOffsetScheme::TwentyEightByte
        );
        assert_eq!(
            detect_offset_scheme("Canon OPTURA 60"),
            CanonOffsetScheme::TwentyEightByte
        );

        // Test 16-byte models
        assert_eq!(
            detect_offset_scheme("Canon PowerShot S70"),
            CanonOffsetScheme::SixteenByte
        );
        assert_eq!(
            detect_offset_scheme("Canon IXUS 400"),
            CanonOffsetScheme::SixteenByte
        );
    }

    #[test]
    fn test_canon_tiff_footer_parse() {
        // Test little-endian footer
        let le_footer = [0x49, 0x49, 0x2a, 0x00, 0x10, 0x00, 0x00, 0x00];
        let footer = CanonTiffFooter::parse(&le_footer, ByteOrder::LittleEndian).unwrap();
        assert_eq!(footer.original_offset, 0x10);

        // Test big-endian footer
        let be_footer = [0x4d, 0x4d, 0x00, 0x2a, 0x00, 0x00, 0x00, 0x10];
        let footer = CanonTiffFooter::parse(&be_footer, ByteOrder::BigEndian).unwrap();
        assert_eq!(footer.original_offset, 0x10);

        // Test invalid header
        let invalid_footer = [0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00];
        assert!(CanonTiffFooter::parse(&invalid_footer, ByteOrder::LittleEndian).is_err());

        // Test byte order mismatch
        assert!(CanonTiffFooter::parse(&le_footer, ByteOrder::BigEndian).is_err());
    }

    #[test]
    fn test_camera_settings_table() {
        let table = create_camera_settings_table();

        // Test MacroMode tag
        let macro_tag = table.get(&1).unwrap();
        assert_eq!(macro_tag.name, "MacroMode");
        assert_eq!(
            macro_tag.print_conv.as_ref().unwrap().get(&1),
            Some(&"Macro".to_string())
        );

        // Test FocusMode tag
        let focus_tag = table.get(&7).unwrap();
        assert_eq!(focus_tag.name, "FocusMode");
        assert_eq!(
            focus_tag.print_conv.as_ref().unwrap().get(&0),
            Some(&"One-shot AF".to_string())
        );
    }
}
