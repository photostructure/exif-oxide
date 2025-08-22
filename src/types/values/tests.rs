//! Tests for TagValue functionality

use super::*;
use std::collections::HashMap;

#[test]
fn test_tagvalue_from_str() {
    let tag_value: TagValue = "Hello".into();
    assert_eq!(tag_value, TagValue::String("Hello".to_string()));
}

#[test]
fn test_tagvalue_from_string() {
    let s = "World".to_string();
    let tag_value = TagValue::from(s);
    assert_eq!(tag_value, TagValue::String("World".to_string()));
}

#[test]
fn test_tagvalue_from_string_ref() {
    let s = "Test".to_string();
    let tag_value = TagValue::from(&s);
    assert_eq!(tag_value, TagValue::String("Test".to_string()));
}

#[test]
fn test_tagvalue_string_method() {
    let tag_value = TagValue::string("Convenience");
    assert_eq!(tag_value, TagValue::String("Convenience".to_string()));
}

#[test]
fn test_tagvalue_string_method_with_owned_string() {
    let s = "Owned".to_string();
    let tag_value = TagValue::string(s);
    assert_eq!(tag_value, TagValue::String("Owned".to_string()));
}

#[test]
fn test_all_string_creation_methods_equivalent() {
    let str_literal = "test";

    let tag1: TagValue = str_literal.into();
    let tag2 = TagValue::from(str_literal);
    let tag3 = TagValue::string(str_literal);
    let tag4 = TagValue::String(str_literal.to_string());

    assert_eq!(tag1, tag2);
    assert_eq!(tag2, tag3);
    assert_eq!(tag3, tag4);
}

#[test]
fn test_object_variant() {
    let mut map = HashMap::new();
    map.insert("city".to_string(), TagValue::string("New York"));
    map.insert("country".to_string(), TagValue::string("USA"));

    let tag_value = TagValue::Object(map);

    assert!(tag_value.as_object().is_some());
    assert_eq!(tag_value.as_object().unwrap().len(), 2);
    assert_eq!(
        tag_value
            .as_object()
            .unwrap()
            .get("city")
            .unwrap()
            .as_string(),
        Some("New York")
    );
}

#[test]
fn test_array_variant() {
    let values = vec![
        TagValue::string("keyword1"),
        TagValue::string("keyword2"),
        TagValue::U32(123),
    ];

    let tag_value = TagValue::Array(values);

    assert!(tag_value.as_array().is_some());
    assert_eq!(tag_value.as_array().unwrap().len(), 3);
    assert_eq!(
        tag_value.as_array().unwrap()[0].as_string(),
        Some("keyword1")
    );
}

#[test]
fn test_nested_structures() {
    // Test nested XMP-like structure
    let mut contact_info = HashMap::new();
    contact_info.insert("CiAdrCity".to_string(), TagValue::string("Paris"));
    contact_info.insert("CiAdrCtry".to_string(), TagValue::string("France"));

    let mut main_object = HashMap::new();
    main_object.insert("ContactInfo".to_string(), TagValue::Object(contact_info));
    main_object.insert(
        "Keywords".to_string(),
        TagValue::Array(vec![TagValue::string("travel"), TagValue::string("europe")]),
    );

    let xmp = TagValue::Object(main_object);

    // Test access to nested data
    let contact = xmp
        .as_object()
        .unwrap()
        .get("ContactInfo")
        .unwrap()
        .as_object()
        .unwrap();
    assert_eq!(contact.get("CiAdrCity").unwrap().as_string(), Some("Paris"));

    let keywords = xmp
        .as_object()
        .unwrap()
        .get("Keywords")
        .unwrap()
        .as_array()
        .unwrap();
    assert_eq!(keywords.len(), 2);
}

#[test]
fn test_display_formatting() {
    // Test Object display
    let mut map = HashMap::new();
    map.insert("key1".to_string(), TagValue::string("value1"));
    map.insert("key2".to_string(), TagValue::U32(42));
    let obj = TagValue::Object(map);
    let display = format!("{obj}");
    assert!(display.contains(r#""key1": value1"#) || display.contains(r#""key2": 42"#));

    // Test Array display
    let arr = TagValue::Array(vec![TagValue::string("item1"), TagValue::U32(123)]);
    assert_eq!(format!("{arr}"), "[item1, 123]");
}

#[test]
fn test_string_with_numeric_detection() {
    // Numeric strings should become F64 values
    assert_eq!(
        TagValue::string_with_numeric_detection("14.0"),
        TagValue::F64(14.0)
    );
    assert_eq!(
        TagValue::string_with_numeric_detection("2.8"),
        TagValue::F64(2.8)
    );
    assert_eq!(
        TagValue::string_with_numeric_detection("-5.2"),
        TagValue::F64(-5.2)
    );
    assert_eq!(
        TagValue::string_with_numeric_detection("42"),
        TagValue::U16(42)
    );

    // Scientific notation
    assert_eq!(
        TagValue::string_with_numeric_detection("1.23e4"),
        TagValue::F64(12300.0)
    );
    assert_eq!(
        TagValue::string_with_numeric_detection("1.5e-3"),
        TagValue::F64(0.0015)
    );

    // Non-numeric strings should remain strings
    assert_eq!(
        TagValue::string_with_numeric_detection("24.0 mm"),
        TagValue::String("24.0 mm".to_string())
    );
    assert_eq!(
        TagValue::string_with_numeric_detection("1/200"),
        TagValue::String("1/200".to_string())
    );
    assert_eq!(
        TagValue::string_with_numeric_detection("f/2.8"),
        TagValue::String("f/2.8".to_string())
    );
    assert_eq!(
        TagValue::string_with_numeric_detection("text"),
        TagValue::String("text".to_string())
    );

    // Edge cases
    assert_eq!(
        TagValue::string_with_numeric_detection("0"),
        TagValue::U16(0)
    );
    assert_eq!(
        TagValue::string_with_numeric_detection("0.0"),
        TagValue::F64(0.0)
    );

    // Leading zeros not allowed for multi-digit numbers (ExifTool regex constraint)
    assert_eq!(
        TagValue::string_with_numeric_detection("01.5"),
        TagValue::String("01.5".to_string())
    );
}

#[test]
fn test_tagvalue_arithmetic() {
    // Test division operations
    let val = TagValue::I32(100);
    assert_eq!(&val / 4, TagValue::I32(25));
    
    let val = TagValue::F64(10.0);
    assert_eq!(&val / 4, TagValue::F64(2.5));
    
    // Test string to number conversion
    let val = TagValue::String("256".to_string());
    assert_eq!(&val / 4, TagValue::F64(64.0));
    
    // Test non-numeric string fallback
    let val = TagValue::String("hello".to_string());
    assert_eq!(&val / 4, TagValue::String("(hello / 4)".to_string()));

    // Test multiplication
    let val = TagValue::I32(25);
    assert_eq!(&val * 4, TagValue::I32(100));
    
    // Test addition
    let val = TagValue::I32(10);
    assert_eq!(&val + 5, TagValue::I32(15));
    
    // Test subtraction
    let val = TagValue::I32(20);
    assert_eq!(&val - 5, TagValue::I32(15));

    // Test rational arithmetic
    let val = TagValue::Rational(100, 4); // 25.0
    assert_eq!(&val / 5, TagValue::F64(5.0));
}