//! Codegen-time registry for PrintConv/ValueConv mappings
//! 
//! This module provides compile-time lookup of Perl expressions to Rust function paths.
//! The registry is used during code generation to emit direct function calls,
//! eliminating runtime lookup overhead.

use std::collections::HashMap;
use std::sync::LazyLock;
use std::process::Command;
use std::sync::Mutex;
use crate::expression_compiler::CompiledExpression;

/// Classification of ValueConv expressions for code generation
#[derive(Debug, Clone)]
pub enum ValueConvType {
    /// Simple arithmetic expression that can be compiled to inline code
    CompiledExpression(CompiledExpression),
    /// Complex expression requiring a custom function
    CustomFunction(&'static str, &'static str), // (module_path, function_name)
}

// Cache for normalized expressions to avoid repeated subprocess calls
static NORMALIZATION_CACHE: LazyLock<Mutex<HashMap<String, String>>> = 
    LazyLock::new(|| Mutex::new(HashMap::new()));

// Registry maps Perl expressions to (module_path, function_name)
static PRINTCONV_REGISTRY: LazyLock<HashMap<&'static str, (&'static str, &'static str)>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    
    // Common sprintf patterns
    m.insert("sprintf(\"%.1f mm\", $val)", ("crate::implementations::print_conv", "focallength_print_conv"));
    m.insert("sprintf(\"%.1f\", $val)", ("crate::implementations::print_conv", "decimal_1_print_conv"));
    m.insert("sprintf(\"%.2f\", $val)", ("crate::implementations::print_conv", "decimal_2_print_conv"));
    m.insert("sprintf(\"%+d\", $val)", ("crate::implementations::print_conv", "signed_int_print_conv"));
    m.insert("sprintf(\"%.3f mm\", $val)", ("crate::implementations::print_conv", "focal_length_3_decimals_print_conv"));
    
    // Conditional expressions
    m.insert("$val =~ /^(inf|undef)$/ ? $val : \"$val m\"", ("crate::implementations::print_conv", "gpsaltitude_print_conv"));
    
    // Module-scoped functions
    m.insert("GPS::ConvertTimeStamp($val)", ("crate::implementations::value_conv", "gpstimestamp_value_conv"));
    m.insert("ID3::ConvertTimeStamp($val)", ("crate::implementations::value_conv", "id3_timestamp_value_conv"));
    
    // Complex expressions (placeholder names from tag_kit.pl)
    // These need to be mapped to appropriate implementations based on the tag
    // For now, we'll need specific mappings
    m.insert("complex_expression_printconv", ("crate::implementations::print_conv", "complex_expression_print_conv"));
    
    // ExifTool function calls that should be mapped to our implementations
    m.insert("Image::ExifTool::Exif::PrintExposureTime($val)", ("crate::implementations::print_conv", "exposuretime_print_conv"));
    m.insert("Image::ExifTool::Exif::PrintFNumber($val)", ("crate::implementations::print_conv", "fnumber_print_conv"));
    m.insert("Image::ExifTool::Exif::PrintFraction($val)", ("crate::implementations::print_conv", "print_fraction"));
    
    // Manual function mappings (these come through as Manual type with function names)
    m.insert("fnumber_print_conv", ("crate::implementations::print_conv", "fnumber_print_conv"));
    m.insert("exposuretime_print_conv", ("crate::implementations::print_conv", "exposuretime_print_conv"));
    m.insert("focallength_print_conv", ("crate::implementations::print_conv", "focallength_print_conv"));
    
    // Canon focal length formatting - ExifTool Canon.pm PrintConv: "$val mm"
    m.insert("\"$val mm\"", ("crate::implementations::print_conv", "focal_length_mm_print_conv"));
    m.insert("lensinfo_print_conv", ("crate::implementations::print_conv", "lensinfo_print_conv"));
    m.insert("iso_print_conv", ("crate::implementations::print_conv", "iso_print_conv"));
    m.insert("orientation_print_conv", ("crate::implementations::print_conv", "orientation_print_conv"));
    m.insert("resolutionunit_print_conv", ("crate::implementations::print_conv", "resolutionunit_print_conv"));
    m.insert("ycbcrpositioning_print_conv", ("crate::implementations::print_conv", "ycbcrpositioning_print_conv"));
    m.insert("gpsaltitude_print_conv", ("crate::implementations::print_conv", "gpsaltitude_print_conv"));
    m.insert("gpsaltituderef_print_conv", ("crate::implementations::print_conv", "gpsaltituderef_print_conv"));
    m.insert("gpslatituderef_print_conv", ("crate::implementations::print_conv", "gpslatituderef_print_conv"));
    m.insert("gpslongituderef_print_conv", ("crate::implementations::print_conv", "gpslongituderef_print_conv"));
    m.insert("gpslatitude_print_conv", ("crate::implementations::print_conv", "gpslatitude_print_conv"));
    m.insert("gpslongitude_print_conv", ("crate::implementations::print_conv", "gpslongitude_print_conv"));
    m.insert("gpsdestlatitude_print_conv", ("crate::implementations::print_conv", "gpsdestlatitude_print_conv"));
    m.insert("gpsdestlongitude_print_conv", ("crate::implementations::print_conv", "gpsdestlongitude_print_conv"));
    m.insert("flash_print_conv", ("crate::implementations::print_conv", "flash_print_conv"));
    m.insert("colorspace_print_conv", ("crate::implementations::print_conv", "colorspace_print_conv"));
    m.insert("whitebalance_print_conv", ("crate::implementations::print_conv", "whitebalance_print_conv"));
    m.insert("meteringmode_print_conv", ("crate::implementations::print_conv", "meteringmode_print_conv"));
    m.insert("exposureprogram_print_conv", ("crate::implementations::print_conv", "exposureprogram_print_conv"));
    m.insert("composite_gps_gpsaltitude_print_conv", ("crate::implementations::print_conv", "composite_gps_gpsaltitude_print_conv"));
    
    m
});

// Tag-specific registry for ComplexHash and other special cases
// Key format: "ModuleName::TagName" (e.g., "Exif_pm::Flash") for module-specific
// or just "TagName" (e.g., "Flash") for universal tags
static TAG_SPECIFIC_PRINTCONV: LazyLock<HashMap<&'static str, (&'static str, &'static str)>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    
    // Module-specific tags (highest priority)
    m.insert("Canon_pm::SelfTimer", ("crate::implementations::print_conv", "canon_selftimer_print_conv"));
    // m.insert("Canon_pm::WhiteBalance", ("crate::implementations::print_conv", "canon_white_balance_print_conv"));
    
    // Universal tags (work across all modules - fallback)
    m.insert("Flash", ("crate::implementations::print_conv", "flash_print_conv"));
    m.insert("LensInfo", ("crate::implementations::print_conv", "lensinfo_print_conv"));
    
    // GPS reference tags (ComplexHash types)
    m.insert("GPSAltitudeRef", ("crate::implementations::print_conv", "gpsaltituderef_print_conv"));
    m.insert("GPSLatitudeRef", ("crate::implementations::print_conv", "gpslatituderef_print_conv"));
    m.insert("GPSLongitudeRef", ("crate::implementations::print_conv", "gpslongituderef_print_conv"));
    
    // EXIF component configuration tags
    m.insert("ComponentsConfiguration", ("crate::implementations::print_conv", "componentsconfiguration_print_conv"));
    m.insert("FileSource", ("crate::implementations::print_conv", "filesource_print_conv"));
    m.insert("InteropVersion", ("crate::implementations::print_conv", "interopversion_print_conv"));
    
    // Add other tag-specific mappings here as needed
    
    m
});

static VALUECONV_REGISTRY: LazyLock<HashMap<&'static str, (&'static str, &'static str)>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    
    // GPS conversions
    m.insert("Image::ExifTool::GPS::ToDegrees($val)", ("crate::implementations::value_conv", "gps_coordinate_value_conv"));
    m.insert("Image::ExifTool::GPS::ConvertTimeStamp($val)", ("crate::implementations::value_conv", "gpstimestamp_value_conv"));
    
    // APEX conversions
    m.insert("IsFloat($val) && abs($val) < 100 ? 2**(-$val) : 0", ("crate::implementations::value_conv", "apex_shutter_speed_value_conv"));
    m.insert("2**($val / 2)", ("crate::implementations::value_conv", "apex_aperture_value_conv"));
    
    // Canon ValueConv expressions (normalized)
    m.insert("exp($val / 32 * log(2)) * 100", ("crate::implementations::value_conv", "canon_auto_iso_value_conv"));
    m.insert("exp($val / 32 * log(2)) * 100 / 32", ("crate::implementations::value_conv", "canon_base_iso_value_conv"));
    m.insert("($val >> 16) | (($val & 0xffff) << 16)", ("crate::implementations::value_conv", "canon_file_number_value_conv"));
    m.insert("(($val & 0xffc0) >> 6) * 10000 + (($val >> 16) & 0xff) + (($val & 0x3f) << 8)", ("crate::implementations::value_conv", "canon_directory_number_value_conv"));

    // Manual function mappings
    m.insert("gpslatitude_value_conv", ("crate::implementations::value_conv", "gps_coordinate_value_conv"));
    m.insert("gpslongitude_value_conv", ("crate::implementations::value_conv", "gps_coordinate_value_conv"));
    m.insert("gpsdestlatitude_value_conv", ("crate::implementations::value_conv", "gps_coordinate_value_conv"));
    m.insert("gpsdestlongitude_value_conv", ("crate::implementations::value_conv", "gps_coordinate_value_conv"));
    m.insert("gpstimestamp_value_conv", ("crate::implementations::value_conv", "gpstimestamp_value_conv"));
    m.insert("gpsdatestamp_value_conv", ("crate::implementations::value_conv", "gpsdatestamp_value_conv"));
    m.insert("whitebalance_value_conv", ("crate::implementations::value_conv", "whitebalance_value_conv"));
    m.insert("apex_shutter_speed_value_conv", ("crate::implementations::value_conv", "apex_shutter_speed_value_conv"));
    m.insert("apex_aperture_value_conv", ("crate::implementations::value_conv", "apex_aperture_value_conv"));
    m.insert("apex_exposure_compensation_value_conv", ("crate::implementations::value_conv", "apex_exposure_compensation_value_conv"));
    m.insert("fnumber_value_conv", ("crate::implementations::value_conv", "fnumber_value_conv"));
    m.insert("exposuretime_value_conv", ("crate::implementations::value_conv", "exposuretime_value_conv"));
    m.insert("focallength_value_conv", ("crate::implementations::value_conv", "focallength_value_conv"));
    
    // Common simple patterns found in supported tags
    m.insert("$val=~s/ +$//; $val", ("crate::implementations::value_conv", "trim_whitespace_value_conv"));
    m.insert("$val=~s/^.*: //;$val", ("crate::implementations::value_conv", "remove_prefix_colon_value_conv"));
    m.insert("2 ** (-$val/3)", ("crate::implementations::value_conv", "power_neg_div_3_value_conv"));
    m.insert("$val ? 10 / $val : 0", ("crate::implementations::value_conv", "reciprocal_10_value_conv"));
    m.insert("$val ? 2 ** (6 - $val/8) : 0", ("crate::implementations::value_conv", "sony_exposure_time_value_conv"));
    m.insert("$val ? exp(($val/8-6)*log(2))*100 : $val", ("crate::implementations::value_conv", "sony_iso_value_conv"));
    m.insert("2 ** (($val/8 - 1) / 2)", ("crate::implementations::value_conv", "sony_fnumber_value_conv"));
    m.insert("Image::ExifTool::Exif::ExifDate($val)", ("crate::implementations::value_conv", "exif_date_value_conv"));
    
    // ExifTool function calls for datetime conversions
    m.insert("require Image::ExifTool::XMP;\nreturn Image::ExifTool::XMP::ConvertXMPDate($val);", ("crate::implementations::value_conv", "xmp_date_value_conv"));
    
    // String processing patterns
    m.insert("length($val) > 32 ? \\$val : $val", ("crate::implementations::value_conv", "reference_long_string_value_conv"));
    m.insert("length($val) > 64 ? \\$val : $val", ("crate::implementations::value_conv", "reference_very_long_string_value_conv"));
    
    m
});

/// Look up a tag-specific PrintConv in the registry
/// First tries module-specific lookup (Module::Tag), then universal lookup (Tag)
pub fn lookup_tag_specific_printconv(module: &str, tag_name: &str) -> Option<(&'static str, &'static str)> {
    // First try module-specific lookup
    let module_key = format!("{}::{}", module, tag_name);
    if let Some(result) = TAG_SPECIFIC_PRINTCONV.get(module_key.as_str()).copied() {
        return Some(result);
    }
    
    // Then try universal lookup
    TAG_SPECIFIC_PRINTCONV.get(tag_name).copied()
}

/// Look up PrintConv implementation by Perl expression
/// Tries module-scoped lookup first, then unscoped
pub fn lookup_printconv(expr: &str, module: &str) -> Option<(&'static str, &'static str)> {
    // Normalize the expression for consistent lookup
    let normalized_expr = normalize_expression(expr);
    
    // Normalize module name (GPS_pm -> GPS)
    let normalized_module = module.replace("_pm", "");
    
    // Try module-scoped first with normalized expression
    let scoped_key = format!("{}::{}", normalized_module, normalized_expr);
    if let Some(value) = PRINTCONV_REGISTRY.get(scoped_key.as_str()) {
        return Some(*value);
    }
    
    // Fall back to exact match with normalized expression
    PRINTCONV_REGISTRY.get(normalized_expr.as_str()).copied()
}

/// Look up ValueConv implementation by Perl expression
pub fn lookup_valueconv(expr: &str, module: &str) -> Option<(&'static str, &'static str)> {
    // First try exact match (more efficient and avoids normalization issues)
    if let Some(value) = VALUECONV_REGISTRY.get(expr) {
        return Some(*value);
    }
    
    // Try module-scoped exact match
    let normalized_module = module.replace("_pm", "");
    let scoped_key = format!("{}::{}", normalized_module, expr);
    if let Some(value) = VALUECONV_REGISTRY.get(scoped_key.as_str()) {
        return Some(*value);
    }
    
    // Fall back to normalization for complex expressions
    let normalized_expr = normalize_expression(expr);
    
    // Try normalized module-scoped lookup
    let normalized_scoped_key = format!("{}::{}", normalized_module, normalized_expr);
    if let Some(value) = VALUECONV_REGISTRY.get(normalized_scoped_key.as_str()) {
        return Some(*value);
    }
    
    // Try normalized global lookup
    VALUECONV_REGISTRY.get(normalized_expr.as_str()).copied()
}

/// Normalize expression for consistent lookup
/// Uses Perl to normalize Perl expressions
pub fn normalize_expression(expr: &str) -> String {
    // Check cache first
    if let Ok(cache) = NORMALIZATION_CACHE.lock() {
        if let Some(normalized) = cache.get(expr) {
            return normalized.clone();
        }
    }
    
    // Use Perl normalization
    let normalized = match normalize_with_perl(expr) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Warning: Failed to normalize expression '{}': {}", expr, e);
            eprintln!("Using original expression");
            expr.to_string()
        }
    };
    
    // Cache the result
    if let Ok(mut cache) = NORMALIZATION_CACHE.lock() {
        cache.insert(expr.to_string(), normalized.clone());
    }
    
    normalized
}

/// Batch normalize multiple expressions in a single Perl call
/// This is much more efficient than calling normalize_expression repeatedly
pub fn batch_normalize_expressions(expressions: &[String]) -> Result<HashMap<String, String>, String> {
    // Filter out expressions that are already cached
    let uncached: Vec<String> = {
        let cache = NORMALIZATION_CACHE.lock().map_err(|_| "Cache lock failed")?;
        expressions.iter()
            .filter(|expr| !cache.contains_key(*expr))
            .cloned()
            .collect()
    };
    
    if uncached.is_empty() {
        // All expressions are cached, return cached results
        let cache = NORMALIZATION_CACHE.lock().map_err(|_| "Cache lock failed")?;
        return Ok(expressions.iter()
            .filter_map(|expr| cache.get(expr).map(|normalized| (expr.clone(), normalized.clone())))
            .collect());
    }
    
    // Batch normalize uncached expressions
    let batch_results = normalize_batch_with_perl(&uncached)?;
    
    // Update cache with new results
    {
        let mut cache = NORMALIZATION_CACHE.lock().map_err(|_| "Cache lock failed")?;
        for (original, normalized) in &batch_results {
            cache.insert(original.clone(), normalized.clone());
        }
    }
    
    // Return all results (cached + new)
    let cache = NORMALIZATION_CACHE.lock().map_err(|_| "Cache lock failed")?;
    Ok(expressions.iter()
        .filter_map(|expr| cache.get(expr).map(|normalized| (expr.clone(), normalized.clone())))
        .collect())
}

/// Call Perl script to normalize multiple expressions in batch
fn normalize_batch_with_perl(expressions: &[String]) -> Result<HashMap<String, String>, String> {
    use std::io::Write;
    use std::process::Stdio;
    
    // Find the normalize script by searching up from current directory
    let script_path = find_normalize_script()
        .ok_or_else(|| "Could not find normalize_expression.pl script".to_string())?;
    
    // Set up Perl environment for local::lib
    let home_dir = std::env::var("HOME")
        .map_err(|_| "HOME environment variable not set".to_string())?;
    let perl5lib = format!("{}/perl5/lib/perl5", home_dir);
    
    // Call the Perl script with stdin and proper environment
    let mut child = Command::new("perl")
        .arg("-I")
        .arg(&perl5lib)
        .arg("-Mlocal::lib")
        .arg(&script_path)
        .env("PERL5LIB", &perl5lib)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to execute Perl: {}", e))?;
    
    // Write all expressions to stdin, separated by the delimiter
    if let Some(mut stdin) = child.stdin.take() {
        let batch_input = expressions.join("\n\n\n\n");
        stdin.write_all(batch_input.as_bytes())
            .map_err(|e| format!("Failed to write to Perl stdin: {}", e))?;
    }
    
    // Wait for completion and get output
    let output = child.wait_with_output()
        .map_err(|e| format!("Failed to wait for Perl process: {}", e))?;
    
    if output.status.success() {
        let stdout_str = String::from_utf8(output.stdout)
            .map_err(|e| format!("Invalid UTF-8 in Perl output: {}", e))?;
        
        // Parse the batch response - each normalized expression is separated by the same delimiter
        let normalized_expressions: Vec<&str> = stdout_str.split("\n\n\n\n").collect();
        
        if normalized_expressions.len() != expressions.len() {
            return Err(format!(
                "Batch normalization mismatch: sent {} expressions, got {} results",
                expressions.len(),
                normalized_expressions.len()
            ));
        }
        
        // Create mapping from original to normalized
        let mut results = HashMap::new();
        for (original, normalized) in expressions.iter().zip(normalized_expressions.iter()) {
            results.insert(original.clone(), normalized.trim().to_string());
        }
        
        Ok(results)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Perl script failed: {}", stderr))
    }
}

/// Call Perl script to normalize expression
fn normalize_with_perl(expr: &str) -> Result<String, String> {
    use std::io::Write;
    use std::process::Stdio;
    
    // Find the normalize script by searching up from current directory
    let script_path = find_normalize_script()
        .ok_or_else(|| "Could not find normalize_expression.pl script".to_string())?;
    
    // Set up Perl environment for local::lib
    let home_dir = std::env::var("HOME")
        .map_err(|_| "HOME environment variable not set".to_string())?;
    let perl5lib = format!("{}/perl5/lib/perl5", home_dir);
    
    // Call the Perl script with stdin and proper environment
    let mut child = Command::new("perl")
        .arg("-I")
        .arg(&perl5lib)
        .arg("-Mlocal::lib")
        .arg(&script_path)
        .env("PERL5LIB", &perl5lib)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to execute Perl: {}", e))?;
    
    // Write expression to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(expr.as_bytes())
            .map_err(|e| format!("Failed to write to Perl stdin: {}", e))?;
    }
    
    // Wait for completion and get output
    let output = child.wait_with_output()
        .map_err(|e| format!("Failed to wait for Perl process: {}", e))?;
    
    if output.status.success() {
        String::from_utf8(output.stdout)
            .map_err(|e| format!("Invalid UTF-8 in Perl output: {}", e))
            .map(|s| s.trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Perl script failed: {}", stderr))
    }
}

/// Find the normalize_expression.pl script by searching up the directory tree
fn find_normalize_script() -> Option<std::path::PathBuf> {
    let mut current = std::env::current_dir().ok()?;
    
    // Search up to 5 levels
    for _ in 0..5 {
        // Check if we're in the codegen directory
        let script_path = current.join("extractors").join("normalize_expression.pl");
        if script_path.exists() {
            return Some(script_path);
        }
        
        // Check if we're in the project root
        let codegen_script = current.join("codegen").join("extractors").join("normalize_expression.pl");
        if codegen_script.exists() {
            return Some(codegen_script);
        }
        
        // Move up one directory
        current = current.parent()?.to_path_buf();
    }
    
    None
}

/// Classify a ValueConv expression for code generation
/// 
/// Determines whether an expression can be compiled to inline arithmetic code
/// or requires a custom function implementation.
pub fn classify_valueconv_expression(expr: &str, module: &str) -> ValueConvType {
    // First check if it's a compilable arithmetic expression
    if CompiledExpression::is_compilable(expr) {
        match CompiledExpression::compile(expr) {
            Ok(compiled) => return ValueConvType::CompiledExpression(compiled),
            Err(_) => {
                // Fall through to custom function lookup
                eprintln!("Warning: Expression '{}' looked compilable but failed compilation", expr);
            }
        }
    }
    
    // Look up custom function in registry
    if let Some((module_path, func_name)) = lookup_valueconv(expr, module) {
        ValueConvType::CustomFunction(module_path, func_name)
    } else {
        // Fallback - treat as unregistered custom function
        // This preserves existing behavior for unknown expressions
        ValueConvType::CustomFunction("crate::implementations::missing", "missing_value_conv")
    }
}

/// Get a list of all simple arithmetic expressions that can be compiled
/// Used for documentation and debugging
pub fn get_compilable_expressions() -> Vec<&'static str> {
    let mut compilable = Vec::new();
    
    // Check all expressions in the registry
    for &expr in VALUECONV_REGISTRY.keys() {
        if CompiledExpression::is_compilable(expr) {
            compilable.push(expr);
        }
    }
    
    compilable.sort();
    compilable
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_module_scoped_lookup() {
        // Test direct lookup of a known value
        let result = lookup_valueconv("Image::ExifTool::GPS::ConvertTimeStamp($val)", "GPS_pm");
        assert_eq!(result, Some(("crate::implementations::value_conv", "gpstimestamp_value_conv")));
    }
    
    #[test]
    fn test_manual_printconv_lookup() {
        let result = lookup_printconv("fnumber_print_conv", "Exif_pm");
        assert_eq!(result, Some(("crate::implementations::print_conv", "fnumber_print_conv")));
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
        assert_eq!(
            normalize_expression("\"$val mm\""),
            "\"$val mm\""
        );
        
        assert_eq!(
            normalize_expression("\"$val m\""),
            "\"$val m\""
        );
        
        assert_eq!(
            normalize_expression("\"$val C\""),
            "\"$val C\""
        );
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
        assert_eq!(
            normalize_expression("\"$val m\""),
            "\"$val m\""
        );
        
        // Spaces in regex patterns should be preserved
        assert_eq!(
            normalize_expression("$val =~ tr/ /./"),
            "$val =~ tr/ /./"
        );
        
        // Spaces between operators should be preserved
        assert_eq!(
            normalize_expression("$val > 0"),
            "$val > 0"
        );
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
        let mut normalization_issues = Vec::new();
        
        // Check PRINTCONV_REGISTRY keys
        for &key in PRINTCONV_REGISTRY.keys() {
            let normalized = normalize_expression(key);
            if key != normalized {
                normalization_issues.push(format!(
                    "PRINTCONV_REGISTRY key '{}' should be normalized to '{}'", 
                    key, normalized
                ));
            }
        }
        
        // Check VALUECONV_REGISTRY keys
        for &key in VALUECONV_REGISTRY.keys() {
            let normalized = normalize_expression(key);
            if key != normalized {
                normalization_issues.push(format!(
                    "VALUECONV_REGISTRY key '{}' should be normalized to '{}'", 
                    key, normalized
                ));
            }
        }
        
        // Check TAG_SPECIFIC_PRINTCONV keys (excluding module-scoped ones)
        for &key in TAG_SPECIFIC_PRINTCONV.keys() {
            // Skip module-scoped keys (contain ::)
            if !key.contains("::") {
                let normalized = normalize_expression(key);
                if key != normalized {
                    normalization_issues.push(format!(
                        "TAG_SPECIFIC_PRINTCONV key '{}' should be normalized to '{}'", 
                        key, normalized
                    ));
                }
            }
        }
        
        if !normalization_issues.is_empty() {
            panic!("Registry contains keys that need normalization:\n{}", 
                   normalization_issues.join("\n"));
        }
    }
    
    #[test]
    fn test_classify_valueconv_expression() {
        // Test simple arithmetic expressions get compiled
        match classify_valueconv_expression("$val / 8", "Canon_pm") {
            ValueConvType::CompiledExpression(_) => {},
            _ => panic!("Expected compiled expression"),
        }
        
        match classify_valueconv_expression("($val - 104) / 8", "Nikon_pm") {
            ValueConvType::CompiledExpression(_) => {},
            _ => panic!("Expected compiled expression"),
        }
        
        // Test complex expressions get custom functions
        match classify_valueconv_expression("$val ? 10 / $val : 0", "Sony_pm") {
            ValueConvType::CustomFunction(_, _) => {},
            _ => panic!("Expected custom function"),
        }
        
        match classify_valueconv_expression("exp($val / 32 * log(2)) * 100", "Canon_pm") {
            ValueConvType::CustomFunction(_, _) => {},
            _ => panic!("Expected custom function"),
        }
    }
    
    #[test]
    fn test_get_compilable_expressions() {
        let compilable = get_compilable_expressions();
        
        // Should include simple arithmetic expressions
        assert!(compilable.contains(&"$val / 8"));
        assert!(compilable.contains(&"$val * 100"));
        assert!(compilable.contains(&"($val-104)/8"));
        
        // Should not include complex expressions
        assert!(!compilable.contains(&"$val ? 10 / $val : 0"));
        assert!(!compilable.contains(&"exp($val / 32 * log(2)) * 100"));
        
        println!("Found {} compilable expressions", compilable.len());
        for expr in &compilable[..5.min(compilable.len())] {
            println!("  - {}", expr);
        }
    }
}