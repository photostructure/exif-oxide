//! Function call registry for mapping complex ExifTool function calls to Rust implementations
//!
//! This module provides lookup tables for complex function calls that can't be handled
//! by the AST generation system, including:
//! - ExifTool module functions (Image::ExifTool::Canon::CanonEv)
//! - Perl builtin functions (sprintf, substr, uc, lc)
//! - Custom multi-line scripts
//!
//! ## Design: Direct Function Name Matching
//!
//! The registry uses **exact function signature matching** without normalization,
//! similar to the PrintConv/ValueConv registry approach. This means:
//! - Fast lookup with no parsing overhead
//! - Predictable mapping from ExifTool source to Rust implementation
//! - Multiple entries for different parameter patterns if needed
//!
//! ## Integration with Unified Expression System
//!
//! This registry serves as a fallback for the unified expression system when
//! AST generation encounters complex function calls that require specialized handling.
//! The flow is: AST parsing → Simple/Complex classification → Function registry lookup
//! → Rust implementation dispatch

use std::collections::HashMap;
use std::sync::LazyLock;
use tracing::trace;

/// Function implementation types for different categories of function calls
#[derive(Debug, Clone)]
pub enum FunctionImplementation {
    /// Perl builtin functions (sprintf, substr, etc.)
    Builtin(BuiltinFunction),
    /// ExifTool module functions (Image::ExifTool::Canon::*, etc.)
    ExifToolModule(ModuleFunction),
    /// Multi-line perl scripts or complex conditional logic
    CustomScript(ScriptFunction),
}

/// Builtin Perl function mappings to Rust implementations
#[derive(Debug, Clone)]
pub struct BuiltinFunction {
    pub module_path: &'static str,
    pub function_name: &'static str,
    pub parameter_pattern: &'static str, // For validation/documentation
}

/// ExifTool module function mappings
#[derive(Debug, Clone)]
pub struct ModuleFunction {
    pub module_path: &'static str,
    pub function_name: &'static str,
    pub exiftool_module: &'static str, // Original ExifTool module (Canon, GPS, etc.)
}

/// Custom script function mappings for complex multi-line expressions
#[derive(Debug, Clone)]
pub struct ScriptFunction {
    pub module_path: &'static str,
    pub function_name: &'static str,
    pub description: &'static str, // Brief description of what the script does
}

// Registry for function call lookups
static FUNCTION_CALL_REGISTRY: LazyLock<HashMap<&'static str, FunctionImplementation>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();

        // Perl builtin functions
        m.insert(
            "sprintf",
            FunctionImplementation::Builtin(BuiltinFunction {
                module_path: "crate::implementations::builtins",
                function_name: "sprintf_impl",
                parameter_pattern: "(format_string, ...args)",
            }),
        );

        m.insert(
            "substr",
            FunctionImplementation::Builtin(BuiltinFunction {
                module_path: "crate::implementations::builtins",
                function_name: "substr_impl",
                parameter_pattern: "(string, offset, length?)",
            }),
        );

        m.insert(
            "uc",
            FunctionImplementation::Builtin(BuiltinFunction {
                module_path: "crate::implementations::builtins",
                function_name: "uc_impl",
                parameter_pattern: "(string)",
            }),
        );

        m.insert(
            "lc",
            FunctionImplementation::Builtin(BuiltinFunction {
                module_path: "crate::implementations::builtins",
                function_name: "lc_impl",
                parameter_pattern: "(string)",
            }),
        );

        // ExifTool module functions - Canon
        m.insert(
            "Image::ExifTool::Canon::CanonEv",
            FunctionImplementation::ExifToolModule(ModuleFunction {
                module_path: "crate::implementations::canon",
                function_name: "canon_ev",
                exiftool_module: "Canon",
            }),
        );

        m.insert(
            "Image::ExifTool::Canon::CanonEvInv",
            FunctionImplementation::ExifToolModule(ModuleFunction {
                module_path: "crate::implementations::canon",
                function_name: "canon_ev_inv",
                exiftool_module: "Canon",
            }),
        );

        m.insert(
            "Image::ExifTool::Canon::CalcSensorDiag",
            FunctionImplementation::ExifToolModule(ModuleFunction {
                module_path: "crate::implementations::canon",
                function_name: "calc_sensor_diag",
                exiftool_module: "Canon",
            }),
        );

        m.insert(
            "Image::ExifTool::Canon::PrintLensID",
            FunctionImplementation::ExifToolModule(ModuleFunction {
                module_path: "crate::implementations::canon",
                function_name: "print_lens_id",
                exiftool_module: "Canon",
            }),
        );

        // ExifTool module functions - GPS
        m.insert(
            "Image::ExifTool::GPS::ToDegrees",
            FunctionImplementation::ExifToolModule(ModuleFunction {
                module_path: "crate::implementations::gps",
                function_name: "to_degrees",
                exiftool_module: "GPS",
            }),
        );

        m.insert(
            "Image::ExifTool::GPS::ToDMS",
            FunctionImplementation::ExifToolModule(ModuleFunction {
                module_path: "crate::implementations::gps",
                function_name: "to_dms",
                exiftool_module: "GPS",
            }),
        );

        // ExifTool module functions - XMP
        m.insert(
            "Image::ExifTool::XMP::ConvertXMPDate",
            FunctionImplementation::ExifToolModule(ModuleFunction {
                module_path: "crate::implementations::xmp",
                function_name: "convert_xmp_date",
                exiftool_module: "XMP",
            }),
        );

        m.insert(
            "Image::ExifTool::XMP::FormatXMPDate",
            FunctionImplementation::ExifToolModule(ModuleFunction {
                module_path: "crate::implementations::xmp",
                function_name: "format_xmp_date",
                exiftool_module: "XMP",
            }),
        );

        m.insert(
            "Image::ExifTool::XMP::UnescapeXML",
            FunctionImplementation::ExifToolModule(ModuleFunction {
                module_path: "crate::implementations::xmp",
                function_name: "unescape_xml",
                exiftool_module: "XMP",
            }),
        );

        // ExifTool module functions - QuickTime
        m.insert(
            "Image::ExifTool::QuickTime::CalcSampleRate",
            FunctionImplementation::ExifToolModule(ModuleFunction {
                module_path: "crate::implementations::quicktime",
                function_name: "calc_sample_rate",
                exiftool_module: "QuickTime",
            }),
        );

        m.insert(
            "Image::ExifTool::QuickTime::UnpackLang",
            FunctionImplementation::ExifToolModule(ModuleFunction {
                module_path: "crate::implementations::quicktime",
                function_name: "unpack_lang",
                exiftool_module: "QuickTime",
            }),
        );

        // Complex script functions (multi-line conditionals, complex regex, etc.)
        m.insert(
            "complex_binary_data_condition",
            FunctionImplementation::CustomScript(ScriptFunction {
                module_path: "crate::implementations::complex_conditions",
                function_name: "complex_binary_data_condition",
                description: "Multi-line conditional for binary data parsing",
            }),
        );

        m.insert(
            "complex_makernote_dispatch",
            FunctionImplementation::CustomScript(ScriptFunction {
                module_path: "crate::implementations::complex_conditions",
                function_name: "complex_makernote_dispatch",
                description: "Complex maker note format detection and dispatch",
            }),
        );

        m.insert(
            "complex_regex_with_binary",
            FunctionImplementation::CustomScript(ScriptFunction {
                module_path: "crate::implementations::complex_conditions",
                function_name: "complex_regex_with_binary",
                description: "Complex regex patterns operating on binary data",
            }),
        );

        m
    });

// Registry for function signature patterns (for more flexible matching)
// Key: normalized function pattern, Value: function name for main registry lookup
static FUNCTION_PATTERN_REGISTRY: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();

        // sprintf patterns with different formatting
        m.insert("sprintf(", "sprintf");
        m.insert("sprintf (", "sprintf"); // with space

        // Different parameter patterns for common functions
        m.insert(
            "Image::ExifTool::Canon::CanonEv(",
            "Image::ExifTool::Canon::CanonEv",
        );
        m.insert(
            "Image::ExifTool::GPS::ToDegrees(",
            "Image::ExifTool::GPS::ToDegrees",
        );
        m.insert(
            "Image::ExifTool::GPS::ToDMS(",
            "Image::ExifTool::GPS::ToDMS",
        );

        // Add more patterns as needed for flexible matching

        m
    });

/// Look up a function implementation by exact function name or call pattern
pub fn lookup_function(function_call: &str) -> Option<&'static FunctionImplementation> {
    trace!("🔍 Function lookup for: '{}'", function_call);

    // First try exact match
    if let Some(implementation) = FUNCTION_CALL_REGISTRY.get(function_call) {
        trace!("✅ Found exact function match: '{}'", function_call);
        return Some(implementation);
    }

    // Try pattern matching for flexible lookup
    for (pattern, canonical_name) in FUNCTION_PATTERN_REGISTRY.iter() {
        if function_call.starts_with(pattern) {
            if let Some(implementation) = FUNCTION_CALL_REGISTRY.get(canonical_name) {
                trace!(
                    "✅ Found pattern match: '{}' -> '{}'",
                    function_call,
                    canonical_name
                );
                return Some(implementation);
            }
        }
    }

    trace!(
        "❌ No function implementation found for: '{}'",
        function_call
    );
    None
}

/// Look up a function by category for debugging/introspection
pub fn lookup_functions_by_category(
    category: FunctionCategory,
) -> Vec<(&'static str, &'static FunctionImplementation)> {
    FUNCTION_CALL_REGISTRY
        .iter()
        .filter(|(_, implementation)| match (category, implementation) {
            (FunctionCategory::Builtin, FunctionImplementation::Builtin(_)) => true,
            (FunctionCategory::ExifToolModule, FunctionImplementation::ExifToolModule(_)) => true,
            (FunctionCategory::CustomScript, FunctionImplementation::CustomScript(_)) => true,
            _ => false,
        })
        .map(|(name, implementation)| (*name, implementation))
        .collect()
}

/// Categories for function lookup filtering
#[derive(Debug, Clone, Copy)]
pub enum FunctionCategory {
    Builtin,
    ExifToolModule,
    CustomScript,
}

/// Check if a function call looks like it needs registry lookup
/// This is used by the unified expression system to determine fallback strategy
pub fn needs_function_registry_lookup(expression: &str) -> bool {
    // Check for ExifTool module function patterns
    if expression.contains("Image::ExifTool::") {
        return true;
    }

    // Check for Perl builtins in complex contexts
    let builtins = ["sprintf(", "substr(", "uc(", "lc("];
    for builtin in &builtins {
        if expression.contains(builtin) {
            return true;
        }
    }

    // Check for multi-line expressions (heuristic)
    if expression.lines().count() > 1 {
        return true;
    }

    // Check for complex regex patterns
    if expression.contains("=~") && expression.contains("/") {
        let regex_count = expression.matches('/').count();
        if regex_count >= 2 {
            // At least one regex pattern
            return true;
        }
    }

    false
}

/// Get function implementation details for code generation
pub fn get_function_details(function_name: &str) -> Option<FunctionDetails> {
    lookup_function(function_name).map(|implementation| match implementation {
        FunctionImplementation::Builtin(builtin) => FunctionDetails {
            module_path: builtin.module_path.to_string(),
            function_name: builtin.function_name.to_string(),
            category: "builtin".to_string(),
            description: format!("Perl builtin: {}", builtin.parameter_pattern),
        },
        FunctionImplementation::ExifToolModule(module_func) => FunctionDetails {
            module_path: module_func.module_path.to_string(),
            function_name: module_func.function_name.to_string(),
            category: "exiftool_module".to_string(),
            description: format!("ExifTool {} module function", module_func.exiftool_module),
        },
        FunctionImplementation::CustomScript(script) => FunctionDetails {
            module_path: script.module_path.to_string(),
            function_name: script.function_name.to_string(),
            category: "custom_script".to_string(),
            description: script.description.to_string(),
        },
    })
}

/// Detailed function information for code generation and documentation
#[derive(Debug, Clone)]
pub struct FunctionDetails {
    pub module_path: String,
    pub function_name: String,
    pub category: String,
    pub description: String,
}

/// Get access to the function registry for testing
#[cfg(test)]
pub fn get_function_registry() -> &'static HashMap<&'static str, FunctionImplementation> {
    &FUNCTION_CALL_REGISTRY
}

/// Get access to the pattern registry for testing  
#[cfg(test)]
pub fn get_pattern_registry() -> &'static HashMap<&'static str, &'static str> {
    &FUNCTION_PATTERN_REGISTRY
}
