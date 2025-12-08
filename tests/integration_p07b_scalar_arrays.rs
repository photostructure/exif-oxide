//! Integration test for P07b: Complete Scalar Array Extraction for Nikon XLAT
//!
//! This test validates that:
//! 1. XLAT arrays are extracted during codegen
//! 2. XLAT files are generated (xlat_0.rs, xlat_1.rs)
//! 3. XLAT constants are properly declared as modules in nikon/mod.rs
//! 4. XLAT constants can be imported and used

use std::fs;
use std::path::Path;

#[test]
fn test_nikon_xlat_arrays_generated() {
    // This test requires the codegen to have been run
    // In CI, this would be guaranteed by build dependencies

    // Directory naming convention: Nikon_pm (matching ExifTool module name)
    let nikon_dir = Path::new("src/generated/Nikon_pm");
    let xlat_0_path = nikon_dir.join("xlat_0.rs");
    let xlat_1_path = nikon_dir.join("xlat_1.rs");
    let mod_path = nikon_dir.join("mod.rs");

    // Check that xlat files exist
    assert!(
        xlat_0_path.exists(),
        "xlat_0.rs should be generated in src/generated/Nikon_pm/. \
         Run 'make clean-all codegen' to regenerate."
    );

    assert!(
        xlat_1_path.exists(),
        "xlat_1.rs should be generated in src/generated/Nikon_pm/. \
         Run 'make clean-all codegen' to regenerate."
    );

    // Check that mod.rs exists and declares the xlat modules
    assert!(
        mod_path.exists(),
        "mod.rs should exist in src/generated/Nikon_pm/"
    );

    let mod_content = fs::read_to_string(&mod_path).expect("Should be able to read Nikon_pm/mod.rs");

    assert!(
        mod_content.contains("pub mod xlat_0;"),
        "Nikon_pm/mod.rs should declare 'pub mod xlat_0;'. Found content:\n{}",
        mod_content
    );

    assert!(
        mod_content.contains("pub mod xlat_1;"),
        "Nikon_pm/mod.rs should declare 'pub mod xlat_1;'. Found content:\n{}",
        mod_content
    );

    // Check the content of xlat_0.rs
    let xlat_0_content =
        fs::read_to_string(&xlat_0_path).expect("Should be able to read xlat_0.rs");

    assert!(
        xlat_0_content.contains("pub static XLAT_0:"),
        "xlat_0.rs should contain 'pub static XLAT_0:'. Found content:\n{}",
        xlat_0_content
    );

    assert!(
        xlat_0_content.contains("[u8; 256]"),
        "xlat_0.rs should declare XLAT_0 as '[u8; 256]'. Found content:\n{}",
        xlat_0_content
    );

    // Check the content of xlat_1.rs
    let xlat_1_content =
        fs::read_to_string(&xlat_1_path).expect("Should be able to read xlat_1.rs");

    assert!(
        xlat_1_content.contains("pub static XLAT_1:"),
        "xlat_1.rs should contain 'pub static XLAT_1:'. Found content:\n{}",
        xlat_1_content
    );

    assert!(
        xlat_1_content.contains("[u8; 256]"),
        "xlat_1.rs should declare XLAT_1 as '[u8; 256]'. Found content:\n{}",
        xlat_1_content
    );

    // Verify the arrays contain actual data (not just zeros)
    assert!(
        xlat_0_content.contains("0xc1") || xlat_0_content.contains("193"),
        "xlat_0.rs should contain actual hex/decimal values from ExifTool. Found content:\n{}",
        xlat_0_content
    );

    assert!(
        xlat_1_content.contains("0xa7") || xlat_1_content.contains("167"),
        "xlat_1.rs should contain actual hex/decimal values from ExifTool. Found content:\n{}",
        xlat_1_content
    );
}

#[test]
#[cfg(feature = "integration-tests")]
fn test_xlat_arrays_can_be_imported() {
    // This test actually imports the generated constants to verify they work
    // It requires the test-helpers feature to access the generated modules

    use exif_oxide::generated::Nikon_pm::xlat_0::XLAT_0;
    use exif_oxide::generated::Nikon_pm::xlat_1::XLAT_1;

    // Basic sanity checks on the arrays
    assert_eq!(XLAT_0.len(), 256, "XLAT_0 should have 256 elements");
    assert_eq!(XLAT_1.len(), 256, "XLAT_1 should have 256 elements");

    // Check some known values from the ExifTool source
    // From Nikon.pm line 13506-13507: the first array starts with [0xc1,0xbf,0x6d,...]
    assert_eq!(
        XLAT_0[0], 0xc1,
        "First element of XLAT_0 should be 0xc1 (193)"
    );
    assert_eq!(
        XLAT_0[1], 0xbf,
        "Second element of XLAT_0 should be 0xbf (191)"
    );
    assert_eq!(
        XLAT_0[2], 0x6d,
        "Third element of XLAT_0 should be 0x6d (109)"
    );

    // The second array starts with [0xa7,0xbc,0xc9,...] (from ExifTool source)
    // We'll verify at least the first element once we see the actual extraction
    assert_ne!(XLAT_1[0], 0, "First element of XLAT_1 should not be zero");
}

#[test]
fn test_scalar_array_strategy_handles_xlat() {
    use codegen::field_extractor::{FieldMetadata, FieldSymbol};
    use codegen::strategies::{ExtractionContext, ExtractionStrategy, ScalarArrayStrategy};
    use serde_json::json;

    let mut strategy = ScalarArrayStrategy::new();

    // Create a mock xlat symbol like what would be extracted from Nikon.pm
    let xlat_symbol = FieldSymbol {
        symbol_type: "array".to_string(),
        name: "xlat".to_string(),
        data: json!([
            [193, 191, 109, 13], // First few elements of xlat[0]
            [167, 188, 201, 25]  // First few elements of xlat[1]
        ]),
        module: "Nikon".to_string(),
        metadata: FieldMetadata {
            size: 2,
            is_composite_table: 0,
        },
    };

    // Test that the strategy can handle this symbol
    assert!(
        strategy.can_handle(&xlat_symbol),
        "ScalarArrayStrategy should be able to handle xlat arrays"
    );

    // Test extraction
    let mut context = ExtractionContext::new("test_output".to_string());
    strategy
        .extract(&xlat_symbol, &mut context)
        .expect("Should be able to extract xlat arrays");

    // Test generation
    let files = strategy
        .finish_extraction(&mut context)
        .expect("Should be able to generate xlat files");

    assert_eq!(
        files.len(),
        2,
        "Should generate 2 files for nested xlat array"
    );

    // Verify file names
    let filenames: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
    assert!(
        filenames.contains(&"nikon/xlat_0.rs"),
        "Should generate nikon/xlat_0.rs. Generated: {:?}",
        filenames
    );
    assert!(
        filenames.contains(&"nikon/xlat_1.rs"),
        "Should generate nikon/xlat_1.rs. Generated: {:?}",
        filenames
    );

    // Verify content contains expected constants
    let xlat_0_file = files.iter().find(|f| f.path == "nikon/xlat_0.rs").unwrap();
    assert!(
        xlat_0_file.content.contains("pub static XLAT_0: [u8; 4]"),
        "xlat_0.rs should contain proper constant declaration"
    );

    let xlat_1_file = files.iter().find(|f| f.path == "nikon/xlat_1.rs").unwrap();
    assert!(
        xlat_1_file.content.contains("pub static XLAT_1: [u8; 4]"),
        "xlat_1.rs should contain proper constant declaration"
    );
}
