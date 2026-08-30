//! Operand-rendering helpers for normalized binary operations.
//!
//! Operator recognition and precedence belong exclusively to
//! `ExpressionPrecedenceNormalizer`; the visitor consumes its typed nodes.

/// Wrap literals for string concatenation - both numeric and string literals need TagValue conversion.
/// For concat(), both operands must be &TagValue, so we need to wrap:
/// - String literals like "0x" -> TagValue::string("0x")
/// - Numeric literals like 100i32 -> Into::<TagValue>::into(100i32)
pub fn wrap_for_string_concat(s: &str) -> String {
    let trimmed = s.trim();
    // String literals (surrounded by quotes)
    if trimmed.starts_with('"') && trimmed.ends_with('"') {
        format!("TagValue::string({trimmed})")
    }
    // Numeric literals
    else if trimmed.ends_with("i32") || trimmed.ends_with("f64") || trimmed.ends_with("u32") {
        format!("Into::<TagValue>::into({trimmed})")
    }
    // Already a complex expression or TagValue - use as-is
    else {
        s.to_string()
    }
}

/// Check if a condition string is already a boolean expression (comparison, etc.)
pub fn is_boolean_expression(s: &str) -> bool {
    s.contains("==")
        || s.contains("!=")
        || s.contains("<=")
        || s.contains(">=")
        || s.contains(".is_truthy()")
        || s.contains(".is_empty()")
        || s.contains(".contains(")
        || s.contains(".is_match(")
        || s.starts_with('!')      // Negation: !expr
        || s.starts_with("(!")     // Negation in parens: (!expr)
        // Simple < and > need special handling to avoid matching << and >>
        || (s.contains('<') && !s.contains("<<") && !s.contains("<="))
        || (s.contains('>') && !s.contains(">>") && !s.contains(">="))
}

/// Wrap a ternary condition with .is_truthy() if needed.
/// In Perl, `$val ? ... : ...` checks truthiness (non-zero, non-empty).
/// Also handles expressions like `($val & 0x01)` that return TagValue.
pub fn wrap_condition_for_bool(condition: &str) -> String {
    // Already a boolean expression - no wrapping needed
    if is_boolean_expression(condition) {
        return condition.to_string();
    }

    // Bare variable reference
    if condition == "val" || condition == "val_pt" {
        return format!("{condition}.is_truthy()");
    }

    // Expressions involving val that produce TagValue need is_truthy()
    if condition.contains("val") || condition.contains("val_pt") {
        return format!("({condition}).is_truthy()");
    }

    // Context lookups ($$self{Field}) return TagValue, need is_truthy()
    if condition.contains("ctx.and_then") || condition.contains("get_data_member") {
        return format!("({condition}).is_truthy()");
    }

    condition.to_string()
}

/// Wrap a ternary branch with appropriate conversion for ownership.
///
/// - Bare variable references need .clone()
/// - Bare integer/float literals need explicit TagValue conversion
/// - String literals need explicit TagValue conversion
///
/// Uses turbofish syntax to avoid type inference ambiguity.
pub fn wrap_branch_for_owned(branch: &str) -> String {
    if branch == "val" || branch == "val_pt" {
        format!("{branch}.clone()")
    } else if branch.ends_with("i32") || branch.ends_with("u32") || branch.ends_with("f64") {
        // Bare integer/float literal - use explicit TagValue conversion
        format!("Into::<TagValue>::into({branch})")
    } else if branch.starts_with('"') && branch.ends_with('"') {
        // String literal - use explicit TagValue conversion
        format!("Into::<TagValue>::into({branch})")
    } else {
        branch.to_string()
    }
}
