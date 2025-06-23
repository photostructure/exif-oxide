//! File type detection system based on ExifTool's algorithm

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool.pm"]

use crate::error::Result;

// Include the auto-generated magic numbers
mod magic_numbers;
pub use magic_numbers::{FileType, MagicPattern, EXTENSION_LOOKUP, MAGIC_NUMBERS, MIME_TYPES};

// TIFF-based RAW format detection
mod tiff_raw;

/// Test length for initial file read (ExifTool default)
const _DEFAULT_TEST_LEN: usize = 1024;

/// File detection result with metadata
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// Detected file type
    pub file_type: FileType,
    /// MIME type (includes File:MIMEType field)
    pub mime_type: String,
    /// Whether detection required weak heuristics
    pub weak_detection: bool,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f32,
}

impl FileInfo {
    /// Get the File:MIMEType field value for ExifTool compatibility
    pub fn file_mime_type(&self) -> String {
        self.mime_type.clone()
    }
}

/// EXIFTOOL-IMPL: Main file type detection logic (ExifTool.pm::ImageInfo)
/// This function implements ExifTool's file type detection algorithm,
/// including precedence rules and weak detection handling
pub fn detect_file_type(data: &[u8]) -> Result<FileInfo> {
    // Ensure we have enough data
    if data.is_empty() {
        return Ok(FileInfo {
            file_type: FileType::Unknown,
            mime_type: "application/octet-stream".to_string(),
            weak_detection: false,
            confidence: 0.0,
        });
    }

    // Try magic number detection first
    if let Some(mut info) = detect_by_magic(data) {
        // Special handling for TIFF-based formats
        if info.file_type == FileType::TIFF {
            // Check if this is actually a RAW format using Make field detection
            if let Some(raw_type) = tiff_raw::detect_raw_by_make(data, 0) {
                info.file_type = raw_type;
                info.mime_type = MIME_TYPES
                    .get(&raw_type)
                    .unwrap_or(&"application/octet-stream")
                    .to_string();
            }
        }
        return Ok(info);
    }

    // Fall back to content analysis for weak magic types
    if let Some(info) = detect_weak_types(data) {
        return Ok(info);
    }

    // Unknown file type
    Ok(FileInfo {
        file_type: FileType::Unknown,
        mime_type: "application/octet-stream".to_string(),
        weak_detection: false,
        confidence: 0.0,
    })
}

/// EXIFTOOL-IMPL: Magic number pattern matching
fn detect_by_magic(data: &[u8]) -> Option<FileInfo> {
    // Special handling for RIFF-based formats
    if data.len() >= 12 && &data[0..4] == b"RIFF" {
        let format_id = &data[8..12];
        let (file_type, weak) = match format_id {
            b"WEBP" => (FileType::WEBP, false),
            b"AVI " => (FileType::AVI, false),
            _ => return None, // Unknown RIFF format
        };

        let mime_type = MIME_TYPES
            .get(&file_type)
            .unwrap_or(&"application/octet-stream")
            .to_string();

        return Some(FileInfo {
            file_type,
            mime_type,
            weak_detection: weak,
            confidence: 1.0,
        });
    }

    // Special handling for QuickTime-based formats (MP4, MOV, HEIF, HEIC, AVIF, CR3, video formats)
    if data.len() >= 12 && &data[4..8] == b"ftyp" {
        let brand = &data[8..12];
        let file_type = match brand {
            // Standard MP4/MOV formats
            b"isom" | b"mp41" | b"mp42" => FileType::MP4,
            b"qt  " => FileType::MOV,
            // Image formats
            b"heic" => FileType::HEIC,
            b"heif" => FileType::HEIF,
            b"avif" => FileType::AVIF,
            // Canon formats (CR3 still images vs CRM video)
            b"crx " => detect_canon_crx_format(data).unwrap_or(FileType::CR3),
            // Video formats
            b"3gp4" | b"3gp5" | b"3gp6" | b"3gp7" | b"3ge6" | b"3ge7" | b"3gg6" => {
                FileType::ThreeGPP
            }
            b"3g2a" | b"3g2b" | b"3g2c" => FileType::ThreeGPP2,
            b"M4V " | b"M4VH" | b"M4VP" => FileType::M4V,
            // HEIF/HEIC sequences (video)
            b"hevc" => FileType::HEICS, // HEIC sequence
            b"msf1" => FileType::HEIFS, // HEIF sequence
            _ => return None,           // Unknown QuickTime brand
        };

        let mime_type = MIME_TYPES
            .get(&file_type)
            .unwrap_or(&"application/octet-stream")
            .to_string();

        return Some(FileInfo {
            file_type,
            mime_type,
            weak_detection: false,
            confidence: 1.0,
        });
    }

    // Special handling for older QuickTime MOV format (moov/mdat boxes without ftyp)
    if data.len() >= 8 {
        let fourcc = &data[4..8];
        if fourcc == b"moov" || fourcc == b"mdat" {
            let mime_type = MIME_TYPES
                .get(&FileType::MOV)
                .unwrap_or(&"application/octet-stream")
                .to_string();

            return Some(FileInfo {
                file_type: FileType::MOV,
                mime_type,
                weak_detection: false,
                confidence: 1.0,
            });
        }
    }

    // Check each file type's magic patterns
    for (file_type, patterns) in MAGIC_NUMBERS.iter() {
        // Skip RIFF-based and QuickTime-based formats as they're handled above
        if matches!(
            file_type,
            FileType::WEBP
                | FileType::AVI
                | FileType::MP4
                | FileType::MOV
                | FileType::HEIF
                | FileType::HEIC
                | FileType::AVIF
                | FileType::CR3
                | FileType::CRM
                | FileType::ThreeGPP
                | FileType::ThreeGPP2
                | FileType::M4V
                | FileType::HEIFS
                | FileType::HEICS
        ) {
            continue;
        }

        for pattern in patterns {
            if matches_pattern(data, pattern) {
                let mime_type = MIME_TYPES
                    .get(file_type)
                    .unwrap_or(&"application/octet-stream")
                    .to_string();

                return Some(FileInfo {
                    file_type: *file_type,
                    mime_type,
                    weak_detection: pattern.weak,
                    confidence: if pattern.weak { 0.7 } else { 1.0 },
                });
            }
        }
    }
    None
}

/// Check if data matches a magic pattern
fn matches_pattern(data: &[u8], pattern: &MagicPattern) -> bool {
    // Check if we have enough data
    let required_len = pattern.offset + pattern.pattern.len();
    if data.len() < required_len {
        return false;
    }

    // Check basic pattern match
    let slice = &data[pattern.offset..pattern.offset + pattern.pattern.len()];
    if slice != pattern.pattern {
        return false;
    }

    // Handle special cases that need additional validation
    if pattern.weak {
        // RIFF-based formats need to check the format identifier at offset 8
        if pattern.pattern == b"RIFF" && data.len() >= 12 {
            let _format_id = &data[8..12];
            // This function is called for each pattern, so we need to check
            // which specific RIFF format we're looking for based on the calling pattern
            // For now, just return true and let the caller distinguish
            return true;
        }

        // CRW: Check for proper HEAP structure
        if pattern.pattern == b"HEAP" && data.len() >= 14 {
            // Should have II/MM at start and HEAP at offset 6
            return (data[0..2] == [0x49, 0x49] || data[0..2] == [0x4d, 0x4d])
                && data[6..10] == [0x48, 0x45, 0x41, 0x50];
        }
    }

    true
}

/// EXIFTOOL-IMPL: Weak magic detection requiring additional validation
fn detect_weak_types(_data: &[u8]) -> Option<FileInfo> {
    // MP3 detection would go here if FileType::MP3 was defined
    // For now, just return None for weak types

    // Additional weak type detection...
    None
}

/// EXIFTOOL-QUIRK: Special detection for RAW formats using TIFF structure
/// Many RAW formats (CR2, NEF, ARW) use TIFF container but need additional checks
pub fn detect_raw_variant(data: &[u8], tiff_header_offset: usize) -> Option<FileType> {
    if data.len() < tiff_header_offset + 8 {
        return None;
    }

    // Check byte order
    let byte_order = &data[tiff_header_offset..tiff_header_offset + 2];
    let is_little_endian = byte_order == b"II";
    let is_big_endian = byte_order == b"MM";

    if !is_little_endian && !is_big_endian {
        return None;
    }

    // EXIFTOOL-QUIRK: Canon CR2 - look for "CR" at offset 8
    if data.len() >= tiff_header_offset + 10 {
        let cr_marker = &data[tiff_header_offset + 8..tiff_header_offset + 10];
        if cr_marker == b"CR" {
            return Some(FileType::CR2);
        }
    }

    // EXIFTOOL-QUIRK: Nikon detection - need to parse IFD for maker info
    // For a simple heuristic without full IFD parsing:
    // Many Nikon files have specific IFD patterns
    if data.len() >= 16 && is_little_endian {
        // Check for common Nikon IFD patterns
        let ifd_offset = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
        if ifd_offset == 8 || ifd_offset == 10 {
            // This is a common pattern in Nikon files
            // TODO: Full implementation would parse Make tag (0x010F)
            return Some(FileType::NRW);
        }
    }

    // EXIFTOOL-QUIRK: Sony detection
    // For Sony formats, we would need to parse the IFD to find Make tag
    // For now, use simple heuristics based on file structure patterns
    if data.len() >= 16 {
        // Sony SR2 files often have specific patterns
        // TODO: Implement proper IFD parsing to check Make field

        // EXIFTOOL-QUIRK: Olympus detection
        // Olympus ORF files use TIFF container
        // TODO: Check for Olympus Make in IFD

        // EXIFTOOL-QUIRK: Pentax detection
        // Pentax PEF files use TIFF container
        // TODO: Check for Pentax Make in IFD
    }

    // For now, return None for unidentified TIFF-based formats
    // A full implementation would parse the IFD to check the Make field (tag 0x010F)
    None
}

/// EXIFTOOL-LOGIC: Detect nested formats (e.g., JPEG in HEIF)
pub fn detect_nested_formats(data: &[u8], parent: FileType) -> Vec<FileType> {
    let mut nested = Vec::new();

    // Example: HEIF can contain JPEG thumbnails
    if parent == FileType::HEIF {
        // Scan for JPEG markers
        for i in 0..data.len().saturating_sub(3) {
            if &data[i..i + 3] == b"\xff\xd8\xff" {
                nested.push(FileType::JPEG);
                break;
            }
        }
    }

    nested
}

/// EXIFTOOL-QUIRK: Distinguish Canon CR3 (still) vs CRM (video)
/// Both use "crx " brand, but CRM files contain video tracks
fn detect_canon_crx_format(data: &[u8]) -> Option<FileType> {
    // Simple heuristic: look for common video-related atoms in the file
    // CRM files typically have video track atoms like "trak", "mdia", "vide"

    // Search for video-related QuickTime atoms that indicate this is a movie
    let search_len = 1024.min(data.len());

    // Look for common video atoms: "trak", "mdia", "vide"
    for i in 0..search_len.saturating_sub(4) {
        let fourcc = &data[i..i + 4];
        if fourcc == b"trak" || fourcc == b"mdia" || fourcc == b"vide" {
            return Some(FileType::CRM);
        }
    }

    // Default to CR3 (still image)
    Some(FileType::CR3)
}

/// Detect file type by extension when magic number detection fails
pub fn detect_by_extension(extension: &str) -> Option<FileInfo> {
    let ext_upper = extension.to_uppercase();

    if let Some((file_type, _description)) = EXTENSION_LOOKUP.get(ext_upper.as_str()) {
        let mime_type = MIME_TYPES
            .get(file_type)
            .unwrap_or(&"application/octet-stream")
            .to_string();

        Some(FileInfo {
            file_type: *file_type,
            mime_type,
            weak_detection: true,
            confidence: 0.5, // Lower confidence for extension-only detection
        })
    } else {
        None
    }
}

/// EXIFTOOL-IMPL: SetFileType equivalent - returns file info with proper MIME type
pub fn get_file_info(file_type: FileType, base_type: Option<FileType>) -> FileInfo {
    // Try specific file type first
    let mime_type = if let Some(mime) = MIME_TYPES.get(&file_type) {
        mime.to_string()
    } else if let Some(base) = base_type {
        // Fall back to base type MIME
        MIME_TYPES
            .get(&base)
            .unwrap_or(&"application/octet-stream")
            .to_string()
    } else {
        "application/octet-stream".to_string()
    };

    FileInfo {
        file_type,
        mime_type,
        weak_detection: false,
        confidence: 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jpeg_detection() {
        let jpeg_data = b"\xff\xd8\xff\xe0\x00\x10JFIF";
        let info = detect_file_type(jpeg_data).unwrap();

        assert_eq!(info.file_type, FileType::JPEG);
        assert_eq!(info.mime_type, "image/jpeg");
        assert!(!info.weak_detection);
        assert_eq!(info.confidence, 1.0);
    }

    #[test]
    fn test_png_detection() {
        let png_data = b"\x89PNG\r\n\x1a\n";
        let info = detect_file_type(png_data).unwrap();

        assert_eq!(info.file_type, FileType::PNG);
        assert_eq!(info.mime_type, "image/png");
    }

    #[test]
    fn test_unknown_detection() {
        let unknown_data = b"Some random data";
        let info = detect_file_type(unknown_data).unwrap();

        assert_eq!(info.file_type, FileType::Unknown);
        assert_eq!(info.mime_type, "application/octet-stream");
        assert_eq!(info.confidence, 0.0);
    }

    #[test]
    fn test_extension_detection() {
        let info = detect_by_extension("jpg").unwrap();
        assert_eq!(info.file_type, FileType::JPEG);
        assert_eq!(info.mime_type, "image/jpeg");
        assert!(info.weak_detection);
        assert_eq!(info.confidence, 0.5);
    }

    #[test]
    fn test_avif_detection() {
        // AVIF file: size (4 bytes) + "ftyp" + "avif" + ...
        let avif_data = b"\x00\x00\x00\x20ftypavifsomething_else_here";
        let info = detect_file_type(avif_data).unwrap();

        assert_eq!(info.file_type, FileType::AVIF);
        assert_eq!(info.mime_type, "image/avif");
        assert!(!info.weak_detection);
        assert_eq!(info.confidence, 1.0);
    }
}
