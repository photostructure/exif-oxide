//! Individual normalization passes for AST transformation
//!
//! Each pass handles a specific type of normalization pattern.
//! Pass ordering is critical - see main normalizer module for details.

// CONSOLIDATED NORMALIZERS: Unified precedence climbing approach
mod expression_precedence;

// PRESERVED NORMALIZERS: Structural transformations that cannot be unified via precedence
mod conditional_statements;
mod sneaky_conditional_assignment;

// LEGACY NORMALIZERS: Kept for reference but no longer used in active pipeline
mod binary_operators;
mod function_calls;
mod join_unpack;
mod safe_division;
mod string_ops;
mod ternary;

// ACTIVE EXPORTS: Only the normalizers used in the current pipeline
pub use conditional_statements::ConditionalStatementsNormalizer;
pub use expression_precedence::ExpressionPrecedenceNormalizer;
pub use sneaky_conditional_assignment::SneakyConditionalAssignmentNormalizer;

// LEGACY EXPORTS: Available for debugging/comparison but not used in pipeline
pub use binary_operators::BinaryOperatorNormalizer;
pub use function_calls::FunctionCallNormalizer;
pub use join_unpack::JoinUnpackPass;
pub use safe_division::SafeDivisionNormalizer;
pub use string_ops::StringOpNormalizer;
pub use ternary::TernaryNormalizer;

#[cfg(test)]
mod tests;
