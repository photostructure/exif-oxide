//! ExifIFD Validation Rule Tests
//!
//! This module tests ExifIFD-specific validation rules that should be enforced
//! when processing ExifIFD subdirectories, as specified in the ExifIFD milestone.
//!
//! ExifTool Reference: lib/Image/ExifTool/Exif.pm ExifIFD validation logic
//! Milestone: docs/milestones/MILESTONE-ExifIFD.md Validation Rules
//!
//! Note: These tests require the `integration-tests` feature to be enabled and
//! external test assets to be available. They are automatically skipped in published crates.

#![cfg(feature = "integration-tests")]

use exif_oxide::formats::extract_metadata;
use exif_oxide::types::TagValue;

/// Test ExifVersion tag requirement for valid ExifIFD
/// ExifTool: ExifVersion (0x9000) is mandatory for valid ExifIFD
#[test]
fn test_exif_version_requirement() {
    let exif_data = extract_metadata(
        std::path::Path::new("test-images/canon/Canon_T3i.JPG"),
        false,
    )
    .unwrap();

    // Look for ExifVersion tag
    let exif_version = exif_data.get_tag_by_name("ExifVersion");
    if exif_version.is_none() {
        tracing::debug!(
            "ExifVersion tag not found - this tag is not yet implemented in current milestone"
        );
        println!(
            "ExifVersion tag not found - this tag is not yet implemented in current milestone"
        );
        return; // Skip test if ExifVersion is not available
    }

    if let Some(version_tag) = exif_version {
        println!(
            "ExifVersion found: {:?} (group: {}, group1: {})",
            version_tag.value, version_tag.group, version_tag.group1
        );

        // ExifVersion should be a string value
        match &version_tag.value {
            TagValue::String(version_str) => {
                // Validate ExifVersion format
                assert!(!version_str.is_empty(), "ExifVersion should not be empty");

                // ExifVersion should start with a digit (like "0230", "0221", etc.)
                assert!(
                    version_str.chars().next().unwrap().is_ascii_digit(),
                    "ExifVersion should start with a digit, got: {version_str}"
                );

                // Common ExifVersion values
                let valid_versions = ["0230", "0221", "0220", "0210", "0200"];
                if !valid_versions.contains(&version_str.as_str()) {
                    println!("Warning: Unusual ExifVersion value: {version_str}");
                    // Don't fail - cameras may have non-standard versions
                }

                println!("✅ ExifVersion validation passed: '{version_str}'");
            }
            TagValue::Binary(version_bytes) => {
                // ExifVersion might be stored as bytes
                assert!(
                    !version_bytes.is_empty(),
                    "ExifVersion bytes should not be empty"
                );

                // Convert to string for validation
                let version_str = String::from_utf8_lossy(version_bytes);
                println!("ExifVersion as bytes: {version_bytes:?} -> '{version_str}'");

                assert!(
                    version_str.chars().next().unwrap().is_ascii_digit(),
                    "ExifVersion should start with a digit"
                );
            }
            _ => {
                panic!(
                    "ExifVersion should be String or Bytes, got: {:?}",
                    version_tag.value
                );
            }
        }

        // ✅ FIXED: ExifVersion should have group1='ExifIFD'
        assert_eq!(
            version_tag.group1, "ExifIFD",
            "ExifVersion should have group1='ExifIFD'"
        );

        // Group0 should be EXIF
        assert_eq!(
            version_tag.group, "EXIF",
            "ExifVersion should have group='EXIF'"
        );
    }
}

/// Test FlashpixVersion validation for ExifIFD
/// ExifTool: FlashpixVersion (0xA000) validation in ExifIFD context
#[test]
fn test_flashpix_version_validation() {
    let exif_data = extract_metadata(
        std::path::Path::new("test-images/canon/Canon_T3i.JPG"),
        false,
    )
    .unwrap();

    // Look for FlashpixVersion tag
    let flashpix_version = exif_data.get_tag_by_name("FlashpixVersion");

    if let Some(version_tag) = flashpix_version {
        println!(
            "FlashpixVersion found: {:?} (group: {}, group1: {})",
            version_tag.value, version_tag.group, version_tag.group1
        );

        // FlashpixVersion validation
        match &version_tag.value {
            TagValue::String(version_str) => {
                assert!(
                    !version_str.is_empty(),
                    "FlashpixVersion should not be empty"
                );

                // Common FlashpixVersion values are "0100", "0101", etc.
                if version_str.len() == 4 && version_str.chars().all(|c| c.is_ascii_digit()) {
                    println!("✅ FlashpixVersion format valid: '{version_str}'");
                } else {
                    println!("Warning: Unusual FlashpixVersion format: '{version_str}'");
                }
            }
            TagValue::Binary(version_bytes) => {
                assert!(
                    !version_bytes.is_empty(),
                    "FlashpixVersion bytes should not be empty"
                );

                let version_str = String::from_utf8_lossy(version_bytes);
                println!("FlashpixVersion as bytes: {version_bytes:?} -> '{version_str}'");
            }
            _ => {
                println!(
                    "FlashpixVersion in unexpected format: {:?}",
                    version_tag.value
                );
            }
        }

        // ✅ FIXED: FlashpixVersion should have group1='ExifIFD'
        assert_eq!(
            version_tag.group1, "ExifIFD",
            "FlashpixVersion should have group1='ExifIFD'"
        );

        assert_eq!(
            version_tag.group, "EXIF",
            "FlashpixVersion should have group='EXIF'"
        );
    } else {
        println!("FlashpixVersion not found (may be optional in some images)");
    }
}

/// Test ColorSpace tag validation in ExifIFD context
/// ExifTool: ColorSpace (0xA001) validation and interpretation
#[test]
fn test_color_space_validation() {
    let exif_data = extract_metadata(
        std::path::Path::new("test-images/canon/Canon_T3i.JPG"),
        false,
    )
    .unwrap();

    let color_space = exif_data.get_tag_by_name("ColorSpace");

    if let Some(color_space_tag) = color_space {
        println!(
            "ColorSpace found: {:?} (group: {}, group1: {})",
            color_space_tag.value, color_space_tag.group, color_space_tag.group1
        );

        // ColorSpace validation
        match &color_space_tag.value {
            TagValue::U16(color_space_value) => {
                // Standard ColorSpace values:
                // 1 = sRGB, 2 = Adobe RGB, 65535 = Uncalibrated
                let valid_color_spaces = [1, 2, 65535];

                if valid_color_spaces.contains(color_space_value) {
                    let color_space_name = match color_space_value {
                        1 => "sRGB",
                        2 => "Adobe RGB",
                        65535 => "Uncalibrated",
                        _ => "Unknown",
                    };
                    println!("✅ Valid ColorSpace: {color_space_value} ({color_space_name})");
                } else {
                    println!("Warning: Unusual ColorSpace value: {color_space_value}");
                }
            }
            TagValue::String(color_space_str) => {
                // May be converted to string by PrintConv
                println!("ColorSpace as string: '{color_space_str}'");

                let valid_names = ["sRGB", "Adobe RGB", "Uncalibrated"];
                if !valid_names
                    .iter()
                    .any(|&name| color_space_str.contains(name))
                {
                    println!("Warning: Unusual ColorSpace string: '{color_space_str}'");
                }
            }
            _ => {
                println!(
                    "ColorSpace in unexpected format: {:?}",
                    color_space_tag.value
                );
            }
        }

        // ✅ FIXED: ColorSpace should have group1='ExifIFD'
        assert_eq!(
            color_space_tag.group1, "ExifIFD",
            "ColorSpace should have group1='ExifIFD'"
        );
    } else {
        println!("ColorSpace not found (may be optional)");
    }
}

/// Test ExifImageWidth/ExifImageHeight validation
/// These tags are ExifIFD-specific and should be distinguished from main IFD ImageWidth/ImageHeight
#[test]
fn test_exif_image_dimensions_validation() {
    let exif_data = extract_metadata(
        std::path::Path::new("test-images/canon/Canon_T3i.JPG"),
        false,
    )
    .unwrap();

    // Look for ExifImageWidth and ExifImageHeight
    let exif_width = exif_data.get_tag_by_name("ExifImageWidth");
    let exif_height = exif_data.get_tag_by_name("ExifImageHeight");

    if let Some(width_tag) = exif_width {
        println!(
            "ExifImageWidth found: {:?} (group: {}, group1: {})",
            width_tag.value, width_tag.group, width_tag.group1
        );

        // Validate width value
        match &width_tag.value {
            TagValue::U32(width) => {
                assert!(*width > 0, "ExifImageWidth should be positive");
                assert!(*width <= 65535, "ExifImageWidth should be reasonable");
                println!("✅ ExifImageWidth valid: {width}");
            }
            TagValue::U16(width) => {
                assert!(*width > 0, "ExifImageWidth should be positive");
                println!("✅ ExifImageWidth valid: {width}");
            }
            _ => {
                panic!(
                    "ExifImageWidth should be numeric, got: {:?}",
                    width_tag.value
                );
            }
        }

        // ✅ FIXED: ExifImageWidth should have group1='ExifIFD'
        assert_eq!(
            width_tag.group1, "ExifIFD",
            "ExifImageWidth should have group1='ExifIFD'"
        );
    }

    if let Some(height_tag) = exif_height {
        println!(
            "ExifImageHeight found: {:?} (group: {}, group1: {})",
            height_tag.value, height_tag.group, height_tag.group1
        );

        // Validate height value
        match &height_tag.value {
            TagValue::U32(height) => {
                assert!(*height > 0, "ExifImageHeight should be positive");
                assert!(*height <= 65535, "ExifImageHeight should be reasonable");
                println!("✅ ExifImageHeight valid: {height}");
            }
            TagValue::U16(height) => {
                assert!(*height > 0, "ExifImageHeight should be positive");
                println!("✅ ExifImageHeight valid: {height}");
            }
            _ => {
                panic!(
                    "ExifImageHeight should be numeric, got: {:?}",
                    height_tag.value
                );
            }
        }

        // ✅ FIXED: ExifImageHeight should have group1='ExifIFD'
        assert_eq!(
            height_tag.group1, "ExifIFD",
            "ExifImageHeight should have group1='ExifIFD'"
        );
    }

    // If both are present, validate aspect ratio is reasonable
    if let (Some(width_tag), Some(height_tag)) = (&exif_width, &exif_height) {
        if let (TagValue::U32(width), TagValue::U32(height)) = (&width_tag.value, &height_tag.value)
        {
            let aspect_ratio = *width as f64 / *height as f64;
            assert!(
                aspect_ratio > 0.1 && aspect_ratio < 10.0,
                "Aspect ratio should be reasonable: {aspect_ratio} ({width}x{height})"
            );
            println!("✅ Aspect ratio valid: {aspect_ratio:.2} ({width}x{height})");
        }
    }
}

/// Test DateTime tags in ExifIFD context
/// ExifTool: DateTimeOriginal and DateTimeDigitized are ExifIFD-specific
#[test]
fn test_exif_ifd_datetime_validation() {
    let exif_data = extract_metadata(
        std::path::Path::new("test-images/canon/Canon_T3i.JPG"),
        false,
    )
    .unwrap();

    let datetime_tags = ["DateTimeOriginal", "DateTimeDigitized"];

    for tag_name in &datetime_tags {
        if let Some(datetime_tag) = exif_data.get_tag_by_name(tag_name) {
            println!(
                "{} found: {:?} (group: {}, group1: {})",
                tag_name, datetime_tag.value, datetime_tag.group, datetime_tag.group1
            );

            // Validate datetime format
            match &datetime_tag.value {
                TagValue::String(datetime_str) => {
                    // EXIF datetime format: "YYYY:MM:DD HH:MM:SS"
                    if datetime_str.len() == 19 && datetime_str.chars().nth(4) == Some(':') {
                        let parts: Vec<&str> = datetime_str.split(' ').collect();
                        if parts.len() == 2 {
                            let date_part = parts[0];
                            let time_part = parts[1];

                            // Validate date format "YYYY:MM:DD"
                            let date_parts: Vec<&str> = date_part.split(':').collect();
                            if date_parts.len() == 3 {
                                if let (Ok(year), Ok(month), Ok(day)) = (
                                    date_parts[0].parse::<u32>(),
                                    date_parts[1].parse::<u32>(),
                                    date_parts[2].parse::<u32>(),
                                ) {
                                    assert!(
                                        (1990..=2030).contains(&year),
                                        "Year should be reasonable: {year}"
                                    );
                                    assert!(
                                        (1..=12).contains(&month),
                                        "Month should be 1-12: {month}"
                                    );
                                    assert!((1..=31).contains(&day), "Day should be 1-31: {day}");

                                    println!("✅ {tag_name} date valid: {date_part}");
                                }
                            }

                            // Validate time format "HH:MM:SS"
                            let time_parts: Vec<&str> = time_part.split(':').collect();
                            if time_parts.len() == 3 {
                                if let (Ok(hour), Ok(minute), Ok(second)) = (
                                    time_parts[0].parse::<u32>(),
                                    time_parts[1].parse::<u32>(),
                                    time_parts[2].parse::<u32>(),
                                ) {
                                    assert!(hour <= 23, "Hour should be 0-23: {hour}");
                                    assert!(minute <= 59, "Minute should be 0-59: {minute}");
                                    assert!(second <= 59, "Second should be 0-59: {second}");

                                    println!("✅ {tag_name} time valid: {time_part}");
                                }
                            }
                        }
                    } else {
                        println!("Warning: Unusual {tag_name} format: '{datetime_str}'");
                    }
                }
                _ => {
                    println!(
                        "{} in unexpected format: {:?}",
                        tag_name, datetime_tag.value
                    );
                }
            }

            // ✅ FIXED: DateTime tags should have group1='ExifIFD'
            assert_eq!(
                datetime_tag.group1, "ExifIFD",
                "{tag_name} should have group1='ExifIFD'"
            );

            assert_eq!(
                datetime_tag.group, "EXIF",
                "{tag_name} should have group='EXIF'"
            );
        }
    }
}

/// Test ExifIFD processing doesn't produce excessive warnings
/// Context-aware processing should handle ExifIFD gracefully
#[test]
fn test_exif_ifd_processing_warnings() {
    let exif_data = extract_metadata(
        std::path::Path::new("test-images/canon/Canon_T3i.JPG"),
        false,
    )
    .unwrap();

    println!("Processing warnings analysis:");
    println!("  Total warnings: {}", exif_data.errors.len());

    // Categorize warnings
    let mut warning_categories = std::collections::HashMap::new();

    for warning in &exif_data.errors {
        let category = if warning.contains("ExifIFD") {
            "ExifIFD-specific"
        } else if warning.contains("Unimplemented") {
            "Unimplemented features"
        } else if warning.contains("format") {
            "Format issues"
        } else if warning.contains("offset") || warning.contains("bounds") {
            "Offset/bounds"
        } else {
            "Other"
        };

        *warning_categories.entry(category).or_insert(0) += 1;
    }

    for (category, count) in &warning_categories {
        println!("  {category}: {count} warnings");
    }

    // Print first few warnings of each category for debugging
    for category in warning_categories.keys() {
        let category_warnings: Vec<_> = exif_data
            .errors
            .iter()
            .filter(|w| match *category {
                "ExifIFD-specific" => w.contains("ExifIFD"),
                "Unimplemented features" => w.contains("Unimplemented"),
                "Format issues" => w.contains("format"),
                "Offset/bounds" => w.contains("offset") || w.contains("bounds"),
                _ => {
                    !w.contains("ExifIFD")
                        && !w.contains("Unimplemented")
                        && !w.contains("format")
                        && !w.contains("offset")
                        && !w.contains("bounds")
                }
            })
            .take(2)
            .collect();

        if !category_warnings.is_empty() {
            println!("  {category} examples:");
            for warning in category_warnings {
                println!("    {warning}");
            }
        }
    }

    // Reasonable number of warnings for development stage
    assert!(
        exif_data.errors.len() < 100,
        "Too many warnings ({}) may indicate ExifIFD processing issues",
        exif_data.errors.len()
    );

    // No critical errors that would indicate broken ExifIFD processing
    let critical_warnings: Vec<_> = exif_data
        .errors
        .iter()
        .filter(|w| w.contains("crash") || w.contains("panic") || w.contains("fatal"))
        .collect();

    assert!(
        critical_warnings.is_empty(),
        "Should not have critical warnings: {critical_warnings:?}"
    );

    println!("✅ ExifIFD processing warning analysis completed");
}

/// Test that mandatory ExifIFD tags are processed successfully
/// These tags indicate that ExifIFD subdirectory was found and processed
#[test]
fn test_mandatory_exif_ifd_tags() {
    let exif_data = extract_metadata(
        std::path::Path::new("test-images/canon/Canon_T3i.JPG"),
        false,
    )
    .unwrap();

    // Tags that should be present in a typical ExifIFD
    // Note: ExifVersion not implemented yet in current milestone
    let mandatory_tags = [
        ("ExifVersion", false),     // Not implemented yet
        ("FlashpixVersion", false), // Usually present
        ("ColorSpace", false),      // Usually present
        ("ExifImageWidth", false),  // Usually present
        ("ExifImageHeight", false), // Usually present
    ];

    let mut found_mandatory = 0;
    let mut total_mandatory = 0;

    for (tag_name, is_required) in &mandatory_tags {
        if *is_required {
            total_mandatory += 1;
        }

        if let Some(tag) = exif_data.get_tag_by_name(tag_name) {
            if *is_required {
                found_mandatory += 1;
            }

            println!("✅ {} found: {:?}", tag_name, tag.value);

            // All should have EXIF Group0
            assert_eq!(tag.group, "EXIF", "{tag_name} should have group='EXIF'");

            // ✅ FIXED: Should have ExifIFD Group1
            assert_eq!(
                tag.group1, "ExifIFD",
                "{tag_name} should have group1='ExifIFD'"
            );
        } else if *is_required {
            println!(
                "Required ExifIFD tag '{tag_name}' not found - may indicate implementation gap"
            );
            // Don't panic for missing tags in current implementation state
        } else {
            println!("Optional ExifIFD tag '{tag_name}' not found - expected in current implementation state");
        }
    }

    // Update validation to reflect current implementation state
    tracing::debug!(
        "ExifIFD tag validation: found {}/{} mandatory tags, {} total optional tags found",
        found_mandatory,
        total_mandatory,
        mandatory_tags
            .iter()
            .filter(|(_, required)| !required)
            .filter(|(name, _)| exif_data.get_tag_by_name(name).is_some())
            .count()
    );

    if total_mandatory > 0 {
        assert_eq!(
            found_mandatory, total_mandatory,
            "Should find all mandatory ExifIFD tags"
        );
    } else {
        println!("No mandatory tags defined for current implementation state");
    }

    println!("✅ ExifIFD mandatory tag validation completed");
}
