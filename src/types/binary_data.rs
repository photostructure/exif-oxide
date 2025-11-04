//! Binary data processing types for ProcessBinaryData
//!
//! This module defines the types used for ExifTool's ProcessBinaryData
//! functionality, including format definitions and table structures.

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
    /// TODO: Implement when unified expression system is complete
    fn evaluate_with_unified_system(&self, expr: &str) -> std::result::Result<usize, ExifError> {
        // Unified expression system not yet implemented
        // This will cause the caller to fall back to specialized logic
        Err(ExifError::NotImplemented(format!(
            "Unified expression evaluation not yet implemented for: {expr}"
        )))
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
    /// Tag name (for simple tags) or name of first variant (for conditional arrays)
    pub name: String,
    /// Conditional variants for this tag
    /// ExifTool: Array of tag definitions with different conditions
    /// For simple tags, this contains a single variant with no condition
    pub variants: Vec<BinaryDataTagVariant>,
    /// Legacy fields for backward compatibility - TODO: Remove once all code migrated
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
    /// Group assignment for this tag (e.g., 0 for MakerNotes, 2 for Camera)
    /// ExifTool: Individual tags can override the table's default group
    pub group: Option<u8>,
}

/// Individual variant of a binary data tag with optional condition
/// ExifTool: Single tag definition that may be part of a conditional array
#[derive(Debug, Clone)]
pub struct BinaryDataTagVariant {
    /// Tag name for this variant
    pub name: String,
    /// Condition for when this variant applies
    /// ExifTool: Condition => '$self{Model} =~ /\b(20D|350D)\b/' or None for default
    pub condition: Option<String>,
    /// Data format specification (may contain expressions)
    /// ExifTool: Format field can be "int16s" or "int16s[$val{0}]"
    pub format_spec: Option<FormatSpec>,
    /// Legacy format field for backward compatibility
    pub format: Option<BinaryDataFormat>,
    /// Bit mask for extracting value
    pub mask: Option<u32>,
    /// PrintConv lookup table
    pub print_conv: Option<HashMap<u32, String>>,
    /// ValueConv expression for value conversion
    /// ExifTool: ValueConv => 'exp($val/32*log(2))*100'
    pub value_conv: Option<String>,
    /// PrintConv expression for display conversion
    /// ExifTool: PrintConv => 'sprintf("%.0f",$val)'
    pub print_conv_expr: Option<String>,
    /// DataMember name if this tag should be stored for later use
    /// ExifTool: DataMember => 'Name' in tag definition
    pub data_member: Option<String>,
    /// Group assignment for this variant
    pub group: Option<u8>,
    /// Priority for this variant (lower = higher priority)
    /// ExifTool: Priority => 0
    pub priority: Option<i32>,
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
            // Check variants first (new system)
            if !tag.variants.is_empty() {
                return tag.variants.iter().any(|variant| {
                    variant
                        .format_spec
                        .as_ref()
                        .is_some_and(|spec| spec.needs_evaluation())
                });
            }
            // Legacy system fallback
            if let Some(format_spec) = &tag.format_spec {
                return format_spec.needs_evaluation();
            }
        }
        false
    }

    /// Get effective format for a tag (format_spec takes precedence)
    /// For conditional tags, returns the format of the first variant
    pub fn get_tag_format_spec(&self, index: u32) -> Option<FormatSpec> {
        if let Some(tag) = self.tags.get(&index) {
            // Check variants first (new system)
            if !tag.variants.is_empty() {
                if let Some(first_variant) = tag.variants.first() {
                    if let Some(format_spec) = &first_variant.format_spec {
                        return Some(format_spec.clone());
                    }
                    if let Some(format) = &first_variant.format {
                        return Some(FormatSpec::Fixed(*format));
                    }
                }
                // Use table default for variants
                return Some(FormatSpec::Fixed(self.default_format));
            }
            // Legacy system fallback
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

impl BinaryDataTag {
    /// Get the active variant for this tag based on processor context
    /// ExifTool: Evaluates Condition fields to select appropriate variant
    pub fn get_active_variant(&self, context: &ProcessorContext) -> Option<&BinaryDataTagVariant> {
        // If we have variants, evaluate them
        if !self.variants.is_empty() {
            for variant in &self.variants {
                if let Some(condition) = &variant.condition {
                    // Evaluate condition against context (camera model, etc.)
                    if evaluate_condition(condition, context) {
                        return Some(variant);
                    }
                } else {
                    // No condition = default variant, use if no conditioned variants match
                    return Some(variant);
                }
            }
            // If no variant matches, return the first one as fallback
            return self.variants.first();
        }

        // Legacy: create a variant from the old structure
        None
    }

    /// Create a simple tag with a single variant (for backward compatibility)
    pub fn simple(name: String) -> Self {
        Self {
            name: name.clone(),
            variants: vec![BinaryDataTagVariant {
                name,
                condition: None,
                format_spec: None,
                format: None,
                mask: None,
                print_conv: None,
                value_conv: None,
                print_conv_expr: None,
                data_member: None,
                group: None,
                priority: None,
            }],
            // Legacy fields
            format_spec: None,
            format: None,
            mask: None,
            print_conv: None,
            data_member: None,
            group: None,
        }
    }

    /// Create a tag from legacy fields (for backward compatibility)
    /// This automatically creates a single variant from the legacy structure
    pub fn from_legacy(
        name: String,
        format_spec: Option<FormatSpec>,
        format: Option<BinaryDataFormat>,
        mask: Option<u32>,
        print_conv: Option<HashMap<u32, String>>,
        data_member: Option<String>,
        group: Option<u8>,
    ) -> Self {
        Self {
            name: name.clone(),
            variants: vec![BinaryDataTagVariant {
                name,
                condition: None,
                format_spec: format_spec.clone(),
                format,
                mask,
                print_conv: print_conv.clone(),
                value_conv: None,
                print_conv_expr: None,
                data_member: data_member.clone(),
                group,
                priority: None,
            }],
            // Keep legacy fields for backward compatibility
            format_spec,
            format,
            mask,
            print_conv, // Keep for backward compatibility
            data_member,
            group,
        }
    }
}

/// Evaluate a condition string against processor context
/// ExifTool: Condition evaluation like '$self{Model} =~ /\b(20D|350D)\b/'
/// TODO: Implement when unified expression system is complete
fn evaluate_condition(condition: &str, context: &ProcessorContext) -> bool {
    // Unified expression system not yet implemented
    // For now, return false and log the condition
    tracing::debug!(
        "Condition evaluation not yet implemented: {}. Returning false.",
        condition
    );
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processor_registry::ProcessorContext;

    #[test]
    fn test_conditional_array_exposure_time() {
        // Test conditional array support for Canon ShotInfo ExposureTime
        // Reference: Canon.pm tag 22 with condition for 20D/350D vs other models
        // This validates the ExposureTime behavior mentioned in the transcript

        // Create a BinaryDataTag with conditional variants for ExposureTime
        let exposure_time_tag = BinaryDataTag {
            name: "ExposureTime".to_string(),
            variants: vec![
                // Variant 1: For 20D/350D models (includes *1000/32)
                BinaryDataTagVariant {
                    name: "ExposureTime".to_string(),
                    condition: Some("$model =~ /(20D|350D|REBEL XT|Kiss Digital N)/".to_string()),
                    format_spec: None,
                    format: None,
                    mask: None,
                    print_conv: None,
                    value_conv: Some(
                        "exp(-Image::ExifTool::Canon::CanonEv($val)*log(2))*1000/32".to_string(),
                    ),
                    print_conv_expr: Some(
                        "Image::ExifTool::Exif::PrintExposureTime($val)".to_string(),
                    ),
                    data_member: None,
                    group: None,
                    priority: Some(0),
                },
                // Variant 2: For other models (no *1000/32)
                BinaryDataTagVariant {
                    name: "ExposureTime".to_string(),
                    condition: None, // Default fallback
                    format_spec: None,
                    format: None,
                    mask: None,
                    print_conv: None,
                    value_conv: Some(
                        "exp(-Image::ExifTool::Canon::CanonEv($val)*log(2))".to_string(),
                    ),
                    print_conv_expr: Some(
                        "Image::ExifTool::Exif::PrintExposureTime($val)".to_string(),
                    ),
                    data_member: None,
                    group: None,
                    priority: Some(0),
                },
            ],
            // Legacy fields for backward compatibility
            format_spec: None,
            format: None,
            mask: None,
            print_conv: None,
            data_member: None,
            group: None,
        };

        use crate::formats::FileFormat;

        // Test context for Canon 20D (should match first variant)
        let canon_20d_context = ProcessorContext {
            file_format: FileFormat::Jpeg,
            manufacturer: Some("Canon".to_string()),
            model: Some("Canon EOS 20D".to_string()),
            firmware: None,
            format_version: None,
            table_name: "Canon::ShotInfo".to_string(),
            tag_id: None,
            directory_path: Vec::new(),
            data_offset: 0,
            parent_tags: std::collections::HashMap::new(),
            parameters: std::collections::HashMap::new(),
            byte_order: None,
            base_offset: 0,
            data_size: None,
        };

        // Test context for Canon 5D (should match second variant - default)
        let canon_5d_context = ProcessorContext {
            file_format: FileFormat::Jpeg,
            manufacturer: Some("Canon".to_string()),
            model: Some("Canon EOS 5D".to_string()),
            firmware: None,
            format_version: None,
            table_name: "Canon::ShotInfo".to_string(),
            tag_id: None,
            directory_path: Vec::new(),
            data_offset: 0,
            parent_tags: std::collections::HashMap::new(),
            parameters: std::collections::HashMap::new(),
            byte_order: None,
            base_offset: 0,
            data_size: None,
        };

        // Get active variant for 20D - should match the conditional variant
        let variant_20d = exposure_time_tag.get_active_variant(&canon_20d_context);
        assert!(variant_20d.is_some(), "Should find a variant for Canon 20D");

        let variant_20d = variant_20d.unwrap();
        assert_eq!(variant_20d.name, "ExposureTime");
        assert!(
            variant_20d.condition.is_some(),
            "20D should match conditional variant"
        );
        assert!(
            variant_20d
                .value_conv
                .as_ref()
                .unwrap()
                .contains("*1000/32"),
            "20D variant should include *1000/32 conversion"
        );

        // Get active variant for 5D - should match the default variant
        let variant_5d = exposure_time_tag.get_active_variant(&canon_5d_context);
        assert!(variant_5d.is_some(), "Should find a variant for Canon 5D");

        let variant_5d = variant_5d.unwrap();
        assert_eq!(variant_5d.name, "ExposureTime");
        assert!(
            variant_5d.condition.is_none(),
            "5D should match default variant"
        );
        assert!(
            !variant_5d.value_conv.as_ref().unwrap().contains("*1000/32"),
            "5D variant should NOT include *1000/32 conversion"
        );

        // Verify that both variants have the same print conversion
        assert_eq!(variant_20d.print_conv_expr, variant_5d.print_conv_expr);
        assert_eq!(
            variant_20d.print_conv_expr,
            Some("Image::ExifTool::Exif::PrintExposureTime($val)".to_string())
        );
    }

    #[test]
    fn test_simple_tag_creation() {
        // Test the simple tag creation helper
        let simple_tag = BinaryDataTag::simple("TestTag".to_string());

        assert_eq!(simple_tag.name, "TestTag");
        assert_eq!(simple_tag.variants.len(), 1);
        assert_eq!(simple_tag.variants[0].name, "TestTag");
        assert!(simple_tag.variants[0].condition.is_none());
    }

    #[test]
    fn test_from_legacy_creation() {
        // Test the from_legacy helper
        let mut print_conv = HashMap::new();
        print_conv.insert(1u32, "On".to_string());
        print_conv.insert(0u32, "Off".to_string());

        let legacy_tag = BinaryDataTag::from_legacy(
            "TestTag".to_string(),
            Some(FormatSpec::Fixed(BinaryDataFormat::Int16u)),
            Some(BinaryDataFormat::Int16u),
            None,
            Some(print_conv.clone()),
            Some("TestMember".to_string()),
            Some(0),
        );

        assert_eq!(legacy_tag.name, "TestTag");
        assert_eq!(legacy_tag.variants.len(), 1);
        assert_eq!(legacy_tag.variants[0].name, "TestTag");
        assert_eq!(legacy_tag.variants[0].print_conv, Some(print_conv));
        assert_eq!(
            legacy_tag.variants[0].data_member,
            Some("TestMember".to_string())
        );
    }
}
