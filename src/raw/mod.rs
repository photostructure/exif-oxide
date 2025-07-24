//! RAW Image Format Support Module
//!
//! This module implements comprehensive RAW metadata extraction for all mainstream manufacturers.
//! It provides a unified interface for processing manufacturer-specific RAW formats while
//! following the Trust ExifTool principle exactly.
//!
//! ## Supported Formats (Milestone 17 Implementation)
//!
//! - **Kyocera** (.raw) - Milestone 17a: Simple ProcessBinaryData format (173 lines)
//! - **Minolta** (MRW) - Milestone 17b: Multi-block format with TTW, PRD, WBG blocks
//! - **Panasonic** (RW2) - Milestone 17b: Entry-based offset handling
//! - **Olympus** (ORF) - Milestone 17c: Multiple IFD navigation
//! - **Canon** (CR2, CRW, CR3) - Milestone 17d: Complex TIFF-based with 169 ProcessBinaryData sections
//! - **Nikon** (NEF, NRW) - Future: Integration with existing Nikon implementation
//! - **Sony** (ARW, SR2, SRF) - Future: Advanced offset management
//! - **Fujifilm** (RAF) - Future: Non-TIFF format
//!
//! ## Architecture
//!
//! The RAW processing system uses a hybrid approach:
//! - **Format Detection**: FileTypeDetector identifies RAW file types
//! - **RAW Processor**: Central dispatcher routes to manufacturer handlers
//! - **Handler Traits**: Manufacturer-specific processing implementations
//! - **TIFF Foundation**: Leverages existing TIFF infrastructure for TIFF-based formats
//!
//! ## Trust ExifTool Compliance
//!
//! All implementations strictly follow ExifTool's processing logic:
//! - Exact offset calculations and data parsing
//! - Identical tag naming and grouping
//! - Preserved quirks and manufacturer-specific handling
//! - No "improvements" or "optimizations" to the original logic

pub mod detector;
pub mod offset;
pub mod processor;
pub mod utils;

// Re-export main types for convenience
pub use detector::{detect_raw_format, RawFormat};
pub use offset::{EntryBasedOffsetProcessor, OffsetContext, SimpleOffsetProcessor};
pub use processor::{RawFormatHandler, RawProcessor};

// Import format-specific handlers (will expand as we add more formats)
pub mod formats;

// Re-export format handlers and utility functions
pub use formats::canon::get_canon_tag_name;
pub use formats::kyocera::get_kyocera_tag_name;
pub use formats::minolta::get_minolta_tag_name;
pub use formats::olympus::get_olympus_tag_name;
pub use formats::panasonic::get_panasonic_tag_name;
pub use formats::sony::get_sony_tag_name;

#[cfg(test)]
mod tests;
