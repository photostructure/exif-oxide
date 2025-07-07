//! Tag table selection and lookup functions for Nikon cameras
//!
//! **Trust ExifTool**: This code translates ExifTool's tag lookup logic verbatim.
//!
//! ExifTool Reference: lib/Image/ExifTool/Nikon.pm tag table selection logic

use super::{tables::*, NikonTagTable};
use tracing::debug;

/// Get appropriate tag table for camera model
/// ExifTool: Model-specific tag table selection logic
pub fn select_nikon_tag_table(model: &str) -> &'static NikonTagTable {
    // Model-specific table selection
    // ExifTool: Condition evaluation for model-specific tables
    if model.contains("Z 9") {
        debug!("Selected Nikon Z9 ShotInfo table for model: {}", model);
        &NIKON_Z9_SHOT_INFO
    } else if model.contains("Z 8") {
        debug!("Selected Nikon Z8 ShotInfo table for model: {}", model);
        &NIKON_Z8_SHOT_INFO
    } else if model.contains("Z 6III") {
        debug!("Selected Nikon Z6III ShotInfo table for model: {}", model);
        &NIKON_Z6III_SHOT_INFO
    } else if model.contains("D850") {
        debug!("Selected Nikon D850 ShotInfo table for model: {}", model);
        &NIKON_D850_SHOT_INFO
    } else if model.contains("D6") {
        debug!("Selected Nikon D6 ShotInfo table for model: {}", model);
        &NIKON_D6_SHOT_INFO
    } else {
        // Default to main table for all other Nikon cameras
        debug!("Selected Nikon Main table for model: {}", model);
        &NIKON_MAIN_TAGS
    }
}

/// Look up Nikon tag name by ID
/// ExifTool: Tag table lookup functionality with model-specific precedence
pub fn get_nikon_tag_name(tag_id: u16, model: &str) -> Option<&'static str> {
    let table = select_nikon_tag_table(model);

    // First check model-specific table
    if let Some((_, name, _)) = table.tags.iter().find(|(id, _, _)| *id == tag_id) {
        return Some(name);
    }

    // If not found in model-specific table, fall back to main table
    // ExifTool: Nikon.pm main table serves as fallback for all models
    if table.name != "Nikon::Main" {
        NIKON_MAIN_TAGS
            .tags
            .iter()
            .find(|(id, _, _)| *id == tag_id)
            .map(|(_, name, _)| *name)
    } else {
        None // Already checked main table
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_selection_z9() {
        let table = select_nikon_tag_table("NIKON Z 9");
        assert_eq!(table.name, "Nikon::ShotInfoZ9");
    }

    #[test]
    fn test_table_selection_generic() {
        let table = select_nikon_tag_table("NIKON D7500");
        assert_eq!(table.name, "Nikon::Main");
    }

    #[test]
    fn test_tag_name_lookup() {
        // Use main table for tag lookup test
        let name = get_nikon_tag_name(0x0004, "NIKON D7500");
        assert_eq!(name, Some("Quality"));

        // Test model-specific tag lookup
        let name = get_nikon_tag_name(0x0004, "NIKON D850");
        assert_eq!(name, Some("FirmwareVersion")); // D850 uses ShotInfo table
    }

    #[test]
    fn test_encryption_key_tags_present() {
        // Verify encryption key tags are included in main table
        let serial_tag = get_nikon_tag_name(0x001D, "NIKON D7500");
        assert_eq!(serial_tag, Some("SerialNumber"));

        let count_tag = get_nikon_tag_name(0x00A7, "NIKON D7500");
        assert_eq!(count_tag, Some("ShutterCount"));
    }

    // Phase 3 Model-specific table tests
    #[test]
    fn test_model_specific_table_selection() {
        // Test Z9
        let table = select_nikon_tag_table("NIKON Z 9");
        assert_eq!(table.name, "Nikon::ShotInfoZ9");
        assert_eq!(table.model_condition, Some("NIKON Z 9"));

        // Test Z8
        let table = select_nikon_tag_table("NIKON Z 8");
        assert_eq!(table.name, "Nikon::ShotInfoZ8");
        assert_eq!(table.model_condition, Some("NIKON Z 8"));

        // Test Z6III
        let table = select_nikon_tag_table("NIKON Z 6III");
        assert_eq!(table.name, "Nikon::ShotInfoZ6III");
        assert_eq!(table.model_condition, Some("NIKON Z 6III"));

        // Test D850
        let table = select_nikon_tag_table("NIKON D850");
        assert_eq!(table.name, "Nikon::ShotInfoD850");
        assert_eq!(table.model_condition, Some("NIKON D850"));

        // Test D6
        let table = select_nikon_tag_table("NIKON D6");
        assert_eq!(table.name, "Nikon::ShotInfoD6");
        assert_eq!(table.model_condition, Some("NIKON D6"));

        // Test fallback to main table
        let table = select_nikon_tag_table("NIKON D7500");
        assert_eq!(table.name, "Nikon::Main");
        assert!(table.model_condition.is_none());
    }
}
