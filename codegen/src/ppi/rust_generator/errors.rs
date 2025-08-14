//! Error types for PPI Rust code generation

/// Error types for code generation
#[derive(Debug, thiserror::Error)]
pub enum CodeGenError {
    #[error("Unsupported AST structure: {0}")]
    UnsupportedStructure(String),

    #[error("Unsupported operator: {0}")]
    #[allow(dead_code)]
    UnsupportedOperator(String),

    #[error("Unsupported function: {0}")]
    UnsupportedFunction(String),

    #[error("Unsupported context: {0}")]
    #[allow(dead_code)]
    UnsupportedContext(String),

    #[error("Unsupported token type: {0}")]
    UnsupportedToken(String),

    #[error("Missing content for: {0}")]
    MissingContent(String),

    #[error("Invalid self-reference: {0}")]
    InvalidSelfReference(String),

    #[error("Invalid number format: {0}")]
    InvalidNumber(String),

    #[error("Formatting error: {0}")]
    Format(#[from] std::fmt::Error),
}
