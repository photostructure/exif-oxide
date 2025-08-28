//! Join/Unpack Pattern Pass - Multi-Token Recognition
//!
//! This pass handles complex multi-token patterns that span multiple sibling nodes,
//! specifically the join+unpack combination commonly found in ExifTool expressions.
//!
//! Pattern: `join " ", unpack "H2H2", val`
//! PPI parsing: [join, " ", unpack, "H2H2", val] as flat siblings
//! Required: FunctionCall(join, [" ", FunctionCall(unpack, ["H2H2", val])])
//!
//! This is the core pattern that single-node processing cannot handle, requiring
//! multi-token pattern recognition across sibling nodes.

use crate::ppi::normalizer::{multi_pass::RewritePass, utils};
use crate::ppi::types::PpiNode;
use tracing::{debug, trace};

/// Recognizes and transforms join+unpack multi-token patterns
///
/// ExifTool source: Multiple modules use `join " ", unpack "format", data` patterns
/// for formatting binary data as hexadecimal strings with separators.
///
/// Examples from ExifTool:
/// - `join " ", unpack "H2H2", $val` → "12 34" from binary [0x12, 0x34]
/// - `join ":", unpack "H2*", $val` → "12:34:56" from binary data
///
/// This pass runs last in the multi-pass pipeline to handle combinations
/// of patterns that may have been normalized by earlier passes.
pub struct JoinUnpackPass;

impl RewritePass for JoinUnpackPass {
    fn name(&self) -> &str {
        "JoinUnpackPass"
    }

    fn transform(&self, node: PpiNode) -> PpiNode {
        // Only process PPI::Statement nodes that could contain the pattern
        if node.class != "PPI::Statement" {
            return node;
        }

        // Look for join + separator + unpack + format + data pattern
        if let Some(nested_call) = self.recognize_join_unpack_pattern(&node) {
            trace!("Found join+unpack pattern, creating nested function call");
            return nested_call;
        }

        // No pattern matched, return node unchanged
        node
    }
}

impl JoinUnpackPass {
    /// Recognize join + unpack multi-token pattern in statement children
    ///
    /// Pattern structure:
    /// 1. PPI::Token::Word with content "join"
    /// 2. Separator (string literal or variable)
    /// 3. Optional comma operator
    /// 4. PPI::Token::Word with content "unpack"  
    /// 5. Format string (usually quoted string like "H2H2")
    /// 6. Optional comma operator
    /// 7. Data source (variable like "$val" or "val")
    ///
    /// Returns nested function call structure if pattern matches
    fn recognize_join_unpack_pattern(&self, node: &PpiNode) -> Option<PpiNode> {
        let children = &node.children;

        // Need at least 5 tokens: join, separator, unpack, format, data
        // (commas are optional and may be present)
        if children.len() < 5 {
            return None;
        }

        // Find join function at start
        let join_pos = self.find_join_function(children)?;

        // Find unpack function after join
        let unpack_pos = self.find_unpack_function(children, join_pos + 1)?;

        // Extract pattern components
        let separator = self.extract_separator(children, join_pos, unpack_pos)?;
        let (format, data) = self.extract_unpack_args(children, unpack_pos)?;

        // Create nested function call structure
        let unpack_call = utils::create_function_call("unpack", vec![format, data]);
        let join_call = utils::create_function_call("join", vec![separator, unpack_call]);

        debug!("Transformed join+unpack pattern into nested function calls");
        Some(join_call)
    }

    /// Find position of "join" function word in children
    fn find_join_function(&self, children: &[PpiNode]) -> Option<usize> {
        children.iter().position(|child| {
            child.class == "PPI::Token::Word"
                && child.content.as_ref().map_or(false, |c| c == "join")
        })
    }

    /// Find position of "unpack" function word after given start position
    fn find_unpack_function(&self, children: &[PpiNode], start_pos: usize) -> Option<usize> {
        children
            .iter()
            .skip(start_pos)
            .position(|child| {
                child.class == "PPI::Token::Word"
                    && child.content.as_ref().map_or(false, |c| c == "unpack")
            })
            .map(|pos| pos + start_pos)
    }

    /// Extract separator argument between join and unpack
    ///
    /// The separator is typically the first argument after "join", which can be:
    /// - String literal: "\" \"", "\":\"", etc.
    /// - Variable: $sep, etc.
    fn extract_separator(
        &self,
        children: &[PpiNode],
        join_pos: usize,
        unpack_pos: usize,
    ) -> Option<PpiNode> {
        // Look for separator between join and unpack positions
        for i in (join_pos + 1)..unpack_pos {
            let child = &children[i];

            // Skip comma operators
            if child.class == "PPI::Token::Operator"
                && child.content.as_ref().map_or(false, |c| c == ",")
            {
                continue;
            }

            // Found potential separator - string, variable, or other value
            if self.is_value_node(child) {
                return Some(child.clone());
            }
        }

        None
    }

    /// Extract format and data arguments for unpack function
    ///
    /// Looks for the pattern: unpack "format" data
    /// Returns (format_node, data_node) tuple
    fn extract_unpack_args(
        &self,
        children: &[PpiNode],
        unpack_pos: usize,
    ) -> Option<(PpiNode, PpiNode)> {
        let mut format_node = None;
        let mut data_node = None;

        // Look for arguments after unpack position
        for child in children.iter().skip(unpack_pos + 1) {
            // Skip comma operators
            if child.class == "PPI::Token::Operator"
                && child.content.as_ref().map_or(false, |c| c == ",")
            {
                continue;
            }

            if self.is_value_node(child) {
                if format_node.is_none() {
                    format_node = Some(child.clone());
                } else if data_node.is_none() {
                    data_node = Some(child.clone());
                    break; // Found both arguments
                }
            }
        }

        match (format_node, data_node) {
            (Some(format), Some(data)) => Some((format, data)),
            _ => None,
        }
    }

    /// Check if a node represents a value (string, variable, number, etc.)
    ///
    /// This excludes operators and structural elements, focusing on actual data.
    fn is_value_node(&self, node: &PpiNode) -> bool {
        match node.class.as_str() {
            "PPI::Token::Quote::Double"
            | "PPI::Token::Quote::Single"
            | "PPI::Token::Symbol"
            | "PPI::Token::Word"
            | "PPI::Token::Number" => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create test PpiNode from JSON-like structure
    fn create_test_node(class: &str, content: Option<&str>) -> PpiNode {
        PpiNode {
            class: class.to_string(),
            content: content.map(|s| s.to_string()),
            children: vec![],
            symbol_type: None,
            numeric_value: None,
            string_value: content.map(|s| s.to_string()),
            structure_bounds: None,
        }
    }

    #[test]
    fn test_join_unpack_pattern_recognition() {
        let pass = JoinUnpackPass;

        // Create AST for: join " ", unpack "H2H2", val
        let statement = PpiNode {
            class: "PPI::Statement".to_string(),
            content: None,
            children: vec![
                create_test_node("PPI::Token::Word", Some("join")),
                create_test_node("PPI::Token::Quote::Double", Some("\" \"")),
                create_test_node("PPI::Token::Operator", Some(",")),
                create_test_node("PPI::Token::Word", Some("unpack")),
                create_test_node("PPI::Token::Quote::Double", Some("\"H2H2\"")),
                create_test_node("PPI::Token::Operator", Some(",")),
                create_test_node("PPI::Token::Word", Some("val")),
            ],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        let result = pass.transform(statement);

        // Should transform to nested function call structure
        assert_eq!(result.class, "FunctionCall");
        assert_eq!(result.content, Some("join".to_string()));
        assert_eq!(result.children.len(), 2);

        // First argument should be separator
        assert_eq!(result.children[0].content, Some("\" \"".to_string()));

        // Second argument should be unpack function call
        assert_eq!(result.children[1].class, "FunctionCall");
        assert_eq!(result.children[1].content, Some("unpack".to_string()));
        assert_eq!(result.children[1].children.len(), 2);
        assert_eq!(
            result.children[1].children[0].content,
            Some("\"H2H2\"".to_string())
        );
        assert_eq!(
            result.children[1].children[1].content,
            Some("val".to_string())
        );
    }

    #[test]
    fn test_join_unpack_without_commas() {
        let pass = JoinUnpackPass;

        // Create AST for: join " " unpack "H2H2" val (no commas)
        let statement = PpiNode {
            class: "PPI::Statement".to_string(),
            content: None,
            children: vec![
                create_test_node("PPI::Token::Word", Some("join")),
                create_test_node("PPI::Token::Quote::Double", Some("\" \"")),
                create_test_node("PPI::Token::Word", Some("unpack")),
                create_test_node("PPI::Token::Quote::Double", Some("\"H2H2\"")),
                create_test_node("PPI::Token::Word", Some("val")),
            ],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        let result = pass.transform(statement);

        // Should still recognize pattern without commas
        assert_eq!(result.class, "FunctionCall");
        assert_eq!(result.content, Some("join".to_string()));
    }

    #[test]
    fn test_non_join_unpack_pattern() {
        let pass = JoinUnpackPass;

        // Create AST that doesn't match pattern
        let statement = PpiNode {
            class: "PPI::Statement".to_string(),
            content: None,
            children: vec![
                create_test_node("PPI::Token::Word", Some("length")),
                create_test_node("PPI::Token::Symbol", Some("$val")),
            ],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        let result = pass.transform(statement);

        // Should return unchanged
        assert_eq!(result.class, "PPI::Statement");
        assert_eq!(result.children.len(), 2);
    }

    #[test]
    fn test_different_separator() {
        let pass = JoinUnpackPass;

        // Create AST for: join ":", unpack "H2*", data
        let statement = PpiNode {
            class: "PPI::Statement".to_string(),
            content: None,
            children: vec![
                create_test_node("PPI::Token::Word", Some("join")),
                create_test_node("PPI::Token::Quote::Double", Some("\":\"")),
                create_test_node("PPI::Token::Operator", Some(",")),
                create_test_node("PPI::Token::Word", Some("unpack")),
                create_test_node("PPI::Token::Quote::Double", Some("\"H2*\"")),
                create_test_node("PPI::Token::Operator", Some(",")),
                create_test_node("PPI::Token::Symbol", Some("$data")),
            ],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        let result = pass.transform(statement);

        // Should handle different separator
        assert_eq!(result.children[0].content, Some("\":\"".to_string()));
        assert_eq!(
            result.children[1].children[0].content,
            Some("\"H2*\"".to_string())
        );
        assert_eq!(
            result.children[1].children[1].content,
            Some("$data".to_string())
        );
    }

    #[test]
    fn test_insufficient_tokens() {
        let pass = JoinUnpackPass;

        // Create AST with too few tokens to match pattern
        let statement = PpiNode {
            class: "PPI::Statement".to_string(),
            content: None,
            children: vec![
                create_test_node("PPI::Token::Word", Some("join")),
                create_test_node("PPI::Token::Quote::Double", Some("\" \"")),
            ],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        let result = pass.transform(statement);

        // Should return unchanged - not enough tokens for pattern
        assert_eq!(result.class, "PPI::Statement");
        assert_eq!(result.children.len(), 2);
    }
}
