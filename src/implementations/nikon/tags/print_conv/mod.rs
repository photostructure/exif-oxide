//! PrintConv functions for Nikon tags
//!
//! **Trust ExifTool**: This code translates ExifTool's PrintConv functions verbatim.
//!
//! ExifTool Reference: lib/Image/ExifTool/Nikon.pm PrintConv definitions
//!
//! This module contains human-readable conversion functions for Nikon tag values,
//! organized into submodules by functionality:
//! - `basic`: Core tag conversions (ISO, quality, white balance, etc.)
//! - `af`: Autofocus and vibration reduction conversions
//! - `advanced`: Advanced feature conversions (HDR, subject detection, etc.)

pub mod advanced;
pub mod af;
pub mod basic;
