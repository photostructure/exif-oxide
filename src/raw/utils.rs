//! RAW processing utilities and helper functions

/// Reverse a byte string (Kyocera-specific utility)
/// ExifTool: KyoceraRaw.pm ReverseString function
/// Kyocera stores strings in byte-reversed format for unknown reasons
pub fn reverse_string(input: &[u8]) -> String {
    let reversed_bytes: Vec<u8> = input.iter().copied().rev().collect();
    String::from_utf8_lossy(&reversed_bytes)
        .trim_start_matches('\0')
        .trim_end_matches('\0')
        .to_string()
}

/// Calculate exposure time from Kyocera-specific encoding
/// ExifTool: KyoceraRaw.pm ExposureTime calculation
/// Formula: 2^(val/8) / 16000
pub fn kyocera_exposure_time(val: u32) -> f64 {
    if val == 0 {
        return 0.0;
    }
    let exponent = val as f64 / 8.0;
    2_f64.powf(exponent) / 16000.0
}

/// Calculate F-number from Kyocera-specific encoding  
/// ExifTool: KyoceraRaw.pm FNumber calculation
/// Formula: 2^(val/16)
pub fn kyocera_fnumber(val: u32) -> f64 {
    if val == 0 {
        return 0.0;
    }
    let exponent = val as f64 / 16.0;
    2_f64.powf(exponent)
}

/// Convert Kyocera internal ISO values to standard ISO speeds
/// ExifTool: KyoceraRaw.pm ISO tag PrintConv (lines 56-70)
/// Maps internal values 7-19 to ISO speeds 25-400
///
/// Now uses the generated tag kit system with PrintConv lookup
pub fn kyocera_iso_lookup(val: u32) -> Option<u32> {
    // Temporarily disabled due to missing generated code
    // use crate::generated::KyoceraRaw_pm::tag_kit::apply_print_conv;
    // use crate::types::TagValue;

    // Use the tag kit system to apply PrintConv for ISO tag (id=52)
    // let input_value = TagValue::U32(val);
    // let result = apply_print_conv(52, &input_value, &mut errors, &mut warnings);

    // Parse the result back to u32 if it's a valid ISO value
    // if let TagValue::String(iso_str) = result {
    //     if let Ok(iso_value) = iso_str.parse::<u32>() {
    //         return Some(iso_value);
    //     }
    // }

    // Fallback to simple lookup for now
    match val {
        7 => Some(25),
        8 => Some(32),
        9 => Some(40),
        10 => Some(50),
        11 => Some(64),
        12 => Some(80),
        13 => Some(100),
        14 => Some(125),
        15 => Some(160),
        16 => Some(200),
        17 => Some(250),
        18 => Some(320),
        19 => Some(400),
        _ => None,
    }
}

/// Helper function to extract ImageWidth/ImageHeight from any IFD
/// Returns (width, height) if both found, None otherwise
fn extract_dimensions_from_ifd(
    data: &[u8],
    ifd_offset: usize,
    is_little_endian: bool,
) -> Option<(u32, u32)> {
    use tracing::debug;

    // Validate IFD offset
    if ifd_offset >= data.len() || ifd_offset + 2 > data.len() {
        debug!("Invalid IFD offset: 0x{:x}", ifd_offset);
        return None;
    }

    // Read number of IFD entries
    let entry_count = if is_little_endian {
        u16::from_le_bytes([data[ifd_offset], data[ifd_offset + 1]])
    } else {
        u16::from_be_bytes([data[ifd_offset], data[ifd_offset + 1]])
    } as usize;

    debug!("IFD at 0x{:x} contains {} entries", ifd_offset, entry_count);

    // Validate entry count
    let entries_start = ifd_offset + 2;
    let entries_end = entries_start + (entry_count * 12);
    if entries_end > data.len() {
        debug!("IFD entries extend beyond file end");
        return None;
    }

    let mut width: Option<u32> = None;
    let mut height: Option<u32> = None;

    // Scan IFD entries for ImageWidth (0x0100) and ImageHeight (0x0101)
    for i in 0..entry_count {
        let entry_offset = entries_start + (i * 12);
        if entry_offset + 12 > data.len() {
            break;
        }

        // Read IFD entry: tag(2) + type(2) + count(4) + value/offset(4)
        let tag_id = if is_little_endian {
            u16::from_le_bytes([data[entry_offset], data[entry_offset + 1]])
        } else {
            u16::from_be_bytes([data[entry_offset], data[entry_offset + 1]])
        };

        let data_type = if is_little_endian {
            u16::from_le_bytes([data[entry_offset + 2], data[entry_offset + 3]])
        } else {
            u16::from_be_bytes([data[entry_offset + 2], data[entry_offset + 3]])
        };

        let count = if is_little_endian {
            u32::from_le_bytes([
                data[entry_offset + 4],
                data[entry_offset + 5],
                data[entry_offset + 6],
                data[entry_offset + 7],
            ])
        } else {
            u32::from_be_bytes([
                data[entry_offset + 4],
                data[entry_offset + 5],
                data[entry_offset + 6],
                data[entry_offset + 7],
            ])
        };

        match tag_id {
            0x0100 => {
                // ImageWidth
                debug!(
                    "Found ImageWidth in SubIFD: type={}, count={}",
                    data_type, count
                );
                if count == 1 {
                    let value = if is_little_endian {
                        u32::from_le_bytes([
                            data[entry_offset + 8],
                            data[entry_offset + 9],
                            data[entry_offset + 10],
                            data[entry_offset + 11],
                        ])
                    } else {
                        u32::from_be_bytes([
                            data[entry_offset + 8],
                            data[entry_offset + 9],
                            data[entry_offset + 10],
                            data[entry_offset + 11],
                        ])
                    };

                    // Handle both SHORT (3) and LONG (4) types
                    width = match data_type {
                        3 => Some(value & 0xFFFF), // SHORT (16-bit)
                        4 => Some(value),          // LONG (32-bit)
                        _ => Some(value & 0xFFFF), // Default to 16-bit
                    };
                }
            }
            0x0101 => {
                // ImageHeight
                debug!(
                    "Found ImageHeight in SubIFD: type={}, count={}",
                    data_type, count
                );
                if count == 1 {
                    let value = if is_little_endian {
                        u32::from_le_bytes([
                            data[entry_offset + 8],
                            data[entry_offset + 9],
                            data[entry_offset + 10],
                            data[entry_offset + 11],
                        ])
                    } else {
                        u32::from_be_bytes([
                            data[entry_offset + 8],
                            data[entry_offset + 9],
                            data[entry_offset + 10],
                            data[entry_offset + 11],
                        ])
                    };

                    // Handle both SHORT (3) and LONG (4) types
                    height = match data_type {
                        3 => Some(value & 0xFFFF), // SHORT (16-bit)
                        4 => Some(value),          // LONG (32-bit)
                        _ => Some(value & 0xFFFF), // Default to 16-bit
                    };
                }
            }
            _ => {
                // Skip other tags
            }
        }

        // Early exit if we found both dimensions
        if width.is_some() && height.is_some() {
            break;
        }
    }

    if let (Some(w), Some(h)) = (width, height) {
        Some((w, h))
    } else {
        None
    }
}

/// Parse TIFF header to determine byte order and IFD0 offset
/// Returns (is_little_endian, ifd0_offset) or None if invalid
fn parse_tiff_header(data: &[u8]) -> Option<(bool, usize)> {
    use tracing::debug;

    // Validate minimum TIFF header size
    if data.len() < 8 {
        debug!("RAW file too small for TIFF header");
        return None;
    }

    // Read TIFF header to determine byte order and IFD0 offset
    match &data[0..4] {
        [0x49, 0x49, 0x2A, 0x00] => {
            // Little-endian TIFF (II*\0)
            let ifd0_offset = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
            Some((true, ifd0_offset))
        }
        [0x49, 0x49, 0x55, 0x00] => {
            // Panasonic RW2 format (IIU\0) - little-endian like standard TIFF
            // ExifTool: PanasonicRaw.pm - RW2 uses TIFF structure but with different magic
            let ifd0_offset = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
            debug!("Detected Panasonic RW2 format (IIU\\0)");
            Some((true, ifd0_offset))
        }
        [0x4D, 0x4D, 0x00, 0x2A] => {
            // Big-endian TIFF (MM\0*)
            let ifd0_offset = u32::from_be_bytes([data[4], data[5], data[6], data[7]]) as usize;
            Some((false, ifd0_offset))
        }
        _ => {
            debug!("Invalid TIFF magic bytes in RAW file");
            None
        }
    }
}

/// Panasonic RW2 sensor border data for dimension calculation
#[derive(Debug, Default)]
struct SensorBorders {
    top: Option<u16>,
    left: Option<u16>,
    bottom: Option<u16>,
    right: Option<u16>,
}

impl SensorBorders {
    /// Calculate dimensions from sensor borders: width = right - left, height = bottom - top
    /// ExifTool: PanasonicRaw.pm:675-690 (Composite tags)
    fn calculate_dimensions(&self) -> Option<(u32, u32)> {
        if let (Some(left), Some(right), Some(top), Some(bottom)) =
            (self.left, self.right, self.top, self.bottom)
        {
            let width = right as u32 - left as u32;
            let height = bottom as u32 - top as u32;
            Some((width, height))
        } else {
            None
        }
    }
}

/// Parse a single IFD entry and extract value based on data type
/// Returns the value as u32, handling both SHORT (16-bit) and LONG (32-bit) types
fn parse_ifd_entry_value(
    data: &[u8],
    entry_offset: usize,
    data_type: u16,
    is_little_endian: bool,
) -> Option<u32> {
    if entry_offset + 12 > data.len() {
        return None;
    }

    let value = if is_little_endian {
        u32::from_le_bytes([
            data[entry_offset + 8],
            data[entry_offset + 9],
            data[entry_offset + 10],
            data[entry_offset + 11],
        ])
    } else {
        u32::from_be_bytes([
            data[entry_offset + 8],
            data[entry_offset + 9],
            data[entry_offset + 10],
            data[entry_offset + 11],
        ])
    };

    // Handle different data types (SHORT=3, LONG=4)
    match data_type {
        3 => Some(value & 0xFFFF), // SHORT (16-bit)
        4 => Some(value),          // LONG (32-bit)
        _ => Some(value),          // Default handling
    }
}

/// Apply dimension priority logic: prefer larger dimensions over small thumbnails
/// This handles cases where IFD0 has thumbnail dimensions and SubIFD has full resolution
fn apply_dimension_priority(
    mut ifd0_width: Option<u32>,
    mut ifd0_height: Option<u32>,
    sub_width: Option<u32>,
    sub_height: Option<u32>,
) -> (Option<u32>, Option<u32>) {
    use tracing::debug;

    if sub_width.is_some() && sub_height.is_some() {
        let sub_w = sub_width.unwrap();
        let sub_h = sub_height.unwrap();

        if let (Some(ifd0_w), Some(ifd0_h)) = (ifd0_width, ifd0_height) {
            // Both IFD0 and SubIFD have dimensions - choose the larger one
            let ifd0_pixels = ifd0_w as u64 * ifd0_h as u64;
            let sub_pixels = sub_w as u64 * sub_h as u64;

            debug!(
                "Dimension priority: IFD0={}x{} ({} pixels), SubIFD={}x{} ({} pixels)",
                ifd0_w, ifd0_h, ifd0_pixels, sub_w, sub_h, sub_pixels
            );

            if sub_pixels > ifd0_pixels {
                debug!("Using SubIFD dimensions (larger)");
                ifd0_width = Some(sub_w);
                ifd0_height = Some(sub_h);
            } else {
                debug!("Using IFD0 dimensions (larger or equal)");
            }
        } else {
            // Only SubIFD has dimensions, use them
            debug!("Using SubIFD dimensions (IFD0 missing dimensions)");
            ifd0_width = Some(sub_w);
            ifd0_height = Some(sub_h);
        }
    } else if ifd0_width.is_none() || ifd0_height.is_none() {
        // Use SubIFD dimensions if available and IFD0 dimensions are incomplete
        if let Some(sub_w) = sub_width {
            if ifd0_width.is_none() {
                ifd0_width = Some(sub_w);
                debug!("Using SubIFD width (IFD0 missing width)");
            }
        }
        if let Some(sub_h) = sub_height {
            if ifd0_height.is_none() {
                ifd0_height = Some(sub_h);
                debug!("Using SubIFD height (IFD0 missing height)");
            }
        }
    }

    (ifd0_width, ifd0_height)
}

type DimensionScanResult = (Option<u32>, Option<u32>, Option<usize>, SensorBorders);

/// Scan IFD entries for dimension-related data (dimensions, SubIFD pointer, sensor borders)
/// Returns (image_width, image_height, sub_ifd_offset, sensor_borders)
fn scan_ifd_for_dimensions(
    data: &[u8],
    ifd_offset: usize,
    is_little_endian: bool,
) -> crate::types::Result<DimensionScanResult> {
    use tracing::debug;

    // Validate IFD offset
    if ifd_offset >= data.len() || ifd_offset + 2 > data.len() {
        debug!("Invalid IFD offset: 0x{:x}", ifd_offset);
        return Ok((None, None, None, SensorBorders::default()));
    }

    // Read number of IFD entries
    let entry_count = if is_little_endian {
        u16::from_le_bytes([data[ifd_offset], data[ifd_offset + 1]])
    } else {
        u16::from_be_bytes([data[ifd_offset], data[ifd_offset + 1]])
    } as usize;

    debug!("IFD at 0x{:x} contains {} entries", ifd_offset, entry_count);

    // Validate entry count and available data
    let entries_start = ifd_offset + 2;
    let entries_end = entries_start + (entry_count * 12);
    if entries_end > data.len() {
        debug!("IFD entries extend beyond file end");
        return Ok((None, None, None, SensorBorders::default()));
    }

    // Scan IFD entries for relevant tags
    // For Sony ARW, dimensions are often in SubIFD (tag 0x014a) rather than IFD0
    // For Panasonic RW2, dimensions are calculated from sensor border tags (0x04-0x07)
    // ExifTool: Exif.pm tags 0x100 and 0x101 definitions; PanasonicRaw.pm Composite tags
    let mut image_width: Option<u32> = None;
    let mut image_height: Option<u32> = None;
    let mut sub_ifd_offset: Option<usize> = None;
    let mut sensor_borders = SensorBorders::default();

    for i in 0..entry_count {
        let entry_offset = entries_start + (i * 12);
        if entry_offset + 12 > data.len() {
            debug!("IFD entry {} extends beyond file end", i);
            break;
        }

        // Read IFD entry: tag(2) + type(2) + count(4) + value/offset(4)
        let tag_id = if is_little_endian {
            u16::from_le_bytes([data[entry_offset], data[entry_offset + 1]])
        } else {
            u16::from_be_bytes([data[entry_offset], data[entry_offset + 1]])
        };

        let data_type = if is_little_endian {
            u16::from_le_bytes([data[entry_offset + 2], data[entry_offset + 3]])
        } else {
            u16::from_be_bytes([data[entry_offset + 2], data[entry_offset + 3]])
        };

        let count = if is_little_endian {
            u32::from_le_bytes([
                data[entry_offset + 4],
                data[entry_offset + 5],
                data[entry_offset + 6],
                data[entry_offset + 7],
            ])
        } else {
            u32::from_be_bytes([
                data[entry_offset + 4],
                data[entry_offset + 5],
                data[entry_offset + 6],
                data[entry_offset + 7],
            ])
        };

        // Log all tags found in IFD for debugging
        debug!(
            "IFD entry {}: tag_id=0x{:04x}, type={}, count={}",
            i, tag_id, data_type, count
        );

        match tag_id {
            0x0100 => {
                // ImageWidth - ExifTool: Exif.pm:460
                debug!("Found ImageWidth tag: type={}, count={}", data_type, count);
                if count == 1 {
                    if let Some(value) =
                        parse_ifd_entry_value(data, entry_offset, data_type, is_little_endian)
                    {
                        image_width = Some(value);
                        debug!("ImageWidth = {}", value);
                    }
                }
            }
            0x0101 => {
                // ImageHeight (called ImageLength by EXIF spec) - ExifTool: Exif.pm:473
                debug!("Found ImageHeight tag: type={}, count={}", data_type, count);
                if count == 1 {
                    if let Some(value) =
                        parse_ifd_entry_value(data, entry_offset, data_type, is_little_endian)
                    {
                        image_height = Some(value);
                        debug!("ImageHeight = {}", value);
                    }
                }
            }
            0x04 => {
                // SensorTopBorder - Panasonic RW2 (tag 0x04)
                // ExifTool: PanasonicRaw.pm:82
                debug!(
                    "Found SensorTopBorder tag: type={}, count={}",
                    data_type, count
                );
                if count == 1 && data_type == 3 {
                    // Should be int16u type (3)
                    let value = if is_little_endian {
                        u16::from_le_bytes([data[entry_offset + 8], data[entry_offset + 9]])
                    } else {
                        u16::from_be_bytes([data[entry_offset + 8], data[entry_offset + 9]])
                    };
                    sensor_borders.top = Some(value);
                    debug!("SensorTopBorder = {}", value);
                }
            }
            0x05 => {
                // SensorLeftBorder - Panasonic RW2 (tag 0x05)
                // ExifTool: PanasonicRaw.pm:83
                debug!(
                    "Found SensorLeftBorder tag: type={}, count={}",
                    data_type, count
                );
                if count == 1 && data_type == 3 {
                    let value = if is_little_endian {
                        u16::from_le_bytes([data[entry_offset + 8], data[entry_offset + 9]])
                    } else {
                        u16::from_be_bytes([data[entry_offset + 8], data[entry_offset + 9]])
                    };
                    sensor_borders.left = Some(value);
                    debug!("SensorLeftBorder = {}", value);
                }
            }
            0x06 => {
                // SensorBottomBorder - Panasonic RW2 (tag 0x06)
                // ExifTool: PanasonicRaw.pm:84
                debug!(
                    "Found SensorBottomBorder tag: type={}, count={}",
                    data_type, count
                );
                if count == 1 && data_type == 3 {
                    let value = if is_little_endian {
                        u16::from_le_bytes([data[entry_offset + 8], data[entry_offset + 9]])
                    } else {
                        u16::from_be_bytes([data[entry_offset + 8], data[entry_offset + 9]])
                    };
                    sensor_borders.bottom = Some(value);
                    debug!("SensorBottomBorder = {}", value);
                }
            }
            0x07 => {
                // SensorRightBorder - Panasonic RW2 (tag 0x07)
                // ExifTool: PanasonicRaw.pm:85
                debug!(
                    "Found SensorRightBorder tag: type={}, count={}",
                    data_type, count
                );
                if count == 1 && data_type == 3 {
                    let value = if is_little_endian {
                        u16::from_le_bytes([data[entry_offset + 8], data[entry_offset + 9]])
                    } else {
                        u16::from_be_bytes([data[entry_offset + 8], data[entry_offset + 9]])
                    };
                    sensor_borders.right = Some(value);
                    debug!("SensorRightBorder = {}", value);
                }
            }
            0x014a => {
                // SubIFD pointer - Sony ARW stores dimensions here, DNG may have multiple SubIFDs
                debug!("Found SubIFD tag: type={}, count={}", data_type, count);

                if data_type == 4 && count >= 1 {
                    // LONG pointer(s)
                    let offset = if count == 1 {
                        // Single SubIFD pointer stored directly in value field
                        parse_ifd_entry_value(data, entry_offset, data_type, is_little_endian)
                            .map(|v| v as usize)
                    } else {
                        // Multiple SubIFD pointers - value field points to array, read first offset
                        let array_offset =
                            parse_ifd_entry_value(data, entry_offset, data_type, is_little_endian)
                                .map(|v| v as usize);

                        if let Some(array_offset) = array_offset {
                            // Read first SubIFD pointer from array
                            if array_offset + 4 <= data.len() {
                                let first_offset = if is_little_endian {
                                    u32::from_le_bytes([
                                        data[array_offset],
                                        data[array_offset + 1],
                                        data[array_offset + 2],
                                        data[array_offset + 3],
                                    ])
                                } else {
                                    u32::from_be_bytes([
                                        data[array_offset],
                                        data[array_offset + 1],
                                        data[array_offset + 2],
                                        data[array_offset + 3],
                                    ])
                                } as usize;
                                Some(first_offset)
                            } else {
                                debug!("SubIFD array offset 0x{:x} out of bounds", array_offset);
                                None
                            }
                        } else {
                            None
                        }
                    };

                    if let Some(offset) = offset {
                        if offset > 0 {
                            debug!(
                                "SubIFD pointer at offset 0x{:x} (first of {} SubIFDs)",
                                offset, count
                            );
                            sub_ifd_offset = Some(offset);
                        }
                    }
                }
            }
            _ => {
                // Skip other tags - we only need dimensions and SubIFD
            }
        }

        // Note: Don't exit early even if dimensions found in IFD0 - we need to scan for SubIFD
        // which may contain full-resolution dimensions (IFD0 might only have thumbnails)
    }

    Ok((image_width, image_height, sub_ifd_offset, sensor_borders))
}

/// Extract all SubIFD pointers from a TIFF file following ExifTool's algorithm
/// Returns a vector of SubIFD offsets in the order they appear in the SubIFD tag
/// ExifTool: Exif.pm handles up to 10 SubIFDs (MaxSubdirs => 10)
fn extract_all_subifd_pointers(
    data: &[u8],
    ifd0_offset: usize,
    is_little_endian: bool,
) -> crate::types::Result<Vec<usize>> {
    use tracing::debug;

    // Validate IFD offset
    if ifd0_offset >= data.len() || ifd0_offset + 2 > data.len() {
        debug!(
            "Invalid IFD0 offset for SubIFD extraction: 0x{:x}",
            ifd0_offset
        );
        return Ok(Vec::new());
    }

    // Read number of IFD entries
    let entry_count = if is_little_endian {
        u16::from_le_bytes([data[ifd0_offset], data[ifd0_offset + 1]])
    } else {
        u16::from_be_bytes([data[ifd0_offset], data[ifd0_offset + 1]])
    } as usize;

    let entries_start = ifd0_offset + 2;
    let entries_end = entries_start + (entry_count * 12);

    if entries_end > data.len() {
        debug!("IFD0 entries extend beyond file end during SubIFD extraction");
        return Ok(Vec::new());
    }

    // Scan IFD0 entries for SubIFD tag (0x014a)
    for i in 0..entry_count {
        let entry_offset = entries_start + (i * 12);
        if entry_offset + 12 > data.len() {
            break;
        }

        // Read IFD entry: tag(2) + type(2) + count(4) + value/offset(4)
        let tag_id = if is_little_endian {
            u16::from_le_bytes([data[entry_offset], data[entry_offset + 1]])
        } else {
            u16::from_be_bytes([data[entry_offset], data[entry_offset + 1]])
        };

        if tag_id == 0x014a {
            // Found SubIFD tag
            let data_type = if is_little_endian {
                u16::from_le_bytes([data[entry_offset + 2], data[entry_offset + 3]])
            } else {
                u16::from_be_bytes([data[entry_offset + 2], data[entry_offset + 3]])
            };

            let count = if is_little_endian {
                u32::from_le_bytes([
                    data[entry_offset + 4],
                    data[entry_offset + 5],
                    data[entry_offset + 6],
                    data[entry_offset + 7],
                ])
            } else {
                u32::from_be_bytes([
                    data[entry_offset + 4],
                    data[entry_offset + 5],
                    data[entry_offset + 6],
                    data[entry_offset + 7],
                ])
            };

            debug!("Found SubIFD tag: type={}, count={}", data_type, count);

            if data_type == 4 && count >= 1 {
                // LONG pointer(s) - ExifTool supports up to MaxSubdirs=10
                let max_subdirs = 10_u32.min(count); // Limit to ExifTool's MaxSubdirs
                let mut subifd_offsets = Vec::new();

                if count == 1 {
                    // Single SubIFD pointer stored directly in value field
                    let offset = if is_little_endian {
                        u32::from_le_bytes([
                            data[entry_offset + 8],
                            data[entry_offset + 9],
                            data[entry_offset + 10],
                            data[entry_offset + 11],
                        ])
                    } else {
                        u32::from_be_bytes([
                            data[entry_offset + 8],
                            data[entry_offset + 9],
                            data[entry_offset + 10],
                            data[entry_offset + 11],
                        ])
                    } as usize;

                    if offset > 0 {
                        subifd_offsets.push(offset);
                    }
                } else {
                    // Multiple SubIFD pointers - value field points to array
                    let array_offset = if is_little_endian {
                        u32::from_le_bytes([
                            data[entry_offset + 8],
                            data[entry_offset + 9],
                            data[entry_offset + 10],
                            data[entry_offset + 11],
                        ])
                    } else {
                        u32::from_be_bytes([
                            data[entry_offset + 8],
                            data[entry_offset + 9],
                            data[entry_offset + 10],
                            data[entry_offset + 11],
                        ])
                    } as usize;

                    // Read all SubIFD pointers from array (each is 4 bytes)
                    for j in 0..max_subdirs {
                        let pointer_offset = array_offset + (j as usize * 4);

                        if pointer_offset + 4 <= data.len() {
                            let subifd_offset = if is_little_endian {
                                u32::from_le_bytes([
                                    data[pointer_offset],
                                    data[pointer_offset + 1],
                                    data[pointer_offset + 2],
                                    data[pointer_offset + 3],
                                ])
                            } else {
                                u32::from_be_bytes([
                                    data[pointer_offset],
                                    data[pointer_offset + 1],
                                    data[pointer_offset + 2],
                                    data[pointer_offset + 3],
                                ])
                            } as usize;

                            if subifd_offset > 0 {
                                subifd_offsets.push(subifd_offset);
                            }
                        } else {
                            debug!(
                                "SubIFD pointer {} at offset 0x{:x} out of bounds",
                                j, pointer_offset
                            );
                            break;
                        }
                    }
                }

                debug!(
                    "Extracted {} SubIFD pointers: {:?}",
                    subifd_offsets.len(),
                    subifd_offsets
                );
                return Ok(subifd_offsets);
            }
        }
    }

    debug!("No SubIFD tag found in IFD0");
    Ok(Vec::new())
}

/// Extract TIFF dimension tags (ImageWidth/ImageHeight) from IFD0 for TIFF-based RAW files
/// ExifTool: lib/Image/ExifTool/Exif.pm:351-473 (tags 0x0100, 0x0101)
/// Used by both Sony ARW and Canon CR2 files which are TIFF-based
pub fn extract_tiff_dimensions(
    reader: &mut crate::exif::ExifReader,
    data: &[u8],
) -> crate::types::Result<()> {
    use crate::types::TagValue;
    use tracing::debug;

    debug!("extract_tiff_dimensions: Starting TIFF dimension extraction from RAW file");

    // Parse TIFF header
    let (is_little_endian, ifd0_offset) = match parse_tiff_header(data) {
        Some(header) => header,
        None => return Ok(()),
    };

    debug!(
        "RAW TIFF format: {} endian, IFD0 at offset 0x{:x}",
        if is_little_endian { "little" } else { "big" },
        ifd0_offset
    );

    // Scan IFD0 for dimension data
    let (image_width, image_height, sub_ifd_offset, sensor_borders) =
        scan_ifd_for_dimensions(data, ifd0_offset, is_little_endian)?;

    // Check all SubIFDs if available for full-resolution dimensions
    // ExifTool processes all SubIFDs sequentially (SubIFD0, SubIFD1, etc.)
    // NEF files often store dimensions in SubIFD1, not SubIFD0
    let (sub_width, sub_height) = if let Some(_first_sub_offset) = sub_ifd_offset {
        // Extract all SubIFD pointers from the SubIFD tag (0x014a)
        let all_sub_offsets = extract_all_subifd_pointers(data, ifd0_offset, is_little_endian)?;

        debug!(
            "Found {} SubIFD pointers: {:?}",
            all_sub_offsets.len(),
            all_sub_offsets
        );

        // Iterate through all SubIFDs looking for dimensions (matching ExifTool's algorithm)
        let mut found_width = None;
        let mut found_height = None;

        for (index, sub_offset) in all_sub_offsets.iter().enumerate() {
            debug!(
                "Checking SubIFD{} at offset 0x{:x} for dimensions",
                index, sub_offset
            );

            if let Some((width, height)) =
                extract_dimensions_from_ifd(data, *sub_offset, is_little_endian)
            {
                debug!("Found dimensions in SubIFD{}: {}x{}", index, width, height);
                found_width = Some(width);
                found_height = Some(height);
                break; // Use first SubIFD that has both dimensions
            }
        }

        (found_width, found_height)
    } else {
        (None, None)
    };

    // Apply dimension priority logic: prefer larger dimensions over small thumbnails
    let (mut image_width, mut image_height) =
        apply_dimension_priority(image_width, image_height, sub_width, sub_height);

    // Calculate Panasonic RW2 dimensions from sensor border tags if available
    if let Some((panasonic_width, panasonic_height)) = sensor_borders.calculate_dimensions() {
        debug!(
            "Calculated Panasonic RW2 dimensions: {}x{} (from sensor borders)",
            panasonic_width, panasonic_height
        );

        // Use Panasonic dimensions if standard TIFF dimensions not found
        if image_width.is_none() {
            image_width = Some(panasonic_width);
        }
        if image_height.is_none() {
            image_height = Some(panasonic_height);
        }
    }

    // Add extracted sensor border tags to reader as individual tags
    // These are needed for Panasonic RW2 files - ExifTool: PanasonicRaw.pm:82-85
    if let Some(value) = sensor_borders.top {
        reader.legacy_insert_tag(0x04, TagValue::U16(value), "EXIF");
        debug!("Added SensorTopBorder (0x04) = {}", value);
    }
    if let Some(value) = sensor_borders.left {
        reader.legacy_insert_tag(0x05, TagValue::U16(value), "EXIF");
        debug!("Added SensorLeftBorder (0x05) = {}", value);
    }
    if let Some(value) = sensor_borders.bottom {
        reader.legacy_insert_tag(0x06, TagValue::U16(value), "EXIF");
        debug!("Added SensorBottomBorder (0x06) = {}", value);
    }
    if let Some(value) = sensor_borders.right {
        reader.legacy_insert_tag(0x07, TagValue::U16(value), "EXIF");
        debug!("Added SensorRightBorder (0x07) = {}", value);
    }

    // Add extracted dimensions to reader as EXIF tags
    // Note: File: group tags are handled at a higher level in formats/mod.rs
    // Here we add them as standard EXIF tags following ExifTool's approach
    //
    // However, for RW2 files, we don't add sensor border dimensions as EXIF tags
    // because File:ImageWidth/ImageHeight comes from JPEG preview, not sensor borders
    // The sensor border dimensions become Composite:ImageWidth/ImageHeight instead
    let file_type = reader.get_original_file_type().unwrap_or_default();
    let skip_exif_dimensions =
        file_type == "RW2" && sensor_borders.calculate_dimensions().is_some();

    if !skip_exif_dimensions {
        if let Some(width) = image_width {
            // Add ImageWidth tag (0x0100) - ExifTool: Exif.pm:460
            reader.legacy_insert_tag(0x0100, TagValue::U32(width), "EXIF");
            debug!("Added EXIF:ImageWidth (0x0100) = {}", width);
        }

        if let Some(height) = image_height {
            // Add ImageHeight tag (0x0101) - ExifTool: Exif.pm:473
            reader.legacy_insert_tag(0x0101, TagValue::U32(height), "EXIF");
            debug!("Added EXIF:ImageHeight (0x0101) = {}", height);
        }
    } else {
        debug!("Skipping EXIF dimensions for RW2 file - using File group dimensions from JPEG preview instead");
    }

    if image_width.is_none() || image_height.is_none() {
        debug!("Warning: Could not extract both image dimensions from TIFF structure");
    }

    Ok(())
}
