//! Manufacturer-specific maker note parsing
//!
//! Each camera manufacturer has their own format for storing additional
//! metadata in the MakerNotes tag (0x927c). This module provides a
//! framework for parsing these manufacturer-specific formats.

use crate::core::{Endian, ExifValue};
use crate::error::Result;
use std::collections::HashMap;

pub mod canon;

/// Trait for manufacturer-specific maker note parsers
pub trait MakerNoteParser: Send + Sync {
    /// Parse maker note data into tag/value pairs
    fn parse(
        &self,
        data: &[u8],
        byte_order: Endian,
        base_offset: usize,
    ) -> Result<HashMap<u16, ExifValue>>;

    /// Get the manufacturer name
    fn manufacturer(&self) -> &'static str;
}

/// Detected manufacturer type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Manufacturer {
    Canon,
    Nikon,
    Sony,
    Fujifilm,
    Olympus,
    Panasonic,
    Unknown,
}

impl Manufacturer {
    /// Detect manufacturer from Make tag value
    pub fn from_make(make: &str) -> Self {
        let make_lower = make.to_lowercase();

        if make_lower.contains("canon") {
            Manufacturer::Canon
        } else if make_lower.contains("nikon") {
            Manufacturer::Nikon
        } else if make_lower.contains("sony") {
            Manufacturer::Sony
        } else if make_lower.contains("fujifilm") || make_lower.contains("fuji") {
            Manufacturer::Fujifilm
        } else if make_lower.contains("olympus") {
            Manufacturer::Olympus
        } else if make_lower.contains("panasonic") {
            Manufacturer::Panasonic
        } else {
            Manufacturer::Unknown
        }
    }

    /// Get a parser for this manufacturer
    pub fn parser(&self) -> Option<Box<dyn MakerNoteParser>> {
        match self {
            Manufacturer::Canon => Some(Box::new(canon::CanonMakerNoteParser)),
            // Other manufacturers not implemented yet
            _ => None,
        }
    }
}

/// Parse maker notes based on manufacturer
pub fn parse_maker_notes(
    data: &[u8],
    make: &str,
    byte_order: Endian,
    base_offset: usize,
) -> Result<HashMap<u16, ExifValue>> {
    let manufacturer = Manufacturer::from_make(make);

    match manufacturer.parser() {
        Some(parser) => parser.parse(data, byte_order, base_offset),
        None => {
            // Return empty map for unsupported manufacturers
            Ok(HashMap::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manufacturer_detection() {
        assert_eq!(Manufacturer::from_make("Canon"), Manufacturer::Canon);
        assert_eq!(Manufacturer::from_make("Canon EOS 5D"), Manufacturer::Canon);
        assert_eq!(
            Manufacturer::from_make("CANON DIGITAL IXUS"),
            Manufacturer::Canon
        );
        assert_eq!(
            Manufacturer::from_make("NIKON CORPORATION"),
            Manufacturer::Nikon
        );
        assert_eq!(Manufacturer::from_make("SONY"), Manufacturer::Sony);
        assert_eq!(Manufacturer::from_make("FUJIFILM"), Manufacturer::Fujifilm);
        assert_eq!(
            Manufacturer::from_make("OLYMPUS CORPORATION"),
            Manufacturer::Olympus
        );
        assert_eq!(
            Manufacturer::from_make("Panasonic"),
            Manufacturer::Panasonic
        );
        assert_eq!(Manufacturer::from_make("Apple"), Manufacturer::Unknown);
    }
}
