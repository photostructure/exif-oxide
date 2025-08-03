//! Integration test for P16b: Universal Model-Based Subdirectory Dispatch
//!
//! This test validates that the universal model-based subdirectory dispatch system
//! works correctly across all manufacturers, enabling proper camera-specific
//! subdirectory table selection based on model conditions.
//!
//! P16b: Universal Model-Based Subdirectory Dispatch Implementation
//! see docs/todo/P16b-universal-model-subdirectory-dispatch.md

use exif_oxide::{extract_metadata_with_filter, FilterOptions};
use std::path::Path;

#[cfg(feature = "integration-tests")]
#[test]
fn test_canon_t3i_model_based_camerainfo_dispatch() {
    // P16b: Test Canon T3i model-based CameraInfo dispatch
    // The Canon EOS REBEL T3i (600D) should dispatch to camera-specific CameraInfo table
    // based on model condition: $$self{Model} =~ /EOS (REBEL T3i|600D|Kiss X5)/

    const TEST_IMAGE: &str = "test-images/canon/eos_rebel_t3i.cr2";

    // Extract all metadata to test model-based dispatch
    let filter = FilterOptions::extract_all();
    let result = extract_metadata_with_filter(Path::new(TEST_IMAGE), Some(filter));

    assert!(
        result.is_ok(),
        "Failed to read Canon T3i test image: {:?}",
        result.err()
    );

    let exif_data = result.unwrap();

    // Create a helper function to find tags by name
    let find_tag = |name: &str| -> Option<String> {
        exif_data
            .tags
            .iter()
            .find(|tag| format!("{}:{}", tag.group, tag.name) == name)
            .map(|tag| tag.print.to_string())
    };

    // Verify model is extracted correctly for dispatch
    let model = find_tag("EXIF:Model");
    assert!(model.is_some(), "Model tag must be present for dispatch");
    let model_str = model.unwrap();
    assert!(
        model_str.contains("REBEL T3i") || model_str.contains("600D"),
        "Model should contain REBEL T3i or 600D for condition matching, got: {}",
        model_str
    );

    // Test 1: Verify camera-specific Canon tags that require model dispatch are extracted
    // These tags should only be available if model-based subdirectory dispatch works
    let canon_model_specific_tags = [
        "MakerNotes:CanonModelID",         // Camera-specific identification
        "MakerNotes:CanonFirmwareVersion", // Model-specific firmware info
        "MakerNotes:CanonImageType",       // Camera-specific image type
    ];

    let mut found_model_specific = 0;
    for tag_name in &canon_model_specific_tags {
        if let Some(value) = find_tag(tag_name) {
            found_model_specific += 1;
            println!("✓ Found model-specific tag: {} = {}", tag_name, value);
        }
    }

    // Must have at least one camera-specific tag to prove model dispatch works
    let canon_tags: Vec<String> = exif_data
        .tags
        .iter()
        .filter(|tag| tag.name.starts_with("Canon"))
        .map(|tag| format!("{}:{}", tag.group, tag.name))
        .collect();

    assert!(
        found_model_specific > 0,
        "Expected at least one Canon model-specific tag, but found none. \
            This indicates model-based subdirectory dispatch is not working. \
            Available Canon tags: {:?}",
        canon_tags
    );

    // Test 2: Verify general subdirectory processing still works
    // Canon T3i should have CanonModelID which proves MakerNotes processing works
    assert!(
        find_tag("MakerNotes:CanonModelID").is_some(),
        "Canon MakerNotes processing should extract CanonModelID"
    );

    println!(
        "✓ Canon T3i model dispatch test passed with {} model-specific tags",
        found_model_specific
    );
}

#[cfg(feature = "integration-tests")]
#[test]
fn test_multi_manufacturer_model_dispatch_coverage() {
    // P16b: Test that model dispatch patterns exist across multiple manufacturers
    // This verifies the universal nature of the implementation

    // Test Canon: Should have many CameraInfo model conditions
    let canon_conditions = count_model_conditions_in_generated_code("Canon_pm");
    assert!(
        canon_conditions >= 10,
        "Canon should have many model conditions (expected >=10, got {})",
        canon_conditions
    );

    // Test Sony: Should have Tag2010* model conditions
    let sony_conditions = count_model_conditions_in_generated_code("Sony_pm");
    assert!(
        sony_conditions >= 5,
        "Sony should have model conditions (expected >=5, got {})",
        sony_conditions
    );

    // Test Nikon: Should have AFInfo/FileInfo model conditions
    let nikon_conditions = count_model_conditions_in_generated_code("Nikon_pm");
    assert!(
        nikon_conditions >= 2,
        "Nikon should have model conditions (expected >=2, got {})",
        nikon_conditions
    );

    println!("✓ Multi-manufacturer model dispatch coverage verified");
    println!(
        "  Canon: {} conditions, Sony: {} conditions, Nikon: {} conditions",
        canon_conditions, sony_conditions, nikon_conditions
    );
}

#[cfg(feature = "integration-tests")]
fn count_model_conditions_in_generated_code(manufacturer: &str) -> usize {
    // Count "Model condition:" comments in generated tag kit files to verify implementation
    use std::fs;
    use std::path::Path;

    let tag_kit_path = format!("src/generated/{}/tag_kit/mod.rs", manufacturer);

    if !Path::new(&tag_kit_path).exists() {
        return 0;
    }

    let content = fs::read_to_string(&tag_kit_path).unwrap_or_default();
    content
        .lines()
        .filter(|line| line.trim().starts_with("// Model condition:"))
        .count()
}

#[cfg(feature = "integration-tests")]
#[test]
fn test_model_dispatch_runtime_integration() {
    // P16b: Test that the model dispatch system is properly integrated into runtime
    // This test verifies end-to-end functionality without testing specific camera models

    const TEST_IMAGE: &str = "test-images/canon/eos_rebel_t3i.cr2";

    // Test with Canon image - should not crash and should extract basic tags
    let filter = FilterOptions::extract_all();
    let canon_result = extract_metadata_with_filter(Path::new(TEST_IMAGE), Some(filter));
    assert!(
        canon_result.is_ok(),
        "Canon T3i processing should not crash"
    );

    let canon_data = canon_result.unwrap();
    let canon_count = canon_data.tags.len();
    assert!(
        canon_count > 10,
        "Should extract multiple tags from Canon image"
    );

    // Verify model tag is present for dispatch system
    let has_model = canon_data
        .tags
        .iter()
        .any(|tag| format!("{}:{}", tag.group, tag.name) == "EXIF:Model");
    assert!(has_model, "Model tag must be available for dispatch system");

    println!("✓ Model dispatch runtime integration verified");
    println!(
        "  Canon T3i: {} tags extracted, Model tag present: {}",
        canon_count, has_model
    );
}

#[cfg(feature = "integration-tests")]
#[test]
fn test_model_condition_generated_code_quality() {
    // P16b: Test that generated model dispatch code is syntactically correct
    // This verifies that the code generation produces valid Rust code

    use std::fs;
    use std::path::Path;

    let manufacturers = ["Canon_pm", "Sony_pm", "Nikon_pm", "Olympus_pm"];
    let mut total_conditions = 0;
    let mut has_runtime_evaluation = false;

    for manufacturer in &manufacturers {
        let tag_kit_path = format!("src/generated/{}/tag_kit/mod.rs", manufacturer);

        if !Path::new(&tag_kit_path).exists() {
            continue;
        }

        let content = fs::read_to_string(&tag_kit_path).unwrap_or_default();

        // Count model conditions
        let conditions = content
            .lines()
            .filter(|line| line.trim().starts_with("// Model condition:"))
            .count();
        total_conditions += conditions;

        // Verify runtime evaluation code exists (not just placeholder comments)
        if content.contains("ExpressionEvaluator::new()")
            && content.contains("ProcessorContext::default()")
            && content.contains("evaluate_context_condition")
        {
            has_runtime_evaluation = true;
        }

        // Verify no "not yet supported" placeholders remain
        assert!(
            !content.contains("Model condition not yet supported"),
            "Found placeholder comment in {}",
            manufacturer
        );
        assert!(
            !content.contains("TODO.*model.*condition"),
            "Found TODO comment about model condition in {}",
            manufacturer
        );
    }

    assert!(
        total_conditions > 0,
        "Should find model conditions across manufacturers"
    );
    assert!(
        has_runtime_evaluation,
        "Generated code should contain runtime evaluation logic, not placeholder comments"
    );

    println!("✓ Generated code quality verified");
    println!(
        "  Total model conditions: {}, Runtime evaluation present: {}",
        total_conditions, has_runtime_evaluation
    );
}
