//! Generated conversion reference lists
//!
//! This file is automatically generated by codegen.
//! DO NOT EDIT MANUALLY - changes will be overwritten.

use std::collections::HashSet;
use std::sync::LazyLock;

/// All unique PrintConv references found in tag definitions
pub static PRINT_CONV_REFS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let mut set = HashSet::new();
    set.insert("ambienttemperature_print_conv");
    set.insert("aperturevalue_print_conv");
    set.insert("cfapattern_print_conv");
    set.insert("cfaplanecolor_print_conv");
    set.insert("chromaticaberrationcorrection_print_conv");
    set.insert("colorspace_print_conv");
    set.insert("compositeimage_print_conv");
    set.insert("contrast_print_conv");
    set.insert("createdate_print_conv");
    set.insert("customrendered_print_conv");
    set.insert("datetimeoriginal_print_conv");
    set.insert("distortioncorrection_print_conv");
    set.insert("dngbackwardversion_print_conv");
    set.insert("dngversion_print_conv");
    set.insert("exposuremode_print_conv");
    set.insert("exposureprogram_print_conv");
    set.insert("exposuretime_print_conv");
    set.insert("filesource_print_conv");
    set.insert("flash_print_conv");
    set.insert("fnumber_print_conv");
    set.insert("focallength_print_conv");
    set.insert("focallengthin35mmformat_print_conv");
    set.insert("focalplaneresolutionunit_print_conv");
    set.insert("framerate_print_conv");
    set.insert("gaincontrol_print_conv");
    set.insert("interopindex_print_conv");
    set.insert("iso_print_conv");
    set.insert("lensinfo_print_conv");
    set.insert("lightsource_print_conv");
    set.insert("maxaperturevalue_print_conv");
    set.insert("meteringmode_print_conv");
    set.insert("modifydate_print_conv");
    set.insert("orientation_print_conv");
    set.insert("photometricinterpretation_print_conv");
    set.insert("planarconfiguration_print_conv");
    set.insert("previewdatetime_print_conv");
    set.insert("resolutionunit_print_conv");
    set.insert("saturation_print_conv");
    set.insert("scenecapturetype_print_conv");
    set.insert("scenetype_print_conv");
    set.insert("sensingmethod_print_conv");
    set.insert("sensitivitytype_print_conv");
    set.insert("sharpness_print_conv");
    set.insert("shutterspeedvalue_print_conv");
    set.insert("sonyrawfiletype_print_conv");
    set.insert("spatialfrequencyresponse_print_conv");
    set.insert("subfiletype_print_conv");
    set.insert("subjectdistance_print_conv");
    set.insert("subjectdistancerange_print_conv");
    set.insert("vignettingcorrection_print_conv");
    set.insert("whitebalance_print_conv");
    set.insert("ycbcrpositioning_print_conv");
    set.insert("ycbcrsubsampling_print_conv");
    set.insert("gpsaltitude_print_conv");
    set.insert("gpsaltituderef_print_conv");
    set.insert("gpsdestbearingref_print_conv");
    set.insert("gpsdestdistanceref_print_conv");
    set.insert("gpsdestlatitude_print_conv");
    set.insert("gpsdestlatituderef_print_conv");
    set.insert("gpsdestlongitude_print_conv");
    set.insert("gpsdestlongituderef_print_conv");
    set.insert("gpsdifferential_print_conv");
    set.insert("gpshpositioningerror_print_conv");
    set.insert("gpsimgdirectionref_print_conv");
    set.insert("gpslatitude_print_conv");
    set.insert("gpslatituderef_print_conv");
    set.insert("gpslongitude_print_conv");
    set.insert("gpslongituderef_print_conv");
    set.insert("gpsmeasuremode_print_conv");
    set.insert("gpsspeedref_print_conv");
    set.insert("gpsstatus_print_conv");
    set.insert("gpstimestamp_print_conv");
    set.insert("gpstrackref_print_conv");
    set.insert("aperture_print_conv");
    set.insert("bluebalance_print_conv");
    set.insert("circleofconfusion_print_conv");
    set.insert("focallength35efl_print_conv");
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
    set.insert("gpsdatetime_print_conv");
    set
});

/// All unique ValueConv references found in tag definitions
pub static VALUE_CONV_REFS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let mut set = HashSet::new();
    set.insert("aperturevalue_value_conv");
    set.insert("brightness_value_conv");
    set.insert("contrast_value_conv");
    set.insert("converter_value_conv");
    set.insert("lens_value_conv");
    set.insert("maxaperturevalue_value_conv");
    set.insert("ownername_value_conv");
    set.insert("previewdatetime_value_conv");
    set.insert("rawdatauniqueid_value_conv");
    set.insert("saturation_value_conv");
    set.insert("serialnumber_value_conv");
    set.insert("shadows_value_conv");
    set.insert("sharpness_value_conv");
    set.insert("shutterspeedvalue_value_conv");
    set.insert("subsectime_value_conv");
    set.insert("subsectimedigitized_value_conv");
    set.insert("subsectimeoriginal_value_conv");
    set.insert("tilebytecounts_value_conv");
    set.insert("tileoffsets_value_conv");
    set.insert("whitebalance_value_conv");
    set.insert("xpauthor_value_conv");
    set.insert("xpcomment_value_conv");
    set.insert("xpkeywords_value_conv");
    set.insert("xpsubject_value_conv");
    set.insert("xptitle_value_conv");
    set.insert("gpsdatestamp_value_conv");
    set.insert("gpsdestlatitude_value_conv");
    set.insert("gpsdestlongitude_value_conv");
    set.insert("gpslatitude_value_conv");
    set.insert("gpslongitude_value_conv");
    set.insert("gpstimestamp_value_conv");
    set.insert("aperture_value_conv");
    set.insert("bluebalance_value_conv");
    set.insert("cfapattern_value_conv");
    set.insert("circleofconfusion_value_conv");
    set.insert("datetimeoriginal_value_conv");
    set.insert("focallength35efl_value_conv");
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
    set.insert("gpsaltitude_value_conv");
    set.insert("gpsdatetime_value_conv");
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
