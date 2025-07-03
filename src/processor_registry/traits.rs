//! Core trait definitions for the enhanced processor architecture
//!
//! This module defines the fundamental traits that enable ExifTool's sophisticated
//! processor dispatch system while maintaining Rust's type safety and performance.

use crate::types::{Result, TagValue};
use std::collections::HashMap;
use std::sync::Arc;

use super::{ProcessorCapability, ProcessorContext};

/// Core trait for binary data processors
///
/// This trait abstracts ExifTool's various processing functions (ProcessBinaryData,
/// ProcessNikonEncrypted, ProcessSerialData, etc.) into a unified interface that
/// supports capability assessment and rich context passing.
///
/// ## ExifTool Reference
///
/// ExifTool processors have the signature: `sub ProcessorName($$$) { my ($et, $dirInfo, $tagTablePtr) = @_; }`
/// This trait provides equivalent functionality with Rust type safety.
pub trait BinaryDataProcessor: Send + Sync {
    /// Assess this processor's capability to handle the given context
    ///
    /// Returns a capability assessment that helps the registry select the most
    /// appropriate processor for the current data and context.
    ///
    /// ## ExifTool Reference
    ///
    /// This implements the conditional logic found in ExifTool's Condition expressions:
    /// ```perl
    /// {
    ///     Condition => '$$self{Model} =~ /EOS R5/',
    ///     SubDirectory => { ProcessProc => \&ProcessCanonSerialData }
    /// }
    /// ```
    fn can_process(&self, context: &ProcessorContext) -> ProcessorCapability;

    /// Process binary data and extract tags
    ///
    /// This is the core processing function that extracts metadata from binary data
    /// using the provided context. Returns extracted tags, warnings, and potential
    /// nested processors for recursive processing.
    ///
    /// ## ExifTool Reference
    ///
    /// Equivalent to ExifTool's main processor functions like ProcessBinaryData,
    /// ProcessNikonEncrypted, etc.
    fn process_data(&self, data: &[u8], context: &ProcessorContext) -> Result<ProcessorResult>;

    /// Get metadata about this processor
    ///
    /// Returns descriptive metadata for debugging, documentation, and introspection.
    /// This helps with understanding processor selection decisions and capabilities.
    fn get_metadata(&self) -> ProcessorMetadata;
}

/// Result returned by processor after processing binary data
///
/// This structure encapsulates everything a processor produces, including extracted
/// tags, warnings, and instructions for further processing.
///
/// ## ExifTool Reference
///
/// ExifTool processors modify the ExifTool object state and may trigger recursive
/// processing. This structure captures those side effects explicitly.
#[derive(Debug)]
pub struct ProcessorResult {
    /// Tags extracted by the processor
    /// Maps tag names to their extracted values
    pub extracted_tags: HashMap<String, TagValue>,

    /// Warnings generated during processing
    /// Used for non-fatal issues like data corruption or unexpected formats
    pub warnings: Vec<String>,

    /// Additional processors to run for nested processing
    /// Each entry specifies a processor and the context it should use
    /// This enables ExifTool's recursive SubDirectory processing
    pub next_processors: Vec<(ProcessorKey, ProcessorContext)>,
}

impl ProcessorResult {
    /// Create a new empty processor result
    pub fn new() -> Self {
        Self {
            extracted_tags: HashMap::new(),
            warnings: Vec::new(),
            next_processors: Vec::new(),
        }
    }

    /// Create processor result with extracted tags
    pub fn with_tags(extracted_tags: HashMap<String, TagValue>) -> Self {
        Self {
            extracted_tags,
            warnings: Vec::new(),
            next_processors: Vec::new(),
        }
    }

    /// Add a warning to the result
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Add a tag to the extracted results
    pub fn add_tag(&mut self, name: String, value: TagValue) {
        self.extracted_tags.insert(name, value);
    }

    /// Add a nested processor for recursive processing
    pub fn add_nested_processor(&mut self, key: ProcessorKey, context: ProcessorContext) {
        self.next_processors.push((key, context));
    }
}

impl Default for ProcessorResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Metadata about a processor for introspection and debugging
///
/// This provides human-readable information about what a processor does,
/// what it supports, and what context it requires.
#[derive(Debug, Clone)]
pub struct ProcessorMetadata {
    /// Human-readable name of the processor
    pub name: String,

    /// Description of what this processor handles
    pub description: String,

    /// Manufacturers this processor supports (e.g., ["Canon", "Nikon"])
    pub supported_manufacturers: Vec<String>,

    /// Required context fields for this processor to function
    /// Used for validation and error reporting
    pub required_context: Vec<String>,

    /// Optional context fields that enhance processing
    pub optional_context: Vec<String>,

    /// Example conditions where this processor would be selected
    pub example_conditions: Vec<String>,
}

impl ProcessorMetadata {
    /// Create new processor metadata
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            supported_manufacturers: Vec::new(),
            required_context: Vec::new(),
            optional_context: Vec::new(),
            example_conditions: Vec::new(),
        }
    }

    /// Add supported manufacturer
    pub fn with_manufacturer(mut self, manufacturer: String) -> Self {
        self.supported_manufacturers.push(manufacturer);
        self
    }

    /// Add required context field
    pub fn with_required_context(mut self, field: String) -> Self {
        self.required_context.push(field);
        self
    }

    /// Add optional context field
    pub fn with_optional_context(mut self, field: String) -> Self {
        self.optional_context.push(field);
        self
    }

    /// Add example condition
    pub fn with_example_condition(mut self, condition: String) -> Self {
        self.example_conditions.push(condition);
        self
    }
}

/// Unique identifier for a processor in the registry
///
/// This provides a hierarchical naming scheme that mirrors ExifTool's processor
/// organization: namespace (manufacturer) + processor name + optional variant.
///
/// ## ExifTool Reference
///
/// ExifTool processors are referenced by function names like:
/// - `ProcessBinaryData` (generic)
/// - `ProcessNikonEncrypted` (manufacturer-specific)
/// - `ProcessCanonSerialData` (manufacturer + type specific)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProcessorKey {
    /// Namespace (typically manufacturer: "Canon", "Nikon", "EXIF", etc.)
    pub namespace: String,

    /// Processor name within the namespace ("SerialData", "AFInfo", "Encrypted", etc.)
    pub processor_name: String,

    /// Optional variant for model-specific processors ("MkII", "Z9", etc.)
    pub variant: Option<String>,
}

impl ProcessorKey {
    /// Create a new processor key
    pub fn new(namespace: String, processor_name: String) -> Self {
        Self {
            namespace,
            processor_name,
            variant: None,
        }
    }

    /// Create a processor key with variant
    pub fn with_variant(namespace: String, processor_name: String, variant: String) -> Self {
        Self {
            namespace,
            processor_name,
            variant: Some(variant),
        }
    }
}

impl std::fmt::Display for ProcessorKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.variant {
            Some(variant) => write!(
                f,
                "{}::{}::{}",
                self.namespace, self.processor_name, variant
            ),
            None => write!(f, "{}::{}", self.namespace, self.processor_name),
        }
    }
}

/// Arc-wrapped processor for efficient sharing in the registry
pub type SharedProcessor = Arc<dyn BinaryDataProcessor>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_key_formatting() {
        let key = ProcessorKey::new("Canon".to_string(), "SerialData".to_string());
        assert_eq!(format!("{key}"), "Canon::SerialData");

        let key_with_variant = ProcessorKey::with_variant(
            "Canon".to_string(),
            "SerialData".to_string(),
            "MkII".to_string(),
        );
        assert_eq!(format!("{key_with_variant}"), "Canon::SerialData::MkII");
    }

    #[test]
    fn test_processor_result_creation() {
        let mut result = ProcessorResult::new();
        result.add_tag("TestTag".to_string(), TagValue::String("test".to_string()));
        result.add_warning("Test warning".to_string());

        assert_eq!(result.extracted_tags.len(), 1);
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.next_processors.len(), 0);
    }

    #[test]
    fn test_processor_metadata_builder() {
        let metadata =
            ProcessorMetadata::new("Test Processor".to_string(), "A test processor".to_string())
                .with_manufacturer("Canon".to_string())
                .with_required_context("manufacturer".to_string())
                .with_example_condition("$model =~ /EOS R5/".to_string());

        assert_eq!(metadata.supported_manufacturers, vec!["Canon"]);
        assert_eq!(metadata.required_context, vec!["manufacturer"]);
        assert_eq!(metadata.example_conditions, vec!["$model =~ /EOS R5/"]);
    }
}
