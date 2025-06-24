//! Container format parsers for metadata extraction
//!
//! Many file formats use container structures (RIFF, QuickTime) that wrap
//! metadata in specific chunks or atoms. This module provides parsers for
//! these container formats.

pub mod quicktime;
pub mod riff;

use crate::error::Result;
use std::io::{Read, Seek};

/// Common trait for container parsers
pub trait ContainerParser {
    /// Extract metadata from the container
    fn extract_metadata<R: Read + Seek>(reader: &mut R) -> Result<Option<Vec<u8>>>;
}
