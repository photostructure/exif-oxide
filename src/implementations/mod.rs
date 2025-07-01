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
    registry::register_print_conv(
        "resolutionunit_print_conv",
        print_conv::resolutionunit_print_conv,
    );
    registry::register_print_conv(
        "ycbcrpositioning_print_conv",
        print_conv::ycbcrpositioning_print_conv,
    );
    registry::register_print_conv(
        "gpsaltituderef_print_conv",
        print_conv::gpsaltituderef_print_conv,
    );
    registry::register_print_conv(
        "gpslatituderef_print_conv",
        print_conv::gpslatituderef_print_conv,
    );
    registry::register_print_conv(
        "gpslongituderef_print_conv",
        print_conv::gpslongituderef_print_conv,
    );

    // Register new Milestone 7 PrintConv implementations
    registry::register_print_conv("flash_print_conv", print_conv::flash_print_conv);
    registry::register_print_conv("colorspace_print_conv", print_conv::colorspace_print_conv);
    registry::register_print_conv(
        "whitebalance_print_conv",
        print_conv::whitebalance_print_conv,
    );
    registry::register_print_conv(
        "meteringmode_print_conv",
        print_conv::meteringmode_print_conv,
    );
    registry::register_print_conv(
        "exposureprogram_print_conv",
        print_conv::exposureprogram_print_conv,
    );
}
