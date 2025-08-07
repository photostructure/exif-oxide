//! Type definitions for strategy system
//!
//! Common types used across different extraction strategies.

use std::collections::HashMap;

/// Tag information extracted from ExifTool tag tables
#[derive(Debug, Clone)]
pub struct TagInfo {
    /// Tag name (e.g., "GPSLatitudeRef")
    pub name: &'static str,
    
    /// Data format (e.g., "string", "int8u", "rational64u")
    pub format: &'static str,
    
    /// PrintConv conversion logic (if any)
    pub print_conv: Option<PrintConv>,
}

/// Binary data entry from ProcessBinaryData tables
#[derive(Debug, Clone)]
pub struct BinaryDataEntry {
    /// Entry name (e.g., "CroppedImageWidth")
    pub name: &'static str,
    
    /// Data format (e.g., "int32u", "int8u")
    pub format: &'static str,
    
    /// Number of values to read
    pub count: u64,
    
    /// Conditional logic for extraction (if any)
    pub condition: Option<String>,
}

/// Composite tag information with dependencies
#[derive(Debug, Clone)]
pub struct CompositeTagInfo {
    /// Composite tag name (e.g., "GPSPosition")
    pub name: &'static str,
    
    /// Group assignments (0 -> Family, 1 -> GPS, etc.)
    pub groups: HashMap<u8, &'static str>,
    
    /// Required source tags (must all be present)
    pub require: Vec<&'static str>,
    
    /// Desired source tags (optional but preferred)
    pub desire: Vec<&'static str>,
    
    /// Value conversion logic
    pub value_conv: Option<ValueConv>,
}

/// Print conversion type
#[derive(Debug, Clone)]
pub enum PrintConv {
    /// No conversion
    None,
    
    /// Simple lookup table
    Simple(HashMap<String, &'static str>),
    
    /// Expression to evaluate
    Expression(String),
    
    /// Manual function reference (module_path, function_name)
    Manual(&'static str, &'static str),
    
    /// Complex conversion requiring custom logic
    Complex,
}

/// Value conversion type  
#[derive(Debug, Clone)]
pub enum ValueConv {
    /// No conversion
    None,
    
    /// Simple numeric conversion
    Numeric(f64),
    
    /// Expression to evaluate
    Expression(String),
    
    /// Complex conversion requiring custom logic
    Complex,
}