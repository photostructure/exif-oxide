//! Rust Code Generator from PPI Structures
//!
//! Converts PPI AST nodes into Rust source code that calls runtime support
//! functions from the `ast::` module for DRY code generation.
//!
//! Trust ExifTool: Generated code preserves exact Perl evaluation semantics.

// Module declarations
pub mod errors;
pub mod expressions;
pub mod functions;
pub mod generator;
pub mod pattern_matching;
pub mod signature;
pub mod visitor;
pub mod visitor_advanced;
pub mod visitor_tokens;

#[cfg(test)]
pub mod tests;

// Re-export everything for backward compatibility
pub use errors::CodeGenError;
pub use expressions::{
    BinaryOperationsHandler, ComplexPatternHandler, ExpressionCombiner, NormalizedAstHandler,
    StringOperationsHandler,
};
pub use functions::FunctionGenerator;
pub use generator::RustGenerator;
pub use visitor::PpiVisitor;

// Import types
use crate::ppi::types::*;

// Implement traits by delegating to the generator module
impl PpiVisitor for RustGenerator {
    fn expression_type(&self) -> &ExpressionType {
        &self.expression_type
    }

    fn visit_document(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        generator::RustGenerator::visit_document(self, node)
    }

    fn visit_statement(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        generator::RustGenerator::visit_statement(self, node)
    }

    fn visit_expression(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        generator::RustGenerator::visit_expression(self, node)
    }
}

impl ExpressionCombiner for RustGenerator {
    fn expression_type(&self) -> &ExpressionType {
        &self.expression_type
    }
}

impl BinaryOperationsHandler for RustGenerator {
    fn expression_type(&self) -> &ExpressionType {
        &self.expression_type
    }

    fn handle_regex_operation(
        &self,
        left: &str,
        op: &str,
        right: &str,
    ) -> Result<String, CodeGenError> {
        StringOperationsHandler::handle_regex_operation(self, left, op, right)
    }
}

impl StringOperationsHandler for RustGenerator {
    fn expression_type(&self) -> &ExpressionType {
        &self.expression_type
    }

    fn combine_statement_parts(
        &self,
        parts: &[String],
        children: &[PpiNode],
    ) -> Result<String, CodeGenError> {
        ExpressionCombiner::combine_statement_parts(self, parts, children)
    }

    fn process_function_args(&self, children: &[PpiNode]) -> Result<Vec<String>, CodeGenError> {
        NormalizedAstHandler::process_function_args(self, children)
    }
}

impl NormalizedAstHandler for RustGenerator {
    fn expression_type(&self) -> &ExpressionType {
        &self.expression_type
    }

    fn combine_statement_parts(
        &self,
        parts: &[String],
        children: &[PpiNode],
    ) -> Result<String, CodeGenError> {
        ExpressionCombiner::combine_statement_parts(self, parts, children)
    }
}

impl ComplexPatternHandler for RustGenerator {
    fn expression_type(&self) -> &ExpressionType {
        &self.expression_type
    }
}

impl FunctionGenerator for RustGenerator {}
