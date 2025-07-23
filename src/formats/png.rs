//! PNG file format processing
//!
//! This module handles PNG (Portable Network Graphics) file processing,
//! extracting metadata from PNG chunks following ExifTool's implementation.
//!
//! Reference: third-party/exiftool/lib/Image/ExifTool/PNG.pm

use crate::types::{Result, TagEntry, TagValue};

/// PNG file signature: \x89PNG\r\n\x1a\n
const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";

/// PNG IHDR chunk data structure
/// ExifTool reference: PNG.pm:387-423 (ImageHeader table)
#[derive(Debug, Clone)]
pub struct IhdrData {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub color_type: u8,
    pub compression: u8,
    pub filter: u8,
    pub interlace: u8,
}

impl IhdrData {
    /// Convert ColorType value to human-readable string
    /// ExifTool reference: PNG.pm:403-409 (ColorType PrintConv)
    pub fn color_type_description(&self) -> &'static str {
        match self.color_type {
            0 => "Grayscale",
            2 => "RGB",
            3 => "Palette",
            4 => "Grayscale with Alpha",
            6 => "RGB with Alpha",
            _ => "Unknown",
        }
    }

    /// Convert Compression value to human-readable string
    /// ExifTool reference: PNG.pm:412-414 (Compression PrintConv)
    pub fn compression_description(&self) -> &'static str {
        match self.compression {
            0 => "Deflate/Inflate",
            _ => "Unknown",
        }
    }

    /// Convert Filter value to human-readable string
    /// ExifTool reference: PNG.pm:415-417 (Filter PrintConv)
    pub fn filter_description(&self) -> &'static str {
        match self.filter {
            0 => "Adaptive",
            _ => "Unknown",
        }
    }

    /// Convert Interlace value to human-readable string
    /// ExifTool reference: PNG.pm:419-421 (Interlace PrintConv)
    pub fn interlace_description(&self) -> &'static str {
        match self.interlace {
            0 => "Noninterlaced",
            1 => "Adam7 Interlace",
            _ => "Unknown",
        }
    }
}

/// Parse PNG IHDR chunk to extract image dimensions and properties
///
/// PNG file structure:
/// - PNG signature: 8 bytes (\x89PNG\r\n\x1a\n)
/// - IHDR chunk length: 4 bytes big-endian (always 13 for IHDR)
/// - IHDR chunk type: "IHDR" (4 bytes ASCII)  
/// - IHDR data: 13 bytes
///   - Width: 4 bytes big-endian (u32)
///   - Height: 4 bytes big-endian (u32)
///   - BitDepth: 1 byte
///   - ColorType: 1 byte
///   - Compression: 1 byte
///   - Filter: 1 byte
///   - Interlace: 1 byte
/// - CRC: 4 bytes big-endian
///
/// ExifTool reference: PNG.pm:387-423 (ImageHeader processing)
pub fn parse_png_ihdr(data: &[u8]) -> Result<IhdrData> {
    // Verify PNG signature
    if data.len() < PNG_SIGNATURE.len() || &data[..PNG_SIGNATURE.len()] != PNG_SIGNATURE {
        return Err(crate::types::ExifError::InvalidFormat(
            "Invalid PNG signature".to_string(),
        ));
    }

    let mut offset = PNG_SIGNATURE.len(); // Skip 8-byte PNG signature

    // Read IHDR chunk length (4 bytes big-endian)
    if data.len() < offset + 4 {
        return Err(crate::types::ExifError::InvalidFormat(
            "PNG file too short for IHDR length".to_string(),
        ));
    }
    let chunk_length = u32::from_be_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]);
    offset += 4;

    // Verify IHDR chunk length (must be 13)
    if chunk_length != 13 {
        return Err(crate::types::ExifError::InvalidFormat(format!(
            "Invalid IHDR chunk length: expected 13, got {}",
            chunk_length
        )));
    }

    // Read IHDR chunk type (4 bytes ASCII: "IHDR")
    if data.len() < offset + 4 {
        return Err(crate::types::ExifError::InvalidFormat(
            "PNG file too short for IHDR type".to_string(),
        ));
    }
    let chunk_type = &data[offset..offset + 4];
    if chunk_type != b"IHDR" {
        return Err(crate::types::ExifError::InvalidFormat(format!(
            "Expected IHDR chunk, got {}",
            String::from_utf8_lossy(chunk_type)
        )));
    }
    offset += 4;

    // Read IHDR data (13 bytes)
    if data.len() < offset + 13 {
        return Err(crate::types::ExifError::InvalidFormat(
            "PNG file too short for IHDR data".to_string(),
        ));
    }

    // Parse IHDR data following ExifTool's ImageHeader table structure
    // PNG.pm:391-423 - PROCESS_PROC => ProcessBinaryData with specific offsets
    let width = u32::from_be_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]);
    let height = u32::from_be_bytes([
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ]);
    let bit_depth = data[offset + 8];
    let color_type = data[offset + 9];
    let compression = data[offset + 10];
    let filter = data[offset + 11];
    let interlace = data[offset + 12];

    Ok(IhdrData {
        width,
        height,
        bit_depth,
        color_type,
        compression,
        filter,
        interlace,
    })
}

/// Create PNG TagEntry objects from IHDR data
///
/// Following ExifTool's PNG group assignment and tag naming conventions.
/// ExifTool assigns PNG metadata to "PNG" group (not "File" group like JPEG).
///
/// ExifTool reference: PNG.pm:100-102 (GROUPS => { 2 => 'Image' })
pub fn create_png_tag_entries(ihdr: &IhdrData) -> Vec<TagEntry> {
    vec![
        // PNG:ImageWidth - ExifTool PNG.pm:391-394
        TagEntry {
            group: "PNG".to_string(),
            group1: "PNG".to_string(),
            name: "ImageWidth".to_string(),
            value: TagValue::U32(ihdr.width),
            print: TagValue::U32(ihdr.width),
        },
        // PNG:ImageHeight - ExifTool PNG.pm:395-398
        TagEntry {
            group: "PNG".to_string(),
            group1: "PNG".to_string(),
            name: "ImageHeight".to_string(),
            value: TagValue::U32(ihdr.height),
            print: TagValue::U32(ihdr.height),
        },
        // PNG:BitDepth - ExifTool PNG.pm:399
        TagEntry {
            group: "PNG".to_string(),
            group1: "PNG".to_string(),
            name: "BitDepth".to_string(),
            value: TagValue::U8(ihdr.bit_depth),
            print: TagValue::U8(ihdr.bit_depth),
        },
        // PNG:ColorType - ExifTool PNG.pm:400-410 (with PrintConv)
        TagEntry {
            group: "PNG".to_string(),
            group1: "PNG".to_string(),
            name: "ColorType".to_string(),
            value: TagValue::String(ihdr.color_type.to_string()),
            print: TagValue::String(ihdr.color_type_description().to_string()),
        },
        // PNG:Compression - ExifTool PNG.pm:411-414 (with PrintConv)
        TagEntry {
            group: "PNG".to_string(),
            group1: "PNG".to_string(),
            name: "Compression".to_string(),
            value: TagValue::String(ihdr.compression.to_string()),
            print: TagValue::String(ihdr.compression_description().to_string()),
        },
        // PNG:Filter - ExifTool PNG.pm:415-418 (with PrintConv)
        TagEntry {
            group: "PNG".to_string(),
            group1: "PNG".to_string(),
            name: "Filter".to_string(),
            value: TagValue::String(ihdr.filter.to_string()),
            print: TagValue::String(ihdr.filter_description().to_string()),
        },
        // PNG:Interlace - ExifTool PNG.pm:419-422 (with PrintConv)
        TagEntry {
            group: "PNG".to_string(),
            group1: "PNG".to_string(),
            name: "Interlace".to_string(),
            value: TagValue::String(ihdr.interlace.to_string()),
            print: TagValue::String(ihdr.interlace_description().to_string()),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_png_signature_validation() {
        let valid_png = b"\x89PNG\r\n\x1a\n";
        assert_eq!(&valid_png[..], PNG_SIGNATURE);

        let invalid_png = b"\x89PNG\r\n\x1a\x00";
        assert_ne!(&invalid_png[..], PNG_SIGNATURE);
    }

    #[test]
    fn test_color_type_descriptions() {
        let ihdr = IhdrData {
            width: 100,
            height: 200,
            bit_depth: 8,
            color_type: 6, // RGB with Alpha
            compression: 0,
            filter: 0,
            interlace: 0,
        };

        assert_eq!(ihdr.color_type_description(), "RGB with Alpha");
    }

    #[test]
    fn test_ihdr_parsing_invalid_signature() {
        let invalid_data = b"\x00\x00\x00\x00";
        let result = parse_png_ihdr(invalid_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_png_tag_entries() {
        let ihdr = IhdrData {
            width: 1130,
            height: 726,
            bit_depth: 8,
            color_type: 6, // RGB with Alpha
            compression: 0,
            filter: 0,
            interlace: 0,
        };

        let entries = create_png_tag_entries(&ihdr);

        // Should have 7 entries: width, height, bit_depth, color_type, compression, filter, interlace
        assert_eq!(entries.len(), 7);

        // Check ImageWidth entry
        let width_entry = entries.iter().find(|e| e.name == "ImageWidth").unwrap();
        assert_eq!(width_entry.group, "PNG");
        assert_eq!(width_entry.value, TagValue::U32(1130));

        // Check ImageHeight entry
        let height_entry = entries.iter().find(|e| e.name == "ImageHeight").unwrap();
        assert_eq!(height_entry.group, "PNG");
        assert_eq!(height_entry.value, TagValue::U32(726));

        // Check ColorType entry (should have PrintConv applied)
        let color_type_entry = entries.iter().find(|e| e.name == "ColorType").unwrap();
        assert_eq!(
            color_type_entry.print,
            TagValue::String("RGB with Alpha".to_string())
        );
    }
}
