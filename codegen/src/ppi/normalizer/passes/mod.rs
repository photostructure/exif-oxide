//! Individual normalization passes for AST transformation
//!
//! Each pass handles a specific type of normalization pattern.
//! Pass ordering is critical - see main normalizer module for details.

mod function_calls;
mod nested_functions;
mod postfix_conditional;
mod safe_division;
mod sneaky_conditional_assignment;
mod sprintf;
mod string_ops;
mod ternary;

pub use function_calls::FunctionCallNormalizer;
pub use nested_functions::NestedFunctionNormalizer;
pub use postfix_conditional::PostfixConditionalNormalizer;
pub use safe_division::SafeDivisionNormalizer;
pub use sneaky_conditional_assignment::SneakyConditionalAssignmentNormalizer;
pub use sprintf::SprintfNormalizer;
pub use string_ops::StringOpNormalizer;
pub use ternary::TernaryNormalizer;

#[cfg(test)]
mod tests;
