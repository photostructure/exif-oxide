//! Comprehensive unit tests for Nikon implementation components
//!
//! **Trust ExifTool**: These tests validate our translations against ExifTool behavior.
//!
//! Test coverage for Phase 1 components:
//! - Multi-format detection and signature validation
//! - Offset calculation accuracy and bounds checking  
//! - Encryption key management and validation
//! - Tag table selection and lookup functionality
//! - Lens database operations and pattern matching

use super::*;

#[cfg(test)]
mod detection_tests {
    use super::super::detection::*;

    #[test]
    fn test_nikon_format3_detection() {
        let format3_data = b"\x02\x10\x00\x00extra_data_follows";
        assert_eq!(
            detect_nikon_format(format3_data),
            Some(NikonFormat::Format3)
        );
    }

    #[test]
    fn test_nikon_format2_detection() {
        let format2_data = b"\x02\x00\x00\x00extra_data_follows";
        assert_eq!(
            detect_nikon_format(format2_data),
            Some(NikonFormat::Format2)
        );
    }

    #[test]
    fn test_nikon_format1_fallback() {
        let format1_data = b"\x01\x00\x00\x00unknown_signature";
        assert_eq!(
            detect_nikon_format(format1_data),
            Some(NikonFormat::Format1)
        );
    }

    #[test]
    fn test_nikon_format_insufficient_data() {
        let short_data = b"\x02\x10"; // Only 2 bytes
        assert_eq!(detect_nikon_format(short_data), None);
    }

    #[test]
    fn test_nikon_corporation_signature() {
        assert!(detect_nikon_signature("NIKON CORPORATION"));
    }

    #[test]
    fn test_nikon_short_signature() {
        assert!(detect_nikon_signature("NIKON"));
    }

    #[test]
    fn test_non_nikon_signatures() {
        assert!(!detect_nikon_signature("Canon"));
        assert!(!detect_nikon_signature("SONY"));
        assert!(!detect_nikon_signature("Panasonic"));
        assert!(!detect_nikon_signature("NIKON SCAN")); // Partial match should fail
        assert!(!detect_nikon_signature("nikon")); // Case sensitive
        assert!(!detect_nikon_signature("")); // Empty string
    }
}

#[cfg(test)]
mod offset_scheme_tests {
    use super::super::detection::NikonFormat;
    use super::super::offset_schemes::*;

    #[test]
    fn test_format3_offset_calculation() {
        let data_pos = 100;
        let base = calculate_nikon_base_offset(NikonFormat::Format3, data_pos);
        assert_eq!(base, 110); // 100 + 0x0a
    }

    #[test]
    fn test_format2_offset_calculation() {
        let data_pos = 200;
        let base = calculate_nikon_base_offset(NikonFormat::Format2, data_pos);
        assert_eq!(base, 208); // 200 + 0x08
    }

    #[test]
    fn test_format1_offset_calculation() {
        let data_pos = 50;
        let base = calculate_nikon_base_offset(NikonFormat::Format1, data_pos);
        assert_eq!(base, 56); // 50 + 0x06
    }

    #[test]
    fn test_offset_validation_success() {
        let result = validate_nikon_offset(NikonFormat::Format3, 100, 500);
        assert!(result.is_ok());
    }

    #[test]
    fn test_offset_validation_beyond_bounds() {
        let result = validate_nikon_offset(NikonFormat::Format3, 490, 500);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("beyond data bounds"));
    }

    #[test]
    fn test_format3_tiff_header_space_requirement() {
        // Format3 needs 8 bytes for TIFF header at calculated offset
        // Base offset = 490 + 0x0a = 500, needs 500 + 8 = 508, but only 507 available
        let result = validate_nikon_offset(NikonFormat::Format3, 490, 507);
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("TIFF header"));
    }

    #[test]
    fn test_format1_ifd_space_requirement() {
        // Format1 needs 2 bytes for IFD entry count at calculated offset
        // Base offset = 494 + 0x06 = 500, needs 500 + 2 = 502, but only 501 available
        let result = validate_nikon_offset(NikonFormat::Format1, 494, 501);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("IFD entry count"));
    }

    #[test]
    fn test_header_size_calculations() {
        assert_eq!(get_nikon_header_size(NikonFormat::Format3), 18); // 0x0a + 8
        assert_eq!(get_nikon_header_size(NikonFormat::Format2), 10); // 0x08 + 2
        assert_eq!(get_nikon_header_size(NikonFormat::Format1), 8); // 0x06 + 2
    }

    #[test]
    fn test_zero_data_pos() {
        let base = calculate_nikon_base_offset(NikonFormat::Format3, 0);
        assert_eq!(base, 10); // 0 + 0x0a
    }

    #[test]
    fn test_large_data_pos() {
        let data_pos = 0x10000;
        let base = calculate_nikon_base_offset(NikonFormat::Format2, data_pos);
        assert_eq!(base, 0x10008); // 0x10000 + 0x08
    }
}

#[cfg(test)]
mod encryption_tests {
    use super::super::encryption::*;

    #[test]
    fn test_encryption_keys_initialization() {
        let keys = NikonEncryptionKeys::new("NIKON Z 9".to_string());

        assert_eq!(keys.camera_model, "NIKON Z 9");
        assert!(!keys.has_required_keys());
        assert!(keys.get_serial_key().is_none());
        assert!(keys.get_count_key().is_none());
    }

    #[test]
    fn test_serial_key_storage() {
        let mut keys = NikonEncryptionKeys::new("NIKON D850".to_string());
        keys.store_serial_key("12345678".to_string());

        assert_eq!(keys.get_serial_key(), Some("12345678"));
        assert!(!keys.has_required_keys()); // Still need count key
    }

    #[test]
    fn test_count_key_storage() {
        let mut keys = NikonEncryptionKeys::new("NIKON Z 9".to_string());
        keys.store_count_key(1000);

        assert_eq!(keys.get_count_key(), Some(1000));
        assert!(!keys.has_required_keys()); // Still need serial key
    }

    #[test]
    fn test_complete_key_set() {
        let mut keys = NikonEncryptionKeys::new("NIKON D850".to_string());
        keys.store_serial_key("87654321".to_string());
        keys.store_count_key(1500);

        assert!(keys.has_required_keys());
        assert_eq!(keys.get_serial_key(), Some("87654321"));
        assert_eq!(keys.get_count_key(), Some(1500));
    }

    #[test]
    fn test_additional_parameters() {
        let mut keys = NikonEncryptionKeys::new("NIKON Z 9".to_string());
        keys.set_parameter("DecryptStart".to_string(), "0x100".to_string());
        keys.set_parameter("Algorithm".to_string(), "AES".to_string());

        assert_eq!(keys.get_parameter("DecryptStart"), Some("0x100"));
        assert_eq!(keys.get_parameter("Algorithm"), Some("AES"));
        assert_eq!(keys.get_parameter("Unknown"), None);
    }

    #[test]
    fn test_encryption_validation_without_keys() {
        let keys = NikonEncryptionKeys::new("NIKON D850".to_string());
        let result = validate_encryption_keys(&keys, "NIKON D850");

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Required encryption keys not available"));
    }

    #[test]
    fn test_encryption_validation_with_keys() {
        let mut keys = NikonEncryptionKeys::new("NIKON Z 9".to_string());
        keys.store_serial_key("Z9SERIAL".to_string());
        keys.store_count_key(2000);

        let result = validate_encryption_keys(&keys, "NIKON Z 9");
        assert!(result.is_ok());
    }

    #[test]
    fn test_encryption_key_overwrite() {
        let mut keys = NikonEncryptionKeys::new("NIKON D780".to_string());

        keys.store_serial_key("FIRST".to_string());
        assert_eq!(keys.get_serial_key(), Some("FIRST"));

        keys.store_serial_key("SECOND".to_string());
        assert_eq!(keys.get_serial_key(), Some("SECOND"));
    }
}

#[cfg(test)]
mod tag_tests {
    use super::super::tags::*;
    use crate::types::TagValue;

    #[test]
    fn test_nikon_main_table_structure() {
        assert_eq!(NIKON_MAIN_TAGS.name, "Nikon::Main");
        assert!(NIKON_MAIN_TAGS.model_condition.is_none());
        assert!(!NIKON_MAIN_TAGS.tags.is_empty());

        // Verify we have a reasonable number of tags
        assert!(NIKON_MAIN_TAGS.tags.len() >= 50);
    }

    #[test]
    fn test_nikon_z9_table_structure() {
        assert_eq!(NIKON_Z9_SHOT_INFO.name, "Nikon::ShotInfoZ9");
        assert_eq!(NIKON_Z9_SHOT_INFO.model_condition, Some("NIKON Z 9"));
        assert!(!NIKON_Z9_SHOT_INFO.tags.is_empty());
    }

    #[test]
    fn test_table_selection_z9() {
        let table = select_nikon_tag_table("NIKON Z 9");
        assert_eq!(table.name, "Nikon::ShotInfoZ9");
    }

    #[test]
    fn test_table_selection_d850() {
        let table = select_nikon_tag_table("NIKON D850");
        assert_eq!(table.name, "Nikon::Main");
    }

    #[test]
    fn test_table_selection_generic() {
        let table = select_nikon_tag_table("NIKON D780");
        assert_eq!(table.name, "Nikon::Main");
    }

    #[test]
    fn test_tag_name_lookup_quality() {
        let name = get_nikon_tag_name(0x0004, "NIKON D850");
        assert_eq!(name, Some("Quality"));
    }

    #[test]
    fn test_tag_name_lookup_white_balance() {
        let name = get_nikon_tag_name(0x0005, "NIKON D780");
        assert_eq!(name, Some("WhiteBalance"));
    }

    #[test]
    fn test_tag_name_lookup_unknown() {
        let name = get_nikon_tag_name(0xFFFF, "NIKON D850");
        assert_eq!(name, None);
    }

    #[test]
    fn test_encryption_key_tags_present() {
        // Verify critical encryption key tags are mapped
        let serial_tag = get_nikon_tag_name(0x001D, "NIKON D850");
        assert_eq!(serial_tag, Some("SerialNumber"));

        let count_tag = get_nikon_tag_name(0x00A7, "NIKON D850");
        assert_eq!(count_tag, Some("ShutterCount"));
    }

    #[test]
    fn test_quality_print_conv() {
        let value = TagValue::I32(3);
        let result = nikon_quality_conv(&value).unwrap();
        assert_eq!(result, "VGA Fine");

        let value = TagValue::I32(1);
        let result = nikon_quality_conv(&value).unwrap();
        assert_eq!(result, "VGA Basic");
    }

    #[test]
    fn test_white_balance_print_conv() {
        let value = TagValue::I32(0);
        let result = nikon_white_balance_conv(&value).unwrap();
        assert_eq!(result, "Auto");

        let value = TagValue::I32(2);
        let result = nikon_white_balance_conv(&value).unwrap();
        assert_eq!(result, "Daylight");
    }

    #[test]
    fn test_unknown_tag_conversion() {
        let value = TagValue::I32(999);
        let result = nikon_quality_conv(&value).unwrap();
        assert_eq!(result, "Unknown");

        let result = nikon_white_balance_conv(&value).unwrap();
        assert_eq!(result, "Unknown");
    }

    #[test]
    fn test_non_integer_tag_conversion() {
        let value = TagValue::String("invalid".to_string());
        let result = nikon_quality_conv(&value).unwrap();
        assert!(result.contains("Unknown"));
    }
}

#[cfg(test)]
mod lens_database_tests {
    use super::super::lens_database::*;

    #[test]
    fn test_lens_lookup_50mm_f18() {
        // Test AF Nikkor 50mm f/1.8 lookup
        let lens_data = [0x06, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x01];
        let result = lookup_nikon_lens(&lens_data);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "AF Nikkor 50mm f/1.8");
    }

    #[test]
    fn test_lens_lookup_z_mount_50mm() {
        // Test Nikkor Z 50mm f/1.8 S lookup
        let lens_data = [0xC4, 0x4C, 0x32, 0x32, 0x14, 0x14, 0xDF, 0x0E];
        let result = lookup_nikon_lens(&lens_data);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "Nikkor Z 50mm f/1.8 S");
    }

    #[test]
    fn test_lens_lookup_teleconverter() {
        // Test TC-20E teleconverter lookup
        let lens_data = [0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02];
        let result = lookup_nikon_lens(&lens_data);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "TC-20E");
    }

    #[test]
    fn test_lens_lookup_insufficient_data() {
        let short_data = [0x06, 0x00, 0x00]; // Only 3 bytes
        let result = lookup_nikon_lens(&short_data);

        assert!(result.is_none());
    }

    #[test]
    fn test_lens_lookup_no_match() {
        let unknown_data = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        let result = lookup_nikon_lens(&unknown_data);

        assert!(result.is_none());
    }

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
        assert_eq!(
            LensCategory::from_description("Unknown Lens"),
            LensCategory::Unknown
        );
    }

    #[test]
    fn test_get_lens_category() {
        let lens_data = [0x06, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x01];
        let category = get_nikon_lens_category(&lens_data);

        assert_eq!(category, LensCategory::Af);
    }

    #[test]
    fn test_get_lens_category_unknown() {
        let unknown_data = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        let category = get_nikon_lens_category(&unknown_data);

        assert_eq!(category, LensCategory::Unknown);
    }

    #[test]
    fn test_database_statistics() {
        let (total, categories) = get_database_stats();

        assert!(total > 0);
        assert!(!categories.is_empty());

        // Verify we have different lens categories represented
        assert!(categories.contains_key(&LensCategory::Af));
        assert!(categories.contains_key(&LensCategory::AfS));
        assert!(categories.contains_key(&LensCategory::Teleconverter));
    }

    #[test]
    fn test_get_lenses_by_category() {
        let af_s_lenses = get_lenses_by_category(LensCategory::AfS);
        assert!(!af_s_lenses.is_empty());

        // Verify all returned lenses are actually AF-S
        for lens in af_s_lenses {
            assert_eq!(lens.category, LensCategory::AfS);
            assert!(lens.description.contains("AF-S"));
        }
    }

    #[test]
    fn test_teleconverter_category() {
        let teleconverters = get_lenses_by_category(LensCategory::Teleconverter);
        assert!(!teleconverters.is_empty());

        for tc in teleconverters {
            assert_eq!(tc.category, LensCategory::Teleconverter);
            assert!(tc.description.contains("TC-"));
        }
    }

    #[test]
    fn test_third_party_lenses() {
        let third_party = get_lenses_by_category(LensCategory::ThirdParty);

        for lens in third_party {
            assert_eq!(lens.category, LensCategory::ThirdParty);
            assert!(
                lens.description.contains("Sigma")
                    || lens.description.contains("Tamron")
                    || lens.description.contains("Tokina")
            );
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::types::TagValue;

    #[test]
    fn test_nikon_signature_and_format_detection() {
        // Test complete detection flow
        assert!(detect_nikon_signature("NIKON CORPORATION"));

        let format3_data = b"\x02\x10\x00\x00";
        assert_eq!(
            detection::detect_nikon_format(format3_data),
            Some(detection::NikonFormat::Format3)
        );
    }

    #[test]
    fn test_offset_calculation_and_validation() {
        let format = detection::NikonFormat::Format3;
        let data_pos = 100;
        let base = offset_schemes::calculate_nikon_base_offset(format, data_pos);

        assert_eq!(base, 110);

        let validation = offset_schemes::validate_nikon_offset(format, data_pos, 500);
        assert!(validation.is_ok());
    }

    #[test]
    fn test_encryption_keys_and_tag_lookup() {
        let mut keys = encryption::NikonEncryptionKeys::new("NIKON D850".to_string());

        // Simulate finding encryption key tags
        let serial_tag = tags::get_nikon_tag_name(0x001D, "NIKON D850");
        assert_eq!(serial_tag, Some("SerialNumber"));

        keys.store_serial_key("D850SERIAL".to_string());
        keys.store_count_key(1000);

        assert!(keys.has_required_keys());
    }

    #[test]
    fn test_lens_and_tag_integration() {
        // Test lens lookup combined with tag processing
        let lens_data = [0x06, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x01];
        let lens_result = lens_database::lookup_nikon_lens(&lens_data);

        assert!(lens_result.is_some());

        // Test quality tag conversion
        let quality_value = TagValue::I32(3);
        let quality_result = tags::nikon_quality_conv(&quality_value);

        assert!(quality_result.is_ok());
        assert_eq!(quality_result.unwrap(), "VGA Fine");
    }
}
