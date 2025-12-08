//! Shared PPI Pipeline Processing
//!
//! Common functionality for processing Perl expressions through the complete PPI pipeline.
//! Used by both debug-ppi and generate-expression-tests tools.
//!
//! Trust ExifTool: This module preserves exact PPI pipeline behavior without modification.

use anyhow::{Context, Result};
use serde_json;
use std::process::Command;
use tracing::{debug, info};

use crate::ppi::normalizer;
use crate::ppi::rust_generator::generator::RustGenerator;
use crate::ppi::types::{ExpressionContext, ExpressionType, PpiNode};

/// Output from the complete PPI pipeline processing
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PipelineOutput {
    pub original_expression: String,
    pub raw_ppi_ast: PpiNode,
    pub normalized_ast: PpiNode,
    pub generated_rust: String,
    pub ast_hash: String,
}

/// Process a Perl expression through the complete PPI pipeline
///
/// This is the shared logic used by both debug-ppi and generate-expression-tests.
///
/// # Arguments
/// * `expression` - The Perl expression to process
/// * `expr_type` - Type of expression (PrintConv, ValueConv, Condition)  
/// * `function_name` - Name to use for the generated function
///
/// # Returns
/// Complete pipeline output including AST at each stage and generated Rust code
#[allow(dead_code)]
pub fn process_perl_expression(
    expression: &str,
    expr_type: ExpressionType,
    function_name: &str,
) -> Result<PipelineOutput> {
    info!("Processing Perl expression: {}", expression);

    // Step 1: Call ppi_ast.pl to get raw AST
    let raw_ast_json =
        call_ppi_ast_script(expression).context("Failed to parse expression with ppi_ast.pl")?;

    // Step 2: Parse JSON into PpiNode
    let raw_ppi_ast: PpiNode =
        serde_json::from_str(&raw_ast_json).context("Failed to parse PPI AST JSON into PpiNode")?;

    // Step 3: Apply normalization
    let normalized_ast = normalizer::normalize_multi_pass(raw_ppi_ast.clone());

    // Step 4: Generate Rust code
    let generator =
        RustGenerator::new(expr_type, function_name.to_string(), expression.to_string());

    let generated_rust = generator
        .generate_function(&normalized_ast)
        .context("Failed to generate Rust code from normalized AST")?;

    // Step 5: Generate AST hash for deduplication (placeholder for now)
    let ast_hash = generate_ast_hash(&normalized_ast);

    debug!(
        "Pipeline processing complete for expression: {}",
        expression
    );

    Ok(PipelineOutput {
        original_expression: expression.to_string(),
        raw_ppi_ast,
        normalized_ast,
        generated_rust,
        ast_hash,
    })
}

/// Process a Perl expression through the complete PPI pipeline with specified context
///
/// This variant allows specifying the expression context (Regular or Composite).
/// Use `ExpressionContext::Composite` for composite tag expressions that use
/// `$val[n]`, `$prt[n]`, `$raw[n]` array access patterns.
///
/// # Arguments
/// * `expression` - The Perl expression to process
/// * `expr_type` - Type of expression (PrintConv, ValueConv, Condition)
/// * `expr_context` - Context for code generation (Regular or Composite)
/// * `function_name` - Name to use for the generated function
///
/// # Returns
/// Complete pipeline output including AST at each stage and generated Rust code
#[allow(dead_code)]
pub fn process_perl_expression_with_context(
    expression: &str,
    expr_type: ExpressionType,
    expr_context: ExpressionContext,
    function_name: &str,
) -> Result<PipelineOutput> {
    info!(
        "Processing Perl expression with {:?} context: {}",
        expr_context, expression
    );

    // Step 1: Call ppi_ast.pl to get raw AST
    let raw_ast_json =
        call_ppi_ast_script(expression).context("Failed to parse expression with ppi_ast.pl")?;

    // Step 2: Parse JSON into PpiNode
    let raw_ppi_ast: PpiNode =
        serde_json::from_str(&raw_ast_json).context("Failed to parse PPI AST JSON into PpiNode")?;

    // Step 3: Apply normalization
    let normalized_ast = normalizer::normalize_multi_pass(raw_ppi_ast.clone());

    // Step 4: Generate Rust code with the specified context
    let generator = RustGenerator::with_context(
        expr_type,
        expr_context,
        function_name.to_string(),
        expression.to_string(),
    );

    let generated_rust = generator
        .generate_function(&normalized_ast)
        .context("Failed to generate Rust code from normalized AST")?;

    // Step 5: Generate AST hash for deduplication (placeholder for now)
    let ast_hash = generate_ast_hash(&normalized_ast);

    debug!(
        "Pipeline processing complete for expression: {}",
        expression
    );

    Ok(PipelineOutput {
        original_expression: expression.to_string(),
        raw_ppi_ast,
        normalized_ast,
        generated_rust,
        ast_hash,
    })
}

/// Call the ppi_ast.pl script to parse a Perl expression into JSON AST
///
/// This function is shared between debug-ppi and generate-expression-tests
/// to ensure consistent behavior.
#[allow(dead_code)]
pub fn call_ppi_ast_script(expression: &str) -> Result<String> {
    debug!("Calling ppi_ast.pl with expression: {}", expression);

    let script_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/ppi_ast.pl");

    let output = Command::new(script_path)
        .arg(expression)
        .output()
        .context("Failed to execute ppi_ast.pl script")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "ppi_ast.pl failed with exit code {}: {}",
            output.status.code().unwrap_or(-1),
            stderr
        ));
    }

    let stdout =
        String::from_utf8(output.stdout).context("ppi_ast.pl output is not valid UTF-8")?;

    debug!("ppi_ast.pl output: {}", stdout.trim());
    Ok(stdout.trim().to_string())
}

/// Generate hash for AST deduplication
///
/// TODO: Implement proper structural hashing for AST deduplication
/// This will be used by the fn_registry system for function deduplication.
#[allow(dead_code)]
fn generate_ast_hash(ast: &PpiNode) -> String {
    // For now, generate a placeholder hash based on the AST structure
    // Later this should generate a proper structural hash for deduplication
    format!("ast_{:x}", ast.class.len())
}
