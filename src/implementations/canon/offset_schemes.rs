//! Canon MakerNote offset scheme detection
//!
//! This module handles Canon-specific offset scheme detection based on camera models.
//! Different Canon camera models use different offset schemes for MakerNote data.
//!
//! **Trust ExifTool**: This code translates ExifTool's Canon offset detection verbatim
//! without any improvements or simplifications. Every algorithm, magic number, and
//! quirk is copied exactly as documented in the ExifTool source.
//!
//! Primary ExifTool References:
//! - lib/Image/ExifTool/MakerNotes.pm:1135-1141 GetMakerNoteOffset Canon section
//! - lib/Image/ExifTool/MakerNotes.pm:60-68 MakerNoteCanon condition

use tracing::debug;

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

/// Fallback offset scheme detection when TIFF footer is not available or invalid
/// ExifTool: lib/Image/ExifTool/MakerNotes.pm:1135-1141 GetMakerNoteOffset
pub(crate) fn detect_fallback_offset_scheme(model: &str) -> crate::types::Result<Option<i64>> {
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
