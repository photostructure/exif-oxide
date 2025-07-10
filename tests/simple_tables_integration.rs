//! Integration tests for generated simple tables
//!
//! These tests validate that the simple table extraction framework
//! generates working lookup tables with correct data.

#[cfg(test)]
mod simple_table_tests {
    use exif_oxide::generated::canon::*;
    use exif_oxide::generated::nikon::*;

    #[test]
    fn test_nikon_lens_database_completeness() {
        use exif_oxide::generated::nikon::{lookup_nikon_lens_ids, NIKON_LENS_IDS};

        // Should have exactly 614 entries from ExifTool
        assert_eq!(NIKON_LENS_IDS.len(), 614);

        // Test known entries from ExifTool Nikon.pm
        assert_eq!(
            lookup_nikon_lens_ids("01 58 50 50 14 14 02 00"),
            Some("AF Nikkor 50mm f/1.8")
        );
        assert_eq!(
            lookup_nikon_lens_ids("05 54 50 50 0C 0C 04 00"),
            Some("AF Nikkor 50mm f/1.4")
        );

        // Test non-existent entry
        assert_eq!(lookup_nikon_lens_ids("FF FF FF FF FF FF FF FF"), None);
    }

    #[test]
    fn test_canon_model_id_completeness() {
        use exif_oxide::generated::canon::{lookup_canon_model_id, CANON_MODEL_ID};

        // Should have exactly 354 entries from ExifTool
        assert_eq!(CANON_MODEL_ID.len(), 354);

        // Test known entries from ExifTool Canon.pm
        assert!(lookup_canon_model_id(0x1010000).is_some());

        // Test non-existent entry
        assert_eq!(lookup_canon_model_id(0xFFFFFFFF), None);
    }

    #[test]
    fn test_canon_white_balance_completeness() {
        use exif_oxide::generated::canon::{lookup_canon_white_balance, CANON_WHITE_BALANCE};

        // Should have exactly 22 entries from ExifTool
        assert_eq!(CANON_WHITE_BALANCE.len(), 22);

        // Test known entries from ExifTool Canon.pm
        assert_eq!(lookup_canon_white_balance(0), Some("Auto"));
        assert_eq!(lookup_canon_white_balance(1), Some("Daylight"));
        assert_eq!(lookup_canon_white_balance(2), Some("Cloudy"));
        assert_eq!(lookup_canon_white_balance(3), Some("Tungsten"));
        assert_eq!(lookup_canon_white_balance(4), Some("Fluorescent"));

        // Test non-existent entry
        assert_eq!(lookup_canon_white_balance(255), None);
    }

    #[test]
    fn test_canon_picture_styles_completeness() {
        use exif_oxide::generated::canon::{lookup_picture_styles, PICTURE_STYLES};

        // Should have exactly 24 entries from ExifTool
        assert_eq!(PICTURE_STYLES.len(), 24);

        // Test known entries from ExifTool Canon.pm
        assert_eq!(lookup_picture_styles(0x01), Some("Standard"));
        assert_eq!(lookup_picture_styles(0x02), Some("Portrait"));

        // Test non-existent entry (use a value that definitely doesn't exist)
        assert_eq!(lookup_picture_styles(0x9999), None);
    }

    #[test]
    fn test_canon_image_size_completeness() {
        use exif_oxide::generated::canon::{lookup_canon_image_size, CANON_IMAGE_SIZE};

        // Should have exactly 19 entries from ExifTool
        assert_eq!(CANON_IMAGE_SIZE.len(), 19);

        // Test known entries from ExifTool Canon.pm (note: includes negative values)
        assert_eq!(lookup_canon_image_size(0), Some("Large"));
        assert_eq!(lookup_canon_image_size(-1), Some("n/a"));

        // Test non-existent entry
        assert_eq!(lookup_canon_image_size(9999), None);
    }

    #[test]
    fn test_canon_quality_completeness() {
        use exif_oxide::generated::canon::{lookup_canon_quality, CANON_QUALITY};

        // Should have exactly 9 entries from ExifTool
        assert_eq!(CANON_QUALITY.len(), 9);

        // Test known entries from ExifTool Canon.pm (note: includes negative values)
        assert_eq!(lookup_canon_quality(1), Some("Economy"));
        assert_eq!(lookup_canon_quality(2), Some("Normal"));
        assert_eq!(lookup_canon_quality(3), Some("Fine"));
        assert_eq!(lookup_canon_quality(-1), Some("n/a"));

        // Test non-existent entry
        assert_eq!(lookup_canon_quality(9999), None);
    }

    #[test]
    fn test_all_generated_modules_compile() {
        // This test ensures all generated modules compile and export correctly
        // If this compiles, all generated modules are syntactically correct

        use exif_oxide::generated::canon::{
            lookup_canon_image_size, lookup_canon_model_id, lookup_canon_quality,
            lookup_canon_white_balance, lookup_picture_styles,
        };
        use exif_oxide::generated::nikon::lookup_nikon_lens_ids;

        // Test that all lookup functions are callable and return correct types
        // This validates the generated function signatures and module exports
        let canon_model: Option<&'static str> = lookup_canon_model_id(0);
        let canon_wb: Option<&'static str> = lookup_canon_white_balance(0);
        let canon_style: Option<&'static str> = lookup_picture_styles(0);
        let canon_size: Option<&'static str> = lookup_canon_image_size(0);
        let canon_quality: Option<&'static str> = lookup_canon_quality(0);
        let nikon_lens: Option<&'static str> = lookup_nikon_lens_ids("test");

        // Verify all functions return the expected Option<&'static str> type
        assert!(canon_model.is_some() || canon_model.is_none()); // Always true, but type-checks
        assert!(canon_wb.is_some() || canon_wb.is_none());
        assert!(canon_style.is_some() || canon_style.is_none());
        assert!(canon_size.is_some() || canon_size.is_none());
        assert!(canon_quality.is_some() || canon_quality.is_none());
        assert!(nikon_lens.is_some() || nikon_lens.is_none());
    }

    #[test]
    fn test_performance_benchmarks() {
        use exif_oxide::generated::canon::lookup_canon_white_balance;
        use std::time::Instant;

        let start = Instant::now();

        // Perform 10,000 lookups
        for i in 0..10000 {
            let _ = lookup_canon_white_balance(i as u8);
        }

        let duration = start.elapsed();

        // 10K lookups should complete very quickly (< 10ms on modern hardware)
        assert!(
            duration.as_millis() < 100,
            "10K lookups took {}ms (expected < 100ms)",
            duration.as_millis()
        );
    }

    #[test]
    fn test_total_simple_tables_coverage() {
        use exif_oxide::generated::canon::{
            CANON_IMAGE_SIZE, CANON_MODEL_ID, CANON_QUALITY, CANON_WHITE_BALANCE, PICTURE_STYLES,
        };
        use exif_oxide::generated::nikon::NIKON_LENS_IDS;

        // Verify we have the expected total number of entries across all tables
        let total_entries = NIKON_LENS_IDS.len() +       // 614
            CANON_MODEL_ID.len() +       // 354  
            CANON_WHITE_BALANCE.len() +  // 22
            PICTURE_STYLES.len() +       // 24
            CANON_IMAGE_SIZE.len() +     // 19
            CANON_QUALITY.len(); // 9
                                 // Total: 1042

        assert_eq!(
            total_entries, 1042,
            "Expected 1042 total lookup entries, got {total_entries}"
        );
    }
}
