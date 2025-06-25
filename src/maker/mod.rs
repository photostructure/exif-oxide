//! Manufacturer-specific maker note parsing
//!
//! Each camera manufacturer has their own format for storing additional
//! metadata in the MakerNotes tag (0x927c). This module provides a
//! framework for parsing these manufacturer-specific formats.

use crate::core::{Endian, ExifValue};
use crate::error::Result;
use std::collections::HashMap;

pub mod apple;
pub mod canon;
pub mod fujifilm;
pub mod hasselblad;
pub mod leica;
pub mod nikon;
pub mod olympus;
pub mod panasonic;
pub mod pentax;
pub mod samsung;
pub mod sigma;
pub mod sony;

pub mod casio;
pub mod kodak;

pub mod dji;
pub mod gopro;
pub mod minolta;
pub mod ricoh;
// pub mod phaseone;
// pub mod qualcomm;
// pub mod red;

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
    Apple,
    Canon,
    Nikon,
    Sony,
    Fujifilm,
    Olympus,
    Panasonic,
    Pentax,
    Leica,
    Samsung,
    Sigma,
    Hasselblad,
    // Phase 3 manufacturers (ready for implementation)
    Casio,
    Kodak,
    Minolta,
    GoPro,
    DJI,
    Ricoh,
    PhaseOne,
    Qualcomm,
    Red,
    Unknown,
}

impl Manufacturer {
    /// Detect manufacturer from Make tag value
    pub fn from_make(make: &str) -> Self {
        let make_lower = make.to_lowercase();

        if make_lower.contains("apple") {
            Manufacturer::Apple
        } else if make_lower.contains("canon") {
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
        } else if make_lower.contains("pentax") || make_lower.contains("asahi") {
            // EXIFTOOL-SOURCE: lib/Image/ExifTool/MakerNotes.pm:776,788 ($$self{Make}=~/^Asahi/)
            Manufacturer::Pentax
        } else if make_lower.contains("ricoh") {
            Manufacturer::Ricoh
        } else if make_lower.contains("leica") {
            Manufacturer::Leica
        } else if make_lower.contains("samsung") {
            Manufacturer::Samsung
        } else if make_lower.contains("sigma") {
            Manufacturer::Sigma
        } else if make_lower.contains("hasselblad") {
            Manufacturer::Hasselblad
        } else if make_lower.contains("casio") {
            Manufacturer::Casio
        } else if make_lower.contains("kodak") {
            Manufacturer::Kodak
        } else if make_lower.contains("minolta") {
            Manufacturer::Minolta
        } else if make_lower.contains("gopro") {
            Manufacturer::GoPro
        } else if make_lower.contains("dji") {
            Manufacturer::DJI
        } else if make_lower.contains("phase one") || make_lower.contains("phaseone") {
            Manufacturer::PhaseOne
        } else if make_lower.contains("qualcomm") {
            Manufacturer::Qualcomm
        } else if make_lower.contains("red") {
            Manufacturer::Red
        } else {
            Manufacturer::Unknown
        }
    }

    /// Get a parser for this manufacturer
    pub fn parser(&self) -> Option<Box<dyn MakerNoteParser>> {
        match self {
            Manufacturer::Apple => Some(Box::new(apple::AppleMakerNoteParser)),
            Manufacturer::Canon => Some(Box::new(canon::CanonMakerNoteParser)),
            Manufacturer::Fujifilm => Some(Box::new(fujifilm::FujifilmMakerNoteParser)),
            Manufacturer::Hasselblad => Some(Box::new(hasselblad::HasselbladMakerNoteParser)),
            Manufacturer::Leica => Some(Box::new(leica::LeicaMakerNoteParser)),
            Manufacturer::Nikon => Some(Box::new(nikon::NikonMakerNoteParser)),
            Manufacturer::Olympus => Some(Box::new(olympus::OlympusMakerNoteParser)),
            Manufacturer::Panasonic => Some(Box::new(panasonic::PanasonicMakerNoteParser)),
            Manufacturer::Pentax => Some(Box::new(pentax::PentaxMakerNoteParser)),
            Manufacturer::Samsung => Some(Box::new(samsung::SamsungMakerNoteParser)),
            Manufacturer::Sigma => Some(Box::new(sigma::SigmaMakerNoteParser)),
            Manufacturer::Sony => Some(Box::new(sony::SonyMakerNoteParser)),
            // Phase 3 manufacturers (commented until implemented)
            Manufacturer::Casio => Some(Box::new(casio::CasioMakerNoteParser)),
            Manufacturer::Kodak => Some(Box::new(kodak::KodakMakerNoteParser)),
            Manufacturer::Minolta => Some(Box::new(minolta::MinoltaMakerNoteParser)),
            Manufacturer::GoPro => Some(Box::new(gopro::GoProMakerNoteParser)),
            Manufacturer::DJI => Some(Box::new(dji::DJIMakerNoteParser)),
            Manufacturer::Ricoh => Some(Box::new(ricoh::RicohMakerNoteParser)),
            // Manufacturer::PhaseOne => Some(Box::new(phaseone::PhaseOneMakerNoteParser)),
            // Manufacturer::Qualcomm => Some(Box::new(qualcomm::QualcommMakerNoteParser)),
            // Manufacturer::Red => Some(Box::new(red::RedMakerNoteParser)),
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
        assert_eq!(Manufacturer::from_make("PENTAX"), Manufacturer::Pentax);
        assert_eq!(
            Manufacturer::from_make("RICOH IMAGING"),
            Manufacturer::Ricoh
        );
        assert_eq!(Manufacturer::from_make("LEICA"), Manufacturer::Leica);
        assert_eq!(
            Manufacturer::from_make("Leica Camera AG"),
            Manufacturer::Leica
        );
        assert_eq!(Manufacturer::from_make("SIGMA"), Manufacturer::Sigma);
        assert_eq!(
            Manufacturer::from_make("Sigma Corporation"),
            Manufacturer::Sigma
        );
        assert_eq!(
            Manufacturer::from_make("Hasselblad"),
            Manufacturer::Hasselblad
        );
        assert_eq!(
            Manufacturer::from_make("HASSELBLAD"),
            Manufacturer::Hasselblad
        );
        assert_eq!(Manufacturer::from_make("Apple"), Manufacturer::Apple);
        assert_eq!(Manufacturer::from_make("APPLE"), Manufacturer::Apple);
        assert_eq!(Manufacturer::from_make("Samsung"), Manufacturer::Samsung);
        assert_eq!(Manufacturer::from_make("SAMSUNG"), Manufacturer::Samsung);

        // Phase 3 manufacturers
        assert_eq!(Manufacturer::from_make("CASIO"), Manufacturer::Casio);
        assert_eq!(
            Manufacturer::from_make("Casio Computer Co.,Ltd."),
            Manufacturer::Casio
        );
        assert_eq!(
            Manufacturer::from_make("EASTMAN KODAK COMPANY"),
            Manufacturer::Kodak
        );
        assert_eq!(Manufacturer::from_make("Kodak"), Manufacturer::Kodak);
        assert_eq!(Manufacturer::from_make("MINOLTA"), Manufacturer::Minolta);
        assert_eq!(
            Manufacturer::from_make("Minolta Co., Ltd."),
            Manufacturer::Minolta
        );
        assert_eq!(Manufacturer::from_make("GoPro"), Manufacturer::GoPro);
        assert_eq!(Manufacturer::from_make("GOPRO"), Manufacturer::GoPro);
        assert_eq!(Manufacturer::from_make("DJI"), Manufacturer::DJI);
        assert_eq!(Manufacturer::from_make("RICOH"), Manufacturer::Ricoh);
        assert_eq!(
            Manufacturer::from_make("RICOH IMAGING"),
            Manufacturer::Ricoh
        ); // Note: Ricoh cameras under Pentax
        assert_eq!(Manufacturer::from_make("Phase One"), Manufacturer::PhaseOne);
        assert_eq!(Manufacturer::from_make("QUALCOMM"), Manufacturer::Qualcomm);
        assert_eq!(Manufacturer::from_make("RED"), Manufacturer::Red);
    }
}
