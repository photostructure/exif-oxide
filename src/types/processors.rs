//! Processor types for EXIF data processing dispatch
//!
//! This module defines the processor type hierarchy used for ExifTool's
//! PROCESS_PROC dispatch system, including manufacturer-specific processors
//! and conditional dispatch logic.

use std::collections::HashMap;

/// Processor types for PROCESS_PROC dispatch system
/// ExifTool: Different processing procedures for different data formats
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessorType {
    /// Standard EXIF IFD processing (default)
    /// ExifTool: ProcessExif function
    Exif,
    /// Binary data processing with format tables
    /// ExifTool: ProcessBinaryData function
    BinaryData,
    /// GPS IFD processing
    /// ExifTool: Uses ProcessExif but with GPS-specific context
    Gps,
    /// Canon manufacturer-specific processing
    Canon(CanonProcessor),
    /// Nikon manufacturer-specific processing  
    Nikon(NikonProcessor),
    /// Sony manufacturer-specific processing
    Sony(SonyProcessor),
    /// Generic manufacturer processing
    Generic(String),
}

/// Canon-specific processor variants
/// ExifTool: Canon.pm has multiple processing procedures
#[derive(Debug, Clone, PartialEq)]
pub enum CanonProcessor {
    /// Standard Canon EXIF processing
    Main,
    /// Canon CameraSettings processing
    /// ExifTool: ProcessBinaryData for CameraSettings table
    CameraSettings,
    /// Canon AFInfo processing
    /// ExifTool: ProcessSerialData for AFInfo table
    AfInfo,
    /// Canon AFInfo2 processing  
    /// ExifTool: ProcessSerialData for AFInfo2 table
    AfInfo2,
    /// Canon serial data processing (generic)
    /// ExifTool: ProcessSerialData
    SerialData,
    /// Canon binary data processing (generic)
    BinaryData,
}

/// Nikon-specific processor variants
/// ExifTool: Nikon.pm has multiple processing procedures
#[derive(Debug, Clone, PartialEq)]
pub enum NikonProcessor {
    /// Standard Nikon EXIF processing
    Main,
    /// Nikon encrypted data processing
    /// ExifTool: ProcessNikonEncrypted
    Encrypted,
}

/// Sony-specific processor variants
/// ExifTool: Sony.pm has multiple processing procedures and signature detection
#[derive(Debug, Clone, PartialEq)]
pub enum SonyProcessor {
    /// Standard Sony EXIF processing with MakerNotes namespace
    /// ExifTool: Image::ExifTool::Sony::Main
    Main,
    /// Sony PIC format processing
    /// ExifTool: Image::ExifTool::Sony::PIC (DSC-H200/J20/W370/W510, MHS-TS20)
    Pic,
    /// Sony SRF format processing  
    /// ExifTool: Image::ExifTool::Sony::SRF
    Srf,
    /// Sony Ericsson mobile phone format
    /// ExifTool: Image::ExifTool::Sony::Ericsson
    Ericsson,
}

/// Conditional processor configuration for runtime dispatch
/// ExifTool: SubDirectory with Condition expressions
#[derive(Debug, Clone)]
pub struct ConditionalProcessor {
    /// Runtime condition to evaluate (None = unconditional)
    /// ExifTool: Condition => '$$valPt =~ /pattern/' expressions
    pub condition: Option<crate::conditions::Condition>,
    /// Processor to use when condition matches
    /// ExifTool: SubDirectory ProcessProc selection
    pub processor: ProcessorType,
    /// Parameters passed to processor
    /// ExifTool: SubDirectory parameters (DecryptStart, ByteOrder, etc.)
    pub parameters: HashMap<String, String>,
}

impl ConditionalProcessor {
    /// Create unconditional processor (always matches)
    /// ExifTool: SubDirectory without Condition
    pub fn unconditional(processor: ProcessorType) -> Self {
        Self {
            condition: None,
            processor,
            parameters: HashMap::new(),
        }
    }

    /// Create conditional processor with parameters
    /// ExifTool: SubDirectory with Condition and parameters
    pub fn conditional(
        condition: crate::conditions::Condition,
        processor: ProcessorType,
        parameters: HashMap<String, String>,
    ) -> Self {
        Self {
            condition: Some(condition),
            processor,
            parameters,
        }
    }
}

/// Processor dispatch configuration
/// ExifTool: Combination of table PROCESS_PROC and SubDirectory ProcessProc
#[derive(Debug, Clone)]
pub struct ProcessorDispatch {
    /// Table-level default processor
    /// ExifTool: $$tagTablePtr{PROCESS_PROC}
    pub table_processor: Option<ProcessorType>,
    /// Conditional processor selection by tag ID
    /// ExifTool: Multiple SubDirectory entries with Condition expressions
    pub conditional_processors: HashMap<u16, Vec<ConditionalProcessor>>,
    /// Legacy subdirectory overrides (backwards compatibility)
    /// ExifTool: $$subdir{ProcessProc} without conditions
    pub subdirectory_overrides: HashMap<u16, ProcessorType>,
    /// Global parameters passed to processor
    /// ExifTool: Table-level parameters
    pub parameters: HashMap<String, String>,
}

impl Default for ProcessorDispatch {
    fn default() -> Self {
        Self {
            table_processor: Some(ProcessorType::Exif), // Default fallback
            conditional_processors: HashMap::new(),
            subdirectory_overrides: HashMap::new(),
            parameters: HashMap::new(),
        }
    }
}

impl ProcessorDispatch {
    /// Create new dispatch configuration with table processor
    /// ExifTool: Table PROCESS_PROC setting
    pub fn with_table_processor(processor: ProcessorType) -> Self {
        Self {
            table_processor: Some(processor),
            ..Default::default()
        }
    }

    /// Add conditional processor for specific tag
    /// ExifTool: SubDirectory with Condition support
    pub fn add_conditional_processor(&mut self, tag_id: u16, conditional: ConditionalProcessor) {
        self.conditional_processors
            .entry(tag_id)
            .or_default()
            .push(conditional);
    }

    /// Add legacy subdirectory override (backwards compatibility)
    /// ExifTool: Simple SubDirectory ProcessProc override
    pub fn add_subdirectory_override(&mut self, tag_id: u16, processor: ProcessorType) {
        self.subdirectory_overrides.insert(tag_id, processor);
    }

    /// Set global parameter
    /// ExifTool: Table-level parameters
    pub fn set_parameter(&mut self, key: String, value: String) {
        self.parameters.insert(key, value);
    }
}
