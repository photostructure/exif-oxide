//! Nikon-specific BinaryDataProcessor implementations
//!
//! These processors implement the BinaryDataProcessor trait for Nikon camera data
//! while delegating to the existing Nikon implementation modules. This maintains
//! the "Trust ExifTool" principle by reusing proven processing logic.
//!
//! ## ExifTool Reference
//!
//! Nikon.pm ProcessNikonEncrypted and related processing functions

use super::super::{
    BinaryDataProcessor, ProcessorCapability, ProcessorContext, ProcessorMetadata, ProcessorResult,
};
use crate::implementations::nikon;
use crate::types::{Result, TagValue};
use std::collections::HashMap;
use tracing::debug;

/// Nikon Encrypted Data processor using existing implementation
///
/// Processes Nikon encrypted maker note data using the existing `nikon::encryption`
/// implementation. This processor detects encryption and manages keys for decryption.
///
/// ## ExifTool Reference
///
/// Nikon.pm ProcessNikonEncrypted function and encryption key management
pub struct NikonEncryptedDataProcessor;

impl BinaryDataProcessor for NikonEncryptedDataProcessor {
    fn can_process(&self, context: &ProcessorContext) -> ProcessorCapability {
        // Check for Nikon manufacturer (exact match with ExifTool patterns)
        if !context.is_manufacturer("NIKON CORPORATION") && !context.is_manufacturer("NIKON") {
            return ProcessorCapability::Incompatible;
        }

        // Perfect match for Nikon encrypted data tables
        if context.table_name.starts_with("Nikon::")
            && (context.table_name.contains("Encrypted") || context.table_name.contains("LensData"))
        {
            return ProcessorCapability::Perfect;
        }

        // Good match for Nikon-specific tables if we have encryption keys available
        if context.table_name.starts_with("Nikon::")
            && context.get_nikon_encryption_keys().is_some()
        {
            return ProcessorCapability::Good; // Can process with keys
        }

        // Only compatible with Nikon-specific tables
        ProcessorCapability::Incompatible
    }

    fn process_data(&self, data: &[u8], context: &ProcessorContext) -> Result<ProcessorResult> {
        debug!(
            "Processing Nikon encrypted data with {} bytes for table: {}",
            data.len(),
            context.table_name
        );

        let mut result = ProcessorResult::new();

        // Get encryption keys from context if available
        let keys = context.get_nikon_encryption_keys();

        // Detect encryption signature and process accordingly
        if data.len() >= 4 {
            // Check for common Nikon encryption signatures
            // ExifTool: Nikon.pm encryption detection patterns
            let has_encryption_signature = detect_nikon_encryption_signature(data);

            if has_encryption_signature {
                result.add_tag(
                    "EncryptionDetected".to_string(),
                    "Nikon encryption detected".into(),
                );

                // Process encryption status based on available keys
                if let Some(encryption_keys) = keys {
                    let (ref serial, count) = encryption_keys;
                    if !serial.is_empty() && count > 0 {
                        // We have keys - could decrypt if implemented
                        let status = format!(
                            "Encrypted data with keys available (serial: {serial}, count: {count})"
                        );
                        result.add_tag("EncryptionStatus".to_string(), TagValue::String(status));

                        // Use existing encryption processing (Phase 1: detection only)
                        // This delegates to the proven Nikon encryption logic
                        // For Phase 1, we work with the tuple format from get_nikon_encryption_keys
                        let mut nikon_keys = nikon::encryption::NikonEncryptionKeys::new(
                            context.manufacturer.clone().unwrap_or_default(),
                        );
                        nikon_keys.store_serial_key(serial.clone());
                        nikon_keys.store_count_key(count);
                        process_encrypted_data_with_existing_impl(
                            data,
                            context,
                            &nikon_keys,
                            &mut result,
                        )?;
                    } else {
                        result.add_tag(
                            "EncryptionStatus".to_string(),
                            TagValue::String(
                                "Encrypted data detected - encryption keys incomplete".to_string(),
                            ),
                        );
                        result.add_warning(
                            "Nikon encryption keys not available for decryption".to_string(),
                        );
                    }
                } else {
                    result.add_tag(
                        "EncryptionStatus".to_string(),
                        TagValue::String(
                            "Encrypted data detected - no encryption context".to_string(),
                        ),
                    );
                    result.add_warning("No Nikon encryption context available".to_string());
                }
            } else {
                // No encryption detected - process as regular binary data
                result.add_tag(
                    "EncryptionStatus".to_string(),
                    "No encryption detected".into(),
                );

                // Could process as regular Nikon binary data here
                process_unencrypted_nikon_data(data, context, &mut result)?;
            }
        } else {
            result.add_warning("Nikon data too short for encryption detection".to_string());
        }

        debug!(
            "Nikon encrypted data processor extracted {} tags with {} warnings",
            result.extracted_tags.len(),
            result.warnings.len()
        );

        Ok(result)
    }

    fn get_metadata(&self) -> ProcessorMetadata {
        ProcessorMetadata::new(
            "Nikon Encrypted Data Processor".to_string(),
            "Detects and processes Nikon encrypted maker note data".to_string(),
        )
        .with_manufacturer("NIKON CORPORATION".to_string())
        .with_manufacturer("NIKON".to_string())
        .with_required_context("manufacturer".to_string())
        .with_optional_context("nikon_encryption_keys".to_string())
        .with_example_condition("manufacturer.contains('NIKON') && encryption_detected".to_string())
    }
}

/// Nikon AF Info processor for autofocus data
///
/// Processes Nikon AF (autofocus) information using existing implementation logic.
/// This handles both encrypted and unencrypted AF data depending on the camera model.
///
/// ## ExifTool Reference
///
/// Nikon.pm AF processing and AFInfo table handling
pub struct NikonAFInfoProcessor;

impl BinaryDataProcessor for NikonAFInfoProcessor {
    fn can_process(&self, context: &ProcessorContext) -> ProcessorCapability {
        // Only process Nikon-specific tables, not standard EXIF directories
        // ExifTool Nikon.pm only processes Nikon:: prefixed tables
        if !context.is_manufacturer("NIKON CORPORATION") && !context.is_manufacturer("NIKON") {
            return ProcessorCapability::Incompatible;
        }

        // Perfect match for Nikon AF-related tables
        if context.table_name.starts_with("Nikon::")
            && (context.table_name.contains("AFInfo") || context.table_name.contains("AF"))
            && !context.table_name.contains("Encrypted")
        {
            return ProcessorCapability::Perfect;
        }

        // Good match for other Nikon-specific tables that might contain AF info
        if context.table_name.starts_with("Nikon::") && !context.table_name.contains("Encrypted") {
            return ProcessorCapability::Good;
        }

        // Incompatible with non-Nikon tables (like ExifIFD, GPS, etc.)
        ProcessorCapability::Incompatible
    }

    fn process_data(&self, data: &[u8], context: &ProcessorContext) -> Result<ProcessorResult> {
        debug!(
            "Processing Nikon AF data with {} bytes for table: {}",
            data.len(),
            context.table_name
        );

        let mut result = ProcessorResult::new();

        // Delegate to existing Nikon AF processing implementation
        // For Phase 1, create basic AF data extraction
        // TODO: Phase 2+ will implement full AF processing logic
        let mut af_data = HashMap::new();
        if data.len() >= 2 {
            let value = context
                .byte_order
                .unwrap_or(crate::tiff_types::ByteOrder::LittleEndian)
                .read_u16(data, 0)?;
            af_data.insert("AFValue".to_string(), crate::types::TagValue::U16(value));
        }

        for (tag_name, tag_value) in af_data {
            result.add_tag(tag_name, tag_value);
        }

        // Add metadata about the AF processing
        if let Some(model) = &context.model {
            result.add_tag(
                "ProcessedModel".to_string(),
                TagValue::String(model.clone()),
            );
        }

        if result.extracted_tags.is_empty() {
            result.add_warning("No Nikon AF data extracted".to_string());
        } else {
            debug!(
                "Nikon AF processor extracted {} tags",
                result.extracted_tags.len()
            );
        }

        Ok(result)
    }

    fn get_metadata(&self) -> ProcessorMetadata {
        ProcessorMetadata::new(
            "Nikon AF Info Processor".to_string(),
            "Processes Nikon autofocus information and AF point data".to_string(),
        )
        .with_manufacturer("NIKON CORPORATION".to_string())
        .with_manufacturer("NIKON".to_string())
        .with_required_context("manufacturer".to_string())
        .with_optional_context("model".to_string())
        .with_example_condition(
            "manufacturer.contains('NIKON') && table.contains('AF')".to_string(),
        )
    }
}

/// Nikon Lens Data processor for lens information
///
/// Processes Nikon lens data which may be encrypted depending on the camera model.
/// Delegates to existing lens database and processing logic.
///
/// ## ExifTool Reference
///
/// Nikon.pm LensData processing and lens database lookup
pub struct NikonLensDataProcessor;

impl BinaryDataProcessor for NikonLensDataProcessor {
    fn can_process(&self, context: &ProcessorContext) -> ProcessorCapability {
        // Check for Nikon manufacturer
        if !context.is_manufacturer("NIKON CORPORATION") && !context.is_manufacturer("NIKON") {
            return ProcessorCapability::Incompatible;
        }

        // Perfect match for Nikon-specific lens data tables
        if context.table_name.starts_with("Nikon::")
            && (context.table_name.contains("LensData") || context.table_name.contains("Lens"))
        {
            return ProcessorCapability::Perfect;
        }

        ProcessorCapability::Incompatible
    }

    fn process_data(&self, data: &[u8], context: &ProcessorContext) -> Result<ProcessorResult> {
        debug!(
            "Processing Nikon lens data with {} bytes for table: {}",
            data.len(),
            context.table_name
        );

        let mut result = ProcessorResult::new();

        // Check if this lens data is encrypted
        if detect_nikon_encryption_signature(data) {
            result.add_warning(
                "Nikon lens data is encrypted - decryption not implemented".to_string(),
            );
            result.add_tag(
                "LensDataStatus".to_string(),
                "Encrypted lens data detected".into(),
            );
        } else {
            // Process unencrypted lens data using existing implementation
            // For Phase 1, create basic lens data extraction
            // TODO: Phase 2+ will implement full lens database lookup
            let mut lens_info = HashMap::new();
            if data.len() >= 4 {
                let lens_id = context
                    .byte_order
                    .unwrap_or(crate::tiff_types::ByteOrder::LittleEndian)
                    .read_u32(data, 0)?;
                lens_info.insert("LensID".to_string(), crate::types::TagValue::U32(lens_id));
            }

            for (tag_name, tag_value) in lens_info {
                result.add_tag(tag_name, tag_value);
            }

            result.add_tag(
                "LensDataStatus".to_string(),
                "Unencrypted lens data processed".into(),
            );
        }

        debug!(
            "Nikon lens data processor extracted {} tags",
            result.extracted_tags.len()
        );

        Ok(result)
    }

    fn get_metadata(&self) -> ProcessorMetadata {
        ProcessorMetadata::new(
            "Nikon Lens Data Processor".to_string(),
            "Processes Nikon lens information and database lookup".to_string(),
        )
        .with_manufacturer("NIKON CORPORATION".to_string())
        .with_manufacturer("NIKON".to_string())
        .with_required_context("manufacturer".to_string())
        .with_example_condition(
            "manufacturer.contains('NIKON') && table.contains('Lens')".to_string(),
        )
    }
}

/// Detect Nikon encryption signature in data
///
/// This function checks for common Nikon encryption patterns to determine
/// if the data is encrypted and needs special processing.
///
/// ## ExifTool Reference
///
/// Nikon.pm encryption detection patterns
fn detect_nikon_encryption_signature(data: &[u8]) -> bool {
    if data.len() < 4 {
        return false;
    }

    // Check for common Nikon encryption signatures
    // ExifTool: Nikon.pm ProcessNikonEncrypted signature detection
    match &data[0..4] {
        [0x02, 0x00, 0x00, 0x00] => true, // Type 2 encryption
        [0x02, 0x04, _, _] => true,       // Type 2.04 encryption
        [0x04, 0x02, _, _] => true,       // Type 4.02 encryption
        _ => false,
    }
}

/// Process encrypted data using existing implementation
///
/// This function delegates to the existing Nikon encryption processing
/// logic while integrating with the new processor system.
fn process_encrypted_data_with_existing_impl(
    _data: &[u8],
    context: &ProcessorContext,
    keys: &nikon::encryption::NikonEncryptionKeys,
    result: &mut ProcessorResult,
) -> Result<()> {
    // This is Phase 1 implementation - detection and key management only
    // Real decryption would be implemented in later phases

    debug!("Processing encrypted Nikon data with existing implementation");

    // Validate encryption keys
    let manufacturer = context.manufacturer.as_deref().unwrap_or("Unknown");
    if let Err(e) = nikon::encryption::validate_encryption_keys(keys, manufacturer) {
        result.add_warning(format!("Encryption key validation failed: {e}"));
        return Ok(());
    }

    // Add information about the encryption context
    result.add_tag(
        "EncryptionKeys".to_string(),
        TagValue::string(format!(
            "Serial: {}, Count: {}",
            keys.get_serial_key().unwrap_or("none"),
            keys.get_count_key()
                .map(|c| c.to_string())
                .unwrap_or("none".to_string())
        )),
    );

    // TODO: Phase 2+ implementation would add actual decryption here
    // For now, we just detect and report the encryption status

    Ok(())
}

/// Process unencrypted Nikon data
///
/// This function processes Nikon data that is not encrypted using
/// standard binary data processing techniques.
fn process_unencrypted_nikon_data(
    data: &[u8],
    context: &ProcessorContext,
    result: &mut ProcessorResult,
) -> Result<()> {
    debug!("Processing unencrypted Nikon data");

    // Basic processing for unencrypted Nikon data
    // This would be expanded based on the specific table type

    let byte_order = context
        .byte_order
        .unwrap_or(crate::tiff_types::ByteOrder::LittleEndian);

    if data.len() >= 2 {
        // Extract some basic information
        let value = byte_order.read_u16(data, 0)?;
        result.add_tag("FirstValue".to_string(), TagValue::U16(value));
    }

    if data.len() >= 4 {
        let value = byte_order.read_u32(data, 0)?;
        result.add_tag("FirstU32Value".to_string(), TagValue::U32(value));
    }

    result.add_tag("DataLength".to_string(), TagValue::U32(data.len() as u32));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::FileFormat;
    use crate::processor_registry::ProcessorContext;

    #[test]
    fn test_nikon_encrypted_processor_capability() {
        let processor = NikonEncryptedDataProcessor;

        // Perfect match for Nikon encrypted data
        let context = ProcessorContext::new(FileFormat::Jpeg, "Nikon::Encrypted".to_string())
            .with_manufacturer("NIKON CORPORATION".to_string());

        assert_eq!(
            processor.can_process(&context),
            ProcessorCapability::Perfect
        );

        // Incompatible for non-Nikon
        let context = ProcessorContext::new(FileFormat::Jpeg, "Nikon::Encrypted".to_string())
            .with_manufacturer("Canon".to_string());

        assert_eq!(
            processor.can_process(&context),
            ProcessorCapability::Incompatible
        );
    }

    #[test]
    fn test_nikon_af_processor_capability() {
        let processor = NikonAFInfoProcessor;

        // Perfect match for Nikon AF data
        let context = ProcessorContext::new(FileFormat::Jpeg, "Nikon::AFInfo".to_string())
            .with_manufacturer("NIKON CORPORATION".to_string());

        assert_eq!(
            processor.can_process(&context),
            ProcessorCapability::Perfect
        );

        // Good match for general Nikon data
        let context = ProcessorContext::new(FileFormat::Jpeg, "Nikon::Main".to_string())
            .with_manufacturer("NIKON".to_string());

        assert_eq!(processor.can_process(&context), ProcessorCapability::Good);
    }

    #[test]
    fn test_nikon_lens_processor_capability() {
        let processor = NikonLensDataProcessor;

        // Perfect match for Nikon lens data
        let context = ProcessorContext::new(FileFormat::Jpeg, "Nikon::LensData".to_string())
            .with_manufacturer("NIKON CORPORATION".to_string());

        assert_eq!(
            processor.can_process(&context),
            ProcessorCapability::Perfect
        );

        // Incompatible for non-lens data
        let context = ProcessorContext::new(FileFormat::Jpeg, "Nikon::Other".to_string())
            .with_manufacturer("NIKON CORPORATION".to_string());

        assert_eq!(
            processor.can_process(&context),
            ProcessorCapability::Incompatible
        );
    }

    #[test]
    fn test_encryption_signature_detection() {
        // Test Type 2 encryption signature
        let encrypted_data = vec![0x02, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04];
        assert!(detect_nikon_encryption_signature(&encrypted_data));

        // Test non-encrypted data
        let normal_data = vec![0x01, 0x02, 0x03, 0x04];
        assert!(!detect_nikon_encryption_signature(&normal_data));

        // Test short data
        let short_data = vec![0x01, 0x02];
        assert!(!detect_nikon_encryption_signature(&short_data));
    }

    #[test]
    fn test_processor_metadata() {
        let processor = NikonEncryptedDataProcessor;
        let metadata = processor.get_metadata();

        assert_eq!(metadata.name, "Nikon Encrypted Data Processor");
        assert!(metadata
            .supported_manufacturers
            .contains(&"NIKON CORPORATION".to_string()));
        assert!(metadata
            .supported_manufacturers
            .contains(&"NIKON".to_string()));
        assert!(metadata
            .required_context
            .contains(&"manufacturer".to_string()));
    }
}
