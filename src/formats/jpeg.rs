//! JPEG-specific processing and segment scanning
//!
//! This module implements JPEG segment scanning to locate EXIF data,
//! following ExifTool's JPEG.pm implementation for segment parsing
//! and EXIF data extraction.

use crate::hash::ImageDataHasher;
use crate::types::{ExifError, Result};
use std::collections::{BTreeMap, HashMap};
use std::io::{Read, Seek, SeekFrom};

/// JPEG segment types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JpegSegment {
    /// Start of Image (0xD8)
    Soi,
    /// Application segments 0-15 (APP0-APP15)
    App(u8),
    /// Start of Frame segments (0xC0-0xCF except 0xC4, 0xC8, 0xCC)
    /// Contains the SOF marker value (0xC0-0xCF)
    Sof(u8),
    /// Define Huffman Table (0xC4)
    Dht,
    /// Start of Scan (0xDA)
    Sos,
    /// End of Image (0xD9)
    Eoi,
    /// Other segments
    Other(u8),
}

impl JpegSegment {
    fn from_marker(marker: u8) -> Self {
        match marker {
            0xD8 => Self::Soi,
            0xE0..=0xEF => Self::App(marker - 0xE0),
            // SOF markers: 0xC0-0xCF except 0xC4 (DHT), 0xC8 (JPGA), 0xCC (DAC)
            // ExifTool: lib/Image/ExifTool.pm:7317-7319
            0xC0..=0xCF => {
                if marker == 0xC4 {
                    Self::Dht
                } else if marker == 0xC8 || marker == 0xCC {
                    Self::Other(marker) // JPGA and DAC are not SOF
                } else {
                    Self::Sof(marker)
                }
            }
            0xDA => Self::Sos,
            0xD9 => Self::Eoi,
            _ => Self::Other(marker),
        }
    }

    /// Check if this is an APP1 segment (contains EXIF)
    #[allow(dead_code)]
    fn is_app1(&self) -> bool {
        matches!(self, Self::App(1))
    }

    /// Get the marker byte for this segment
    #[allow(dead_code)]
    fn marker_byte(&self) -> u8 {
        match self {
            Self::Soi => 0xD8,
            Self::App(app_num) => 0xE0 + app_num,
            Self::Sof(marker) => *marker,
            Self::Dht => 0xC4,
            Self::Sos => 0xDA,
            Self::Eoi => 0xD9,
            Self::Other(marker) => *marker,
        }
    }
}

/// JPEG segment scanner result
#[derive(Debug)]
pub struct JpegSegmentInfo {
    pub segment_type: JpegSegment,
    pub offset: u64,
    pub length: u16,
    pub has_exif: bool,
    pub has_xmp: bool,
}

/// SOF (Start of Frame) data extracted from JPEG
/// ExifTool: lib/Image/ExifTool.pm:7321-7336
#[derive(Debug)]
pub struct SofData {
    pub encoding_process: u8,              // marker - 0xc0
    pub bits_per_sample: u8,               // precision from SOF
    pub image_height: u16,                 // height from SOF
    pub image_width: u16,                  // width from SOF
    pub color_components: u8,              // number of color components
    pub ycbcr_subsampling: Option<String>, // calculated from component data
}

/// Extended XMP segment data
///
/// Extended XMP is split across multiple APP1 segments, each containing:
/// - 35 bytes: signature "http://ns.adobe.com/xmp/extension/\0"
/// - 32 bytes: GUID (MD5 hash of full extended XMP data)
/// - 4 bytes: total size of extended XMP data
/// - 4 bytes: offset for this XMP data portion
/// - Remaining: XMP data chunk
///
/// ExifTool: lib/Image/ExifTool.pm:7731-7754 (Extended XMP parsing)
#[derive(Debug)]
pub struct ExtendedXmpInfo {
    pub guid: String,
    pub total_size: u32,
    pub chunk_offset: u32,
    pub segment_offset: u64, // File offset to start of XMP data chunk
    pub chunk_length: u16,   // Length of this chunk
}

/// Parse SOF (Start of Frame) segment data
/// ExifTool: lib/Image/ExifTool.pm:7321-7336
fn parse_sof_data(marker: u8, segment_data: &[u8]) -> Result<SofData> {
    // Minimum SOF data is 6 bytes (precision, height, width, components)
    if segment_data.len() < 6 {
        return Err(ExifError::ParseError("SOF segment too short".to_string()));
    }

    // ExifTool: my ($p, $h, $w, $n) = unpack('Cn2C', $$segDataPt);
    let bits_per_sample = segment_data[0]; // 'C' = unsigned char
    let image_height = u16::from_be_bytes([segment_data[1], segment_data[2]]); // 'n' = big-endian u16
    let image_width = u16::from_be_bytes([segment_data[3], segment_data[4]]); // 'n' = big-endian u16
    let color_components = segment_data[5]; // 'C' = unsigned char

    // Calculate EncodingProcess (marker - 0xc0)
    // ExifTool: lib/Image/ExifTool.pm:7322
    let encoding_process = marker.wrapping_sub(0xc0);

    // Calculate YCbCrSubSampling if we have 3 components and enough data
    // ExifTool: lib/Image/ExifTool.pm:7327-7336
    let ycbcr_subsampling = if color_components == 3 && segment_data.len() >= 6 + 3 * 3 {
        let mut hmin = 255u8;
        let mut hmax = 0u8;
        let mut vmin = 255u8;
        let mut vmax = 0u8;

        // Process each component starting at offset 6
        for i in 0..3 {
            let component_offset = 6 + i * 3 + 1; // Skip component ID byte
            if component_offset < segment_data.len() {
                let sampling = segment_data[component_offset];
                let h = sampling >> 4;
                let v = sampling & 0x0f;

                if h < hmin {
                    hmin = h;
                }
                if h > hmax {
                    hmax = h;
                }
                if v < vmin {
                    vmin = v;
                }
                if v > vmax {
                    vmax = v;
                }
            }
        }

        // Calculate subsampling as per ExifTool
        if hmin > 0 && vmin > 0 {
            let hs = hmax / hmin;
            let vs = vmax / vmin;
            Some(format!("{hs} {vs}"))
        } else {
            None
        }
    } else {
        None
    };

    Ok(SofData {
        encoding_process,
        bits_per_sample,
        image_height,
        image_width,
        color_components,
        ycbcr_subsampling,
    })
}

/// Scan JPEG file for all APP1 segments containing EXIF or XMP data,
/// and also extract image dimensions from SOF markers
///
/// Returns information about the first APP1 segment found, prioritizing EXIF over XMP,
/// and optionally SOF data if found.
pub fn scan_jpeg_segments<R: Read + Seek>(
    mut reader: R,
) -> Result<(Option<JpegSegmentInfo>, Option<SofData>)> {
    // Verify JPEG magic bytes
    let mut magic = [0u8; 2];
    reader.read_exact(&mut magic)?;
    if magic != [0xFF, 0xD8] {
        return Err(ExifError::InvalidFormat(
            "Not a valid JPEG file (missing 0xFFD8 magic bytes)".to_string(),
        ));
    }

    let mut current_pos = 2u64; // After SOI marker
    let mut found_exif: Option<JpegSegmentInfo> = None;
    let mut found_xmp: Option<JpegSegmentInfo> = None;
    let mut sof_data: Option<SofData> = None;

    loop {
        // Read segment marker
        let mut marker_bytes = [0u8; 2];
        if reader.read_exact(&mut marker_bytes).is_err() {
            // End of file reached
            break;
        }

        if marker_bytes[0] != 0xFF {
            return Err(ExifError::ParseError(
                "Invalid JPEG segment marker".to_string(),
            ));
        }

        let segment = JpegSegment::from_marker(marker_bytes[1]);
        current_pos += 2;

        match segment {
            JpegSegment::Soi => {
                // Already processed
                continue;
            }
            JpegSegment::Eoi => {
                // End of image
                break;
            }
            JpegSegment::Sos => {
                // Start of scan - no more metadata segments
                break;
            }
            JpegSegment::App(app_num) => {
                // Read segment length
                let mut length_bytes = [0u8; 2];
                reader.read_exact(&mut length_bytes)?;
                let length = u16::from_be_bytes(length_bytes);
                current_pos += 2;

                if app_num == 1 {
                    // APP1 segment - check for EXIF or XMP
                    let segment_start = current_pos; // Start of segment data

                    // Try EXIF first (6 bytes: "Exif\0\0")
                    let mut exif_header = [0u8; 6];
                    if reader.read_exact(&mut exif_header).is_ok()
                        && &exif_header[0..4] == b"Exif"
                        && exif_header[4] == 0
                        && exif_header[5] == 0
                    {
                        // Found EXIF - store it and continue scanning
                        found_exif = Some(JpegSegmentInfo {
                            segment_type: segment,
                            offset: current_pos + 6, // After "Exif\0\0" (6 bytes)
                            length: length - 8, // Subtract segment length header (2 bytes) + "Exif\0\0" (6 bytes) = 8 total
                            has_exif: true,
                            has_xmp: false,
                        });
                    } else {
                        // Reset and try XMP (29 bytes: "http://ns.adobe.com/xap/1.0/\0")
                        reader.seek(SeekFrom::Start(segment_start))?;
                        let mut xmp_header = [0u8; 29];
                        if reader.read_exact(&mut xmp_header).is_ok()
                            && &xmp_header == b"http://ns.adobe.com/xap/1.0/\0"
                        {
                            // Found XMP - store it and continue scanning
                            found_xmp = Some(JpegSegmentInfo {
                                segment_type: segment,
                                offset: current_pos + 29, // After XMP identifier(29)
                                length: length - 31, // Subtract segment length header(2) + XMP identifier(29)
                                has_exif: false,
                                has_xmp: true,
                            });
                        }
                    }

                    // Reset to start of segment data for skipping
                    reader.seek(SeekFrom::Start(segment_start))?;
                }

                // Skip to next segment
                let segment_data_length = length.saturating_sub(2) as u64;
                reader.seek(SeekFrom::Current(segment_data_length as i64))?;
                current_pos += segment_data_length;
            }
            JpegSegment::Sof(marker) => {
                // Read segment length
                let mut length_bytes = [0u8; 2];
                reader.read_exact(&mut length_bytes)?;
                let length = u16::from_be_bytes(length_bytes);
                current_pos += 2;

                // Read SOF data if we haven't found one yet
                // ExifTool: lib/Image/ExifTool.pm:7320-7336
                if sof_data.is_none() && length >= 8 {
                    // Minimum SOF size
                    let mut sof_segment_data = vec![0u8; (length - 2) as usize];
                    if reader.read_exact(&mut sof_segment_data).is_ok() {
                        if let Ok(sof) = parse_sof_data(marker, &sof_segment_data) {
                            sof_data = Some(sof);
                        }
                        current_pos += (length - 2) as u64;
                    } else {
                        break;
                    }
                } else {
                    // Skip if already have SOF data or too short
                    let segment_data_length = length.saturating_sub(2) as u64;
                    reader.seek(SeekFrom::Current(segment_data_length as i64))?;
                    current_pos += segment_data_length;
                }
            }
            _ => {
                // Other segments - skip them
                let mut length_bytes = [0u8; 2];
                if reader.read_exact(&mut length_bytes).is_ok() {
                    let length = u16::from_be_bytes(length_bytes);
                    let segment_data_length = length.saturating_sub(2) as u64;
                    reader.seek(SeekFrom::Current(segment_data_length as i64))?;
                    current_pos += 2 + segment_data_length;
                } else {
                    break;
                }
            }
        }
    }

    // Prioritize EXIF over XMP (following ExifTool behavior)
    Ok((found_exif.or(found_xmp), sof_data))
}

/// Result of scanning JPEG for XMP segments
pub struct XmpScanResult {
    pub regular_xmp: Option<JpegSegmentInfo>,
    pub extended_xmp: Vec<ExtendedXmpInfo>,
}

/// Scan JPEG file for all XMP segments
///
/// Returns regular XMP segment info (if found) and all Extended XMP segments.
/// Extended XMP segments contain GUID-based chunks that need reassembly.
pub fn scan_jpeg_xmp_segments<R: Read + Seek>(mut reader: R) -> Result<XmpScanResult> {
    // Verify JPEG magic bytes
    let mut magic = [0u8; 2];
    reader.read_exact(&mut magic)?;
    if magic != [0xFF, 0xD8] {
        return Err(ExifError::InvalidFormat(
            "Not a valid JPEG file (missing 0xFFD8 magic bytes)".to_string(),
        ));
    }

    let mut regular_xmp = None;
    let mut extended_xmp = Vec::new();
    let mut current_pos = 2u64; // After SOI marker

    loop {
        // Read segment marker
        let mut marker_bytes = [0u8; 2];
        if reader.read_exact(&mut marker_bytes).is_err() {
            break;
        }

        if marker_bytes[0] != 0xFF {
            return Err(ExifError::ParseError(
                "Invalid JPEG segment marker".to_string(),
            ));
        }

        let segment = JpegSegment::from_marker(marker_bytes[1]);
        current_pos += 2;

        match segment {
            JpegSegment::Soi => continue,
            JpegSegment::Eoi | JpegSegment::Sos => break,
            JpegSegment::App(1) => {
                // Read segment length
                let mut length_bytes = [0u8; 2];
                reader.read_exact(&mut length_bytes)?;
                let length = u16::from_be_bytes(length_bytes);
                current_pos += 2;

                let segment_start = current_pos;

                // Check for regular XMP identifier
                let mut xmp_header = [0u8; 29];
                if reader.read_exact(&mut xmp_header).is_ok()
                    && &xmp_header == b"http://ns.adobe.com/xap/1.0/\0"
                {
                    // Store first regular XMP segment only
                    if regular_xmp.is_none() {
                        regular_xmp = Some(JpegSegmentInfo {
                            segment_type: segment,
                            offset: current_pos + 29,
                            length: length - 31, // Subtract length header + identifier
                            has_exif: false,
                            has_xmp: true,
                        });
                    }

                    // Skip to next segment
                    // ExifTool: lib/Image/ExifTool/JPEG.pm:436-440 - seek to next segment
                    let remaining = (length - 31) as u64;
                    reader.seek(SeekFrom::Current(remaining as i64))?;
                    current_pos = segment_start + (length - 2) as u64;
                    continue;
                }

                // Reset and check for Extended XMP identifier
                reader.seek(SeekFrom::Start(segment_start))?;
                let mut ext_xmp_header = [0u8; 35];
                if reader.read_exact(&mut ext_xmp_header).is_ok()
                    && &ext_xmp_header[0..35] == b"http://ns.adobe.com/xmp/extension/\0"
                {
                    // Read Extended XMP header fields
                    // ExifTool: lib/Image/ExifTool.pm:7738-7751
                    // off len -- extended XMP header (75 bytes total):
                    //   0  35 bytes - signature
                    //  35  32 bytes - GUID (MD5 hash of full extended XMP data in ASCII)
                    //  67   4 bytes - total size of extended XMP data
                    //  71   4 bytes - offset for this XMP data portion

                    // Read GUID (32 bytes)
                    let mut guid_bytes = [0u8; 32];
                    reader.read_exact(&mut guid_bytes)?;
                    let guid = String::from_utf8_lossy(&guid_bytes).to_string();

                    // Validate GUID contains only alphanumeric characters
                    // ExifTool: lib/Image/ExifTool.pm:7741-7742
                    if !guid.chars().all(|c| c.is_ascii_alphanumeric()) {
                        // Skip invalid Extended XMP segment
                        reader.seek(SeekFrom::Start(segment_start))?;
                        let segment_data_length = length.saturating_sub(2) as u64;
                        reader.seek(SeekFrom::Current(segment_data_length as i64))?;
                        current_pos = segment_start + segment_data_length;
                        continue;
                    }

                    // Read total size (4 bytes)
                    // ExifTool: lib/Image/ExifTool.pm:7739 - unpack('x67N2', $$segDataPt)
                    let mut size_bytes = [0u8; 4];
                    reader.read_exact(&mut size_bytes)?;
                    let total_size = u32::from_be_bytes(size_bytes);

                    // Read chunk offset (4 bytes)
                    let mut offset_bytes = [0u8; 4];
                    reader.read_exact(&mut offset_bytes)?;
                    let chunk_offset = u32::from_be_bytes(offset_bytes);

                    // Extended XMP header is 75 bytes total (35 + 32 + 4 + 4)
                    // ExifTool: lib/Image/ExifTool.pm:7751 - $$extXMP{$off} = substr($$segDataPt, 75)
                    extended_xmp.push(ExtendedXmpInfo {
                        guid,
                        total_size,
                        chunk_offset,
                        segment_offset: current_pos + 75, // After full header
                        chunk_length: length - 77, // Subtract length header (2) + extended header (75)
                    });

                    // Skip to next segment
                    // ExifTool: lib/Image/ExifTool.pm:7753-7754 - processing next segment
                    let remaining = (length - 77) as u64;
                    reader.seek(SeekFrom::Current(remaining as i64))?;
                    current_pos = segment_start + (length - 2) as u64;
                    continue;
                }

                // Not XMP - skip this APP1 segment
                reader.seek(SeekFrom::Start(segment_start))?;
                let segment_data_length = length.saturating_sub(2) as u64;
                reader.seek(SeekFrom::Current(segment_data_length as i64))?;
                current_pos = segment_start + segment_data_length;
            }
            _ => {
                // Other segments - skip them
                let mut length_bytes = [0u8; 2];
                if reader.read_exact(&mut length_bytes).is_ok() {
                    let length = u16::from_be_bytes(length_bytes);
                    let segment_data_length = length.saturating_sub(2) as u64;
                    reader.seek(SeekFrom::Current(segment_data_length as i64))?;
                    current_pos += 2 + segment_data_length;
                } else {
                    break;
                }
            }
        }
    }

    Ok(XmpScanResult {
        regular_xmp,
        extended_xmp,
    })
}

/// Extract HasExtendedXMP GUID from regular XMP data
///
/// This searches for the xmpNote:HasExtendedXMP property in the XMP packet
/// which contains the GUID of the extended XMP data to reassemble.
///
/// ExifTool: lib/Image/ExifTool.pm:7485 - my $goodGuid = $$self{VALUE}{HasExtendedXMP} || '';
fn extract_has_extended_xmp_guid(xmp_data: &[u8]) -> Option<String> {
    // Convert to string for searching
    let xmp_str = std::str::from_utf8(xmp_data).ok()?;

    // Look for HasExtendedXMP property
    // ExifTool: lib/Image/ExifTool/XMP.pm:2380-2390 - HasExtendedXMP extraction
    // Can be in attribute format: xmpNote:HasExtendedXMP="GUID"
    // Or element format: <xmpNote:HasExtendedXMP>GUID</xmpNote:HasExtendedXMP>

    // First try element format (more common)
    if let Some(start_pos) = xmp_str.find("<xmpNote:HasExtendedXMP>") {
        let guid_start = start_pos + "<xmpNote:HasExtendedXMP>".len();
        if let Some(end_pos) = xmp_str[guid_start..].find("</xmpNote:HasExtendedXMP>") {
            let guid = &xmp_str[guid_start..guid_start + end_pos];
            // Validate GUID is 32 alphanumeric characters
            // ExifTool: lib/Image/ExifTool.pm:7741 - $$extXMP{GUID} =~ /[^0-9a-fA-F]/
            if guid.len() == 32 && guid.chars().all(|c| c.is_ascii_alphanumeric()) {
                return Some(guid.to_string());
            }
        }
    }

    // Also try without namespace prefix
    if let Some(start_pos) = xmp_str.find("<HasExtendedXMP>") {
        let guid_start = start_pos + "<HasExtendedXMP>".len();
        if let Some(end_pos) = xmp_str[guid_start..].find("</HasExtendedXMP>") {
            let guid = &xmp_str[guid_start..guid_start + end_pos];
            // Validate GUID is 32 alphanumeric characters
            // ExifTool: lib/Image/ExifTool.pm:7741 - $$extXMP{GUID} =~ /[^0-9a-fA-F]/
            if guid.len() == 32 && guid.chars().all(|c| c.is_ascii_alphanumeric()) {
                return Some(guid.to_string());
            }
        }
    }

    // Try attribute format
    let patterns = [
        "xmpNote:HasExtendedXMP=\"",
        "xmpNote:HasExtendedXMP='",
        "HasExtendedXMP=\"",
        "HasExtendedXMP='",
    ];

    for pattern in &patterns {
        if let Some(start_pos) = xmp_str.find(pattern) {
            let guid_start = start_pos + pattern.len();
            let quote_char = pattern.chars().last()?;

            // Find closing quote
            if let Some(end_pos) = xmp_str[guid_start..].find(quote_char) {
                let guid = &xmp_str[guid_start..guid_start + end_pos];

                // Validate GUID is 32 alphanumeric characters
                if guid.len() == 32 && guid.chars().all(|c| c.is_ascii_alphanumeric()) {
                    return Some(guid.to_string());
                }
            }
        }
    }

    None
}

/// Extract XMP data from JPEG file
///
/// This function scans for APP1 segments containing XMP data and returns
/// the raw XMP packet(s). For Extended XMP, multiple segments are reassembled.
///
/// ExifTool: lib/Image/ExifTool.pm:7482-7524 (Extended XMP reassembly)
pub fn extract_jpeg_xmp<R: Read + Seek>(mut reader: R) -> Result<Vec<u8>> {
    let scan_result = scan_jpeg_xmp_segments(&mut reader)?;

    // First, check if we have regular XMP
    if let Some(regular_xmp) = &scan_result.regular_xmp {
        // Read regular XMP data
        reader.seek(SeekFrom::Start(regular_xmp.offset))?;
        let mut xmp_data = vec![0u8; regular_xmp.length as usize];
        reader.read_exact(&mut xmp_data)?;

        // Check if this XMP contains HasExtendedXMP property
        if let Some(has_extended_guid) = extract_has_extended_xmp_guid(&xmp_data) {
            // We have Extended XMP to reassemble
            // ExifTool: lib/Image/ExifTool.pm:7487-7488
            if !scan_result.extended_xmp.is_empty() {
                // Group Extended XMP chunks by GUID
                let mut guid_chunks: HashMap<String, BTreeMap<u32, Vec<u8>>> = HashMap::new();

                for ext_info in &scan_result.extended_xmp {
                    // Only process chunks matching the HasExtendedXMP GUID
                    if ext_info.guid == has_extended_guid {
                        // Read chunk data
                        reader.seek(SeekFrom::Start(ext_info.segment_offset))?;
                        let mut chunk_data = vec![0u8; ext_info.chunk_length as usize];
                        reader.read_exact(&mut chunk_data)?;

                        // Store chunk indexed by offset for ordered reassembly
                        // ExifTool: lib/Image/ExifTool.pm:7751 - $$extXMP{$off} = substr($$segDataPt, 75)
                        guid_chunks
                            .entry(ext_info.guid.clone())
                            .or_default()
                            .insert(ext_info.chunk_offset, chunk_data);
                    }
                }

                // Reassemble Extended XMP with matching GUID
                if let Some(chunks) = guid_chunks.get(&has_extended_guid) {
                    // Check if we have all chunks
                    let mut expected_offset = 0u32;
                    let mut total_size = 0u32;
                    let mut is_complete = true;

                    // Get total size from first matching segment
                    for ext_info in &scan_result.extended_xmp {
                        if ext_info.guid == has_extended_guid {
                            total_size = ext_info.total_size;
                            break;
                        }
                    }

                    // Verify we have all chunks in sequence
                    // ExifTool: lib/Image/ExifTool.pm:7494-7498 - check for missing chunks
                    for (offset, chunk) in chunks {
                        if *offset != expected_offset {
                            is_complete = false;
                            break;
                        }
                        expected_offset += chunk.len() as u32;
                    }

                    if is_complete && expected_offset == total_size {
                        // Combine regular XMP with Extended XMP
                        // The Extended XMP is appended after the regular XMP
                        // ExifTool: lib/Image/ExifTool.pm:7506-7507
                        let mut combined_xmp = xmp_data;
                        for chunk in chunks.values() {
                            combined_xmp.extend_from_slice(chunk);
                        }
                        return Ok(combined_xmp);
                    }
                }
            }
        }

        // Return just regular XMP if no Extended XMP or incomplete
        return Ok(xmp_data);
    }

    // No regular XMP - check for Extended XMP only (unusual but possible)
    if !scan_result.extended_xmp.is_empty() {
        // Group Extended XMP chunks by GUID
        let mut guid_chunks: HashMap<String, BTreeMap<u32, Vec<u8>>> = HashMap::new();

        for ext_info in &scan_result.extended_xmp {
            // Read chunk data
            reader.seek(SeekFrom::Start(ext_info.segment_offset))?;
            let mut chunk_data = vec![0u8; ext_info.chunk_length as usize];
            reader.read_exact(&mut chunk_data)?;

            // Store chunk indexed by offset for ordered reassembly
            guid_chunks
                .entry(ext_info.guid.clone())
                .or_default()
                .insert(ext_info.chunk_offset, chunk_data);
        }

        // Find the first complete Extended XMP
        // ExifTool: lib/Image/ExifTool.pm:7493-7500
        for (guid, chunks) in guid_chunks {
            // Check if we have all chunks
            let mut expected_offset = 0u32;
            let mut total_size = 0u32;
            let mut is_complete = true;

            // Get total size from first matching segment
            for ext_info in &scan_result.extended_xmp {
                if ext_info.guid == guid {
                    total_size = ext_info.total_size;
                    break;
                }
            }

            // Verify we have all chunks in sequence
            for (offset, chunk) in &chunks {
                if *offset != expected_offset {
                    is_complete = false;
                    break;
                }
                expected_offset += chunk.len() as u32;
            }

            if is_complete && expected_offset == total_size {
                // Reassemble complete Extended XMP
                // ExifTool: lib/Image/ExifTool.pm:7506-7507
                let mut reassembled = Vec::with_capacity(total_size as usize);
                for (_, chunk) in chunks {
                    reassembled.extend_from_slice(&chunk);
                }
                return Ok(reassembled);
            }
        }

        return Err(ExifError::InvalidFormat(
            "Incomplete Extended XMP data".to_string(),
        ));
    }

    Err(ExifError::InvalidFormat(
        "No XMP data found in JPEG file".to_string(),
    ))
}

/// Extract EXIF data from JPEG file
///
/// This function scans the JPEG for APP1 segments containing EXIF data
/// and returns the raw EXIF/TIFF data for further processing.
pub fn extract_jpeg_exif<R: Read + Seek>(mut reader: R) -> Result<Vec<u8>> {
    // Scan for EXIF segment
    reader.seek(SeekFrom::Start(0))?;
    let (segment_info, _sof_data) = scan_jpeg_segments(&mut reader)?;

    match segment_info {
        Some(info) if info.has_exif => {
            // Read EXIF data
            reader.seek(SeekFrom::Start(info.offset))?;
            let mut exif_data = vec![0u8; info.length as usize];
            reader.read_exact(&mut exif_data)?;
            Ok(exif_data)
        }
        _ => Err(ExifError::InvalidFormat(
            "No EXIF data found in JPEG file".to_string(),
        )),
    }
}

/// Extract IPTC data from JPEG APP13 segments
///
/// This function scans JPEG segments for APP13 segments containing Adobe Photoshop
/// Image Resource Blocks with IPTC data (Resource ID 0x0404).
pub fn extract_jpeg_iptc<R: Read + Seek>(
    mut reader: R,
) -> Result<std::collections::HashMap<String, crate::types::TagValue>> {
    use crate::formats::parse_iptc_from_app13;
    use std::collections::HashMap;

    reader.seek(SeekFrom::Start(0))?;

    // Read JPEG header
    let mut header = [0u8; 2];
    reader.read_exact(&mut header)?;

    if header != [0xFF, 0xD8] {
        return Err(ExifError::InvalidFormat(
            "Not a valid JPEG file".to_string(),
        ));
    }

    let mut combined_iptc_tags = HashMap::new();

    // Scan for APP13 segments
    loop {
        let mut marker_bytes = [0u8; 2];
        match reader.read_exact(&mut marker_bytes) {
            Ok(_) => {}
            Err(_) => break, // End of file
        }

        if marker_bytes[0] != 0xFF {
            return Err(ExifError::InvalidFormat("Invalid JPEG marker".to_string()));
        }

        let marker = marker_bytes[1];

        // Check for end of image or start of scan
        if marker == 0xD9 || marker == 0xDA {
            break;
        }

        // Skip segments without length
        if marker == 0xD0
            || marker == 0xD1
            || marker == 0xD2
            || marker == 0xD3
            || marker == 0xD4
            || marker == 0xD5
            || marker == 0xD6
            || marker == 0xD7
            || marker == 0x01
        {
            continue;
        }

        // Read segment length
        let mut length_bytes = [0u8; 2];
        reader.read_exact(&mut length_bytes)?;
        let length = u16::from_be_bytes(length_bytes) as usize;

        if length < 2 {
            return Err(ExifError::InvalidFormat(
                "Invalid segment length".to_string(),
            ));
        }

        // Check if this is an APP13 segment (0xED)
        if marker == 0xED {
            // Read APP13 segment data
            let mut app13_data = vec![0u8; length - 2]; // Subtract 2 for length bytes
            reader.read_exact(&mut app13_data)?;

            // Try to parse IPTC from this APP13 segment
            match parse_iptc_from_app13(&app13_data) {
                Ok(iptc_tags) => {
                    if !iptc_tags.is_empty() {
                        // Merge IPTC tags from this segment
                        combined_iptc_tags.extend(iptc_tags);
                    }
                }
                Err(e) => {
                    // Log error but continue scanning for other APP13 segments
                    tracing::debug!("Failed to parse IPTC from APP13 segment: {}", e);
                }
            }
        } else {
            // Skip other segments
            reader.seek(SeekFrom::Current((length - 2) as i64))?;
        }
    }

    if combined_iptc_tags.is_empty() {
        Err(ExifError::InvalidFormat(
            "No APP13 segment with IPTC data found".to_string(),
        ))
    } else {
        Ok(combined_iptc_tags)
    }
}

/// Hash JPEG image data (scan data from SOS to EOI)
///
/// ExifTool Reference: lib/Image/ExifTool.pm:7217-7406
///
/// JPEG image data hashing includes:
/// - SOS (Start of Scan) marker 0xFF 0xDA and its segment data
/// - RST (Restart) markers 0xFF 0xD0-0xD7 within scan data
/// - Stuffed bytes 0xFF 0x00 (representing literal 0xFF in image data)
/// - All entropy-coded image data
///
/// Hashing stops at EOI (0xFF 0xD9).
///
/// The hash includes the marker bytes themselves (0xFF + marker type).
pub fn hash_jpeg_scan_data<R: Read + Seek>(
    reader: &mut R,
    hasher: &mut ImageDataHasher,
) -> Result<u64> {
    // Verify JPEG magic bytes and scan to SOS
    reader.seek(SeekFrom::Start(0))?;
    let mut magic = [0u8; 2];
    reader.read_exact(&mut magic)?;
    if magic != [0xFF, 0xD8] {
        return Err(ExifError::InvalidFormat(
            "Not a valid JPEG file (missing 0xFFD8 magic bytes)".to_string(),
        ));
    }

    // Scan through segments to find SOS
    loop {
        let mut marker_bytes = [0u8; 2];
        if reader.read_exact(&mut marker_bytes).is_err() {
            return Ok(0); // EOF before SOS
        }

        if marker_bytes[0] != 0xFF {
            return Err(ExifError::ParseError(
                "Invalid JPEG segment marker".to_string(),
            ));
        }

        let marker = marker_bytes[1];

        match marker {
            0xD8 => continue,     // SOI - skip
            0xD9 => return Ok(0), // EOI before SOS - no image data
            0xDA => {
                // SOS found - start hashing
                // ExifTool: lib/Image/ExifTool.pm:7359-7364
                // Hash the SOS marker
                hasher.update(&[0xFF, 0xDA]);

                // Read SOS segment length
                let mut length_bytes = [0u8; 2];
                reader.read_exact(&mut length_bytes)?;
                let length = u16::from_be_bytes(length_bytes) as usize;

                // Hash the length bytes as part of SOS
                hasher.update(&length_bytes);

                // Read and hash SOS segment data (headers before entropy data)
                if length > 2 {
                    let mut sos_header = vec![0u8; length - 2];
                    reader.read_exact(&mut sos_header)?;
                    hasher.update(&sos_header);
                }

                // Now hash entropy-coded data until EOI
                // ExifTool: lib/Image/ExifTool.pm:7363-7406
                break;
            }
            0x00 | 0xD0..=0xD7 | 0x01 => {
                // Markers without length (standalone markers)
                // RST0-RST7, TEM - these shouldn't appear before SOS
                continue;
            }
            _ => {
                // Regular segment with length - skip it
                let mut length_bytes = [0u8; 2];
                reader.read_exact(&mut length_bytes)?;
                let length = u16::from_be_bytes(length_bytes);
                let skip = length.saturating_sub(2) as i64;
                reader.seek(SeekFrom::Current(skip))?;
            }
        }
    }

    // Now read entropy-coded data until EOI
    // JPEG entropy data uses byte stuffing: 0xFF is followed by 0x00 for literal 0xFF
    // Real markers are 0xFF followed by non-zero marker type
    //
    // ExifTool hashes:
    // - 0xFF 0x00 (stuffed bytes) - includes the 0xFF00 in hash
    // - 0xFF 0xD0-0xD7 (RST markers) - includes marker in hash
    // - Regular data bytes
    //
    // ExifTool: lib/Image/ExifTool.pm:7366-7406
    let mut bytes_hashed = 4u64; // Already hashed 0xFF 0xDA + length (2 bytes)

    // Read in chunks for efficiency, but process byte-by-byte for marker detection
    const CHUNK_SIZE: usize = 65536;
    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut pending_ff = false;

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break; // EOF
        }

        let mut i = 0;
        while i < bytes_read {
            let byte = buffer[i];

            if pending_ff {
                pending_ff = false;

                match byte {
                    0x00 => {
                        // Stuffed byte - hash 0xFF 0x00
                        // ExifTool: lib/Image/ExifTool.pm:7366 - $marker == 0x00
                        hasher.update(&[0xFF, 0x00]);
                        bytes_hashed += 2;
                    }
                    0xD0..=0xD7 => {
                        // RST marker - hash 0xFF + marker
                        // ExifTool: lib/Image/ExifTool.pm:7367 - $marker >= 0xd0 and $marker <= 0xd7
                        hasher.update(&[0xFF, byte]);
                        bytes_hashed += 2;
                    }
                    0xD9 => {
                        // EOI - end of image data
                        // Don't hash EOI marker (ExifTool doesn't include it)
                        return Ok(bytes_hashed);
                    }
                    0xDA => {
                        // Another SOS (shouldn't happen in valid JPEG but handle it)
                        // ExifTool: lib/Image/ExifTool.pm:7364 - $marker == 0xda
                        hasher.update(&[0xFF, 0xDA]);
                        bytes_hashed += 2;

                        // Read and hash the SOS segment
                        let mut length_bytes = [0u8; 2];
                        // Need to read from buffer or refill
                        if i + 2 < bytes_read {
                            length_bytes[0] = buffer[i + 1];
                            length_bytes[1] = buffer[i + 2];
                            i += 2;
                        } else {
                            // Need to read more data
                            reader.read_exact(&mut length_bytes)?;
                        }
                        hasher.update(&length_bytes);
                        bytes_hashed += 2;

                        let length = u16::from_be_bytes(length_bytes) as usize;
                        if length > 2 {
                            let mut sos_data = vec![0u8; length - 2];
                            reader.read_exact(&mut sos_data)?;
                            hasher.update(&sos_data);
                            bytes_hashed += (length - 2) as u64;
                        }
                    }
                    0xFF => {
                        // 0xFF 0xFF - keep pending
                        pending_ff = true;
                    }
                    _ => {
                        // Other marker in scan data - unusual but skip
                        // ExifTool skips these
                        tracing::debug!("Unexpected marker 0x{:02X} in JPEG scan data", byte);
                    }
                }
            } else if byte == 0xFF {
                pending_ff = true;
            } else {
                // Regular data byte - hash it
                hasher.update(&[byte]);
                bytes_hashed += 1;
            }

            i += 1;
        }
    }

    // Handle trailing 0xFF at EOF (shouldn't happen in valid JPEG)
    if pending_ff {
        hasher.update(&[0xFF]);
        bytes_hashed += 1;
    }

    Ok(bytes_hashed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_jpeg_segment_from_marker() {
        assert_eq!(JpegSegment::from_marker(0xD8), JpegSegment::Soi);
        assert_eq!(JpegSegment::from_marker(0xE1), JpegSegment::App(1));
        assert_eq!(JpegSegment::from_marker(0xC0), JpegSegment::Sof(0xC0));
        assert_eq!(JpegSegment::from_marker(0xC1), JpegSegment::Sof(0xC1));
        assert_eq!(JpegSegment::from_marker(0xC2), JpegSegment::Sof(0xC2));
        assert_eq!(JpegSegment::from_marker(0xC4), JpegSegment::Dht); // DHT, not SOF
        assert_eq!(JpegSegment::from_marker(0xC8), JpegSegment::Other(0xC8)); // JPGA, not SOF
        assert_eq!(JpegSegment::from_marker(0xCC), JpegSegment::Other(0xCC)); // DAC, not SOF
        assert_eq!(JpegSegment::from_marker(0xDA), JpegSegment::Sos);
        assert_eq!(JpegSegment::from_marker(0xD9), JpegSegment::Eoi);
    }

    #[test]
    fn test_jpeg_segment_is_app1() {
        assert!(JpegSegment::App(1).is_app1());
        assert!(!JpegSegment::App(0).is_app1());
        assert!(!JpegSegment::Soi.is_app1());
    }

    #[test]
    fn test_jpeg_segment_marker_byte() {
        assert_eq!(JpegSegment::Soi.marker_byte(), 0xD8);
        assert_eq!(JpegSegment::App(1).marker_byte(), 0xE1);
        assert_eq!(JpegSegment::Sof(0xC0).marker_byte(), 0xC0);
        assert_eq!(JpegSegment::Sof(0xC2).marker_byte(), 0xC2);
        assert_eq!(JpegSegment::Eoi.marker_byte(), 0xD9);
    }

    #[test]
    fn test_scan_jpeg_segments_invalid_magic() {
        let invalid_jpeg = [0x12, 0x34, 0x56, 0x78];
        let cursor = Cursor::new(invalid_jpeg);
        let result = scan_jpeg_segments(cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_jpeg_segments_minimal() {
        // Minimal JPEG: SOI + EOI
        let minimal_jpeg = [0xFF, 0xD8, 0xFF, 0xD9];
        let cursor = Cursor::new(minimal_jpeg);
        let (segment_info, sof_data) = scan_jpeg_segments(cursor).unwrap();
        assert!(segment_info.is_none()); // No EXIF data
        assert!(sof_data.is_none()); // No SOF data
    }

    #[test]
    fn test_scan_jpeg_segments_with_app1_exif() {
        // JPEG with APP1 segment containing EXIF
        let jpeg_data = vec![
            0xFF, 0xD8, // SOI
            0xFF, 0xE1, // APP1 marker
            0x00, 0x10, // Segment length (16 bytes)
            0x45, 0x78, 0x69, 0x66, 0x00, 0x00, // "Exif\0\0"
            0x49, 0x49, 0x2A, 0x00, // TIFF header (minimal)
            0x08, 0x00, 0x00, 0x00, // IFD offset
            0xFF, 0xD9, // EOI
        ];

        let cursor = Cursor::new(&jpeg_data);
        let (segment_info_opt, sof_data) = scan_jpeg_segments(cursor).unwrap();
        assert!(segment_info_opt.is_some());
        assert!(sof_data.is_none()); // No SOF in this test

        let segment_info = segment_info_opt.unwrap();
        assert!(segment_info.has_exif);
        assert!(!segment_info.has_xmp);
        assert_eq!(segment_info.offset, 12); // After SOI(2) + APP1 marker(2) + length(2) + "Exif\0\0"(6) = 12
        assert_eq!(segment_info.length, 8); // 16 - 8 = 8 bytes of TIFF data
    }

    #[test]
    fn test_scan_jpeg_segments_with_app1_xmp() {
        // JPEG with APP1 segment containing XMP
        let xmp_identifier = b"http://ns.adobe.com/xap/1.0/\0"; // 29 bytes
        let xmp_packet = b"<?xml?><x:xmpmeta></x:xmpmeta>"; // 30 bytes
        let segment_length = 2 + xmp_identifier.len() + xmp_packet.len(); // length field (2) + identifier + packet

        let mut jpeg_data = vec![
            0xFF,
            0xD8, // SOI
            0xFF,
            0xE1, // APP1 marker
            (segment_length >> 8) as u8,
            (segment_length & 0xFF) as u8, // Segment length
        ];

        // XMP identifier and packet
        jpeg_data.extend_from_slice(xmp_identifier);
        jpeg_data.extend_from_slice(xmp_packet);

        // EOI
        jpeg_data.extend_from_slice(&[0xFF, 0xD9]);

        let cursor = Cursor::new(&jpeg_data);
        let (segment_info_opt, sof_data) = scan_jpeg_segments(cursor).unwrap();
        assert!(segment_info_opt.is_some());
        assert!(sof_data.is_none()); // No SOF in this test

        let segment_info = segment_info_opt.unwrap();
        assert!(!segment_info.has_exif);
        assert!(segment_info.has_xmp);
        // Offset should be after SOI(2) + APP1 marker(2) + length(2) + XMP identifier(29) = 35
        assert_eq!(segment_info.offset, 35);
        // Length should be segment_length - length_field(2) - identifier(29) = 30
        assert_eq!(segment_info.length, 30);
    }

    #[test]
    fn test_scan_jpeg_xmp_segments() {
        // JPEG with XMP segment
        let xmp_identifier = b"http://ns.adobe.com/xap/1.0/\0"; // 29 bytes
        let xmp_packet = b"<?xml?><x:xmpmeta></x:xmpmeta>"; // 30 bytes
        let segment_length = 2 + xmp_identifier.len() + xmp_packet.len(); // 2 + 29 + 30 = 61

        let mut jpeg_data = vec![
            0xFF,
            0xD8, // SOI
            0xFF,
            0xE1, // APP1 marker
            (segment_length >> 8) as u8,
            (segment_length & 0xFF) as u8, // Segment length
        ];

        // XMP identifier and packet
        jpeg_data.extend_from_slice(xmp_identifier);
        jpeg_data.extend_from_slice(xmp_packet);

        // EOI
        jpeg_data.extend_from_slice(&[0xFF, 0xD9]);

        let cursor = Cursor::new(&jpeg_data);
        let result = scan_jpeg_xmp_segments(cursor).unwrap();
        assert!(result.regular_xmp.is_some());
        assert!(result.extended_xmp.is_empty());

        let segment_info = result.regular_xmp.unwrap();
        assert!(segment_info.has_xmp);
        assert_eq!(segment_info.length, 30); // Just the XMP packet size
    }

    #[test]
    fn test_extract_jpeg_xmp() {
        // JPEG with XMP segment
        let xmp_identifier = b"http://ns.adobe.com/xap/1.0/\0"; // 29 bytes
        let xmp_packet = b"<?xml?><x:xmpmeta></x:xmpmeta>"; // 30 bytes
        let segment_length = 2 + xmp_identifier.len() + xmp_packet.len(); // 2 + 29 + 30 = 61

        let mut jpeg_data = vec![
            0xFF,
            0xD8, // SOI
            0xFF,
            0xE1, // APP1 marker
            (segment_length >> 8) as u8,
            (segment_length & 0xFF) as u8, // Segment length
        ];

        // XMP identifier and packet
        jpeg_data.extend_from_slice(xmp_identifier);
        jpeg_data.extend_from_slice(xmp_packet);

        // EOI
        jpeg_data.extend_from_slice(&[0xFF, 0xD9]);

        let cursor = Cursor::new(&jpeg_data);
        let result = extract_jpeg_xmp(cursor);
        assert!(result.is_ok());

        let xmp_data = result.unwrap();
        assert_eq!(xmp_data, xmp_packet);
    }

    #[test]
    fn test_hash_jpeg_scan_data_minimal() {
        use crate::hash::{ImageDataHasher, ImageHashType};

        // Minimal JPEG with SOS and scan data
        // Structure: SOI + SOS segment + scan data + EOI
        let jpeg_data = vec![
            0xFF, 0xD8, // SOI
            0xFF, 0xDA, // SOS marker
            0x00, 0x08, // SOS length (8 bytes)
            0x01, 0x00, 0x00, 0x3F, 0x00, 0x00, // SOS header data (6 bytes after length)
            // Scan data - some bytes that need no byte stuffing
            0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC,
            0xFF, 0xD9, // EOI
        ];

        let mut cursor = Cursor::new(&jpeg_data);
        let mut hasher = ImageDataHasher::new(ImageHashType::Md5);

        let bytes_hashed = hash_jpeg_scan_data(&mut cursor, &mut hasher).unwrap();
        // SOS marker (2) + length (2) + header (6) + scan data (6) = 16 bytes
        assert!(bytes_hashed > 0);

        let hash = hasher.finalize().unwrap();
        assert_eq!(hash.len(), 32); // MD5 = 32 hex chars
    }

    #[test]
    fn test_hash_jpeg_scan_data_with_stuffed_bytes() {
        use crate::hash::{ImageDataHasher, ImageHashType};

        // JPEG with 0xFF in scan data (requires byte stuffing)
        let jpeg_data = vec![
            0xFF, 0xD8, // SOI
            0xFF, 0xDA, // SOS marker
            0x00, 0x04, // SOS length (4 bytes = 2 + 2 header bytes)
            0x01, 0x00, // SOS header data (2 bytes)
            // Scan data with stuffed byte (0xFF followed by 0x00)
            0x12, 0xFF, 0x00, 0x34, // 0xFF 0x00 = literal 0xFF in data
            0xFF, 0xD9, // EOI
        ];

        let mut cursor = Cursor::new(&jpeg_data);
        let mut hasher = ImageDataHasher::new(ImageHashType::Md5);

        let bytes_hashed = hash_jpeg_scan_data(&mut cursor, &mut hasher).unwrap();
        assert!(bytes_hashed > 0);
    }

    #[test]
    fn test_hash_jpeg_scan_data_with_rst_marker() {
        use crate::hash::{ImageDataHasher, ImageHashType};

        // JPEG with RST marker in scan data
        let jpeg_data = vec![
            0xFF, 0xD8, // SOI
            0xFF, 0xDA, // SOS marker
            0x00, 0x04, // SOS length
            0x01, 0x00, // SOS header
            // Scan data with RST0 marker
            0x12, 0xFF, 0xD0, 0x34, // RST0 marker
            0xFF, 0xD9, // EOI
        ];

        let mut cursor = Cursor::new(&jpeg_data);
        let mut hasher = ImageDataHasher::new(ImageHashType::Md5);

        let bytes_hashed = hash_jpeg_scan_data(&mut cursor, &mut hasher).unwrap();
        assert!(bytes_hashed > 0);
    }

    #[test]
    fn test_hash_jpeg_scan_data_no_sos() {
        use crate::hash::{ImageDataHasher, ImageHashType};

        // JPEG without SOS (just SOI + EOI)
        let jpeg_data = vec![
            0xFF, 0xD8, // SOI
            0xFF, 0xD9, // EOI
        ];

        let mut cursor = Cursor::new(&jpeg_data);
        let mut hasher = ImageDataHasher::new(ImageHashType::Md5);

        let bytes_hashed = hash_jpeg_scan_data(&mut cursor, &mut hasher).unwrap();
        assert_eq!(bytes_hashed, 0); // No scan data to hash
    }

    #[test]
    fn test_hash_jpeg_scan_data_with_app_segment() {
        use crate::hash::{ImageDataHasher, ImageHashType};

        // JPEG with APP1 segment before SOS (common case)
        let jpeg_data = vec![
            0xFF, 0xD8, // SOI
            0xFF, 0xE1, // APP1 marker
            0x00, 0x06, // APP1 length (6 bytes = 2 + 4 data)
            0x45, 0x78, 0x69, 0x66, // "Exif" (just header, no full EXIF)
            0xFF, 0xDA, // SOS marker
            0x00, 0x04, // SOS length
            0x01, 0x00, // SOS header
            // Scan data
            0xAB, 0xCD, 0xEF,
            0xFF, 0xD9, // EOI
        ];

        let mut cursor = Cursor::new(&jpeg_data);
        let mut hasher = ImageDataHasher::new(ImageHashType::Md5);

        let bytes_hashed = hash_jpeg_scan_data(&mut cursor, &mut hasher).unwrap();
        // Should skip APP1 and only hash SOS + scan data
        assert!(bytes_hashed > 0);
    }
}
