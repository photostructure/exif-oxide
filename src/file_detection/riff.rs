//! RIFF format detection and validation
//!
//! Handles detection of RIFF-based formats like AVI, WAV, WEBP, etc.

/// Detect actual RIFF format type from buffer
/// ExifTool RIFF.pm:2037-2046 - Detects specific RIFF variant
pub fn detect_riff_type(buffer: &[u8]) -> Option<String> {
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
pub fn is_riff_based_format(file_type: &str) -> bool {
    // Check against ExifTool's fileTypeLookup - formats that map to RIFF
    // From file_type_lookup.rs analysis
    matches!(
        file_type,
        "AVI" | "WAV" | "WEBP" | "LA" | "OFR" | "PAC" | "WV"
    )
}

/// Validate RIFF container and detect specific format
/// ExifTool equivalent: RIFF.pm:2037-2046 ProcessRIFF()
/// CRITICAL: Follows ExifTool's exact RIFF detection logic
pub fn validate_riff_format(expected_type: &str, buffer: &[u8]) -> bool {
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
