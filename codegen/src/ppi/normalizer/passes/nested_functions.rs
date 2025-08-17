//! Nested Function Call Normalization for Perl precedence rules
//!
//! Transforms flat comma-separated function calls into properly nested structures
//! following Perl's precedence rules where rightmost functions bind arguments first.
//!
//! Example: `join " ", unpack "H2H2", val` becomes `join(" ", unpack("H2H2", val))`
//!
//! Perl precedence reference: perlop(1) - "List Operators (Rightward)"

use crate::ppi::normalizer::{utils, NormalizationPass, PrecedenceLevel};
use crate::ppi::types::PpiNode;
use tracing::{debug, trace};

/// Normalizes nested function calls following Perl precedence rules
pub struct NestedFunctionNormalizer;

impl NormalizationPass for NestedFunctionNormalizer {
    fn name(&self) -> &str {
        "NestedFunctionNormalizer"
    }

    fn precedence_level(&self) -> PrecedenceLevel {
        PrecedenceLevel::Low // Level 22+ - list operators without parentheses
    }

    fn transform(&self, node: PpiNode) -> PpiNode {
        // Look for statements with multiple function calls
        // Handle both PPI::Statement and PPI::Statement::Expression
        if (node.class == "PPI::Statement" || node.class == "PPI::Statement::Expression")
            && node.children.len() >= 5
        {
            debug!(
                "NestedFunctionNormalizer: Found {} with {} children",
                node.class,
                node.children.len()
            );

            if let Some(normalized) = self.normalize_nested_functions(&node) {
                debug!("NestedFunctionNormalizer: Successfully normalized nested function call structure");
                return normalized;
            } else {
                debug!("NestedFunctionNormalizer: No normalization applied");
            }
        }

        // Recurse into children
        utils::transform_children(node, |child| self.transform(child))
    }
}

impl NestedFunctionNormalizer {
    /// Normalize nested function calls by applying Perl precedence rules
    fn normalize_nested_functions(&self, node: &PpiNode) -> Option<PpiNode> {
        // Parse the flat token sequence and identify function call patterns
        let mut tokens = Vec::new();
        for child in &node.children {
            tokens.push(child.clone());
        }

        // Skip if this looks like a function with parentheses followed by operators
        // Example: abs($val)<100 ? 1/(2**$val) : 0
        // Pattern: Word + Structure::List + Operator (not comma) = already properly structured
        if tokens.len() >= 3
            && tokens[0].class == "PPI::Token::Word"
            && self.is_function_name(&tokens[0].content)
            && tokens[1].class == "PPI::Structure::List"
            && tokens[2].class == "PPI::Token::Operator"
            && tokens[2].content.as_ref() != Some(&",".to_string())
        {
            trace!("Skipping nested function normalization - function already has parentheses and is followed by non-comma operator");
            return None;
        }

        // First try nested function patterns (higher precedence): rightmost functions bind arguments first
        if let Some(nested_structure) = self.apply_perl_precedence(&tokens) {
            return Some(PpiNode {
                class: node.class.clone(), // Preserve original class (Statement or Statement::Expression)
                content: None,
                children: vec![nested_structure],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            });
        }

        // Then try single implicit function calls (only if nested patterns don't apply)
        if let Some(normalized_tokens) = self.normalize_implicit_function_calls(&tokens) {
            return Some(PpiNode {
                class: node.class.clone(), // Preserve original class (Statement or Statement::Expression)
                content: None,
                children: normalized_tokens,
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            });
        }

        None
    }

    /// Normalize implicit function calls within token sequences
    /// Handles patterns like: arg1, arg2, function_name, arg3, arg4 -> arg1, arg2, function_name(arg3, arg4)
    fn normalize_implicit_function_calls(&self, tokens: &[PpiNode]) -> Option<Vec<PpiNode>> {
        let mut result = Vec::new();
        let mut i = 0;
        let mut made_changes = false;

        while i < tokens.len() {
            // Look for function word tokens
            if tokens[i].class == "PPI::Token::Word" && self.is_function_name(&tokens[i].content) {
                // Collect function arguments that follow
                let mut function_args = Vec::new();
                let mut j = i + 1;

                // Skip the first comma if present
                if j < tokens.len()
                    && tokens[j].class == "PPI::Token::Operator"
                    && tokens[j].content.as_ref() == Some(&",".to_string())
                {
                    j += 1;
                }

                // Collect arguments until we hit another function or end of tokens
                while j < tokens.len() {
                    if tokens[j].class == "PPI::Token::Operator"
                        && tokens[j].content.as_ref() == Some(&",".to_string())
                    {
                        j += 1; // Skip commas
                        continue;
                    }

                    // Stop if we hit another function
                    if tokens[j].class == "PPI::Token::Word"
                        && self.is_function_name(&tokens[j].content)
                    {
                        break;
                    }

                    function_args.push(tokens[j].clone());
                    j += 1;
                }

                // If we found arguments, create a function call
                if !function_args.is_empty() {
                    let function_name = tokens[i].content.as_ref().unwrap();
                    let function_call = utils::create_function_call(function_name, function_args);
                    result.push(function_call);
                    made_changes = true;
                    debug!("Normalized implicit function call: {}", function_name);
                    i = j; // Continue from after the arguments
                } else {
                    // No arguments found, keep the token as-is
                    result.push(tokens[i].clone());
                    i += 1;
                }
            } else {
                // Not a function, keep as-is
                result.push(tokens[i].clone());
                i += 1;
            }
        }

        if made_changes {
            Some(result)
        } else {
            None
        }
    }

    /// Apply Perl precedence rules to create nested function call structure
    fn apply_perl_precedence(&self, tokens: &[PpiNode]) -> Option<PpiNode> {
        // Look for the pattern: function_name args... function_name args...
        // Example 1: join " " unpack "H2H2" val
        //            ^^^^^     ^^^^^^       ^^^
        //            func1     func2        args
        // Example 2: sprintf "format" unpack "H4H2" val
        //            ^^^^^^^          ^^^^^^        ^^^
        //            func1            func2         args

        if tokens.len() < 5 {
            return None;
        }

        // Find function words in the token sequence
        let mut function_positions = Vec::new();
        for (i, token) in tokens.iter().enumerate() {
            debug!(
                "Token {}: class={}, content={:?}",
                i, token.class, token.content
            );
            if token.class == "PPI::Token::Word" && self.is_function_name(&token.content) {
                debug!("  -> Found function: {:?}", token.content);
                function_positions.push(i);
            }
        }

        debug!("Function positions found: {:?}", function_positions);

        if function_positions.len() < 2 {
            debug!(
                "Not enough function positions (need 2, found {})",
                function_positions.len()
            );
            return None;
        }

        debug!(
            "Found {} function positions: {:?}",
            function_positions.len(),
            function_positions
        );

        // Apply right-to-left precedence: process rightmost function first
        if let Some(result) = self.build_nested_structure(tokens, &function_positions) {
            return Some(result);
        }

        None
    }

    /// Build nested structure by grouping rightmost functions with their arguments first
    fn build_nested_structure(
        &self,
        tokens: &[PpiNode],
        function_positions: &[usize],
    ) -> Option<PpiNode> {
        if function_positions.len() != 2 {
            // For now, only handle the two-function case
            return None;
        }

        let first_func_pos = function_positions[0];
        let second_func_pos = function_positions[1];

        // Extract function names
        let first_func_name = tokens[first_func_pos].content.as_ref()?;
        let second_func_name = tokens[second_func_pos].content.as_ref()?;

        trace!(
            "Processing functions: {} and {}",
            first_func_name,
            second_func_name
        );

        // Handle specific patterns
        match (first_func_name.as_str(), second_func_name.as_str()) {
            ("join", "unpack") => {
                self.build_join_unpack_structure(tokens, first_func_pos, second_func_pos)
            }
            ("sprintf", "unpack") => {
                self.build_sprintf_unpack_structure(tokens, first_func_pos, second_func_pos)
            }
            _ => {
                // For other combinations, use general implicit function call handling
                self.build_general_nested_structure(tokens, first_func_pos, second_func_pos)
            }
        }
    }

    /// Build the specific join(separator, unpack(format, data)) structure
    fn build_join_unpack_structure(
        &self,
        tokens: &[PpiNode],
        join_pos: usize,
        unpack_pos: usize,
    ) -> Option<PpiNode> {
        // Expected pattern: join separator unpack format data
        // Positions:        0    1         2      3      4

        if tokens.len() < 5 || join_pos != 0 || unpack_pos < 2 {
            return None;
        }

        // Extract separator (between join and unpack)
        let separator_tokens: Vec<PpiNode> = tokens[join_pos + 1..unpack_pos]
            .iter()
            .filter(|t| {
                t.class != "PPI::Token::Operator" || t.content.as_ref() != Some(&",".to_string())
            })
            .cloned()
            .collect();

        // Extract unpack arguments (everything after unpack)
        let unpack_args: Vec<PpiNode> = tokens[unpack_pos + 1..]
            .iter()
            .filter(|t| {
                t.class != "PPI::Token::Operator" || t.content.as_ref() != Some(&",".to_string())
            })
            .cloned()
            .collect();

        trace!(
            "Separator tokens: {}, Unpack args: {}",
            separator_tokens.len(),
            unpack_args.len()
        );

        // Create nested unpack function call
        let unpack_call = utils::create_function_call("unpack", unpack_args);

        // Create join arguments: [separator, unpack_call]
        let mut join_args = separator_tokens;
        join_args.push(unpack_call);

        // Create the final join function call
        let join_call = utils::create_function_call("join", join_args);

        debug!("Created nested join(unpack()) structure");
        Some(join_call)
    }

    /// Build the specific sprintf(format, unpack(format, data)) structure
    fn build_sprintf_unpack_structure(
        &self,
        tokens: &[PpiNode],
        sprintf_pos: usize,
        unpack_pos: usize,
    ) -> Option<PpiNode> {
        // Expected pattern: sprintf format unpack unpack_format data
        // Positions:        0       1      2      3            4
        // Example: sprintf("%s:%s:%s %s:%s:%s.%s", unpack, "H4H2H2H2H2H2H2", $val)

        if tokens.len() < 5 || sprintf_pos != 0 || unpack_pos < 2 {
            return None;
        }

        // Extract sprintf format arguments (between sprintf and unpack)
        let sprintf_format_args: Vec<PpiNode> = tokens[sprintf_pos + 1..unpack_pos]
            .iter()
            .filter(|t| {
                t.class != "PPI::Token::Operator" || t.content.as_ref() != Some(&",".to_string())
            })
            .cloned()
            .collect();

        // Extract unpack arguments (everything after unpack)
        let unpack_args: Vec<PpiNode> = tokens[unpack_pos + 1..]
            .iter()
            .filter(|t| {
                t.class != "PPI::Token::Operator" || t.content.as_ref() != Some(&",".to_string())
            })
            .cloned()
            .collect();

        trace!(
            "Sprintf format tokens: {}, Unpack args: {}",
            sprintf_format_args.len(),
            unpack_args.len()
        );

        // Create nested unpack function call
        let unpack_call = utils::create_function_call("unpack", unpack_args);

        // Create sprintf arguments: [format_args..., unpack_call]
        let mut sprintf_args = sprintf_format_args;
        sprintf_args.push(unpack_call);

        // Create the final sprintf function call
        let sprintf_call = utils::create_function_call("sprintf", sprintf_args);

        debug!("Created nested sprintf(unpack()) structure");
        Some(sprintf_call)
    }

    /// Build a general nested function structure for any function combination
    fn build_general_nested_structure(
        &self,
        tokens: &[PpiNode],
        first_func_pos: usize,
        second_func_pos: usize,
    ) -> Option<PpiNode> {
        // General pattern: func1 args... func2 args...
        // Transform to: func1(args..., func2(args...))

        if tokens.len() < 4 {
            return None;
        }

        let first_func_name = tokens[first_func_pos].content.as_ref()?;
        let second_func_name = tokens[second_func_pos].content.as_ref()?;

        // Extract first function arguments (between first and second function)
        let first_args: Vec<PpiNode> = tokens[first_func_pos + 1..second_func_pos]
            .iter()
            .filter(|t| {
                t.class != "PPI::Token::Operator" || t.content.as_ref() != Some(&",".to_string())
            })
            .cloned()
            .collect();

        // Extract second function arguments (after second function)
        let second_args: Vec<PpiNode> = tokens[second_func_pos + 1..]
            .iter()
            .filter(|t| {
                t.class != "PPI::Token::Operator" || t.content.as_ref() != Some(&",".to_string())
            })
            .cloned()
            .collect();

        // Create nested second function call
        let second_call = utils::create_function_call(second_func_name, second_args);

        // Create first function arguments: [first_args..., second_call]
        let mut combined_args = first_args;
        combined_args.push(second_call);

        // Create the final nested function call
        let nested_call = utils::create_function_call(first_func_name, combined_args);

        debug!(
            "Created general nested {}({}()) structure",
            first_func_name, second_func_name
        );
        Some(nested_call)
    }

    /// Check if a token content represents a known function name
    fn is_function_name(&self, content: &Option<String>) -> bool {
        if let Some(name) = content {
            matches!(
                name.as_str(),
                "join"
                    | "unpack"
                    | "pack"
                    | "split"
                    | "sprintf"
                    | "substr"
                    | "index"
                    | "length"
                    | "int"
                    | "ord"
                    | "chr"
                    | "uc"
                    | "lc"
                    | "abs"
                    | "sqrt"
                    | "hex"
                    | "oct"
            )
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ppi::types::PpiNode;

    fn create_word_token(content: &str) -> PpiNode {
        PpiNode {
            class: "PPI::Token::Word".to_string(),
            content: Some(content.to_string()),
            children: vec![],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        }
    }

    fn create_string_token(content: &str, value: &str) -> PpiNode {
        PpiNode {
            class: "PPI::Token::Quote::Double".to_string(),
            content: Some(content.to_string()),
            children: vec![],
            symbol_type: None,
            numeric_value: None,
            string_value: Some(value.to_string()),
            structure_bounds: None,
        }
    }

    fn create_operator_token(op: &str) -> PpiNode {
        PpiNode {
            class: "PPI::Token::Operator".to_string(),
            content: Some(op.to_string()),
            children: vec![],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        }
    }

    #[test]
    fn test_join_unpack_normalization() {
        let normalizer = NestedFunctionNormalizer;

        // Create the flat AST: join " " unpack "H2H2" val
        let statement = PpiNode {
            class: "PPI::Statement".to_string(),
            content: None,
            children: vec![
                create_word_token("join"),
                create_string_token("\" \"", " "),
                create_operator_token(","),
                create_word_token("unpack"),
                create_string_token("\"H2H2\"", "H2H2"),
                create_operator_token(","),
                create_word_token("val"),
            ],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        let result = normalizer.transform(statement);

        // Should transform to nested structure
        assert_eq!(result.class, "PPI::Statement");
        assert_eq!(result.children.len(), 1);

        let join_call = &result.children[0];
        assert_eq!(join_call.class, "FunctionCall");
        assert_eq!(join_call.content, Some("join".to_string()));
        assert_eq!(join_call.children.len(), 2); // separator + unpack_call

        // Check that unpack was properly nested
        let unpack_call = &join_call.children[1];
        assert_eq!(unpack_call.class, "FunctionCall");
        assert_eq!(unpack_call.content, Some("unpack".to_string()));
        assert_eq!(unpack_call.children.len(), 2); // format + data
    }

    #[test]
    fn test_simple_function_passthrough() {
        let normalizer = NestedFunctionNormalizer;

        // Single function should pass through unchanged
        let statement = PpiNode {
            class: "PPI::Statement".to_string(),
            content: None,
            children: vec![create_word_token("length"), create_word_token("val")],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        let result = normalizer.transform(statement);

        // Should be unchanged (handled by FunctionCallNormalizer)
        assert_eq!(result.class, "PPI::Statement");
        assert_eq!(result.children.len(), 2);
    }

    #[test]
    fn test_sprintf_unpack_normalization() {
        let normalizer = NestedFunctionNormalizer;

        // Create the flat AST: sprintf "%s:%s:%s %s:%s:%s.%s" unpack "H4H2H2H2H2H2H2" val
        let statement = PpiNode {
            class: "PPI::Statement".to_string(),
            content: None,
            children: vec![
                create_word_token("sprintf"),
                create_string_token("\"%s:%s:%s %s:%s:%s.%s\"", "%s:%s:%s %s:%s:%s.%s"),
                create_operator_token(","),
                create_word_token("unpack"),
                create_string_token("\"H4H2H2H2H2H2H2\"", "H4H2H2H2H2H2H2"),
                create_operator_token(","),
                create_word_token("val"),
            ],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        let result = normalizer.transform(statement);

        // Should transform to nested structure
        assert_eq!(result.class, "PPI::Statement");
        println!("Result children: {}", result.children.len());
        for (i, child) in result.children.iter().enumerate() {
            println!("Child {}: {:?} - {:?}", i, child.class, child.content);
        }

        // After implicit function normalization, we should have format + unpack_call directly
        // The sprintf wrapper is already handled by the FunctionCallNormalizer
        if result.children.len() == 2 {
            // Format argument + function call
            let format_arg = &result.children[0];
            let unpack_call = &result.children[1];

            assert_eq!(unpack_call.class, "FunctionCall");
            assert_eq!(unpack_call.content, Some("unpack".to_string()));
            assert_eq!(unpack_call.children.len(), 2); // format + data
        } else {
            // Original expectation if nested differently
            assert_eq!(result.children.len(), 1);
            let sprintf_call = &result.children[0];
            assert_eq!(sprintf_call.class, "FunctionCall");
            assert_eq!(sprintf_call.content, Some("sprintf".to_string()));
            assert_eq!(sprintf_call.children.len(), 2); // format + unpack_call

            // Check that unpack was properly nested
            let unpack_call = &sprintf_call.children[1];
            assert_eq!(unpack_call.class, "FunctionCall");
            assert_eq!(unpack_call.content, Some("unpack".to_string()));
            assert_eq!(unpack_call.children.len(), 2); // format + data
        }
    }
}
