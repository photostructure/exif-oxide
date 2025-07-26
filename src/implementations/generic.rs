//! Generic PrintConv handling for Simple lookups and fallback cases

use crate::types::TagValue;
use std::collections::HashMap;

/// Apply fallback PrintConv handling for any PrintConvType
/// This avoids duplicating the match logic across all generated modules
pub fn apply_fallback_print_conv(
    tag_id: u32,
    value: &TagValue,
    print_conv_type: PrintConvTypeRef,
) -> TagValue {
    match print_conv_type {
        PrintConvTypeRef::None => value.clone(),
        PrintConvTypeRef::Simple(lookup) => apply_simple_print_conv(value, lookup),
        PrintConvTypeRef::Expression(expr) => {
            crate::implementations::missing::missing_print_conv(tag_id, expr, value)
        }
        PrintConvTypeRef::Manual(func_name) => {
            crate::implementations::missing::missing_print_conv(tag_id, func_name, value)
        }
    }
}

/// Reference to PrintConvType variants that works across all modules
pub enum PrintConvTypeRef<'a> {
    None,
    Simple(&'a HashMap<String, &'static str>),
    Expression(&'a str),
    Manual(&'a str),
}

/// Convert any module's PrintConvType to PrintConvTypeRef
/// This macro works because all PrintConvType enums have the same structure
#[macro_export]
macro_rules! to_print_conv_ref {
    ($print_conv:expr) => {
        match $print_conv {
            PrintConvType::None => $crate::implementations::generic::PrintConvTypeRef::None,
            PrintConvType::Simple(lookup) => {
                $crate::implementations::generic::PrintConvTypeRef::Simple(lookup)
            }
            PrintConvType::Expression(expr) => {
                $crate::implementations::generic::PrintConvTypeRef::Expression(expr)
            }
            PrintConvType::Manual(func_name) => {
                $crate::implementations::generic::PrintConvTypeRef::Manual(func_name)
            }
        }
    };
}

/// Apply Simple PrintConv lookup - converts value to string key and looks up in HashMap
/// Returns the looked up value or "Unknown (original_value)" if not found
pub fn apply_simple_print_conv(
    value: &TagValue,
    lookup: &HashMap<String, &'static str>,
) -> TagValue {
    let key = match value {
        TagValue::U8(v) => v.to_string(),
        TagValue::U16(v) => v.to_string(),
        TagValue::U32(v) => v.to_string(),
        TagValue::I16(v) => v.to_string(),
        TagValue::I32(v) => v.to_string(),
        TagValue::String(s) => s.clone(),
        _ => return value.clone(),
    };

    if let Some(result) = lookup.get(&key) {
        TagValue::String(result.to_string())
    } else {
        TagValue::String(format!("Unknown ({})", value))
    }
}
