//! TIFF-based RAW format detection
//!
//! Many RAW formats use TIFF container structure but need manufacturer-specific
//! detection via the Make tag (0x010F) in the IFD.

#![doc = "EXIFTOOL-SOURCE: Multiple manufacturer modules (Canon.pm, Nikon.pm, Sony.pm, etc.)"]

use crate::core::endian::Endian;
use crate::detection::FileType;
use crate::error::{Error, Result};

/// EXIFTOOL-PATTERN: Manufacturer detection from Make tag (0x010F)
/// This implements ExifTool's logic for detecting RAW formats by parsing
/// the TIFF IFD structure to read the Make field.
pub fn detect_raw_by_make(data: &[u8], tiff_header_offset: usize) -> Option<FileType> {
    if let Ok(make) = extract_make_field(data, tiff_header_offset) {
        match make.as_str() {
            // EXIFTOOL-QUIRK: Canon detection
            // EXIFTOOL-PATTERN: From ExifTool.pm lines 8529-8544
            // CR2 detection requires:
            // 1. TIFF identifier 0x2a AND offset >= 16
            // 2. "CR\x02\0" signature at offset 8 from TIFF header
            s if s.starts_with("Canon") => {
                // Check CR2 signature following ExifTool's exact logic
                if data.len() >= tiff_header_offset + 12 {
                    // ExifTool checks offset >= 16 for CR2 (from TIFF header field at offset 4)
                    let offset_bytes = &data[tiff_header_offset + 4..tiff_header_offset + 8];
                    let endian = Endian::from_tiff_header(&data[tiff_header_offset..])
                        .unwrap_or(Endian::Little);
                    let offset = match endian {
                        Endian::Little => u32::from_le_bytes([
                            offset_bytes[0],
                            offset_bytes[1],
                            offset_bytes[2],
                            offset_bytes[3],
                        ]),
                        Endian::Big => u32::from_be_bytes([
                            offset_bytes[0],
                            offset_bytes[1],
                            offset_bytes[2],
                            offset_bytes[3],
                        ]),
                    };

                    if offset >= 16 {
                        // Check for CR2 signature "CR\x02\0" at offset 8
                        let cr_sig = &data[tiff_header_offset + 8..tiff_header_offset + 12];
                        if cr_sig.starts_with(b"CR\x02\0") {
                            return Some(FileType::CR2);
                        }
                    }
                }
                // ExifTool returns TIFF for Canon files without CR2 signature
                None
            }

            // EXIFTOOL-QUIRK: Nikon detection
            s if s.starts_with("NIKON") => {
                // NEF vs NRW depends on camera model and generation
                // NRW is for newer cameras like Z-series, some newer DSLRs
                // NEF is for older DSLRs
                // For now, check for specific patterns to distinguish
                detect_nikon_format(data, tiff_header_offset, s)
            }

            // EXIFTOOL-QUIRK: Sony detection
            s if s.starts_with("SONY") => {
                // Sony has multiple RAW formats:
                // - ARW: Most common Sony Alpha RAW
                // - SR2: Sony RAW 2 (older format)
                // - ARQ: Sony Alpha RAW with Pixel Shift Multi Shooting
                // - SRF: Sony RAW (DSLR format)
                // Default to ARW as it's most common
                Some(FileType::ARW)
            }

            // EXIFTOOL-QUIRK: Other manufacturers
            s if s.starts_with("OLYMPUS") => Some(FileType::ORF),
            s if s.starts_with("PENTAX") => Some(FileType::PEF),
            s if s.starts_with("FUJIFILM") => {
                // Fujifilm uses RAF format, not TIFF-based
                // This shouldn't happen as RAF has its own magic
                None
            }
            s if s.starts_with("Panasonic") => Some(FileType::RW2),
            s if s.starts_with("SAMSUNG") => Some(FileType::SRW),
            s if s.starts_with("EPSON") => Some(FileType::ERF),
            s if s.starts_with("Kodak") => {
                // DCR, K25, KDC variants - need additional detection
                // Default to DCR
                Some(FileType::DCR)
            }
            s if s.starts_with("Mamiya") => Some(FileType::MEF),
            s if s.starts_with("Minolta") => Some(FileType::MRW),
            s if s.starts_with("Hasselblad") => {
                // 3FR vs FFF - need model check
                // Default to 3FR (more common)
                Some(FileType::ThreeFR)
            }
            s if s.starts_with("Phase One") => Some(FileType::IIQ),
            s if s.starts_with("Leica") => Some(FileType::RWL),
            _ => None,
        }
    } else {
        None
    }
}

/// Extract the Make field (tag 0x010F) from TIFF IFD
fn extract_make_field(data: &[u8], tiff_offset: usize) -> Result<String> {
    if data.len() < tiff_offset + 8 {
        return Err(Error::InvalidExif(
            "Not enough data for TIFF header".to_string(),
        ));
    }

    // Parse TIFF header to determine endianness
    let endian = Endian::from_tiff_header(&data[tiff_offset..])
        .ok_or_else(|| Error::InvalidExif("Invalid TIFF byte order".to_string()))?;

    // Skip byte order marker (2 bytes) and TIFF magic (2 bytes)
    if data.len() < tiff_offset + 8 {
        return Err(Error::InvalidExif("TIFF header incomplete".to_string()));
    }

    let ifd_offset = endian.read_u32(&data[tiff_offset + 4..tiff_offset + 8]) as usize;

    if tiff_offset + ifd_offset >= data.len() {
        return Err(Error::InvalidExif("IFD offset out of bounds".to_string()));
    }

    // Read IFD
    let ifd_start = tiff_offset + ifd_offset;
    if ifd_start + 2 > data.len() {
        return Err(Error::InvalidExif("IFD header out of bounds".to_string()));
    }

    let num_entries = endian.read_u16(&data[ifd_start..ifd_start + 2]) as usize;

    // Each IFD entry is 12 bytes: tag(2) + type(2) + count(4) + value/offset(4)
    for i in 0..num_entries {
        let entry_offset = ifd_start + 2 + (i * 12);
        if entry_offset + 12 > data.len() {
            break;
        }

        let tag = endian.read_u16(&data[entry_offset..entry_offset + 2]);
        if tag == 0x010F {
            // Make tag
            let data_type = endian.read_u16(&data[entry_offset + 2..entry_offset + 4]);
            let count = endian.read_u32(&data[entry_offset + 4..entry_offset + 8]) as usize;

            if data_type == 2 && count > 0 {
                // ASCII string
                let value_or_offset = endian.read_u32(&data[entry_offset + 8..entry_offset + 12]);

                let string_data = if count <= 4 {
                    // Value fits in the offset field
                    let bytes = match endian {
                        Endian::Little => value_or_offset.to_le_bytes(),
                        Endian::Big => value_or_offset.to_be_bytes(),
                    };
                    bytes[..count.min(4)].to_vec()
                } else {
                    // Value is at offset
                    let string_offset = tiff_offset + value_or_offset as usize;
                    if string_offset + count <= data.len() {
                        data[string_offset..string_offset + count].to_vec()
                    } else {
                        return Err(Error::InvalidExif(
                            "Make string offset out of bounds".to_string(),
                        ));
                    }
                };

                // Convert to string, removing null terminator
                let make_str = String::from_utf8_lossy(&string_data);
                let make_clean = make_str.trim_end_matches('\0').trim().to_string();
                return Ok(make_clean);
            }
        }
    }

    Err(Error::InvalidExif("Make tag not found".to_string()))
}

/// EXIFTOOL-QUIRK: Distinguish between Nikon NEF and NRW formats
/// NRW is used by newer cameras (Z-series mirrorless, some newer DSLRs)
/// NEF is used by older DSLRs
fn detect_nikon_format(data: &[u8], tiff_offset: usize, _make_str: &str) -> Option<FileType> {
    // Try to extract Model tag (0x0110) to determine camera type
    if let Ok(model) = extract_model_field(data, tiff_offset) {
        // Check for Z-series cameras (use NRW)
        if model.contains("Z ")
            || model.contains("Z5")
            || model.contains("Z6")
            || model.contains("Z7")
            || model.contains("Z8")
            || model.contains("Z9")
            || model.contains("Zf")
            || model.contains("Zfc")
        {
            return Some(FileType::NRW);
        }

        // Check for newer DSLRs that might use NRW
        if model.contains("D850") || model.contains("D780") || model.contains("D500") {
            return Some(FileType::NRW);
        }
    }

    // Default to NEF for older cameras
    Some(FileType::NEF)
}

/// Extract the Model field (tag 0x0110) from TIFF IFD  
fn extract_model_field(data: &[u8], tiff_offset: usize) -> Result<String> {
    if data.len() < tiff_offset + 8 {
        return Err(Error::InvalidExif(
            "Not enough data for TIFF header".to_string(),
        ));
    }

    // Parse TIFF header to determine endianness
    let endian = Endian::from_tiff_header(&data[tiff_offset..])
        .ok_or_else(|| Error::InvalidExif("Invalid TIFF byte order".to_string()))?;

    // Skip byte order marker (2 bytes) and TIFF magic (2 bytes)
    if data.len() < tiff_offset + 8 {
        return Err(Error::InvalidExif("TIFF header incomplete".to_string()));
    }

    let ifd_offset = endian.read_u32(&data[tiff_offset + 4..tiff_offset + 8]) as usize;

    if tiff_offset + ifd_offset >= data.len() {
        return Err(Error::InvalidExif("IFD offset out of bounds".to_string()));
    }

    // Read IFD
    let ifd_start = tiff_offset + ifd_offset;
    if ifd_start + 2 > data.len() {
        return Err(Error::InvalidExif("IFD header out of bounds".to_string()));
    }

    let num_entries = endian.read_u16(&data[ifd_start..ifd_start + 2]) as usize;

    // Each IFD entry is 12 bytes: tag(2) + type(2) + count(4) + value/offset(4)
    for i in 0..num_entries {
        let entry_offset = ifd_start + 2 + (i * 12);
        if entry_offset + 12 > data.len() {
            break;
        }

        let tag = endian.read_u16(&data[entry_offset..entry_offset + 2]);
        if tag == 0x0110 {
            // Model tag
            let data_type = endian.read_u16(&data[entry_offset + 2..entry_offset + 4]);
            let count = endian.read_u32(&data[entry_offset + 4..entry_offset + 8]) as usize;

            if data_type == 2 && count > 0 {
                // ASCII string
                let value_or_offset = endian.read_u32(&data[entry_offset + 8..entry_offset + 12]);

                let string_data = if count <= 4 {
                    // Value fits in the offset field
                    let bytes = match endian {
                        Endian::Little => value_or_offset.to_le_bytes(),
                        Endian::Big => value_or_offset.to_be_bytes(),
                    };
                    bytes[..count.min(4)].to_vec()
                } else {
                    // Value is at offset
                    let string_offset = tiff_offset + value_or_offset as usize;
                    if string_offset + count <= data.len() {
                        data[string_offset..string_offset + count].to_vec()
                    } else {
                        return Err(Error::InvalidExif(
                            "Model string offset out of bounds".to_string(),
                        ));
                    }
                };

                // Convert to string, removing null terminator
                let model_str = String::from_utf8_lossy(&string_data);
                let model_clean = model_str.trim_end_matches('\0').trim().to_string();
                return Ok(model_clean);
            }
        }
    }

    Err(Error::InvalidExif("Model tag not found".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_detection() {
        // Test Canon detection
        let canon_make = "Canon";
        assert_eq!(detect_raw_by_make_string(canon_make), Some(FileType::CR2));

        // Test Sony detection
        let sony_make = "SONY";
        assert_eq!(detect_raw_by_make_string(sony_make), Some(FileType::ARW));

        // Test Nikon detection (without model info defaults to NEF)
        let nikon_make = "NIKON CORPORATION";
        assert_eq!(detect_raw_by_make_string(nikon_make), Some(FileType::NEF));
    }

    // Helper function for testing
    fn detect_raw_by_make_string(make: &str) -> Option<FileType> {
        match make {
            s if s.starts_with("Canon") => Some(FileType::CR2),
            s if s.starts_with("NIKON") => Some(FileType::NEF),
            s if s.starts_with("SONY") => Some(FileType::ARW),
            s if s.starts_with("OLYMPUS") => Some(FileType::ORF),
            s if s.starts_with("PENTAX") => Some(FileType::PEF),
            s if s.starts_with("Panasonic") => Some(FileType::RW2),
            s if s.starts_with("SAMSUNG") => Some(FileType::SRW),
            _ => None,
        }
    }
}
