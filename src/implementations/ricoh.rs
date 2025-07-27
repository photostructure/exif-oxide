//! RICOH MakerNote signature detection and offset calculation
//!
//! RICOH cameras use different MakerNote formats depending on the model.
//! This module implements signature detection following ExifTool's logic.
//!
//! ExifTool Reference: lib/Image/ExifTool/MakerNotes.pm lines 873-924

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RicohSignature {
    /// Standard IFD format with "Ricoh" + 0xcf signature (6 bytes total)
    /// ExifTool: MakerNoteRicoh condition
    StandardIfd,
    /// Pentax-compatible format for newer models (GR III+)
    /// ExifTool: MakerNoteRicohPentax condition
    PentaxCompatible,
    /// Special TIFF-like format with padding issues (HZ15, XG-1)
    /// ExifTool: MakerNoteRicoh2 condition
    TiffLike,
    /// Text-based format for DC/RDC models
    /// ExifTool: MakerNoteRicohText condition
    TextFormat,
}

impl RicohSignature {
    /// Get the data offset for this signature type
    /// This is how many bytes to skip before the actual IFD/data starts
    pub fn data_offset(self) -> usize {
        match self {
            RicohSignature::StandardIfd => 8, // ExifTool: Start => '$valuePtr + 8'
            RicohSignature::PentaxCompatible => 0, // No signature prefix
            RicohSignature::TiffLike => 8,    // MM/II + special header
            RicohSignature::TextFormat => 0,  // No IFD structure
        }
    }

    /// Get the base offset for subdirectory calculations
    pub fn base_offset(self) -> i64 {
        match self {
            RicohSignature::StandardIfd => 0, // Relative to original MakerNotes offset
            RicohSignature::PentaxCompatible => 0,
            RicohSignature::TiffLike => 0,
            RicohSignature::TextFormat => 0,
        }
    }
}

/// Detect RICOH MakerNote signature following ExifTool's conditional logic
/// ExifTool: lib/Image/ExifTool/MakerNotes.pm lines 873-924
pub fn detect_ricoh_signature(make: &str, maker_note_data: &[u8]) -> Option<RicohSignature> {
    // Must be a RICOH camera
    if !is_ricoh_makernote(make) {
        return None;
    }

    // Need at least 8 bytes to check signatures
    if maker_note_data.len() < 8 {
        return None;
    }

    // Check for Pentax-compatible format first (highest priority)
    // ExifTool: MakerNoteRicohPentax condition (line 873)
    // Requires data to start with "RICOH\0" followed by "II" or "MM"
    if maker_note_data.len() >= 8
        && maker_note_data.starts_with(b"RICOH\x00")
        && (maker_note_data[6..8] == b"II"[..] || maker_note_data[6..8] == b"MM"[..])
    {
        return Some(RicohSignature::PentaxCompatible);
    }

    // Check for standard IFD format with "Ricoh" signature
    // ExifTool: MakerNoteRicoh condition (line 884)
    if maker_note_data.starts_with(b"Ricoh") ||
       maker_note_data.starts_with(b"      ") ||  // 6 spaces
       maker_note_data.starts_with(b"MM\x00\x2a") ||  // Big-endian TIFF
       maker_note_data.starts_with(b"II\x2a\x00")
    {
        // Little-endian TIFF

        // ExifTool exclusion: check for special patterns that indicate Ricoh2 format
        if (maker_note_data.starts_with(b"MM\x00\x2a\x00\x00\x00\x08\x00")
            && maker_note_data.len() > 11)
            || (maker_note_data.starts_with(b"II\x2a\x00\x08\x00\x00\x00")
                && maker_note_data.len() > 10)
        {
            // This matches MakerNoteRicoh2 pattern
            return Some(RicohSignature::TiffLike);
        }

        // Special exclusion for RICOH WG-M1 model
        // ExifTool: "$$self{Model} ne 'RICOH WG-M1'"
        // For now, we'll handle this as standard IFD unless we encounter issues

        return Some(RicohSignature::StandardIfd);
    }

    // Check for text-based format (lowest priority - fallback)
    // ExifTool: MakerNoteRicohText condition (line 917)
    // Text format should start with "Rev" or "Rv"
    if maker_note_data.starts_with(b"Rev") || maker_note_data.starts_with(b"Rv") {
        return Some(RicohSignature::TextFormat);
    }

    // No recognized signature
    None
}

/// Check if the Make field indicates a RICOH camera
/// ExifTool: Various conditions check for RICOH make
pub fn is_ricoh_makernote(make: &str) -> bool {
    make.contains("RICOH") || make.starts_with("PENTAX RICOH")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ricoh_make_detection() {
        assert!(is_ricoh_makernote("RICOH"));
        assert!(is_ricoh_makernote("PENTAX RICOH"));
        assert!(is_ricoh_makernote("RICOH IMAGING COMPANY, LTD."));
        assert!(!is_ricoh_makernote("Canon"));
        assert!(!is_ricoh_makernote("SONY"));
        assert!(!is_ricoh_makernote("PENTAX")); // Must include RICOH
    }

    #[test]
    fn test_ricoh_signature_detection() {
        // Test standard IFD format with "Ricoh" signature
        let ricoh_data = b"Ricoh\xcf\x00\x00\x00\x09";
        assert_eq!(
            detect_ricoh_signature("RICOH", ricoh_data),
            Some(RicohSignature::StandardIfd)
        );

        // Test TIFF format
        let tiff_be_data = b"MM\x00\x2a\x00\x00\x00\x08";
        assert_eq!(
            detect_ricoh_signature("RICOH", tiff_be_data),
            Some(RicohSignature::StandardIfd)
        );

        // Test text format
        let text_data = b"Rev2219;";
        assert_eq!(
            detect_ricoh_signature("RICOH", text_data),
            Some(RicohSignature::TextFormat)
        );

        // Test PENTAX RICOH with TIFF magic - goes to StandardIfd per ExifTool
        // Need at least 8 bytes of data
        assert_eq!(
            detect_ricoh_signature("PENTAX RICOH", b"MM\x00\x2a\x00\x00\x00\x08"),
            Some(RicohSignature::StandardIfd)
        );

        // Test true Pentax-compatible format with RICOH\0 signature
        let pentax_data = b"RICOH\x00II";
        assert_eq!(
            detect_ricoh_signature("PENTAX RICOH", pentax_data),
            Some(RicohSignature::PentaxCompatible)
        );

        // Test non-RICOH make
        assert_eq!(detect_ricoh_signature("Canon", ricoh_data), None);
    }

    #[test]
    fn test_signature_offsets() {
        assert_eq!(RicohSignature::StandardIfd.data_offset(), 8);
        assert_eq!(RicohSignature::PentaxCompatible.data_offset(), 0);
        assert_eq!(RicohSignature::TiffLike.data_offset(), 8);
        assert_eq!(RicohSignature::TextFormat.data_offset(), 0);
    }
}
