//! ProcessorCapability assessment system
//!
//! This module provides the capability assessment framework that enables the
//! registry to select the most appropriate processor for given data and context.

/// Assessment of a processor's capability to handle specific data
///
/// This enum provides a nuanced way for processors to indicate how well they
/// can handle particular data, enabling the registry to make optimal selections
/// while maintaining fallback options.
///
/// ## ExifTool Reference
///
/// ExifTool uses conditional expressions to determine processor applicability:
/// ```perl
/// {
///     Condition => '$$self{Model} =~ /EOS R5/',  # Perfect match
///     SubDirectory => { ProcessProc => \&ProcessCanonSerialDataMkII }
/// },
/// {
///     Condition => '$$self{Make} eq "Canon"',    # Good match
///     SubDirectory => { ProcessProc => \&ProcessCanonSerialData }
/// }
/// ```
///
/// This enum captures these preference levels explicitly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessorCapability {
    /// Exact match for this data - highest priority
    ///
    /// Use this when the processor is specifically designed for the exact
    /// combination of manufacturer, model, format version, etc.
    ///
    /// Example: Canon EOS R5 serial data processor when processing EOS R5 data
    Perfect,

    /// Compatible and good choice - second priority
    ///
    /// Use this when the processor is designed for this manufacturer/format
    /// but may not be the most specific option available.
    ///
    /// Example: Generic Canon processor when processing Canon data
    Good,

    /// Can handle but not optimal - third priority
    ///
    /// Use this when the processor can handle the data but it's not its
    /// primary purpose. Often used by generic processors as fallbacks.
    ///
    /// Example: Generic EXIF processor when processing manufacturer data
    Fallback,

    /// Cannot process this data - will be filtered out
    ///
    /// Use this when the processor cannot handle the data due to format
    /// incompatibility, missing required context, etc.
    Incompatible,
}

impl PartialOrd for ProcessorCapability {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ProcessorCapability {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority values are "greater"
        self.priority_score().cmp(&other.priority_score())
    }
}

impl ProcessorCapability {
    /// Check if this capability indicates the processor can handle the data
    pub fn is_compatible(&self) -> bool {
        match self {
            ProcessorCapability::Perfect => true,
            ProcessorCapability::Good => true,
            ProcessorCapability::Fallback => true,
            ProcessorCapability::Incompatible => false,
        }
    }

    /// Get priority score for sorting (higher is better)
    pub fn priority_score(&self) -> u8 {
        match self {
            ProcessorCapability::Perfect => 100,
            ProcessorCapability::Good => 75,
            ProcessorCapability::Fallback => 25,
            ProcessorCapability::Incompatible => 0,
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            ProcessorCapability::Perfect => "Perfect match - optimal processor for this data",
            ProcessorCapability::Good => "Good match - compatible and recommended",
            ProcessorCapability::Fallback => "Fallback option - can process but not optimal",
            ProcessorCapability::Incompatible => "Incompatible - cannot process this data",
        }
    }

    /// Combine multiple capabilities to get overall assessment
    ///
    /// This is useful when a processor evaluates multiple criteria and needs
    /// to provide an overall capability assessment.
    pub fn combine(capabilities: &[ProcessorCapability]) -> ProcessorCapability {
        if capabilities.is_empty() {
            return ProcessorCapability::Incompatible;
        }

        // If any are incompatible, the whole assessment is incompatible
        if capabilities.contains(&ProcessorCapability::Incompatible) {
            return ProcessorCapability::Incompatible;
        }

        // Return the lowest (worst) capability that's still compatible
        capabilities
            .iter()
            .min_by_key(|c| c.priority_score())
            .cloned()
            .unwrap_or(ProcessorCapability::Incompatible)
    }

    /// Create capability based on boolean conditions
    ///
    /// Helper for simple processors that just need to check if they're compatible.
    pub fn from_boolean(is_compatible: bool) -> ProcessorCapability {
        if is_compatible {
            ProcessorCapability::Good
        } else {
            ProcessorCapability::Incompatible
        }
    }

    /// Create capability with manufacturer and model specificity
    ///
    /// Helper for manufacturer-specific processors that want to indicate
    /// different capability levels based on specificity.
    pub fn from_specificity(
        manufacturer_matches: bool,
        model_matches: bool,
        is_primary_purpose: bool,
    ) -> ProcessorCapability {
        if !manufacturer_matches {
            return ProcessorCapability::Incompatible;
        }

        match (model_matches, is_primary_purpose) {
            (true, true) => ProcessorCapability::Perfect,
            (true, false) => ProcessorCapability::Good,
            (false, true) => ProcessorCapability::Good,
            (false, false) => ProcessorCapability::Fallback,
        }
    }
}

/// Assessment details for debugging and analysis
///
/// This structure provides detailed information about why a processor
/// returned a particular capability assessment, useful for debugging
/// processor selection decisions.
#[derive(Debug, Clone)]
pub struct CapabilityAssessment {
    /// The capability level assessed
    pub capability: ProcessorCapability,

    /// Explanation of why this capability was assigned
    pub reason: String,

    /// Factors that contributed to the assessment
    pub factors: Vec<CapabilityFactor>,

    /// Missing requirements that prevented higher capability
    pub missing_requirements: Vec<String>,
}

impl CapabilityAssessment {
    /// Create a new capability assessment
    pub fn new(capability: ProcessorCapability, reason: String) -> Self {
        Self {
            capability,
            reason,
            factors: Vec::new(),
            missing_requirements: Vec::new(),
        }
    }

    /// Add a contributing factor
    pub fn add_factor(mut self, factor: CapabilityFactor) -> Self {
        self.factors.push(factor);
        self
    }

    /// Add a missing requirement
    pub fn add_missing_requirement(mut self, requirement: String) -> Self {
        self.missing_requirements.push(requirement);
        self
    }

    /// Get a detailed explanation including all factors
    pub fn detailed_explanation(&self) -> String {
        let mut explanation = format!("{}: {}", self.capability.description(), self.reason);

        if !self.factors.is_empty() {
            explanation.push_str("\nFactors:");
            for factor in &self.factors {
                explanation.push_str(&format!("\n  - {}", factor.description()));
            }
        }

        if !self.missing_requirements.is_empty() {
            explanation.push_str("\nMissing requirements:");
            for req in &self.missing_requirements {
                explanation.push_str(&format!("\n  - {req}"));
            }
        }

        explanation
    }
}

/// Individual factors that contribute to capability assessment
///
/// These represent specific checks or conditions that a processor
/// evaluates when determining its capability level.
#[derive(Debug, Clone)]
pub enum CapabilityFactor {
    /// Manufacturer matches processor's target
    ManufacturerMatch(String),

    /// Model matches processor's target
    ModelMatch(String),

    /// Format version is supported
    FormatVersionSupported(String),

    /// Required context field is available
    RequiredContextAvailable(String),

    /// Table name matches processor's scope
    TableNameMatch(String),

    /// Data pattern matches expected format
    DataPatternMatch(String),

    /// Custom condition result
    CustomCondition(String, bool),
}

impl CapabilityFactor {
    /// Get human-readable description of this factor
    pub fn description(&self) -> String {
        match self {
            CapabilityFactor::ManufacturerMatch(manufacturer) => {
                format!("Manufacturer '{manufacturer}' matches processor target")
            }
            CapabilityFactor::ModelMatch(model) => {
                format!("Model '{model}' matches processor target")
            }
            CapabilityFactor::FormatVersionSupported(version) => {
                format!("Format version '{version}' is supported")
            }
            CapabilityFactor::RequiredContextAvailable(field) => {
                format!("Required context field '{field}' is available")
            }
            CapabilityFactor::TableNameMatch(table) => {
                format!("Table name '{table}' matches processor scope")
            }
            CapabilityFactor::DataPatternMatch(pattern) => {
                format!("Data matches expected pattern '{pattern}'")
            }
            CapabilityFactor::CustomCondition(condition, result) => {
                format!(
                    "Custom condition '{}': {}",
                    condition,
                    if *result { "passed" } else { "failed" }
                )
            }
        }
    }

    /// Get the impact this factor has on capability (positive or negative)
    pub fn impact(&self) -> i8 {
        match self {
            CapabilityFactor::ManufacturerMatch(_) => 20,
            CapabilityFactor::ModelMatch(_) => 25,
            CapabilityFactor::FormatVersionSupported(_) => 15,
            CapabilityFactor::RequiredContextAvailable(_) => 10,
            CapabilityFactor::TableNameMatch(_) => 15,
            CapabilityFactor::DataPatternMatch(_) => 30,
            CapabilityFactor::CustomCondition(_, true) => 10,
            CapabilityFactor::CustomCondition(_, false) => -20,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_ordering() {
        assert!(ProcessorCapability::Perfect > ProcessorCapability::Good);
        assert!(ProcessorCapability::Good > ProcessorCapability::Fallback);
        assert!(ProcessorCapability::Fallback > ProcessorCapability::Incompatible);
    }

    #[test]
    fn test_capability_priority_scores() {
        assert_eq!(ProcessorCapability::Perfect.priority_score(), 100);
        assert_eq!(ProcessorCapability::Good.priority_score(), 75);
        assert_eq!(ProcessorCapability::Fallback.priority_score(), 25);
        assert_eq!(ProcessorCapability::Incompatible.priority_score(), 0);
    }

    #[test]
    fn test_capability_compatibility() {
        assert!(ProcessorCapability::Perfect.is_compatible());
        assert!(ProcessorCapability::Good.is_compatible());
        assert!(ProcessorCapability::Fallback.is_compatible());
        assert!(!ProcessorCapability::Incompatible.is_compatible());
    }

    #[test]
    fn test_capability_combination() {
        let capabilities = vec![
            ProcessorCapability::Perfect,
            ProcessorCapability::Good,
            ProcessorCapability::Fallback,
        ];
        assert_eq!(
            ProcessorCapability::combine(&capabilities),
            ProcessorCapability::Fallback
        );

        let capabilities = vec![
            ProcessorCapability::Perfect,
            ProcessorCapability::Incompatible,
        ];
        assert_eq!(
            ProcessorCapability::combine(&capabilities),
            ProcessorCapability::Incompatible
        );

        let capabilities = vec![ProcessorCapability::Perfect, ProcessorCapability::Good];
        assert_eq!(
            ProcessorCapability::combine(&capabilities),
            ProcessorCapability::Good
        );
    }

    #[test]
    fn test_capability_from_specificity() {
        assert_eq!(
            ProcessorCapability::from_specificity(false, false, false),
            ProcessorCapability::Incompatible
        );
        assert_eq!(
            ProcessorCapability::from_specificity(true, true, true),
            ProcessorCapability::Perfect
        );
        assert_eq!(
            ProcessorCapability::from_specificity(true, false, true),
            ProcessorCapability::Good
        );
        assert_eq!(
            ProcessorCapability::from_specificity(true, false, false),
            ProcessorCapability::Fallback
        );
    }

    #[test]
    fn test_capability_assessment() {
        let assessment = CapabilityAssessment::new(
            ProcessorCapability::Good,
            "Manufacturer matches".to_string(),
        )
        .add_factor(CapabilityFactor::ManufacturerMatch("Canon".to_string()))
        .add_missing_requirement("Model information".to_string());

        assert_eq!(assessment.capability, ProcessorCapability::Good);
        assert_eq!(assessment.factors.len(), 1);
        assert_eq!(assessment.missing_requirements.len(), 1);

        let explanation = assessment.detailed_explanation();
        assert!(explanation.contains("Canon"));
        assert!(explanation.contains("Model information"));
    }

    #[test]
    fn test_capability_factor_impact() {
        let factor = CapabilityFactor::ManufacturerMatch("Canon".to_string());
        assert_eq!(factor.impact(), 20);

        let factor = CapabilityFactor::CustomCondition("test".to_string(), false);
        assert_eq!(factor.impact(), -20);
    }
}
