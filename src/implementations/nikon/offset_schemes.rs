//! Nikon offset scheme calculations for different MakerNote formats
//!
//! **Trust ExifTool**: This code translates ExifTool's Nikon offset calculations verbatim.
//!
//! ExifTool Reference: lib/Image/ExifTool/Nikon.pm lines 890-934
//!
//! Nikon cameras use different offset schemes based on the MakerNote format version:
//! - Format1: Early cameras with offset at +0x06
//! - Format2: Mid-generation with offset at +0x08  
//! - Format3: Modern cameras with TIFF header at +0x0a

use super::detection::NikonFormat;
use tracing::{debug, trace};

/// Calculate Nikon base offset for IFD processing based on format version
/// ExifTool: Nikon.pm offset calculation logic (lines 890-934)
pub fn calculate_nikon_base_offset(format: NikonFormat, data_pos: usize) -> usize {
    let base_offset = match format {
        // Format3: Modern Nikon with TIFF header at 0x0a
        // ExifTool: if ($format eq 'Format3') { $base = $dataPos + 0x0a; }
        NikonFormat::Format3 => {
            let offset = data_pos + 0x0a;
            debug!("Nikon Format3 base offset: {:#x} (data_pos + 0x0a)", offset);
            offset
        }
        // Format2: Mid-generation Nikon with offset at 0x08
        // ExifTool: if ($format eq 'Format2') { $base = $dataPos + 0x08; }
        NikonFormat::Format2 => {
            let offset = data_pos + 0x08;
            debug!("Nikon Format2 base offset: {:#x} (data_pos + 0x08)", offset);
            offset
        }
        // Format1: Early Nikon format with offset at 0x06
        // ExifTool: Default handling for early format
        NikonFormat::Format1 => {
            let offset = data_pos + 0x06;
            debug!("Nikon Format1 base offset: {:#x} (data_pos + 0x06)", offset);
            offset
        }
    };

    trace!(
        "Calculated Nikon base offset: {:#x} for format {:?}",
        base_offset,
        format
    );
    base_offset
}

/// Validate Nikon offset bounds and format consistency
/// ExifTool: Nikon.pm offset validation and bounds checking
pub fn validate_nikon_offset(
    format: NikonFormat,
    data_pos: usize,
    data_len: usize,
) -> Result<(), String> {
    let base_offset = calculate_nikon_base_offset(format, data_pos);

    // Basic bounds checking
    if base_offset >= data_len {
        return Err(format!(
            "Nikon {format:?} base offset {base_offset:#x} beyond data bounds ({data_len})"
        ));
    }

    // Format-specific validation
    match format {
        NikonFormat::Format3 => {
            // Format3 needs space for TIFF header validation
            if base_offset + 8 > data_len {
                return Err(format!(
                    "Nikon Format3 needs 8 bytes for TIFF header at {:#x}, only {} available",
                    base_offset,
                    data_len - base_offset
                ));
            }
        }
        NikonFormat::Format2 | NikonFormat::Format1 => {
            // Other formats need minimum IFD space (2 bytes for entry count)
            if base_offset + 2 > data_len {
                return Err(format!(
                    "Nikon {:?} needs 2 bytes for IFD entry count at {:#x}, only {} available",
                    format,
                    base_offset,
                    data_len - base_offset
                ));
            }
        }
    }

    debug!(
        "Nikon offset validation passed for {:?} at {:#x}",
        format, base_offset
    );
    Ok(())
}

/// Get expected minimum header size for Nikon format
/// ExifTool: Format-specific header size requirements
pub fn get_nikon_header_size(format: NikonFormat) -> usize {
    match format {
        // Format3 has TIFF header + IFD structure
        NikonFormat::Format3 => 0x0a + 8, // TIFF header offset + min TIFF header size
        // Format2 has intermediate header
        NikonFormat::Format2 => 0x08 + 2, // Header offset + min IFD size
        // Format1 has minimal header
        NikonFormat::Format1 => 0x06 + 2, // Header offset + min IFD size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format3_offset_calculation() {
        let data_pos = 100;
        let base = calculate_nikon_base_offset(NikonFormat::Format3, data_pos);
        assert_eq!(base, 110); // 100 + 0x0a
    }

    #[test]
    fn test_format2_offset_calculation() {
        let data_pos = 200;
        let base = calculate_nikon_base_offset(NikonFormat::Format2, data_pos);
        assert_eq!(base, 208); // 200 + 0x08
    }

    #[test]
    fn test_format1_offset_calculation() {
        let data_pos = 50;
        let base = calculate_nikon_base_offset(NikonFormat::Format1, data_pos);
        assert_eq!(base, 56); // 50 + 0x06
    }

    #[test]
    fn test_offset_validation_success() {
        let result = validate_nikon_offset(NikonFormat::Format3, 100, 500);
        assert!(result.is_ok());
    }

    #[test]
    fn test_offset_validation_beyond_bounds() {
        let result = validate_nikon_offset(NikonFormat::Format3, 490, 500);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("beyond data bounds"));
    }

    #[test]
    fn test_format3_insufficient_tiff_header_space() {
        // Base offset = 490 + 0x0a = 500, needs 500 + 8 = 508, but only 507 available
        let result = validate_nikon_offset(NikonFormat::Format3, 490, 507);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("TIFF header"));
    }

    #[test]
    fn test_format1_insufficient_ifd_space() {
        // Base offset = 493 + 0x06 = 499, needs 499 + 2 = 501, but only 500 available
        let result = validate_nikon_offset(NikonFormat::Format1, 493, 500);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("IFD entry count"));
    }

    #[test]
    fn test_header_size_calculations() {
        assert_eq!(get_nikon_header_size(NikonFormat::Format3), 18); // 0x0a + 8
        assert_eq!(get_nikon_header_size(NikonFormat::Format2), 10); // 0x08 + 2
        assert_eq!(get_nikon_header_size(NikonFormat::Format1), 8); // 0x06 + 2
    }
}
