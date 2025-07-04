//! ProcessorContext for rich metadata passing between processing layers
//!
//! This module provides the ProcessorContext structure that carries comprehensive
//! metadata about the current processing state, enabling sophisticated processor
//! selection and conditional dispatch.

use crate::formats::FileFormat;
use crate::types::TagValue;
use std::collections::HashMap;

/// Rich context passed to processors for capability assessment and processing
///
/// This structure provides all the information a processor needs to determine
/// if it can handle the current data and to perform sophisticated processing.
/// It mirrors ExifTool's combination of $self (ExifTool object state) and
/// $dirInfo (directory information).
///
/// ## ExifTool Reference
///
/// ExifTool processors receive context through multiple mechanisms:
/// - `$self` - ExifTool object with extracted tags and state
/// - `$dirInfo` - Directory information hash with processing parameters
/// - Global state like current file type, byte order, etc.
#[derive(Debug, Clone)]
pub struct ProcessorContext {
    /// File format being processed
    /// Used for processor selection (e.g., JPEG vs TIFF vs RAW)
    pub file_format: FileFormat,

    /// Camera manufacturer extracted from Make tag
    /// Primary factor in processor selection
    pub manufacturer: Option<String>,

    /// Camera model extracted from Model tag
    /// Used for model-specific processor variants
    pub model: Option<String>,

    /// Firmware version if available
    /// Some processors need firmware-specific handling
    pub firmware: Option<String>,

    /// Format version for format-specific processing
    /// Used by processors that handle multiple format generations
    pub format_version: Option<String>,

    /// Current table name being processed
    /// Helps processors understand their context (e.g., "Canon::AFInfo")
    pub table_name: String,

    /// Current tag ID being processed (if applicable)
    /// Used for tag-specific processor selection
    pub tag_id: Option<u16>,

    /// IFD hierarchy path for nested processing
    /// Tracks the path through nested IFDs (e.g., ["IFD0", "ExifIFD", "MakerNotes"])
    pub directory_path: Vec<String>,

    /// Current data offset in the file
    /// Important for offset-based processors and validation
    pub data_offset: usize,

    /// Previously extracted tags available as context
    /// Processors can use these for conditional logic and cross-references
    pub parent_tags: HashMap<String, TagValue>,

    /// Additional parameters from SubDirectory configuration
    /// ExifTool: SubDirectory parameters like DecryptStart, ByteOrder, etc.
    pub parameters: HashMap<String, String>,

    /// Byte order for current data processing
    /// Some processors need explicit byte order information
    pub byte_order: Option<crate::tiff_types::ByteOrder>,

    /// Base offset for relative address calculations
    /// Critical for processors that handle offset-based data
    pub base_offset: usize,

    /// Size of data being processed (if known)
    /// Used for bounds checking and validation
    pub data_size: Option<usize>,
}

impl ProcessorContext {
    /// Create a new processor context with minimal required information
    pub fn new(file_format: FileFormat, table_name: String) -> Self {
        Self {
            file_format,
            manufacturer: None,
            model: None,
            firmware: None,
            format_version: None,
            table_name,
            tag_id: None,
            directory_path: Vec::new(),
            data_offset: 0,
            parent_tags: HashMap::new(),
            parameters: HashMap::new(),
            byte_order: None,
            base_offset: 0,
            data_size: None,
        }
    }

    /// Create context with manufacturer and model information
    pub fn with_camera_info(
        file_format: FileFormat,
        table_name: String,
        manufacturer: Option<String>,
        model: Option<String>,
    ) -> Self {
        Self {
            file_format,
            manufacturer,
            model,
            table_name: table_name.clone(),
            ..Self::new(file_format, table_name)
        }
    }

    /// Set manufacturer information
    pub fn with_manufacturer(mut self, manufacturer: String) -> Self {
        self.manufacturer = Some(manufacturer);
        self
    }

    /// Set model information
    pub fn with_model(mut self, model: String) -> Self {
        self.model = Some(model);
        self
    }

    /// Set firmware version
    pub fn with_firmware(mut self, firmware: String) -> Self {
        self.firmware = Some(firmware);
        self
    }

    /// Set format version
    pub fn with_format_version(mut self, version: String) -> Self {
        self.format_version = Some(version);
        self
    }

    /// Set current tag ID
    pub fn with_tag_id(mut self, tag_id: u16) -> Self {
        self.tag_id = Some(tag_id);
        self
    }

    /// Set directory path
    pub fn with_directory_path(mut self, path: Vec<String>) -> Self {
        self.directory_path = path;
        self
    }

    /// Set data offset
    pub fn with_data_offset(mut self, offset: usize) -> Self {
        self.data_offset = offset;
        self
    }

    /// Set parent tags
    pub fn with_parent_tags(mut self, tags: HashMap<String, TagValue>) -> Self {
        self.parent_tags = tags;
        self
    }

    /// Add a parent tag
    pub fn add_parent_tag(&mut self, name: String, value: TagValue) {
        self.parent_tags.insert(name, value);
    }

    /// Add a parent tag (builder pattern)
    pub fn with_parent_tag(mut self, name: String, value: TagValue) -> Self {
        self.parent_tags.insert(name, value);
        self
    }

    /// Set processing parameters
    pub fn with_parameters(mut self, parameters: HashMap<String, String>) -> Self {
        self.parameters = parameters;
        self
    }

    /// Add a processing parameter
    pub fn add_parameter(&mut self, key: String, value: String) {
        self.parameters.insert(key, value);
    }

    /// Set byte order
    pub fn with_byte_order(mut self, byte_order: crate::tiff_types::ByteOrder) -> Self {
        self.byte_order = Some(byte_order);
        self
    }

    /// Set base offset
    pub fn with_base_offset(mut self, base_offset: usize) -> Self {
        self.base_offset = base_offset;
        self
    }

    /// Set data size
    pub fn with_data_size(mut self, size: usize) -> Self {
        self.data_size = Some(size);
        self
    }

    /// Get a parameter value by key
    pub fn get_parameter(&self, key: &str) -> Option<&String> {
        self.parameters.get(key)
    }

    /// Get a parent tag value by name
    pub fn get_parent_tag(&self, name: &str) -> Option<&TagValue> {
        self.parent_tags.get(name)
    }

    /// Check if a required context field is available
    pub fn has_required_field(&self, field: &str) -> bool {
        match field {
            "manufacturer" => self.manufacturer.is_some(),
            "model" => self.model.is_some(),
            "firmware" => self.firmware.is_some(),
            "format_version" => self.format_version.is_some(),
            "tag_id" => self.tag_id.is_some(),
            "byte_order" => self.byte_order.is_some(),
            _ => {
                // Check parameters and parent tags
                self.parameters.contains_key(field) || self.parent_tags.contains_key(field)
            }
        }
    }

    /// Validate that all required fields are present
    pub fn validate_required_fields(&self, required_fields: &[String]) -> Result<(), Vec<String>> {
        let missing_fields: Vec<String> = required_fields
            .iter()
            .filter(|field| !self.has_required_field(field))
            .cloned()
            .collect();

        if missing_fields.is_empty() {
            Ok(())
        } else {
            Err(missing_fields)
        }
    }

    /// Create a derived context for nested processing
    ///
    /// This creates a new context based on the current one but updated for
    /// processing a nested structure (like a SubDirectory).
    pub fn derive_for_nested(&self, table_name: String, tag_id: Option<u16>) -> Self {
        let mut derived = self.clone();
        derived.table_name = table_name;
        derived.tag_id = tag_id;

        // Add current directory to path for nested processing
        if !self.table_name.is_empty() {
            derived.directory_path.push(self.table_name.clone());
        }

        derived
    }

    /// Get the current directory path as a string
    pub fn get_directory_path_string(&self) -> String {
        if self.directory_path.is_empty() {
            self.table_name.clone()
        } else {
            format!("{}/{}", self.directory_path.join("/"), self.table_name)
        }
    }

    /// Check if this context represents a specific manufacturer
    pub fn is_manufacturer(&self, manufacturer: &str) -> bool {
        self.manufacturer
            .as_ref()
            .map(|m| m.eq_ignore_ascii_case(manufacturer))
            .unwrap_or(false)
    }

    /// Check if the model matches a pattern
    pub fn model_matches(&self, pattern: &str) -> bool {
        self.model
            .as_ref()
            .map(|m| m.contains(pattern))
            .unwrap_or(false)
    }

    /// Get encryption key information for Nikon processors
    ///
    /// This is a specialized method for Nikon's encrypted data processing
    /// that extracts the serial number and shutter count for decryption.
    pub fn get_nikon_encryption_keys(&self) -> Option<(String, u32)> {
        let serial = self
            .get_parent_tag("SerialNumber")?
            .as_string()?
            .to_string();
        let shutter_count = self.get_parent_tag("ShutterCount")?.as_u32()?;
        Some((serial, shutter_count))
    }
}

impl Default for ProcessorContext {
    fn default() -> Self {
        Self::new(FileFormat::Jpeg, String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let context = ProcessorContext::new(FileFormat::Jpeg, "EXIF::Main".to_string());
        assert_eq!(context.file_format, FileFormat::Jpeg);
        assert_eq!(context.table_name, "EXIF::Main");
        assert!(context.manufacturer.is_none());
    }

    #[test]
    fn test_context_builder_pattern() {
        let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::Main".to_string())
            .with_manufacturer("Canon".to_string())
            .with_model("EOS R5".to_string())
            .with_tag_id(0x0001);

        assert_eq!(context.manufacturer, Some("Canon".to_string()));
        assert_eq!(context.model, Some("EOS R5".to_string()));
        assert_eq!(context.tag_id, Some(0x0001));
    }

    #[test]
    fn test_required_field_validation() {
        let mut context = ProcessorContext::new(FileFormat::Jpeg, "Canon::Main".to_string())
            .with_manufacturer("Canon".to_string());

        assert!(context.has_required_field("manufacturer"));
        assert!(!context.has_required_field("model"));

        let required = vec!["manufacturer".to_string(), "model".to_string()];
        let result = context.validate_required_fields(&required);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), vec!["model"]);

        context = context.with_model("EOS R5".to_string());
        let result = context.validate_required_fields(&required);
        assert!(result.is_ok());
    }

    #[test]
    fn test_context_derivation() {
        let parent_context = ProcessorContext::new(FileFormat::Jpeg, "Canon::Main".to_string())
            .with_manufacturer("Canon".to_string())
            .with_directory_path(vec!["IFD0".to_string()]);

        let derived = parent_context.derive_for_nested("Canon::AFInfo".to_string(), Some(0x0001));

        assert_eq!(derived.table_name, "Canon::AFInfo");
        assert_eq!(derived.tag_id, Some(0x0001));
        assert_eq!(derived.directory_path, vec!["IFD0", "Canon::Main"]);
        assert_eq!(derived.manufacturer, Some("Canon".to_string()));
    }

    #[test]
    fn test_manufacturer_checking() {
        let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::Main".to_string())
            .with_manufacturer("CANON".to_string());

        assert!(context.is_manufacturer("Canon"));
        assert!(context.is_manufacturer("canon"));
        assert!(context.is_manufacturer("CANON"));
        assert!(!context.is_manufacturer("Nikon"));
    }

    #[test]
    fn test_model_matching() {
        let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::Main".to_string())
            .with_model("Canon EOS R5".to_string());

        assert!(context.model_matches("EOS R5"));
        assert!(context.model_matches("Canon"));
        assert!(!context.model_matches("R6"));
    }
}
