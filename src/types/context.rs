//! Expression evaluation context for PrintConv/ValueConv/Condition functions
//!
//! This module provides the ExifContext type that gives generated expression functions
//! access to tag values, processing state, and options during evaluation.

use crate::types::{DataMemberValue, TagValue};
use std::collections::HashMap;

/// Expression evaluation context providing access to ExifTool's `$self` object equivalent
///
/// Based on ExifTool.pm research, `$self` contains three main namespaces:
/// 1. **DataMembers** - Variables like `$self{FocalUnits}`, `$self{Make}` (stored directly in $self)
/// 2. **State Variables** - Processing state like `$self{DIR_NAME}`, `$self{FILE_TYPE}`
/// 3. **Options** - Configuration via `$self->Options("OptionName")` (TODO: implement later)
///
/// Key insight: Expression `$self{TagName}` accesses DataMember variables, NOT final tag values.
/// DataMembers are set by RawConv like `$$self{Make} = $val` and cleared between files.
#[derive(Debug, Clone)]
pub struct ExifContext {
    /// DataMember variables (lowercase names in ExifTool, cleared between files)
    ///
    /// These correspond to ExifTool's `$$self{VarName}` direct storage pattern.
    /// Examples: FocalUnits, Make, Model, AFAreaXPosition, FujiLayout, TimeScale
    ///
    /// Set by: `DataMember => 'VarName'` and `RawConv => '$$self{VarName} = $val'`
    /// Used by: `ValueConv => '$val / ($$self{FocalUnits} || 1)'`
    pub data_members: HashMap<String, TagValue>,

    /// Processing state variables (uppercase names, reset during Init)
    ///
    /// These correspond to ExifTool's processing state stored in `$self`.
    /// Examples: DIR_NAME, FILE_TYPE, TIFF_TYPE, BASE, PROCESSED
    pub state: HashMap<String, String>,

    /// Current directory path stack (corresponds to $$self{PATH})
    /// Used for directory hierarchy tracking during nested IFD processing
    pub path: Vec<String>,

    /// Base offset for pointer calculations (corresponds to $$self{BASE})
    /// Used in offset calculations for subdirectories and maker notes
    pub base_offset: u64,
    // TODO: Add Options support later
    // /// Processing options and flags (corresponds to $$self{OPTIONS})
    // ///
    // /// Accessed via `$self->Options("OptionName")` in ExifTool expressions
    // /// Examples: Unknown, DateFormat, QuickTimeUTC, ExtractEmbedded
    // pub options: HashMap<String, OptionValue>,
}

// TODO: Uncomment when implementing Options support
// /// Option value type for ExifTool options
// #[derive(Debug, Clone, PartialEq)]
// pub enum OptionValue {
//     Boolean(bool),
//     String(String),
//     Integer(i64),
// }

impl ExifContext {
    /// Create new empty context
    pub fn new() -> Self {
        Self {
            data_members: HashMap::new(),
            state: HashMap::new(),
            path: Vec::new(),
            base_offset: 0,
            // options: HashMap::new(),  // TODO: uncomment later
        }
    }

    /// Get DataMember value by name (e.g., "FocalUnits", "Make", "Model")
    ///
    /// This corresponds to `$$self{VarName}` access in ExifTool expressions.
    /// Returns None if DataMember not found.
    pub fn get_data_member(&self, name: &str) -> Option<&TagValue> {
        self.data_members.get(name)
    }

    /// Set DataMember value by name
    ///
    /// This corresponds to `$$self{VarName} = $val` assignment in RawConv.
    pub fn set_data_member(&mut self, name: String, value: TagValue) {
        self.data_members.insert(name, value);
    }

    /// Check if DataMember exists (for conditional expressions)
    pub fn has_data_member(&self, name: &str) -> bool {
        self.data_members.contains_key(name)
    }

    /// Get state variable by name (e.g., "DIR_NAME", "FILE_TYPE", "TIFF_TYPE")
    ///
    /// These are processing state variables that change during file processing.
    /// Returns None if state variable not found.
    pub fn get_state(&self, name: &str) -> Option<&str> {
        self.state.get(name).map(|s| s.as_str())
    }

    /// Set state variable by name
    pub fn set_state(&mut self, name: String, value: String) {
        self.state.insert(name, value);
    }

    /// Clear all DataMembers (corresponds to ExifTool's Init clearing lowercase variables)
    ///
    /// Called between files to reset DataMember state.
    /// From ExifTool.pm:4263: `delete $$self{$_} foreach grep /[a-z]/, keys %$self;`
    pub fn clear_data_members(&mut self) {
        self.data_members.clear();
    }

    /// Push directory to path stack (corresponds to $$self{PATH})
    pub fn push_path(&mut self, dir: String) {
        self.path.push(dir);
        // Update DIR_NAME state to current directory
        self.set_state("DIR_NAME".to_string(), dir);
    }

    /// Pop directory from path stack
    pub fn pop_path(&mut self) -> Option<String> {
        let popped = self.path.pop();
        // Update DIR_NAME to parent directory
        if let Some(parent) = self.path.last() {
            self.set_state("DIR_NAME".to_string(), parent.clone());
        }
        popped
    }

    /// Get current directory name (last path component)
    pub fn current_dir(&self) -> Option<&str> {
        self.path.last().map(|s| s.as_str())
    }

    /// Get full path as string
    pub fn path_string(&self) -> String {
        self.path.join("/")
    }

    // TODO: Add Options support methods later
    // /// Get option value by name
    // pub fn get_option(&self, name: &str) -> Option<&OptionValue> {
    //     self.options.get(name)
    // }
    //
    // /// Check if boolean option is enabled (corresponds to $self->Options("OptionName"))
    // pub fn is_option_enabled(&self, name: &str) -> bool {
    //     match self.get_option(name) {
    //         Some(OptionValue::Boolean(b)) => *b,
    //         _ => false,
    //     }
    // }
}

impl Default for ExifContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = ExifContext::new();
        assert!(ctx.data_members.is_empty());
        assert!(ctx.state.is_empty());
        assert!(ctx.path.is_empty());
        assert_eq!(ctx.base_offset, 0);
    }

    #[test]
    fn test_data_member_access() {
        let mut ctx = ExifContext::new();

        // Set and get DataMember (like $$self{FocalUnits} = 100)
        ctx.set_data_member("FocalUnits".to_string(), TagValue::U16(100));
        assert_eq!(ctx.get_data_member("FocalUnits"), Some(&TagValue::U16(100)));
        assert_eq!(ctx.get_data_member("Missing"), None);
        assert!(ctx.has_data_member("FocalUnits"));
        assert!(!ctx.has_data_member("Missing"));

        // Test clearing (like ExifTool's Init method)
        ctx.clear_data_members();
        assert!(!ctx.has_data_member("FocalUnits"));
    }

    #[test]
    fn test_state_variables() {
        let mut ctx = ExifContext::new();

        // Set state variables (like $$self{FILE_TYPE} = "JPEG")
        ctx.set_state("FILE_TYPE".to_string(), "JPEG".to_string());
        ctx.set_state("TIFF_TYPE".to_string(), "APP1".to_string());

        assert_eq!(ctx.get_state("FILE_TYPE"), Some("JPEG"));
        assert_eq!(ctx.get_state("TIFF_TYPE"), Some("APP1"));
        assert_eq!(ctx.get_state("Missing"), None);
    }

    #[test]
    fn test_path_management() {
        let mut ctx = ExifContext::new();

        // Push paths and verify DIR_NAME updates
        ctx.push_path("IFD0".to_string());
        assert_eq!(ctx.current_dir(), Some("IFD0"));
        assert_eq!(ctx.get_state("DIR_NAME"), Some("IFD0"));

        ctx.push_path("ExifIFD".to_string());
        assert_eq!(ctx.current_dir(), Some("ExifIFD"));
        assert_eq!(ctx.get_state("DIR_NAME"), Some("ExifIFD"));
        assert_eq!(ctx.path_string(), "IFD0/ExifIFD");

        // Pop and verify DIR_NAME reverts
        assert_eq!(ctx.pop_path(), Some("ExifIFD".to_string()));
        assert_eq!(ctx.current_dir(), Some("IFD0"));
        assert_eq!(ctx.get_state("DIR_NAME"), Some("IFD0"));
    }

    #[test]
    fn test_exiftool_expression_patterns() {
        let mut ctx = ExifContext::new();

        // Simulate ExifTool DataMember patterns
        ctx.set_data_member("Make".to_string(), TagValue::String("Canon".to_string()));
        ctx.set_data_member("Model".to_string(), TagValue::String("EOS 5D".to_string()));
        ctx.set_data_member("FocalUnits".to_string(), TagValue::U16(1000));

        // Test pattern: $val / ($$self{FocalUnits} || 1)
        if let Some(TagValue::U16(units)) = ctx.get_data_member("FocalUnits") {
            let focal_length = 50000; // Example raw value
            let converted = focal_length as f64 / *units as f64;
            assert_eq!(converted, 50.0);
        }

        // Test pattern: $$self{Model} =~ /EOS 5D/
        if let Some(TagValue::String(model)) = ctx.get_data_member("Model") {
            assert!(model.contains("EOS 5D"));
        }
    }
}
