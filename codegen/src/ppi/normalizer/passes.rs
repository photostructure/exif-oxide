//! Individual normalization passes for AST transformation

use super::{utils, NormalizationPass};
use crate::ppi::types::PpiNode;
use tracing::trace;

/// Normalizes safe division patterns like `$val ? 1/$val : 0`
pub struct SafeDivisionNormalizer;

impl NormalizationPass for SafeDivisionNormalizer {
    fn name(&self) -> &str {
        "SafeDivisionNormalizer"
    }

    fn transform(&self, node: PpiNode) -> PpiNode {
        // Pattern: $val ? N / $val : 0
        if utils::is_ternary(&node) {
            if let Some((condition, true_branch, false_branch)) = utils::extract_ternary(&node) {
                // Check if this matches safe division pattern
                if self.matches_safe_division(&condition, &true_branch, &false_branch) {
                    trace!("Found safe division pattern");

                    // Extract numerator from true branch
                    let numerator = self.extract_numerator(&true_branch);
                    let denominator = condition[0].clone(); // The condition variable

                    // Special case: if numerator is 1, use safe_reciprocal
                    if self.is_one(&numerator) {
                        return utils::create_function_call("safe_reciprocal", vec![denominator]);
                    }

                    // General case: safe_division
                    return utils::create_function_call(
                        "safe_division",
                        vec![numerator, denominator],
                    );
                }
            }
        }

        // Recurse into children
        utils::transform_children(node, |child| self.transform(child))
    }
}

impl SafeDivisionNormalizer {
    fn matches_safe_division(
        &self,
        condition: &[PpiNode],
        true_branch: &[PpiNode],
        false_branch: &[PpiNode],
    ) -> bool {
        // Condition should be a single variable (e.g., $val)
        if condition.len() != 1 || condition[0].class != "PPI::Token::Symbol" {
            return false;
        }

        // False branch should be 0
        if false_branch.len() != 1 {
            return false;
        }
        if let Some(num_val) = false_branch[0].numeric_value {
            if num_val != 0.0 {
                return false;
            }
        } else {
            return false;
        }

        // True branch should be division with same variable
        if true_branch.len() < 3 {
            return false;
        }

        // Look for division operator
        let has_division = true_branch.iter().any(|node| {
            node.class == "PPI::Token::Operator" && node.content.as_deref() == Some("/")
        });

        if !has_division {
            return false;
        }

        // Check that denominator matches condition variable
        let cond_var = &condition[0].content;
        true_branch
            .iter()
            .any(|node| node.class == "PPI::Token::Symbol" && &node.content == cond_var)
    }

    fn extract_numerator(&self, true_branch: &[PpiNode]) -> PpiNode {
        // Find the number before the division operator
        for (i, node) in true_branch.iter().enumerate() {
            if node.class == "PPI::Token::Operator" && node.content.as_deref() == Some("/") {
                if i > 0 {
                    return true_branch[i - 1].clone();
                }
            }
        }

        // Default to 1 if not found
        PpiNode {
            class: "PPI::Token::Number".to_string(),
            content: Some("1".to_string()),
            children: vec![],
            symbol_type: None,
            numeric_value: Some(1.0),
            string_value: None,
            structure_bounds: None,
        }
    }

    fn is_one(&self, node: &PpiNode) -> bool {
        node.numeric_value == Some(1.0) || node.content.as_deref() == Some("1")
    }
}

/// Normalizes function calls to consistent structure
pub struct FunctionCallNormalizer;

impl NormalizationPass for FunctionCallNormalizer {
    fn name(&self) -> &str {
        "FunctionCallNormalizer"
    }

    fn transform(&self, node: PpiNode) -> PpiNode {
        // Pattern: word followed by arguments (e.g., "length $val")
        if node.class == "PPI::Statement" && node.children.len() >= 2 {
            if let Some(func_name) = self.extract_function_name(&node.children[0]) {
                if self.is_known_function(&func_name) {
                    trace!("Found function call pattern: {}", func_name);

                    // Collect arguments (everything after the function name)
                    let args: Vec<PpiNode> = node.children.iter().skip(1).cloned().collect();

                    return utils::create_function_call(&func_name, args);
                }
            }
        }

        // Recurse into children
        utils::transform_children(node, |child| self.transform(child))
    }
}

impl FunctionCallNormalizer {
    fn extract_function_name(&self, node: &PpiNode) -> Option<String> {
        if node.class == "PPI::Token::Word" {
            node.content.clone()
        } else {
            None
        }
    }

    fn is_known_function(&self, name: &str) -> bool {
        // Common Perl functions we want to normalize
        matches!(
            name,
            "length"
                | "int"
                | "sprintf"
                | "substr"
                | "index"
                | "join"
                | "split"
                | "unpack"
                | "pack"
                | "ord"
                | "chr"
                | "uc"
                | "lc"
                | "abs"
                | "sqrt"
                | "hex"
                | "oct"
        )
    }
}

/// Normalizes string operations (concatenation and repetition)
pub struct StringOpNormalizer;

impl NormalizationPass for StringOpNormalizer {
    fn name(&self) -> &str {
        "StringOpNormalizer"
    }

    fn transform(&self, node: PpiNode) -> PpiNode {
        if node.class == "PPI::Statement" {
            // Look for string concatenation (.)
            if let Some(concat_result) = self.try_normalize_concat(&node) {
                return concat_result;
            }

            // Look for string repetition (x)
            if let Some(repeat_result) = self.try_normalize_repeat(&node) {
                return repeat_result;
            }
        }

        // Recurse into children
        utils::transform_children(node, |child| self.transform(child))
    }
}

impl StringOpNormalizer {
    fn try_normalize_concat(&self, node: &PpiNode) -> Option<PpiNode> {
        // Find concatenation operator positions
        let concat_positions: Vec<usize> = node
            .children
            .iter()
            .enumerate()
            .filter_map(|(i, child)| {
                if child.class == "PPI::Token::Operator" && child.content.as_deref() == Some(".") {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();

        if concat_positions.is_empty() {
            return None;
        }

        trace!("Found string concatenation pattern");

        // Collect all operands between dots
        let mut operands = Vec::new();
        let mut start = 0;

        for &dot_pos in &concat_positions {
            // Collect all nodes from start to dot_pos as one operand
            let operand_children: Vec<PpiNode> = node.children[start..dot_pos].to_vec();

            if operand_children.len() == 1 {
                operands.push(operand_children[0].clone());
            } else if !operand_children.is_empty() {
                // Wrap multiple nodes in a Statement
                operands.push(PpiNode {
                    class: "PPI::Statement".to_string(),
                    content: None,
                    children: operand_children,
                    symbol_type: None,
                    numeric_value: None,
                    string_value: None,
                    structure_bounds: None,
                });
            }
            start = dot_pos + 1;
        }

        // Add final operand after last dot
        let final_children: Vec<PpiNode> = node.children[start..].to_vec();
        if final_children.len() == 1 {
            operands.push(final_children[0].clone());
        } else if !final_children.is_empty() {
            operands.push(PpiNode {
                class: "PPI::Statement".to_string(),
                content: None,
                children: final_children,
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            });
        }

        Some(PpiNode {
            class: "StringConcat".to_string(),
            content: None,
            children: operands,
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        })
    }

    fn try_normalize_repeat(&self, node: &PpiNode) -> Option<PpiNode> {
        // Find repetition operator (x)
        let repeat_pos = node.children.iter().position(|child| {
            child.class == "PPI::Token::Operator" && child.content.as_deref() == Some("x")
        })?;

        if repeat_pos > 0 && repeat_pos + 1 < node.children.len() {
            trace!("Found string repetition pattern");

            let string_operand = node.children[repeat_pos - 1].clone();
            let count_operand = node.children[repeat_pos + 1].clone();

            return Some(PpiNode {
                class: "StringRepeat".to_string(),
                content: None,
                children: vec![string_operand, count_operand],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            });
        }

        None
    }
}

/// Normalizes sprintf with string concatenation patterns into specialized function calls
///
/// Handles: sprintf("base" . "part" x count, args...)
/// From ExifTool Canon.pm line 763: sprintf("%19d %4d %6d" . " %3d %4d %6d" x 8, split(" ",$val))
///
/// This pattern combines sprintf with:
/// 1. String concatenation (.) to join format strings
/// 2. String repetition (x) to repeat parts of the format
///
/// The normalizer converts this to a specialized FunctionCall that can generate
/// direct calls to crate::fmt::sprintf_with_string_concat_repeat
pub struct SprintfNormalizer;

impl NormalizationPass for SprintfNormalizer {
    fn name(&self) -> &'static str {
        "SprintfNormalizer"
    }

    fn transform(&self, node: PpiNode) -> PpiNode {
        if let Some(normalized) = self.try_normalize_sprintf(&node) {
            trace!("Normalized sprintf with string concatenation pattern");
            normalized
        } else {
            // Recursively transform children
            let transformed_children: Vec<PpiNode> = node
                .children
                .into_iter()
                .map(|child| self.transform(child))
                .collect();

            PpiNode {
                children: transformed_children,
                ..node
            }
        }
    }
}

impl SprintfNormalizer {
    /// Detect and normalize sprintf with string concatenation patterns
    ///
    /// Pattern: sprintf(format_expr . repeat_expr, args...)
    /// Where format_expr might be: "base_format" . "concat_part" x count
    fn try_normalize_sprintf(&self, node: &PpiNode) -> Option<PpiNode> {
        // Look for sprintf function call pattern
        if !self.is_sprintf_call(node) {
            return None;
        }

        // Find the first argument to sprintf (the format expression)
        let format_arg = node.children.get(0)?;

        // Check if the format argument contains string concatenation
        if let Some((base_format, concat_part, repeat_count)) =
            self.extract_concat_repeat_pattern(format_arg)
        {
            trace!("Found sprintf with string concat/repeat pattern: base='{}', concat='{}', repeat={}", 
                   base_format, concat_part, repeat_count);

            // Create normalized sprintf node with extracted components
            let mut children = vec![
                self.create_string_node(&base_format),
                self.create_string_node(&concat_part),
                self.create_number_node(repeat_count),
            ];

            // Add remaining arguments (skip the first format argument)
            children.extend(node.children.iter().skip(1).cloned());

            return Some(PpiNode {
                class: "FunctionCall".to_string(),
                content: Some("sprintf_with_string_concat_repeat".to_string()),
                children,
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            });
        }

        None
    }

    fn is_sprintf_call(&self, node: &PpiNode) -> bool {
        // Check if this looks like: sprintf(args...)
        if node.children.is_empty() {
            return false;
        }

        // Look for function name "sprintf" in various possible locations
        if let Some(first_child) = node.children.first() {
            if first_child.class == "PPI::Token::Word" {
                return first_child.content.as_deref() == Some("sprintf");
            }
        }

        // Also check if this is already normalized as a function call
        if node.class == "FunctionCall" {
            return node.content.as_deref() == Some("sprintf");
        }

        false
    }

    /// Extract base_format, concat_part, and repeat_count from format expression
    ///
    /// Handles patterns like: "base" . "part" x count
    fn extract_concat_repeat_pattern(
        &self,
        format_node: &PpiNode,
    ) -> Option<(String, String, usize)> {
        // Look for the pattern: expr . expr x count
        // We need to find concatenation (.) and repetition (x) operators

        let mut concat_pos = None;
        let mut repeat_pos = None;

        for (i, child) in format_node.children.iter().enumerate() {
            if child.class == "PPI::Token::Operator" {
                match child.content.as_deref() {
                    Some(".") if concat_pos.is_none() => concat_pos = Some(i),
                    Some("x") => repeat_pos = Some(i),
                    _ => {}
                }
            }
        }

        // We need both concatenation and repetition for this pattern
        let concat_pos = concat_pos?;
        let repeat_pos = repeat_pos?;

        // Ensure proper ordering: base . part x count
        if concat_pos >= repeat_pos {
            return None;
        }

        // Extract components
        let base_format = self.extract_string_value(&format_node.children[0])?;
        let concat_part = self.extract_string_value(&format_node.children[concat_pos + 1])?;
        let repeat_count = self.extract_numeric_value(&format_node.children[repeat_pos + 1])?;

        Some((base_format, concat_part, repeat_count))
    }

    fn extract_string_value(&self, node: &PpiNode) -> Option<String> {
        match &node.class[..] {
            "PPI::Token::Quote::Double" | "PPI::Token::Quote::Single" => node
                .content
                .clone()
                .map(|s| s.trim_matches('"').trim_matches('\'').to_string()),
            _ => node.string_value.clone(),
        }
    }

    fn extract_numeric_value(&self, node: &PpiNode) -> Option<usize> {
        if let Some(num) = node.numeric_value {
            Some(num as usize)
        } else if let Some(ref content) = node.content {
            content.parse().ok()
        } else {
            None
        }
    }

    fn create_string_node(&self, value: &str) -> PpiNode {
        PpiNode {
            class: "PPI::Token::Quote::Double".to_string(),
            content: Some(format!("\"{}\"", value)),
            string_value: Some(value.to_string()),
            children: vec![],
            symbol_type: None,
            numeric_value: None,
            structure_bounds: None,
        }
    }

    fn create_number_node(&self, value: usize) -> PpiNode {
        PpiNode {
            class: "PPI::Token::Number".to_string(),
            content: Some(value.to_string()),
            numeric_value: Some(value as f64),
            children: vec![],
            symbol_type: None,
            string_value: None,
            structure_bounds: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_division_normalization() {
        // Create AST for: $val ? 1 / $val : 0
        let ast = PpiNode {
            class: "PPI::Statement".to_string(),
            content: None,
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
            children: vec![
                PpiNode {
                    class: "PPI::Token::Symbol".to_string(),
                    content: Some("$val".to_string()),
                    children: vec![],
                    symbol_type: Some("scalar".to_string()),
                    numeric_value: None,
                    string_value: None,
                    structure_bounds: None,
                },
                PpiNode {
                    class: "PPI::Token::Operator".to_string(),
                    content: Some("?".to_string()),
                    children: vec![],
                    symbol_type: None,
                    numeric_value: None,
                    string_value: None,
                    structure_bounds: None,
                },
                PpiNode {
                    class: "PPI::Token::Number".to_string(),
                    content: Some("1".to_string()),
                    children: vec![],
                    symbol_type: None,
                    numeric_value: Some(1.0),
                    string_value: None,
                    structure_bounds: None,
                },
                PpiNode {
                    class: "PPI::Token::Operator".to_string(),
                    content: Some("/".to_string()),
                    children: vec![],
                    symbol_type: None,
                    numeric_value: None,
                    string_value: None,
                    structure_bounds: None,
                },
                PpiNode {
                    class: "PPI::Token::Symbol".to_string(),
                    content: Some("$val".to_string()),
                    children: vec![],
                    symbol_type: Some("scalar".to_string()),
                    numeric_value: None,
                    string_value: None,
                    structure_bounds: None,
                },
                PpiNode {
                    class: "PPI::Token::Operator".to_string(),
                    content: Some(":".to_string()),
                    children: vec![],
                    symbol_type: None,
                    numeric_value: None,
                    string_value: None,
                    structure_bounds: None,
                },
                PpiNode {
                    class: "PPI::Token::Number".to_string(),
                    content: Some("0".to_string()),
                    children: vec![],
                    symbol_type: None,
                    numeric_value: Some(0.0),
                    string_value: None,
                    structure_bounds: None,
                },
            ],
        };

        let normalizer = SafeDivisionNormalizer;
        let result = normalizer.transform(ast);

        assert_eq!(result.class, "FunctionCall");
        assert_eq!(result.content.as_deref(), Some("safe_reciprocal"));
        assert_eq!(result.children.len(), 1);
    }

    #[test]
    fn test_function_call_normalization() {
        // Create AST for: length $val
        let ast = PpiNode {
            class: "PPI::Statement".to_string(),
            content: None,
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
            children: vec![
                PpiNode {
                    class: "PPI::Token::Word".to_string(),
                    content: Some("length".to_string()),
                    children: vec![],
                    symbol_type: None,
                    numeric_value: None,
                    string_value: None,
                    structure_bounds: None,
                },
                PpiNode {
                    class: "PPI::Token::Symbol".to_string(),
                    content: Some("$val".to_string()),
                    children: vec![],
                    symbol_type: Some("scalar".to_string()),
                    numeric_value: None,
                    string_value: None,
                    structure_bounds: None,
                },
            ],
        };

        let normalizer = FunctionCallNormalizer;
        let result = normalizer.transform(ast);

        assert_eq!(result.class, "FunctionCall");
        assert_eq!(result.content.as_deref(), Some("length"));
        assert_eq!(result.children.len(), 1);
    }

    #[test]
    fn test_string_concat_normalization() {
        // Create AST for: "a" . "b"
        let ast = PpiNode {
            class: "PPI::Statement".to_string(),
            content: None,
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
            children: vec![
                PpiNode {
                    class: "PPI::Token::Quote::Double".to_string(),
                    content: Some("\"a\"".to_string()),
                    children: vec![],
                    symbol_type: None,
                    numeric_value: None,
                    string_value: Some("a".to_string()),
                    structure_bounds: None,
                },
                PpiNode {
                    class: "PPI::Token::Operator".to_string(),
                    content: Some(".".to_string()),
                    children: vec![],
                    symbol_type: None,
                    numeric_value: None,
                    string_value: None,
                    structure_bounds: None,
                },
                PpiNode {
                    class: "PPI::Token::Quote::Double".to_string(),
                    content: Some("\"b\"".to_string()),
                    children: vec![],
                    symbol_type: None,
                    numeric_value: None,
                    string_value: Some("b".to_string()),
                    structure_bounds: None,
                },
            ],
        };

        let normalizer = StringOpNormalizer;
        let result = normalizer.transform(ast);

        assert_eq!(result.class, "StringConcat");
        assert_eq!(result.children.len(), 2);
    }
}
