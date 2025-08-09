//! Nikon lens database and identification system
//!
//! **Trust ExifTool**: This code translates ExifTool's Nikon lens database verbatim.
//!
//! ExifTool Reference: lib/Image/ExifTool/Nikon.pm %nikonLensIDs hash (614 entries)
//!
//! Nikon's lens identification system uses 8-byte lens data patterns to identify
//! specific lens models, including third-party lenses and teleconverters.
//!
//! Implementation: Uses generated lens database from ExifTool simple table extraction

use crate::generated::Nikon_pm::nikon_lens_ids::{lookup_nikon_lens_ids, NIKON_LENS_IDS};
use std::collections::HashMap;
use tracing::{debug, trace, warn};

/// Nikon lens database entry structure
/// ExifTool: nikonLensIDs hash entry format
#[derive(Debug, Clone)]
pub struct NikonLensEntry {
    /// 8-byte lens ID pattern in hex format
    /// ExifTool: Hash key format "50 1 0C 00 02 00 14 02"
    pub id_pattern: String,

    /// Human-readable lens description
    /// ExifTool: Hash value - lens name and specifications
    pub description: String,

    /// Optional lens categories for filtering
    /// ExifTool: Implied from description patterns
    pub category: LensCategory,
}

/// Lens category classification for organization
/// ExifTool: Implicit categorization from lens descriptions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LensCategory {
    /// Nikon AF-S lenses
    AfS,
    /// Nikon AF lenses  
    Af,
    /// Nikon manual focus lenses
    Manual,
    /// Third-party lenses (Sigma, Tamron, etc.)
    ThirdParty,
    /// Teleconverters
    Teleconverter,
    /// Unknown or unclassified
    Unknown,
}

impl LensCategory {
    /// Determine category from lens description
    /// ExifTool: Implicit categorization logic
    pub fn from_description(description: &str) -> Self {
        if description.contains("AF-S") {
            LensCategory::AfS
        } else if description.contains("AF ") && !description.contains("AF-S") {
            LensCategory::Af
        } else if description.contains("TC-") || description.contains("Teleconverter") {
            LensCategory::Teleconverter
        } else if description.contains("Sigma")
            || description.contains("Tamron")
            || description.contains("Tokina")
        {
            LensCategory::ThirdParty
        } else {
            LensCategory::Unknown
        }
    }
}

/// Get all lenses in a specific category from the generated database
/// Utility function for browsing lens database
pub fn get_lenses_by_category_from_generated() -> HashMap<LensCategory, Vec<(String, String)>> {
    let mut categories: HashMap<LensCategory, Vec<(String, String)>> = HashMap::new();

    for (pattern, description) in NIKON_LENS_IDS.iter() {
        let category = LensCategory::from_description(description);
        categories
            .entry(category)
            .or_default()
            .push((pattern.to_string(), description.to_string()));
    }

    categories
}

/// Look up Nikon lens by 8-byte lens data
/// ExifTool: Nikon.pm LensIDConv function
pub fn lookup_nikon_lens(lens_data: &[u8]) -> Option<String> {
    if lens_data.len() < 8 {
        warn!(
            "Insufficient lens data for Nikon lookup: {} bytes",
            lens_data.len()
        );
        return None;
    }

    // Format 8-byte lens data as hex string pattern
    // ExifTool: sprintf("%.2X %.2X %.2X %.2X %.2X %.2X %.2X %.2X", unpack("C*", $val))
    let id_pattern = format!(
        "{:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}",
        lens_data[0],
        lens_data[1],
        lens_data[2],
        lens_data[3],
        lens_data[4],
        lens_data[5],
        lens_data[6],
        lens_data[7]
    );

    trace!("Looking up Nikon lens with pattern: {}", id_pattern);

    // Use generated lookup table from ExifTool simple table extraction
    if let Some(description) = lookup_nikon_lens_ids(&id_pattern) {
        debug!(
            "Found exact Nikon lens match: {} -> {}",
            id_pattern, description
        );
        return Some(description.to_string());
    }

    // TODO: Future enhancement could add pattern matching for lens variants
    // ExifTool uses complex pattern matching for lens variations:
    // - Firmware version differences
    // - Regional model variations
    // - Lens adapter combinations

    debug!("No Nikon lens match found for pattern: {}", id_pattern);
    None
}

/// Get lens category for a given lens ID
/// ExifTool: Implicit categorization from lens database
pub fn get_nikon_lens_category(lens_data: &[u8]) -> LensCategory {
    if let Some(description) = lookup_nikon_lens(lens_data) {
        LensCategory::from_description(&description)
    } else {
        LensCategory::Unknown
    }
}

/// Get all lenses in a specific category
/// Utility function for browsing lens database
pub fn get_lenses_by_category(category: LensCategory) -> Vec<(String, String)> {
    let mut matching_lenses = Vec::new();

    for (pattern, description) in NIKON_LENS_IDS.iter() {
        let lens_category = LensCategory::from_description(description);
        if lens_category == category {
            matching_lenses.push((pattern.to_string(), description.to_string()));
        }
    }

    matching_lenses
}

/// Get database statistics
/// Debugging and validation utility
pub fn get_database_stats() -> (usize, HashMap<LensCategory, usize>) {
    let total = NIKON_LENS_IDS.len();
    let mut category_counts = HashMap::new();

    for (_, description) in NIKON_LENS_IDS.iter() {
        let category = LensCategory::from_description(description);
        *category_counts.entry(category).or_insert(0) += 1;
    }

    debug!(
        "Nikon lens database stats: {} total, {:?} by category",
        total, category_counts
    );
    (total, category_counts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lens_category_classification() {
        assert_eq!(
            LensCategory::from_description("AF-S Nikkor 85mm f/1.4G"),
            LensCategory::AfS
        );
        assert_eq!(
            LensCategory::from_description("AF Nikkor 50mm f/1.8"),
            LensCategory::Af
        );
        assert_eq!(
            LensCategory::from_description("TC-20E"),
            LensCategory::Teleconverter
        );
        assert_eq!(
            LensCategory::from_description("Sigma 35mm f/1.4 DG HSM Art"),
            LensCategory::ThirdParty
        );
    }

    #[test]
    fn test_database_completeness() {
        let (total, categories) = get_database_stats();

        // Should have the full ExifTool database (614 entries)
        assert!(
            total >= 600,
            "Database should have at least 600 entries, found {total}"
        );

        // Should have good representation across categories
        assert!(categories.contains_key(&LensCategory::AfS));
        assert!(categories.contains_key(&LensCategory::ThirdParty));
        assert!(categories.contains_key(&LensCategory::Teleconverter));

        // Should have substantial number of AF-S lenses
        let af_s_count = categories.get(&LensCategory::AfS).unwrap_or(&0);
        assert!(
            *af_s_count >= 50,
            "Should have at least 50 AF-S lenses, found {af_s_count}"
        );

        // Should have good third-party representation
        let third_party_count = categories.get(&LensCategory::ThirdParty).unwrap_or(&0);
        assert!(
            *third_party_count >= 20,
            "Should have at least 20 third-party lenses, found {third_party_count}"
        );
    }

    #[test]
    fn test_get_lenses_by_category() {
        let af_s_lenses = get_lenses_by_category(LensCategory::AfS);
        assert!(!af_s_lenses.is_empty());

        // Verify all returned lenses are actually AF-S
        for (_pattern, description) in &af_s_lenses {
            assert!(description.contains("AF-S"));
        }

        // Test third-party category
        let third_party_lenses = get_lenses_by_category(LensCategory::ThirdParty);
        assert!(!third_party_lenses.is_empty());

        // Verify third-party lenses are properly categorized
        for (_pattern, description) in &third_party_lenses {
            assert!(
                description.contains("Sigma")
                    || description.contains("Tamron")
                    || description.contains("Tokina")
                    || description.contains("Samyang")
                    || description.contains("Rokinon")
                    || description.contains("Sony")
            );
        }

        // Test teleconverter category
        let tc_lenses = get_lenses_by_category(LensCategory::Teleconverter);
        assert!(!tc_lenses.is_empty());

        for (_pattern, description) in &tc_lenses {
            assert!(description.contains("TC-"));
        }
    }

    #[test]
    fn test_generated_lookup_function_works() {
        // Test that the generated lookup function from simple table extraction works
        use crate::generated::Nikon_pm::nikon_lens_ids::lookup_nikon_lens_ids;

        // Test AF Nikkor 50mm f/1.8 - ExifTool Nikon.pm:96
        let result = lookup_nikon_lens_ids("01 58 50 50 14 14 02 00");
        assert_eq!(result, Some("AF Nikkor 50mm f/1.8"));

        // Test non-existent pattern
        let result = lookup_nikon_lens_ids("FF FF FF FF FF FF FF FF");
        assert_eq!(result, None);
    }
}
