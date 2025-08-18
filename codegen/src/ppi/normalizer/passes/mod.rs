//! Individual normalization passes for AST transformation
//!
//! Each pass handles a specific type of normalization pattern.
//! Pass ordering is critical - see main normalizer module for details.

mod conditional_statements;
mod function_calls;
mod join_unpack;
mod safe_division;
mod sneaky_conditional_assignment;
mod string_ops;
mod ternary;

pub use conditional_statements::ConditionalStatementsNormalizer;
pub use function_calls::FunctionCallNormalizer;
pub use join_unpack::JoinUnpackPass;
pub use safe_division::SafeDivisionNormalizer;
pub use sneaky_conditional_assignment::SneakyConditionalAssignmentNormalizer;
pub use string_ops::StringOpNormalizer;
pub use ternary::TernaryNormalizer;

#[cfg(test)]
mod tests;
