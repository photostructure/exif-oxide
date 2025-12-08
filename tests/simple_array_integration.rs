use std::process::Command;

#[cfg(test)]
mod simple_array_integration_tests {
    use super::*;

    #[test]
    #[ignore = "Requires codegen/scripts/validate_arrays.pl which is not present"]
    fn test_nikon_xlat_arrays_match_exiftool() {
        // Run the validation script and check exit code
        let output = Command::new("perl")
            .args([
                "scripts/validate_arrays.pl",
                "config/Nikon_pm/simple_array.json",
            ])
            .current_dir("codegen")
            .output()
            .expect("Failed to run validation script - ensure perl is installed");

        if !output.status.success() {
            panic!(
                "Array validation failed!\nSTDOUT:\n{}\nSTDERR:\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        // Verify we got the success message
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("ALL ARRAYS VALIDATED SUCCESSFULLY"),
            "Expected success message not found in output:\n{}",
            stdout
        );
    }

    #[test]
    fn test_xlat_arrays_have_correct_size_and_key_values() {
        // Import the generated arrays
        use exif_oxide::generated::Nikon_pm::xlat_0::XLAT_0;
        use exif_oxide::generated::Nikon_pm::xlat_1::XLAT_1;

        // Test array sizes
        assert_eq!(XLAT_0.len(), 256, "XLAT_0 should have exactly 256 elements");
        assert_eq!(XLAT_1.len(), 256, "XLAT_1 should have exactly 256 elements");

        // Test XLAT_0 key values: first, middle, last
        assert_eq!(XLAT_0[0], 193, "XLAT_0[0] should be 193 (0xc1)");
        assert_eq!(
            XLAT_0[127], 47,
            "XLAT_0[127] should be 47 (0x2f) - middle value"
        );
        assert_eq!(
            XLAT_0[255], 199,
            "XLAT_0[255] should be 199 (0xc7) - last value"
        );

        // Test XLAT_1 key values: first, middle, last
        assert_eq!(XLAT_1[0], 167, "XLAT_1[0] should be 167 (0xa7)");
        assert_eq!(
            XLAT_1[127], 107,
            "XLAT_1[127] should be 107 (0x6b) - middle value"
        );
        assert_eq!(
            XLAT_1[255], 47,
            "XLAT_1[255] should be 47 (0x2f) - last value"
        );
    }

    #[test]
    fn test_xlat_arrays_are_accessible_at_runtime() {
        // Import the generated arrays
        use exif_oxide::generated::Nikon_pm::xlat_0::XLAT_0;
        use exif_oxide::generated::Nikon_pm::xlat_1::XLAT_1;

        // Test that we can access arrays with bounds checking
        assert_eq!(XLAT_0.first(), Some(&193));
        assert_eq!(XLAT_0.get(255), Some(&199));
        assert_eq!(XLAT_0.get(256), None); // Out of bounds

        assert_eq!(XLAT_1.first(), Some(&167));
        assert_eq!(XLAT_1.get(255), Some(&47));
        assert_eq!(XLAT_1.get(256), None); // Out of bounds

        // Test direct array access (this would panic on invalid bounds)
        assert_eq!(XLAT_0[42], XLAT_0[42]); // Tautology but tests direct access works
        assert_eq!(XLAT_1[100], XLAT_1[100]); // Tautology but tests direct access works
    }

    #[test]
    fn test_xlat_arrays_cryptographic_properties() {
        // Import the generated arrays
        use exif_oxide::generated::Nikon_pm::xlat_0::XLAT_0;
        use exif_oxide::generated::Nikon_pm::xlat_1::XLAT_1;
        use std::collections::HashSet;

        // Verify arrays contain diverse values (not all zeros, not all the same)
        let xlat0_set: HashSet<u8> = XLAT_0.iter().copied().collect();
        let xlat1_set: HashSet<u8> = XLAT_1.iter().copied().collect();

        // Cryptographic arrays should have good distribution
        assert!(
            xlat0_set.len() > 50,
            "XLAT_0 should have diverse values, got {} unique values",
            xlat0_set.len()
        );
        assert!(
            xlat1_set.len() > 50,
            "XLAT_1 should have diverse values, got {} unique values",
            xlat1_set.len()
        );

        // Arrays should be different from each other
        assert_ne!(XLAT_0[0], XLAT_1[0], "XLAT arrays should differ");
        assert_ne!(
            &XLAT_0[..],
            &XLAT_1[..],
            "XLAT_0 and XLAT_1 should be different arrays"
        );

        // Test that arrays span the full u8 range reasonably
        let xlat0_min = *XLAT_0.iter().min().unwrap();
        let xlat0_max = *XLAT_0.iter().max().unwrap();
        assert!(
            xlat0_max - xlat0_min > 200,
            "XLAT_0 should span most of u8 range"
        );

        let xlat1_min = *XLAT_1.iter().min().unwrap();
        let xlat1_max = *XLAT_1.iter().max().unwrap();
        assert!(
            xlat1_max - xlat1_min > 200,
            "XLAT_1 should span most of u8 range"
        );
    }
}
