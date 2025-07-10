//! Compatibility layer for legacy file_types module references
//!
//! This module provides compatibility functions that were previously in the generated
//! file_types module but are now distributed across different modules in the new
//! simplified codegen architecture.

/// Lookup MIME type for a file format
/// This is a compatibility wrapper for the new location
pub use crate::generated::ExifTool_pm::lookup_mime_types;

/// Resolve file type from extension, following aliases
/// This function previously existed in the generated file_types module
/// but is no longer generated in the new architecture.
///
/// Returns a tuple of (formats, description) where formats is a vector of
/// supported format strings and description is a human-readable description.
pub fn resolve_file_type(extension: &str) -> Option<(Vec<&'static str>, &'static str)> {
    // Based on ExifTool.pm %fileTypeLookup hash structure
    // This is a compatibility shim until the proper codegen is restored

    // Convert to uppercase for case-insensitive lookup
    let ext_upper = extension.to_uppercase();

    // Handle aliases first
    let lookup_key = match ext_upper.as_str() {
        "3GP2" => "3G2",
        "3GPP" => "3GP",
        "AIF" => "AIFF",
        "AIFC" => "AIFF",
        "AIT" => "AI",
        "AZW" => "MOBI",
        "AZW3" => "MOBI",
        "CAP" => "PCAP",
        "DC3" => "DICM",
        "DCM" => "DICM",
        "JPG" => "JPEG",
        "TIF" => "TIFF",
        _ => ext_upper.as_str(),
    };

    // Return format array and description based on ExifTool's fileTypeLookup
    match lookup_key {
        // Image formats
        "JPEG" => Some((vec!["JPEG"], "JPEG image")),
        "PNG" => Some((vec!["PNG"], "Portable Network Graphics")),
        "TIFF" => Some((vec!["TIFF"], "Tagged Image File Format")),
        "GIF" => Some((vec!["GIF"], "Graphics Interchange Format")),
        "BMP" => Some((vec!["BMP"], "Windows Bitmap")),
        "WEBP" => Some((vec!["RIFF"], "Google Web Picture")),
        "HEIC" => Some((vec!["MOV"], "High Efficiency Image Format still image")),
        "HEIF" => Some((vec!["MOV"], "High Efficiency Image Format")),
        "AVIF" => Some((vec!["MOV"], "AV1 Image File Format")),

        // Canon formats
        "CR2" => Some((vec!["TIFF"], "Canon RAW 2 format")),
        "CR3" => Some((vec!["MOV"], "Canon RAW 3 format")),
        "CRW" => Some((vec!["CRW"], "Canon RAW format")),
        "CRM" => Some((vec!["MOV"], "Canon RAW Movie")),

        // Other camera RAW formats
        "NEF" => Some((vec!["TIFF"], "Nikon Electronic Format")),
        "NRW" => Some((vec!["TIFF"], "Nikon RAW 2 format")),
        "ARW" => Some((vec!["TIFF"], "Sony Alpha RAW format")),
        "ARQ" => Some((vec!["TIFF"], "Sony Alpha Pixel-Shift RAW format")),
        "RAF" => Some((vec!["FUJIFILM"], "FujiFilm RAW format")),
        "ORF" => Some((vec!["TIFF"], "Olympus RAW format")),
        "RW2" => Some((vec!["TIFF"], "Panasonic RAW format")),
        "RWL" => Some((vec!["TIFF"], "Leica RAW format")),
        "DNG" => Some((vec!["TIFF"], "Digital Negative")),
        "3FR" => Some((vec!["TIFF"], "Hasselblad RAW format")),
        "IIQ" => Some((vec!["TIFF"], "Phase One RAW format")),
        "MRW" => Some((vec!["MRW"], "Minolta RAW format")),

        // Video formats
        "MP4" => Some((vec!["MOV"], "MPEG-4 video")),
        "MOV" => Some((vec!["MOV"], "Apple QuickTime movie")),
        "AVI" => Some((vec!["RIFF"], "Audio Video Interleaved")),
        "M2TS" => Some((vec!["M2TS"], "MPEG-2 Transport Stream")),
        "MTS" => Some((vec!["M2TS"], "MPEG-2 Transport Stream")), // Alias
        "3GP" => Some((vec!["MOV"], "3rd Gen. Partnership Project audio/video")),
        "3G2" => Some((vec!["MOV"], "3rd Gen. Partnership Project 2 audio/video")),
        "FLV" => Some((vec!["FLV"], "Flash Video")),
        "MKV" => Some((vec!["MKV"], "Matroska multimedia container")),
        "WEBM" => Some((vec!["MKV"], "Google Web Movie")),
        "WMV" => Some((vec!["ASF"], "Windows Media Video")),
        "ASF" => Some((vec!["ASF"], "Microsoft Advanced Systems Format")),

        // Audio formats
        "MP3" => Some((vec!["MP3"], "MPEG-1 Audio Layer 3")),
        "WAV" => Some((vec!["RIFF"], "Windows digital audio")),
        "FLAC" => Some((vec!["FLAC"], "Free Lossless Audio Codec")),
        "AAC" => Some((vec!["AAC"], "Advanced Audio Coding")),
        "M4A" => Some((vec!["MOV"], "MPEG-4 audio")),
        "AIFF" => Some((vec!["AIFF"], "Audio Interchange File Format")),
        "APE" => Some((vec!["APE"], "Monkey's Audio format")),
        "WMA" => Some((vec!["ASF"], "Windows Media Audio")),

        // Document formats
        "PDF" => Some((vec!["PDF"], "Adobe Portable Document Format")),
        "PSD" => Some((vec!["PSD"], "Adobe Photoshop")),
        "EPS" => Some((vec!["EPS"], "Encapsulated PostScript")),
        "PS" => Some((vec!["PS"], "PostScript")),
        "AI" => Some((vec!["PDF", "PS"], "Adobe Illustrator")),
        "INDD" => Some((vec!["IND"], "Adobe InDesign")),
        "XMP" => Some((vec!["XMP"], "Extensible Metadata Platform")),

        // Archive formats
        "ZIP" => Some((vec!["ZIP"], "ZIP archive")),
        "TAR" => Some((vec!["TAR"], "Tape ARchive")),
        "7Z" => Some((vec!["7Z"], "7z archive")),
        "GZIP" => Some((vec!["GZIP"], "GNU ZIP archive")),
        "BZ2" => Some((vec!["BZ2"], "BZIP2 archive")),

        // Other formats
        "XML" => Some((vec!["XML"], "Extensible Markup Language")),
        "JSON" => Some((vec!["JSON"], "JavaScript Object Notation")),
        "TXT" => Some((vec!["TXT"], "Plain text file")),
        "CSV" => Some((vec!["TXT"], "Comma-Separated Values")),
        "ICO" => Some((vec!["ICO"], "Windows icon")),
        "CUR" => Some((vec!["ICO"], "Windows Cursor")),
        "MOBI" => Some((vec!["MOBI"], "Mobipocket electronic book")),
        "EPUB" => Some((vec!["ZIP"], "Electronic Publication")),

        // JPEG 2000 formats
        "JP2" => Some((vec!["JP2"], "JPEG 2000 image")),
        "J2C" => Some((vec!["J2C"], "JPEG 2000 codestream")),
        "JPC" => Some((vec!["J2C"], "JPEG 2000 codestream")),
        "JPX" => Some((vec!["JP2"], "JPEG 2000 with extensions")),

        // Real Media formats
        "RM" => Some((vec!["Real"], "Real Media")),
        "RA" => Some((vec!["Real"], "Real Audio")),
        "RV" => Some((vec!["Real"], "Real Video")),
        "RMVB" => Some((vec!["Real"], "Real Media Variable Bitrate")),

        _ => None,
    }
}

/// Get magic number pattern for a file type
/// This function is no longer generated in the new architecture.
/// Magic number patterns are now handled differently.
pub fn get_magic_number_pattern(_file_type: &str) -> Option<&'static str> {
    // In the new architecture, magic number patterns are not exposed
    // as simple strings. The file detection logic handles them internally.
    // Return None to trigger fallback behavior in the detection code.
    None
}

/// Get primary format for a file type
pub fn get_primary_format(file_type: &str) -> Option<String> {
    resolve_file_type(file_type).map(|(formats, _)| formats[0].to_string())
}

/// Check if a file type supports a specific format
pub fn supports_format(file_type: &str, format: &str) -> bool {
    resolve_file_type(file_type)
        .map(|(formats, _)| formats.contains(&format))
        .unwrap_or(false)
}

/// Get all extensions that support a given format
pub fn extensions_for_format(format: &str) -> Vec<String> {
    let mut extensions = Vec::new();

    // Check all known extensions to see which ones support this format
    let all_extensions = [
        "JPEG", "JPG", "PNG", "TIFF", "TIF", "GIF", "BMP", "WEBP", "HEIC", "HEIF", "AVIF", "CR2",
        "CR3", "CRW", "CRM", "NEF", "NRW", "ARW", "ARQ", "RAF", "ORF", "RW2", "RWL", "DNG", "3FR",
        "IIQ", "MRW", "MP4", "MOV", "AVI", "M2TS", "MTS", "3GP", "3G2", "FLV", "MKV", "WEBM",
        "WMV", "ASF", "MP3", "WAV", "FLAC", "AAC", "M4A", "AIFF", "AIF", "APE", "WMA", "PDF",
        "PSD", "EPS", "PS", "AI", "INDD", "XMP", "ZIP", "TAR", "7Z", "GZIP", "BZ2", "XML", "JSON",
        "TXT", "CSV", "ICO", "CUR", "MOBI", "EPUB", "JP2", "J2C", "JPC", "JPX", "RM", "RA", "RV",
        "RMVB",
    ];

    for ext in all_extensions {
        if supports_format(ext, format) {
            extensions.push(ext.to_string());
        }
    }

    extensions
}

/// Re-export the file_types compatibility module at the expected location
pub mod file_types {
    pub use super::extensions_for_format;
    pub use super::get_magic_number_pattern;
    pub use super::get_primary_format;
    pub use super::lookup_mime_types;
    pub use super::resolve_file_type;
    pub use super::supports_format;
}
