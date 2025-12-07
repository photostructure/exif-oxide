//! Test support utilities for codegen and integration tests
//!
//! This module provides utilities for testing generated code,
//! available only when the test-helpers feature is enabled.

use crate::{ExifContext, TagValue};

/// Create a basic ExifContext for testing
pub fn create_test_context() -> ExifContext {
    ExifContext::new()
}

/// Create a test context with some sample data
pub fn create_test_context_with_data() -> ExifContext {
    let mut context = ExifContext::new();
    context.set_data_member("Make", TagValue::string("Canon"));
    context.set_data_member("Model", TagValue::string("EOS R5"));
    context.set_data_member("FocalUnits", TagValue::U32(1000));
    context
}

/// Create a TagValue from common test inputs
pub fn test_tag_value(input: &str) -> TagValue {
    match input {
        "bytes" => TagValue::Binary(vec![0x12, 0x34, 0x56, 0x78]),
        "string" => TagValue::string("test string"),
        "number" => TagValue::U32(42),
        "float" => TagValue::F64(std::f64::consts::PI),
        _ => TagValue::string(input),
    }
}
