//! Minolta RAW PrintConv implementations
//!
//! This module contains PrintConv functions for Minolta MRW format tags.
//! All implementations use lookup tables from ExifTool source code,
//! following the Trust ExifTool principle exactly.
//!
//! ExifTool Reference: lib/Image/ExifTool/MinoltaRaw.pm

use crate::types::TagValue;

/// Minolta PRD StorageMethod PrintConv
/// ExifTool: lib/Image/ExifTool/MinoltaRaw.pm:94-99
/// PrintConv => { 82 => 'Padded', 89 => 'Linear' }
pub fn prd_storage_method_print_conv(val: &TagValue) -> TagValue {
    let Some(key) = val.as_i64() else {
        return val.clone();
    };

    match key {
        82 => TagValue::string("Padded"),
        89 => TagValue::string("Linear"),
        _ => TagValue::string(format!("Unknown ({})", key)),
    }
}

/// Minolta PRD BayerPattern PrintConv
/// ExifTool: lib/Image/ExifTool/MinoltaRaw.pm:104-108
/// PrintConv => { 1 => 'RGGB', 4 => 'GBRG' }
pub fn prd_bayer_pattern_print_conv(val: &TagValue) -> TagValue {
    let Some(key) = val.as_i64() else {
        return val.clone();
    };

    match key {
        1 => TagValue::string("RGGB"),
        4 => TagValue::string("GBRG"),
        _ => TagValue::string(format!("Unknown ({})", key)),
    }
}

/// Minolta RIF ProgramMode PrintConv
/// ExifTool: lib/Image/ExifTool/MinoltaRaw.pm:165-174
/// PrintConv => { 0 => 'None', 1 => 'Portrait', 2 => 'Text', 3 => 'Night Portrait', 4 => 'Sunset', 5 => 'Sports' }
pub fn rif_program_mode_print_conv(val: &TagValue) -> TagValue {
    let Some(key) = val.as_i64() else {
        return val.clone();
    };

    match key {
        0 => TagValue::string("None"),
        1 => TagValue::string("Portrait"),
        2 => TagValue::string("Text"),
        3 => TagValue::string("Night Portrait"),
        4 => TagValue::string("Sunset"),
        5 => TagValue::string("Sports"),
        _ => TagValue::string(format!("Unknown ({})", key)),
    }
}

/// Minolta RIF ZoneMatching PrintConv
/// ExifTool: lib/Image/ExifTool/MinoltaRaw.pm:283-287
/// PrintConv => { 0 => 'ISO Setting Used', 1 => 'High Key', 2 => 'Low Key' }
pub fn rif_zone_matching_print_conv(val: &TagValue) -> TagValue {
    let Some(key) = val.as_i64() else {
        return val.clone();
    };

    match key {
        0 => TagValue::string("ISO Setting Used"),
        1 => TagValue::string("High Key"),
        2 => TagValue::string("Low Key"),
        _ => TagValue::string(format!("Unknown ({})", key)),
    }
}

/// Minolta RIF ZoneMatching74 PrintConv (for tag offset 74)
/// ExifTool: lib/Image/ExifTool/MinoltaRaw.pm:305-309
/// Same as ZoneMatching but for Sony models at offset 74
pub fn rif_zone_matching_74_print_conv(val: &TagValue) -> TagValue {
    // Same lookup table as ZoneMatching
    rif_zone_matching_print_conv(val)
}

/// Apply PrintConv to Minolta PRD block tags
///
/// This function maps PRD tag IDs to their corresponding PrintConv functions.
/// ExifTool: MinoltaRaw.pm PRD hash PrintConv fields
pub fn apply_prd_print_conv(tag_name: &str, val: &TagValue) -> TagValue {
    match tag_name {
        "StorageMethod" => prd_storage_method_print_conv(val),
        "BayerPattern" => prd_bayer_pattern_print_conv(val),
        _ => val.clone(), // No PrintConv for this tag
    }
}

/// Apply PrintConv to Minolta RIF block tags
///
/// This function maps RIF tag IDs to their corresponding PrintConv functions.
/// ExifTool: MinoltaRaw.pm RIF hash PrintConv fields
pub fn apply_rif_print_conv(tag_name: &str, val: &TagValue) -> TagValue {
    match tag_name {
        "ProgramMode" => rif_program_mode_print_conv(val),
        "ZoneMatching" => rif_zone_matching_print_conv(val),
        "ZoneMatching74" => rif_zone_matching_74_print_conv(val),
        _ => val.clone(), // No PrintConv for this tag
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prd_storage_method_print_conv() {
        // Test known values from generated table
        let result = prd_storage_method_print_conv(&TagValue::U8(82));
        assert_eq!(result, TagValue::string("Padded"));

        let result = prd_storage_method_print_conv(&TagValue::U8(89));
        assert_eq!(result, TagValue::string("Linear"));

        // Test unknown value
        let result = prd_storage_method_print_conv(&TagValue::U8(99));
        assert_eq!(result, TagValue::string("Unknown (99)"));
    }

    #[test]
    fn test_prd_bayer_pattern_print_conv() {
        // Test known values from generated table
        let result = prd_bayer_pattern_print_conv(&TagValue::U8(1));
        assert_eq!(result, TagValue::string("RGGB"));

        let result = prd_bayer_pattern_print_conv(&TagValue::U8(4));
        assert_eq!(result, TagValue::string("GBRG"));

        // Test unknown value
        let result = prd_bayer_pattern_print_conv(&TagValue::U8(99));
        assert_eq!(result, TagValue::string("Unknown (99)"));
    }

    #[test]
    fn test_rif_program_mode_print_conv() {
        // Test known values from generated table
        let result = rif_program_mode_print_conv(&TagValue::U8(0));
        assert_eq!(result, TagValue::string("None"));

        let result = rif_program_mode_print_conv(&TagValue::U8(1));
        assert_eq!(result, TagValue::string("Portrait"));

        let result = rif_program_mode_print_conv(&TagValue::U8(5));
        assert_eq!(result, TagValue::string("Sports"));

        // Test unknown value
        let result = rif_program_mode_print_conv(&TagValue::U8(99));
        assert_eq!(result, TagValue::string("Unknown (99)"));
    }

    #[test]
    fn test_rif_zone_matching_print_conv() {
        // Test known values from generated table
        let result = rif_zone_matching_print_conv(&TagValue::U8(0));
        assert_eq!(result, TagValue::string("ISO Setting Used"));

        let result = rif_zone_matching_print_conv(&TagValue::U8(1));
        assert_eq!(result, TagValue::string("High Key"));

        let result = rif_zone_matching_print_conv(&TagValue::U8(2));
        assert_eq!(result, TagValue::string("Low Key"));

        // Test unknown value
        let result = rif_zone_matching_print_conv(&TagValue::U8(99));
        assert_eq!(result, TagValue::string("Unknown (99)"));
    }

    #[test]
    fn test_apply_prd_print_conv() {
        // Test StorageMethod conversion
        let result = apply_prd_print_conv("StorageMethod", &TagValue::U8(82));
        assert_eq!(result, TagValue::string("Padded"));

        // Test BayerPattern conversion
        let result = apply_prd_print_conv("BayerPattern", &TagValue::U8(1));
        assert_eq!(result, TagValue::string("RGGB"));

        // Test unknown tag - should return original value
        let original = TagValue::U8(42);
        let result = apply_prd_print_conv("UnknownTag", &original);
        assert_eq!(result, original);
    }

    #[test]
    fn test_apply_rif_print_conv() {
        // Test ProgramMode conversion
        let result = apply_rif_print_conv("ProgramMode", &TagValue::U8(1));
        assert_eq!(result, TagValue::string("Portrait"));

        // Test ZoneMatching conversion
        let result = apply_rif_print_conv("ZoneMatching", &TagValue::U8(2));
        assert_eq!(result, TagValue::string("Low Key"));

        // Test unknown tag - should return original value
        let original = TagValue::U8(42);
        let result = apply_rif_print_conv("UnknownTag", &original);
        assert_eq!(result, original);
    }
}
