//! Tests for the conversion registry modules

use super::normalization::batch_normalize_expressions;
use super::printconv_registry::{get_printconv_registry, get_tag_specific_printconv};
use super::valueconv_registry::get_valueconv_registry;
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
fn test_normalize_sprintf_expressions() {
    // Basic sprintf patterns from ExifTool
    assert_eq!(
        normalize_expression("sprintf( \"%.1f mm\" , $val )"),
        "sprintf(\"%.1f mm\", $val)"
    );

    assert_eq!(
        normalize_expression("sprintf(\"%.1f\",$val)"),
        "sprintf(\"%.1f\", $val)"
    );

    assert_eq!(
        normalize_expression("sprintf( \"%.2f\" , $val )"),
        "sprintf(\"%.2f\", $val)"
    );

    assert_eq!(
        normalize_expression("sprintf(\"%+d\",$val)"),
        "sprintf(\"%+d\", $val)"
    );

    assert_eq!(
        normalize_expression("sprintf(\"0x%x\", $val)"),
        "sprintf(\"0x%x\", $val)"
    );

    assert_eq!(
        normalize_expression("sprintf(\"%+.1f\",$val)"),
        "sprintf(\"%+.1f\", $val)"
    );
}

#[test]
fn test_normalize_regex_expressions() {
    // Regex substitution patterns - Perl::Tidy splits multi-statement lines
    assert_eq!(
        normalize_expression("$val =~ tr/ /./; $val"),
        "$val =~ tr/ /./;\n$val"
    );

    assert_eq!(
        normalize_expression("$val=~tr/ /:/; $val"),
        "$val =~ tr/ /:/;\n$val"
    );

    assert_eq!(
        normalize_expression("$val =~ /^(inf|undef)$/ ? $val : \"$val m\""),
        "$val =~ /^(inf|undef)$/ ? $val : \"$val m\""
    );

    assert_eq!(
        normalize_expression("$val=~s/\\s+$//;$val"),
        "$val =~ s/\\s+$//;\n$val"
    );

    assert_eq!(
        normalize_expression("$val =~ /^4194303.999/ ? \"n/a\" : $val"),
        "$val =~ /^4194303.999/ ? \"n/a\" : $val"
    );
}

#[test]
fn test_normalize_ternary_expressions() {
    // Ternary conditional patterns
    assert_eq!(
        normalize_expression("$val ? $val : \"Auto\""),
        "$val ? $val : \"Auto\""
    );

    assert_eq!(
        normalize_expression("$val ? \"$val m\" : \"inf\""),
        "$val ? \"$val m\" : \"inf\""
    );

    assert_eq!(
        normalize_expression("$val > 0 ? \"+$val\" : $val"),
        "$val > 0 ? \"+$val\" : $val"
    );

    assert_eq!(
        normalize_expression("IsInt($val) ? \"$val C\" : $val"),
        "IsInt($val) ? \"$val C\" : $val"
    );
}

#[test]
fn test_normalize_function_calls() {
    // Module function calls
    assert_eq!(
        normalize_expression("Image::ExifTool::Exif::PrintExposureTime($val)"),
        "Image::ExifTool::Exif::PrintExposureTime($val)"
    );

    assert_eq!(
        normalize_expression("Image::ExifTool::Exif::PrintFNumber($val)"),
        "Image::ExifTool::Exif::PrintFNumber($val)"
    );

    assert_eq!(
        normalize_expression("Image::ExifTool::GPS::ToDMS($self, $val, 1, \"N\")"),
        "Image::ExifTool::GPS::ToDMS($self, $val, 1, \"N\")"
    );

    assert_eq!(
        normalize_expression("ConvertDuration($val)"),
        "ConvertDuration($val)"
    );
}

#[test]
fn test_normalize_complex_expressions() {
    // Complex multi-statement expressions - Perl::Tidy splits statements
    assert_eq!(
        normalize_expression("$val=~s/^(.*?) (\\d+) (\\d+)$/$1 Rating=$2 Count=$3/s; $val"),
        "$val =~ s/^(.*?) (\\d+) (\\d+)$/$1 Rating=$2 Count=$3/s;\n$val"
    );

    assert_eq!(
        normalize_expression("$val=sprintf(\"%x\",$val);$val=~s/(.{3})$/\\.$1/;$val"),
        "$val = sprintf(\"%x\", $val);\n$val =~ s/(.{3})$/\\.$1/;\n$val"
    );

    assert_eq!(
        normalize_expression("$val=~s/(\\S+) (\\S+)/$1 m, $2 ft/; $val"),
        "$val =~ s/(\\S+) (\\S+)/$1 m, $2 ft/;\n$val"
    );
}

#[test]
fn test_normalize_string_concatenation() {
    // String concatenation patterns
    assert_eq!(normalize_expression("\"$val mm\""), "\"$val mm\"");

    assert_eq!(normalize_expression("\"$val m\""), "\"$val m\"");

    assert_eq!(normalize_expression("\"$val C\""), "\"$val C\"");
}

#[test]
fn test_normalize_whitespace_handling() {
    // Excessive whitespace
    assert_eq!(
        normalize_expression("  sprintf  (  \"%.1f\"  ,  $val  )  "),
        "sprintf(\"%.1f\", $val)"
    );

    // Tabs and newlines
    assert_eq!(
        normalize_expression("sprintf(\t\"%.1f\"\t,\t$val\t)"),
        "sprintf(\"%.1f\", $val)"
    );

    // Mixed whitespace
    assert_eq!(
        normalize_expression("$val\n?\n$val\n:\n\"Auto\""),
        "$val\n? $val\n: \"Auto\""
    );
}

#[test]
fn test_normalize_preserves_important_spaces() {
    // Spaces in strings should be preserved
    assert_eq!(normalize_expression("\"$val m\""), "\"$val m\"");

    // Spaces in regex patterns should be preserved
    assert_eq!(normalize_expression("$val =~ tr/ /./"), "$val =~ tr/ /./");

    // Spaces between operators should be preserved
    assert_eq!(normalize_expression("$val > 0"), "$val > 0");
}

#[test]
fn test_normalize_edge_cases() {
    // Empty string - Perl script will fail, so we get original back
    assert_eq!(normalize_expression(""), "");

    // Just whitespace
    assert_eq!(normalize_expression("   \t\n  "), "");

    // Single variable
    assert_eq!(normalize_expression("$val"), "$val");

    // Expression with no spaces
    assert_eq!(
        normalize_expression("sprintf(\"%.1f\",$val)"),
        "sprintf(\"%.1f\", $val)"
    );
}

#[test]
fn test_registry_keys_are_normalized() {
    let mut all_keys = Vec::new();
    let mut key_sources = Vec::new();

    // Collect all keys from PRINTCONV_REGISTRY
    for &key in get_printconv_registry().keys() {
        all_keys.push(key.to_string());
        key_sources.push(("PRINTCONV_REGISTRY", key));
    }

    // Collect all keys from VALUECONV_REGISTRY
    for &key in get_valueconv_registry().keys() {
        all_keys.push(key.to_string());
        key_sources.push(("VALUECONV_REGISTRY", key));
    }

    // Collect non-module-scoped keys from TAG_SPECIFIC_PRINTCONV
    for &key in get_tag_specific_printconv().keys() {
        // Skip module-scoped keys (contain ::)
        if !key.contains("::") {
            all_keys.push(key.to_string());
            key_sources.push(("TAG_SPECIFIC_PRINTCONV", key));
        }
    }

    // Batch normalize all keys at once
    let normalized_map = match batch_normalize_expressions(&all_keys) {
        Ok(map) => map,
        Err(e) => panic!("Failed to batch normalize expressions: {}", e),
    };

    // Check for normalization issues
    let mut normalization_issues = Vec::new();
    for (i, (source, _original_key)) in key_sources.iter().enumerate() {
        let key = &all_keys[i];
        if let Some(normalized) = normalized_map.get(key) {
            if key != normalized {
                normalization_issues.push(format!(
                    "{} key '{}' should be normalized to '{}'",
                    source, key, normalized
                ));
            }
        }
    }

    if !normalization_issues.is_empty() {
        panic!(
            "Registry contains keys that need normalization:\n{}",
            normalization_issues.join("\n")
        );
    }
}

#[test]
fn test_classify_valueconv_expression() {
    // Test simple arithmetic expressions get compiled
    match classify_valueconv_expression("$val / 8", "Canon_pm") {
        ValueConvType::CompiledExpression(_) => {}
        _ => panic!("Expected compiled expression"),
    }

    match classify_valueconv_expression("($val - 104) / 8", "Nikon_pm") {
        ValueConvType::CompiledExpression(_) => {}
        _ => panic!("Expected compiled expression"),
    }

    // Test ternary expressions are now compiled! (NEW CAPABILITY)
    match classify_valueconv_expression("$val >= 0 ? $val : undef", "Sony_pm") {
        ValueConvType::CompiledExpression(_) => {}
        _ => panic!("Expected compiled expression for ternary"),
    }

    match classify_valueconv_expression("$val > 655.345 ? \"inf\" : \"$val m\"", "Canon_pm") {
        ValueConvType::CompiledExpression(_) => {}
        _ => panic!("Expected compiled expression for ternary with strings"),
    }

    // Test truly complex expressions still get custom functions
    match classify_valueconv_expression("IsFloat($val) && abs($val) < 100", "Sony_pm") {
        ValueConvType::CustomFunction(_, _) => {}
        _ => panic!("Expected custom function for complex logic"),
    }

    match classify_valueconv_expression("$val =~ s/ +$//", "Canon_pm") {
        ValueConvType::CustomFunction(_, _) => {}
        _ => panic!("Expected custom function for regex"),
    }
}
