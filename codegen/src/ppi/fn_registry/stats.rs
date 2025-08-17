//! Statistics tracking for PPI function registry
//!
//! This module handles tracking conversion processing success rates
//! and registry statistics for monitoring and debugging purposes.

use crate::ppi::ExpressionType;

/// Statistics for tracking conversion processing success rates
#[derive(Debug, Default, Clone)]
pub struct ConversionStats {
    /// PrintConv attempts and successes
    pub print_conv_attempts: usize,
    pub print_conv_successes: usize,
    /// ValueConv attempts and successes  
    pub value_conv_attempts: usize,
    pub value_conv_successes: usize,
    /// Condition attempts and successes
    pub condition_attempts: usize,
    pub condition_successes: usize,
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

    /// Record a successful conversion processing
    pub fn record_success(&mut self, expression_type: ExpressionType) {
        match expression_type {
            ExpressionType::PrintConv => self.print_conv_successes += 1,
            ExpressionType::ValueConv => self.value_conv_successes += 1,
            ExpressionType::Condition => self.condition_successes += 1,
        }
    }

    /// Calculate success rate for a given expression type
    pub fn success_rate(&self, expression_type: ExpressionType) -> f64 {
        let (attempts, successes) = match expression_type {
            ExpressionType::PrintConv => (self.print_conv_attempts, self.print_conv_successes),
            ExpressionType::ValueConv => (self.value_conv_attempts, self.value_conv_successes),
            ExpressionType::Condition => (self.condition_attempts, self.condition_successes),
        };

        if attempts == 0 {
            0.0
        } else {
            (successes as f64 / attempts as f64) * 100.0
        }
    }
}

/// Registry statistics for overall performance monitoring
#[derive(Debug, Clone)]
pub struct RegistryStats {
    /// Total number of unique functions registered
    pub total_functions: usize,
    /// Total number of AST registrations (including duplicates)
    pub total_registrations: usize,
    /// Number of deduplicated cases (total_registrations - total_functions)
    pub deduplicated_count: usize,
    /// Conversion processing statistics
    pub conversion_stats: ConversionStats,
}

impl RegistryStats {
    /// Calculate deduplication percentage
    pub fn deduplication_percentage(&self) -> f64 {
        if self.total_registrations == 0 {
            0.0
        } else {
            (self.deduplicated_count as f64 / self.total_registrations as f64) * 100.0
        }
    }
}
