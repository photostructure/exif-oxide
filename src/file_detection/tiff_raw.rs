//! TIFF-based RAW format detection
//!
//! Handles validation of TIFF-based RAW formats that require specific signature detection.

/// Check if a file type is a TIFF-based RAW format requiring deeper analysis
/// ExifTool.pm:8531-8612 - RAW formats detected in DoProcessTIFF()
pub fn is_tiff_based_raw_format(file_type: &str) -> bool {
    // RAW formats that use TIFF structure but need specific detection
    // Based on ExifTool's DoProcessTIFF() implementation
    // Note: CR3 is MOV-based, MRW has its own magic number pattern
    matches!(
        file_type,
        "CR2" | "NEF" | "NRW" | "RW2" | "RWL" | "ARW" | "DNG" | "ORF" | "IIQ" | "3FR"
    )
}

/// Validate TIFF-based RAW format with specific signature detection
/// ExifTool equivalent: DoProcessTIFF() in ExifTool.pm:8531-8612
/// CRITICAL: Follows ExifTool's exact RAW format detection logic
pub fn validate_tiff_raw_format(file_type: &str, buffer: &[u8]) -> bool {
    // Need at least 16 bytes for TIFF header + potential signatures
    if buffer.len() < 16 {
        return false;
    }

    // First check basic TIFF magic number
    if !buffer.starts_with(b"II") && !buffer.starts_with(b"MM") {
        return false;
    }

    // CRITICAL: CR3 is MOV-based, not TIFF-based! Check for MOV signature first
    // ExifTool.pm - CR3 uses QuickTime.pm not TIFF processing
    if file_type == "CR3" && buffer.len() >= 12 && &buffer[4..8] == b"ftyp" {
        // This is a MOV-based file, not TIFF - return false to prevent TIFF processing
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
            // NEF/NRW detection: Both use standard TIFF structure
            // The distinction is made by file extension (see file type detection logic)
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
            // This case exists only for completeness - validate_tiff_raw_format checks MOV signature
            false
        }
        _ => false,
    }
}
