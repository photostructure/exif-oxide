//! Debug PPI Pipeline - End-to-end debugging tool for Perl expression processing
//!
//! This tool takes a Perl expression and shows every step of the processing pipeline:
//! 1. Original Perl expression
//! 2. Raw PPI AST (from ppi_ast.pl)
//! 3. Normalized AST (after running through normalizer)
//! 4. Generated Rust code
//!
//! Usage:
//!   cargo run --bin debug-ppi 'sprintf("%s:%s", unpack "H2H2", $val)'
//!
//! This can help debug normalization issues and understand the PPI rust generation pipeline.

use anyhow::{Context, Result};
use clap::Parser;
use serde_json;
use std::process::Command;
use tracing::{debug, info};

use codegen::ppi::normalizer;
use codegen::ppi::rust_generator::generator::RustGenerator;
use codegen::ppi::types::{ExpressionType, PpiNode};

#[derive(Parser)]
#[command(
    name = "debug-ppi",
    about = "Debug PPI Pipeline - shows Perl expr → AST → Normalized AST → Rust code",
    long_about = "This tool takes a Perl expression and shows every step of the processing pipeline:
1. Original Perl expression
2. Raw PPI AST (from ppi_ast.pl)
3. Normalized AST (after running through normalizer)
4. Generated Rust code

This is invaluable for debugging normalization issues and understanding the pipeline."
)]
struct Args {
    /// Perl expression to process
    #[arg(help = "Perl expression to debug (e.g., 'sprintf(\"%s\", unpack \"H2\", $val)')")]
    expression: String,

    /// Expression type for code generation
    #[arg(
        short = 't',
        long = "type",
        value_enum,
        default_value = "print-conv",
        help = "Expression type for code generation"
    )]
    expr_type: ExpressionTypeArg,

    /// Function name for generated code
    #[arg(
        short = 'f',
        long = "function",
        default_value = "debug_function",
        help = "Function name for generated code"
    )]
    function_name: String,

    /// Show detailed AST structure
    #[arg(
        short = 'v',
        long = "verbose",
        help = "Show detailed AST structure (pretty-printed JSON)"
    )]
    verbose: bool,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum ExpressionTypeArg {
    #[value(name = "print-conv")]
    PrintConv,
    #[value(name = "value-conv")]
    ValueConv,
    #[value(name = "condition")]
    Condition,
}

impl From<ExpressionTypeArg> for ExpressionType {
    fn from(arg: ExpressionTypeArg) -> Self {
        match arg {
            ExpressionTypeArg::PrintConv => ExpressionType::PrintConv,
            ExpressionTypeArg::ValueConv => ExpressionType::ValueConv,
            ExpressionTypeArg::Condition => ExpressionType::Condition,
        }
    }
}

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    println!("🔧 PPI Pipeline Debugger");
    println!("{}", "=".repeat(50));
    println!();

    // Step 1: Show original expression
    println!("📝 STEP 1: Original Perl Expression");
    println!("{}", "-".repeat(30));
    println!("{}", args.expression);
    println!();

    // Step 2: Parse with PPI to get raw AST
    println!("🌳 STEP 2: Raw PPI AST");
    println!("{}", "-".repeat(30));
    let raw_ast_json = call_ppi_ast_script(&args.expression)
        .context("Failed to parse expression with ppi_ast.pl")?;

    if args.verbose {
        println!("Raw AST JSON (formatted):");
        match serde_json::from_str::<serde_json::Value>(&raw_ast_json) {
            Ok(json) => {
                let pretty = serde_json::to_string_pretty(&json).unwrap_or(raw_ast_json.clone());
                println!("{}", pretty);
            }
            Err(_) => {
                println!("{}", raw_ast_json);
            }
        }
    } else {
        println!("✅ Successfully parsed with PPI");
    }
    println!();

    // Step 3: Parse JSON into PpiNode
    debug!("Parsing JSON into PpiNode");
    let raw_ast: PpiNode =
        serde_json::from_str(&raw_ast_json).context("Failed to parse PPI AST JSON into PpiNode")?;

    // Step 4: Run through normalizer
    println!("🔄 STEP 3: Normalized AST");
    println!("{}", "-".repeat(30));
    let normalized_ast = normalizer::normalize_multi_pass(raw_ast.clone());

    // Check if normalization actually changed anything
    let ast_changed = format!("{:?}", raw_ast) != format!("{:?}", normalized_ast);
    if ast_changed {
        println!("✨ AST was normalized (structure changed)");
        if args.verbose {
            println!("Normalized AST (cleaned):");
            print_cleaned_ast(&normalized_ast, 0);
        }
    } else {
        println!("➡️  No normalization applied (AST unchanged)");
    }
    println!();

    // Step 5: Generate Rust code
    println!("🦀 STEP 4: Generated Rust Code");
    println!("{}", "-".repeat(30));

    let expr_type = ExpressionType::from(args.expr_type);
    let generator = RustGenerator::new(
        expr_type,
        args.function_name.clone(),
        args.expression.clone(),
    );

    match generator.generate_function(&normalized_ast) {
        Ok(rust_code) => {
            println!("✅ Successfully generated Rust code:");
            println!();
            println!("{}", rust_code);
        }
        Err(e) => {
            println!("❌ Failed to generate Rust code:");
            println!("Error: {}", e);

            // Show some debugging info
            println!();
            println!("🔍 Debugging Information:");
            println!("Raw AST structure: {:?}", raw_ast);
            println!("Normalized AST structure: {:?}", normalized_ast);
            return Err(e.into());
        }
    }

    println!();
    println!("{}", "=".repeat(50));
    println!("🎉 Pipeline completed successfully!");

    Ok(())
}

/// Call the ppi_ast.pl script to parse a Perl expression into JSON AST
fn call_ppi_ast_script(expression: &str) -> Result<String> {
    info!("Calling ppi_ast.pl with expression: {}", expression);

    let output = Command::new("./scripts/ppi_ast.pl")
        .arg(expression)
        .current_dir(".")
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

    debug!("ppi_ast.pl output: {}", stdout);
    Ok(stdout.trim().to_string())
}

/// Print a cleaned version of the AST that omits None/empty fields for better readability
fn print_cleaned_ast(node: &PpiNode, indent: usize) {
    let indent_str = "  ".repeat(indent);

    // Check if this is a simple leaf node (no children, only basic fields)
    let is_simple_leaf =
        node.children.is_empty() && node.numeric_value.is_none() && node.structure_bounds.is_none();

    if is_simple_leaf {
        // Compact format for simple nodes
        print!("{}PpiNode {{ class: {:?}", indent_str, node.class);

        if let Some(ref content) = node.content {
            print!(", content: {:?}", content);
        }

        if let Some(ref string_value) = node.string_value {
            if node.content.as_ref() != Some(string_value) {
                print!(", string_value: {:?}", string_value);
            }
        }

        if let Some(ref symbol_type) = node.symbol_type {
            print!(", symbol_type: {:?}", symbol_type);
        }

        print!(" }}");
        return;
    }

    // Multi-line format for complex nodes
    print!("{}PpiNode {{", indent_str);

    // Always show class
    print!(" class: {:?}", node.class);

    // Only show content if it's Some
    if let Some(ref content) = node.content {
        print!(", content: {:?}", content);
    }

    // Only show string_value if it's Some and different from content
    if let Some(ref string_value) = node.string_value {
        if node.content.as_ref() != Some(string_value) {
            print!(", string_value: {:?}", string_value);
        }
    }

    // Only show symbol_type if it's Some
    if let Some(ref symbol_type) = node.symbol_type {
        print!(", symbol_type: {:?}", symbol_type);
    }

    // Only show numeric_value if it's Some
    if let Some(ref numeric_value) = node.numeric_value {
        print!(", numeric_value: {:?}", numeric_value);
    }

    // Only show structure_bounds if it's Some
    if let Some(ref structure_bounds) = node.structure_bounds {
        print!(", structure_bounds: {:?}", structure_bounds);
    }

    // Handle children
    if !node.children.is_empty() {
        print!(", children: [");
        println!();
        for (i, child) in node.children.iter().enumerate() {
            print_cleaned_ast(child, indent + 1);
            if i < node.children.len() - 1 {
                print!(",");
            }
            println!();
        }
        print!("{}]", indent_str);
    }

    print!(" }}");
}
