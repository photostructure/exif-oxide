//! Perl division semantics in generated conversions
//!
//! Perl's `/` is always floating-point division (perlop, "Multiplicative
//! Operators"); integer division only happens under a lexical `use integer`,
//! which ExifTool never enables in a conversion. Generated conversions that
//! divide must therefore keep the fractional part.
//!
//! Ground truth for Canon SelfTimer (third-party/exiftool/lib/Image/ExifTool/Canon.pm:2231):
//!
//! ```text
//! $ perl -e '$val=15;    print((($val&0xfff)/10)." s".($val&0x4000 ? ", Custom" : ""))'
//! 1.5 s
//! $ perl -e '$val=100;   print((($val&0xfff)/10)." s")'
//! 10 s
//! $ perl -e '$val=0x4014;print((($val&0xfff)/10)." s".($val&0x4000 ? ", Custom" : ""))'
//! 2 s, Custom
//! ```

use exif_oxide::generated::Canon_pm::camera_settings_tags::CANON_CAMERASETTINGS_TAGS;
use exif_oxide::types::{PrintConv, TagValue};

/// Canon::CameraSettings tag 2 is SelfTimer. Resolve the PrintConv through the
/// generated table rather than by hashed function name, so a codegen re-run that
/// renames the function does not silently skip this test.
fn self_timer_print(raw: TagValue) -> String {
    let info = CANON_CAMERASETTINGS_TAGS
        .get(&2)
        .expect("Canon::CameraSettings tag 2 exists");
    assert_eq!(info.name, "SelfTimer");
    match info.print_conv {
        Some(PrintConv::Function(f)) => f(&raw, None).to_string(),
        ref other => panic!("SelfTimer PrintConv is not a generated function: {other:?}"),
    }
}

#[test]
fn test_canon_self_timer_keeps_fractional_seconds() {
    // The bug: integer-preserving division rendered this as "1 s".
    assert_eq!(self_timer_print(TagValue::U16(15)), "1.5 s");
    assert_eq!(self_timer_print(TagValue::I32(15)), "1.5 s");
    assert_eq!(self_timer_print(TagValue::U32(15)), "1.5 s");
}

#[test]
fn test_canon_self_timer_whole_seconds_have_no_trailing_zero() {
    // Perl prints 100/10 as "10", not "10.0".
    assert_eq!(self_timer_print(TagValue::U16(100)), "10 s");
    assert_eq!(self_timer_print(TagValue::U16(20)), "2 s");
}

#[test]
fn test_canon_self_timer_custom_bit_and_off() {
    assert_eq!(self_timer_print(TagValue::U16(0x4014)), "2 s, Custom");
    assert_eq!(self_timer_print(TagValue::U16(0)), "Off");
}

/// The other half of the fix: whole-number floats must serialize as JSON
/// integers, matching Perl's stringification.
///
/// ```text
/// $ third-party/exiftool/exiftool -FNumber# -FocalLength# -ExposureTime# -j \
///       test-images/canon/eos_60d.jpg
///   "FNumber": 4, "FocalLength": 55, "ExposureTime": 0.0005
/// ```
#[test]
fn test_whole_number_floats_serialize_as_json_integers() {
    assert_eq!(serde_json::to_string(&TagValue::F64(4.0)).unwrap(), "4");
    assert_eq!(serde_json::to_string(&TagValue::F64(55.0)).unwrap(), "55");
    assert_eq!(serde_json::to_string(&TagValue::F64(2.8)).unwrap(), "2.8");
    assert_eq!(
        serde_json::to_string(&TagValue::F64(0.0005)).unwrap(),
        "0.0005"
    );
}
