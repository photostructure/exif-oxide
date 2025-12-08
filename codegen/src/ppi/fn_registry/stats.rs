//! Statistics tracking for PPI function registry
//!
//! This module handles tracking conversion processing success rates
//! and registry statistics for monitoring and debugging purposes.

use crate::ppi::ExpressionType;

/// Statistics for tracking conversion processing success rates
#[derive(Debug, Default, Clone)]
pub struct ConversionStats {
    /// PrintConv attempts and outcome tracking
    pub print_conv_attempts: usize,
    pub print_conv_ppi_successes: usize,
    pub print_conv_registry_successes: usize,
    pub print_conv_placeholder_fallbacks: usize,
    /// ValueConv attempts and outcome tracking
    pub value_conv_attempts: usize,
    pub value_conv_ppi_successes: usize,
    pub value_conv_registry_successes: usize,
    pub value_conv_placeholder_fallbacks: usize,
    /// Condition attempts and outcome tracking
    pub condition_attempts: usize,
    pub condition_ppi_successes: usize,
    pub condition_registry_successes: usize,
    pub condition_placeholder_fallbacks: usize,
}

impl ConversionStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an attempt to process a conversion
    pub fn record_attempt(&mut self, expression_type: ExpressionType) {
        match expression_type {
            ExpressionType::PrintConv => self.print_conv_attempts += 1,
            ExpressionType::ValueConv => self.value_conv_attempts += 1,
            ExpressionType::Condition => self.condition_attempts += 1,
        }
    }

    /// Record a successful PPI conversion processing
    pub fn record_ppi_success(&mut self, expression_type: ExpressionType) {
        match expression_type {
            ExpressionType::PrintConv => self.print_conv_ppi_successes += 1,
            ExpressionType::ValueConv => self.value_conv_ppi_successes += 1,
            ExpressionType::Condition => self.condition_ppi_successes += 1,
        }
    }

    /// Record a successful registry fallback
    pub fn record_registry_success(&mut self, expression_type: ExpressionType) {
        match expression_type {
            ExpressionType::PrintConv => self.print_conv_registry_successes += 1,
            ExpressionType::ValueConv => self.value_conv_registry_successes += 1,
            ExpressionType::Condition => self.condition_registry_successes += 1,
        }
    }

    /// Record a placeholder fallback (final fallback)
    pub fn record_placeholder_fallback(&mut self, expression_type: ExpressionType) {
        match expression_type {
            ExpressionType::PrintConv => self.print_conv_placeholder_fallbacks += 1,
            ExpressionType::ValueConv => self.value_conv_placeholder_fallbacks += 1,
            ExpressionType::Condition => self.condition_placeholder_fallbacks += 1,
        }
    }

    /// Record a successful conversion processing (legacy method for backward compatibility)
    #[allow(dead_code)]
    pub fn record_success(&mut self, expression_type: ExpressionType) {
        self.record_ppi_success(expression_type);
    }

    /// Calculate PPI success rate for a given expression type
    #[allow(dead_code)]
    pub fn ppi_success_rate(&self, expression_type: ExpressionType) -> f64 {
        let (attempts, successes) = match expression_type {
            ExpressionType::PrintConv => (self.print_conv_attempts, self.print_conv_ppi_successes),
            ExpressionType::ValueConv => (self.value_conv_attempts, self.value_conv_ppi_successes),
            ExpressionType::Condition => (self.condition_attempts, self.condition_ppi_successes),
        };

        if attempts == 0 {
            0.0
        } else {
            (successes as f64 / attempts as f64) * 100.0
        }
    }

    /// Calculate registry fallback success rate for a given expression type
    #[allow(dead_code)]
    pub fn registry_success_rate(&self, expression_type: ExpressionType) -> f64 {
        let (attempts, successes) = match expression_type {
            ExpressionType::PrintConv => {
                (self.print_conv_attempts, self.print_conv_registry_successes)
            }
            ExpressionType::ValueConv => {
                (self.value_conv_attempts, self.value_conv_registry_successes)
            }
            ExpressionType::Condition => {
                (self.condition_attempts, self.condition_registry_successes)
            }
        };

        if attempts == 0 {
            0.0
        } else {
            (successes as f64 / attempts as f64) * 100.0
        }
    }

    /// Calculate total success rate (PPI + registry) for a given expression type
    #[allow(dead_code)]
    pub fn total_success_rate(&self, expression_type: ExpressionType) -> f64 {
        let (attempts, ppi_successes, registry_successes) = match expression_type {
            ExpressionType::PrintConv => (
                self.print_conv_attempts,
                self.print_conv_ppi_successes,
                self.print_conv_registry_successes,
            ),
            ExpressionType::ValueConv => (
                self.value_conv_attempts,
                self.value_conv_ppi_successes,
                self.value_conv_registry_successes,
            ),
            ExpressionType::Condition => (
                self.condition_attempts,
                self.condition_ppi_successes,
                self.condition_registry_successes,
            ),
        };

        if attempts == 0 {
            0.0
        } else {
            ((ppi_successes + registry_successes) as f64 / attempts as f64) * 100.0
        }
    }

    /// Calculate success rate for a given expression type (legacy method for backward compatibility)
    pub fn success_rate(&self, expression_type: ExpressionType) -> f64 {
        self.ppi_success_rate(expression_type)
    }
}

/// Registry statistics for overall performance monitoring
#[derive(Debug, Clone)]
pub struct RegistryStats {
    /// Conversion processing statistics
    pub conversion_stats: ConversionStats,
}
