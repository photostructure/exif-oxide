//! Binary data processing types for ProcessBinaryData
//!
//! This module defines the types used for ExifTool's ProcessBinaryData
//! functionality, including format definitions and table structures.

use crate::expressions::ExpressionEvaluator as UnifiedExpressionEvaluator;
use crate::processor_registry::ProcessorContext;
use crate::types::{DataMemberValue, ExifError, TagValue};
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

// Static regexes for format expression parsing
// ExifTool: lib/Image/ExifTool.pm:9850-9856 format parsing patterns
static ARRAY_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(.+)\[(.+)\]$").unwrap());
static VAL_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\$val\{([^}]+)\}").unwrap());

/// Binary data formats for ProcessBinaryData
/// ExifTool: lib/Image/ExifTool.pm %formatSize and @formatName arrays
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryDataFormat {
    /// Fixed formats - these don't require expression evaluation
    /// Unsigned 8-bit integer
    /// ExifTool: int8u
    Int8u,
    /// Signed 8-bit integer
    /// ExifTool: int8s
    Int8s,
    /// Unsigned 16-bit integer
    /// ExifTool: int16u
    Int16u,
    /// Signed 16-bit integer
    /// ExifTool: int16s
    Int16s,
    /// Unsigned 32-bit integer
    /// ExifTool: int32u
    Int32u,
    /// Signed 32-bit integer
    /// ExifTool: int32s
    Int32s,
    /// 32-bit floating point
    /// ExifTool: float
    Float,
    /// 64-bit floating point
    /// ExifTool: double
    Double,
    /// Null-terminated string
    /// ExifTool: string
    String,
    /// Pascal string (first byte is length)
    /// ExifTool: pstring
    PString,
    /// Binary/undefined data
    /// ExifTool: undef
    Undef,
    /// Variable string with null termination
    /// ExifTool: var_string
    VarString,
}

/// Format specification that may contain expressions
/// ExifTool: Format with expressions like "int16s[$val{0}]"
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatSpec {
    /// Fixed format without expressions
    Fixed(BinaryDataFormat),
    /// Variable format with expression for count
    /// ExifTool: "int16s[$val{0}]" -> Array { base: Int16s, count_expr: "$val{0}" }
    Array {
        base_format: BinaryDataFormat,
        count_expr: String,
    },
    /// Variable-length string with expression for length
    /// ExifTool: "string[$val{3}]" -> StringWithLength { length_expr: "$val{3}" }
    StringWithLength { length_expr: String },
}

/// Resolved format after expression evaluation
/// ExifTool: Formats after $val{} expressions have been evaluated
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedFormat {
    /// Single value format
    Single(BinaryDataFormat),
    /// Array of values with known count
    Array(BinaryDataFormat, usize),
    /// String with known length
    StringWithLength(usize),
    /// Variable null-terminated string
    VarString,
}

impl BinaryDataFormat {
    /// Get byte size for this format
    /// ExifTool: lib/Image/ExifTool.pm %formatSize array
    pub fn byte_size(self) -> usize {
        match self {
            BinaryDataFormat::Int8u | BinaryDataFormat::Int8s | BinaryDataFormat::Undef => 1,
            BinaryDataFormat::Int16u | BinaryDataFormat::Int16s => 2,
            BinaryDataFormat::Int32u | BinaryDataFormat::Int32s | BinaryDataFormat::Float => 4,
            BinaryDataFormat::Double => 8,
            BinaryDataFormat::String | BinaryDataFormat::PString | BinaryDataFormat::VarString => 1, // Variable length
        }
    }

    /// Parse format string to enum (for fixed formats only)
    /// ExifTool: lib/Image/ExifTool.pm format name lookup
    pub fn parse_format(format: &str) -> std::result::Result<Self, ExifError> {
        match format {
            "int8u" => Ok(BinaryDataFormat::Int8u),
            "int8s" => Ok(BinaryDataFormat::Int8s),
            "int16u" => Ok(BinaryDataFormat::Int16u),
            "int16s" => Ok(BinaryDataFormat::Int16s),
            "int32u" => Ok(BinaryDataFormat::Int32u),
            "int32s" => Ok(BinaryDataFormat::Int32s),
            "float" => Ok(BinaryDataFormat::Float),
            "double" => Ok(BinaryDataFormat::Double),
            "string" => Ok(BinaryDataFormat::String),
            "pstring" => Ok(BinaryDataFormat::PString),
            "undef" => Ok(BinaryDataFormat::Undef),
            "var_string" => Ok(BinaryDataFormat::VarString),
            _ => Err(ExifError::ParseError(format!(
                "Unknown binary data format: {format}"
            ))),
        }
    }
}

impl FormatSpec {
    /// Parse format specification from string
    /// ExifTool: lib/Image/ExifTool.pm:9850-9856 format parsing with expressions
    pub fn parse(format_str: &str) -> std::result::Result<Self, ExifError> {
        // Check for array format with expression: "format[$expression]"
        if let Some(captures) = ARRAY_REGEX.captures(format_str) {
            let base_format_str = captures.get(1).unwrap().as_str();
            let count_expr = captures.get(2).unwrap().as_str().to_string();

            if base_format_str == "string" {
                // Special case: string[$expr] -> StringWithLength
                Ok(FormatSpec::StringWithLength {
                    length_expr: count_expr,
                })
            } else {
                // Regular array format: int16s[$expr] -> Array
                let base_format = BinaryDataFormat::parse_format(base_format_str)?;
                Ok(FormatSpec::Array {
                    base_format,
                    count_expr,
                })
            }
        } else {
            // Fixed format without expressions
            let format = BinaryDataFormat::parse_format(format_str)?;
            Ok(FormatSpec::Fixed(format))
        }
    }

    /// Check if this format spec contains expressions that need evaluation
    pub fn needs_evaluation(&self) -> bool {
        match self {
            FormatSpec::Fixed(_) => false,
            FormatSpec::Array { .. } => true,
            FormatSpec::StringWithLength { .. } => true,
        }
    }
}

impl ResolvedFormat {
    /// Get the byte size for extracting this format
    pub fn byte_size(&self) -> usize {
        match self {
            ResolvedFormat::Single(format) => format.byte_size(),
            ResolvedFormat::Array(format, count) => format.byte_size() * count,
            ResolvedFormat::StringWithLength(length) => *length,
            ResolvedFormat::VarString => 1, // Variable length, will scan for null
        }
    }

    /// Get the base format for value extraction
    pub fn base_format(&self) -> BinaryDataFormat {
        match self {
            ResolvedFormat::Single(format) => *format,
            ResolvedFormat::Array(format, _) => *format,
            ResolvedFormat::StringWithLength(_) => BinaryDataFormat::String,
            ResolvedFormat::VarString => BinaryDataFormat::VarString,
        }
    }

    /// Get the count for array formats (1 for single values)
    pub fn count(&self) -> usize {
        match self {
            ResolvedFormat::Single(_) => 1,
            ResolvedFormat::Array(_, count) => *count,
            ResolvedFormat::StringWithLength(_) => 1,
            ResolvedFormat::VarString => 1,
        }
    }
}

/// Expression evaluator for format specifications
/// ExifTool: lib/Image/ExifTool.pm:9853-9856 eval mechanism for $val{} expressions
pub struct ExpressionEvaluator<'a> {
    /// Current $val hash - values from current binary data block
    /// ExifTool: %val hash populated during ProcessBinaryData
    val_hash: HashMap<u32, DataMemberValue>,
    /// Global DataMember values from $$self{}
    /// ExifTool: $$self{DataMember} values
    data_members: &'a HashMap<String, DataMemberValue>,
}

impl<'a> ExpressionEvaluator<'a> {
    /// Create new expression evaluator
    pub fn new(
        val_hash: HashMap<u32, DataMemberValue>,
        data_members: &'a HashMap<String, DataMemberValue>,
    ) -> Self {
        Self {
            val_hash,
            data_members,
        }
    }

    /// Evaluate expression using unified expression system
    /// Attempts to use the unified ExpressionEvaluator before falling back to specialized logic
    fn evaluate_with_unified_system(&self, expr: &str) -> std::result::Result<usize, ExifError> {
        let mut evaluator = UnifiedExpressionEvaluator::new();
        let mut processor_context = ProcessorContext::default();

        // Add val_hash values to processor context
        for (index, value) in &self.val_hash {
            // Convert DataMemberValue to TagValue
            if let Ok(tag_value) = self.data_member_to_tag_value(value) {
                processor_context
                    .parent_tags
                    .insert(format!("val{{{index}}}"), tag_value);
            }
        }

        // Add data_members to processor context
        for (name, value) in self.data_members.iter() {
            if let Ok(tag_value) = self.data_member_to_tag_value(value) {
                processor_context
                    .parent_tags
                    .insert(name.clone(), tag_value);
            }
        }

        // Try to evaluate the expression
        match evaluator.evaluate_context_condition(&processor_context, expr) {
            Ok(true) => Ok(1),  // Boolean true -> count of 1
            Ok(false) => Ok(0), // Boolean false -> count of 0
            Err(_) => {
                // Try as a numeric expression (this would need extension to ExpressionEvaluator)
                Err(ExifError::ParseError(format!(
                    "Unified expression evaluation failed for: {expr}"
                )))
            }
        }
    }

    /// Convert DataMemberValue to TagValue for unified expression system
    fn data_member_to_tag_value(&self, value: &DataMemberValue) -> Result<TagValue, ExifError> {
        match value {
            DataMemberValue::U8(v) => Ok(TagValue::U8(*v)),
            DataMemberValue::U16(v) => Ok(TagValue::U16(*v)),
            DataMemberValue::U32(v) => Ok(TagValue::U32(*v)),
            DataMemberValue::String(v) => Ok(TagValue::String(v.clone())),
        }
    }

    /// Evaluate a format expression to get a count value using unified expression system
    /// ExifTool: lib/Image/ExifTool.pm:9853-9856 eval $count mechanism
    pub fn evaluate_count_expression(&self, expr: &str) -> std::result::Result<usize, ExifError> {
        // Try unified expression system first
        if let Ok(result) = self.evaluate_with_unified_system(expr) {
            return Ok(result);
        }

        // Handle complex expressions first (before simple $val patterns)
        // ExifTool: Canon.pm:4480 'Format => int16s[int(($val{0}+15)/16)]'
        if let Ok(result) = self.evaluate_complex_expression(expr) {
            return Ok(result);
        }

        // Handle simple $val{N} references (only if complex expression failed)
        if let Some(captures) = VAL_REGEX.captures(expr) {
            let val_ref = captures.get(1).unwrap().as_str();

            // Try parsing as index first
            if let Ok(index) = val_ref.parse::<u32>() {
                if let Some(value) = self.val_hash.get(&index) {
                    return value.as_usize().ok_or_else(|| {
                        ExifError::ParseError(format!(
                            "Value at index {index} cannot be converted to count"
                        ))
                    });
                }
            }

            // Try as DataMember name
            if let Some(value) = self.data_members.get(val_ref) {
                return value.as_usize().ok_or_else(|| {
                    ExifError::ParseError(format!(
                        "DataMember '{val_ref}' cannot be converted to count"
                    ))
                });
            }

            return Err(ExifError::ParseError(format!(
                "Unknown value reference: {val_ref}"
            )));
        }

        // If no patterns match, return error
        Err(ExifError::ParseError(format!(
            "Cannot evaluate expression: {expr}"
        )))
    }

    /// Resolve a format specification by evaluating any expressions
    pub fn resolve_format(
        &self,
        spec: &FormatSpec,
    ) -> std::result::Result<ResolvedFormat, ExifError> {
        match spec {
            FormatSpec::Fixed(format) => Ok(ResolvedFormat::Single(*format)),
            FormatSpec::Array {
                base_format,
                count_expr,
            } => {
                let count = self.evaluate_count_expression(count_expr)?;
                Ok(ResolvedFormat::Array(*base_format, count))
            }
            FormatSpec::StringWithLength { length_expr } => {
                let length = self.evaluate_count_expression(length_expr)?;
                Ok(ResolvedFormat::StringWithLength(length))
            }
        }
    }

    /// Evaluate complex mathematical expressions
    /// ExifTool: Complex expression evaluation like "int(($val{0}+15)/16)"
    /// Reference: third-party/exiftool/lib/Image/ExifTool/Canon.pm:4480
    pub fn evaluate_complex_expression(&self, expr: &str) -> std::result::Result<usize, ExifError> {
        // Pattern for int(($val{N}+CONST)/DIVISOR) - ceiling division for bit arrays
        // ExifTool: Canon.pm uses this pattern for AFPointsInFocus bit array sizing
        static CEILING_DIV_REGEX: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"^int\(\(\$val\{(\d+)\}\+(\d+)\)/(\d+)\)$").unwrap());

        if let Some(captures) = CEILING_DIV_REGEX.captures(expr) {
            let val_index: u32 = captures.get(1).unwrap().as_str().parse().map_err(|_| {
                ExifError::ParseError("Invalid value index in ceiling division".to_string())
            })?;
            let addend: usize = captures.get(2).unwrap().as_str().parse().map_err(|_| {
                ExifError::ParseError("Invalid addend in ceiling division".to_string())
            })?;
            let divisor: usize = captures.get(3).unwrap().as_str().parse().map_err(|_| {
                ExifError::ParseError("Invalid divisor in ceiling division".to_string())
            })?;

            if divisor == 0 {
                return Err(ExifError::ParseError(
                    "Division by zero in expression".to_string(),
                ));
            }

            // Get the value from $val hash
            let val = self.val_hash.get(&val_index).ok_or_else(|| {
                ExifError::ParseError(format!("Value at index {val_index} not found"))
            })?;

            let val_usize = val.as_usize().ok_or_else(|| {
                ExifError::ParseError(format!(
                    "Value at index {val_index} cannot be converted to number"
                ))
            })?;

            // Calculate ceiling division: int((val + addend) / divisor)
            // This is equivalent to: (val + addend + divisor - 1) / divisor
            let result = (val_usize + addend) / divisor;
            return Ok(result);
        }

        // If no patterns match, return error
        Err(ExifError::ParseError(format!(
            "Unsupported complex expression: {expr}"
        )))
    }

    /// Update the $val hash with a new value
    pub fn set_val(&mut self, index: u32, value: DataMemberValue) {
        self.val_hash.insert(index, value);
    }
}

/// Binary data table configuration
/// ExifTool: Tag table with PROCESS_PROC => \&ProcessBinaryData
#[derive(Debug, Clone)]
pub struct BinaryDataTable {
    /// Default format for entries (ExifTool: FORMAT key)
    pub default_format: BinaryDataFormat,
    /// Starting index for unknown tag generation (ExifTool: FIRST_ENTRY key)
    pub first_entry: Option<u32>,
    /// Group hierarchy for tags (ExifTool: GROUPS key)
    pub groups: HashMap<u8, String>,
    /// Tag definitions indexed by position
    pub tags: HashMap<u32, BinaryDataTag>,
    /// Tags that are DataMembers and must be extracted first
    /// ExifTool: DATAMEMBER => [...] array of indices
    pub data_member_tags: Vec<u32>,
    /// Processing order for tags with dependencies
    /// Phase 1: DataMember tags, Phase 2: dependent tags
    pub dependency_order: Vec<u32>,
}

/// Individual tag definition in binary data table
/// ExifTool: Tag info hash structure
#[derive(Debug, Clone)]
pub struct BinaryDataTag {
    /// Tag name
    pub name: String,
    /// Data format specification (may contain expressions)
    /// ExifTool: Format field can be "int16s" or "int16s[$val{0}]"
    pub format_spec: Option<FormatSpec>,
    /// Legacy format field for backward compatibility
    /// TODO: Remove once all code uses format_spec
    pub format: Option<BinaryDataFormat>,
    /// Bit mask for extracting value
    pub mask: Option<u32>,
    /// PrintConv lookup table
    pub print_conv: Option<HashMap<u32, String>>,
    /// DataMember name if this tag should be stored for later use
    /// ExifTool: DataMember => 'Name' in tag definition
    pub data_member: Option<String>,
}

impl Default for BinaryDataTable {
    fn default() -> Self {
        Self {
            default_format: BinaryDataFormat::Int8u,
            first_entry: None,
            groups: HashMap::new(),
            tags: HashMap::new(),
            data_member_tags: Vec::new(),
            dependency_order: Vec::new(),
        }
    }
}

impl BinaryDataTable {
    /// Analyze table to identify DataMember dependencies and set processing order
    /// ExifTool: Implicit dependency analysis during ProcessBinaryData
    pub fn analyze_dependencies(&mut self) {
        // Clear existing analysis
        self.data_member_tags.clear();
        self.dependency_order.clear();

        // Phase 1: Identify DataMember tags
        for (&index, tag) in &self.tags {
            if tag.data_member.is_some() {
                self.data_member_tags.push(index);
            }
        }

        // Sort DataMember tags by index for consistent processing
        self.data_member_tags.sort();

        // Phase 2: Add DataMember tags to processing order first
        self.dependency_order.extend(&self.data_member_tags);

        // Phase 3: Add remaining tags that depend on DataMembers
        let mut remaining_tags: Vec<u32> = self
            .tags
            .keys()
            .filter(|&index| !self.data_member_tags.contains(index))
            .copied()
            .collect();

        // Sort remaining tags by index
        remaining_tags.sort();
        self.dependency_order.extend(remaining_tags);
    }

    /// Check if a tag needs expression evaluation
    pub fn tag_needs_evaluation(&self, index: u32) -> bool {
        if let Some(tag) = self.tags.get(&index) {
            if let Some(format_spec) = &tag.format_spec {
                return format_spec.needs_evaluation();
            }
        }
        false
    }

    /// Get effective format for a tag (format_spec takes precedence)
    pub fn get_tag_format_spec(&self, index: u32) -> Option<FormatSpec> {
        if let Some(tag) = self.tags.get(&index) {
            if let Some(format_spec) = &tag.format_spec {
                return Some(format_spec.clone());
            }
            if let Some(format) = &tag.format {
                return Some(FormatSpec::Fixed(*format));
            }
            // Use table default
            return Some(FormatSpec::Fixed(self.default_format));
        }
        None
    }
}
