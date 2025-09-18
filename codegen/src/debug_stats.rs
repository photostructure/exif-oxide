//! Debug statistics analyzer for P07 impl_registry fallback integration
//!
//! This tool analyzes registry fallback statistics to validate that the three-tier
//! fallback system (PPI → Registry → Placeholder) is working correctly.
//!
//! Usage:
//! ```bash
//! cargo run --bin debug-stats
//! cargo run --bin debug-stats --verbose
//! ```

use anyhow::Result;
use clap::Parser;
use codegen::ppi::fn_registry::{ConversionStats, PpiFunctionRegistry};
use codegen::ppi::ExpressionType;

#[derive(Parser)]
#[command(name = "debug-stats")]
#[command(about = "Analyze PPI registry fallback statistics")]
struct Cli {
    /// Show verbose output with detailed breakdown
    #[arg(short, long)]
    verbose: bool,

    /// Show only registry vs placeholder summary
    #[arg(short, long)]
    summary: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("🔍 P07 Registry Fallback Statistics Analyzer");
    println!("============================================");

    // Create a registry to simulate current state
    // In a real implementation, this would load actual statistics from
    // a codegen run, but for now we'll show the structure
    let registry = PpiFunctionRegistry::new();
    let stats = registry.stats().conversion_stats;

    if cli.summary {
        show_summary(&stats)?;
    } else {
        show_detailed_stats(&stats, cli.verbose)?;
    }

    println!("\n✅ Analysis complete. Use --verbose for more details.");
    Ok(())
}

fn show_summary(stats: &ConversionStats) -> Result<()> {
    println!("\n📊 Summary:");
    println!("------------------");

    let expression_types = vec![
        ExpressionType::PrintConv,
        ExpressionType::ValueConv,
        ExpressionType::Condition,
    ];

    let mut total_registry_successes = 0;
    let mut total_placeholder_fallbacks = 0;
    let mut total_ppi_successes = 0;

    for expr_type in &expression_types {
        let (ppi, registry, placeholder) = get_stats_for_type(stats, *expr_type);
        total_ppi_successes += ppi;
        total_registry_successes += registry;
        total_placeholder_fallbacks += placeholder;
    }

    let total_handled = total_ppi_successes + total_registry_successes;
    let total_expressions = total_handled + total_placeholder_fallbacks;

    if total_expressions == 0 {
        println!("⚠️  No statistics available. Run codegen to generate statistics.");
        return Ok(());
    }

    println!(
        "📈 PPI Generation:     {} ({:.1}%)",
        total_ppi_successes,
        percentage(total_ppi_successes, total_expressions)
    );
    println!(
        "🔄 Registry Fallback:  {} ({:.1}%)",
        total_registry_successes,
        percentage(total_registry_successes, total_expressions)
    );
    println!(
        "📝 Placeholder:        {} ({:.1}%)",
        total_placeholder_fallbacks,
        percentage(total_placeholder_fallbacks, total_expressions)
    );
    println!(
        "📊 Total Coverage:     {} ({:.1}%)",
        total_handled,
        percentage(total_handled, total_expressions)
    );

    if total_registry_successes > 0 {
        println!("\n✅ Registry fallback is working!");
    } else {
        println!("\n⚠️  Registry fallback not active. Check impl_registry integration.");
    }

    Ok(())
}

fn show_detailed_stats(stats: &ConversionStats, verbose: bool) -> Result<()> {
    let expression_types = vec![
        ("PrintConv", ExpressionType::PrintConv),
        ("ValueConv", ExpressionType::ValueConv),
        ("Condition", ExpressionType::Condition),
    ];

    for (type_name, expr_type) in expression_types {
        println!("\n📋 {} Statistics:", type_name);
        println!("{}================", "=".repeat(type_name.len()));

        let (ppi_successes, registry_successes, placeholder_fallbacks) =
            get_stats_for_type(stats, expr_type);

        let total_attempts = get_attempts_for_type(stats, expr_type);
        let total_handled = ppi_successes + registry_successes;

        if total_attempts == 0 {
            println!("   No {} expressions processed", type_name.to_lowercase());
            continue;
        }

        println!("   📊 Total Attempts:        {}", total_attempts);
        println!(
            "   ✅ PPI Generated:         {} ({:.1}%)",
            ppi_successes,
            percentage(ppi_successes, total_attempts)
        );
        println!(
            "   🔄 Registry Fallback:     {} ({:.1}%)",
            registry_successes,
            percentage(registry_successes, total_attempts)
        );
        println!(
            "   📝 Placeholder Fallback:  {} ({:.1}%)",
            placeholder_fallbacks,
            percentage(placeholder_fallbacks, total_attempts)
        );
        println!(
            "   📈 Success Rate:          {} ({:.1}%)",
            total_handled,
            percentage(total_handled, total_attempts)
        );

        if verbose {
            let ppi_rate = stats.ppi_success_rate(expr_type);
            let registry_rate = stats.registry_success_rate(expr_type);
            let total_rate = stats.total_success_rate(expr_type);

            println!("   📐 PPI Success Rate:      {:.1}%", ppi_rate);
            println!("   📐 Registry Success Rate: {:.1}%", registry_rate);
            println!("   📐 Total Success Rate:    {:.1}%", total_rate);
        }

        // Analysis
        if registry_successes > 0 {
            println!("   ✅ Registry integration working for {}", type_name);
        } else if ppi_successes > 0 {
            println!(
                "   ℹ️  PPI handling {} expressions (registry not needed)",
                type_name
            );
        } else {
            println!(
                "   ⚠️  All {} expressions falling back to placeholders",
                type_name
            );
        }
    }

    if verbose {
        show_integration_analysis(stats)?;
    }

    Ok(())
}

fn show_integration_analysis(stats: &ConversionStats) -> Result<()> {
    println!("\n🔍 Integration Analysis:");
    println!("=======================");

    // Check if the three-tier system is working
    let mut has_ppi = false;
    let mut has_registry = false;
    let mut has_placeholders = false;

    for expr_type in [
        ExpressionType::PrintConv,
        ExpressionType::ValueConv,
        ExpressionType::Condition,
    ] {
        let (ppi, registry, placeholder) = get_stats_for_type(stats, expr_type);
        if ppi > 0 {
            has_ppi = true;
        }
        if registry > 0 {
            has_registry = true;
        }
        if placeholder > 0 {
            has_placeholders = true;
        }
    }

    println!(
        "   🔧 PPI Generation:     {}",
        if has_ppi {
            "✅ Active"
        } else {
            "⚠️  Not active"
        }
    );
    println!(
        "   🔄 Registry Fallback:  {}",
        if has_registry {
            "✅ Active"
        } else {
            "⚠️  Not active"
        }
    );
    println!(
        "   📝 Placeholder System: {}",
        if has_placeholders {
            "✅ Active"
        } else {
            "✅ Clean (no missing implementations)"
        }
    );

    if has_ppi && has_registry && has_placeholders {
        println!("\n   ✅ Three-tier fallback system fully operational!");
    } else if has_ppi && has_registry {
        println!("\n   ✅ PPI + Registry working well (clean implementation!)");
    } else if has_ppi && has_placeholders {
        println!("\n   ⚠️  Registry fallback not active - check impl_registry integration");
    } else if has_registry && has_placeholders {
        println!("\n   ⚠️  PPI generation not working - check AST processing");
    } else {
        println!("\n   ❌ System not functioning correctly - check P07 implementation");
    }

    Ok(())
}

fn get_stats_for_type(stats: &ConversionStats, expr_type: ExpressionType) -> (usize, usize, usize) {
    match expr_type {
        ExpressionType::PrintConv => (
            stats.print_conv_ppi_successes,
            stats.print_conv_registry_successes,
            stats.print_conv_placeholder_fallbacks,
        ),
        ExpressionType::ValueConv => (
            stats.value_conv_ppi_successes,
            stats.value_conv_registry_successes,
            stats.value_conv_placeholder_fallbacks,
        ),
        ExpressionType::Condition => (
            stats.condition_ppi_successes,
            stats.condition_registry_successes,
            stats.condition_placeholder_fallbacks,
        ),
    }
}

fn get_attempts_for_type(stats: &ConversionStats, expr_type: ExpressionType) -> usize {
    match expr_type {
        ExpressionType::PrintConv => stats.print_conv_attempts,
        ExpressionType::ValueConv => stats.value_conv_attempts,
        ExpressionType::Condition => stats.condition_attempts,
    }
}

fn percentage(part: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        (part as f64 / total as f64) * 100.0
    }
}
