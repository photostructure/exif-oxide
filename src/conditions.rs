//! Conditional dispatch system for ExifTool compatibility
//!
//! This module implements ExifTool's conditional processor selection based on
//! runtime data patterns, camera models, and other conditions.
//!
//! ExifTool uses conditions like:
//! - `$$valPt =~ /^0204/` - Data pattern matching
//! - `$$self{Model} =~ /\b1DS?$/` - Camera model matching  
//! - `$count == 368` - Count-based conditions
//! - `$format eq "undef"` - Format-based conditions

use crate::types::{ExifError, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::sync::Mutex;

/// Condition types for runtime processor selection
/// ExifTool: Condition expressions in tag tables
#[derive(Debug, Clone)]
pub enum Condition {
    /// Data pattern matching: `$$valPt =~ /pattern/`
    /// ExifTool: Binary signature matching like `$$valPt =~ /^0204/`
    DataPattern(String),

    /// Camera model matching: `$$self{Model} =~ /pattern/`
    /// ExifTool: Model-specific table selection like `$$self{Model} =~ /\b1DS?$/`
    ModelMatch(String),

    /// Camera make matching: `$$self{Make} =~ /pattern/`
    /// ExifTool: Manufacturer detection like `$$self{Make} =~ /^Canon/`
    MakeMatch(String),

    /// Count equality: `$count == 368`
    /// ExifTool: Data structure size validation
    CountEquals(u32),

    /// Count range: `$count >= min && $count <= max`
    /// ExifTool: Flexible count validation for variable structures
    CountRange(u32, u32),

    /// Format equality: `$format eq "undef"`
    /// ExifTool: Format-based dispatch decisions
    FormatEquals(String),

    /// Negation: `$$valPt!~/pattern/`
    /// ExifTool: Negative pattern matching
    Not(Box<Condition>),

    /// Logical AND: Multiple conditions must be true
    /// ExifTool: Complex conditional expressions
    And(Vec<Condition>),

    /// Logical OR: Any condition must be true
    /// ExifTool: Alternative dispatch paths
    Or(Vec<Condition>),
}

/// Evaluation context for condition processing
/// ExifTool: Access to `$$self`, `$$valPt`, `$count`, `$format` variables
#[derive(Debug)]
pub struct EvalContext<'a> {
    /// Data content: `$$valPt`
    /// ExifTool: Binary data for pattern matching
    pub data: &'a [u8],

    /// Entry count: `$count`
    /// ExifTool: Number of data elements or structure size
    pub count: u32,

    /// Data format: `$format`
    /// ExifTool: TIFF format type ("undef", "int16u", etc.)
    pub format: Option<&'a str>,

    /// Camera make: `$$self{Make}`
    /// ExifTool: Manufacturer string for detection
    pub make: Option<&'a str>,

    /// Camera model: `$$self{Model}`
    /// ExifTool: Model string for specific table selection
    pub model: Option<&'a str>,
}

/// Compiled regex cache for performance
/// ExifTool: Perl compiles regexes once, we cache them similarly
static REGEX_CACHE: Lazy<Mutex<HashMap<String, Regex>>> = Lazy::new(|| Mutex::new(HashMap::new()));

impl Condition {
    /// Evaluate condition against provided context
    /// ExifTool: Runtime condition evaluation in processor dispatch
    pub fn evaluate(&self, context: &EvalContext) -> bool {
        match self {
            Condition::DataPattern(pattern) => evaluate_data_pattern(pattern, context.data),
            Condition::ModelMatch(pattern) => context
                .model
                .map(|model| evaluate_string_pattern(pattern, model))
                .unwrap_or(false),
            Condition::MakeMatch(pattern) => context
                .make
                .map(|make| evaluate_string_pattern(pattern, make))
                .unwrap_or(false),
            Condition::CountEquals(expected) => context.count == *expected,
            Condition::CountRange(min, max) => context.count >= *min && context.count <= *max,
            Condition::FormatEquals(expected) => context
                .format
                .map(|format| format == expected)
                .unwrap_or(false),
            Condition::Not(inner) => !inner.evaluate(context),
            Condition::And(conditions) => conditions.iter().all(|c| c.evaluate(context)),
            Condition::Or(conditions) => conditions.iter().any(|c| c.evaluate(context)),
        }
    }

    /// Create a data pattern condition from a regex string
    /// ExifTool: `$$valPt =~ /pattern/` expressions
    pub fn data_pattern(pattern: &str) -> Result<Self> {
        // Validate regex compilation
        Regex::new(pattern).map_err(|e| {
            ExifError::ParseError(format!("Invalid regex pattern '{pattern}': {e}"))
        })?;
        Ok(Condition::DataPattern(pattern.to_string()))
    }

    /// Create a model matching condition from a regex string
    /// ExifTool: `$$self{Model} =~ /pattern/` expressions
    pub fn model_match(pattern: &str) -> Result<Self> {
        // Validate regex compilation
        Regex::new(pattern).map_err(|e| {
            ExifError::ParseError(format!("Invalid regex pattern '{pattern}': {e}"))
        })?;
        Ok(Condition::ModelMatch(pattern.to_string()))
    }

    /// Create a make matching condition from a regex string
    /// ExifTool: `$$self{Make} =~ /pattern/` expressions
    pub fn make_match(pattern: &str) -> Result<Self> {
        // Validate regex compilation
        Regex::new(pattern).map_err(|e| {
            ExifError::ParseError(format!("Invalid regex pattern '{pattern}': {e}"))
        })?;
        Ok(Condition::MakeMatch(pattern.to_string()))
    }
}

/// Evaluate data pattern against binary data
/// ExifTool: `$$valPt =~ /pattern/` matching
fn evaluate_data_pattern(pattern: &str, data: &[u8]) -> bool {
    let regex = get_cached_regex(pattern);

    // Convert binary data to string for pattern matching
    // ExifTool: Treats binary data as string for regex matching
    let data_str = if data.len() > 16 {
        // Only check first 16 bytes for performance (most patterns are prefixes)
        String::from_utf8_lossy(&data[..16])
    } else {
        String::from_utf8_lossy(data)
    };

    regex.is_match(&data_str)
}

/// Evaluate string pattern against text
/// ExifTool: Model/Make string matching
fn evaluate_string_pattern(pattern: &str, text: &str) -> bool {
    let regex = get_cached_regex(pattern);
    regex.is_match(text)
}

/// Get compiled regex from cache, compiling if necessary
/// ExifTool: Perl regex compilation optimization
fn get_cached_regex(pattern: &str) -> Regex {
    // Handle potential mutex poisoning gracefully
    let mut cache = match REGEX_CACHE.lock() {
        Ok(cache) => cache,
        Err(poisoned) => {
            // If the mutex is poisoned, clear it and continue
            // This can happen in tests when threads panic
            poisoned.into_inner()
        }
    };

    if let Some(regex) = cache.get(pattern) {
        return regex.clone();
    }

    // Compile and cache the regex
    match Regex::new(pattern) {
        Ok(regex) => {
            cache.insert(pattern.to_string(), regex.clone());
            regex
        }
        Err(_) => {
            // Fallback for invalid regex - create a regex that never matches any string
            // Use a simple pattern that will never match anything
            let fallback = Regex::new(r"$.^").unwrap(); // End followed by start - impossible
            cache.insert(pattern.to_string(), fallback.clone());
            fallback
        }
    }
}

/// Common condition constructors for ExifTool patterns
/// ExifTool: Frequently used condition patterns
impl Condition {
    /// Canon EOS 1D series detection
    /// ExifTool: `$$self{Model} =~ /\b1DS?$/`
    pub fn canon_1d_series() -> Self {
        Condition::ModelMatch(r"\b1DS?$".to_string())
    }

    /// Canon EOS 1D Mark II detection
    /// ExifTool: `$$self{Model} =~ /\b1Ds? Mark II$/`
    pub fn canon_1d_mark_ii() -> Self {
        Condition::ModelMatch(r"\b1Ds? Mark II$".to_string())
    }

    /// Canon EOS 1D Mark III detection
    /// ExifTool: `$$self{Model} =~ /\b1Ds? Mark III$/`
    pub fn canon_1d_mark_iii() -> Self {
        Condition::ModelMatch(r"\b1Ds? Mark III$".to_string())
    }

    /// Nikon LensData version 0204 detection
    /// ExifTool: `$$valPt =~ /^0204/`
    pub fn nikon_lens_data_0204() -> Self {
        Condition::DataPattern(r"^0204".to_string())
    }

    /// Nikon LensData version 0402 detection
    /// ExifTool: `$$valPt =~ /^0402/`
    pub fn nikon_lens_data_0402() -> Self {
        Condition::DataPattern(r"^0402".to_string())
    }

    /// Sony CameraInfo count variants
    /// ExifTool: `$count == 368 or $count == 5478`
    pub fn sony_camera_info_counts() -> Self {
        Condition::Or(vec![
            Condition::CountEquals(368),
            Condition::CountEquals(5478),
        ])
    }

    /// Canon manufacturer detection
    /// ExifTool: `$$self{Make} =~ /^Canon/`
    pub fn canon_make() -> Self {
        Condition::MakeMatch(r"^Canon".to_string())
    }

    /// Sony manufacturer detection with Hasselblad variants
    /// ExifTool: `$$self{Make}=~/^SONY/ or ($$self{Make}=~/^HASSELBLAD/ and $$self{Model}=~/^(HV|Stellar|Lusso|Lunar)/)`
    pub fn sony_or_hasselblad_variant() -> Self {
        Condition::Or(vec![
            Condition::MakeMatch(r"^SONY".to_string()),
            Condition::And(vec![
                Condition::MakeMatch(r"^HASSELBLAD".to_string()),
                Condition::ModelMatch(r"^(HV|Stellar|Lusso|Lunar)".to_string()),
            ]),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_pattern_evaluation() {
        let data = b"\x02\x04some data";
        let context = EvalContext {
            data,
            count: 10,
            format: None,
            make: None,
            model: None,
        };

        let condition = Condition::data_pattern(r"^0204").unwrap();
        assert!(!condition.evaluate(&context)); // Binary doesn't match text pattern directly

        // Test with text-like binary data
        let text_data = b"0204some data";
        let text_context = EvalContext {
            data: text_data,
            count: 10,
            format: None,
            make: None,
            model: None,
        };

        assert!(condition.evaluate(&text_context));
    }

    #[test]
    fn test_model_matching() {
        let context = EvalContext {
            data: &[],
            count: 0,
            format: None,
            make: Some("Canon"),
            model: Some("EOS 1D"),
        };

        let condition = Condition::canon_1d_series();
        assert!(condition.evaluate(&context));

        let context_mark_ii = EvalContext {
            model: Some("EOS 1Ds Mark II"),
            ..context
        };

        let condition_mark_ii = Condition::canon_1d_mark_ii();
        assert!(condition_mark_ii.evaluate(&context_mark_ii));
    }

    #[test]
    fn test_count_conditions() {
        let context = EvalContext {
            data: &[],
            count: 368,
            format: None,
            make: None,
            model: None,
        };

        let condition = Condition::CountEquals(368);
        assert!(condition.evaluate(&context));

        let range_condition = Condition::CountRange(300, 400);
        assert!(range_condition.evaluate(&context));

        let sony_condition = Condition::sony_camera_info_counts();
        assert!(sony_condition.evaluate(&context));
    }

    #[test]
    fn test_logical_operations() {
        let context = EvalContext {
            data: &[],
            count: 368,
            format: Some("undef"),
            make: Some("Canon"),
            model: Some("EOS 1D"),
        };

        let and_condition = Condition::And(vec![
            Condition::canon_make(),
            Condition::canon_1d_series(),
            Condition::CountEquals(368),
        ]);
        assert!(and_condition.evaluate(&context));

        let or_condition = Condition::Or(vec![
            Condition::CountEquals(999),  // false
            Condition::canon_1d_series(), // true
        ]);
        assert!(or_condition.evaluate(&context));

        let not_condition = Condition::Not(Box::new(Condition::CountEquals(999)));
        assert!(not_condition.evaluate(&context));
    }

    #[test]
    fn test_format_condition() {
        let context = EvalContext {
            data: &[],
            count: 0,
            format: Some("undef"),
            make: None,
            model: None,
        };

        let condition = Condition::FormatEquals("undef".to_string());
        assert!(condition.evaluate(&context));

        let condition_false = Condition::FormatEquals("int16u".to_string());
        assert!(!condition_false.evaluate(&context));
    }

    #[test]
    fn test_regex_caching() {
        // Multiple evaluations should use cached regex
        let pattern = r"^Canon";
        let context = EvalContext {
            data: &[],
            count: 0,
            format: None,
            make: Some("Canon EOS"),
            model: None,
        };

        let condition1 = Condition::MakeMatch(pattern.to_string());
        let condition2 = Condition::MakeMatch(pattern.to_string());

        assert!(condition1.evaluate(&context));
        assert!(condition2.evaluate(&context));

        // Verify cache contains the pattern
        let cache = REGEX_CACHE.lock().unwrap();
        assert!(cache.contains_key(pattern));
    }
}
