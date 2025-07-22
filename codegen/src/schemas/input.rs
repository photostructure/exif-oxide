//! Input schemas for deserializing JSON from Perl extractors

use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

/// JSON structure from extract_tables.pl
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ExtractedData {
    pub extracted_at: String,
    pub exiftool_version: String,
    pub filter_criteria: String,
    pub stats: TagStats,
    pub tags: TagGroups,
    pub conversion_refs: ConversionRefs,
}

/// Tag statistics
#[derive(Debug, Deserialize)]
pub struct TagStats {
    pub exif_count: usize,
    pub gps_count: usize,
    pub total_tags: usize,
}

/// Tag groups (EXIF, GPS, etc)
#[derive(Debug, Deserialize)]
pub struct TagGroups {
    #[serde(default)]
    pub exif: Vec<ExtractedTag>,
    #[serde(default)]
    pub gps: Vec<ExtractedTag>,
}

/// Conversion references extracted from tag definitions
#[derive(Debug, Deserialize)]
pub struct ConversionRefs {
    pub print_conv: Vec<String>,
    pub value_conv: Vec<String>,
}

/// Individual extracted tag from ExifTool
#[derive(Debug, Deserialize)]
pub struct ExtractedTag {
    pub id: String,
    pub name: String,
    pub format: String,
    #[serde(default)]
    pub groups: Vec<String>,
    pub writable: u8,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub print_conv_ref: Option<String>,
    #[serde(default)]
    pub value_conv_ref: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub frequency: Option<f64>,
    #[serde(default)]
    pub mainstream: Option<u8>,
}

/// Composite tag definition
#[derive(Debug, Deserialize)]
pub struct ExtractedCompositeTag {
    pub name: String,
    pub table: String,
    pub full_name: String,
    #[serde(default)]
    pub require: Option<Vec<String>>,
    #[serde(default)]
    pub desire: Option<Vec<String>>,
    #[serde(default)]
    pub print_conv_ref: Option<String>,
    #[serde(default)]
    pub value_conv_ref: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub writable: u8,
    #[serde(default)]
    pub frequency: Option<f64>,
    #[serde(default)]
    pub mainstream: Option<u8>,
}

/// JSON structure from simple_table.pl
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct SimpleTablesData {
    pub extracted_at: String,
    pub extraction_config: String,
    pub total_tables: usize,
    pub tables: HashMap<String, ExtractedTable>,
}

/// Extracted table data from simple_table.pl
#[derive(Debug, Clone, Deserialize)]
pub struct ExtractedTable {
    pub source: TableSource,
    pub metadata: TableMetadata,
    pub entries: Vec<TableEntry>,
}

/// Table source information
#[derive(Debug, Clone, Deserialize)]
pub struct TableSource {
    pub module: String,
    pub hash_name: String,
    pub extracted_at: String,
}

/// Table metadata from extraction
#[derive(Debug, Clone, Deserialize)]
pub struct TableMetadata {
    pub description: String,
    pub constant_name: String,
    pub key_type: String,
    pub entry_count: usize,
}

/// Table configuration from module config files
#[derive(Debug, Deserialize)]
pub struct TableConfig {
    pub module: String,
    pub output_file: String,
    #[serde(default)]
    pub constant_name: Option<String>,
    #[serde(default)]
    pub key_type: Option<String>,
    #[serde(default)]
    pub extraction_type: Option<String>,
    #[serde(default)]
    pub value_type: Option<String>,
    pub description: String,
}

/// Composite tags JSON structure
#[derive(Debug, Deserialize)]
pub struct CompositeData {
    pub extracted_at: String,
    pub exiftool_version: String,
    pub filter_criteria: String,
    pub stats: CompositeStats,
    pub composite_tags: Vec<ExtractedCompositeTag>,
    pub conversion_refs: ConversionRefs,
}

/// Composite tag statistics
#[derive(Debug, Deserialize)]
pub struct CompositeStats {
    pub total_composite_tags: usize,
    pub exif_table: usize,
    pub gps_table: usize,
    pub main_table: usize,
}

/// Individual table entry (polymorphic for different extraction types)
#[derive(Debug, Deserialize, Clone)]
pub struct TableEntry {
    // Standard simple table fields
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub value: Option<Value>,
    
    // Regex pattern fields
    #[serde(default)]
    pub rust_compatible: Option<bool>,
    #[serde(default)]
    pub compatibility_notes: Option<String>,
    
    // File type lookup specific fields
    #[serde(default)]
    pub extension: Option<String>,
    #[serde(default)]
    pub entry_type: Option<String>,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub formats: Option<Vec<String>>,
    #[serde(default)]
    pub description: Option<String>,
}

/// JSON structure from runtime_table.pl
#[derive(Debug, Deserialize)]
pub struct RuntimeTablesData {
    pub source: TableSource,
    pub extracted_at: String,
    pub extraction_config: String,
    pub tables: HashMap<String, ExtractedRuntimeTable>,
}

/// Extracted runtime table data from runtime_table.pl
#[derive(Debug, Clone, Deserialize)]
pub struct ExtractedRuntimeTable {
    pub metadata: RuntimeTableMetadata,
    pub table_structure: ProcessBinaryDataStructure,
    pub tag_definitions: HashMap<String, RuntimeTagDefinition>,
}

/// Runtime table metadata
#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeTableMetadata {
    pub function_name: String,
    pub table_name: String,
    pub processing_mode: String,
    pub format_handling: String,
    pub has_model_conditions: bool,
    pub has_data_member_deps: bool,
    pub has_complex_printconv: bool,
    pub description: String,
}

/// ProcessBinaryData table structure
#[derive(Debug, Clone, Deserialize)]
pub struct ProcessBinaryDataStructure {
    pub format: Option<String>,
    pub first_entry: Option<u32>,
    pub groups: Option<Value>,
    pub data_member: Option<Vec<u32>>,
    pub writable: Option<bool>,
}

/// Runtime tag definition with conditional logic
#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeTagDefinition {
    pub name: String,
    pub offset: String,  // Can be numeric or fractional like "1.5"
    pub format: Option<FormatSpec>,
    pub condition: Option<ConditionSpec>,
    pub print_conv: Option<PrintConvSpec>,
    pub value_conv: Option<ValueConvSpec>,
    pub groups: Option<Value>,
    pub notes: Option<String>,
}

/// Format specification for binary data
#[derive(Debug, Clone, Deserialize)]
pub struct FormatSpec {
    pub base_type: String,  // int16u, int8s, string, etc.
    pub array_size: Option<String>,  // "$val{0}", "int(($val{0}+15)/16)"
    pub is_variable: bool,
}

/// Condition specification for model-dependent tags
#[derive(Debug, Clone, Deserialize)]
pub struct ConditionSpec {
    pub expression: String,  // "$$self{Model} =~ /EOS/"
    pub condition_type: ConditionType,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionType {
    ModelRegex,
    ModelExact, 
    ValueComparison,
    Expression,
}

/// PrintConv specification for value formatting
#[derive(Debug, Clone, Deserialize)]
pub struct PrintConvSpec {
    pub conversion_type: PrintConvType,
    pub data: Value,  // Can be hash table, expression string, or function reference
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrintConvType {
    SimpleHash,        // { 0 => "Off", 1 => "On" }
    PerlExpression,    // q{ return ... }
    FunctionRef,       // \&SomeFunction
    BitwiseOperation,  // Complex bitwise formatting
}

/// ValueConv specification for value conversion
#[derive(Debug, Clone, Deserialize)]
pub struct ValueConvSpec {
    pub conversion_type: ValueConvType,
    pub expression: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueConvType {
    Mathematical,  // exp($val/32*log(2))*100
    Division,      // $val / ($$self{FocalUnits} || 1)
    FunctionCall,  // Image::ExifTool::Canon::CameraISO($val)
    Conditional,   // $val == 0x7fff ? undef : $val
}