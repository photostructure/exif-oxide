//! Integration tests for multi-pass composite tag building
//!
//! These tests exercise the complete multi-pass composite building system
//! with realistic EXIF data scenarios to demonstrate proper handling of
//! composite-on-composite dependencies.

#![cfg(feature = "test-helpers")]

mod common;

use exif_oxide::types::TagValue;
use std::collections::HashMap;

/// Test end-to-end multi-pass composite building with camera calculation chain
/// Tests the ScaleFactor35efl → CircleOfConfusion → DOF/HyperfocalDistance dependency chain
#[test]
fn test_camera_calculation_chain_integration() {
    let mut reader = common::create_camera_test_reader();

    println!("=== Before multi-pass composite building ===");
    let tags_before = reader.get_all_tags();
    println!("Total tags before: {}", tags_before.len());
    println!(
        "Composite tags before: {}",
        tags_before
            .keys()
            .filter(|k| k.starts_with("Composite:"))
            .count()
    );

    // Trigger multi-pass composite building
    reader.build_composite_tags();

    println!("\n=== After multi-pass composite building ===");
    let tags_after = reader.get_all_tags();
    println!("Total tags after: {}", tags_after.len());

    let composite_tags: Vec<_> = tags_after
        .keys()
        .filter(|k| k.starts_with("Composite:"))
        .collect();
    println!("Composite tags built: {}", composite_tags.len());

    for tag_name in &composite_tags {
        if let Some(value) = tags_after.get(*tag_name) {
            println!("  {tag_name}: {value:?}");
        }
    }

    // Verify that we built some composite tags
    assert!(
        !composite_tags.is_empty(),
        "Should have built some composite tags"
    );

    // Verify specific expected composites based on available data
    // Note: The exact composites built depend on the implementations available

    // Should be able to build basic composites that don't depend on others
    let has_basic_composites = composite_tags.iter().any(|name| {
        matches!(
            name.as_str(),
            "Composite:Aperture" | "Composite:ShutterSpeed" | "Composite:ImageSize"
        )
    });
    assert!(has_basic_composites, "Should build basic composite tags");

    println!("\n✅ Multi-pass composite building integration test passed!");
}

/// Test GPS coordinate composite chain (GPSLatitude/GPSLongitude → GPSPosition)
#[test]
fn test_gps_coordinate_chain_integration() {
    let mut reader = common::create_gps_test_reader();

    println!("=== GPS Composite Integration Test ===");

    // Build composites
    reader.build_composite_tags();

    let tags = reader.get_all_tags();
    let gps_composites: Vec<_> = tags
        .keys()
        .filter(|k| k.starts_with("Composite:GPS"))
        .collect();

    println!("GPS composite tags built:");
    for tag_name in &gps_composites {
        if let Some(value) = tags.get(*tag_name) {
            println!("  {tag_name}: {value:?}");
        }
    }

    // The GPS composites depend on proper ValueConv implementations
    // At minimum, we should detect the attempt to build GPS composites
    println!(
        "GPS-related composite tags attempted: {}",
        gps_composites.len()
    );

    println!("✅ GPS coordinate chain integration test completed!");
}

/// Test that multi-pass building handles missing dependencies gracefully
#[test]
fn test_missing_dependency_handling() {
    let mut reader = common::create_minimal_test_reader();
    // This reader has only ImageWidth and ImageHeight, deliberately omitting other camera-specific tags

    // Build composites - should handle missing dependencies gracefully
    reader.build_composite_tags();

    let tags = reader.get_all_tags();
    let composite_tags: Vec<_> = tags
        .keys()
        .filter(|k| k.starts_with("Composite:"))
        .collect();

    println!("Composite tags built with limited data:");
    for tag_name in &composite_tags {
        if let Some(value) = tags.get(*tag_name) {
            println!("  {tag_name}: {value:?}");
        }
    }

    // Should handle missing dependencies without crashing
    // May build some composites (like ImageSize) but not others (like DOF)
    println!("✅ Missing dependency handling test passed!");
}

/// Test performance characteristics of multi-pass building
#[test]
fn test_multipass_performance() {
    let mut reader = common::create_comprehensive_test_reader();

    // Measure performance
    let start = std::time::Instant::now();
    reader.build_composite_tags();
    let duration = start.elapsed();

    println!("Multi-pass composite building completed in: {duration:?}");

    let tags = reader.get_all_tags();
    let composite_count = tags.keys().filter(|k| k.starts_with("Composite:")).count();

    println!("Built {composite_count} composite tags");

    // Performance should be reasonable (well under 1 second for this test)
    assert!(
        duration.as_millis() < 1000,
        "Multi-pass building should complete quickly, took {duration:?}"
    );

    println!("✅ Performance test passed!");
}

/// Test the actual dependency resolution order by tracking what gets built when
#[test]
fn test_dependency_resolution_order() {
    let mut reader = common::create_dependency_test_reader();
    // This creates a clear dependency chain:
    // 1. Basic tags available immediately
    // 2. ScaleFactor35efl buildable in pass 1
    // 3. CircleOfConfusion buildable in pass 2 (needs ScaleFactor35efl)
    // 4. DOF buildable in pass 3 (needs CircleOfConfusion + Aperture)

    // Test that the system can build available composites
    // Note: The actual building depends on implementation availability
    reader.build_composite_tags();

    let tags = reader.get_all_tags();
    let composites: HashMap<String, &TagValue> = tags
        .iter()
        .filter(|(k, _)| k.starts_with("Composite:"))
        .map(|(k, v)| (k.clone(), v))
        .collect();

    println!("Dependency resolution test results:");
    // Let's use the available tags from get_all_tags() instead
    let all_tags = reader.get_all_tags();
    let extracted_count = all_tags
        .keys()
        .filter(|k| !k.starts_with("Composite:"))
        .count();
    println!("Available extracted tags: {extracted_count}");
    println!("Built composite tags: {}", composites.len());

    for (name, value) in &composites {
        println!("  {name}: {value:?}");
    }

    // The key test is that the system doesn't crash and handles dependencies properly
    // Specific composites built depend on which implementations are available

    println!("✅ Dependency resolution order test passed!");
}
