//! Regression tests for parenthesized-expression precedence in the PPI pipeline.
//!
//! ExifTool expressions frequently use explicit parentheses to group a
//! lower-precedence sub-expression, e.g. `1 / (1 + $val/32768)` or
//! `($val - 1) * 3`. The visitor used to drop the grouping node's parentheses
//! when it wrapped a single child (`visit_list`), so the surrounding operator
//! rebound the flattened operands and produced arithmetically wrong code
//! (`1 / 1 + val / 32768`, `val - 1 * 3`). These tests drive the full pipeline
//! (ppi_ast.pl -> normalizer -> Rust generator) and assert the grouping
//! survives.

use codegen::ppi::shared_pipeline::process_perl_expression;
use codegen::ppi::ExpressionType;

fn generated(expression: &str) -> String {
    process_perl_expression(expression, ExpressionType::ValueConv, "test_fn")
        .expect("pipeline should generate code")
        .generated_rust
}

#[test]
fn preserves_grouping_on_divisor() {
    // PanasonicRaw.pm:460 DistortionScale ValueConv.
    let rust = generated("1 / (1 + $val/32768)");
    assert!(
        rust.contains("1i32 / (1i32 + val / 32768i32)"),
        "divisor grouping dropped; got:\n{rust}"
    );
    // The flattened form divided 1 by 1 and then added the rest.
    assert!(
        !rust.contains("1i32 / 1i32 +"),
        "grouping was flattened; got:\n{rust}"
    );
}

#[test]
fn preserves_grouping_on_left_operand() {
    // H264.pm Camera1 Gain ValueConv.
    let rust = generated("($val - 1) * 3");
    assert!(
        rust.contains("(val - 1i32) * 3i32"),
        "left-operand grouping dropped; got:\n{rust}"
    );
    // The flattened form multiplied 1 * 3 first.
    assert!(
        !rust.contains("val - 1i32 * 3i32"),
        "grouping was flattened; got:\n{rust}"
    );
}

#[test]
fn preserves_grouping_on_dividend() {
    let rust = generated("($val + 100) / 2");
    assert!(
        rust.contains("(val + 100i32) / 2i32"),
        "dividend grouping dropped; got:\n{rust}"
    );
    assert!(
        !rust.contains("val + 100i32 / 2i32"),
        "grouping was flattened; got:\n{rust}"
    );
}

#[test]
fn preserves_grouping_in_numeric_comparison() {
    let rust = generated("($val + 1) > 2");
    assert!(
        rust.contains("(val + 1i32) > 2i32"),
        "comparison-operand grouping dropped; got:\n{rust}"
    );
}

#[test]
fn does_not_add_redundant_parens_for_atoms() {
    // A grouping around a single atom is self-delimiting; no parens needed
    // (and redundant parens would trip `unused_parens` under `-D warnings`).
    let rust = generated("($val) * 3");
    assert!(
        rust.contains("val * 3i32"),
        "expected bare atom operand; got:\n{rust}"
    );
    assert!(
        !rust.contains("(val)"),
        "redundant parens around atom; got:\n{rust}"
    );
}

#[test]
fn does_not_add_redundant_parens_for_power_operand() {
    // `**` renders as the self-delimiting `power(..)` call, so a grouping whose
    // child is `**` must not get wrapped again.
    let rust = generated("(2 ** $val) * 3");
    // The power() call is multiplied directly, with no wrapping parens.
    assert!(
        rust.contains("val.clone()) * 3i32"),
        "redundant parens around power() operand; got:\n{rust}"
    );
}
