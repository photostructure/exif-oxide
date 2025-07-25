//! Generated conversion reference lists
//!
//! This file is automatically generated by codegen.
//! DO NOT EDIT MANUALLY - changes will be overwritten.

use std::collections::HashSet;
use std::sync::LazyLock;

/// All unique PrintConv references found in tag definitions
pub static PRINT_CONV_REFS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let mut set = HashSet::new();
    set.insert("2_print_conv");
    set.insert("aperture_print_conv");
    set.insert("bluebalance_print_conv");
    set.insert("cfapattern_print_conv");
    set.insert("circleofconfusion_print_conv");
    set.insert("datetimeoriginal_print_conv");
    set.insert("focallength35efl_print_conv");
    set.insert("gpsaltitude_print_conv");
    set.insert("gpsdatetime_print_conv");
    set.insert("gpsdestlatitude_print_conv");
    set.insert("gpsdestlongitude_print_conv");
    set.insert("gpslatitude_print_conv");
    set.insert("gpslongitude_print_conv");
    set.insert("gpsposition_print_conv");
    set.insert("hyperfocaldistance_print_conv");
    set.insert("imagesize_print_conv");
    set.insert("lensid_print_conv");
    set.insert("lightvalue_print_conv");
    set.insert("megapixels_print_conv");
    set.insert("redbalance_print_conv");
    set.insert("scalefactor35efl_print_conv");
    set.insert("shutterspeed_print_conv");
    set.insert("subseccreatedate_print_conv");
    set.insert("subsecdatetimeoriginal_print_conv");
    set.insert("subsecmodifydate_print_conv");
    set
});

/// All unique ValueConv references found in tag definitions
pub static VALUE_CONV_REFS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let mut set = HashSet::new();
    set.insert("2_value_conv");
    set.insert("aperture_value_conv");
    set.insert("bluebalance_value_conv");
    set.insert("cfapattern_value_conv");
    set.insert("circleofconfusion_value_conv");
    set.insert("datetimeoriginal_value_conv");
    set.insert("focallength35efl_value_conv");
    set.insert("gpsaltitude_value_conv");
    set.insert("gpsdatetime_value_conv");
    set.insert("gpsdestlatitude_value_conv");
    set.insert("gpsdestlongitude_value_conv");
    set.insert("gpslatitude_value_conv");
    set.insert("gpslongitude_value_conv");
    set.insert("gpsposition_value_conv");
    set.insert("hyperfocaldistance_value_conv");
    set.insert("imagesize_value_conv");
    set.insert("lensid_value_conv");
    set.insert("lightvalue_value_conv");
    set.insert("megapixels_value_conv");
    set.insert("previewimagesize_value_conv");
    set.insert("redbalance_value_conv");
    set.insert("scalefactor35efl_value_conv");
    set.insert("shutterspeed_value_conv");
    set
});

/// Check if a PrintConv reference exists
pub fn has_print_conv_ref(name: &str) -> bool {
    PRINT_CONV_REFS.contains(name)
}

/// Check if a ValueConv reference exists
pub fn has_value_conv_ref(name: &str) -> bool {
    VALUE_CONV_REFS.contains(name)
}

/// Get statistics about conversion references
pub fn conversion_ref_stats() -> (usize, usize) {
    (PRINT_CONV_REFS.len(), VALUE_CONV_REFS.len())
}
