//! RAW Image Format Support Module
//!
//! This module implements comprehensive RAW metadata extraction for all mainstream manufacturers.
//! It provides a unified interface for processing manufacturer-specific RAW formats while
//! following the Trust ExifTool principle exactly.
//!
//! ## Supported Formats (Milestone 17 Implementation)
//!
//! - **Kyocera** (.raw) - Milestone 17a: Simple ProcessBinaryData format (173 lines)
//! - **Minolta** (MRW) - Milestone 17b: Multi-block format with TTW, PRD, WBG blocks
//! - **Panasonic** (RW2) - Milestone 17b: Entry-based offset handling
//! - **Olympus** (ORF) - Milestone 17c: Multiple IFD navigation
//! - **Canon** (CR2, CRW, CR3) - Milestone 17d: Complex TIFF-based with 169 ProcessBinaryData sections
//! - **Nikon** (NEF, NRW) - Future: Integration with existing Nikon implementation
//! - **Sony** (ARW, SR2, SRF) - Future: Advanced offset management
//! - **Fujifilm** (RAF) - Future: Non-TIFF format
//!
//! ## Architecture
//!
//! The RAW processing system uses a hybrid approach:
//! - **Format Detection**: FileTypeDetector identifies RAW file types
//! - **RAW Processor**: Central dispatcher routes to manufacturer handlers
//! - **Handler Traits**: Manufacturer-specific processing implementations
//! - **TIFF Foundation**: Leverages existing TIFF infrastructure for TIFF-based formats
//!
//! ## Trust ExifTool Compliance
//!
//! All implementations strictly follow ExifTool's processing logic:
//! - Exact offset calculations and data parsing
//! - Identical tag naming and grouping
//! - Preserved quirks and manufacturer-specific handling
//! - No "improvements" or "optimizations" to the original logic

pub mod detector;
pub mod offset;
pub mod processor;

// Re-export main types for convenience
pub use detector::{detect_raw_format, RawFormat};
pub use offset::{EntryBasedOffsetProcessor, OffsetContext, SimpleOffsetProcessor};
pub use processor::{RawFormatHandler, RawProcessor};

// Import format-specific handlers (will expand as we add more formats)
pub mod formats;

// Re-export format handlers and utility functions
pub use formats::canon::get_canon_tag_name;
pub use formats::kyocera::get_kyocera_tag_name;
pub use formats::minolta::get_minolta_tag_name;
pub use formats::olympus::get_olympus_tag_name;
pub use formats::panasonic::get_panasonic_tag_name;
pub use formats::sony::get_sony_tag_name;

pub mod utils {
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
    /// ExifTool: KyoceraRaw.pm %isoLookup hash
    /// Maps internal values 7-19 to ISO speeds 25-400
    pub fn kyocera_iso_lookup(val: u32) -> Option<u32> {
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

        // Validate minimum TIFF header size
        if data.len() < 8 {
            debug!("RAW file too small for TIFF header");
            return Ok(());
        }

        // Read TIFF header to determine byte order and IFD0 offset
        let (is_little_endian, ifd0_offset) = match &data[0..4] {
            [0x49, 0x49, 0x2A, 0x00] => {
                // Little-endian TIFF
                if data.len() < 8 {
                    debug!("RAW file too small for IFD0 offset");
                    return Ok(());
                }
                let ifd0_offset = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
                (true, ifd0_offset)
            }
            [0x4D, 0x4D, 0x00, 0x2A] => {
                // Big-endian TIFF
                if data.len() < 8 {
                    debug!("RAW file too small for IFD0 offset");
                    return Ok(());
                }
                let ifd0_offset = u32::from_be_bytes([data[4], data[5], data[6], data[7]]) as usize;
                (false, ifd0_offset)
            }
            _ => {
                debug!("Invalid TIFF magic bytes in RAW file");
                return Ok(());
            }
        };

        debug!(
            "RAW TIFF format: {} endian, IFD0 at offset 0x{:x}",
            if is_little_endian { "little" } else { "big" },
            ifd0_offset
        );

        // Validate IFD0 offset
        if ifd0_offset >= data.len() || ifd0_offset + 2 > data.len() {
            debug!("Invalid IFD0 offset: 0x{:x}", ifd0_offset);
            return Ok(());
        }

        // Read number of IFD entries
        let entry_count = if is_little_endian {
            u16::from_le_bytes([data[ifd0_offset], data[ifd0_offset + 1]])
        } else {
            u16::from_be_bytes([data[ifd0_offset], data[ifd0_offset + 1]])
        } as usize;

        debug!("IFD0 contains {} entries", entry_count);

        // Validate entry count and available data
        let entries_start = ifd0_offset + 2;
        let entries_end = entries_start + (entry_count * 12);
        if entries_end > data.len() {
            debug!("IFD0 entries extend beyond file end");
            return Ok(());
        }

        // Scan IFD0 entries for ImageWidth (0x0100) and ImageHeight (0x0101)
        // ExifTool: Exif.pm tags 0x100 and 0x101 definitions
        let mut image_width: Option<u32> = None;
        let mut image_height: Option<u32> = None;

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

            match tag_id {
                0x0100 => {
                    // ImageWidth - ExifTool: Exif.pm:460
                    debug!("Found ImageWidth tag: type={}, count={}", data_type, count);

                    // Read value based on data type and count
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

                        // Handle different data types (SHORT=3, LONG=4)
                        image_width = match data_type {
                            3 => Some(value & 0xFFFF), // SHORT (16-bit)
                            4 => Some(value),          // LONG (32-bit)
                            _ => {
                                debug!("Unexpected ImageWidth data type: {}", data_type);
                                Some(value)
                            }
                        };

                        debug!("ImageWidth = {}", image_width.unwrap());
                    }
                }
                0x0101 => {
                    // ImageHeight (called ImageLength by EXIF spec) - ExifTool: Exif.pm:473
                    debug!("Found ImageHeight tag: type={}, count={}", data_type, count);

                    // Read value based on data type and count
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

                        // Handle different data types (SHORT=3, LONG=4)
                        image_height = match data_type {
                            3 => Some(value & 0xFFFF), // SHORT (16-bit)
                            4 => Some(value),          // LONG (32-bit)
                            _ => {
                                debug!("Unexpected ImageHeight data type: {}", data_type);
                                Some(value)
                            }
                        };

                        debug!("ImageHeight = {}", image_height.unwrap());
                    }
                }
                _ => {
                    // Skip other tags - we only need dimensions
                }
            }

            // Early exit if we found both dimensions
            if image_width.is_some() && image_height.is_some() {
                break;
            }
        }

        // Add extracted dimensions to reader as EXIF tags
        // Note: File: group tags are handled at a higher level in formats/mod.rs
        // Here we add them as standard EXIF tags following ExifTool's approach
        if let Some(width) = image_width {
            // Add ImageWidth tag (0x0100) - ExifTool: Exif.pm:460
            reader
                .extracted_tags
                .insert(0x0100, TagValue::String(width.to_string()));
            debug!("Added EXIF:ImageWidth (0x0100) = {}", width);
        }

        if let Some(height) = image_height {
            // Add ImageHeight tag (0x0101) - ExifTool: Exif.pm:473
            reader
                .extracted_tags
                .insert(0x0101, TagValue::String(height.to_string()));
            debug!("Added EXIF:ImageHeight (0x0101) = {}", height);
        }

        if image_width.is_none() || image_height.is_none() {
            debug!("Warning: Could not extract both image dimensions from TIFF structure");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::utils::*;

    #[test]
    fn test_reverse_string() {
        let input = b"KYOCERA\0";
        let result = reverse_string(input);
        assert_eq!(result, "ARECOYK");

        // Test with null termination
        let input2 = b"TEST\0\0\0";
        let result2 = reverse_string(input2);
        assert_eq!(result2, "TSET");
    }

    #[test]
    fn test_kyocera_exposure_time() {
        // Test some known values
        assert!((kyocera_exposure_time(0) - 0.0).abs() < f64::EPSILON);

        // Test calculation: 2^(val/8) / 16000
        let val = 64; // Should be 2^8 / 16000 = 256 / 16000 = 0.016
        let expected = 2_f64.powf(64.0 / 8.0) / 16000.0;
        assert!((kyocera_exposure_time(val) - expected).abs() < 0.0001);
    }

    #[test]
    fn test_kyocera_fnumber() {
        // Test some known values
        assert!((kyocera_fnumber(0) - 0.0).abs() < f64::EPSILON);

        // Test calculation: 2^(val/16)
        let val = 32; // Should be 2^2 = 4.0
        let expected = 2_f64.powf(32.0 / 16.0);
        assert!((kyocera_fnumber(val) - expected).abs() < 0.0001);
    }

    #[test]
    fn test_kyocera_iso_lookup() {
        // Test known mappings
        assert_eq!(kyocera_iso_lookup(7), Some(25));
        assert_eq!(kyocera_iso_lookup(13), Some(100));
        assert_eq!(kyocera_iso_lookup(19), Some(400));

        // Test invalid values
        assert_eq!(kyocera_iso_lookup(6), None);
        assert_eq!(kyocera_iso_lookup(20), None);
        assert_eq!(kyocera_iso_lookup(100), None);
    }
}
