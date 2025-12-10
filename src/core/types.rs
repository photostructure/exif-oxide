//! Core type definitions for runtime support
//!
//! This module provides the essential types that generated Rust code depends on.
//! These are simplified versions of the types from the main crate, designed to
//! minimize dependencies while providing the necessary functionality.

use std::collections::HashMap;

/// Expression evaluation context for generated functions
///
/// This provides access to ExifTool's DataMembers and state during
/// expression evaluation.
#[derive(Debug, Clone)]
pub struct ExifContext {
    /// DataMember variables from ExifTool  
    pub data_members: HashMap<String, crate::TagValue>,

    /// Processing state variables
    pub state: HashMap<String, String>,

    /// Current directory path stack
    pub path: Vec<String>,

    /// Base offset for pointer calculations
    pub base_offset: u64,
}

impl ExifContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self {
            data_members: HashMap::new(),
            state: HashMap::new(),
            path: Vec::new(),
            base_offset: 0,
        }
    }

    /// Set a data member value
    pub fn set_data_member(&mut self, name: &str, value: crate::TagValue) {
        self.data_members.insert(name.to_string(), value);
    }

    /// Get a data member value
    pub fn get_data_member(&self, name: &str) -> Option<&crate::TagValue> {
        self.data_members.get(name)
    }

    /// Set a state variable
    pub fn set_state(&mut self, name: &str, value: &str) {
        self.state.insert(name.to_string(), value.to_string());
    }

    /// Get a state variable
    pub fn get_state(&self, name: &str) -> Option<&str> {
        self.state.get(name).map(|s| s.as_str())
    }
}

impl Default for ExifContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Error types for runtime operations
#[derive(thiserror::Error, Debug)]
pub enum ExifError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid file format: {0}")]
    InvalidFormat(String),

    #[error("Format error: {0}")]
    FormatError(String),

    #[error("Unsupported feature: {0}")]
    Unsupported(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("File detection error: {0}")]
    FileDetection(String),
}

impl ExifError {
    /// Create a new parse error
    pub fn new(msg: &str) -> Self {
        ExifError::ParseError(msg.to_string())
    }
}

// Note: From<std::io::Error> is automatically derived via #[from] on ExifError::Io

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, ExifError>;

// =============================================================================
// Composite Tag Function Signatures
// =============================================================================
//
// These function types are used by composite tags that compute values from
// multiple source tags. Unlike regular ValueConv/PrintConv which operate on
// a single value, composite functions receive arrays of resolved dependencies:
//
// - vals: The converted values (after ValueConv) from required/desired tags
// - prts: The printed values (after PrintConv) from required/desired tags
// - raws: The raw unconverted values from required/desired tags
//
// See docs/todo/P03c-composite-tags.md for the full design.

/// Function signature for composite tag ValueConv expressions
///
/// Parameters:
/// - `vals`: Slice of TagValues from resolved dependencies (converted values)
/// - `prts`: Slice of TagValues from resolved dependencies (printed values)
/// - `raws`: Slice of TagValues from resolved dependencies (raw unconverted values)
/// - `ctx`: Optional ExifContext for $$self{...} access
///
/// Example composite expressions:
/// - `$val[0] || $val[1]` (Aperture - uses first available)
/// - `$val[0] / $val[1]` (ratio calculations)
/// - `$prt[0], $prt[1]` (GPS position formatting)
pub type CompositeValueConvFn = fn(
    &[crate::TagValue],
    &[crate::TagValue],
    &[crate::TagValue],
    Option<&ExifContext>,
) -> Result<crate::TagValue>;

/// Function signature for composite tag PrintConv expressions
///
/// Parameters are the same as CompositeValueConvFn, but this is applied
/// after the ValueConv to produce human-readable output.
///
/// Example composite PrintConv expressions:
/// - `sprintf("%.1f", $val[0])` (format single dependency)
/// - `$prt[0]` (pass through printed value)
/// - `"$prt[0], $prt[1]"` (combine printed values)
pub type CompositePrintConvFn = fn(
    &[crate::TagValue],
    &[crate::TagValue],
    &[crate::TagValue],
    Option<&ExifContext>,
) -> Result<crate::TagValue>;
