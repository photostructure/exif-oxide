//! Tests for pattern recognition in PPI Rust code generation
//!
//! These tests cover:
//! - Pack C* bit extraction patterns
//! - Join+unpack binary data patterns
//! - Safe division/reciprocal patterns
//! - Sprintf with string operations
//! - Static function generation compliance

use crate::ppi::rust_generator::{
    expressions::{ComplexPatternHandler, ExpressionCombiner},
    RustGenerator,
};
use crate::ppi::{ExpressionType, PpiNode};

// Pack C* bit extraction pattern tests removed - replaced by SKIP_pack_map_bits.json config test

// Join+unpack pattern tests removed - replaced by SKIP_join_unpack.json config test

// Sprintf with string operations test removed - replaced by SKIP_sprintf_concat_ternary.json config test
