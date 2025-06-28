//! PrintConv pattern matching tables
//!
//! This module provides pattern matching tables to map Perl PrintConv patterns
//! to Rust PrintConvId enum variants. Used by the sync tool to automatically
//! recognize common PrintConv patterns and map them to appropriate converters.

use crate::core::print_conv::PrintConvId;
use phf::phf_map;

/// Maps Perl string expressions to PrintConvId
///
/// These are simple string templates that can be mapped directly to
/// formatting functions. Most common patterns involve units or formatting.
pub static PERL_STRING_PATTERNS: phf::Map<&str, PrintConvId> = phf_map! {
    // Unit conversions
    "\"$val mm\"" => PrintConvId::Millimeters,
    "'$val mm'" => PrintConvId::Millimeters,
    "sprintf(\"%.1f\",$val)" => PrintConvId::Float1Decimal,
    "sprintf('%.1f',$val)" => PrintConvId::Float1Decimal,
    "sprintf(\"%.0f\",$val)" => PrintConvId::RoundToInt,
    "sprintf('%.0f',$val)" => PrintConvId::RoundToInt,
    "sprintf(\"%.2f\",$val)" => PrintConvId::Float2Decimal,
    "sprintf('%.2f',$val)" => PrintConvId::Float2Decimal,

    // Pass-through patterns
    "'$val'" => PrintConvId::None,
    "\"$val\"" => PrintConvId::None,
    "$val" => PrintConvId::None,

    // Date/time conversions
    "$self->ConvertDateTime($val)" => PrintConvId::DateTime,
    "ConvertDuration($val)" => PrintConvId::Duration,

    // Common formatting
    "sprintf(\"%d\",$val)" => PrintConvId::Integer,
    "sprintf('%d',$val)" => PrintConvId::Integer,
    "sprintf(\"0x%x\",$val)" => PrintConvId::Hex,
    "sprintf('0x%x',$val)" => PrintConvId::Hex,
};

/// Maps normalized hash patterns to PrintConvId  
///
/// Hash patterns are normalized by sorting key:value pairs and joining with commas.
/// This enables recognition of common on/off, yes/no, and other simple lookup patterns.
pub static HASH_PATTERNS: phf::Map<&str, PrintConvId> = phf_map! {
    // Binary toggles
    "0:Off,1:On" => PrintConvId::OnOff,
    "0:No,1:Yes" => PrintConvId::YesNo,
    "0:False,1:True" => PrintConvId::TrueFalse,
    "0:Disable,1:Enable" => PrintConvId::EnableDisable,
    "0:Disabled,1:Enabled" => PrintConvId::EnableDisable,

    // Image quality
    "0:Normal,1:Fine,2:Extra Fine" => PrintConvId::ImageQuality,
    "1:Best,2:Better,3:Good" => PrintConvId::Quality,
    "1:Fine,2:Normal,3:Basic" => PrintConvId::Quality,

    // Common multi-value patterns
    "0:Unknown,1:Macro,2:Close,3:Distant" => PrintConvId::SubjectDistanceRange,
    "0:Auto,1:Manual" => PrintConvId::AutoManual,
    "0:Single,1:Continuous" => PrintConvId::DriveMode,

    // File formats
    "0:None,1:Bitmap,2:JPEG,3:GIF" => PrintConvId::ImageFormat,
    "0:Real-world Subject,1:Written Document" => PrintConvId::TargetImageType,

    // Common camera settings
    "0:Program,1:Aperture Priority,2:Shutter Priority,3:Manual" => PrintConvId::ExposureMode,
    "0:Matrix,1:Center-weighted,2:Spot" => PrintConvId::MeteringMode,
    "0:Auto,1:Daylight,2:Shade,3:Tungsten,4:Fluorescent,5:Flash" => PrintConvId::WhiteBalance,
};

/// Maps function names to PrintConvId
///
/// ExifTool has standard functions for common conversions like exposure time,
/// f-number, etc. These can be mapped directly to equivalent Rust functions.
pub static FUNCTION_PATTERNS: phf::Map<&str, PrintConvId> = phf_map! {
    // Standard EXIF functions (use actual existing variant names from enum)
    "PrintExposureTime" => PrintConvId::ExposureTime,
    "Image::ExifTool::Exif::PrintExposureTime" => PrintConvId::ExposureTime,
    "PrintFNumber" => PrintConvId::FNumber,
    "Image::ExifTool::Exif::PrintFNumber" => PrintConvId::FNumber,
    "PrintFraction" => PrintConvId::Fraction,
    "Image::ExifTool::Exif::PrintFraction" => PrintConvId::Fraction,

    // Utility functions
    "ConvertBinary" => PrintConvId::ConvertBinary,
    "Image::ExifTool::ConvertBinary" => PrintConvId::ConvertBinary,
    "ConvertDuration" => PrintConvId::Duration,
    "ConvertUnixTime" => PrintConvId::UnixTime,

    // GPS functions
    "PrintGPSCoordinates" => PrintConvId::GPSCoordinate,
    "Image::ExifTool::GPS::PrintGPSCoordinates" => PrintConvId::GPSCoordinate,
};

/// Maps shared hash references to PrintConvId
///
/// ExifTool has large shared lookup tables (like lens databases) that are
/// referenced by multiple tags. These need special handling to generate
/// static lookup tables.
pub static HASH_REF_PATTERNS: phf::Map<&str, PrintConvId> = phf_map! {
    // Lens databases
    "canonLensTypes" => PrintConvId::CanonLensType,
    "nikonLensTypes" => PrintConvId::NikonLensType,
    "pentaxLensTypes" => PrintConvId::PentaxLensType,

    // Camera models
    "canonModelNames" => PrintConvId::CanonModelLookup,
    "nikonModelNames" => PrintConvId::NikonModelLookup,
    "pentaxModelNames" => PrintConvId::PentaxModelLookup,

    // Picture styles
    "canonPictureStyles" => PrintConvId::CanonPictureStyle,
    "canonUserDefStyles" => PrintConvId::CanonUserDefPictureStyle,

    // Flash modes
    "canonFlashModes" => PrintConvId::CanonFlashMode,
    "nikonFlashModes" => PrintConvId::NikonFlashMode,
};

/// Normalize a hash map to a pattern string for matching
///
/// Takes a HashMap and converts it to a normalized string representation
/// by sorting key:value pairs and joining with commas. This enables
/// consistent pattern matching regardless of hash iteration order.
pub fn normalize_hash_pattern(
    map: &std::collections::HashMap<String, serde_json::Value>,
) -> String {
    let mut pairs: Vec<_> = map
        .iter()
        .filter_map(|(k, v)| {
            // Convert JSON values to strings, handling different types
            let value_str = match v {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                _ => return None, // Skip complex types
            };
            Some(format!("{}:{}", k, value_str))
        })
        .collect();

    pairs.sort();
    pairs.join(",")
}

/// Determine PrintConvId from extracted PrintConv data
///
/// This is the main entry point for pattern matching. Takes the JSON data
/// extracted by the Perl script and attempts to map it to a known PrintConvId.
pub fn determine_printconv_id(
    printconv_type: &str,
    printconv_data: Option<&std::collections::HashMap<String, serde_json::Value>>,
    printconv_ref: Option<&str>,
    printconv_source: Option<&str>,
    printconv_func: Option<&str>,
) -> PrintConvId {
    match printconv_type {
        "none" => PrintConvId::None,

        "hash" => {
            if let Some(map) = printconv_data {
                let pattern = normalize_hash_pattern(map);
                if let Some(&id) = HASH_PATTERNS.get(&pattern) {
                    return id;
                }
            }
            PrintConvId::None
        }

        "hash_ref" => {
            if let Some(ref_name) = printconv_ref {
                if let Some(&id) = HASH_REF_PATTERNS.get(ref_name) {
                    return id;
                }
            }
            PrintConvId::None
        }

        "string" => {
            if let Some(source) = printconv_source {
                if let Some(&id) = PERL_STRING_PATTERNS.get(source) {
                    return id;
                }
            }
            PrintConvId::None
        }

        "code_ref" => {
            if let Some(func) = printconv_func {
                if let Some(&id) = FUNCTION_PATTERNS.get(func) {
                    return id;
                }
            }
            PrintConvId::None
        }

        _ => PrintConvId::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::collections::HashMap;

    #[test]
    fn test_string_pattern_lookup() {
        assert_eq!(
            PERL_STRING_PATTERNS.get("\"$val mm\""),
            Some(&PrintConvId::Millimeters)
        );
        assert_eq!(
            PERL_STRING_PATTERNS.get("sprintf(\"%.1f\",$val)"),
            Some(&PrintConvId::Float1Decimal)
        );
    }

    #[test]
    fn test_hash_pattern_lookup() {
        assert_eq!(HASH_PATTERNS.get("0:Off,1:On"), Some(&PrintConvId::OnOff));
        assert_eq!(HASH_PATTERNS.get("0:No,1:Yes"), Some(&PrintConvId::YesNo));
    }

    #[test]
    fn test_function_pattern_lookup() {
        assert_eq!(
            FUNCTION_PATTERNS.get("PrintExposureTime"),
            Some(&PrintConvId::ExposureTime)
        );
        assert_eq!(
            FUNCTION_PATTERNS.get("Image::ExifTool::Exif::PrintFNumber"),
            Some(&PrintConvId::FNumber)
        );
    }

    #[test]
    fn test_hash_ref_pattern_lookup() {
        assert_eq!(
            HASH_REF_PATTERNS.get("canonLensTypes"),
            Some(&PrintConvId::CanonLensType)
        );
        assert_eq!(
            HASH_REF_PATTERNS.get("nikonLensTypes"),
            Some(&PrintConvId::NikonLensType)
        );
    }

    #[test]
    fn test_normalize_hash_pattern() {
        let mut map = HashMap::new();
        map.insert("1".to_string(), Value::String("On".to_string()));
        map.insert("0".to_string(), Value::String("Off".to_string()));

        assert_eq!(normalize_hash_pattern(&map), "0:Off,1:On");
    }

    #[test]
    fn test_determine_printconv_id_hash() {
        let mut map = HashMap::new();
        map.insert("0".to_string(), Value::String("Off".to_string()));
        map.insert("1".to_string(), Value::String("On".to_string()));

        let result = determine_printconv_id("hash", Some(&map), None, None, None);
        assert_eq!(result, PrintConvId::OnOff);
    }

    #[test]
    fn test_determine_printconv_id_string() {
        let result = determine_printconv_id("string", None, None, Some("\"$val mm\""), None);
        assert_eq!(result, PrintConvId::Millimeters);
    }

    #[test]
    fn test_determine_printconv_id_code_ref() {
        let result =
            determine_printconv_id("code_ref", None, None, None, Some("PrintExposureTime"));
        assert_eq!(result, PrintConvId::ExposureTime);
    }

    #[test]
    fn test_determine_printconv_id_hash_ref() {
        let result = determine_printconv_id("hash_ref", None, Some("canonLensTypes"), None, None);
        assert_eq!(result, PrintConvId::CanonLensType);
    }
}
