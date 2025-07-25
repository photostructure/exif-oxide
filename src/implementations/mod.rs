//! Implementation module for exif-oxide
//!
//! This module contains manual implementations of ExifTool's conversion functions.
//! All implementations are direct translations from ExifTool source code.

pub mod canon;
pub mod minolta_raw;
pub mod nikon;
pub mod olympus;
pub mod panasonic_raw;
pub mod print_conv;
pub mod sony;
pub mod value_conv;

use crate::registry;

/// Register all implemented PrintConv and ValueConv functions
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
    registry::register_print_conv("gpsaltitude_print_conv", print_conv::gpsaltitude_print_conv);
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
    registry::register_print_conv("gpslatitude_print_conv", print_conv::gpslatitude_print_conv);
    registry::register_print_conv(
        "gpslongitude_print_conv",
        print_conv::gpslongitude_print_conv,
    );
    registry::register_print_conv(
        "gpsdestlatitude_print_conv",
        print_conv::gpsdestlatitude_print_conv,
    );
    registry::register_print_conv(
        "gpsdestlongitude_print_conv",
        print_conv::gpsdestlongitude_print_conv,
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

    // Register Milestone 8b camera setting PrintConv implementations
    registry::register_print_conv("fnumber_print_conv", print_conv::fnumber_print_conv);
    registry::register_print_conv(
        "exposuretime_print_conv",
        print_conv::exposuretime_print_conv,
    );
    registry::register_print_conv("focallength_print_conv", print_conv::focallength_print_conv);
    registry::register_print_conv("lensinfo_print_conv", print_conv::lensinfo_print_conv);
    registry::register_print_conv("iso_print_conv", print_conv::iso_print_conv);

    // Composite tag PrintConv functions
    registry::register_print_conv(
        "composite_gps_gpsaltitude_print_conv",
        print_conv::composite_gps_gpsaltitude_print_conv,
    );

    // GPS coordinate ValueConv functions - convert to unsigned decimal degrees
    // Sign handling happens in Composite tags that combine coordinate + ref
    registry::register_value_conv(
        "gpslatitude_value_conv",
        value_conv::gps_coordinate_value_conv,
    );
    registry::register_value_conv(
        "gpslongitude_value_conv",
        value_conv::gps_coordinate_value_conv,
    );
    registry::register_value_conv(
        "gpsdestlatitude_value_conv",
        value_conv::gps_coordinate_value_conv,
    );
    registry::register_value_conv(
        "gpsdestlongitude_value_conv",
        value_conv::gps_coordinate_value_conv,
    );
    registry::register_value_conv(
        "gpstimestamp_value_conv",
        value_conv::gpstimestamp_value_conv,
    );
    registry::register_value_conv(
        "gpsdatestamp_value_conv",
        value_conv::gpsdatestamp_value_conv,
    );
    registry::register_value_conv(
        "whitebalance_value_conv",
        value_conv::whitebalance_value_conv,
    );

    // APEX ValueConv functions (for APEX values when we identify the tags)
    registry::register_value_conv(
        "apex_shutter_speed_value_conv",
        value_conv::apex_shutter_speed_value_conv,
    );
    registry::register_value_conv(
        "apex_aperture_value_conv",
        value_conv::apex_aperture_value_conv,
    );
    registry::register_value_conv(
        "apex_exposure_compensation_value_conv",
        value_conv::apex_exposure_compensation_value_conv,
    );
    registry::register_value_conv("fnumber_value_conv", value_conv::fnumber_value_conv);
    registry::register_value_conv(
        "exposuretime_value_conv",
        value_conv::exposuretime_value_conv,
    );
    registry::register_value_conv("focallength_value_conv", value_conv::focallength_value_conv);
}
