//! Composite tag processing module
//!
//! This module provides multi-pass composite tag building functionality,
//! translating ExifTool's composite tag evaluation logic from Perl to Rust.
//!
//! The main entry point is [`resolve_and_compute_composites`] which handles
//! dependency resolution and computation across multiple passes.
//!
//! ## Architecture
//!
//! - **orchestration.rs**: Multi-pass loop that iterates through composite tags
//! - **resolution.rs**: Dependency checking and array building for function calls
//! - **implementations.rs**: Manual fallback implementations for complex composites
//!
//! Generated composite functions are in `src/generated/composite_tags.rs`

mod dispatch;
pub mod implementations;
mod orchestration;
mod resolution;

// Re-export the main public API
pub use orchestration::{handle_unresolved_composites, resolve_and_compute_composites};
pub use resolution::{
    build_available_tags_map, build_available_tags_map_with_conversions, can_build_composite,
    is_dependency_available, resolve_dependency_arrays, TagDependencyValues,
};
