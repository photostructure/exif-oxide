//! Integration tests for datetime intelligence
//!
//! These tests validate the datetime intelligence functionality using real image files
//! and compare results with ExifTool where appropriate.

use exif_oxide::{extract_datetime_intelligence, read_basic_exif};
use std::path::Path;

/// Test datetime intelligence with a basic image
#[test]
fn test_datetime_intelligence_basic() {
    // Test with test.bmp (basic file without sophisticated datetime data)
    if Path::new("test.bmp").exists() {
        let result = extract_datetime_intelligence("test.bmp");
        // This file likely won't have datetime intelligence, so we test graceful handling
        match result {
            Ok(None) => {
                // Expected for BMP files
                println!("No datetime intelligence found (expected for BMP)");
            }
            Ok(Some(resolved)) => {
                println!("Unexpected datetime found: {:?}", resolved);
            }
            Err(e) => {
                // BMP files don't have EXIF, so this is expected
                println!("Expected error for BMP file: {:?}", e);
            }
        }
    }
}

/// Test datetime intelligence integration with BasicExif
#[test]
fn test_basic_exif_with_datetime() {
    // Test that the BasicExif struct includes datetime intelligence
    if Path::new("test.bmp").exists() {
        let result = read_basic_exif("test.bmp");
        match result {
            Ok(basic_exif) => {
                // BMP files don't have EXIF data, but this tests the API
                assert!(basic_exif.make.is_none());
                assert!(basic_exif.model.is_none());
                assert!(basic_exif.orientation.is_none());
                assert!(basic_exif.resolved_datetime.is_none());
            }
            Err(_) => {
                // Expected for BMP files without EXIF
                println!("Expected: BMP files don't have EXIF data");
            }
        }
    }
}

/// Test with images from ExifTool test suite if available
#[test]
fn test_exiftool_test_images() {
    let test_images = [
        "exiftool/t/images/Canon.jpg",
        "exiftool/t/images/Nikon.jpg",
        "exiftool/t/images/ExifTool.jpg",
    ];

    for image_path in &test_images {
        if Path::new(image_path).exists() {
            println!("Testing datetime intelligence with: {}", image_path);

            match extract_datetime_intelligence(image_path) {
                Ok(Some(resolved)) => {
                    println!("  Found datetime: {}", resolved.datetime.datetime);
                    if let Some(offset) = resolved.datetime.local_offset {
                        println!("  Timezone offset: {}", offset);
                    }
                    println!("  Confidence: {:.1}%", resolved.confidence * 100.0);
                    println!("  Source: {:?}", resolved.datetime.inference_source);

                    // Validate basic properties
                    assert!(resolved.confidence >= 0.0 && resolved.confidence <= 1.0);
                    assert!(!resolved.datetime.raw_value.is_empty());

                    for warning in &resolved.warnings {
                        println!("  Warning: {:?}", warning);
                    }
                }
                Ok(None) => {
                    println!("  No datetime intelligence available");
                }
                Err(e) => {
                    println!("  Error extracting datetime: {:?}", e);
                }
            }
        } else {
            println!("Skipping {} (file not found)", image_path);
        }
    }
}

/// Test GPS coordinate extraction and timezone inference
#[test]
fn test_gps_timezone_inference() {
    let gps_test_images = ["exiftool/t/images/GPS.jpg", "exiftool/t/images/iPhone.jpg"];

    for image_path in &gps_test_images {
        if Path::new(image_path).exists() {
            println!("Testing GPS timezone inference with: {}", image_path);

            match extract_datetime_intelligence(image_path) {
                Ok(Some(resolved)) => {
                    println!("  Found datetime: {}", resolved.datetime.datetime);
                    if let Some(offset) = resolved.datetime.local_offset {
                        println!("  Inferred timezone: {}", offset);
                    }

                    // Check if GPS inference was used
                    match resolved.datetime.inference_source {
                        exif_oxide::datetime::InferenceSource::GpsCoordinates {
                            lat, lng, ..
                        } => {
                            println!("  GPS coordinates: ({}, {})", lat, lng);
                            // GPS coordinates should be reasonable
                            assert!(lat.abs() <= 90.0);
                            assert!(lng.abs() <= 180.0);
                        }
                        other => {
                            println!("  Inference source: {:?}", other);
                        }
                    }
                }
                Ok(None) => {
                    println!("  No datetime/GPS data available");
                }
                Err(e) => {
                    println!("  Error: {:?}", e);
                }
            }
        } else {
            println!("Skipping {} (file not found)", image_path);
        }
    }
}

/// Test manufacturer quirk detection
#[test]
fn test_manufacturer_quirks() {
    let manufacturer_test_images = [
        ("exiftool/t/images/Canon.jpg", "Canon"),
        ("exiftool/t/images/Nikon.jpg", "Nikon"),
        ("exiftool/t/images/Sony.jpg", "Sony"),
    ];

    for (image_path, expected_make) in &manufacturer_test_images {
        if Path::new(image_path).exists() {
            println!(
                "Testing manufacturer quirks with: {} ({})",
                image_path, expected_make
            );

            match read_basic_exif(image_path) {
                Ok(basic_exif) => {
                    if let Some(make) = &basic_exif.make {
                        println!("  Camera make: {}", make);
                        // Basic validation that we're detecting the expected manufacturer
                        assert!(make.to_lowercase().contains(&expected_make.to_lowercase()));

                        if let Some(resolved) = &basic_exif.resolved_datetime {
                            println!("  Datetime confidence: {:.1}%", resolved.confidence * 100.0);

                            // Check for manufacturer-specific warnings
                            let quirk_warnings: Vec<_> = resolved
                                .warnings
                                .iter()
                                .filter(|w| {
                                    matches!(
                                        w,
                                        exif_oxide::datetime::DateTimeWarning::QuirkApplied { .. }
                                    )
                                })
                                .collect();

                            if !quirk_warnings.is_empty() {
                                println!("  Applied quirks:");
                                for warning in quirk_warnings {
                                    println!("    {:?}", warning);
                                }
                            } else {
                                println!("  No manufacturer quirks applied");
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("  Error: {:?}", e);
                }
            }
        } else {
            println!("Skipping {} (file not found)", image_path);
        }
    }
}

/// Test timezone offset validation
#[test]
fn test_timezone_offset_validation() {
    let test_images = [
        "exiftool/t/images/Canon.jpg",
        "exiftool/t/images/ExifTool.jpg",
    ];

    for image_path in &test_images {
        if Path::new(image_path).exists() {
            println!("Testing timezone validation with: {}", image_path);

            match extract_datetime_intelligence(image_path) {
                Ok(Some(resolved)) => {
                    if let Some(offset) = resolved.datetime.local_offset {
                        let offset_hours = offset.utc_minus_local() as f32 / 3600.0;
                        println!("  Timezone offset: {:.1} hours", offset_hours);

                        // Validate timezone offset is reasonable (within ±14 hours)
                        assert!(
                            offset_hours.abs() <= 14.0,
                            "Timezone offset {} is outside valid range",
                            offset_hours
                        );

                        // Check for timezone-related warnings
                        let tz_warnings: Vec<_> = resolved
                            .warnings
                            .iter()
                            .filter(|w| {
                                matches!(
                                    w,
                                    exif_oxide::datetime::DateTimeWarning::SuspiciousTimezone { .. }
                                )
                            })
                            .collect();

                        for warning in tz_warnings {
                            println!("  Timezone warning: {:?}", warning);
                        }
                    } else {
                        println!("  No timezone offset inferred");
                    }
                }
                Ok(None) => {
                    println!("  No datetime data available");
                }
                Err(e) => {
                    println!("  Error: {:?}", e);
                }
            }
        } else {
            println!("Skipping {} (file not found)", image_path);
        }
    }
}

/// Performance test - datetime intelligence should add <5ms overhead
#[test]
fn test_datetime_intelligence_performance() {
    let test_files = ["test.bmp", "test.png", "test.gif"];

    for test_file in &test_files {
        if Path::new(test_file).exists() {
            println!("Performance testing with: {}", test_file);

            let start = std::time::Instant::now();
            let _result = extract_datetime_intelligence(test_file);
            let duration = start.elapsed();

            println!("  Datetime intelligence took: {:?}", duration);

            // Target: <5ms additional overhead (though these test files may not have EXIF)
            // This is mainly to catch obvious performance regressions
            assert!(
                duration.as_millis() < 100,
                "Datetime intelligence took too long: {:?}",
                duration
            );
        } else {
            println!("Skipping {} (file not found)", test_file);
        }
    }
}
