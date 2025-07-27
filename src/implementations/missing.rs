//! Track missing PrintConv/ValueConv implementations for --show-missing
//!
//! This module provides tracking for PrintConv/ValueConv expressions that don't
//! have implementations in the registry. This helps identify what needs to be
//! implemented for better compatibility.

use crate::types::TagValue;
use std::sync::{LazyLock, Mutex};

#[derive(Debug, Clone)]
pub struct MissingConversion {
    pub tag_id: u32,
    pub tag_name: String,
    pub group: String,
    pub expression: String,
    pub conv_type: ConversionType,
}

#[derive(Debug, Clone)]
pub enum ConversionType {
    PrintConv,
    ValueConv,
}

static MISSING_CONVERSIONS: LazyLock<Mutex<Vec<MissingConversion>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

/// Record a missing PrintConv implementation
pub fn missing_print_conv(
    tag_id: u32,
    tag_name: &str,
    group: &str,
    expr: &str,
    value: &TagValue,
) -> TagValue {
    let mut missing = MISSING_CONVERSIONS.lock().unwrap();

    // Only record each unique expression once
    let already_recorded = missing
        .iter()
        .any(|m| m.expression == expr && matches!(m.conv_type, ConversionType::PrintConv));

    if !already_recorded {
        missing.push(MissingConversion {
            tag_id,
            tag_name: tag_name.to_string(),
            group: group.to_string(),
            expression: expr.to_string(),
            conv_type: ConversionType::PrintConv,
        });
    }

    value.clone()
}

/// Record a missing ValueConv implementation
pub fn missing_value_conv(
    tag_id: u32,
    tag_name: &str,
    group: &str,
    expr: &str,
    value: &TagValue,
) -> TagValue {
    let mut missing = MISSING_CONVERSIONS.lock().unwrap();

    let already_recorded = missing
        .iter()
        .any(|m| m.expression == expr && matches!(m.conv_type, ConversionType::ValueConv));

    if !already_recorded {
        missing.push(MissingConversion {
            tag_id,
            tag_name: tag_name.to_string(),
            group: group.to_string(),
            expression: expr.to_string(),
            conv_type: ConversionType::ValueConv,
        });
    }

    value.clone()
}

/// Get all missing conversions for --show-missing
pub fn get_missing_conversions() -> Vec<MissingConversion> {
    MISSING_CONVERSIONS.lock().unwrap().clone()
}

/// Clear missing conversions (useful for testing)
pub fn clear_missing_conversions() {
    MISSING_CONVERSIONS.lock().unwrap().clear()
}
