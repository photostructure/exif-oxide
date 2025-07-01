//! Implementation module for exif-oxide
//!
//! This module contains manual implementations of ExifTool's conversion functions.
//! All implementations are direct translations from ExifTool source code.

pub mod print_conv;

use crate::registry;

/// Register all implemented PrintConv functions
/// 
/// This function should be called during library initialization to populate
/// the conversion registry with available implementations.
pub fn register_all_conversions() {
    // Register PrintConv functions
    registry::register_print_conv("orientation_print_conv", print_conv::orientation_print_conv);
    registry::register_print_conv("resolutionunit_print_conv", print_conv::resolutionunit_print_conv);
    registry::register_print_conv("ycbcrpositioning_print_conv", print_conv::ycbcrpositioning_print_conv);
}