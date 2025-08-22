//! Tests for TagValue functionality

use super::TagValue;

#[test]
fn test_string_creation() {
    let tag1: TagValue = "Hello".into();
    let tag2 = TagValue::from("World");
    let tag3 = TagValue::string("Foo");
    let tag4 = TagValue::String("Bar".to_string());

    assert_eq!(tag1, TagValue::String("Hello".to_string()));
    assert_eq!(tag2, TagValue::String("World".to_string()));
    assert_eq!(tag3, TagValue::String("Foo".to_string()));
    assert_eq!(tag4, TagValue::String("Bar".to_string()));
}

#[test]
fn test_numeric_conversion() {
    let u8_val = TagValue::U8(42);
    let u16_val = TagValue::U16(1000);
    let u32_val = TagValue::U32(100000);

    assert_eq!(u8_val.as_u8(), Some(42));
    assert_eq!(u8_val.as_u16(), Some(42));
    assert_eq!(u8_val.as_u32(), Some(42));

    assert_eq!(u16_val.as_u8(), None);
    assert_eq!(u16_val.as_u16(), Some(1000));
    assert_eq!(u16_val.as_u32(), Some(1000));

    assert_eq!(u32_val.as_u8(), None);
    assert_eq!(u32_val.as_u16(), None);
    assert_eq!(u32_val.as_u32(), Some(100000));
}

#[test]
fn test_rational_conversion() {
    let rational = TagValue::Rational(1, 2);
    let srational = TagValue::SRational(-1, 2);

    assert_eq!(rational.as_rational(), Some((1, 2)));
    assert_eq!(rational.as_f64(), Some(0.5));

    assert_eq!(srational.as_srational(), Some((-1, 2)));
    assert_eq!(srational.as_f64(), Some(-0.5));
}

#[test]
fn test_string_with_numeric_detection() {
    // Numeric strings should become numeric values
    assert_eq!(TagValue::string_with_numeric_detection("14"), TagValue::U16(14));
    assert_eq!(TagValue::string_with_numeric_detection("14.0"), TagValue::F64(14.0));
    assert_eq!(TagValue::string_with_numeric_detection("-5"), TagValue::I16(-5));

    // Non-numeric strings should remain strings
    assert_eq!(
        TagValue::string_with_numeric_detection("24.0 mm"),
        TagValue::String("24.0 mm".to_string())
    );
    assert_eq!(
        TagValue::string_with_numeric_detection("Hello"),
        TagValue::String("Hello".to_string())
    );
}