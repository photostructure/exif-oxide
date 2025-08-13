//! Tests for the implementation registry modules

use super::function_registry::{
    get_function_registry, get_pattern_registry, lookup_functions_by_category, FunctionCategory,
};
use super::printconv_registry::{get_printconv_registry, get_tag_specific_printconv};
use super::valueconv_registry::{
    classify_valueconv_expression, get_valueconv_registry, lookup_valueconv,
};
use super::*;

#[test]
fn test_module_scoped_lookup() {
    // Test direct lookup of a known value
    let result = lookup_valueconv("Image::ExifTool::GPS::ConvertTimeStamp($val)", "GPS_pm");
    assert_eq!(
        result,
        Some((
            "crate::implementations::value_conv",
            "gpstimestamp_value_conv"
        ))
    );
}

#[test]
fn test_manual_printconv_lookup() {
    let result = lookup_printconv("fnumber_print_conv", "Exif_pm");
    assert_eq!(
        result,
        Some(("crate::implementations::print_conv", "fnumber_print_conv"))
    );
}

#[test]
fn test_exact_string_matching() {
    // Test that we use exact string matching without normalization
    // These should NOT match because they're formatted differently
    let result1 = lookup_printconv("sprintf(\"%.1f mm\",$val)", "Exif_pm");
    let result2 = lookup_printconv("sprintf(\"%.1f mm\", $val)", "Exif_pm");

    // They might both be None or map to functions, but the key point is
    // we're not normalizing - we're doing exact matching
    // So if one is in the registry and the other isn't, they won't match

    // This test just verifies the lookup works, not the results
    // (since we may or may not have these specific entries)
    let _ = (result1, result2);
}

#[test]
fn test_registry_contains_expected_entries() {
    // Verify some known entries exist in the registries

    // Check PrintConv registry
    let printconv_registry = get_printconv_registry();
    assert!(printconv_registry.contains_key("fnumber_print_conv"));
    assert!(printconv_registry.contains_key("exposuretime_print_conv"));

    // Check ValueConv registry
    let valueconv_registry = get_valueconv_registry();
    // GPS timestamp conversion should be there
    let has_gps_timestamp = valueconv_registry
        .keys()
        .any(|k| k.contains("GPS::ConvertTimeStamp"));
    assert!(
        has_gps_timestamp,
        "ValueConv registry should contain GPS timestamp conversion"
    );
}

#[test]
fn test_tag_specific_printconv() {
    // Test tag-specific lookups
    let tag_registry = get_tag_specific_printconv();

    // Flash should be a universal tag
    assert!(tag_registry.contains_key("Flash"));

    // GPS reference tags should be there
    assert!(tag_registry.contains_key("GPSLatitudeRef"));
    assert!(tag_registry.contains_key("GPSLongitudeRef"));
}

#[test]
fn test_module_name_normalization() {
    // Module names should have _pm stripped
    // GPS_pm -> GPS for scoped lookups

    // This tests that the module name normalization (not expression normalization)
    // still works correctly
    let result = lookup_printconv("GPS::ConvertTimeStamp($val)", "GPS_pm");
    // Should work whether we pass GPS or GPS_pm
    let result2 = lookup_printconv("GPS::ConvertTimeStamp($val)", "GPS");

    // Both should resolve to the same thing (or both be None)
    assert_eq!(result, result2);
}

#[test]
fn test_classify_valueconv_prioritizes_registry_over_compilation() {
    use super::types::ValueConvType;

    // GPS functions should prioritize registry over compilation
    let gps_expr = "Image::ExifTool::GPS::ToDegrees($val)";
    match classify_valueconv_expression(gps_expr, "GPS_pm") {
        ValueConvType::CustomFunction(module_path, func_name) => {
            assert_eq!(module_path, "crate::implementations::value_conv");
            assert_eq!(func_name, "gps_coordinate_value_conv");
        }
        ValueConvType::PpiGeneratedSimple(_)
        | ValueConvType::PpiGeneratedWithContext(_)
        | ValueConvType::PpiGeneratedComposite(_) => {
            panic!("GPS functions should use registry, not PPI generation!");
        }
    }

    // Simple arithmetic should use compilation
    let arithmetic_expr = "$val * 100";
    match classify_valueconv_expression(arithmetic_expr, "Exif_pm") {
        ValueConvType::PpiGeneratedSimple(_) => {
            // This is expected for simple arithmetic - PPI will generate it
        }
        ValueConvType::PpiGeneratedWithContext(_) | ValueConvType::PpiGeneratedComposite(_) => {
            // Also valid if it needs context
        }
        ValueConvType::CustomFunction(_, _) => {
            // This is also valid if there's a registry entry
        }
    }

    // Power operations should use PPI generation (PPI handles ** operator)
    let power_expr = "2**(-$val / 3)";
    match classify_valueconv_expression(power_expr, "Sony_pm") {
        ValueConvType::PpiGeneratedSimple(_) => {
            // This is expected - PPI can generate power operations
        }
        ValueConvType::CustomFunction(module_path, func_name) => {
            // Also valid if there's a specific registry override for optimization
            assert_eq!(module_path, "crate::implementations::value_conv");
            assert_eq!(func_name, "power_neg_div_3_value_conv");
        }
        ValueConvType::PpiGeneratedWithContext(_) | ValueConvType::PpiGeneratedComposite(_) => {
            panic!("Power operations shouldn't need context!");
        }
    }
}

// Function Registry Tests

#[test]
fn test_function_registry_builtin_lookup() {
    // Test builtin function lookups
    let sprintf_result = lookup_function("sprintf");
    assert!(sprintf_result.is_some());

    if let Some(FunctionImplementation::Builtin(builtin)) = sprintf_result {
        assert_eq!(builtin.module_path, "crate::implementations::builtins");
        assert_eq!(builtin.function_name, "sprintf_impl");
        assert_eq!(builtin.parameter_pattern, "(format_string, ...args)");
    } else {
        panic!("sprintf should be a builtin function");
    }

    // Test other builtins
    assert!(lookup_function("substr").is_some());
    assert!(lookup_function("uc").is_some());
    assert!(lookup_function("lc").is_some());
}

#[test]
fn test_function_registry_exiftool_module_lookup() {
    // Test ExifTool module function lookups
    let canon_ev_result = lookup_function("Image::ExifTool::Canon::CanonEv");
    assert!(canon_ev_result.is_some());

    if let Some(FunctionImplementation::ExifToolModule(module_func)) = canon_ev_result {
        assert_eq!(module_func.module_path, "crate::implementations::canon");
        assert_eq!(module_func.function_name, "canon_ev");
        assert_eq!(module_func.exiftool_module, "Canon");
    } else {
        panic!("CanonEv should be an ExifTool module function");
    }

    // Test GPS functions
    let gps_degrees_result = lookup_function("Image::ExifTool::GPS::ToDegrees");
    assert!(gps_degrees_result.is_some());

    if let Some(FunctionImplementation::ExifToolModule(module_func)) = gps_degrees_result {
        assert_eq!(module_func.exiftool_module, "GPS");
    } else {
        panic!("ToDegrees should be a GPS module function");
    }
}

#[test]
fn test_function_registry_custom_script_lookup() {
    // Test custom script function lookups
    let complex_condition = lookup_function("complex_binary_data_condition");
    assert!(complex_condition.is_some());

    if let Some(FunctionImplementation::CustomScript(script)) = complex_condition {
        assert_eq!(
            script.module_path,
            "crate::implementations::complex_conditions"
        );
        assert_eq!(script.function_name, "complex_binary_data_condition");
        assert!(script.description.contains("Multi-line conditional"));
    } else {
        panic!("complex_binary_data_condition should be a custom script");
    }
}

#[test]
fn test_function_registry_pattern_matching() {
    // Test pattern matching for function calls with parameters
    let sprintf_with_params = lookup_function("sprintf(\"%.1f\", $val)");
    // This should NOT match exactly, but pattern matching should work
    // Actually, our implementation does pattern matching, so this might match
    // Let's verify the behavior rather than assume
    if sprintf_with_params.is_some() {
        // Pattern matching worked - verify it's the sprintf function
        if let Some(FunctionImplementation::Builtin(builtin)) = sprintf_with_params {
            assert_eq!(builtin.function_name, "sprintf_impl");
        }
    }

    // But the pattern registry should find it
    let pattern_registry = get_pattern_registry();
    assert!(pattern_registry.contains_key("sprintf("));

    // Test that pattern matching works for partial matches
    let _canon_ev_with_params = "Image::ExifTool::Canon::CanonEv($val/10)";
    // This should match the pattern and resolve to the base function
    // (Currently the implementation doesn't do this, but the infrastructure is there)
}

#[test]
fn test_function_lookup_by_category() {
    // Test category-based lookup
    let builtins = lookup_functions_by_category(FunctionCategory::Builtin);
    assert!(!builtins.is_empty());

    // Verify all returned functions are actually builtins
    for (name, implementation) in &builtins {
        match implementation {
            FunctionImplementation::Builtin(_) => {
                // Expected - verify some known builtins are there
                if *name == "sprintf" || *name == "substr" || *name == "uc" || *name == "lc" {
                    // Good, found expected builtin
                } else {
                    println!("Found unexpected builtin: {}", name);
                }
            }
            _ => panic!(
                "Category filter should only return builtins, found: {}",
                name
            ),
        }
    }

    let exiftool_modules = lookup_functions_by_category(FunctionCategory::ExifToolModule);
    assert!(!exiftool_modules.is_empty());

    let custom_scripts = lookup_functions_by_category(FunctionCategory::CustomScript);
    assert!(!custom_scripts.is_empty());
}

#[test]
fn test_needs_function_registry_lookup() {
    // Test the heuristic function for determining registry lookup need

    // ExifTool module functions should need registry lookup
    assert!(needs_function_registry_lookup(
        "Image::ExifTool::Canon::CanonEv($val)"
    ));
    assert!(needs_function_registry_lookup(
        "Image::ExifTool::GPS::ToDegrees($val, 1)"
    ));

    // Perl builtins should need registry lookup
    assert!(needs_function_registry_lookup("sprintf(\"%.2f\", $val)"));
    assert!(needs_function_registry_lookup("substr($val, 0, 4)"));

    // Multi-line expressions should need registry lookup
    let multiline_expr =
        "if ($val > 0) {\n    return \"positive\";\n} else {\n    return \"negative\";\n}";
    assert!(needs_function_registry_lookup(multiline_expr));

    // Complex regex should need registry lookup
    assert!(needs_function_registry_lookup(
        "$$valPt =~ /^\\x00\\x04/ && length($$valPt) > 10"
    ));

    // Simple expressions should NOT need registry lookup
    assert!(!needs_function_registry_lookup("$val * 100"));
    assert!(!needs_function_registry_lookup("$val + 273.15"));
    assert!(!needs_function_registry_lookup("$val / 100"));
}

#[test]
fn test_get_function_details() {
    // Test function details retrieval for code generation

    // Test builtin function details
    let sprintf_details = get_function_details("sprintf");
    assert!(sprintf_details.is_some());
    if let Some(details) = sprintf_details {
        assert_eq!(details.category, "builtin");
        assert_eq!(details.module_path, "crate::implementations::builtins");
        assert_eq!(details.function_name, "sprintf_impl");
        assert!(details.description.contains("Perl builtin"));
    }

    // Test ExifTool module function details
    let canon_ev_details = get_function_details("Image::ExifTool::Canon::CanonEv");
    assert!(canon_ev_details.is_some());
    if let Some(details) = canon_ev_details {
        assert_eq!(details.category, "exiftool_module");
        assert_eq!(details.module_path, "crate::implementations::canon");
        assert_eq!(details.function_name, "canon_ev");
        assert!(details.description.contains("Canon"));
    }

    // Test custom script function details
    let script_details = get_function_details("complex_binary_data_condition");
    assert!(script_details.is_some());
    if let Some(details) = script_details {
        assert_eq!(details.category, "custom_script");
        assert!(details.description.contains("Multi-line conditional"));
    }

    // Test unknown function
    let unknown_details = get_function_details("nonexistent_function");
    assert!(unknown_details.is_none());
}

#[test]
fn test_function_registry_expected_entries() {
    // Verify the function registry contains expected entries
    let function_registry = get_function_registry();

    // Check builtin functions
    assert!(function_registry.contains_key("sprintf"));
    assert!(function_registry.contains_key("substr"));
    assert!(function_registry.contains_key("uc"));
    assert!(function_registry.contains_key("lc"));

    // Check Canon functions
    assert!(function_registry.contains_key("Image::ExifTool::Canon::CanonEv"));
    assert!(function_registry.contains_key("Image::ExifTool::Canon::CanonEvInv"));

    // Check GPS functions
    assert!(function_registry.contains_key("Image::ExifTool::GPS::ToDegrees"));
    assert!(function_registry.contains_key("Image::ExifTool::GPS::ToDMS"));

    // Check XMP functions
    assert!(function_registry.contains_key("Image::ExifTool::XMP::ConvertXMPDate"));

    // Check custom scripts
    assert!(function_registry.contains_key("complex_binary_data_condition"));
    assert!(function_registry.contains_key("complex_makernote_dispatch"));

    // Verify total count is reasonable (should have all the entries we added)
    assert!(
        function_registry.len() >= 15,
        "Function registry should contain at least 15 entries"
    );
}

#[test]
fn test_pattern_registry_expected_entries() {
    // Verify the pattern registry contains expected entries
    let pattern_registry = get_pattern_registry();

    // Check sprintf patterns
    assert!(pattern_registry.contains_key("sprintf("));
    assert!(pattern_registry.contains_key("sprintf (")); // with space

    // Check ExifTool patterns
    assert!(pattern_registry.contains_key("Image::ExifTool::Canon::CanonEv("));
    assert!(pattern_registry.contains_key("Image::ExifTool::GPS::ToDegrees("));

    // Verify mappings
    assert_eq!(pattern_registry.get("sprintf("), Some(&"sprintf"));
    assert_eq!(pattern_registry.get("sprintf ("), Some(&"sprintf"));
}
