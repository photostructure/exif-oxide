//! File type detection engine following ExifTool's implementation
//!
//! This module implements ExifTool's sophisticated multi-tiered file type detection
//! approach, ported from ExifTool.pm:2913-2999
//!
//! Detection Flow:
//! 1. Extension-based candidates (via generated fileTypeLookup)
//! 2. Magic number validation (via generated magicNumber patterns)
//! 3. Last-ditch embedded signature recovery
//!
//! The implementation preserves ExifTool's exact logic including:
//! - Weak magic types that defer to extension
//! - Extension normalization rules
//! - Conflict resolution patterns
//! - Error recovery mechanisms

pub mod extensions;
pub mod magic_numbers;
pub mod mime_types;
pub mod mov_video;
pub mod riff;
pub mod tiff_raw;

#[cfg(test)]
mod mimetypes_validation;

pub use extensions::{get_candidates_from_extension, has_processing_module, normalize_extension};
pub use magic_numbers::{matches_magic_number, scan_for_embedded_signatures, validate_xmp_pattern};
pub use mime_types::{build_result, get_fallback_mime_type};
pub use mov_video::determine_mov_subtype;
pub use riff::{detect_riff_type, is_riff_based_format, validate_riff_format};
pub use tiff_raw::{is_tiff_based_raw_format, validate_tiff_raw_format};

use std::io::{Read, Seek};
use std::path::Path;

/// Maximum bytes to read for magic number testing
/// ExifTool uses exactly 1024 bytes - ExifTool.pm:2955
const MAGIC_TEST_BUFFER_SIZE: usize = 1024;

#[derive(Debug, Clone, PartialEq)]
pub struct FileTypeDetectionResult {
    /// Detected file type (e.g., "JPEG", "PNG", "CR2")
    pub file_type: String,
    /// Primary format for processing (e.g., "JPEG", "TIFF", "MOV")
    pub format: String,
    /// MIME type string
    pub mime_type: String,
    /// Human-readable description
    pub description: String,
}

#[derive(Debug)]
pub enum FileDetectionError {
    /// File type could not be determined
    UnknownFileType,
    /// IO error reading file
    IoError(std::io::Error),
    /// Invalid file path
    InvalidPath,
}

impl From<std::io::Error> for FileDetectionError {
    fn from(error: std::io::Error) -> Self {
        FileDetectionError::IoError(error)
    }
}

impl std::fmt::Display for FileDetectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileDetectionError::UnknownFileType => write!(f, "Unknown file type"),
            FileDetectionError::IoError(e) => write!(f, "IO error: {e}"),
            FileDetectionError::InvalidPath => write!(f, "Invalid file path"),
        }
    }
}

impl std::error::Error for FileDetectionError {}

/// Main file type detector implementing ExifTool's detection algorithm
pub struct FileTypeDetector;

impl FileTypeDetector {
    /// Create a new file type detector
    pub fn new() -> Self {
        Self
    }

    /// Detect file type from path and file content
    ///
    /// Implements ExifTool's detection flow from ExifTool.pm:2913-2999
    pub fn detect_file_type<R: Read + Seek>(
        &self,
        path: &Path,
        reader: &mut R,
    ) -> Result<FileTypeDetectionResult, FileDetectionError> {
        // Phase 1: Get extension-based candidates
        // ExifTool.pm:2940 - GetFileType($filename)
        let candidates = get_candidates_from_extension(path)?;

        // Phase 2: Read test buffer for magic number validation
        // ExifTool.pm:2955 - Read($raf, $buff, $testLen)
        let mut buffer = vec![0u8; MAGIC_TEST_BUFFER_SIZE];
        let bytes_read = reader.read(&mut buffer)?;
        buffer.truncate(bytes_read);

        // Reset reader position for subsequent processing
        // This is critical so format-specific processors start at the beginning
        reader.seek(std::io::SeekFrom::Start(0))?;

        // Phase 3: Magic number validation against candidates
        // ExifTool.pm:2960-2975 - Test candidates against magic numbers
        // CRITICAL: Test all candidates before giving up, per TRUST-EXIFTOOL.md
        let mut matched_type = None;
        let mut recognized_ext = None;

        for candidate in &candidates {
            // Check if this is a weak magic type that defers to extension
            // ExifTool.pm:1030 - %weakMagic hash contains types with weak magic
            // Note: Most weak magic types (MP3, etc.) rely on extension detection
            let is_weak_magic =
                matches!(candidate.as_str(), "MP3" | "MP4" | "AAC" | "OGG" | "FLAC");
            if is_weak_magic {
                // Weak magic types are fallback only if no strong magic matches
                // ExifTool.pm:2970 - "next if $weakMagic{$type} and defined $recognizedExt"
                if matched_type.is_none() {
                    matched_type = Some(candidate.clone());
                }
                continue;
            }

            // validate_magic_number now checks both file type and format patterns
            if self.validate_magic_number(candidate, &buffer) {
                // Strong magic match - use this type
                matched_type = Some(candidate.clone());
                break;
            }

            // ExifTool behavior: Keep track of recognized extensions with modules
            // Even if magic pattern fails, ExifTool may still process the file
            // if it has a module defined (like JXL -> Jpeg2000)
            if recognized_ext.is_none() && has_processing_module(candidate) {
                recognized_ext = Some(candidate.clone());
            }
        }

        // If no magic match but we have a recognized extension with a module,
        // use that as fallback (mimics ExifTool's behavior for JXL and others)
        if matched_type.is_none() && recognized_ext.is_some() {
            matched_type = recognized_ext;
        }

        if let Some(file_type) = matched_type {
            // Special handling for MOV format to determine specific subtype
            // ExifTool QuickTime.pm:9868-9877 - ftyp brand determines actual file type
            // CRITICAL: Check against the format, not the file type
            use crate::generated::exif_tool::file_type_lookup::resolve_file_type;
            let format = if let Some((formats, _)) = resolve_file_type(&file_type) {
                formats[0]
            } else {
                &file_type
            };

            let detected_type = if format == "MOV" {
                determine_mov_subtype(&buffer).unwrap_or_else(|| file_type.clone())
            } else if is_riff_based_format(&file_type) {
                // For RIFF-based formats, detect the actual type from the header
                // ExifTool RIFF.pm:2038-2039 - Sets file type based on RIFF format identifier
                detect_riff_type(&buffer).unwrap_or_else(|| file_type.clone())
            } else {
                file_type
            };
            return build_result(&detected_type, path);
        }

        // Phase 4: Last-ditch recovery - scan for embedded signatures
        // ExifTool.pm:2976-2983 - Look for JPEG/TIFF embedded in unknown data
        if let Some(embedded_type) = scan_for_embedded_signatures(&buffer) {
            return build_result(&embedded_type, path);
        }

        Err(FileDetectionError::UnknownFileType)
    }

    /// Validate magic number for a file type candidate
    /// ExifTool equivalent: magic number testing in ExifTool.pm:2960-2975
    /// CRITICAL: Must match ExifTool's exact logic per TRUST-EXIFTOOL.md
    fn validate_magic_number(&self, file_type: &str, buffer: &[u8]) -> bool {
        // Special handling for RIFF-based formats (AVI, WAV, WEBP, etc.)
        // ExifTool RIFF.pm:2037-2046 - RIFF container detection with format analysis
        if is_riff_based_format(file_type) {
            return validate_riff_format(file_type, buffer);
        }

        // Special handling for TIFF-based RAW formats that need deeper analysis
        // ExifTool.pm:8531-8612 - DoProcessTIFF() RAW format detection
        if is_tiff_based_raw_format(file_type) {
            return validate_tiff_raw_format(file_type, buffer);
        }

        // Use the new two-HashMap magic number system
        // First try literal patterns (fast), then fall back to regex patterns (slower)
        // This matches the hybrid approach in the generated magic numbers file
        if matches_magic_number(file_type, buffer) {
            return true;
        }

        // If no direct match, check if this file type has a format that has magic patterns
        // ExifTool uses the format (MOV, TIFF, etc.) for magic pattern matching
        use crate::generated::exif_tool::file_type_lookup::resolve_file_type;
        if let Some((formats, _desc)) = resolve_file_type(file_type) {
            // Try magic pattern for the primary format
            if matches_magic_number(formats[0], buffer) {
                return true;
            }
            // ExifTool behavior: If type has magic pattern defined but doesn't match, reject it
            // ExifTool.pm:2960 - "next if $buff !~ /^$magicNumber{$type}/s and not $noMagic{$type}"
            return false;
        }

        // ExifTool behavior: Types without magic numbers can be recognized by extension alone
        // ExifTool.pm:2966 - "next if defined $moduleName{$type} and not $moduleName{$type}"
        // If we reach here, this file type has no magic pattern defined, so allow extension-based detection
        true
    }
}

impl Default for FileTypeDetector {
    fn default() -> Self {
        Self::new()
    }
}
