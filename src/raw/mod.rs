//! RAW Image Format Support Module
//!
//! This module implements comprehensive RAW metadata extraction for all mainstream manufacturers.
//! It provides a unified interface for processing manufacturer-specific RAW formats while
//! following the Trust ExifTool principle exactly.
//!
//! ## Supported Formats (Milestone 17 Implementation)
//!
//! - **Kyocera** (.raw) - Milestone 17a: Simple ProcessBinaryData format (173 lines)
//! - **Canon** (CR2, CR3) - Future: Complex TIFF-based with maker notes
//! - **Nikon** (NEF, NRW) - Future: Integration with existing Nikon implementation
//! - **Sony** (ARW, SR2, SRF) - Future: Advanced offset management
//! - **Olympus** (ORF) - Future: Multiple IFD navigation
//! - **Panasonic** (RW2) - Future: Entry-based offset handling
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
pub use formats::kyocera::get_kyocera_tag_name;
pub use formats::minolta::get_minolta_tag_name;
pub use formats::olympus::get_olympus_tag_name;
pub use formats::panasonic::get_panasonic_tag_name;

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
