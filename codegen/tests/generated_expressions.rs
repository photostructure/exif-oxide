//! Integration test runner for generated expression tests
//!
//! This file includes all generated test modules so that `cargo test` can discover and run them.

// Include the generated test modules via the generated mod.rs
mod generated;

// Re-export everything from generated modules for easier testing
pub use generated::*;
