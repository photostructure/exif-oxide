//! Dispatch rules for sophisticated processor selection
//!
//! This module implements the dispatch rule system that enables sophisticated
//! processor selection logic beyond simple capability assessment. It captures
//! ExifTool's conditional dispatch patterns in a structured, extensible way.

use std::sync::Arc;
use tracing::debug;

use super::{BinaryDataProcessor, ProcessorCapability, ProcessorContext, ProcessorKey};

/// Trait for dispatch rules that influence processor selection
///
/// Dispatch rules provide sophisticated logic for processor selection that
/// goes beyond simple capability assessment. They implement ExifTool's
/// conditional dispatch patterns found in manufacturer modules.
///
/// ## ExifTool Reference
///
/// ExifTool uses various conditional dispatch patterns:
/// ```perl
/// # Canon.pm conditional dispatch
/// {
///     Condition => '$$self{Model} =~ /EOS R5/',
///     SubDirectory => { ProcessProc => \&ProcessCanonSerialDataMkII }
/// },
/// {
///     Condition => '$$self{Model} =~ /EOS.*Mark/',
///     SubDirectory => { ProcessProc => \&ProcessCanonSerialData }
/// }
/// ```
///
/// This trait system captures these patterns in a structured way.
pub trait DispatchRule: Send + Sync {
    /// Check if this rule applies to the given context
    ///
    /// Returns true if this rule should be considered for processor selection
    /// in the current context. This allows rules to scope themselves to
    /// specific manufacturers, file types, or other conditions.
    fn applies_to(&self, context: &ProcessorContext) -> bool;

    /// Select a processor from the available candidates
    ///
    /// Given a list of compatible processors, this rule can select the most
    /// appropriate one based on its specific logic. Returns None if the rule
    /// doesn't want to make a selection.
    ///
    /// The candidates are provided as (key, processor, capability) tuples.
    fn select_processor(
        &self,
        candidates: &[(
            ProcessorKey,
            Arc<dyn BinaryDataProcessor>,
            ProcessorCapability,
        )],
        context: &ProcessorContext,
    ) -> Option<(ProcessorKey, Arc<dyn BinaryDataProcessor>)>;

    /// Get a human-readable description of this rule
    fn description(&self) -> &str;

    /// Get the priority of this rule (higher priority rules are evaluated first)
    fn priority(&self) -> u8 {
        50 // Default medium priority
    }
}

/// Canon-specific dispatch rules
///
/// Implements Canon's processor selection logic including model-specific
/// variants and conditional processor selection.
///
/// ## ExifTool Reference
///
/// Based on Canon.pm dispatch patterns and conditional processing.
pub struct CanonDispatchRule;

impl DispatchRule for CanonDispatchRule {
    fn applies_to(&self, context: &ProcessorContext) -> bool {
        context.is_manufacturer("Canon")
    }

    fn select_processor(
        &self,
        candidates: &[(
            ProcessorKey,
            Arc<dyn BinaryDataProcessor>,
            ProcessorCapability,
        )],
        context: &ProcessorContext,
    ) -> Option<(ProcessorKey, Arc<dyn BinaryDataProcessor>)> {
        debug!(
            "Applying Canon dispatch rule for table: {}",
            context.table_name
        );

        // Canon-specific processor selection logic based on ExifTool Canon.pm
        match context.table_name.as_str() {
            "Canon::SerialData" => {
                // Check for newer Canon models that use enhanced serial data format
                if let Some(model) = &context.model {
                    if model.contains("EOS R5")
                        || model.contains("EOS R6")
                        || model.contains("EOS R3")
                    {
                        if let Some(processor) = self.find_processor_variant(
                            candidates,
                            "Canon",
                            "SerialData",
                            Some("MkII"),
                        ) {
                            debug!(
                                "Selected Canon SerialData MkII processor for model: {}",
                                model
                            );
                            return Some(processor);
                        }
                    }
                }

                // Fall back to standard Canon serial data processor
                self.find_processor_variant(candidates, "Canon", "SerialData", None)
            }

            "Canon::AFInfo" | "Canon::AFInfo2" => {
                // Different AF info processors for different camera generations
                if let Some(af_info_version) = context.get_parent_tag("AFInfoVersion") {
                    match af_info_version.as_u16() {
                        Some(0x0001) => {
                            self.find_processor_variant(candidates, "Canon", "AFInfo", Some("V1"))
                        }
                        Some(0x0002) => {
                            self.find_processor_variant(candidates, "Canon", "AFInfo", Some("V2"))
                        }
                        Some(0x0003) => {
                            self.find_processor_variant(candidates, "Canon", "AFInfo", Some("V3"))
                        }
                        _ => self.find_processor_variant(candidates, "Canon", "AFInfo", None),
                    }
                } else {
                    // Use model-based selection for AF info
                    if let Some(model) = &context.model {
                        if model.contains("1D") {
                            self.find_processor_variant(candidates, "Canon", "AFInfo", Some("1D"))
                        } else if model.contains("5D") {
                            self.find_processor_variant(candidates, "Canon", "AFInfo", Some("5D"))
                        } else {
                            self.find_processor_variant(candidates, "Canon", "AFInfo", None)
                        }
                    } else {
                        self.find_processor_variant(candidates, "Canon", "AFInfo", None)
                    }
                }
            }

            "Canon::CameraSettings" => {
                // Camera settings processing - check for format-specific processors
                if let Some(format_version) = &context.format_version {
                    self.find_processor_variant(
                        candidates,
                        "Canon",
                        "CameraSettings",
                        Some(format_version),
                    )
                } else {
                    self.find_processor_variant(candidates, "Canon", "CameraSettings", None)
                }
            }

            _ => {
                // For other Canon tables, prefer Canon namespace processors
                candidates
                    .iter()
                    .find(|(key, _, _)| key.namespace == "Canon")
                    .map(|(key, processor, _)| (key.clone(), processor.clone()))
            }
        }
    }

    fn description(&self) -> &str {
        "Canon manufacturer-specific processor dispatch"
    }

    fn priority(&self) -> u8 {
        80 // High priority for manufacturer-specific rules
    }
}

impl CanonDispatchRule {
    /// Find a processor variant by namespace, name, and optional variant
    fn find_processor_variant(
        &self,
        candidates: &[(
            ProcessorKey,
            Arc<dyn BinaryDataProcessor>,
            ProcessorCapability,
        )],
        namespace: &str,
        processor_name: &str,
        variant: Option<&str>,
    ) -> Option<(ProcessorKey, Arc<dyn BinaryDataProcessor>)> {
        candidates
            .iter()
            .find(|(key, _, _)| {
                key.namespace == namespace
                    && key.processor_name == processor_name
                    && key.variant.as_deref() == variant
            })
            .map(|(key, processor, _)| (key.clone(), processor.clone()))
    }
}

/// Nikon-specific dispatch rules
///
/// Implements Nikon's processor selection logic including encryption detection
/// and model-specific processing variants.
///
/// ## ExifTool Reference
///
/// Based on Nikon.pm dispatch patterns and ProcessNikonEncrypted conditions.
pub struct NikonDispatchRule;

impl DispatchRule for NikonDispatchRule {
    fn applies_to(&self, context: &ProcessorContext) -> bool {
        context.is_manufacturer("NIKON CORPORATION") || context.is_manufacturer("NIKON")
    }

    fn select_processor(
        &self,
        candidates: &[(
            ProcessorKey,
            Arc<dyn BinaryDataProcessor>,
            ProcessorCapability,
        )],
        context: &ProcessorContext,
    ) -> Option<(ProcessorKey, Arc<dyn BinaryDataProcessor>)> {
        debug!(
            "Applying Nikon dispatch rule for table: {}",
            context.table_name
        );

        // Nikon-specific processor selection logic based on ExifTool Nikon.pm
        match context.table_name.as_str() {
            table_name if table_name.contains("LensData") => {
                // Check for encrypted lens data
                if context.parameters.contains_key("DecryptStart") {
                    debug!("Detected encrypted Nikon lens data");
                    self.find_processor_variant(candidates, "Nikon", "Encrypted", None)
                } else {
                    self.find_processor_variant(candidates, "Nikon", "LensData", None)
                }
            }

            "Nikon::ShotInfo" => {
                // Model-specific shot info processors
                if let Some(model) = &context.model {
                    if model.contains("Z 9") {
                        self.find_processor_variant(candidates, "Nikon", "ShotInfo", Some("Z9"))
                    } else if model.contains("Z 8") {
                        self.find_processor_variant(candidates, "Nikon", "ShotInfo", Some("Z8"))
                    } else if model.contains("Z 6III") {
                        self.find_processor_variant(candidates, "Nikon", "ShotInfo", Some("Z6III"))
                    } else if model.contains("Z 7II") {
                        self.find_processor_variant(candidates, "Nikon", "ShotInfo", Some("Z7II"))
                    } else {
                        self.find_processor_variant(candidates, "Nikon", "ShotInfo", None)
                    }
                } else {
                    self.find_processor_variant(candidates, "Nikon", "ShotInfo", None)
                }
            }

            "Nikon::ColorBalance" => {
                // Version-specific color balance processors
                if let Some(version) = context.get_parameter("Version") {
                    self.find_processor_variant(candidates, "Nikon", "ColorBalance", Some(version))
                } else {
                    self.find_processor_variant(candidates, "Nikon", "ColorBalance", None)
                }
            }

            _ => {
                // Check if this might be encrypted data based on context
                if self.has_encryption_keys(context) && self.might_be_encrypted(context) {
                    debug!("Trying encrypted processor for: {}", context.table_name);
                    if let Some(encrypted_processor) =
                        self.find_processor_variant(candidates, "Nikon", "Encrypted", None)
                    {
                        return Some(encrypted_processor);
                    }
                }

                // Default to any Nikon processor
                candidates
                    .iter()
                    .find(|(key, _, _)| key.namespace == "Nikon")
                    .map(|(key, processor, _)| (key.clone(), processor.clone()))
            }
        }
    }

    fn description(&self) -> &str {
        "Nikon manufacturer-specific processor dispatch with encryption detection"
    }

    fn priority(&self) -> u8 {
        80 // High priority for manufacturer-specific rules
    }
}

impl NikonDispatchRule {
    /// Find a processor variant by namespace, name, and optional variant
    fn find_processor_variant(
        &self,
        candidates: &[(
            ProcessorKey,
            Arc<dyn BinaryDataProcessor>,
            ProcessorCapability,
        )],
        namespace: &str,
        processor_name: &str,
        variant: Option<&str>,
    ) -> Option<(ProcessorKey, Arc<dyn BinaryDataProcessor>)> {
        candidates
            .iter()
            .find(|(key, _, _)| {
                key.namespace == namespace
                    && key.processor_name == processor_name
                    && key.variant.as_deref() == variant
            })
            .map(|(key, processor, _)| (key.clone(), processor.clone()))
    }

    /// Check if encryption keys are available
    fn has_encryption_keys(&self, context: &ProcessorContext) -> bool {
        context.get_nikon_encryption_keys().is_some()
    }

    /// Check if the current context might contain encrypted data
    fn might_be_encrypted(&self, context: &ProcessorContext) -> bool {
        // Check for encryption-related parameters
        context.parameters.contains_key("DecryptStart")
            || context.parameters.contains_key("DecryptLen")
            || context.table_name.contains("Encrypted")
    }
}

/// Format-specific dispatch rule
///
/// Selects processors based on file format and format-specific requirements.
/// This rule handles cases where processor selection depends on the file
/// format rather than manufacturer.
pub struct FormatDispatchRule;

impl DispatchRule for FormatDispatchRule {
    fn applies_to(&self, _context: &ProcessorContext) -> bool {
        true // This rule applies to all contexts
    }

    fn select_processor(
        &self,
        candidates: &[(
            ProcessorKey,
            Arc<dyn BinaryDataProcessor>,
            ProcessorCapability,
        )],
        context: &ProcessorContext,
    ) -> Option<(ProcessorKey, Arc<dyn BinaryDataProcessor>)> {
        // Format-specific processor selection
        match context.file_format {
            crate::formats::FileFormat::Tiff => {
                // Prefer TIFF-specific processors
                candidates
                    .iter()
                    .find(|(key, _, _)| key.processor_name.contains("TIFF"))
                    .map(|(key, processor, _)| (key.clone(), processor.clone()))
            }
            crate::formats::FileFormat::CanonRaw | crate::formats::FileFormat::NikonRaw => {
                // Prefer RAW-specific processors
                candidates
                    .iter()
                    .find(|(key, _, _)| key.processor_name.contains("RAW"))
                    .map(|(key, processor, _)| (key.clone(), processor.clone()))
            }
            _ => None, // Let other rules handle
        }
    }

    fn description(&self) -> &str {
        "Format-specific processor dispatch"
    }

    fn priority(&self) -> u8 {
        30 // Lower priority than manufacturer rules
    }
}

/// Table-specific dispatch rule
///
/// Selects processors based on table name patterns and conventions.
/// This rule implements ExifTool's table-specific processor associations.
pub struct TableDispatchRule;

impl DispatchRule for TableDispatchRule {
    fn applies_to(&self, _context: &ProcessorContext) -> bool {
        true // This rule applies to all contexts
    }

    fn select_processor(
        &self,
        candidates: &[(
            ProcessorKey,
            Arc<dyn BinaryDataProcessor>,
            ProcessorCapability,
        )],
        context: &ProcessorContext,
    ) -> Option<(ProcessorKey, Arc<dyn BinaryDataProcessor>)> {
        // Table name-based processor selection
        if context.table_name.contains("BinaryData") {
            // Prefer binary data processors
            return candidates
                .iter()
                .find(|(key, _, _)| key.processor_name.contains("BinaryData"))
                .map(|(key, processor, _)| (key.clone(), processor.clone()));
        }

        if context.table_name.contains("SerialData") {
            // Prefer serial data processors
            return candidates
                .iter()
                .find(|(key, _, _)| key.processor_name.contains("SerialData"))
                .map(|(key, processor, _)| (key.clone(), processor.clone()));
        }

        if context.table_name.contains("AFInfo") {
            // Prefer AF info processors
            return candidates
                .iter()
                .find(|(key, _, _)| key.processor_name.contains("AFInfo"))
                .map(|(key, processor, _)| (key.clone(), processor.clone()));
        }

        None // No specific preference
    }

    fn description(&self) -> &str {
        "Table name-based processor dispatch"
    }

    fn priority(&self) -> u8 {
        40 // Medium priority
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::FileFormat;
    use crate::processor_registry::{ProcessorMetadata, ProcessorResult};

    struct MockProcessor;
    impl BinaryDataProcessor for MockProcessor {
        fn can_process(&self, _context: &ProcessorContext) -> ProcessorCapability {
            ProcessorCapability::Good
        }
        fn process_data(
            &self,
            _data: &[u8],
            _context: &ProcessorContext,
        ) -> crate::types::Result<ProcessorResult> {
            Ok(ProcessorResult::new())
        }
        fn get_metadata(&self) -> ProcessorMetadata {
            ProcessorMetadata::new("Mock".to_string(), "Mock processor".to_string())
        }
    }

    #[test]
    fn test_canon_dispatch_rule() {
        let rule = CanonDispatchRule;
        let canon_context =
            ProcessorContext::new(FileFormat::Jpeg, "Canon::SerialData".to_string())
                .with_manufacturer("Canon".to_string())
                .with_model("EOS R5".to_string());

        assert!(rule.applies_to(&canon_context));

        // Test with R5 model - should prefer MkII variant
        let candidates = vec![
            (
                ProcessorKey::new("Canon".to_string(), "SerialData".to_string()),
                Arc::new(MockProcessor) as Arc<dyn BinaryDataProcessor>,
                ProcessorCapability::Good,
            ),
            (
                ProcessorKey::with_variant(
                    "Canon".to_string(),
                    "SerialData".to_string(),
                    "MkII".to_string(),
                ),
                Arc::new(MockProcessor) as Arc<dyn BinaryDataProcessor>,
                ProcessorCapability::Good,
            ),
        ];

        let selected = rule.select_processor(&candidates, &canon_context);
        assert!(selected.is_some());
        let (key, _) = selected.unwrap();
        assert_eq!(key.variant, Some("MkII".to_string()));
    }

    #[test]
    fn test_nikon_dispatch_rule() {
        let rule = NikonDispatchRule;
        let nikon_context = ProcessorContext::new(FileFormat::Jpeg, "Nikon::LensData".to_string())
            .with_manufacturer("NIKON CORPORATION".to_string())
            .with_parameters({
                let mut params = std::collections::HashMap::new();
                params.insert("DecryptStart".to_string(), "4".to_string());
                params
            });

        assert!(rule.applies_to(&nikon_context));

        let candidates = vec![
            (
                ProcessorKey::new("Nikon".to_string(), "LensData".to_string()),
                Arc::new(MockProcessor) as Arc<dyn BinaryDataProcessor>,
                ProcessorCapability::Good,
            ),
            (
                ProcessorKey::new("Nikon".to_string(), "Encrypted".to_string()),
                Arc::new(MockProcessor) as Arc<dyn BinaryDataProcessor>,
                ProcessorCapability::Good,
            ),
        ];

        let selected = rule.select_processor(&candidates, &nikon_context);
        assert!(selected.is_some());
        let (key, _) = selected.unwrap();
        assert_eq!(key.processor_name, "Encrypted");
    }

    #[test]
    fn test_format_dispatch_rule() {
        let rule = FormatDispatchRule;
        let tiff_context = ProcessorContext::new(FileFormat::Tiff, "EXIF::Main".to_string());

        assert!(rule.applies_to(&tiff_context));

        let candidates = vec![
            (
                ProcessorKey::new("EXIF".to_string(), "Main".to_string()),
                Arc::new(MockProcessor) as Arc<dyn BinaryDataProcessor>,
                ProcessorCapability::Good,
            ),
            (
                ProcessorKey::new("EXIF".to_string(), "TIFF".to_string()),
                Arc::new(MockProcessor) as Arc<dyn BinaryDataProcessor>,
                ProcessorCapability::Good,
            ),
        ];

        let selected = rule.select_processor(&candidates, &tiff_context);
        assert!(selected.is_some());
        let (key, _) = selected.unwrap();
        assert!(key.processor_name.contains("TIFF"));
    }

    #[test]
    fn test_table_dispatch_rule() {
        let rule = TableDispatchRule;
        let binary_context =
            ProcessorContext::new(FileFormat::Jpeg, "Canon::BinaryData".to_string());

        assert!(rule.applies_to(&binary_context));

        let candidates = vec![
            (
                ProcessorKey::new("Canon".to_string(), "Main".to_string()),
                Arc::new(MockProcessor) as Arc<dyn BinaryDataProcessor>,
                ProcessorCapability::Good,
            ),
            (
                ProcessorKey::new("Canon".to_string(), "BinaryData".to_string()),
                Arc::new(MockProcessor) as Arc<dyn BinaryDataProcessor>,
                ProcessorCapability::Good,
            ),
        ];

        let selected = rule.select_processor(&candidates, &binary_context);
        assert!(selected.is_some());
        let (key, _) = selected.unwrap();
        assert!(key.processor_name.contains("BinaryData"));
    }
}
