//! MOV/Video format detection utilities
//!
//! Handles detection of MOV containers and video format subtypes.

/// Determine specific file type for MOV/MP4 containers based on ftyp brand
/// ExifTool equivalent: QuickTime.pm:9868-9877 ftyp brand detection
pub fn determine_mov_subtype(buffer: &[u8]) -> Option<String> {
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
