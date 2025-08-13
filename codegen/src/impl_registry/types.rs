//! Types for the implementation registry system
//!
//! This module defines the core types used throughout the implementation registry,
//! primarily for PrintConv/ValueConv classification. Function registry types are
//! defined in the function_registry module.

// PPI AST generates functions directly at build time - no runtime expression compilation
// Each type has different signatures depending on whether they need context:
// - ValueConv: fn(&TagValue) -> Result<TagValue> OR fn(&TagValue, &ExifContext) -> Result<TagValue>
// - PrintConv: fn(&TagValue) -> Result<String>
// - Condition: fn(&TagValue, &ExifContext) -> bool
// - Composite ValueConv: fn(&[TagValue], &ExifContext) -> Result<TagValue> (uses resolved dependencies)

/// Classification of ValueConv expressions for code generation
#[derive(Debug, Clone)]
pub enum ValueConvType {
    /// PPI-generated ValueConv without context: fn(&TagValue) -> Result<TagValue>
    PpiGeneratedSimple(String), // e.g., "$val / 100"
    /// PPI-generated ValueConv with context: fn(&TagValue, &ExifContext) -> Result<TagValue>
    PpiGeneratedWithContext(String), // e.g., "$val / ($$self{FocalUnits} || 1)"
    /// PPI-generated Composite ValueConv: fn(&[TagValue], &ExifContext) -> Result<TagValue>
    PpiGeneratedComposite(String), // e.g., "$val[0] + $val[1]" for composite tags
    /// Complex expression requiring a custom function
    CustomFunction(&'static str, &'static str), // (module_path, function_name)
}

/// Classification of PrintConv expressions for code generation
#[derive(Debug, Clone)]
pub enum PrintConvType {
    /// PPI-generated PrintConv function: fn(&TagValue) -> Result<String>
    PpiGeneratedPrintConv(String), // Generated Rust code from PPI AST
    /// Complex expression requiring a custom function  
    CustomFunction(&'static str, &'static str), // (module_path, function_name)
}

/// Classification of Condition expressions for code generation
#[derive(Debug, Clone)]
pub enum ConditionType {
    /// PPI-generated Condition function: fn(&TagValue, &ExifContext) -> bool
    PpiGeneratedCondition(String), // Generated Rust code from PPI AST
    /// Complex expression requiring a custom function
    CustomFunction(&'static str, &'static str), // (module_path, function_name)
}
