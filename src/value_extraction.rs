//! Value extraction functions for EXIF/TIFF data
//!
//! This module contains pure functions for extracting typed values from EXIF/TIFF data,
//! translating ExifTool's value extraction logic from lib/Image/ExifTool/Exif.pm.
//! These functions handle inline vs offset storage, byte order conversion, and array processing.

use crate::tiff_types::{ByteOrder, IfdEntry};
use crate::types::{ExifError, Result, TagValue};
use tracing::debug;

/// Extract ASCII string value from IFD entry
/// ExifTool: lib/Image/ExifTool/Exif.pm:6372-6398 ASCII value handling
pub fn extract_ascii_value(
    data: &[u8],
    entry: &IfdEntry,
    byte_order: ByteOrder,
    tag_id: u16,
) -> Result<String> {
    // debug!("extract_ascii_value: tag {:#x}, count {}, is_inline: {}, value_or_offset: {:#x}",
    //        entry.tag_id, entry.count, entry.is_inline(), entry.value_or_offset);

    let value_data = if entry.is_inline() {
        // Value stored inline in the 4-byte value field
        // ExifTool: lib/Image/ExifTool/Exif.pm:6372 inline value handling
        let bytes = match byte_order {
            ByteOrder::BigEndian => entry.value_or_offset.to_be_bytes(),
            ByteOrder::LittleEndian => entry.value_or_offset.to_le_bytes(),
        };
        // debug!("Inline bytes for tag {:#x}: {:02x?}", entry.tag_id, &bytes[..entry.count.min(4) as usize]);
        bytes[..entry.count.min(4) as usize].to_vec()
    } else {
        // Value stored at offset
        // ExifTool: lib/Image/ExifTool/Exif.pm:6398 offset value handling
        let offset = entry.value_or_offset as usize;
        let size = entry.count as usize;

        if offset + size > data.len() {
            return Err(ExifError::ParseError(format!(
                "ASCII value offset {offset:#x} + size {size} beyond data bounds"
            )));
        }

        data[offset..offset + size].to_vec()
    };

    // Convert bytes to string with null-termination handling
    // ExifTool handles null-terminated strings gracefully
    let null_pos = value_data
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(value_data.len());
    let trimmed = &value_data[..null_pos];

    // Convert to UTF-8, handling invalid sequences gracefully
    match String::from_utf8(trimmed.to_vec()) {
        Ok(s) => {
            // Tag-specific trimming behavior based on ExifTool RawConv expressions
            // ExifTool: lib/Image/ExifTool/Exif.pm tags with RawConv => '$val =~ s/\0.*//; $val'
            // ImageDescription (0x010E) preserves whitespace, unlike Make/Model/Software
            if tag_id == 0x010E {
                // ImageDescription: Only remove null bytes, preserve other whitespace
                // ExifTool: lib/Image/ExifTool/Exif.pm:549 ImageDescription has no RawConv trimming
                Ok(s.replace('\0', ""))
            } else {
                // Other ASCII tags: Apply normal trimming (Make, Model, Software, etc.)
                // ExifTool: Make/Model/Software have RawConv => '$val =~ s/\0.*//; $val'
                Ok(s.trim().to_string())
            }
        }
        Err(_) => {
            // Fallback for invalid UTF-8 - convert lossy with same tag-specific logic
            let s = String::from_utf8_lossy(trimmed);
            if tag_id == 0x010E {
                Ok(s.replace('\0', ""))
            } else {
                Ok(s.trim().to_string())
            }
        }
    }
}

/// Extract SHORT (u16) value from IFD entry
/// ExifTool: lib/Image/ExifTool/Exif.pm:6372-6398 value extraction
pub fn extract_short_value(data: &[u8], entry: &IfdEntry, byte_order: ByteOrder) -> Result<u16> {
    if entry.count != 1 {
        return Err(ExifError::ParseError(format!(
            "SHORT value with count {} not supported yet",
            entry.count
        )));
    }

    if entry.is_inline() {
        // Value stored inline - use lower 2 bytes of value_or_offset
        // ExifTool: lib/Image/ExifTool/Exif.pm:6372 inline value handling
        // The value_or_offset field is always stored in the file's byte order
        let bytes = match byte_order {
            ByteOrder::LittleEndian => entry.value_or_offset.to_le_bytes(),
            ByteOrder::BigEndian => entry.value_or_offset.to_be_bytes(),
        };
        // For inline SHORT values, use the first 2 bytes in the correct order
        Ok(match byte_order {
            ByteOrder::LittleEndian => u16::from_le_bytes([bytes[0], bytes[1]]),
            ByteOrder::BigEndian => u16::from_be_bytes([bytes[0], bytes[1]]),
        })
    } else {
        // Value stored at offset
        let offset = entry.value_or_offset as usize;
        byte_order.read_u16(data, offset)
    }
}

/// Extract BYTE (u8) value from IFD entry
/// ExifTool: lib/Image/ExifTool/Exif.pm:6372-6398 value extraction
pub fn extract_byte_value(data: &[u8], entry: &IfdEntry) -> Result<u8> {
    if entry.count != 1 {
        return Err(ExifError::ParseError(format!(
            "BYTE value with count {} not supported yet",
            entry.count
        )));
    }

    if entry.is_inline() {
        // Value stored inline - use lowest byte of value_or_offset
        // ExifTool: lib/Image/ExifTool/Exif.pm:6372 inline value handling
        Ok(entry.value_or_offset as u8)
    } else {
        // Value stored at offset
        let offset = entry.value_or_offset as usize;
        if offset >= data.len() {
            return Err(ExifError::ParseError(format!(
                "BYTE value offset {offset:#x} beyond data bounds"
            )));
        }
        Ok(data[offset])
    }
}

/// Extract BYTE array values from IFD entry
/// ExifTool: lib/Image/ExifTool/Exif.pm:6372-6398 value extraction
/// This handles BYTE format with count > 1 (arrays of unsigned 8-bit integers)
pub fn extract_byte_array_value(data: &[u8], entry: &IfdEntry) -> Result<Vec<u8>> {
    let count = entry.count as usize;

    if entry.is_inline() && count <= 4 {
        // Value stored inline - extract bytes from value_or_offset
        // ExifTool: lib/Image/ExifTool/Exif.pm:6372 inline value handling
        let bytes = entry.value_or_offset.to_le_bytes();
        Ok(bytes[..count].to_vec())
    } else if entry.is_inline() {
        // Should not happen - more than 4 bytes can't fit inline
        return Err(ExifError::ParseError(format!(
            "BYTE array with count {count} cannot be stored inline"
        )));
    } else {
        // Value stored at offset
        let offset = entry.value_or_offset as usize;
        if offset + count > data.len() {
            return Err(ExifError::ParseError(format!(
                "BYTE array offset {offset:#x} + count {count} beyond data bounds"
            )));
        }
        Ok(data[offset..offset + count].to_vec())
    }
}

/// Extract SHORT array value from IFD entry (SHORT format with count > 1)
/// ExifTool: lib/Image/ExifTool/Exif.pm:6372-6398 value extraction
/// This handles SHORT format with count > 1 (arrays of unsigned 16-bit integers)
/// Critical for Canon binary data extraction (CameraSettings, ShotInfo, etc.)
pub fn extract_short_array_value(
    data: &[u8],
    entry: &IfdEntry,
    byte_order: ByteOrder,
) -> Result<Vec<u16>> {
    let count = entry.count as usize;
    let mut values = Vec::with_capacity(count);

    if entry.is_inline() && count <= 2 {
        // Value stored inline - up to 2 SHORT values can fit in 4 bytes
        // ExifTool: lib/Image/ExifTool/Exif.pm:6372 inline value handling
        let bytes = entry.value_or_offset.to_le_bytes();

        // Convert bytes based on IFD byte order (not necessarily LE)
        for i in 0..count {
            let offset = i * 2;
            let value = match byte_order {
                ByteOrder::LittleEndian => u16::from_le_bytes([bytes[offset], bytes[offset + 1]]),
                ByteOrder::BigEndian => u16::from_be_bytes([bytes[offset], bytes[offset + 1]]),
            };
            values.push(value);
        }
        Ok(values)
    } else if entry.is_inline() {
        // Should not happen - more than 2 SHORT values can't fit inline
        return Err(ExifError::ParseError(format!(
            "SHORT array with count {count} cannot be stored inline"
        )));
    } else {
        // Value stored at offset
        let offset = entry.value_or_offset as usize;
        let total_size = count * 2; // 2 bytes per SHORT

        if offset + total_size > data.len() {
            return Err(ExifError::ParseError(format!(
                "SHORT array offset {offset:#x} + size {total_size} beyond data bounds"
            )));
        }

        // Read each SHORT value with proper byte order
        for i in 0..count {
            let value_offset = offset + (i * 2);
            let value = byte_order.read_u16(data, value_offset)?;
            values.push(value);
        }

        Ok(values)
    }
}

/// Extract LONG (u32) value from IFD entry
/// ExifTool: lib/Image/ExifTool/Exif.pm:6372-6398 value extraction
pub fn extract_long_value(data: &[u8], entry: &IfdEntry, byte_order: ByteOrder) -> Result<u32> {
    if entry.count != 1 {
        return Err(ExifError::ParseError(format!(
            "LONG value with count {} not supported yet",
            entry.count
        )));
    }

    if entry.is_inline() {
        // Value stored inline
        Ok(entry.value_or_offset)
    } else {
        // Value stored at offset
        let offset = entry.value_or_offset as usize;
        byte_order.read_u32(data, offset)
    }
}

/// Extract LONG (u32) array values from IFD entry
/// ExifTool: lib/Image/ExifTool/Exif.pm format 4 (int32u) with count > 1
pub fn extract_long_array(
    data: &[u8],
    entry: &IfdEntry,
    byte_order: ByteOrder,
) -> Result<Vec<u32>> {
    let mut values = Vec::with_capacity(entry.count as usize);
    let bytes_per_value = 4; // Each LONG is 4 bytes
    let total_bytes = entry.count as usize * bytes_per_value;

    debug!(
        "Extracting LONG array: count={}, total_bytes={}",
        entry.count, total_bytes
    );

    if entry.is_inline() && entry.count == 1 {
        // Single value stored inline
        values.push(entry.value_or_offset);
    } else {
        // Multiple values or single value stored at offset
        let offset = entry.value_or_offset as usize;
        if offset + total_bytes > data.len() {
            return Err(ExifError::ParseError(format!(
                "LONG array offset {offset:#x} + {total_bytes} bytes beyond data bounds"
            )));
        }

        for i in 0..entry.count {
            let value_offset = offset + (i as usize * bytes_per_value);
            let value = byte_order.read_u32(data, value_offset)?;
            values.push(value);
        }
    }

    debug!("Successfully extracted {} LONG values", values.len());
    Ok(values)
}

/// Extract RATIONAL (2x u32) value - numerator and denominator
/// ExifTool: lib/Image/ExifTool/Exif.pm format 5 (rational64u)
pub fn extract_rational_value(
    data: &[u8],
    entry: &IfdEntry,
    byte_order: ByteOrder,
) -> Result<TagValue> {
    if entry.count == 1 {
        // Single rational value
        if entry.is_inline() {
            // 8-byte rational cannot fit inline (4-byte field), so this should never happen
            return Err(ExifError::ParseError(
                "RATIONAL value cannot be stored inline".to_string(),
            ));
        }

        // Value stored at offset - read 2x uint32
        let offset = entry.value_or_offset as usize;
        if offset + 8 > data.len() {
            return Err(ExifError::ParseError(format!(
                "RATIONAL value offset {offset:#x} + 8 bytes beyond data bounds"
            )));
        }

        let numerator = byte_order.read_u32(data, offset)?;
        let denominator = byte_order.read_u32(data, offset + 4)?;
        Ok(TagValue::Rational(numerator, denominator))
    } else {
        // Multiple rational values - GPS coordinates use 3 rationals
        if entry.is_inline() {
            return Err(ExifError::ParseError(
                "RATIONAL array cannot be stored inline".to_string(),
            ));
        }

        let offset = entry.value_or_offset as usize;
        let total_size = entry.count as usize * 8; // 8 bytes per rational
        if offset + total_size > data.len() {
            return Err(ExifError::ParseError(format!(
                "RATIONAL array offset {offset:#x} + {total_size} bytes beyond data bounds"
            )));
        }

        let mut rationals = Vec::new();
        for i in 0..entry.count {
            let rat_offset = offset + (i as usize * 8);
            let numerator = byte_order.read_u32(data, rat_offset)?;
            let denominator = byte_order.read_u32(data, rat_offset + 4)?;
            rationals.push((numerator, denominator));
        }
        Ok(TagValue::RationalArray(rationals))
    }
}

/// Extract SRATIONAL (2x i32) value - signed numerator and denominator
/// ExifTool: lib/Image/ExifTool/Exif.pm format 10 (rational64s)
pub fn extract_srational_value(
    data: &[u8],
    entry: &IfdEntry,
    byte_order: ByteOrder,
) -> Result<TagValue> {
    if entry.count == 1 {
        // Single signed rational value
        if entry.is_inline() {
            return Err(ExifError::ParseError(
                "SRATIONAL value cannot be stored inline".to_string(),
            ));
        }

        let offset = entry.value_or_offset as usize;
        if offset + 8 > data.len() {
            return Err(ExifError::ParseError(format!(
                "SRATIONAL value offset {offset:#x} + 8 bytes beyond data bounds"
            )));
        }

        // Read as u32 first, then convert to i32 to handle signed values correctly
        let numerator_u32 = byte_order.read_u32(data, offset)?;
        let denominator_u32 = byte_order.read_u32(data, offset + 4)?;
        let numerator = numerator_u32 as i32;
        let denominator = denominator_u32 as i32;
        Ok(TagValue::SRational(numerator, denominator))
    } else {
        // Multiple signed rational values
        if entry.is_inline() {
            return Err(ExifError::ParseError(
                "SRATIONAL array cannot be stored inline".to_string(),
            ));
        }

        let offset = entry.value_or_offset as usize;
        let total_size = entry.count as usize * 8;
        if offset + total_size > data.len() {
            return Err(ExifError::ParseError(format!(
                "SRATIONAL array offset {offset:#x} + {total_size} bytes beyond data bounds"
            )));
        }

        let mut rationals = Vec::new();
        for i in 0..entry.count {
            let rat_offset = offset + (i as usize * 8);
            let numerator_u32 = byte_order.read_u32(data, rat_offset)?;
            let denominator_u32 = byte_order.read_u32(data, rat_offset + 4)?;
            let numerator = numerator_u32 as i32;
            let denominator = denominator_u32 as i32;
            rationals.push((numerator, denominator));
        }
        Ok(TagValue::SRationalArray(rationals))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tiff_types::TiffFormat;

    #[test]
    fn test_extract_ascii_inline() {
        let entry = IfdEntry {
            tag_id: 0x010f,
            format: TiffFormat::Ascii,
            count: 3,
            value_or_offset: u32::from_le_bytes([b'A', b'B', b'C', 0]), // "ABC" + null byte in little-endian
        };
        let data = &[];
        let result = extract_ascii_value(data, &entry, ByteOrder::LittleEndian, 0x010f).unwrap();
        assert_eq!(result, "ABC");
    }

    #[test]
    fn test_extract_short_inline() {
        let entry = IfdEntry {
            tag_id: 0x0100,
            format: TiffFormat::Short,
            count: 1,
            value_or_offset: 0x12340000, // 0x1234 in big-endian format, stored in first 2 bytes
        };
        let data = &[];
        let result = extract_short_value(data, &entry, ByteOrder::BigEndian).unwrap();
        assert_eq!(result, 0x1234);
    }

    #[test]
    fn test_extract_byte_inline() {
        let entry = IfdEntry {
            tag_id: 0x0101,
            format: TiffFormat::Byte,
            count: 1,
            value_or_offset: 0x42,
        };
        let data = &[];
        let result = extract_byte_value(data, &entry).unwrap();
        assert_eq!(result, 0x42);
    }

    #[test]
    fn test_extract_long_inline() {
        let entry = IfdEntry {
            tag_id: 0x0102,
            format: TiffFormat::Long,
            count: 1,
            value_or_offset: 0x12345678,
        };
        let data = &[];
        let result = extract_long_value(data, &entry, ByteOrder::BigEndian).unwrap();
        assert_eq!(result, 0x12345678);
    }

    #[test]
    fn test_extract_rational_at_offset() {
        let data = [0x00, 0x00, 0x01, 0x2c, 0x00, 0x00, 0x00, 0x01]; // 300/1 in big-endian
        let entry = IfdEntry {
            tag_id: 0x011a,
            format: TiffFormat::Rational,
            count: 1,
            value_or_offset: 0, // Data starts at offset 0
        };
        let result = extract_rational_value(&data, &entry, ByteOrder::BigEndian).unwrap();
        if let TagValue::Rational(num, den) = result {
            assert_eq!(num, 300);
            assert_eq!(den, 1);
        } else {
            panic!("Expected TagValue::Rational");
        }
    }

    #[test]
    fn test_extract_srational_at_offset() {
        let data = [0xff, 0xff, 0xff, 0x9c, 0x00, 0x00, 0x00, 0x01]; // -100/1 in big-endian
        let entry = IfdEntry {
            tag_id: 0x9201,
            format: TiffFormat::SRational,
            count: 1,
            value_or_offset: 0,
        };
        let result = extract_srational_value(&data, &entry, ByteOrder::BigEndian).unwrap();
        if let TagValue::SRational(num, den) = result {
            assert_eq!(num, -100);
            assert_eq!(den, 1);
        } else {
            panic!("Expected TagValue::SRational");
        }
    }
}
