//! Implementation registry for runtime PrintConv/ValueConv lookup
//!
//! This module provides the core registry system that allows runtime lookup
//! of conversion functions rather than generating thousands of stub functions.
//! This approach keeps the codebase manageable while providing flexibility.

use crate::types::{Result, TagValue};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock, RwLock};

/// Function signature for PrintConv implementations
///
/// PrintConv functions convert logical values to display values.
/// The return type can be:
/// - String for human-readable formatting (e.g., "1/100", "Rotate 90 CW")
/// - Numeric TagValue for data that should remain numeric in JSON (e.g., ISO: 100)
///
/// This design allows PrintConv to control JSON serialization type directly,
/// avoiding ExifTool's regex-based type guessing.
pub type PrintConvFn = fn(&TagValue) -> TagValue;

/// Function signature for ValueConv implementations
///
/// ValueConv functions convert raw values to logical values.
/// For example: APEX values to actual f-stop numbers
pub type ValueConvFn = fn(&TagValue) -> Result<TagValue>;

/// Function signature for RawConv implementations
///
/// RawConv functions are applied first to raw tag values before ValueConv/PrintConv.
/// Used for decoding or special processing of raw data (e.g., UserComment character encoding)
pub type RawConvFn = fn(&TagValue) -> Result<TagValue>;

// Global registry instance - uses LazyLock to create a singleton registry that can be
// accessed from anywhere in the application. RwLock allows concurrent reads while protecting writes.
static GLOBAL_REGISTRY: LazyLock<Arc<RwLock<Registry>>> =
    LazyLock::new(|| Arc::new(RwLock::new(Registry::new())));

/// Core registry for conversion function lookup
///
/// The registry maps function names to actual function pointers.
/// This enables runtime dispatch without code generation.
#[derive(Debug)]
pub struct Registry {
    /// PrintConv function registry
    print_conv: HashMap<String, PrintConvFn>,

    /// ValueConv function registry  
    value_conv: HashMap<String, ValueConvFn>,

    /// RawConv function registry
    raw_conv: HashMap<String, RawConvFn>,

    /// Track requested but missing implementations
    missing_print_conv: HashSet<String>,
    missing_value_conv: HashSet<String>,
    missing_raw_conv: HashSet<String>,

    /// Statistics for coverage analysis
    print_conv_hits: HashMap<String, u64>,
    print_conv_misses: HashMap<String, u64>,
}

impl Registry {
    /// Create new empty registry
    pub fn new() -> Self {
        Self {
            print_conv: HashMap::new(),
            value_conv: HashMap::new(),
            raw_conv: HashMap::new(),
            missing_print_conv: HashSet::new(),
            missing_value_conv: HashSet::new(),
            missing_raw_conv: HashSet::new(),
            print_conv_hits: HashMap::new(),
            print_conv_misses: HashMap::new(),
        }
    }

    /// Register a PrintConv function
    ///
    /// # Arguments
    /// * `name` - Function name (e.g., "exif_orientation")
    /// * `func` - Function pointer
    pub fn register_print_conv(&mut self, name: impl Into<String>, func: PrintConvFn) {
        let name = name.into();
        self.print_conv.insert(name, func);
    }

    /// Register a ValueConv function
    ///
    /// # Arguments  
    /// * `name` - Function name (e.g., "apex_shutter_speed")
    /// * `func` - Function pointer
    pub fn register_value_conv(&mut self, name: impl Into<String>, func: ValueConvFn) {
        let name = name.into();
        self.value_conv.insert(name, func);
    }

    /// Register a RawConv function
    ///
    /// # Arguments  
    /// * `name` - Function name (e.g., "convert_exif_text")
    /// * `func` - Function pointer
    pub fn register_raw_conv(&mut self, name: impl Into<String>, func: RawConvFn) {
        let name = name.into();
        self.raw_conv.insert(name, func);
    }

    /// Look up and execute a PrintConv function
    ///
    /// # Arguments
    /// * `name` - Function name to look up
    /// * `value` - Value to convert
    ///
    /// # Returns
    /// Converted value (string or numeric), or the original value if not found
    pub fn apply_print_conv(&mut self, name: &str, value: &TagValue) -> TagValue {
        if let Some(func) = self.print_conv.get(name) {
            // Track successful hit
            *self.print_conv_hits.entry(name.to_string()).or_insert(0) += 1;
            func(value)
        } else {
            // Track miss and return original value
            *self.print_conv_misses.entry(name.to_string()).or_insert(0) += 1;
            self.missing_print_conv.insert(name.to_string());
            value.clone()
        }
    }

    /// Look up and execute a ValueConv function
    ///
    /// # Arguments
    /// * `name` - Function name to look up  
    /// * `value` - Value to convert
    ///
    /// # Returns
    /// Converted value, or original value if not found or conversion failed
    pub fn apply_value_conv(&mut self, name: &str, value: &TagValue) -> TagValue {
        if let Some(func) = self.value_conv.get(name) {
            match func(value) {
                Ok(converted) => converted,
                Err(_) => {
                    // Log error but don't crash - return original value
                    value.clone()
                }
            }
        } else {
            // Track miss and return original value
            self.missing_value_conv.insert(name.to_string());
            value.clone()
        }
    }

    /// Look up and execute a RawConv function
    ///
    /// # Arguments
    /// * `name` - Function name to look up  
    /// * `value` - Value to convert
    ///
    /// # Returns
    /// Converted value, or original value if not found or conversion failed
    pub fn apply_raw_conv(&mut self, name: &str, value: &TagValue) -> TagValue {
        if let Some(func) = self.raw_conv.get(name) {
            match func(value) {
                Ok(converted) => converted,
                Err(_) => {
                    // Log error but don't crash - return original value
                    value.clone()
                }
            }
        } else {
            // Track miss and return original value
            self.missing_raw_conv.insert(name.to_string());
            value.clone()
        }
    }

    /// Get list of missing PrintConv implementations
    pub fn get_missing_print_conv(&self) -> Vec<String> {
        self.missing_print_conv.iter().cloned().collect()
    }

    /// Get list of missing ValueConv implementations  
    pub fn get_missing_value_conv(&self) -> Vec<String> {
        self.missing_value_conv.iter().cloned().collect()
    }

    /// Get list of missing RawConv implementations  
    pub fn get_missing_raw_conv(&self) -> Vec<String> {
        self.missing_raw_conv.iter().cloned().collect()
    }

    /// Get PrintConv coverage statistics
    pub fn get_print_conv_stats(&self) -> (usize, usize, usize) {
        let total_registered = self.print_conv.len();
        let total_hit = self.print_conv_hits.len();
        let total_missed = self.print_conv_misses.len();
        (total_registered, total_hit, total_missed)
    }

    /// Clear all missing implementation tracking (useful for testing)
    pub fn clear_missing_tracking(&mut self) {
        self.missing_print_conv.clear();
        self.missing_value_conv.clear();
        self.missing_raw_conv.clear();
        self.print_conv_misses.clear();
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global registry access functions
///
/// These provide a convenient interface to the singleton registry
///
/// Register a PrintConv function globally
pub fn register_print_conv(name: impl Into<String>, func: PrintConvFn) {
    let mut registry = GLOBAL_REGISTRY.write().unwrap();
    registry.register_print_conv(name, func);
}

/// Register a ValueConv function globally
pub fn register_value_conv(name: impl Into<String>, func: ValueConvFn) {
    let mut registry = GLOBAL_REGISTRY.write().unwrap();
    registry.register_value_conv(name, func);
}

/// Register a RawConv function globally
pub fn register_raw_conv(name: impl Into<String>, func: RawConvFn) {
    let mut registry = GLOBAL_REGISTRY.write().unwrap();
    registry.register_raw_conv(name, func);
}

/// Apply PrintConv globally with tag kit integration
pub fn apply_print_conv(name: &str, value: &TagValue) -> TagValue {
    apply_print_conv_with_tag_id(None, name, value)
}

/// Apply PrintConv globally with tag ID for tag kit integration
pub fn apply_print_conv_with_tag_id(tag_id: Option<u32>, name: &str, value: &TagValue) -> TagValue {
    // First try tag kit if tag ID is available
    if let Some(id) = tag_id {
        // Check if we have an EXIF tag kit for this tag
        if let Some(tag_kit_result) = try_tag_kit_print_conv(id, value) {
            return tag_kit_result;
        }
    }

    // Fall back to manual registry lookup by function name
    let mut registry = GLOBAL_REGISTRY.write().unwrap();
    registry.apply_print_conv(name, value)
}

/// Try to apply PrintConv using tag kit system
fn try_tag_kit_print_conv(tag_id: u32, value: &TagValue) -> Option<TagValue> {
    // For now, try EXIF tag kit (we can extend this to other modules later)
    use crate::expressions::ExpressionEvaluator;
    use crate::generated::Exif_pm::tag_kit;

    // Create temporary containers for errors/warnings
    // TODO: These should be passed through the API to collect for the user
    let mut expression_evaluator = ExpressionEvaluator::new();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let result = tag_kit::apply_print_conv(
        tag_id,
        value,
        &mut expression_evaluator,
        &mut errors,
        &mut warnings,
    );

    // Check if tag kit actually handled this tag (didn't just return the original value)
    if result != *value {
        Some(result)
    } else if tag_kit::EXIF_PM_TAG_KITS.get(&tag_id).is_some() {
        // Tag kit contains this tag but couldn't convert it (e.g., None PrintConv)
        Some(result)
    } else {
        // Tag kit doesn't know about this tag
        None
    }
}

/// Apply ValueConv globally  
pub fn apply_value_conv(name: &str, value: &TagValue) -> TagValue {
    let mut registry = GLOBAL_REGISTRY.write().unwrap();
    registry.apply_value_conv(name, value)
}

/// Apply RawConv globally  
pub fn apply_raw_conv(name: &str, value: &TagValue) -> TagValue {
    let mut registry = GLOBAL_REGISTRY.write().unwrap();
    registry.apply_raw_conv(name, value)
}

/// Get missing implementations for --show-missing
pub fn get_missing_implementations() -> (Vec<String>, Vec<String>) {
    let registry = GLOBAL_REGISTRY.read().unwrap();
    (
        registry.get_missing_print_conv(),
        registry.get_missing_value_conv(),
    )
}

/// Get coverage statistics
pub fn get_coverage_stats() -> (usize, usize, usize) {
    let registry = GLOBAL_REGISTRY.read().unwrap();
    registry.get_print_conv_stats()
}

/// Clear missing tracking (for testing)
pub fn clear_missing_tracking() {
    let mut registry = GLOBAL_REGISTRY.write().unwrap();
    registry.clear_missing_tracking();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_basic_operations() {
        let mut registry = Registry::new();

        // Test PrintConv registration and lookup
        fn test_print_conv(val: &TagValue) -> TagValue {
            TagValue::String(format!("Test: {val}"))
        }

        registry.register_print_conv("test", test_print_conv);

        let value = TagValue::U16(42);
        let result = registry.apply_print_conv("test", &value);
        assert_eq!(result, TagValue::String("Test: 42".to_string()));

        // Test missing function fallback
        let missing_result = registry.apply_print_conv("missing", &value);
        assert_eq!(missing_result, TagValue::U16(42));

        // Test missing tracking
        let missing = registry.get_missing_print_conv();
        assert!(missing.contains(&"missing".to_string()));
    }

    #[test]
    fn test_global_registry() {
        // Clear any previous state
        clear_missing_tracking();

        // Test global registration
        fn test_global(val: &TagValue) -> TagValue {
            TagValue::String(format!("Global: {val}"))
        }

        register_print_conv("global_test", test_global);

        let value = TagValue::String("hello".to_string());
        let result = apply_print_conv("global_test", &value);
        assert_eq!(result, TagValue::String("Global: hello".to_string()));
    }
}
