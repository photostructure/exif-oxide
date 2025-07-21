//! Olympus-specific MakerNote processing
//!
//! This module implements Olympus MakerNote detection following ExifTool's Olympus processing
//! from MakerNotes.pm, focusing on proper namespace handling and binary data processing.
//!
//! **Trust ExifTool**: This code translates ExifTool's Olympus detection patterns verbatim
//! without any improvements or simplifications. Every detection pattern and signature
//! is copied exactly as documented in the ExifTool source.
//!
//! Primary ExifTool References:
//! - lib/Image/ExifTool/MakerNotes.pm:515-533 - Olympus MakerNote detection patterns
//! - lib/Image/ExifTool/Olympus.pm - Olympus tag tables and processing

// Equipment tag lookup now handled by generated code

use tracing::trace;

/// Olympus MakerNote signature patterns from ExifTool
/// ExifTool: lib/Image/ExifTool/MakerNotes.pm:515-533 Olympus detection conditions
#[derive(Debug, Clone, PartialEq)]
pub enum OlympusSignature {
    /// Older Olympus/Epson format starting with "OLYMP\0" or "EPSON\0"
    /// ExifTool: MakerNoteOlympus Condition '$$valPt =~ /^(OLYMP|EPSON)\0/'
    OlympusOld,
    /// Newer Olympus format starting with "OLYMPUS\0"
    /// ExifTool: MakerNoteOlympus2 Condition '$$valPt =~ /^OLYMPUS\0/'
    OlympusNew,
    /// Newest OM System format starting with "OM SYSTEM\0"
    /// ExifTool: MakerNoteOlympus3 Condition '$$valPt =~ /^OM SYSTEM\0/'
    OmSystem,
}

impl OlympusSignature {
    /// Get the byte offset to the actual maker note data
    /// ExifTool: Start parameter in SubDirectory definitions
    pub fn data_offset(&self) -> usize {
        match self {
            OlympusSignature::OlympusOld => 8,  // Start => '$valuePtr + 8'
            OlympusSignature::OlympusNew => 12, // Start => '$valuePtr + 12'
            OlympusSignature::OmSystem => 16,   // Start => '$valuePtr + 16'
        }
    }

    /// Get the base offset adjustment
    /// ExifTool: Base parameter in SubDirectory definitions  
    pub fn base_offset(&self) -> i32 {
        match self {
            OlympusSignature::OlympusOld => 0,   // No Base adjustment
            OlympusSignature::OlympusNew => -12, // Base => '$start - 12'
            OlympusSignature::OmSystem => -16,   // Base => '$start - 16'
        }
    }
}

/// Detect Olympus MakerNote signature from binary data and Make field
/// ExifTool: lib/Image/ExifTool/MakerNotes.pm:515-533 Olympus detection logic
pub fn detect_olympus_signature(_make: &str, maker_note_data: &[u8]) -> Option<OlympusSignature> {
    if maker_note_data.is_empty() {
        return None;
    }

    // Priority order matches ExifTool's table order in MakerNotes.pm

    // 1. MakerNoteOlympus3: OM SYSTEM (newest format)
    // ExifTool: MakerNotes.pm:530 '$$valPt =~ /^OM SYSTEM\0/'
    if maker_note_data.starts_with(b"OM SYSTEM\0") {
        trace!("Detected OM System signature");
        return Some(OlympusSignature::OmSystem);
    }

    // 2. MakerNoteOlympus2: OLYMPUS\0 (newer format)
    // ExifTool: MakerNotes.pm:523 '$$valPt =~ /^OLYMPUS\0/'
    if maker_note_data.starts_with(b"OLYMPUS\0") {
        trace!("Detected Olympus new signature");
        return Some(OlympusSignature::OlympusNew);
    }

    // 3. MakerNoteOlympus: OLYMP\0 or EPSON\0 (older format)
    // ExifTool: MakerNotes.pm:516 '$$valPt =~ /^(OLYMP|EPSON)\0/'
    if maker_note_data.starts_with(b"OLYMP\0") || maker_note_data.starts_with(b"EPSON\0") {
        trace!("Detected Olympus old signature (OLYMP/EPSON)");
        return Some(OlympusSignature::OlympusOld);
    }

    // No Olympus signature detected
    None
}

/// Detect if this is an Olympus MakerNote based on Make field
/// This is used as a fallback when signature detection fails
pub fn is_olympus_makernote(make: &str) -> bool {
    // ExifTool: Check if Make field indicates Olympus
    make.starts_with("OLYMPUS") || make == "OM Digital Solutions"
}
