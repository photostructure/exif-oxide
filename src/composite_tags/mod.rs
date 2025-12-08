//! Composite tag processing module
//!
//! This module provides multi-pass composite tag building functionality,
//! translating ExifTool's composite tag evaluation logic from Perl to Rust.
//!
//! The main entry point is [`resolve_and_compute_composites`] which handles
//! dependency resolution and computation across multiple passes.

mod dispatch;
mod implementations;
mod orchestration;
mod resolution;

// Re-export the main public API
// TEMPORARILY COMMENTED OUT - apply_composite_conversions not generated yet
// pub use orchestration::{apply_composite_conversions, resolve_and_compute_composites};
pub use orchestration::resolve_and_compute_composites;
pub use resolution::{build_available_tags_map, can_build_composite, is_dependency_available};

// Re-export for testing and internal use
// TEMPORARILY COMMENTED OUT - compute_composite_tag not generated yet
// pub use dispatch::compute_composite_tag;
pub use orchestration::handle_unresolved_composites;
