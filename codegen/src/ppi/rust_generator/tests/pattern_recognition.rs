//! Tests for pattern recognition in PPI Rust code generation
//!
//! These tests cover:
//! - Pack C* bit extraction patterns
//! - Join+unpack binary data patterns
//! - Safe division/reciprocal patterns
//! - Sprintf with string operations
//! - Static function generation compliance

use crate::ppi::rust_generator::expressions::{ComplexPatternHandler, ExpressionCombiner};
use crate::ppi::RustGenerator;
use crate::ppi::{CodeGenError, ExpressionType, PpiNode};
use serde_json::json;

#[test]
fn test_pack_c_star_bit_extraction_pattern() {
    // Test the specific pattern from ExifTool: pack "C*", map { (($val>>$_)&0x1f)+0x60 } 10, 5, 0
    let generator = RustGenerator::new(
        ExpressionType::ValueConv,
        "test_pack_bit_extract".to_string(),
        "pack \"C*\", map { (($val>>$_)&0x1f)+0x60 } 10, 5, 0".to_string(),
    );

    // Simulate the parts that would be extracted from this expression by combine_statement_parts
    let parts = vec![
        "pack".to_string(),
        "\"C*\"".to_string(),
        ",".to_string(),
        "map".to_string(),
        "{".to_string(),
        "(".to_string(),
        "(".to_string(),
        "$val".to_string(),
        ">>".to_string(),
        "$_".to_string(),
        ")".to_string(),
        "&".to_string(),
        "0x1f".to_string(), // This should be detected as mask
        ")".to_string(),
        "+".to_string(),
        "0x60".to_string(), // This should be detected as offset
        "}".to_string(),
        ",".to_string(),
        "10".to_string(), // These should be detected as shifts
        ",".to_string(),
        "5".to_string(),
        ",".to_string(),
        "0".to_string(),
    ];

    // Create dummy PpiNode children for the method signature
    let children: Vec<PpiNode> = vec![];

    match generator.combine_statement_parts(&parts, &children) {
        Ok(result) => {
            println!("Generated pack pattern code: {}", result);

            // Verify the result contains our helper function call
            assert!(result.contains("crate::fmt::pack_c_star_bit_extract"));
            assert!(result.contains("val"));
            assert!(result.contains("[10, 5, 0]")); // The shift values
            assert!(result.contains("31")); // 0x1f = 31 (mask)
            assert!(result.contains("96")); // 0x60 = 96 (offset)
        }
        Err(e) => panic!("Pack pattern recognition failed: {:?}", e),
    }
}

#[test]
fn test_pack_c_star_fallback_pattern() {
    // Test fallback when mask/offset aren't clearly detected
    let generator = RustGenerator::new(
        ExpressionType::ValueConv,
        "test_pack_fallback".to_string(),
        "pack \"C*\", map { ... } 8, 4, 0".to_string(),
    );

    let parts = vec![
        "pack".to_string(),
        "\"C*\"".to_string(),
        ",".to_string(),
        "map".to_string(),
        "{".to_string(),
        "complex_expression".to_string(),
        "}".to_string(),
        ",".to_string(),
        "8".to_string(),
        ",".to_string(),
        "4".to_string(),
        ",".to_string(),
        "0".to_string(),
    ];

    let children: Vec<PpiNode> = vec![];

    match generator.combine_statement_parts(&parts, &children) {
        Ok(result) => {
            println!("Fallback pack pattern code: {}", result);

            // When pattern extraction fails, should still use the hardcoded fallback values
            assert!(result.contains("crate::fmt::pack_c_star_bit_extract"));
            assert!(result.contains("[8, 4, 0]")); // The shift values
            assert!(result.contains("31")); // Default mask 0x1f
            assert!(result.contains("96")); // Default offset 0x60
        }
        Err(e) => panic!("Fallback pack pattern recognition failed: {:?}", e),
    }
}

#[test]
fn test_pack_c_star_different_mask_offset() {
    // Test pattern with different mask and offset values
    let generator = RustGenerator::new(
        ExpressionType::ValueConv,
        "test_pack_different_values".to_string(),
        "pack \"C*\", map { (($val>>$_)&0x0f)+0x30 } 12, 8, 4, 0".to_string(),
    );

    let parts = vec![
        "pack".to_string(),
        "\"C*\"".to_string(),
        ",".to_string(),
        "map".to_string(),
        "{".to_string(),
        "(".to_string(),
        "(".to_string(),
        "$val".to_string(),
        ">>".to_string(),
        "$_".to_string(),
        ")".to_string(),
        "&".to_string(),
        "0x0f".to_string(), // Different mask (15)
        ")".to_string(),
        "+".to_string(),
        "0x30".to_string(), // Different offset (48)
        "}".to_string(),
        ",".to_string(),
        "12".to_string(), // Different shift values
        ",".to_string(),
        "8".to_string(),
        ",".to_string(),
        "4".to_string(),
        ",".to_string(),
        "0".to_string(),
    ];

    let children: Vec<PpiNode> = vec![];

    match generator.combine_statement_parts(&parts, &children) {
        Ok(result) => {
            println!("Different values pack pattern code: {}", result);

            assert!(result.contains("crate::fmt::pack_c_star_bit_extract"));
            assert!(result.contains("[12, 8, 4, 0]")); // The shift values
            assert!(result.contains("15")); // 0x0f = 15 (mask)
            assert!(result.contains("48")); // 0x30 = 48 (offset)
        }
        Err(e) => panic!("Different values pack pattern recognition failed: {:?}", e),
    }
}

#[test]
fn test_extract_pack_map_pattern_method() {
    // Test the helper method directly
    let generator = RustGenerator::new(
        ExpressionType::ValueConv,
        "test_extraction".to_string(),
        "test".to_string(),
    );

    // Test successful extraction
    let parts_with_pattern = vec![
        "pack".to_string(),
        "\"C*\"".to_string(),
        "map".to_string(),
        "0x1f".to_string(), // mask
        "0x60".to_string(), // offset
        "10".to_string(),   // shifts
        "5".to_string(),
        "0".to_string(),
    ];

    let children: Vec<PpiNode> = vec![];

    match generator.extract_pack_map_pattern(&parts_with_pattern, &children) {
        Ok(Some((mask, offset, shifts))) => {
            assert_eq!(mask, 31); // 0x1f
            assert_eq!(offset, 96); // 0x60
            assert_eq!(shifts, vec![10, 5, 0]);
        }
        Ok(None) => panic!("Should have extracted pattern"),
        Err(e) => panic!("Extraction failed: {:?}", e),
    }

    // Test no pattern found
    let parts_no_pattern = vec!["join".to_string(), "\" \"".to_string(), "split".to_string()];

    match generator.extract_pack_map_pattern(&parts_no_pattern, &children) {
        Ok(None) => {
            // Expected: no pattern found
        }
        Ok(Some(_)) => panic!("Should not have found pattern in non-pack expression"),
        Err(e) => panic!("Extraction should not error on non-pattern: {:?}", e),
    }
}

#[test]
fn test_join_unpack_pattern() {
    // Test the pattern: join " ", unpack "H2H2", val
    let generator = RustGenerator::new(
        ExpressionType::PrintConv,
        "test_join_unpack".to_string(),
        "join \" \", unpack \"H2H2\", val".to_string(),
    );

    // Simulate the parts that would be extracted from this expression
    let parts = vec![
        "join".to_string(),
        "\" \"".to_string(), // separator
        ",".to_string(),
        "unpack".to_string(),
        "\"H2H2\"".to_string(), // format
        ",".to_string(),
        "val".to_string(), // data
    ];

    let children: Vec<PpiNode> = vec![];

    match generator.combine_statement_parts(&parts, &children) {
        Ok(result) => {
            println!("Generated join+unpack code: {}", result);

            // Verify it uses the join_unpack_binary helper
            assert!(result.contains("crate::fmt::join_unpack_binary"));
            assert!(result.contains("\" \"")); // separator
            assert!(result.contains("\"H2H2\"")); // format
            assert!(result.contains("val")); // data variable
        }
        Err(e) => panic!("Join+unpack pattern recognition failed: {:?}", e),
    }
}

#[test]
fn test_join_unpack_different_separator() {
    // Test with different separator: join "-", unpack "C*", val
    let generator = RustGenerator::new(
        ExpressionType::PrintConv,
        "test_join_unpack_dash".to_string(),
        "join \"-\", unpack \"C*\", val".to_string(),
    );

    let parts = vec![
        "join".to_string(),
        "\"-\"".to_string(), // dash separator
        ",".to_string(),
        "unpack".to_string(),
        "\"C*\"".to_string(), // different format
        ",".to_string(),
        "val".to_string(),
    ];

    let children: Vec<PpiNode> = vec![];

    match generator.combine_statement_parts(&parts, &children) {
        Ok(result) => {
            println!("Generated join+unpack with dash separator: {}", result);

            assert!(result.contains("crate::fmt::join_unpack_binary"));
            assert!(result.contains("\"-\"")); // dash separator
            assert!(result.contains("\"C*\"")); // C* format
            assert!(result.contains("val"));
        }
        Err(e) => panic!("Join+unpack with dash separator failed: {:?}", e),
    }
}

#[test]
fn test_join_unpack_complex_data() {
    // Test with more complex data expression: join " ", unpack "H2H2", $val[0]
    let generator = RustGenerator::new(
        ExpressionType::PrintConv,
        "test_join_unpack_complex".to_string(),
        "join \" \", unpack \"H2H2\", $val[0]".to_string(),
    );

    let parts = vec![
        "join".to_string(),
        "\" \"".to_string(),
        ",".to_string(),
        "unpack".to_string(),
        "\"H2H2\"".to_string(),
        ",".to_string(),
        "$val[0]".to_string(), // more complex data reference
    ];

    let children: Vec<PpiNode> = vec![];

    match generator.combine_statement_parts(&parts, &children) {
        Ok(result) => {
            println!("Generated join+unpack with complex data: {}", result);

            assert!(result.contains("crate::fmt::join_unpack_binary"));
            assert!(result.contains("\" \""));
            assert!(result.contains("\"H2H2\""));
            assert!(result.contains("$val[0]"));
        }
        Err(e) => panic!("Join+unpack with complex data failed: {:?}", e),
    }
}

#[test]
fn test_standalone_unpack_not_affected() {
    // Ensure standalone unpack calls still work and aren't affected by join+unpack pattern
    let generator = RustGenerator::new(
        ExpressionType::PrintConv,
        "test_standalone_unpack".to_string(),
        "unpack \"H2H2\", val".to_string(),
    );

    let parts = vec![
        "unpack".to_string(),
        "\"H2H2\"".to_string(),
        ",".to_string(),
        "val".to_string(),
    ];

    let children: Vec<PpiNode> = vec![];

    match generator.combine_statement_parts(&parts, &children) {
        Ok(result) => {
            println!("Generated standalone unpack code: {}", result);

            // Should use standalone unpack helper, not join+unpack
            assert!(result.contains("crate::fmt::unpack_binary"));
            assert!(!result.contains("join_unpack_binary"));
            assert!(result.contains("\"H2H2\""));
            assert!(result.contains("val"));
        }
        Err(e) => panic!("Standalone unpack pattern failed: {:?}", e),
    }
}

#[test]
fn test_pack_map_pattern_extraction() {
    // Test the restored pack "C*", map { bit extraction } pattern
    // From ExifTool Canon.pm line 1847: pack "C*", map { (($_>>$_)&0x1f)+0x60 } 10, 5, 0
    let generator = RustGenerator::new(
        ExpressionType::ValueConv,
        "test_pack_map".to_string(),
        "pack \"C*\", map { ... } 10, 5, 0".to_string(),
    );

    let parts = vec![
        "pack".to_string(),
        "\"C*\"".to_string(),
        ",".to_string(),
        "map".to_string(),
        "{".to_string(),
        "...".to_string(),
        "}".to_string(),
        "0x1f".to_string(),
        "0x60".to_string(),
        "10".to_string(),
        "5".to_string(),
        "0".to_string(),
    ];

    let children = vec![]; // Empty for this test

    match generator.combine_statement_parts(&parts, &children) {
        Ok(result) => {
            // Should generate pack_c_star_bit_extract call
            assert!(result.contains("crate::fmt::pack_c_star_bit_extract"));
            assert!(result.contains("[10, 5, 0]")); // Shift values as array
            assert!(result.contains("31")); // Mask value (0x1f = 31)
            assert!(result.contains("96")); // Offset value (0x60 = 96)
        }
        Err(e) => panic!("Pack map pattern failed: {:?}", e),
    }
}

#[test]
fn test_safe_division_pattern() {
    // Test the restored safe division pattern recognition
    // From ExifTool Canon.pm: $val ? 1/$val : 0
    let generator = RustGenerator::new(
        ExpressionType::ValueConv,
        "test_safe_reciprocal".to_string(),
        "$val ? 1/$val : 0".to_string(),
    );

    let parts = vec![
        "$val".to_string(),
        "?".to_string(),
        "1".to_string(),
        "/".to_string(),
        "$val".to_string(),
        ":".to_string(),
        "0".to_string(),
    ];

    let children = vec![]; // Empty for this test

    match generator.combine_statement_parts(&parts, &children) {
        Ok(result) => {
            // Should generate safe_reciprocal call
            assert!(result.contains("crate::fmt::safe_reciprocal"));
            assert!(result.contains("$val"));
        }
        Err(e) => panic!("Safe reciprocal pattern failed: {:?}", e),
    }
}

#[test]
fn test_safe_division_with_numerator() {
    // Test safe division with custom numerator: $val ? 10/$val : 0
    let generator = RustGenerator::new(
        ExpressionType::ValueConv,
        "test_safe_division".to_string(),
        "$val ? 10/$val : 0".to_string(),
    );

    let parts = vec![
        "$val".to_string(),
        "?".to_string(),
        "10".to_string(),
        "/".to_string(),
        "$val".to_string(),
        ":".to_string(),
        "0".to_string(),
    ];

    let children = vec![]; // Empty for this test

    match generator.combine_statement_parts(&parts, &children) {
        Ok(result) => {
            // Should generate safe_division call with numerator
            assert!(result.contains("crate::fmt::safe_division"));
            assert!(result.contains("10.0"));
            assert!(result.contains("$val"));
        }
        Err(e) => panic!("Safe division pattern failed: {:?}", e),
    }
}

#[test]
fn test_sprintf_with_string_operations() {
    // Test the restored sprintf with string concatenation pattern
    // From ExifTool Canon.pm: sprintf("%19d %4d %6d" . " %3d %4d %6d" x 8, args)
    let generator = RustGenerator::new(
        ExpressionType::PrintConv,
        "test_sprintf_concat".to_string(),
        "sprintf(\"%19d %4d %6d\" . \" %3d %4d %6d\" x 8, split(\" \",$val))".to_string(),
    );

    let parts = vec![
        "sprintf".to_string(),
        "(".to_string(),
        "\"%19d %4d %6d\"".to_string(),
        ".".to_string(),
        "\" %3d %4d %6d\"".to_string(),
        "x".to_string(),
        "8".to_string(),
        ",".to_string(),
        "split".to_string(),
        "(".to_string(),
        "\" \"".to_string(),
        ",".to_string(),
        "$val".to_string(),
        ")".to_string(),
        ")".to_string(),
    ];

    let children = vec![]; // Empty for this test

    match generator.combine_statement_parts(&parts, &children) {
        Ok(result) => {
            // Should generate sprintf_with_string function call
            assert!(
                result.contains("sprintf_with_string") || result.contains("crate::fmt::sprintf")
            );
        }
        Err(e) => panic!("Sprintf string operations pattern failed: {:?}", e),
    }
}
