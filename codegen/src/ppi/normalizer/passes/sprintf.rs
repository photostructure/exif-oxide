//! Sprintf pattern normalization for AST transformation
//!
//! Transforms sprintf with string concatenation patterns into specialized function calls

use crate::ppi::normalizer::{NormalizationPass, PrecedenceLevel};
use crate::ppi::types::PpiNode;
use tracing::trace;

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
    fn name(&self) -> &str {
        "SprintfNormalizer"
    }

    fn precedence_level(&self) -> PrecedenceLevel {
        PrecedenceLevel::Low // Level 22+ - complex sprintf patterns
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
