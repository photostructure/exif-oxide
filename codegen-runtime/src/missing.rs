//! Track missing PrintConv/ValueConv implementations for development
//!
//! This module provides tracking for PrintConv/ValueConv expressions that don't
//! have implementations in the registry. This helps identify what needs to be
//! implemented for better compatibility when using the --show-missing flag.

use crate::tag_value::TagValue;
use std::collections::HashSet;
use std::cell::RefCell;

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

/// Use a HashSet to track which expressions we've already seen to avoid duplicates
/// and a Vec to store the actual conversions. This avoids linear searches.
struct MissingConversionsState {
    conversions: Vec<MissingConversion>,
    seen_expressions: HashSet<(String, bool)>, // (expression, is_print_conv)
}

// Use thread-local storage to ensure test isolation. Each test thread gets its own
// independent copy of missing conversions state, preventing race conditions between
// concurrent tests while maintaining performance.
thread_local! {
    static MISSING_CONVERSIONS: RefCell<MissingConversionsState> = RefCell::new(
        MissingConversionsState {
            conversions: Vec::new(),
            seen_expressions: HashSet::new(),
        }
    );
}

/// Record a missing PrintConv implementation
///
/// This is called by generated placeholder functions when they can't translate
/// a Perl expression. The information is collected for --show-missing reporting.
pub fn missing_print_conv(
    tag_id: u32,
    tag_name: &str,
    group: &str,
    expr: &str,
    value: &TagValue,
) -> TagValue {
    MISSING_CONVERSIONS.with(|cell| {
        let mut state = cell.borrow_mut();
        
        // Only record each unique expression once (per type)
        let key = (expr.to_string(), true); // true = PrintConv
        if !state.seen_expressions.contains(&key) {
            state.seen_expressions.insert(key);
            state.conversions.push(MissingConversion {
                tag_id,
                tag_name: tag_name.to_string(),
                group: group.to_string(),
                expression: expr.to_string(),
                conv_type: ConversionType::PrintConv,
            });
        }
    });

    value.clone()
}

/// Record a missing ValueConv implementation
///
/// This is called by generated placeholder functions when they can't translate
/// a Perl expression. The information is collected for --show-missing reporting.
pub fn missing_value_conv(
    tag_id: u32,
    tag_name: &str,
    group: &str,
    expr: &str,
    value: &TagValue,
) -> TagValue {
    MISSING_CONVERSIONS.with(|cell| {
        let mut state = cell.borrow_mut();
        
        // Only record each unique expression once (per type)
        let key = (expr.to_string(), false); // false = ValueConv
        if !state.seen_expressions.contains(&key) {
            state.seen_expressions.insert(key);
            state.conversions.push(MissingConversion {
                tag_id,
                tag_name: tag_name.to_string(),
                group: group.to_string(),
                expression: expr.to_string(),
                conv_type: ConversionType::ValueConv,
            });
        }
    });

    value.clone()
}

/// Get all missing conversions for --show-missing
pub fn get_missing_conversions() -> Vec<MissingConversion> {
    MISSING_CONVERSIONS.with(|cell| {
        cell.borrow().conversions.clone()
    })
}

/// Clear missing conversions (useful for testing)
pub fn clear_missing_conversions() {
    MISSING_CONVERSIONS.with(|cell| {
        let mut state = cell.borrow_mut();
        state.conversions.clear();
        state.seen_expressions.clear();
    });
}
