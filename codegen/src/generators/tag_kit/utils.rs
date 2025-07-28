//! Utility functions for tag kit generation
//!
//! This module contains common utility functions used across tag kit generation,
//! including module name mapping and key parsing functions.

/// Convert module name to ExifTool group name
pub(super) fn module_name_to_group(module_name: &str) -> &str {
    match module_name {
        "Exif_pm" => "EXIF",
        "GPS_pm" => "GPS", 
        "Canon_pm" => "Canon",
        "Nikon_pm" => "Nikon",
        "Sony_pm" => "Sony",
        "Olympus_pm" => "Olympus",
        "Panasonic_pm" => "Panasonic",
        "PanasonicRaw_pm" => "PanasonicRaw",
        "MinoltaRaw_pm" => "MinoltaRaw",
        "KyoceraRaw_pm" => "KyoceraRaw",
        "FujiFilm_pm" => "FujiFilM",
        "Casio_pm" => "Casio",
        "QuickTime_pm" => "QuickTime",
        "RIFF_pm" => "RIFF",
        "XMP_pm" => "XMP",
        "PNG_pm" => "PNG",
        _ => module_name.trim_end_matches("_pm"),
    }
}

/// Parse fractional keys (e.g., "10.1" -> 10) or regular integer keys
/// This handles ExifTool's use of fractional keys to distinguish sub-variants:
/// - Canon lens types: "10.1", "10.2" for multiple lenses sharing base ID 10
/// - Nikon tag IDs: "586.1", "590.2" for sub-features within larger tag structures  
/// - Shutter speeds: "37.5", "40.5" for precise intermediate values
pub(super) fn parse_fractional_key_as_i16(key: &str) -> i16 {
    // First try parsing as regular integer
    if let Ok(val) = key.parse::<i16>() {
        return val;
    }
    
    // Try parsing fractional key (e.g., "10.1" -> 10)
    if let Some(dot_pos) = key.find('.') {
        let base_part = &key[..dot_pos];
        if let Ok(base_val) = base_part.parse::<i16>() {
            return base_val;
        }
    }
    
    // Fallback to 0 if parsing fails completely
    0
}

/// Parse fractional keys as u32 for tag IDs
pub(super) fn parse_fractional_key_as_u32(key: &str) -> u32 {
    // First try parsing as regular integer
    if let Ok(val) = key.parse::<u32>() {
        return val;
    }
    
    // Try parsing fractional key (e.g., "586.1" -> 586)
    if let Some(dot_pos) = key.find('.') {
        let base_part = &key[..dot_pos];
        if let Ok(base_val) = base_part.parse::<u32>() {
            return base_val;
        }
    }
    
    // Fallback to 0 if parsing fails completely
    0
}

/// Convert snake_case to PascalCase
pub(super) fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
            }
        })
        .collect()
}