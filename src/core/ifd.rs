//! IFD (Image File Directory) parsing

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Exif.pm"]
#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool.pm"]

use crate::core::{Endian, ExifFormat, ExifValue};
use crate::error::{Error, Result};
use crate::maker;
use crate::tables::lookup_tag;
use std::collections::HashMap;

/// TIFF/EXIF header
#[derive(Debug, Clone, PartialEq)]
pub struct TiffHeader {
    pub byte_order: Endian,
    pub ifd0_offset: u32,
}

impl TiffHeader {
    /// Parse TIFF header from the beginning of EXIF data
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 8 {
            return Err(Error::InvalidExif("TIFF header too short".into()));
        }

        // Check byte order marker
        let byte_order = Endian::from_tiff_header(&data[0..2])
            .ok_or_else(|| Error::InvalidExif("Invalid byte order marker".into()))?;

        // Check magic number (42)
        let magic = byte_order.read_u16(&data[2..4]);
        if magic != 42 {
            return Err(Error::InvalidExif(format!("Invalid TIFF magic: {}", magic)));
        }

        // Read IFD0 offset
        let ifd0_offset = byte_order.read_u32(&data[4..8]);

        Ok(TiffHeader {
            byte_order,
            ifd0_offset,
        })
    }
}

/// A single IFD entry
#[derive(Debug, Clone)]
pub struct IfdEntry {
    pub tag: u16,
    pub format: ExifFormat,
    pub count: u32,
    pub value_offset: u32,
    pub value_data: Vec<u8>,
}

/// Parsed IFD data
#[derive(Debug)]
pub struct ParsedIfd {
    entries: HashMap<u16, ExifValue>,
}

impl ParsedIfd {
    /// Expose the raw entries for advanced usage
    pub fn entries(&self) -> &HashMap<u16, ExifValue> {
        &self.entries
    }

    pub fn get_string(&self, tag: u16) -> Result<Option<String>> {
        match self.entries.get(&tag) {
            Some(ExifValue::Ascii(s)) => Ok(Some(s.clone())),
            Some(_) => Err(Error::InvalidFormat {
                expected: "ASCII string".into(),
                actual: "other format".into(),
            }),
            None => Ok(None),
        }
    }

    pub fn get_u16(&self, tag: u16) -> Result<Option<u16>> {
        match self.entries.get(&tag) {
            Some(ExifValue::U16(v)) => Ok(Some(*v)),
            Some(ExifValue::U16Array(v)) if !v.is_empty() => Ok(Some(v[0])),
            Some(_) => Err(Error::InvalidFormat {
                expected: "U16".into(),
                actual: "other format".into(),
            }),
            None => Ok(None),
        }
    }

    pub fn get_u32(&self, tag: u16) -> Result<Option<u32>> {
        match self.entries.get(&tag) {
            Some(ExifValue::U32(v)) => Ok(Some(*v)),
            Some(ExifValue::U32Array(v)) if !v.is_empty() => Ok(Some(v[0])),
            Some(_) => Err(Error::InvalidFormat {
                expected: "U32".into(),
                actual: "other format".into(),
            }),
            None => Ok(None),
        }
    }

    pub fn get_rational(&self, tag: u16) -> Result<Option<(u32, u32)>> {
        match self.entries.get(&tag) {
            Some(ExifValue::Rational(num, den)) => Ok(Some((*num, *den))),
            Some(ExifValue::RationalArray(v)) if !v.is_empty() => Ok(Some(v[0])),
            Some(_) => Err(Error::InvalidFormat {
                expected: "Rational".into(),
                actual: "other format".into(),
            }),
            None => Ok(None),
        }
    }

    pub fn get_signed_rational(&self, tag: u16) -> Result<Option<(i32, i32)>> {
        match self.entries.get(&tag) {
            Some(ExifValue::SignedRational(num, den)) => Ok(Some((*num, *den))),
            Some(ExifValue::SignedRationalArray(v)) if !v.is_empty() => Ok(Some(v[0])),
            Some(_) => Err(Error::InvalidFormat {
                expected: "SignedRational".into(),
                actual: "other format".into(),
            }),
            None => Ok(None),
        }
    }

    /// Get a value from IFD1 (thumbnail directory) by adding the IFD1 prefix
    pub fn get_ifd1_u32(&self, tag: u16) -> Result<Option<u32>> {
        let prefixed_tag = 0x1000 + tag;
        self.get_u32(prefixed_tag)
    }

    /// Get thumbnail offset from IFD1
    pub fn get_thumbnail_offset(&self) -> Result<Option<u32>> {
        self.get_ifd1_u32(0x201) // ThumbnailOffset
    }

    /// Get thumbnail length from IFD1
    pub fn get_thumbnail_length(&self) -> Result<Option<u32>> {
        self.get_ifd1_u32(0x202) // ThumbnailLength
    }

    /// Get raw binary data for any tag stored as Undefined
    pub fn get_binary_data(&self, tag: u16) -> Option<&[u8]> {
        match self.entries.get(&tag) {
            Some(ExifValue::Undefined(data)) => Some(data),
            _ => None,
        }
    }

    /// Get a numeric value as u32, trying different formats
    pub fn get_numeric_u32(&self, tag: u16) -> Option<u32> {
        match self.entries.get(&tag) {
            Some(ExifValue::U32(v)) => Some(*v),
            Some(ExifValue::U32Array(v)) if !v.is_empty() => Some(v[0]),
            Some(ExifValue::U16(v)) => Some(*v as u32),
            Some(ExifValue::U16Array(v)) if !v.is_empty() => Some(v[0] as u32),
            Some(ExifValue::U8(v)) => Some(*v as u32),
            Some(ExifValue::U8Array(v)) if !v.is_empty() => Some(v[0] as u32),
            Some(ExifValue::Undefined(bytes)) if bytes.len() >= 4 => {
                // Try to parse as little-endian U32 from raw bytes (most common for Canon)
                // TODO: Should use the actual header byte order here
                Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
            }
            _ => None,
        }
    }
}

/// IFD parser
pub struct IfdParser;

impl IfdParser {
    /// Parse EXIF data starting from TIFF header
    pub fn parse(data: Vec<u8>) -> Result<ParsedIfd> {
        let header = TiffHeader::parse(&data)?;
        let ifd_offset = header.ifd0_offset as usize;

        if ifd_offset >= data.len() {
            return Err(Error::InvalidExif("IFD0 offset out of bounds".into()));
        }

        // Parse IFD0 first
        let mut ifd0 = Self::parse_ifd(&data, &header, ifd_offset)?;

        // Get the offset to the next IFD (IFD1 - thumbnails) before parsing sub-IFDs
        let ifd1_offset = Self::get_next_ifd_offset(&data, &header, ifd_offset)?;

        // Get Make tag from IFD0 for use in sub-IFDs
        let make = ifd0.entries.get(&0x10f).and_then(|v| match v {
            ExifValue::Ascii(s) => Some(s.as_str()),
            _ => None,
        });

        // Check for ExifIFD (tag 0x8769) and parse it as well
        if let Some(ExifValue::U32(exif_ifd_offset)) = ifd0.entries.get(&0x8769) {
            let exif_ifd_offset = *exif_ifd_offset as usize;
            if exif_ifd_offset < data.len() {
                match Self::parse_ifd_with_context(&data, &header, exif_ifd_offset, make) {
                    Ok(exif_ifd) => {
                        // Merge ExifIFD entries into IFD0
                        for (tag, value) in exif_ifd.entries {
                            ifd0.entries.insert(tag, value);
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to parse ExifIFD: {}", e);
                    }
                }
            }
        }

        // Parse IFD1 (thumbnail directory) if it exists
        if let Some(ifd1_offset) = ifd1_offset {
            match Self::parse_ifd(&data, &header, ifd1_offset) {
                Ok(ifd1) => {
                    // Merge IFD1 entries with IFD1_ prefix to avoid conflicts
                    for (tag, value) in ifd1.entries {
                        // Use 0x1000 prefix for IFD1 tags to distinguish from IFD0
                        let prefixed_tag = 0x1000 + tag;
                        ifd0.entries.insert(prefixed_tag, value);
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to parse IFD1 (thumbnails): {}", e);
                }
            }
        }

        Ok(ifd0)
    }

    /// Get the offset to the next IFD from the current IFD
    fn get_next_ifd_offset(
        data: &[u8],
        header: &TiffHeader,
        ifd_offset: usize,
    ) -> Result<Option<usize>> {
        // Check if we have enough data for entry count
        if ifd_offset + 2 > data.len() {
            return Ok(None);
        }

        // Read number of entries
        let entry_count = header
            .byte_order
            .read_u16(&data[ifd_offset..ifd_offset + 2]) as usize;

        // Calculate offset to next IFD pointer (after all entries)
        let next_ifd_ptr_offset = ifd_offset + 2 + (entry_count * 12);

        // Check if we have enough data for the next IFD offset
        if next_ifd_ptr_offset + 4 > data.len() {
            return Ok(None);
        }

        // Read next IFD offset
        let next_ifd_offset = header
            .byte_order
            .read_u32(&data[next_ifd_ptr_offset..next_ifd_ptr_offset + 4]);

        // A value of 0 means no next IFD
        if next_ifd_offset == 0 {
            Ok(None)
        } else {
            let offset = next_ifd_offset as usize;
            // Validate the offset is within bounds
            if offset < data.len() {
                Ok(Some(offset))
            } else {
                Ok(None)
            }
        }
    }

    /// Parse a single IFD
    pub fn parse_ifd(data: &[u8], header: &TiffHeader, offset: usize) -> Result<ParsedIfd> {
        Self::parse_ifd_with_context(data, header, offset, None)
    }

    /// Parse a single IFD with optional context (e.g., Make from parent IFD)
    fn parse_ifd_with_context(
        data: &[u8],
        header: &TiffHeader,
        offset: usize,
        make: Option<&str>,
    ) -> Result<ParsedIfd> {
        let mut entries = HashMap::new();

        // Check if we have enough data for entry count
        if offset + 2 > data.len() {
            return Err(Error::InvalidExif("IFD entry count out of bounds".into()));
        }

        // Read number of entries
        let entry_count = header.byte_order.read_u16(&data[offset..offset + 2]) as usize;
        let mut pos = offset + 2;

        // Parse each entry (12 bytes each)
        for _ in 0..entry_count {
            if pos + 12 > data.len() {
                return Err(Error::InvalidExif("IFD entry out of bounds".into()));
            }

            let entry_data = &data[pos..pos + 12];
            let tag = header.byte_order.read_u16(&entry_data[0..2]);
            let format_code = header.byte_order.read_u16(&entry_data[2..4]);
            let count = header.byte_order.read_u32(&entry_data[4..8]);
            let value_offset = header.byte_order.read_u32(&entry_data[8..12]);

            // Parse format
            let format = ExifFormat::from_u16(format_code)
                .ok_or_else(|| Error::InvalidExif(format!("Unknown format: {}", format_code)))?;

            // Calculate total size
            let value_size = format.size() * count as usize;

            // Get value data
            let value_data = if value_size <= 4 {
                // Value fits in the offset field
                entry_data[8..12].to_vec()
            } else {
                // Value is stored at offset
                let value_pos = value_offset as usize;
                if value_pos + value_size > data.len() {
                    // Skip this entry if value is out of bounds
                    pos += 12;
                    continue;
                }
                data[value_pos..value_pos + value_size].to_vec()
            };

            // Special handling for maker notes (tag 0x927c)
            if tag == 0x927c {
                // Check if we have a Make tag to determine manufacturer
                // First check in current entries, then use context if provided
                let make_str = entries
                    .get(&0x10f)
                    .and_then(|v| match v {
                        ExifValue::Ascii(s) => Some(s.as_str()),
                        _ => None,
                    })
                    .or(make);

                if let Some(make_str) = make_str {
                    // Parse maker notes with manufacturer-specific parser
                    match maker::parse_maker_notes(&value_data, make_str, header.byte_order, 0) {
                        Ok(maker_entries) => {
                            // Add maker note entries with a tag prefix to avoid conflicts
                            // Use manufacturer-specific prefixes
                            let manufacturer = maker::Manufacturer::from_make(make_str);
                            let prefix = match manufacturer {
                                maker::Manufacturer::Canon => 0xC000,
                                maker::Manufacturer::Nikon => 0x4E00,
                                maker::Manufacturer::Sony => 0x534F,
                                maker::Manufacturer::Olympus => 0x4F4C,
                                maker::Manufacturer::Pentax => 0x5045,
                                maker::Manufacturer::Fujifilm => 0x4655,
                                maker::Manufacturer::Panasonic => 0x5041,
                                _ => 0xC000, // Default fallback
                            };

                            for (maker_tag, maker_value) in maker_entries {
                                // Check if this is a converted tag (0x8000 bit set)
                                if maker_tag >= 0x8000 {
                                    // This is a converted tag - store it directly without prefix
                                    // Converted tags are already properly namespaced by the maker parser
                                    entries.insert(maker_tag, maker_value);
                                } else if manufacturer == maker::Manufacturer::Canon
                                    && (0x4000..0x8000).contains(&maker_tag)
                                {
                                    // Canon tags in the 0x4xxx range are valid Canon-specific tags
                                    // Store them directly without prefix since they won't overflow
                                    // and are already in a Canon-specific range
                                    entries.insert(maker_tag, maker_value);
                                } else if maker_tag < 0x4000 {
                                    // Normal maker tag - add manufacturer prefix to avoid conflicts
                                    let prefixed_tag = prefix + maker_tag;
                                    entries.insert(prefixed_tag, maker_value);
                                } else {
                                    // Tag is in the high range but not converted - skip to avoid overflow
                                    eprintln!(
                                        "Warning: Skipping maker tag 0x{:04X} to avoid overflow",
                                        maker_tag
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to parse maker notes: {}", e);
                            // Store raw maker note data as fallback
                            entries.insert(tag, ExifValue::Undefined(value_data.clone()));
                        }
                    }
                } else {
                    // No Make tag found, store as raw data
                    entries.insert(tag, ExifValue::Undefined(value_data.clone()));
                }

                pos += 12;
                continue;
            }

            // Special handling for structural tags that should be U32 regardless of lookup table
            let actual_format = match tag {
                0x8769 => ExifFormat::U32, // ExifOffset
                0x8825 => ExifFormat::U32, // GPSOffset
                0x014A => ExifFormat::U32, // SubIFDs
                _ => {
                    // Look up tag definition from generated tables
                    if let Some(tag_info) = lookup_tag(tag) {
                        tag_info.format
                    } else {
                        // Unknown tag - use the format from the file
                        format
                    }
                }
            };

            if lookup_tag(tag).is_some() || matches!(tag, 0x8769 | 0x8825 | 0x014A) {
                // Parse value based on actual format
                match actual_format {
                    ExifFormat::Ascii => {
                        if let Ok(s) = Self::parse_ascii(&value_data) {
                            entries.insert(tag, ExifValue::Ascii(s));
                        }
                    }
                    ExifFormat::U8 => {
                        if count == 1 && !value_data.is_empty() {
                            entries.insert(tag, ExifValue::U8(value_data[0]));
                        } else if !value_data.is_empty() {
                            entries.insert(tag, ExifValue::U8Array(value_data.clone()));
                        }
                    }
                    ExifFormat::U16 => {
                        if count == 1 && value_data.len() >= 2 {
                            let val = header.byte_order.read_u16(&value_data[0..2]);
                            entries.insert(tag, ExifValue::U16(val));
                        } else if value_data.len() >= 2 * count as usize {
                            let mut values = Vec::with_capacity(count as usize);
                            for i in 0..count as usize {
                                let offset = i * 2;
                                values.push(
                                    header.byte_order.read_u16(&value_data[offset..offset + 2]),
                                );
                            }
                            entries.insert(tag, ExifValue::U16Array(values));
                        }
                    }
                    ExifFormat::U32 => {
                        if count == 1 && value_data.len() >= 4 {
                            let val = header.byte_order.read_u32(&value_data[0..4]);
                            entries.insert(tag, ExifValue::U32(val));
                        } else if value_data.len() >= 4 * count as usize {
                            let mut values = Vec::with_capacity(count as usize);
                            for i in 0..count as usize {
                                let offset = i * 4;
                                values.push(
                                    header.byte_order.read_u32(&value_data[offset..offset + 4]),
                                );
                            }
                            entries.insert(tag, ExifValue::U32Array(values));
                        }
                    }
                    ExifFormat::I16 => {
                        if count == 1 && value_data.len() >= 2 {
                            let val = header.byte_order.read_i16(&value_data[0..2]);
                            entries.insert(tag, ExifValue::I16(val));
                        } else if value_data.len() >= 2 * count as usize {
                            let mut values = Vec::with_capacity(count as usize);
                            for i in 0..count as usize {
                                let offset = i * 2;
                                values.push(
                                    header.byte_order.read_i16(&value_data[offset..offset + 2]),
                                );
                            }
                            entries.insert(tag, ExifValue::I16Array(values));
                        }
                    }
                    ExifFormat::I32 => {
                        if count == 1 && value_data.len() >= 4 {
                            let val = header.byte_order.read_i32(&value_data[0..4]);
                            entries.insert(tag, ExifValue::I32(val));
                        } else if value_data.len() >= 4 * count as usize {
                            let mut values = Vec::with_capacity(count as usize);
                            for i in 0..count as usize {
                                let offset = i * 4;
                                values.push(
                                    header.byte_order.read_i32(&value_data[offset..offset + 4]),
                                );
                            }
                            entries.insert(tag, ExifValue::I32Array(values));
                        }
                    }
                    ExifFormat::Rational => {
                        if count == 1 && value_data.len() >= 8 {
                            let num = header.byte_order.read_u32(&value_data[0..4]);
                            let den = header.byte_order.read_u32(&value_data[4..8]);
                            entries.insert(tag, ExifValue::Rational(num, den));
                        } else if value_data.len() >= 8 * count as usize {
                            let mut values = Vec::with_capacity(count as usize);
                            for i in 0..count as usize {
                                let offset = i * 8;
                                let num =
                                    header.byte_order.read_u32(&value_data[offset..offset + 4]);
                                let den = header
                                    .byte_order
                                    .read_u32(&value_data[offset + 4..offset + 8]);
                                values.push((num, den));
                            }
                            entries.insert(tag, ExifValue::RationalArray(values));
                        }
                    }
                    ExifFormat::SignedRational => {
                        if count == 1 && value_data.len() >= 8 {
                            let num = header.byte_order.read_i32(&value_data[0..4]);
                            let den = header.byte_order.read_i32(&value_data[4..8]);
                            entries.insert(tag, ExifValue::SignedRational(num, den));
                        } else if value_data.len() >= 8 * count as usize {
                            let mut values = Vec::with_capacity(count as usize);
                            for i in 0..count as usize {
                                let offset = i * 8;
                                let num =
                                    header.byte_order.read_i32(&value_data[offset..offset + 4]);
                                let den = header
                                    .byte_order
                                    .read_i32(&value_data[offset + 4..offset + 8]);
                                values.push((num, den));
                            }
                            entries.insert(tag, ExifValue::SignedRationalArray(values));
                        }
                    }
                    _ => {
                        // I8, F32, F64, Undefined - store as raw bytes for now
                        entries.insert(tag, ExifValue::Undefined(value_data));
                    }
                }
            } else {
                // Unknown tag - store as undefined
                entries.insert(tag, ExifValue::Undefined(value_data));
            }
            pos += 12;
        }

        Ok(ParsedIfd { entries })
    }

    /// Parse ASCII string from bytes
    fn parse_ascii(data: &[u8]) -> Result<String> {
        // Find null terminator or use whole buffer
        let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
        let s = std::str::from_utf8(&data[..end])
            .map_err(|_| Error::InvalidExif("Invalid ASCII string".into()))?;
        Ok(s.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tiff_header_little_endian() {
        let data = b"II\x2A\x00\x08\x00\x00\x00";
        let header = TiffHeader::parse(data).unwrap();
        assert_eq!(header.byte_order, Endian::Little);
        assert_eq!(header.ifd0_offset, 8);
    }

    #[test]
    fn test_tiff_header_big_endian() {
        let data = b"MM\x00\x2A\x00\x00\x00\x08";
        let header = TiffHeader::parse(data).unwrap();
        assert_eq!(header.byte_order, Endian::Big);
        assert_eq!(header.ifd0_offset, 8);
    }

    #[test]
    fn test_invalid_byte_order() {
        let data = b"XX\x2A\x00\x08\x00\x00\x00";
        assert!(TiffHeader::parse(data).is_err());
    }

    #[test]
    fn test_invalid_magic() {
        let data = b"II\x00\x00\x08\x00\x00\x00";
        assert!(TiffHeader::parse(data).is_err());
    }

    #[test]
    fn test_parse_simple_ifd() {
        // Create a minimal EXIF data with Make tag
        let mut data = Vec::new();

        // TIFF header (little-endian)
        data.extend_from_slice(b"II\x2A\x00\x08\x00\x00\x00");

        // IFD0: 1 entry
        data.extend_from_slice(&[0x01, 0x00]); // Entry count = 1

        // Entry for Make (0x010F)
        data.extend_from_slice(&[0x0F, 0x01]); // Tag = 0x010F
        data.extend_from_slice(&[0x02, 0x00]); // Format = 2 (ASCII)
        data.extend_from_slice(&[0x04, 0x00, 0x00, 0x00]); // Count = 4 (including null)
        data.extend_from_slice(b"Tst\0"); // Value (exactly 4 bytes)

        // Next IFD offset
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        let ifd = IfdParser::parse(data).unwrap();
        assert_eq!(ifd.get_string(0x010F).unwrap(), Some("Tst".to_string()));
    }

    #[test]
    fn test_parse_rational() {
        // Create EXIF data with XResolution tag (rational type)
        let mut data = Vec::new();

        // TIFF header (little-endian)
        data.extend_from_slice(b"II\x2A\x00\x08\x00\x00\x00");

        // IFD0: 1 entry
        data.extend_from_slice(&[0x01, 0x00]); // Entry count = 1

        // Entry for XResolution (0x011A)
        data.extend_from_slice(&[0x1A, 0x01]); // Tag = 0x011A
        data.extend_from_slice(&[0x05, 0x00]); // Format = 5 (Rational)
        data.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // Count = 1
        data.extend_from_slice(&[0x1A, 0x00, 0x00, 0x00]); // Offset to value

        // Next IFD offset
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        // Rational value at offset 0x1A: 72/1 (72 DPI)
        data.extend_from_slice(&[0x48, 0x00, 0x00, 0x00]); // Numerator = 72
        data.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // Denominator = 1

        let ifd = IfdParser::parse(data).unwrap();
        match ifd.entries.get(&0x011A) {
            Some(ExifValue::Rational(num, den)) => {
                assert_eq!(*num, 72);
                assert_eq!(*den, 1);
            }
            _ => panic!("Expected Rational value"),
        }
    }

    #[test]
    fn test_parse_u16_array() {
        // Create EXIF data with BitsPerSample (U16 array)
        let mut data = Vec::new();

        // TIFF header (big-endian this time)
        data.extend_from_slice(b"MM\x00\x2A\x00\x00\x00\x08");

        // IFD0: 1 entry
        data.extend_from_slice(&[0x00, 0x01]); // Entry count = 1

        // Entry for BitsPerSample (0x0102)
        data.extend_from_slice(&[0x01, 0x02]); // Tag = 0x0102
        data.extend_from_slice(&[0x00, 0x03]); // Format = 3 (U16)
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x03]); // Count = 3
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x1A]); // Offset to value

        // Next IFD offset
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        // U16 array at offset 0x1A: [8, 8, 8]
        data.extend_from_slice(&[0x00, 0x08]); // 8
        data.extend_from_slice(&[0x00, 0x08]); // 8
        data.extend_from_slice(&[0x00, 0x08]); // 8

        let ifd = IfdParser::parse(data).unwrap();
        match ifd.entries.get(&0x0102) {
            Some(ExifValue::U16Array(values)) => {
                assert_eq!(values, &vec![8, 8, 8]);
            }
            _ => panic!("Expected U16Array value"),
        }
    }

    #[test]
    fn test_inline_value() {
        // Test that values <= 4 bytes are stored inline
        let mut data = Vec::new();

        // TIFF header (little-endian)
        data.extend_from_slice(b"II\x2A\x00\x08\x00\x00\x00");

        // IFD0: 1 entry
        data.extend_from_slice(&[0x01, 0x00]); // Entry count = 1

        // Entry for Orientation (0x0112)
        data.extend_from_slice(&[0x12, 0x01]); // Tag = 0x0112
        data.extend_from_slice(&[0x03, 0x00]); // Format = 3 (U16)
        data.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // Count = 1
        data.extend_from_slice(&[0x06, 0x00, 0x00, 0x00]); // Value = 6 (inline)

        // Next IFD offset
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        let ifd = IfdParser::parse(data).unwrap();
        assert_eq!(ifd.get_u16(0x0112).unwrap(), Some(6));
    }

    #[test]
    fn test_signed_rational() {
        // Create EXIF data with ExposureCompensation (signed rational)
        let mut data = Vec::new();

        // TIFF header (little-endian)
        data.extend_from_slice(b"II\x2A\x00\x08\x00\x00\x00");

        // IFD0: 1 entry
        data.extend_from_slice(&[0x01, 0x00]); // Entry count = 1

        // Entry for ExposureCompensation (0x9204)
        data.extend_from_slice(&[0x04, 0x92]); // Tag = 0x9204
        data.extend_from_slice(&[0x0A, 0x00]); // Format = 10 (SignedRational)
        data.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // Count = 1
        data.extend_from_slice(&[0x1A, 0x00, 0x00, 0x00]); // Offset to value

        // Next IFD offset
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        // Signed rational at offset 0x1A: -2/3 EV
        data.extend_from_slice(&[0xFE, 0xFF, 0xFF, 0xFF]); // Numerator = -2
        data.extend_from_slice(&[0x03, 0x00, 0x00, 0x00]); // Denominator = 3

        let ifd = IfdParser::parse(data).unwrap();
        match ifd.entries.get(&0x9204) {
            Some(ExifValue::SignedRational(num, den)) => {
                assert_eq!(*num, -2);
                assert_eq!(*den, 3);
            }
            _ => panic!("Expected SignedRational value"),
        }
    }

    #[test]
    fn test_unknown_tag() {
        // Test that unknown tags are stored as Undefined
        let mut data = Vec::new();

        // TIFF header (little-endian)
        data.extend_from_slice(b"II\x2A\x00\x08\x00\x00\x00");

        // IFD0: 1 entry
        data.extend_from_slice(&[0x01, 0x00]); // Entry count = 1

        // Entry for unknown tag (0xDEAD)
        data.extend_from_slice(&[0xAD, 0xDE]); // Tag = 0xDEAD
        data.extend_from_slice(&[0x07, 0x00]); // Format = 7 (Undefined)
        data.extend_from_slice(&[0x04, 0x00, 0x00, 0x00]); // Count = 4
        data.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]); // Value (inline)

        // Next IFD offset
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        let ifd = IfdParser::parse(data).unwrap();
        match ifd.entries.get(&0xDEAD) {
            Some(ExifValue::Undefined(data)) => {
                assert_eq!(data, &vec![0xDE, 0xAD, 0xBE, 0xEF]);
            }
            _ => panic!("Expected Undefined value for unknown tag"),
        }
    }
}
