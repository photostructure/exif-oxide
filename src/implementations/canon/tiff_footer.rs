//! Canon TIFF footer parsing and MakerNote base offset fixing
//!
//! This module handles Canon-specific TIFF footer parsing and MakerNote base offset
//! calculation. Canon MakerNotes include an 8-byte TIFF footer that contains the
//! original offset used for validation and offset adjustment.
//!
//! **Trust ExifTool**: This code translates ExifTool's Canon TIFF footer handling verbatim
//! without any improvements or simplifications. Every algorithm, magic number, and
//! quirk is copied exactly as documented in the ExifTool source.
//!
//! Primary ExifTool References:
//! - lib/Image/ExifTool/MakerNotes.pm:1281-1307 FixBase Canon section

use crate::tiff_types::ByteOrder;
use crate::types::{ExifError, Result};
use std::collections::HashMap;
use tracing::{debug, warn};

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
    use crate::implementations::canon::offset_schemes;

    // Only process Canon maker notes
    // ExifTool: MakerNotes.pm:1281 if ($$et{Make} =~ /^Canon/
    if !offset_schemes::detect_canon_signature(params.make) {
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
                    offset_schemes::detect_fallback_offset_scheme(params.model)
                }
            }
        }
        Err(e) => {
            debug!(
                "Canon TIFF footer parsing failed: {}, falling back to offset scheme detection",
                e
            );
            // Fall back to offset scheme detection when footer is not valid
            offset_schemes::detect_fallback_offset_scheme(params.model)
        }
    }
}
