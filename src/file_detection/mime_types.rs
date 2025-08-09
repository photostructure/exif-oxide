//! MIME type resolution and fallback handling
//!
//! Provides MIME type lookup with fallback strategies.

use super::{FileDetectionError, FileTypeDetectionResult};
use crate::generated::exif_tool::mime_type::lookup_mime_types;
use std::path::Path;

/// Build final detection result from file type
pub fn build_result(
    file_type: &str,
    path: &Path,
) -> Result<FileTypeDetectionResult, FileDetectionError> {
    // Get primary format for processing
    use crate::generated::exif_tool::file_type_lookup::resolve_file_type;
    let (format, description) = if let Some((formats, desc)) = resolve_file_type(file_type) {
        (formats[0].to_string(), desc.to_string())
    } else {
        (file_type.to_string(), format!("{file_type} file"))
    };

    // Get MIME type from generated lookup - try the file type first, then fallback, then the format
    // This ensures file-type-specific MIME types take precedence over generic format MIME types
    let mime_type = lookup_mime_types(file_type)
        .or_else(|| get_fallback_mime_type(file_type))
        .or_else(|| lookup_mime_types(&format))
        .unwrap_or("application/octet-stream")
        .to_string();

    // Special case: ASF files with .wmv extension should use video/x-ms-wmv MIME type
    // ExifTool.pm:9570-9592 SetFileType() applies extension-specific MIME types for ASF/WMV
    // Reference: ExifTool.pm lines 557 (WMV->ASF mapping) and 816 (WMV MIME type)
    let mime_type = if file_type == "ASF" {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "wmv" => "video/x-ms-wmv".to_string(),
                _ => mime_type,
            }
        } else {
            mime_type
        }
    } else {
        mime_type
    };

    Ok(FileTypeDetectionResult {
        file_type: file_type.to_string(),
        format: format.to_string(),
        mime_type,
        description,
    })
}

/// Get fallback MIME types for file types not covered by ExifTool's %mimeType hash
/// These are standard MIME types for common formats
pub fn get_fallback_mime_type(file_type: &str) -> Option<&'static str> {
    match file_type {
        // Image formats
        "JPEG" => Some("image/jpeg"),
        "PNG" => Some("image/png"),
        "TIFF" => Some("image/tiff"),
        "GIF" => Some("image/gif"),
        "BMP" => Some("image/bmp"),
        "WEBP" => Some("image/webp"),
        "AVIF" => Some("image/avif"), // AV1 Image File Format - from ExifTool QuickTime.pm %mimeLookup
        "HEIC" => Some("image/heic"), // HEIC gets its own MIME type
        "HEIF" => Some("image/heif"), // High Efficiency Image Format (general)
        "JP2" => Some("image/jp2"),   // JPEG 2000 Part 1 (ISO/IEC 15444-1)
        "J2C" => Some("image/x-j2c"), // JPEG 2000 Code Stream

        // Video formats
        "AVI" => Some("video/x-msvideo"),
        "3GP" => Some("video/3gpp"),     // 3GPP video format
        "3G2" => Some("video/3gpp2"),    // 3GPP2 video format
        "M4V" => Some("video/x-m4v"),    // Apple M4V video
        "MTS" => Some("video/m2ts"),     // MPEG-2 Transport Stream (alias for M2TS)
        "M2TS" => Some("video/m2ts"),    // MPEG-2 Transport Stream
        "MP4" => Some("video/mp4"),      // MPEG-4 Part 14
        "FLV" => Some("video/x-flv"),    // Flash Video
        "WMV" => Some("video/x-ms-wmv"), // Windows Media Video
        "ASF" => Some("video/x-ms-wmv"), // Advanced Systems Format (usually WMV)

        // Audio formats
        "WAV" => Some("audio/x-wav"), // WAV audio files

        // Document formats
        "XMP" => Some("application/rdf+xml"), // Extensible Metadata Platform
        "PSD" => Some("application/vnd.adobe.photoshop"), // Adobe Photoshop Document
        "EPS" => Some("application/postscript"), // Encapsulated PostScript

        // Other common formats that might be missing
        "RIFF" => Some("application/octet-stream"), // Generic RIFF container

        _ => None,
    }
}
