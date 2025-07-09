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

use crate::generated::file_types::lookup_mime_types;
use std::io::{Read, Seek};
use std::path::Path;

/// Maximum bytes to read for magic number testing
/// ExifTool uses exactly 1024 bytes - ExifTool.pm:2955
const MAGIC_TEST_BUFFER_SIZE: usize = 1024;

/// File types with weak magic numbers that defer to extension detection
/// ExifTool.pm:1030 - only MP3 is marked as weak magic: my %weakMagic = ( MP3 => 1 );
const WEAK_MAGIC_TYPES: &[&str] = &["MP3"];

// All magic number patterns are now generated from ExifTool.pm %magicNumber hash
// See src/generated/file_types/magic_numbers.rs for the complete patterns
// No hardcoded patterns needed - use lookup_magic_number_patterns() for all detection

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
        let candidates = self.get_candidates_from_extension(path)?;

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
        for candidate in &candidates {
            // Check if this is a weak magic type that defers to extension
            if WEAK_MAGIC_TYPES.contains(&candidate.as_str()) {
                // Skip magic testing for weak magic types - trust extension
                return self.build_result(candidate, path);
            }

            if self.validate_magic_number(candidate, &buffer) {
                // Special handling for MOV format to determine specific subtype
                // ExifTool QuickTime.pm:9868-9877 - ftyp brand determines actual file type
                let detected_type = if candidate == "MOV" {
                    self.determine_mov_subtype(&buffer)
                        .unwrap_or_else(|| candidate.clone())
                } else if self.is_riff_based_format(candidate) {
                    // For RIFF-based formats, detect the actual type from the header
                    // ExifTool RIFF.pm:2038-2039 - Sets file type based on RIFF format identifier
                    self.detect_riff_type(&buffer)
                        .unwrap_or_else(|| candidate.clone())
                } else {
                    candidate.clone()
                };
                return self.build_result(&detected_type, path);
            }
        }

        // Phase 4: Last-ditch recovery - scan for embedded signatures
        // ExifTool.pm:2976-2983 - Look for JPEG/TIFF embedded in unknown data
        if let Some(embedded_type) = self.scan_for_embedded_signatures(&buffer) {
            return self.build_result(&embedded_type, path);
        }

        Err(FileDetectionError::UnknownFileType)
    }

    /// Get file type candidates based on file extension
    /// ExifTool equivalent: GetFileType() in ExifTool.pm:9010-9050
    fn get_candidates_from_extension(
        &self,
        path: &Path,
    ) -> Result<Vec<String>, FileDetectionError> {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or(FileDetectionError::InvalidPath)?;

        // Normalize extension to uppercase (ExifTool convention)
        let normalized_ext = self.normalize_extension(extension);

        // Resolve through fileTypeLookup with alias following
        // ExifTool.pm:258-404 %fileTypeLookup hash defines extension mappings
        use crate::generated::file_types::resolve_file_type;
        if let Some((_formats, _description)) = resolve_file_type(&normalized_ext) {
            // For most formats, the extension itself is the file type candidate
            // The formats array tells us what processing module to use, not the file type
            // ExifTool.pm:2940-2950 - GetFileType returns the extension-based type
            
            // Special case: Some extensions are aliases that should map to a different type
            // These are hardcoded in ExifTool.pm GetFileType()
            match normalized_ext.as_str() {
                "3GP2" => Ok(vec!["3G2".to_string()]),  // ExifTool.pm alias
                "MTS" => Ok(vec!["M2TS".to_string()]),  // ExifTool.pm alias
                _ => Ok(vec![normalized_ext.clone()]),   // Use the extension as the type
            }
        } else {
            // Unknown extension - return normalized extension as candidate
            Ok(vec![normalized_ext])
        }
    }

    /// Normalize file extension following ExifTool's rules
    /// ExifTool equivalent: GetFileExtension() in ExifTool.pm:9013-9040
    fn normalize_extension(&self, extension: &str) -> String {
        let upper_ext = extension.to_uppercase();

        // ExifTool hardcoded extension conversions
        // These are critical for consistency - TRUST-EXIFTOOL
        match upper_ext.as_str() {
            "TIF" => "TIFF".to_string(), // ExifTool.pm:9019 - hardcoded for TIFF consistency
            "JPG" => "JPEG".to_string(),
            "3GP2" => "3G2".to_string(),
            "AIF" => "AIFF".to_string(),
            _ => upper_ext,
        }
    }

    /// Convert Perl regex pattern to Rust regex pattern
    /// ExifTool patterns use Perl syntax that needs conversion for Rust regex crate
    #[allow(dead_code)]
    fn convert_perl_pattern_to_rust(&self, pattern: &str) -> String {
        // Convert common Perl regex patterns to Rust-compatible patterns
        // These conversions preserve ExifTool's exact matching behavior

        let mut rust_pattern = pattern.to_string();

        // Handle null bytes and their quantifiers
        // \0 -> \x00, \0{3} -> \x00{3}, \0{0,3} -> \x00{0,3}
        rust_pattern = rust_pattern.replace("\\0", "\\x00");

        // Handle common escape sequences
        rust_pattern = rust_pattern.replace("\\r", "\\x0d");
        rust_pattern = rust_pattern.replace("\\n", "\\x0a");
        rust_pattern = rust_pattern.replace("\\t", "\\x09");

        // Handle Unicode characters by converting to byte sequences
        // For BPG pattern "BPGû" - convert û (U+00FB) to \xfb
        if rust_pattern.contains('û') {
            rust_pattern = rust_pattern.replace('û', "\\xfb");
        }

        // Handle other common Unicode/extended ASCII characters
        rust_pattern = rust_pattern.replace('é', "\\xe9");
        rust_pattern = rust_pattern.replace('ñ', "\\xf1");

        // Fix character classes with hex values - ensure proper escaping
        // These are already mostly correct for Rust regex

        // Handle dot patterns in specific contexts
        // For JXL pattern, dots should match any byte in binary context
        // This is already correct as . matches any byte in bytes regex

        rust_pattern
    }

    /// Match binary magic patterns using specialized logic for common cases
    /// This handles patterns that mix hex bytes with ASCII text more reliably than regex
    #[allow(dead_code)]
    fn match_binary_magic_pattern(&self, file_type: &str, pattern: &str, buffer: &[u8]) -> bool {
        // Handle specific patterns that are known to be problematic with regex
        match file_type {
            "PNG" => {
                // PNG pattern: (\x89P|\x8aM|\x8bJ)NG\r\n\x1a\n
                // Standard PNG: \x89PNG\r\n\x1a\n
                buffer.len() >= 8
                    && (buffer.starts_with(&[0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a])
                        || buffer.starts_with(&[0x8a, 0x4d, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a])
                        || buffer.starts_with(&[0x8b, 0x4a, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]))
            }
            "BPG" => {
                // BPG pattern: BPGû (where û is 0xfb)
                buffer.len() >= 4 && buffer.starts_with(&[0x42, 0x50, 0x47, 0xfb])
            }
            "AAC" => {
                // AAC pattern: \xff[\xf0\xf1]
                buffer.len() >= 2 && buffer[0] == 0xff && (buffer[1] == 0xf0 || buffer[1] == 0xf1)
            }
            "JXL" => {
                // JXL pattern: (\xff\x0a|\0\0\0\x0cJXL \x0d\x0a......ftypjxl )
                if buffer.len() >= 2 && buffer.starts_with(&[0xff, 0x0a]) {
                    true // First alternative matches
                } else if buffer.len() >= 23 {
                    // Second alternative: \0\0\0\x0cJXL \x0d\x0a......ftypjxl
                    buffer.starts_with(&[0x00, 0x00, 0x00, 0x0c]) &&
                    buffer[4..8] == [0x4a, 0x58, 0x4c, 0x20] && // "JXL "
                    buffer[8..10] == [0x0d, 0x0a] &&
                    buffer.len() >= 21 &&
                    buffer[16..21] == [0x66, 0x74, 0x79, 0x70, 0x6a] && // "ftypj"
                    buffer[21..23] == [0x78, 0x6c] // "xl"
                } else {
                    false
                }
            }
            "MKV" => {
                // MKV pattern: \x1a\x45\xdf\xa3
                buffer.len() >= 4 && buffer.starts_with(&[0x1a, 0x45, 0xdf, 0xa3])
            }
            "DV" => {
                // DV pattern: \x1f\x07\0[\x3f\xbf]
                buffer.len() >= 4
                    && buffer.starts_with(&[0x1f, 0x07, 0x00])
                    && (buffer[3] == 0x3f || buffer[3] == 0xbf)
            }
            "JPEG" => {
                // JPEG pattern: \xff\xd8\xff
                buffer.len() >= 3 && buffer.starts_with(&[0xff, 0xd8, 0xff])
            }
            "M2TS" => {
                // M2TS pattern: (....)?\x47
                // Check for 0x47 sync byte at position 0 or 4
                (!buffer.is_empty() && buffer[0] == 0x47)
                    || (buffer.len() >= 5 && buffer[4] == 0x47)
            }
            "TIFF" => {
                // TIFF pattern: (II|MM)
                buffer.len() >= 2 && (buffer.starts_with(b"II") || buffer.starts_with(b"MM"))
            }
            "GIF" => {
                // GIF pattern: GIF8[79]a - matches GIF87a or GIF89a
                buffer.len() >= 6
                    && buffer.starts_with(b"GIF8")
                    && (buffer[4] == b'7' || buffer[4] == b'9')
                    && buffer[5] == b'a'
            }
            "BMP" => {
                // BMP pattern: BM
                buffer.len() >= 2 && buffer.starts_with(b"BM")
            }
            "ZIP" => {
                // ZIP pattern: PK\x03\x04
                buffer.len() >= 4 && buffer.starts_with(&[0x50, 0x4b, 0x03, 0x04])
            }
            "MRW" => {
                // MRW pattern: \0MR[MI]
                buffer.len() >= 4
                    && buffer[0] == 0x00
                    && buffer[1] == 0x4d
                    && buffer[2] == 0x52
                    && (buffer[3] == 0x4d || buffer[3] == 0x49) // M or I
            }
            "MOV" => {
                // MOV pattern: .{4}(free|skip|wide|ftyp|pnot|PICT|pict|moov|mdat|junk|uuid)
                // Check for common QuickTime/MP4 atoms at offset 4
                if buffer.len() >= 8 {
                    let atom = &buffer[4..8];
                    // Accept all valid MOV/MP4 atoms - specific type determined later
                    // ExifTool doesn't exclude HEIC/HEIF brands at this stage
                    atom == b"free"
                        || atom == b"skip"
                        || atom == b"wide"
                        || atom == b"ftyp"
                        || atom == b"pnot"
                        || atom == b"PICT"
                        || atom == b"pict"
                        || atom == b"moov"
                        || atom == b"mdat"
                        || atom == b"junk"
                        || atom == b"uuid"
                } else {
                    false
                }
            }
            "HEIC" => {
                // HEIC detection: MOV/MP4 container with HEVC-specific ftyp brands
                // HEIC is specifically for HEVC-encoded images
                // ExifTool QuickTime.pm:227 - heic/hevc brands map to HEIC
                if buffer.len() >= 12 && &buffer[4..8] == b"ftyp" {
                    let brand = &buffer[8..12];
                    brand == b"heic" || brand == b"hevc"
                } else {
                    false
                }
            }
            "HEIF" => {
                // HEIF detection: MOV/MP4 container with general HEIF ftyp brands
                // HEIF is the general format (not necessarily HEVC)
                // ExifTool QuickTime.pm:229-231 - mif1/msf1/heix brands map to HEIF
                if buffer.len() >= 12 && &buffer[4..8] == b"ftyp" {
                    let brand = &buffer[8..12];
                    brand == b"mif1"
                        || brand == b"msf1"
                        || brand == b"heix"
                        || brand == b"heim"
                        || brand == b"heis"
                        || brand == b"hevx"
                } else {
                    false
                }
            }
            "AVIF" => {
                // AVIF detection: AV1 Image File Format
                // ExifTool QuickTime.pm:232 - avif brand maps to AVIF
                if buffer.len() >= 12 && &buffer[4..8] == b"ftyp" {
                    let brand = &buffer[8..12];
                    brand == b"avif"
                } else {
                    false
                }
            }
            "XMP" => {
                // XMP pattern: \0{0,3}(\xfe\xff|\xff\xfe|\xef\xbb\xbf)?\0{0,3}\s*<
                // ExifTool.pm:1018 - XMP files start with optional BOM and null bytes, then '<'
                self.validate_xmp_pattern(buffer)
            }
            "MP4" => {
                // MP4 is a specific MOV subtype - use same logic as MOV
                // Check for ftyp atom at offset 4 with MP4-specific brands
                if buffer.len() >= 12 && &buffer[4..8] == b"ftyp" {
                    let brand = &buffer[8..12];
                    // MP4 brands: mp41, mp42, mp4v, isom, etc.
                    brand == b"mp41"
                        || brand == b"mp42"
                        || brand == b"mp4v"
                        || brand == b"isom"
                        || brand == b"M4A "
                        || brand == b"M4V "
                        || brand == b"dash"
                } else {
                    // Also check for other MP4 atoms
                    if buffer.len() >= 8 {
                        let atom = &buffer[4..8];
                        atom == b"free"
                            || atom == b"skip"
                            || atom == b"wide"
                            || atom == b"ftyp"
                            || atom == b"moov"
                            || atom == b"mdat"
                            || atom == b"junk"
                            || atom == b"uuid"
                    } else {
                        false
                    }
                }
            }
            "FLV" => {
                // FLV pattern: FLV\x01
                // ExifTool magic number pattern
                buffer.len() >= 4 && buffer.starts_with(&[0x46, 0x4c, 0x56, 0x01])
            }
            "PSD" => {
                // PSD pattern: 8BPS\0[\x01\x02]
                // ExifTool magic number pattern
                buffer.len() >= 6
                    && buffer.starts_with(&[0x38, 0x42, 0x50, 0x53, 0x00])
                    && (buffer[5] == 0x01 || buffer[5] == 0x02)
            }
            "EPS" => {
                // EPS pattern: (%!PS|%!Ad|\xc5\xd0\xd3\xc6)
                // ExifTool magic number pattern
                buffer.len() >= 4
                    && (buffer.starts_with(b"%!PS")
                        || buffer.starts_with(b"%!Ad")
                        || buffer.starts_with(&[0xc5, 0xd0, 0xd3, 0xc6]))
            }
            "J2C" => {
                // J2C pattern: \xff\x4f\xff\x51\0
                // JPEG 2000 codestream magic number
                buffer.len() >= 5 && buffer.starts_with(&[0xff, 0x4f, 0xff, 0x51, 0x00])
            }
            "JP2" => {
                // JP2 pattern: (\0\0\0\x0cjP(  |\x1a\x1a)\x0d\x0a\x87\x0a|\xff\x4f\xff\x51\0)
                // JPEG 2000 file format magic number
                if buffer.len() >= 12 && buffer.starts_with(&[0x00, 0x00, 0x00, 0x0c]) {
                    // Check for jP signature followed by version/compatibility
                    if buffer[4..6] == [0x6a, 0x50] {
                        // "jP"
                        let version = &buffer[6..8];
                        (version == b"  " || version == [0x1a, 0x1a])
                            && buffer.len() >= 12
                            && buffer[8..12] == [0x0d, 0x0a, 0x87, 0x0a]
                    } else {
                        false
                    }
                } else if buffer.len() >= 5 {
                    // Alternative: JPEG 2000 codestream header
                    buffer.starts_with(&[0xff, 0x4f, 0xff, 0x51, 0x00])
                } else {
                    false
                }
            }
            "ASF" => {
                // ASF pattern: \x30\x26\xb2\x75\x8e\x66\xcf\x11\xa6\xd9\x00\xaa\x00\x62\xce\x6c
                // Windows Media ASF/WMV format
                buffer.len() >= 16
                    && buffer.starts_with(&[
                        0x30, 0x26, 0xb2, 0x75, 0x8e, 0x66, 0xcf, 0x11, 0xa6, 0xd9, 0x00, 0xaa,
                        0x00, 0x62, 0xce, 0x6c,
                    ])
            }
            _ => {
                // If pattern is empty, no magic number exists for this type
                if pattern.is_empty() {
                    return false;
                }

                // For ASCII-only patterns, try regex as a fallback
                if !pattern.contains("\\x") {
                    // Pure ASCII pattern - safe to use regex
                    let rust_pattern = self.convert_perl_pattern_to_rust(pattern);
                    match regex::bytes::RegexBuilder::new(&format!("^{rust_pattern}"))
                        .dot_matches_new_line(true)
                        .build()
                    {
                        Ok(re) => re.is_match(buffer),
                        Err(_) => false, // Reject type if regex fails
                    }
                } else {
                    // Contains hex bytes but we don't have specific handling
                    // Default to rejecting unknown patterns
                    false
                }
            }
        }
    }

    /// Validate magic number for a file type candidate
    /// ExifTool equivalent: magic number testing in ExifTool.pm:2960-2975
    /// CRITICAL: Must match ExifTool's exact logic per TRUST-EXIFTOOL.md
    fn validate_magic_number(&self, file_type: &str, buffer: &[u8]) -> bool {
        // Special handling for RIFF-based formats (AVI, WAV, WEBP, etc.)
        // ExifTool RIFF.pm:2037-2046 - RIFF container detection with format analysis
        if self.is_riff_based_format(file_type) {
            return self.validate_riff_format(file_type, buffer);
        }

        // Special handling for TIFF-based RAW formats that need deeper analysis
        // ExifTool.pm:8531-8612 - DoProcessTIFF() RAW format detection
        if self.is_tiff_based_raw_format(file_type) {
            return self.validate_tiff_raw_format(file_type, buffer);
        }

        // Check if we have a generated magic number pattern
        // ExifTool.pm:912-1027 %magicNumber hash defines magic number patterns
        use crate::generated::file_types::get_magic_number_pattern;
        if let Some(_pattern) = get_magic_number_pattern(file_type) {
            // TODO: Use regex patterns when UTF-8 issue is fixed
            // For now, fall back to binary pattern matching
            return self.match_binary_magic_pattern(file_type, _pattern, buffer);
        }

        // Fall back to hardcoded binary patterns for common file types
        self.match_binary_magic_pattern(file_type, "", buffer)
    }

    /// Detect actual RIFF format type from buffer
    /// ExifTool RIFF.pm:2037-2046 - Detects specific RIFF variant
    fn detect_riff_type(&self, buffer: &[u8]) -> Option<String> {
        // Need at least 12 bytes for RIFF header analysis
        if buffer.len() < 12 {
            return None;
        }

        // Extract RIFF magic signature (bytes 0-3) and format identifier (bytes 8-11)
        let magic = &buffer[0..4];
        let format_id = &buffer[8..12];

        // Check RIFF magic signature first
        // ExifTool RIFF.pm:2040 - "if ($buff =~ /^(RIFF|RF64)....(.{4})/s)"
        let is_riff = magic == b"RIFF" || magic == b"RF64";
        if !is_riff {
            // Check for obscure lossless audio variants
            // ExifTool RIFF.pm:2044 - "return 0 unless $buff =~ /^(LA0[234]|OFR |LPAC|wvpk)/"
            let is_audio_variant = magic == b"LA02"
                || magic == b"LA03"
                || magic == b"LA04"
                || magic == b"OFR "
                || magic == b"LPAC"
                || magic == b"wvpk";
            if !is_audio_variant {
                return None;
            }
        }

        // Map format identifier to file type using ExifTool's riffType mapping
        // ExifTool RIFF.pm:49-53 - %riffType hash
        match format_id {
            b"WAVE" => Some("WAV".to_string()),
            b"AVI " => Some("AVI".to_string()), // Note: AVI has trailing space
            b"WEBP" => Some("WEBP".to_string()),
            b"LA02" | b"LA03" | b"LA04" => Some("LA".to_string()),
            b"OFR " => Some("OFR".to_string()),
            b"LPAC" => Some("PAC".to_string()),
            b"wvpk" => Some("WV".to_string()),
            _ => Some("RIFF".to_string()), // Unknown RIFF format
        }
    }

    /// Check if a file type is based on RIFF container format
    /// ExifTool maps these extensions to RIFF format processing
    fn is_riff_based_format(&self, file_type: &str) -> bool {
        // Check against ExifTool's fileTypeLookup - formats that map to RIFF
        // From file_type_lookup.rs analysis
        matches!(
            file_type,
            "AVI" | "WAV" | "WEBP" | "LA" | "OFR" | "PAC" | "WV"
        )
    }

    /// Check if a file type is a TIFF-based RAW format requiring deeper analysis
    /// ExifTool.pm:8531-8612 - RAW formats detected in DoProcessTIFF()
    fn is_tiff_based_raw_format(&self, file_type: &str) -> bool {
        // RAW formats that use TIFF structure but need specific detection
        // Based on ExifTool's DoProcessTIFF() implementation
        // Note: CR3 is MOV-based, MRW has its own magic number pattern
        matches!(
            file_type,
            "CR2" | "NEF" | "NRW" | "RW2" | "RWL" | "ARW" | "DNG" | "ORF" | "IIQ" | "3FR"
        )
    }

    /// Validate RIFF container and detect specific format
    /// ExifTool equivalent: RIFF.pm:2037-2046 ProcessRIFF()
    /// CRITICAL: Follows ExifTool's exact RIFF detection logic
    fn validate_riff_format(&self, expected_type: &str, buffer: &[u8]) -> bool {
        // Need at least 12 bytes for RIFF header analysis
        // ExifTool RIFF.pm:2039 - "return 0 unless $raf->Read($buff, 12) == 12;"
        if buffer.len() < 12 {
            return false;
        }

        // Extract RIFF magic signature (bytes 0-3) and format identifier (bytes 8-11)
        let magic = &buffer[0..4];
        let format_id = &buffer[8..12];

        // Check RIFF magic signature first
        // ExifTool RIFF.pm:2040 - "if ($buff =~ /^(RIFF|RF64)....(.{4})/s)"
        let is_riff = magic == b"RIFF" || magic == b"RF64";
        if !is_riff {
            // Check for obscure lossless audio variants
            // ExifTool RIFF.pm:2044 - "return 0 unless $buff =~ /^(LA0[234]|OFR |LPAC|wvpk)/"
            let is_audio_variant = magic == b"LA02"
                || magic == b"LA03"
                || magic == b"LA04"
                || magic == b"OFR "
                || magic == b"LPAC"
                || magic == b"wvpk";
            if !is_audio_variant {
                return false;
            }
        }

        // Map format identifier to file type using ExifTool's riffType mapping
        // ExifTool RIFF.pm:49-53 - %riffType hash
        let detected_type = match format_id {
            b"WAVE" => "WAV",
            b"AVI " => "AVI", // Note: AVI has trailing space
            b"WEBP" => "WEBP",
            b"LA02" | b"LA03" | b"LA04" => "LA",
            b"OFR " => "OFR",
            b"LPAC" => "PAC",
            b"wvpk" => "WV",
            _ => {
                // Unknown RIFF format - be conservative and allow generic RIFF detection
                // This matches ExifTool's behavior of processing unknown RIFF types
                return expected_type == "RIFF";
            }
        };

        // Check if detected type matches expected type
        expected_type == detected_type
    }

    /// Validate TIFF-based RAW format with specific signature detection
    /// ExifTool equivalent: DoProcessTIFF() in ExifTool.pm:8531-8612
    /// CRITICAL: Follows ExifTool's exact RAW format detection logic
    fn validate_tiff_raw_format(&self, file_type: &str, buffer: &[u8]) -> bool {
        // Need at least 16 bytes for TIFF header + potential signatures
        if buffer.len() < 16 {
            return false;
        }

        // First check basic TIFF magic number
        if !buffer.starts_with(b"II") && !buffer.starts_with(b"MM") {
            return false;
        }

        // Extract byte order and TIFF identifier
        let little_endian = buffer.starts_with(b"II");
        let identifier = if little_endian {
            u16::from_le_bytes([buffer[2], buffer[3]])
        } else {
            u16::from_be_bytes([buffer[2], buffer[3]])
        };

        // Extract IFD offset
        let ifd_offset = if little_endian {
            u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]])
        } else {
            u32::from_be_bytes([buffer[4], buffer[5], buffer[6], buffer[7]])
        } as usize;

        // Apply ExifTool's RAW format detection logic
        match file_type {
            "CR2" => {
                // CR2 detection: ExifTool.pm:8534-8542
                // identifier == 0x2a and offset >= 16, check for CR\x02\0 signature at offset 8
                if identifier == 0x2a && ifd_offset >= 16 && buffer.len() >= 12 {
                    let sig = &buffer[8..12]; // CR2 signature is at offset 8, not at IFD offset
                    sig == b"CR\x02\0" || sig == b"\xba\xb0\xac\xbb"
                } else {
                    false
                }
            }
            "RW2" | "RWL" => {
                // RW2 detection: ExifTool.pm:8544-8550
                // identifier == 0x55 and specific magic signature at offset 8
                if identifier == 0x55 && ifd_offset >= 0x18 && buffer.len() >= 0x18 {
                    let magic_signature = &buffer[0x08..0x18]; // Magic signature is at offset 8, not 0x18
                    magic_signature
                        == b"\x88\xe7\x74\xd8\xf8\x25\x1d\x4d\x94\x7a\x6e\x77\x82\x2b\x5d\x6a"
                } else {
                    false
                }
            }
            "ORF" => {
                // ORF detection: ExifTool.pm:8552-8555
                // identifier == 0x4f52 or 0x5352 (Olympus specific)
                identifier == 0x4f52 || identifier == 0x5352
            }
            "NEF" | "NRW" => {
                // NEF/NRW detection: Standard TIFF structure (0x2a) but trust extension
                // ExifTool confirms these based on make/model, we trust the extension
                identifier == 0x2a
            }
            "ARW" => {
                // ARW detection: Standard TIFF structure (0x2a) but trust extension
                // ExifTool confirms these based on Sony make/model, we trust the extension
                identifier == 0x2a
            }
            "DNG" => {
                // DNG detection: Standard TIFF structure (0x2a) but trust extension
                // ExifTool confirms these based on DNGVersion tag, we trust the extension
                identifier == 0x2a
            }
            "IIQ" => {
                // IIQ detection: Standard TIFF structure (0x2a) but trust extension
                // Phase One format, trust extension
                identifier == 0x2a
            }
            "3FR" => {
                // 3FR detection: Standard TIFF structure (0x2a) but trust extension
                // Hasselblad format, trust extension
                identifier == 0x2a
            }
            "MRW" => {
                // MRW detection: Has its own magic number pattern in ExifTool
                // Should be handled by magic number lookup, not here
                false
            }
            "CR3" => {
                // CR3 is MOV-based, not TIFF-based - should not reach here
                false
            }
            _ => false,
        }
    }

    /// Last-ditch scan for embedded JPEG/TIFF signatures
    /// ExifTool equivalent: ExifTool.pm:2976-2983
    fn scan_for_embedded_signatures(&self, buffer: &[u8]) -> Option<String> {
        // Look for JPEG signature: \xff\xd8\xff
        if let Some(pos) = buffer.windows(3).position(|w| w == b"\xff\xd8\xff") {
            if pos > 0 {
                eprintln!("Warning: Processing JPEG-like data after unknown {pos}-byte header");
            }
            return Some("JPEG".to_string());
        }

        // Look for TIFF signatures: II*\0 or MM\0*
        if let Some(pos) = buffer
            .windows(4)
            .position(|w| w == b"II*\0" || w == b"MM\0*")
        {
            if pos > 0 {
                eprintln!("Warning: Processing TIFF-like data after unknown {pos}-byte header");
            }
            return Some("TIFF".to_string());
        }

        None
    }

    /// Determine specific file type for MOV/MP4 containers based on ftyp brand
    /// ExifTool equivalent: QuickTime.pm:9868-9877 ftyp brand detection
    fn determine_mov_subtype(&self, buffer: &[u8]) -> Option<String> {
        // Need at least 12 bytes for ftyp atom structure
        if buffer.len() >= 12 && &buffer[4..8] == b"ftyp" {
            let brand = &buffer[8..12];
            // Map ftyp brand to specific file type
            // ExifTool QuickTime.pm:227-232 - %ftypLookup entries
            match brand {
                b"heic" | b"hevc" => Some("HEIC".to_string()),
                b"mif1" | b"msf1" | b"heix" => Some("HEIF".to_string()),
                b"avif" => Some("AVIF".to_string()),
                b"crx " => Some("CR3".to_string()), // Canon RAW 3 format
                // Common MP4 brands
                b"mp41" | b"mp42" | b"mp4v" | b"isom" | b"M4A " | b"M4V " | b"dash" | b"avc1" => {
                    Some("MP4".to_string())
                }
                _ => None, // Keep as MOV for other brands
            }
        } else {
            None
        }
    }

    /// Validate XMP pattern: \0{0,3}(\xfe\xff|\xff\xfe|\xef\xbb\xbf)?\0{0,3}\s*<
    /// ExifTool.pm:1018 - XMP files can start with optional BOM and null bytes, then whitespace, then '<'
    fn validate_xmp_pattern(&self, buffer: &[u8]) -> bool {
        if buffer.is_empty() {
            return false;
        }

        let mut pos = 0;

        // Skip up to 3 null bytes at the beginning
        while pos < buffer.len() && pos < 3 && buffer[pos] == 0 {
            pos += 1;
        }

        // Check for optional BOM (Byte Order Mark)
        if pos + 3 <= buffer.len() {
            // UTF-8 BOM: EF BB BF
            if buffer[pos..pos + 3] == [0xef, 0xbb, 0xbf] {
                pos += 3;
            }
        }
        if pos + 2 <= buffer.len() {
            // UTF-16 BE BOM: FE FF or UTF-16 LE BOM: FF FE
            if buffer[pos..pos + 2] == [0xfe, 0xff] || buffer[pos..pos + 2] == [0xff, 0xfe] {
                pos += 2;
            }
        }

        // Skip up to 3 more null bytes after BOM
        while pos < buffer.len() && pos < 6 && buffer[pos] == 0 {
            pos += 1;
        }

        // Skip whitespace (space, tab, newline, carriage return)
        while pos < buffer.len()
            && (buffer[pos] == b' '
                || buffer[pos] == b'\t'
                || buffer[pos] == b'\n'
                || buffer[pos] == b'\r')
        {
            pos += 1;
        }

        // Finally, check for '<' character
        pos < buffer.len() && buffer[pos] == b'<'
    }

    /// Build final detection result from file type
    pub fn build_result(
        &self,
        file_type: &str,
        path: &Path,
    ) -> Result<FileTypeDetectionResult, FileDetectionError> {
        // Get primary format for processing
        use crate::generated::file_types::resolve_file_type;
        let (format, description) = if let Some((formats, desc)) = resolve_file_type(file_type) {
            (formats[0].to_string(), desc.to_string())
        } else {
            (file_type.to_string(), format!("{file_type} file"))
        };

        // Get MIME type from generated lookup - try the file type first, then fallback, then the format
        // This ensures file-type-specific MIME types take precedence over generic format MIME types
        let mime_type = lookup_mime_types(file_type)
            .or_else(|| self.get_fallback_mime_type(file_type))
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
    fn get_fallback_mime_type(&self, file_type: &str) -> Option<&'static str> {
        match file_type {
            // Image formats
            "JPEG" => Some("image/jpeg"),
            "PNG" => Some("image/png"),
            "TIFF" => Some("image/tiff"),
            "GIF" => Some("image/gif"),
            "BMP" => Some("image/bmp"),
            "WEBP" => Some("image/webp"),
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
}

impl Default for FileTypeDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod mimetypes_validation;

#[cfg(test)]
mod debug_lookup;

#[cfg(test)]
mod test_debug;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_extension_normalization() {
        let detector = FileTypeDetector::new();

        assert_eq!(detector.normalize_extension("tif"), "TIFF");
        assert_eq!(detector.normalize_extension("jpg"), "JPEG");
        assert_eq!(detector.normalize_extension("png"), "PNG");
    }

    #[test]
    fn test_jpeg_detection() {
        let detector = FileTypeDetector::new();
        let path = Path::new("test.jpg");

        // JPEG magic number: \xff\xd8\xff
        let jpeg_data = vec![0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10];
        let mut cursor = Cursor::new(jpeg_data);

        let result = detector.detect_file_type(path, &mut cursor).unwrap();
        assert_eq!(result.file_type, "JPEG");
        assert_eq!(result.format, "JPEG");
        assert_eq!(result.mime_type, "image/jpeg");
    }

    #[test]
    fn test_png_detection() {
        let detector = FileTypeDetector::new();
        let path = Path::new("test.png");

        // PNG magic number: \x89PNG\r\n\x1a\n
        let png_data = vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
        let mut cursor = Cursor::new(png_data);

        let result = detector.detect_file_type(path, &mut cursor).unwrap();
        assert_eq!(result.file_type, "PNG");
        assert_eq!(result.format, "PNG");
        assert_eq!(result.mime_type, "image/png");
    }

    #[test]
    fn test_tiff_extension_alias() {
        let detector = FileTypeDetector::new();
        let path = Path::new("test.tif");

        // TIFF magic number: II*\0 (little endian)
        let tiff_data = vec![0x49, 0x49, 0x2a, 0x00];
        let mut cursor = Cursor::new(tiff_data);

        let result = detector.detect_file_type(path, &mut cursor).unwrap();
        assert_eq!(result.file_type, "TIFF");
        assert_eq!(result.format, "TIFF");
        assert_eq!(result.mime_type, "image/tiff");
    }

    #[test]
    fn test_embedded_jpeg_recovery() {
        let detector = FileTypeDetector::new();
        // Use a filename with unknown extension to trigger embedded signature scan
        let path = Path::new("unknown.xyz");

        // Unknown header followed by JPEG signature
        let mut data = vec![0x00, 0x01, 0x02, 0x03]; // Unknown header
        data.extend_from_slice(&[0xff, 0xd8, 0xff]); // JPEG signature
        let mut cursor = Cursor::new(data);

        let result = detector.detect_file_type(path, &mut cursor).unwrap();
        assert_eq!(result.file_type, "JPEG");
    }

    #[test]
    fn test_weak_magic_mp3() {
        let detector = FileTypeDetector::new();
        let path = Path::new("test.mp3");

        // MP3 has weak magic, should rely on extension
        let mp3_data = vec![0x49, 0x44, 0x33]; // ID3 tag (valid MP3 start)
        let mut cursor = Cursor::new(mp3_data);

        let result = detector.detect_file_type(path, &mut cursor).unwrap();
        assert_eq!(result.file_type, "MP3");
        assert_eq!(result.mime_type, "audio/mpeg");
    }

    #[test]
    fn test_unknown_file_type() {
        let detector = FileTypeDetector::new();
        let path = Path::new("test.unknown");

        let unknown_data = vec![0x00, 0x01, 0x02, 0x03];
        let mut cursor = Cursor::new(unknown_data);

        let result = detector.detect_file_type(path, &mut cursor);
        assert!(matches!(result, Err(FileDetectionError::UnknownFileType)));
    }

    #[test]
    fn test_heic_detection() {
        let detector = FileTypeDetector::new();
        let path = Path::new("test.heic");

        // HEIC file with ftyp box and heic brand
        let mut heic_data = Vec::new();
        heic_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x28]); // Box size (40 bytes)
        heic_data.extend_from_slice(b"ftyp"); // Box type (bytes 4-7)
        heic_data.extend_from_slice(b"heic"); // Major brand (bytes 8-11)
        heic_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // Minor version
        heic_data.extend_from_slice(b"mif1"); // Compatible brand
        heic_data.extend_from_slice(b"MiHE"); // Compatible brand
        heic_data.extend_from_slice(b"MiPr"); // Compatible brand
        heic_data.extend_from_slice(b"miaf"); // Compatible brand
        heic_data.extend_from_slice(b"MiHB"); // Compatible brand
        heic_data.extend_from_slice(b"heic"); // Compatible brand

        let mut cursor = Cursor::new(heic_data);

        let result = detector.detect_file_type(path, &mut cursor).unwrap();
        assert_eq!(result.file_type, "HEIC");
        assert_eq!(result.format, "MOV");
        assert_eq!(result.mime_type, "image/heic");
        assert_eq!(
            result.description,
            "High Efficiency Image Format still image"
        );
    }

    #[test]
    fn test_avi_riff_detection() {
        let detector = FileTypeDetector::new();
        let path = Path::new("test.avi");

        // AVI RIFF header: RIFF + size + "AVI " format identifier
        let mut avi_data = Vec::new();
        avi_data.extend_from_slice(b"RIFF"); // RIFF magic (bytes 0-3)
        avi_data.extend_from_slice(&[0x24, 0x00, 0x00, 0x00]); // File size - 8 (bytes 4-7)
        avi_data.extend_from_slice(b"AVI "); // AVI format identifier (bytes 8-11)
        avi_data.extend_from_slice(b"LIST"); // Chunk header start (bytes 12+)
        let mut cursor = Cursor::new(avi_data);

        let result = detector.detect_file_type(path, &mut cursor).unwrap();
        assert_eq!(result.file_type, "AVI");
        assert_eq!(result.format, "RIFF");
        assert_eq!(result.mime_type, "video/x-msvideo");
    }

    #[test]
    fn test_wav_riff_detection() {
        let detector = FileTypeDetector::new();
        let path = Path::new("test.wav");

        // WAV RIFF header: RIFF + size + "WAVE" format identifier
        let mut wav_data = Vec::new();
        wav_data.extend_from_slice(b"RIFF"); // RIFF magic (bytes 0-3)
        wav_data.extend_from_slice(&[0x24, 0x00, 0x00, 0x00]); // File size - 8 (bytes 4-7)
        wav_data.extend_from_slice(b"WAVE"); // WAVE format identifier (bytes 8-11)
        wav_data.extend_from_slice(b"fmt "); // Format chunk start (bytes 12+)
        let mut cursor = Cursor::new(wav_data);

        let result = detector.detect_file_type(path, &mut cursor).unwrap();
        assert_eq!(result.file_type, "WAV");
        assert_eq!(result.format, "RIFF");
        assert_eq!(result.mime_type, "audio/x-wav");
    }

    #[test]
    fn test_webp_riff_detection() {
        let detector = FileTypeDetector::new();
        let path = Path::new("test.webp");

        // WebP RIFF header: RIFF + size + "WEBP" format identifier
        let mut webp_data = Vec::new();
        webp_data.extend_from_slice(b"RIFF"); // RIFF magic (bytes 0-3)
        webp_data.extend_from_slice(&[0x20, 0x00, 0x00, 0x00]); // File size - 8 (bytes 4-7)
        webp_data.extend_from_slice(b"WEBP"); // WEBP format identifier (bytes 8-11)
        webp_data.extend_from_slice(b"VP8 "); // VP8 chunk header (bytes 12+)
        let mut cursor = Cursor::new(webp_data);

        let result = detector.detect_file_type(path, &mut cursor).unwrap();
        assert_eq!(result.file_type, "WEBP");
        assert_eq!(result.format, "RIFF");
        assert_eq!(result.mime_type, "image/webp");
    }

    #[test]
    fn test_riff_format_mismatch() {
        let detector = FileTypeDetector::new();
        let path = Path::new("test.avi"); // AVI extension

        // But contains WAV data - should fail validation
        let mut wav_data = Vec::new();
        wav_data.extend_from_slice(b"RIFF"); // RIFF magic
        wav_data.extend_from_slice(&[0x24, 0x00, 0x00, 0x00]); // File size
        wav_data.extend_from_slice(b"WAVE"); // WAVE format (not AVI)
        wav_data.extend_from_slice(b"fmt "); // Format chunk
        let mut cursor = Cursor::new(wav_data);

        // Should fail since extension says AVI but content is WAV
        let result = detector.detect_file_type(path, &mut cursor);
        assert!(matches!(result, Err(FileDetectionError::UnknownFileType)));
    }
}
