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

use anyhow::Result;
use clap::Parser;
use serde_json;

use codegen::ppi::shared_pipeline::{
    process_perl_expression, process_perl_expression_with_context,
};
use codegen::ppi::types::{ExpressionContext, ExpressionType, PpiNode};

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

    /// Use composite tag context ($val[n] → vals.get(n) instead of get_array_element)
    #[arg(
        short = 'c',
        long = "composite",
        help = "Generate code for composite tag context (uses vals/prts/raws slices)"
    )]
    composite: bool,
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
    if args.composite {
        println!("📦 Using COMPOSITE context (vals/prts/raws slices)");
    }
    println!();

    // Process through shared pipeline
    let expr_type = ExpressionType::from(args.expr_type);
    let pipeline_output = if args.composite {
        process_perl_expression_with_context(
            &args.expression,
            expr_type,
            ExpressionContext::Composite,
            &args.function_name,
        )?
    } else {
        process_perl_expression(&args.expression, expr_type, &args.function_name)?
    };

    // Step 1: Show original expression
    println!("📝 STEP 1: Original Perl Expression");
    println!("{}", "-".repeat(30));
    println!("{}", pipeline_output.original_expression);
    println!();

    // Step 2: Raw PPI AST
    println!("🌳 STEP 2: Raw PPI AST");
    println!("{}", "-".repeat(30));
    if args.verbose {
        println!("Raw AST (formatted):");
        match serde_json::to_string_pretty(&pipeline_output.raw_ppi_ast) {
            Ok(pretty) => println!("{}", pretty),
            Err(_) => println!("{:?}", pipeline_output.raw_ppi_ast),
        }
    } else {
        println!("✅ Successfully parsed with PPI");
    }
    println!();

    // Step 3: Normalized AST
    println!("🔄 STEP 3: Normalized AST");
    println!("{}", "-".repeat(30));

    // Check if normalization actually changed anything
    let ast_changed = format!("{:?}", pipeline_output.raw_ppi_ast)
        != format!("{:?}", pipeline_output.normalized_ast);
    if ast_changed {
        println!("✨ AST was normalized (structure changed)");
        if args.verbose {
            println!("Normalized AST (cleaned):");
            print_cleaned_ast(&pipeline_output.normalized_ast, 0);
        }
    } else {
        println!("➡️  No normalization applied (AST unchanged)");
    }
    println!();

    // Step 4: Generated Rust Code
    println!("🦀 STEP 4: Generated Rust Code");
    println!("{}", "-".repeat(30));
    println!("✅ Successfully generated Rust code:");
    println!();
    println!("{}", pipeline_output.generated_rust);

    println!();
    println!("{}", "=".repeat(50));
    println!("🎉 Pipeline completed successfully!");

    Ok(())
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
