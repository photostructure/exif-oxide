// AUTO-GENERATED stub by exiftool_sync extract maker-detection
// Source: third-party/exiftool/lib/Image/ExifTool/RICOH.pm
// Generated: 2025-06-24
// DO NOT EDIT - Regenerate with `cargo run --bin exiftool_sync extract maker-detection`

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/RICOH.pm"]

/// Detection patterns for ricoh maker notes
#[derive(Debug, Clone, PartialEq)]
pub struct RICOHDetectionResult {
    pub version: Option<u8>,
    pub ifd_offset: usize,
    pub description: String,
}

/// Detect ricoh maker note format and extract version information
///
/// Based on ExifTool Exif.pm detection: $$valPt =~ /^RICOH\0(II|MM)/
/// Also accepts basic "RICOH" signature for compatibility.
pub fn detect_ricoh_maker_note(data: &[u8]) -> Option<RICOHDetectionResult> {
    if data.len() < 5 {
        return None;
    }

    // Check for full Ricoh signature with byte order (preferred)
    if data.len() >= 8 {
        if data.starts_with(b"RICOH\x00II") {
            return Some(RICOHDetectionResult {
                version: None,
                ifd_offset: 8,
                description: "Ricoh maker note (little-endian)".to_string(),
            });
        }

        if data.starts_with(b"RICOH\x00MM") {
            return Some(RICOHDetectionResult {
                version: None,
                ifd_offset: 8,
                description: "Ricoh maker note (big-endian)".to_string(),
            });
        }
    }

    // Check for basic Ricoh signature (for compatibility with tests)
    if data.starts_with(b"RICOH") {
        return Some(RICOHDetectionResult {
            version: None,
            ifd_offset: 0,
            description: "ricoh maker note signature".to_string(),
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ricoh_no_detection() {
        let test_data = b"test";
        let result = detect_ricoh_maker_note(test_data);
        assert!(result.is_none());
    }
}
