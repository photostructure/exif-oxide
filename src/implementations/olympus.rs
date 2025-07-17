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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_olympus_signature_detection() {
        // Test OM SYSTEM signature (newest)
        let om_data = b"OM SYSTEM\0\x00\x00\x00\x00test data";
        assert_eq!(
            detect_olympus_signature("OM Digital Solutions", om_data),
            Some(OlympusSignature::OmSystem)
        );

        // Test OLYMPUS signature (newer)
        let olympus_new_data = b"OLYMPUS\0\x00\x00\x00\x00test data";
        assert_eq!(
            detect_olympus_signature("OLYMPUS CORPORATION", olympus_new_data),
            Some(OlympusSignature::OlympusNew)
        );

        // Test OLYMP signature (older)
        let olympus_old_data = b"OLYMP\0\x00\x00test data";
        assert_eq!(
            detect_olympus_signature("OLYMPUS OPTICAL CO.,LTD", olympus_old_data),
            Some(OlympusSignature::OlympusOld)
        );

        // Test EPSON signature (older variant)
        let epson_data = b"EPSON\0\x00\x00test data";
        assert_eq!(
            detect_olympus_signature("SEIKO EPSON CORP.", epson_data),
            Some(OlympusSignature::OlympusOld)
        );

        // Test no signature
        let no_sig_data = b"CANON\0\x00\x00test data";
        assert_eq!(detect_olympus_signature("Canon", no_sig_data), None);

        // Test empty data
        assert_eq!(detect_olympus_signature("OLYMPUS", &[]), None);
    }

    #[test]
    fn test_olympus_offsets() {
        assert_eq!(OlympusSignature::OlympusOld.data_offset(), 8);
        assert_eq!(OlympusSignature::OlympusOld.base_offset(), 0);

        assert_eq!(OlympusSignature::OlympusNew.data_offset(), 12);
        assert_eq!(OlympusSignature::OlympusNew.base_offset(), -12);

        assert_eq!(OlympusSignature::OmSystem.data_offset(), 16);
        assert_eq!(OlympusSignature::OmSystem.base_offset(), -16);
    }

    #[test]
    fn test_is_olympus_makernote() {
        assert!(is_olympus_makernote("OLYMPUS CORPORATION"));
        assert!(is_olympus_makernote("OLYMPUS OPTICAL CO.,LTD"));
        assert!(is_olympus_makernote("OLYMPUS"));
        assert!(is_olympus_makernote("OM Digital Solutions"));

        assert!(!is_olympus_makernote("Canon"));
        assert!(!is_olympus_makernote("Nikon"));
        assert!(!is_olympus_makernote("Sony"));
    }
}
