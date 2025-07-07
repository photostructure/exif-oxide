//! Nikon tag ID mappings and model-specific table structures
//!
//! **Trust ExifTool**: This code translates ExifTool's Nikon tag definitions verbatim.
//!
//! ExifTool Reference: lib/Image/ExifTool/Nikon.pm tag tables (135 total)
//!
//! This module provides the foundation for Nikon's extensive tag system including:
//! - Primary tag ID mappings (Nikon::Main table)
//! - Model-specific tag tables (ShotInfo variants, Z-series specific)
//! - Conditional tag processing based on camera model
//!
//! Phase 1 Implementation: Core tag structure and mainstream tag mappings
//! Phase 2+ Implementation: Complete tag tables and PrintConv functions

// Submodules
pub mod lookup;
pub mod print_conv;
pub mod tables;

// Re-export public items
pub use lookup::{get_nikon_tag_name, select_nikon_tag_table};
pub use tables::*;

/// Type alias for Nikon tag definition tuples
/// ExifTool: Tag table entry structure (tag_id, name, optional print_conv function)
pub type NikonTagEntry = (
    u16,
    &'static str,
    Option<fn(&crate::types::TagValue) -> Result<String, String>>,
);

/// Nikon tag table structure for model-specific processing
/// ExifTool: Nikon.pm model-specific tag table organization
#[derive(Debug, Clone)]
pub struct NikonTagTable {
    /// Table name for identification
    /// ExifTool: $$tagTablePtr{TABLE_NAME}
    pub name: &'static str,

    /// Optional model condition for table selection
    /// ExifTool: Condition => '$$self{Model} =~ /pattern/'
    pub model_condition: Option<&'static str>,

    /// Tag definitions (tag_id, name, optional print_conv function)
    /// ExifTool: Tag table hash with ID => { Name => ..., PrintConv => ... }
    pub tags: &'static [NikonTagEntry],
}
