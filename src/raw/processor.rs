//! RAW processing orchestration and handler traits
//!
//! This module implements the central RAW processor that dispatches to
//! manufacturer-specific handlers based on detected format.

use crate::exif::ExifReader;
use crate::file_detection::FileTypeDetectionResult;
use crate::types::{ExifError, Result};
use std::collections::HashMap;

use super::{detector::detect_raw_format, RawFormat};

/// Trait for manufacturer-specific RAW format handlers
/// Each RAW format (Kyocera, Canon, Nikon, etc.) implements this trait
/// ExifTool: Each manufacturer module has ProcessBinaryData or custom processing
pub trait RawFormatHandler: Send + Sync {
    /// Process RAW format data and extract metadata
    /// ExifTool: Usually ProcessBinaryData but can be custom for complex formats
    fn process_raw(&self, reader: &mut ExifReader, data: &[u8]) -> Result<()>;

    /// Get handler name for debugging and logging
    fn name(&self) -> &'static str;

    /// Validate that this data is the correct format for this handler
    /// ExifTool: Each module has format validation logic
    fn validate_format(&self, data: &[u8]) -> bool;
}

/// Central RAW processor that routes to manufacturer-specific handlers
/// ExifTool: Main ExifTool dispatcher routes to manufacturer modules
pub struct RawProcessor {
    /// Registered format handlers
    handlers: HashMap<RawFormat, Box<dyn RawFormatHandler>>,
}

impl RawProcessor {
    /// Create new RAW processor with all supported handlers registered
    pub fn new() -> Self {
        let mut handlers: HashMap<RawFormat, Box<dyn RawFormatHandler>> = HashMap::new();

        // Register Kyocera handler
        // ExifTool: KyoceraRaw.pm module registration
        handlers.insert(
            RawFormat::Kyocera,
            Box::new(super::formats::kyocera::KyoceraRawHandler::new()),
        );

        // Register Minolta handler
        // ExifTool: MinoltaRaw.pm module registration
        handlers.insert(
            RawFormat::Minolta,
            Box::new(super::formats::minolta::MinoltaRawHandler::new()),
        );

        // Register Panasonic handler
        // ExifTool: PanasonicRaw.pm module registration
        handlers.insert(
            RawFormat::Panasonic,
            Box::new(super::formats::panasonic::PanasonicRawHandler::new()),
        );

        // Register Olympus handler
        // ExifTool: Olympus.pm module registration
        handlers.insert(
            RawFormat::Olympus,
            Box::new(super::formats::olympus::OlympusRawHandler::new()),
        );

        // Register Canon handler
        // ExifTool: Canon.pm module registration
        handlers.insert(
            RawFormat::Canon,
            Box::new(super::formats::canon::CanonRawHandler::new()),
        );

        // Register Sony handler
        // ExifTool: Sony.pm module registration
        handlers.insert(
            RawFormat::Sony,
            Box::new(super::formats::sony::SonyRawHandler::new()),
        );

        // Future handlers will be registered here:
        // handlers.insert(RawFormat::Nikon, Box::new(NikonRawHandler::new()));

        Self { handlers }
    }

    /// Process RAW file data
    /// ExifTool: Main entry point that detects format and dispatches to appropriate module
    pub fn process_raw(
        &self,
        reader: &mut ExifReader,
        data: &[u8],
        detection_result: &FileTypeDetectionResult,
    ) -> Result<()> {
        // Detect RAW format
        let format = detect_raw_format(detection_result);

        // Get the appropriate handler
        if let Some(handler) = self.handlers.get(&format) {
            // Validate format before processing
            if !handler.validate_format(data) {
                return Err(ExifError::ParseError(format!(
                    "Invalid {} RAW format - failed validation",
                    format.name()
                )));
            }

            // Process the RAW data
            handler.process_raw(reader, data)?;
        } else {
            return Err(ExifError::Unsupported(format!(
                "Unsupported RAW format: {}",
                format.name()
            )));
        }

        Ok(())
    }

    /// Get list of supported formats
    pub fn supported_formats(&self) -> Vec<RawFormat> {
        self.handlers.keys().copied().collect()
    }
}

impl Default for RawProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TagValue;

    // Mock handler for testing
    #[allow(dead_code)]
    struct MockRawHandler {
        name: String,
        should_validate: bool,
    }

    impl MockRawHandler {
        #[allow(dead_code)]
        fn new(name: String, should_validate: bool) -> Self {
            Self {
                name,
                should_validate,
            }
        }
    }

    impl RawFormatHandler for MockRawHandler {
        fn process_raw(&self, reader: &mut ExifReader, _data: &[u8]) -> Result<()> {
            // Add a test tag to verify processing was called
            reader.add_test_tag(
                0x100,
                TagValue::String("test".to_string()),
                "TestRAW",
                "TestIFD",
            );
            Ok(())
        }

        fn name(&self) -> &'static str {
            // This is a bit of a hack for testing - in real code, name should be static
            "MockHandler"
        }

        fn validate_format(&self, _data: &[u8]) -> bool {
            self.should_validate
        }
    }

    #[test]
    fn test_raw_processor_creation() {
        let processor = RawProcessor::new();
        let supported = processor.supported_formats();

        assert!(supported.contains(&RawFormat::Kyocera));
        assert!(supported.contains(&RawFormat::Minolta));
        assert!(supported.contains(&RawFormat::Panasonic));
        assert!(supported.contains(&RawFormat::Olympus));
        assert!(supported.contains(&RawFormat::Canon));
        assert_eq!(supported.len(), 6); // Should have exactly 6 supported formats
    }

    #[test]
    fn test_raw_processor_unsupported_format() {
        let processor = RawProcessor::new();
        let mut reader = ExifReader::new();

        let detection_result = FileTypeDetectionResult {
            file_type: "UNKNOWN".to_string(),
            format: "UNKNOWN".to_string(),
            mime_type: "application/octet-stream".to_string(),
            description: "Unknown format".to_string(),
        };

        let data = vec![0u8; 100];
        let result = processor.process_raw(&mut reader, &data, &detection_result);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported RAW format"));
    }
}
