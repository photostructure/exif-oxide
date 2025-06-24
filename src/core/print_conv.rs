//! Print conversion functions for EXIF tag values
//!
//! This module provides a table-driven system for converting raw EXIF values
//! into human-readable strings, matching ExifTool's PrintConv functionality.
//! Instead of porting thousands of lines of Perl conversion code, we identify
//! common patterns and create reusable conversion functions.

use crate::core::ExifValue;

/// Enumeration of all print conversion functions
///
/// This enum captures all unique PrintConv patterns found across ExifTool's
/// manufacturer modules. By cataloging these patterns, we can reuse conversion
/// logic across all manufacturers instead of duplicating it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrintConvId {
    /// No conversion - return raw value as string
    None,

    /// Simple on/off conversion (0=Off, 1=On)
    OnOff,

    /// Yes/No conversion (0=No, 1=Yes)  
    YesNo,

    /// Image size conversion (width x height)
    ImageSize,

    /// Quality settings (1=Best, 2=Better, 3=Good, etc.)
    Quality,

    /// Flash mode lookup
    FlashMode,

    /// Focus mode lookup  
    FocusMode,

    /// White balance lookup
    WhiteBalance,

    /// Metering mode lookup
    MeteringMode,

    /// ISO speed conversion
    IsoSpeed,

    /// Exposure compensation (+/- EV)
    ExposureCompensation,

    /// Pentax-specific conversions
    PentaxModelLookup,
    PentaxPictureMode,
    PentaxLensType,

    /// Canon-specific conversions  
    CanonCameraSettings,
    CanonImageType,

    /// Nikon-specific conversions
    NikonLensType,
    NikonFlashMode,

    /// Sony-specific conversions
    SonyLensType,
    SonySceneMode,
    // More can be added as needed...
}

/// Apply print conversion to an EXIF value
pub fn apply_print_conv(value: &ExifValue, conv_id: PrintConvId) -> String {
    match conv_id {
        PrintConvId::None => exif_value_to_string(value),

        PrintConvId::OnOff => match as_u32(value) {
            Some(0) => "Off".to_string(),
            Some(1) => "On".to_string(),
            _ => exif_value_to_string(value),
        },

        PrintConvId::YesNo => match as_u32(value) {
            Some(0) => "No".to_string(),
            Some(1) => "Yes".to_string(),
            _ => exif_value_to_string(value),
        },

        PrintConvId::ImageSize => {
            // Handle different value formats for image size
            match value {
                ExifValue::U16Array(arr) if arr.len() >= 2 => {
                    format!("{}x{}", arr[0], arr[1])
                }
                ExifValue::U32Array(arr) if arr.len() >= 2 => {
                    format!("{}x{}", arr[0], arr[1])
                }
                _ => exif_value_to_string(value),
            }
        }

        PrintConvId::Quality => match as_u32(value) {
            Some(1) => "Best".to_string(),
            Some(2) => "Better".to_string(),
            Some(3) => "Good".to_string(),
            Some(4) => "Normal".to_string(),
            Some(5) => "Economy".to_string(),
            _ => exif_value_to_string(value),
        },

        PrintConvId::FlashMode => match as_u32(value) {
            Some(0) => "Auto".to_string(),
            Some(1) => "On".to_string(),
            Some(2) => "Off".to_string(),
            Some(3) => "Red-eye reduction".to_string(),
            _ => exif_value_to_string(value),
        },

        PrintConvId::FocusMode => match as_u32(value) {
            Some(0) => "Auto".to_string(),
            Some(1) => "Manual".to_string(),
            Some(2) => "Macro".to_string(),
            Some(3) => "Single".to_string(),
            Some(4) => "Continuous".to_string(),
            _ => exif_value_to_string(value),
        },

        PrintConvId::WhiteBalance => match as_u32(value) {
            Some(0) => "Auto".to_string(),
            Some(1) => "Daylight".to_string(),
            Some(2) => "Shade".to_string(),
            Some(3) => "Fluorescent".to_string(),
            Some(4) => "Tungsten".to_string(),
            Some(5) => "Manual".to_string(),
            _ => exif_value_to_string(value),
        },

        PrintConvId::MeteringMode => match as_u32(value) {
            Some(0) => "Multi-segment".to_string(),
            Some(1) => "Center-weighted".to_string(),
            Some(2) => "Spot".to_string(),
            _ => exif_value_to_string(value),
        },

        PrintConvId::IsoSpeed => {
            // Handle various ISO representations
            match as_u32(value) {
                Some(n) if n < 50 => format!("ISO {}", 50 << n), // Power of 2 encoding
                Some(n) => format!("ISO {}", n),                 // Direct value
                _ => exif_value_to_string(value),
            }
        }

        PrintConvId::ExposureCompensation => {
            match as_i32(value) {
                Some(n) => {
                    let ev = n as f32 / 3.0; // Common 1/3 EV steps
                    if ev >= 0.0 {
                        format!("+{:.1} EV", ev)
                    } else {
                        format!("{:.1} EV", ev)
                    }
                }
                _ => exif_value_to_string(value),
            }
        }

        // Pentax-specific conversions
        PrintConvId::PentaxModelLookup => pentax_model_lookup(value),
        PrintConvId::PentaxPictureMode => pentax_picture_mode(value),
        PrintConvId::PentaxLensType => pentax_lens_type(value),

        // Manufacturer-specific conversions will be implemented as needed
        _ => {
            // For now, return raw value for unimplemented conversions
            // TODO: Implement remaining conversion functions
            exif_value_to_string(value)
        }
    }
}

/// Helper to extract u32 value from ExifValue
fn as_u32(value: &ExifValue) -> Option<u32> {
    match value {
        ExifValue::U32(n) => Some(*n),
        ExifValue::U16(n) => Some(*n as u32),
        ExifValue::U8(n) => Some(*n as u32),
        ExifValue::I32(n) if *n >= 0 => Some(*n as u32),
        ExifValue::I16(n) if *n >= 0 => Some(*n as u32),
        _ => None,
    }
}

/// Helper to extract i32 value from ExifValue
fn as_i32(value: &ExifValue) -> Option<i32> {
    match value {
        ExifValue::I32(n) => Some(*n),
        ExifValue::I16(n) => Some(*n as i32),
        ExifValue::U32(n) if *n <= i32::MAX as u32 => Some(*n as i32),
        ExifValue::U16(n) => Some(*n as i32),
        ExifValue::U8(n) => Some(*n as i32),
        _ => None,
    }
}

/// Convert ExifValue to a simple string representation
fn exif_value_to_string(value: &ExifValue) -> String {
    match value {
        ExifValue::Ascii(s) => s.clone(),
        ExifValue::U8(n) => n.to_string(),
        ExifValue::U8Array(arr) => format!("{:?}", arr),
        ExifValue::U16(n) => n.to_string(),
        ExifValue::U16Array(arr) => format!("{:?}", arr),
        ExifValue::U32(n) => n.to_string(),
        ExifValue::U32Array(arr) => format!("{:?}", arr),
        ExifValue::I16(n) => n.to_string(),
        ExifValue::I16Array(arr) => format!("{:?}", arr),
        ExifValue::I32(n) => n.to_string(),
        ExifValue::I32Array(arr) => format!("{:?}", arr),
        ExifValue::Rational(num, den) => {
            if *den == 1 {
                num.to_string()
            } else {
                format!("{}/{}", num, den)
            }
        }
        ExifValue::RationalArray(arr) => {
            let strs: Vec<String> = arr
                .iter()
                .map(|(num, den)| {
                    if *den == 1 {
                        num.to_string()
                    } else {
                        format!("{}/{}", num, den)
                    }
                })
                .collect();
            format!("[{}]", strs.join(", "))
        }
        ExifValue::SignedRational(num, den) => {
            if *den == 1 {
                num.to_string()
            } else {
                format!("{}/{}", num, den)
            }
        }
        ExifValue::SignedRationalArray(arr) => {
            let strs: Vec<String> = arr
                .iter()
                .map(|(num, den)| {
                    if *den == 1 {
                        num.to_string()
                    } else {
                        format!("{}/{}", num, den)
                    }
                })
                .collect();
            format!("[{}]", strs.join(", "))
        }
        ExifValue::Undefined(data) => format!("Undefined({})", data.len()),
        ExifValue::BinaryData(len) => format!("BinaryData({})", len),
    }
}

/// Pentax model lookup conversion
fn pentax_model_lookup(value: &ExifValue) -> String {
    // Simplified version - in practice this would be a large lookup table
    // generated from ExifTool's %pentaxModelType hash
    match as_u32(value) {
        Some(0x12926) => "Optio 330/430".to_string(),
        Some(0x12958) => "Optio 230".to_string(),
        Some(0x12962) => "Optio 330GS".to_string(),
        Some(0x1296c) => "Optio 450/550".to_string(),
        Some(0x12971) => "*ist D".to_string(),
        Some(0x12994) => "*ist DS".to_string(),
        Some(0x129b2) => "Optio S".to_string(),
        Some(0x129bc) => "Optio S V1.01".to_string(),
        // ... would contain hundreds more entries from ExifTool
        _ => format!("Unknown ({})", exif_value_to_string(value)),
    }
}

/// Pentax picture mode conversion
fn pentax_picture_mode(value: &ExifValue) -> String {
    match as_u32(value) {
        Some(0) => "Auto".to_string(),
        Some(1) => "Night-scene".to_string(),
        Some(2) => "Manual".to_string(),
        Some(3) => "Multiple-exposure".to_string(),
        Some(5) => "Portrait".to_string(),
        Some(6) => "Landscape".to_string(),
        Some(8) => "Sport".to_string(),
        Some(9) => "Macro".to_string(),
        Some(11) => "Soft".to_string(),
        Some(12) => "Surf & Snow".to_string(),
        Some(13) => "Sunset or Candlelight".to_string(),
        Some(14) => "Autumn".to_string(),
        Some(15) => "Fireworks".to_string(),
        Some(17) => "Dynamic (Enhanced)".to_string(),
        Some(18) => "Objects in Motion".to_string(),
        Some(19) => "Text".to_string(),
        _ => format!("Unknown ({})", exif_value_to_string(value)),
    }
}

/// Pentax lens type conversion  
fn pentax_lens_type(value: &ExifValue) -> String {
    // This would be generated from ExifTool's %pentaxLensTypes hash
    // For now, a simplified version
    match exif_value_to_string(value).as_str() {
        "0 0" => "M-42 or No Lens".to_string(),
        "1 0" => "K or M Lens".to_string(),
        "2 0" => "A Series Lens".to_string(),
        "3 0" => "Sigma".to_string(),
        _ => format!("Unknown ({})", exif_value_to_string(value)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_on_off_conversion() {
        assert_eq!(
            apply_print_conv(&ExifValue::U32(0), PrintConvId::OnOff),
            "Off"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(1), PrintConvId::OnOff),
            "On"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(2), PrintConvId::OnOff),
            "2"
        );
    }

    #[test]
    fn test_image_size_conversion() {
        let size = ExifValue::U16Array(vec![1920, 1080]);
        assert_eq!(apply_print_conv(&size, PrintConvId::ImageSize), "1920x1080");
    }

    #[test]
    fn test_pentax_picture_mode() {
        assert_eq!(
            apply_print_conv(&ExifValue::U32(0), PrintConvId::PentaxPictureMode),
            "Auto"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(5), PrintConvId::PentaxPictureMode),
            "Portrait"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(999), PrintConvId::PentaxPictureMode),
            "Unknown (999)"
        );
    }

    #[test]
    fn test_no_conversion() {
        assert_eq!(
            apply_print_conv(&ExifValue::U32(42), PrintConvId::None),
            "42"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::Ascii("test".to_string()), PrintConvId::None),
            "test"
        );
    }
}
