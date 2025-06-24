//! Fujifilm ProcessBinaryData tables
//!
//! This module defines the ProcessBinaryData tables used by Fujifilm cameras
//! for structured binary data in maker notes. These tables are ported from
//! ExifTool's FujiFilm.pm file.

use crate::core::binary_data::{BinaryDataTable, BinaryDataTableBuilder};
use crate::core::types::ExifFormat;
use std::collections::HashMap;

/// Create all Fujifilm ProcessBinaryData tables
pub fn create_fujifilm_binary_tables() -> HashMap<&'static str, BinaryDataTable> {
    let mut tables = HashMap::new();

    // PrioritySettings table (tag 0x102b)
    tables.insert("PrioritySettings", create_priority_settings_table());

    // FocusSettings table (tag 0x102d)
    tables.insert("FocusSettings", create_focus_settings_table());

    // AFCSettings table (tag 0x102e)
    tables.insert("AFCSettings", create_afc_settings_table());

    // DriveSettings table (tag 0x1103)
    tables.insert("DriveSettings", create_drive_settings_table());

    // MyMenuSettings table (tag 0x1400)
    tables.insert("MyMenuSettings", create_my_menu_settings_table());

    // FilmSimulation table (tag 0x1401)
    tables.insert("FilmSimulation", create_film_simulation_table());

    // Rating table (tag 0x1438)
    tables.insert("Rating", create_rating_table());

    // ImageGeneration table (tag 0x1443)
    tables.insert("ImageGeneration", create_image_generation_table());

    tables
}

/// PrioritySettings table (tag 0x102b)
/// Focus Priority settings from X-T3
fn create_priority_settings_table() -> BinaryDataTable {
    BinaryDataTableBuilder::new("PrioritySettings", ExifFormat::U16)
        .add_masked_field(0, "AF-SPriority", ExifFormat::U16, 0x000f, 0)
        .add_masked_field(0, "AF-CPriority", ExifFormat::U16, 0x00f0, 4)
        .build()
}

/// FocusSettings table (tag 0x102d)
/// Focus settings from X-T3
fn create_focus_settings_table() -> BinaryDataTable {
    BinaryDataTableBuilder::new("FocusSettings", ExifFormat::U32)
        .add_masked_field(0, "FocusMode2", ExifFormat::U32, 0x0000000f, 0)
        .add_masked_field(0, "PreAF", ExifFormat::U32, 0x000000f0, 4)
        .add_masked_field(0, "InstantAFSetting", ExifFormat::U32, 0x00000f00, 8)
        .add_masked_field(0, "AFMode", ExifFormat::U32, 0x0000f000, 12)
        .add_masked_field(0, "AFPointDisplay", ExifFormat::U32, 0x000f0000, 16)
        .add_masked_field(0, "FocusCheck", ExifFormat::U32, 0x00f00000, 20)
        .add_masked_field(0, "AFIlluminator", ExifFormat::U32, 0x0f000000, 24)
        .add_masked_field(0, "MFAssist", ExifFormat::U32, 0xf0000000, 28)
        .build()
}

/// AFCSettings table (tag 0x102e)  
/// AF-C settings
fn create_afc_settings_table() -> BinaryDataTable {
    BinaryDataTableBuilder::new("AFCSettings", ExifFormat::U32)
        .add_field(0, "AF-CSetting", ExifFormat::U32, 1)
        .add_masked_field(0, "AF-CTrackingSensitivity", ExifFormat::U32, 0x000f, 0)
        .add_masked_field(
            0,
            "AF-CSpeedTrackingSensitivity",
            ExifFormat::U32,
            0x00f0,
            4,
        )
        .add_masked_field(0, "AF-CZoneAreaSwitching", ExifFormat::U32, 0x0f00, 8)
        .build()
}

/// DriveSettings table (tag 0x1103)
/// Drive mode settings
fn create_drive_settings_table() -> BinaryDataTable {
    BinaryDataTableBuilder::new("DriveSettings", ExifFormat::U32)
        .add_field(0, "DriveMode", ExifFormat::U32, 1)
        .add_masked_field(0, "SelfTimerTime", ExifFormat::U32, 0x000000ff, 0)
        .add_masked_field(0, "ContinuousShots", ExifFormat::U32, 0x0000ff00, 8)
        .add_masked_field(0, "ContinuousMode", ExifFormat::U32, 0x00ff0000, 16)
        .add_masked_field(0, "BurstMode", ExifFormat::U32, 0xff000000, 24)
        .build()
}

/// MyMenuSettings table (tag 0x1400)
/// Custom menu settings
fn create_my_menu_settings_table() -> BinaryDataTable {
    BinaryDataTableBuilder::new("MyMenuSettings", ExifFormat::U16)
        .add_field(0, "MyMenuSetting1", ExifFormat::U16, 1)
        .add_field(1, "MyMenuSetting2", ExifFormat::U16, 1)
        .add_field(2, "MyMenuSetting3", ExifFormat::U16, 1)
        .add_field(3, "MyMenuSetting4", ExifFormat::U16, 1)
        .add_field(4, "MyMenuSetting5", ExifFormat::U16, 1)
        .add_field(5, "MyMenuSetting6", ExifFormat::U16, 1)
        .add_field(6, "MyMenuSetting7", ExifFormat::U16, 1)
        .add_field(7, "MyMenuSetting8", ExifFormat::U16, 1)
        .build()
}

/// FilmSimulation table (tag 0x1401)
/// Film simulation settings
fn create_film_simulation_table() -> BinaryDataTable {
    BinaryDataTableBuilder::new("FilmSimulation", ExifFormat::U32)
        .add_field(0, "FilmMode", ExifFormat::U32, 1)
        .add_masked_field(0, "GrainEffect", ExifFormat::U32, 0x000000ff, 0)
        .add_masked_field(0, "ColorChromeEffect", ExifFormat::U32, 0x0000ff00, 8)
        .add_masked_field(0, "ColorChromeBlue", ExifFormat::U32, 0x00ff0000, 16)
        .add_masked_field(0, "SmoothSkinEffect", ExifFormat::U32, 0xff000000, 24)
        .build()
}

/// Rating table (tag 0x1438)
/// Image rating information
fn create_rating_table() -> BinaryDataTable {
    BinaryDataTableBuilder::new("Rating", ExifFormat::U32)
        .add_field(0, "Rating", ExifFormat::U32, 1)
        .add_field(1, "RatingDate", ExifFormat::U32, 1)
        .build()
}

/// ImageGeneration table (tag 0x1443)
/// Image generation information  
fn create_image_generation_table() -> BinaryDataTable {
    BinaryDataTableBuilder::new("ImageGeneration", ExifFormat::U32)
        .add_field(0, "ImageGeneration", ExifFormat::U32, 1)
        .add_field(1, "ImageGenerationMode", ExifFormat::U32, 1)
        .add_field(2, "ImageUniqueID", ExifFormat::U32, 4) // Array of 4 values
        .build()
}

/// Tag mapping for Fujifilm ProcessBinaryData tables
/// Maps tag IDs to table names for lookup
pub fn get_fujifilm_binary_tag_mapping() -> HashMap<u16, &'static str> {
    let mut mapping = HashMap::new();

    // Map tag IDs to table names
    mapping.insert(0x102b, "PrioritySettings");
    mapping.insert(0x102d, "FocusSettings");
    mapping.insert(0x102e, "AFCSettings");
    mapping.insert(0x1103, "DriveSettings");
    mapping.insert(0x1400, "MyMenuSettings");
    mapping.insert(0x1401, "FilmSimulation");
    mapping.insert(0x1438, "Rating");
    mapping.insert(0x1443, "ImageGeneration");

    mapping
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fujifilm_binary_tables_creation() {
        let tables = create_fujifilm_binary_tables();

        // Should have all 8 tables
        assert_eq!(tables.len(), 8);

        // Check that specific tables exist
        assert!(tables.contains_key("PrioritySettings"));
        assert!(tables.contains_key("FocusSettings"));
        assert!(tables.contains_key("AFCSettings"));
        assert!(tables.contains_key("DriveSettings"));
        assert!(tables.contains_key("MyMenuSettings"));
        assert!(tables.contains_key("FilmSimulation"));
        assert!(tables.contains_key("Rating"));
        assert!(tables.contains_key("ImageGeneration"));
    }

    #[test]
    fn test_fujifilm_binary_tag_mapping() {
        let mapping = get_fujifilm_binary_tag_mapping();

        // Should have 8 mappings
        assert_eq!(mapping.len(), 8);

        // Check specific mappings
        assert_eq!(mapping.get(&0x102b), Some(&"PrioritySettings"));
        assert_eq!(mapping.get(&0x102d), Some(&"FocusSettings"));
        assert_eq!(mapping.get(&0x102e), Some(&"AFCSettings"));
        assert_eq!(mapping.get(&0x1103), Some(&"DriveSettings"));
    }

    #[test]
    fn test_priority_settings_table() {
        let table = create_priority_settings_table();
        assert_eq!(table.name, "PrioritySettings");
        assert_eq!(table.default_format, ExifFormat::U16);
        assert_eq!(table.fields.len(), 2);
    }

    #[test]
    fn test_focus_settings_table() {
        let table = create_focus_settings_table();
        assert_eq!(table.name, "FocusSettings");
        assert_eq!(table.default_format, ExifFormat::U32);
        assert_eq!(table.fields.len(), 8);
    }
}
